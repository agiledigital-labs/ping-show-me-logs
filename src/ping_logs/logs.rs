use crate::AppMutState;
use crate::errors::ShowMeErrors;
use actix_web::web::Data;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Level {
  Debug,
  Warning,
  Warn,
  Info,
  Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeOutcomeInfo {
  node_extra_logging: Option<serde_json::Map<String, serde_json::Value>>,
  node_id: String,
  pub(crate) node_outcome: String,
  pub(crate) display_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NodeOutcome {
  pub(crate) info: NodeOutcomeInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PingPayload {
  context: Option<String>,
  level: Level,
  pub(crate) entries: Option<Vec<NodeOutcome>>,
  logger: Option<String>,
  message: Option<String>,
  pub(crate) transaction_id: String,
  pub(crate) tracking_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResultingLog {
  pub(crate) payload: PingPayload,
  pub(crate) timestamp: DateTime<Utc>,
  #[serde(rename = "type")]
  data_type: String,
  source: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GenericLog {
  ResultingLog(ResultingLog),
  Other(Value),
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Logs {
  pub(crate) result: Vec<GenericLog>,
  pub paged_result_cooke: Option<String>,
  result_count: Option<i32>,
  total_paged_results_policy: String,
  total_paged_results: i16,
  remaining_paged_results: i16,
}

impl Logs {
  pub fn filter_logs(self, level: Level) -> Logs {
    let result = self
      .result
      .iter()
      .filter(|t| match t {
        GenericLog::ResultingLog(tt) => tt.payload.level == level,
        GenericLog::Other(_) => false,
      })
      .cloned()
      .collect::<Vec<_>>()
      .clone();
    Logs { result, ..self }
  }
}

pub(crate) async fn tail_logs(
  client: &Client,
  app_state: &Data<AppMutState>,
  tail_header: Option<String>,
  query_filter: Option<&str>,
) -> Result<Logs, ShowMeErrors> {
  let params = [
    ("source", "am-everything,idm-everything"),
    ("_queryFilter", query_filter.unwrap_or_else(|| "")),
    ("_pagedResultCookie", &tail_header.unwrap_or("".to_string())),
  ];

  let url = &app_state.log;
  let sec = &app_state.sec;
  let key = &app_state.key;

  let text = client
    .get(format!("{url}/tail"))
    .query(&params)
    .header("x-api-key", key)
    .header("x-api-secret", sec)
    .send()
    .await?
    .text()
    .await?;

  let logs: Logs = serde_json::from_str(&text)?;

  Ok(logs)
}

pub(crate) async fn get_logs(
  client: &Client,
  app_state: &Data<AppMutState>,
  transaction_id: &str,
  query_filter: Option<&str>,
) -> Result<Logs, ShowMeErrors> {
  let params = [
    ("source", "am-everything,idm-everything"),
    ("transactionId", transaction_id),
    ("_queryFilter", query_filter.unwrap_or_else(|| "")),
  ];

  let url = &app_state.log;
  let sec = &app_state.sec;
  let key = &app_state.key;

  match client
    .get(url)
    .query(&params)
    .header("x-api-key", key)
    .header("x-api-secret", sec)
    .send()
    .await
  {
    Ok(res) => match res.bytes().await {
      Ok(bty) => Ok(serde_json::from_slice(&bty)?),
      Err(e) => {
        Err(ShowMeErrors::PingApiError(/* reqwest::Error */ e))
      }
    },
    Err(e) => {
      Err(ShowMeErrors::PingApiError(/* reqwest::Error */ e))
    }
  }
}
