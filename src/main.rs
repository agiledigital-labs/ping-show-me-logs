use crate::errors::ShowMeErrors;
use crate::front::{am, idm, index};
use crate::ping_logs::logs::{GenericLog, tail_logs};
use crate::ping_logs::service::log_api;
use crate::token::{Token, get_usable_token};
use crate::trees::journeys::AuthenticationTreeList;
use crate::trees::service::trees_api;
use crate::workers::scripts::{ScriptConfig, list_scripts};
use actix_web::rt::time::sleep;
use actix_web::{App, HttpServer, Responder, rt, web};
use futures_util::StreamExt as _;
use reqwest::Client;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::Duration;
use tantivy::schema::{STORED, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, doc};
use tempfile::TempDir;

mod errors;
mod front;
mod ping_logs;
mod token;
mod trees;
mod watcher;
mod workers;

const MAX_SIZE: usize = 50;

fn add_to_rolling_buffer(deque: &Mutex<VecDeque<String>>, value: String) -> Option<()> {
  match deque
    .lock()
    .map_err(|_| ShowMeErrors::IdLockError("Failed to lock id vec".to_string()))
  {
    Ok(mut locked_queue) => {
      if !locked_queue.contains(&value) {
        if locked_queue.len() == MAX_SIZE {
          locked_queue.pop_front();
        }
        println!("{}, {}", locked_queue.len(), &value);
        locked_queue.push_back(value);
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
  reader: IndexReader,
  schema: Schema,
  rolling_id_list: Mutex<VecDeque<String>>,
}

// this could be done with rust embed
#[derive(Debug, Default)]
struct NodeOutcomeEdge {
  name: String,
  outcome: String,
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

  let index_path = TempDir::new()?;
  let mut schema_builder = Schema::builder();

  schema_builder.add_text_field("transactionId", TEXT | STORED);

  let schema = schema_builder.build();
  let search_index = Index::create_in_dir(&index_path, schema.clone())?;
  let transaction_id_schema = schema.get_field("transactionId")?;

  let reader = search_index
    .reader_builder()
    .reload_policy(ReloadPolicy::OnCommitWithDelay)
    .try_into()?;

  let state = web::Data::new(AppMutState {
    transaction_id: Mutex::new(String::new()),
    authentication_tree,
    token,
    token_str: token_mux,
    payload: Mutex::new(payload_up),
    sec,
    key,
    log: url,
    script_config: Mutex::new(HashMap::new()),
    reader,
    schema,
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

      // Otherwise this lock would only go out of scope when the sleep endds.
      drop(sct);

      sleep(Duration::from_secs(300)).await;
    }
    Ok::<(), ShowMeErrors>(())
  });

  let tail_logs_data = state.clone();
  rt::spawn(async move {
    let client = Client::new();
    let mut cookie: Option<String> = None;
    let mut index_writer = search_index.writer(50_000_000)?;
    loop {
      let logs = tail_logs(&client, &tail_logs_data, cookie, None).await?;

      cookie = logs.paged_result_cooke;

      println!("{:?}", cookie);
      let docs = logs
        .result
        .iter()
        .map(|t| {
          let res = match t {
            GenericLog::ResultingLog(t) => {
              index_writer.add_document(
                doc!(transaction_id_schema => t.payload.transaction_id.split_at(36).0),
              )?;
              let id = t.payload.transaction_id.split_at(36).0.to_string();
              let is_new = add_to_rolling_buffer(&tail_logs_data.rolling_id_list, id.clone());
              if is_new.is_some() { Some(id) } else { None }
            }
            _t => None,
          };
          Ok(res)
        })
        .collect::<Result<Vec<Option<String>>, ShowMeErrors>>()?;
      let stmp = index_writer.commit()?;
      sleep(Duration::from_secs(2)).await;
    }
    Ok::<(), ShowMeErrors>(())
  });

  HttpServer::new(move || {
    let cors = actix_cors::Cors::permissive().allow_any_header();
    App::new()
      .app_data(state.clone())
      .wrap(cors)
      .service(
        web::scope("/api")
          .configure(trees_api)
          .configure(log_api)
          .service(web::scope("/monitoring").service(am).service(idm)),
      )
      .route("/{filename:.*}", web::get().to(index))
  })
  .bind(("0.0.0.0", 8081))?
  .run()
  .await?;
  Ok(())
}
