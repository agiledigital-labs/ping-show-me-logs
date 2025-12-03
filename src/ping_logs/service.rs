use crate::AppMutState;
use crate::errors::ShowMeErrors;
use crate::ping_logs::logs::{Level, Logs, get_logs};
use actix_web::web::Query;
use actix_web::{Responder, get, post, web};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tantivy::collector::TopDocs;
use tantivy::query::FuzzyTermQuery;
use tantivy::{Document, TantivyDocument, Term, doc};

#[derive(Debug, Deserialize, Clone)]
struct WatchFr {
  fr_id: String,
}

#[derive(Debug, Deserialize, Clone)]
enum Filters {
  All,
  Warn,
  Error,
  Debug,
  Default,
}

#[derive(Deserialize)]
struct ScriptLogs {
  fr_id: String,
  script_id: String,
}

#[get("/transaction/ids")]
async fn list_transaction_ids(
  data: web::Data<AppMutState>,
) -> Result<web::Json<Vec<String>>, crate::errors::ShowMeErrors> {
  let rolling_list: Vec<String> = data
    .rolling_id_list
    .lock()
    .map_err(|_| ShowMeErrors::IdLockError("failed".to_string()))?
    .iter()
    .cloned()
    .collect();

  dbg!(&rolling_list);

  Ok(web::Json(rolling_list))
}

#[get("/transaction/demo/search")]
async fn searcher_poc(data: web::Data<AppMutState>) -> Result<web::Json<Value>, ShowMeErrors> {
  let sercher = data.reader.searcher();

  let quere = FuzzyTermQuery::new(
    Term::from_field_text(data.schema.get_field("transactionId")?, ""),
    2,
    true,
  );

  let docs = sercher
    .search(&quere, &TopDocs::with_limit(25))?
    .iter()
    .map(|(_, t)| {
      println!("{:?}", &t);
      sercher
        .doc::<TantivyDocument>(*t)
        .unwrap()
        .to_json(&data.schema)
    })
    .collect::<Vec<String>>();

  dbg!(docs);

  let test: Value = serde_json::from_str("[]")?;

  Ok(web::Json(test))
}

#[get("/{fr_id}/script/{script_id}")]
async fn script_logs(
  path: web::Path<ScriptLogs>,
  query: Query<LogsRequest>,
  data: web::Data<AppMutState>,
) -> Result<web::Json<Logs>, ShowMeErrors> {
  let formatted_query = format!(
    "/payload/logger sw \"scripts.AUTHENTICATION_TREE_DECISION_NODE.{}\"",
    path.script_id
  );

  let query_filter = Some(formatted_query.as_str());

  match get_logs(&Client::new(), &data, &path.fr_id, query_filter).await {
    Ok(ll) => Ok(match query.filters.clone() {
      None => web::Json(ll),
      Some(filter) => match filter {
        Filters::Warn => web::Json(ll.filter_logs(Level::Warning)),
        Filters::Error => web::Json(ll.filter_logs(Level::Error)),
        Filters::Debug => web::Json(ll.filter_logs(Level::Debug)),
        _ => web::Json(ll),
      },
    }),
    Err(err) => {
      println!("{}", err);
      Err(ShowMeErrors::NoLogsFound(path.fr_id.clone()))
    }
  }
}

#[derive(Debug, Deserialize, Clone)]
struct LogsRequest {
  filters: Option<Filters>,
  script_id: Option<String>,
  script_name: Option<String>,
}

#[get("/{fr_id}")]
async fn logs(
  fr_id: web::Path<String>,
  query: Query<LogsRequest>,
  data: web::Data<AppMutState>,
) -> Result<web::Json<Logs>, ShowMeErrors> {
  let id = fr_id.into_inner();

  let script_filter = query.script_id.clone().map(|script_id| {
    format!(
      "/payload/logger sw \"scripts.AUTHENTICATION_TREE_DECISION_NODE.{}\"",
      script_id
    )
  });

  let node_filter = query
    .script_name
    .clone()
    .map(|script_name| format!("/payload/entries/info/displayName eq \"{}\"", script_name));

  let defined_filters: Vec<String> = vec![node_filter, script_filter]
    .iter()
    .filter(|maybe_filter| maybe_filter.is_some())
    .map(|filter| filter.as_ref().cloned().unwrap())
    .collect();

  let query_filter = defined_filters.clone().join(" or ");

  match get_logs(&Client::new(), &data, &id, Some(query_filter.as_str())).await {
    Ok(ll) => Ok(match query.filters.clone() {
      None => web::Json(ll),
      Some(filter) => match filter {
        Filters::Warn => web::Json(ll.filter_logs(Level::Warning)),
        Filters::Error => web::Json(ll.filter_logs(Level::Error)),
        Filters::Debug => web::Json(ll.filter_logs(Level::Debug)),
        _ => web::Json(ll),
      },
    }),
    Err(err) => {
      println!("{}", err);
      Err(ShowMeErrors::NoLogsFound(id))
    }
  }
}

#[get("/watch")]
async fn get_watch(
  data: web::Data<AppMutState>,
  query: Query<LogsRequest>,
) -> Result<web::Json<Logs>, ShowMeErrors> {
  Ok(match data.transaction_id.lock() {
    Ok(id) => match get_logs(&Client::new(), &data, &*id, None).await {
      Ok(ll) => match query.filters.clone() {
        None => web::Json(ll),
        Some(filter) => match filter {
          Filters::Warn => web::Json(ll.filter_logs(Level::Warning)),
          Filters::Error => web::Json(ll.filter_logs(Level::Error)),
          Filters::Debug => web::Json(ll.filter_logs(Level::Debug)),
          _ => web::Json(ll),
        },
      },
      Err(err) => {
        println!("{}", err);
        web::Json(Logs::default())
      }
    },
    _ => web::Json(Logs::default()),
  })
}

#[post("/watch")]
async fn set_watch(
  data: web::Data<AppMutState>,
  payload: web::Json<WatchFr>,
) -> Result<impl Responder, ShowMeErrors> {
  let mut id = data
    .transaction_id
    .lock()
    .map_err(|_| ShowMeErrors::SharedLocking("transaction_id".into()))?;
  *id = payload.fr_id.clone();

  drop(id);

  Ok("success")
}

// this function could be located in a different module
pub fn log_api(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/logs")
      .service(script_logs)
      .service(logs)
      .service(get_watch)
      .service(logs)
      .service(list_transaction_ids)
      .service(set_watch),
  );
}
