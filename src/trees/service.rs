use crate::errors::ShowMeErrors;
use crate::ping_logs::logs::{GenericLog, get_logs};
use crate::token::get_usable_token;
use crate::trees::journeys::{ReactFlowEdge, ReactFlowNode};
use crate::trees::nodes::{NodeConfig, NodeData};
use crate::workers::scripts::ScriptConfig;
use crate::{AppMutState, NodeOutcomeEdge};
use actix_web::web::Query;
use actix_web::{get, web};
use chrono::{DateTime, Utc};
use futures_util::future;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Deserialize)]
struct JourneyFlowQuery {
  transaction_id: Option<String>,
}

#[get("/{name}/flow")]
async fn journey_flow(
  name: web::Path<String>,
  query: Query<JourneyFlowQuery>,
  data: web::Data<AppMutState>,
) -> Result<web::Json<FlowPayload>, ShowMeErrors> {
  let transaction_id = &query.transaction_id;
  let tree = data.authentication_tree.get_tree(&name);

  let node_outcomes = (match transaction_id {
    Some(id) => get_node_outcomes(id, data).await,
    None => Ok(vec![]),
  })
  .unwrap_or(vec![]);

  match tree {
    None => Err(ShowMeErrors::NoLogsFound(
      "ToDo: Make a real error".to_string(),
    )),
    Some(tree_jouney) => Ok(web::Json(FlowPayload {
      nodes: tree_jouney.generate_nodes(),
      edges: tree_jouney.generate_edges(&node_outcomes),
    })),
  }
}

#[derive(Serialize, Default)]
struct JourneyTransaction {
  transaction_id: String,
  timestamp: DateTime<Utc>,
}

#[get("/scripts")]
async fn list_scripts(
  data: web::Data<AppMutState>,
) -> Result<web::Json<HashMap<String, ScriptConfig>>, ShowMeErrors> {
  let saved_scripts = (*data
    .script_config
    .lock()
    .map_err(|_| ShowMeErrors::SharedLocking("get scripts endpoint".to_string()))?)
  .clone();

  Ok(web::Json(saved_scripts))
}

#[get("/{name}/transactions")]
async fn get_journey_transactions(
  journey_name: web::Path<String>,
  data: web::Data<AppMutState>,
) -> Result<web::Json<Vec<JourneyTransaction>>, ShowMeErrors> {
  let journey_transactions = get_logs(
    &Client::new(),
    &data,
    "", // Blank transaction ID effectively runs a * search.
    Some(
      format!(
        "/payload/entries/info/treeName eq \"{}\" and /payload/entries/info/nodeOutcome pr and /payload/eventName eq \"AM-NODE-LOGIN-COMPLETED\"",
        journey_name
      )
        .as_str(),
    ),
  )
    .await;

  match journey_transactions {
    Ok(transactions) => Ok(web::Json(
      transactions
        .result
        .iter()
        .map(|log| match log {
          GenericLog::ResultingLog(ll) => JourneyTransaction {
            timestamp: ll.timestamp,
            transaction_id: ll.payload.transaction_id.clone(),
          },
          GenericLog::Other(ll) => {
            dbg!(ll);
            JourneyTransaction::default()
          }
        })
        .collect(),
    )),
    Err(err) => Err(err),
  }
}

#[get("/{name}/scripts")]
async fn journey_script(
  name: web::Path<String>,
  data: web::Data<AppMutState>,
) -> Result<web::Json<HashMap<String, (NodeConfig, NodeData)>>, ShowMeErrors> {
  let (token_str, payload) = get_usable_token(&data.token, &data.payload, &data.token_str).await?;

  let dom = &data.token.dom;
  let tree = match data.authentication_tree.get_tree(&name.into_inner()) {
    None => HashMap::new(),
    Some(data) => data.get_node_info(dom, &token_str).await?,
  };

  Ok(web::Json(tree))
}

#[derive(Serialize)]
struct FlowPayload {
  nodes: Vec<ReactFlowNode>,
  edges: Vec<ReactFlowEdge>,
}

#[derive(Debug, Deserialize, Clone)]
struct JourneyFilter {
  starts_with: Option<String>,
  ends_with: Option<String>,
  contains: Option<String>,
}

#[get("")]
async fn get_journey(
  data: web::Data<AppMutState>,
  query: Query<JourneyFilter>,
) -> Result<web::Json<Vec<String>>, ShowMeErrors> {
  let sw = query.starts_with.clone().unwrap_or_default();
  let ew = query.ends_with.clone().unwrap_or_default();
  let cont = query.contains.clone().unwrap_or_default();

  let tree_list = data
    .authentication_tree
    .get_tree_list()
    .iter()
    .filter(|t| {
      if query.contains.is_some() {
        t.contains(&cont)
      } else {
        true
      }
    })
    .filter(|t| {
      if query.ends_with.is_some() {
        t.ends_with(&ew)
      } else {
        true
      }
    })
    .filter(|t| {
      if query.starts_with.is_some() {
        t.starts_with(&sw)
      } else {
        true
      }
    })
    .map(|t| t.clone())
    .collect();

  Ok(web::Json(tree_list))
}

async fn get_node_outcomes(
  transaction_id: &str,
  data: web::Data<AppMutState>,
) -> Result<Vec<NodeOutcomeEdge>, ShowMeErrors> {
  let client = &Client::new();

  // Get latest node outcomes with tracking IDs
  let most_recent_transaction_logs = get_logs(
    client,
    &data,
    &transaction_id,
    Some("/payload/entries/info/nodeOutcome pr"),
  )
  .await
  .map_or(vec![], |some_logs| some_logs.result);

  let mut tracking_ids: Vec<String> = most_recent_transaction_logs
    .clone()
    .iter()
    .flat_map(|l| match l {
      GenericLog::ResultingLog(log) => log.payload.tracking_ids.clone(),
      GenericLog::Other(_) => {
        vec![]
      }
    })
    .collect::<HashSet<_>>()
    .into_iter()
    .collect();

  tracking_ids.sort();

  let async_logs = future::join_all(if !tracking_ids.is_empty() {
    tracking_ids
      .into_iter() // takes ownership of each String
      .map(|tracking_id| {
        let local_state = data.clone();
        async move {
          println!(
            "Getting logs for additional tracking ID [{:?}]...",
            tracking_id
          );

          let some_logs = get_logs(
            client,
            &local_state,
            "",
            Some(&format!(
              "/payload/trackingIds eq \"{}\" and /payload/entries/info/nodeOutcome pr",
              tracking_id
            )),
          )
          .await;

          let final_logs = some_logs.map_or(vec![], |some_logs| some_logs.result);

          println!(
            "Got  [{:?}] logs for tracking ID [{:?}].",
            final_logs.len(),
            tracking_id,
          );

          final_logs
        }
      })
      .collect()
  } else {
    vec![]
  })
  .await;

  // Perform query for all other node outcomes in the journey with the tracking ID
  let all_transaction_logs: Vec<&GenericLog> =
    async_logs.iter().flat_map(|results| results).collect();

  Ok(
    all_transaction_logs
      .iter()
      .map(|log| match log {
        GenericLog::ResultingLog(ll) => {
          let thing = &ll.payload.entries.clone().unwrap_or(vec![])[0];
          NodeOutcomeEdge {
            name: thing.info.display_name.clone(),
            outcome: thing.info.node_outcome.clone(),
          }
        }
        GenericLog::Other(_) => NodeOutcomeEdge::default(),
      })
      .collect(),
  )
}

// this function could be located in a different module
pub fn trees_api(cfg: &mut web::ServiceConfig) {
  cfg.service(
    web::scope("/journey")
      .service(journey_flow)
      .service(journey_script)
      .service(get_journey)
      .service(list_scripts)
      .service(get_journey_transactions),
  );
}
