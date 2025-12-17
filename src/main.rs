use crate::errors::ShowMeErrors;
use crate::front::{am, idm, index};
use crate::log_watcher::LogWatcher;
use crate::ping_logs::service::log_api;
use crate::token::{Token, get_usable_token};
use crate::trees::journeys::AuthenticationTreeList;
use crate::trees::service::trees_api;
use crate::workers::scripts::{ScriptConfig, list_scripts};
use crate::ws_server::{LogsServer, LogsServerHandle};
use actix_web::rt::time::sleep;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, rt, web};
use futures_util::StreamExt as _;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::Duration;
use tantivy::doc;
use tokio::{spawn, try_join};

mod errors;
mod front;
mod log_watcher;
mod ping_logs;
mod token;
mod trees;
mod watcher;
mod workers;
mod ws_handler;
mod ws_server;

const MAX_SIZE: usize = 50;
/// Connection ID.
pub type ConnId = u64;

/// Room ID.
pub type JourneyId = String;
pub type TransactionId = String;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct TransactionIdWs {
  id: TransactionId,
  journey: JourneyId,
}

impl TransactionIdWs {
  pub fn from_tuple((id, journey): (String, String)) -> Self {
    Self { id, journey }
  }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum WsMsg {
  String(String),
  TransactionIdWs(TransactionIdWs),
}

impl From<&str> for WsMsg {
  fn from(value: &str) -> Self {
    Self::String(value.to_string())
  }
}

impl From<String> for WsMsg {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}
impl From<(TransactionId, JourneyId)> for WsMsg {
  fn from(value: (TransactionId, JourneyId)) -> Self {
    Self::TransactionIdWs(TransactionIdWs::from_tuple(value))
  }
}

/// Message sent to a room/client.
pub type Msg = WsMsg;

fn add_to_rolling_buffer(
  deque: &Mutex<VecDeque<TransactionIdWs>>,
  value: impl Into<TransactionIdWs>,
) -> Option<()> {
  match deque
    .lock()
    .map_err(|_| ShowMeErrors::IdLockError("Failed to lock id vec".to_string()))
  {
    Ok(mut locked_queue) => {
      let local_value = value.into();
      if !locked_queue.contains(&local_value) {
        if locked_queue.len() == MAX_SIZE {
          locked_queue.pop_front();
        }
        println!("Transaction queue length: {}.", locked_queue.len());
        locked_queue.push_back(local_value);
        Some(())
      } else {
        None
      }
    }
    Err(_) => None,
  }
}

struct AppMutState {
  transaction_id: Mutex<String>,
  authentication_tree: AuthenticationTreeList,
  token: Token,
  token_str: Mutex<String>,
  payload: Mutex<token::Payload>,
  sec: String,
  key: String,
  log: String,
  script_config: Mutex<HashMap<String, ScriptConfig>>,
  rolling_id_list: Mutex<VecDeque<TransactionIdWs>>,
}

// this could be done with rust embed
#[derive(Debug, Default)]
struct NodeOutcomeEdge {
  name: String,
  outcome: String,
}

/// Handshake and start the websocket handler with heartbeats.
async fn ws_server(
  req: HttpRequest,
  stream: web::Payload,
  server: web::Data<LogsServerHandle>,
) -> Result<HttpResponse, ShowMeErrors> {
  let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

  // spawn websocket handler (and don't await it) so that the response is returned immediately
  tokio::task::spawn_local(ws_handler::chat_ws((**server).clone(), session, msg_stream));

  Ok(res)
}

#[actix_web::main]
async fn main() -> Result<(), ShowMeErrors> {
  let (token, payload_init) = Token::new().await?;
  let payload_mux_init = Mutex::new(payload_init);

  let client = Client::new();
  let token_mux = Mutex::new("".to_string());

  // ToDo - If this was a Arc<Mutex> it would not need the be &mut
  let (token_str, payload_up) = get_usable_token(&token, &payload_mux_init, &token_mux).await?;

  let authentication_tree: AuthenticationTreeList = serde_json::from_slice(&client.get(format!("{}/am/json/realms/root/realms/alpha/realm-config/authentication/authenticationtrees/trees?_queryFilter=true", token.dom, )).header("authorization", format!("Bearer {}", token_str)).send().await?.bytes().await?)?;

  let url = std::env::var("LOGS_ENDPOINT")?;
  let key = std::env::var("PING_KEY")?;
  let sec = std::env::var("PING_SEC")?;

  let state = web::Data::new(AppMutState {
    transaction_id: Mutex::new(String::new()),
    authentication_tree: authentication_tree.clone(),
    token,
    token_str: token_mux,
    payload: Mutex::new(payload_up),
    sec,
    key,
    log: url,
    script_config: Mutex::new(HashMap::new()),
    rolling_id_list: Mutex::new(VecDeque::new()),
  });

  let data = state.clone();
  rt::spawn(async move {
    let client = Client::new();
    loop {
      let (token_str, payload) =
        get_usable_token(&data.token, &data.payload, &data.token_str).await?;
      let scripts = list_scripts(&client, &data.token.dom, &token_str).await?;

      let mut sct = data
        .script_config
        .lock()
        .map_err(|_| ShowMeErrors::SharedLocking("script list".into()))?;

      *sct = scripts;

      // Otherwise this lock would only go out of scope when the sleep ends.
      drop(sct);

      sleep(Duration::from_secs(300)).await;
    }
    Ok::<(), ShowMeErrors>(())
  });

  let (logs_server, server_tx) = LogsServer::new(authentication_tree);
  let watcher_state = state.clone();
  let (watcher, reader, schema) = LogWatcher::new(server_tx.clone(), watcher_state)?;
  let log_command_server = spawn(logs_server.run());

  let log_watcher = spawn(watcher.watch());

  let http_server = HttpServer::new(move || {
    let cors = actix_cors::Cors::permissive().allow_any_header();
    App::new()
      .app_data(state.clone())
      .app_data(web::Data::new(server_tx.clone()))
      .app_data(web::Data::new((reader.clone(), schema.clone())))
      .wrap(cors)
      .service(
        web::scope("/api")
          .configure(trees_api)
          .configure(log_api)
          .service(web::resource("/ws").route(web::get().to(ws_server)))
          .service(web::scope("/monitoring").service(am).service(idm)),
      )
      .route("/{filename:.*}", web::get().to(index))
  })
  .bind(("0.0.0.0", 8081))?
  .run();

  try_join!(
    http_server,
    async move { log_watcher.await.unwrap() },
    async move { log_command_server.await.unwrap() }
  )?;
  Ok(())
}
