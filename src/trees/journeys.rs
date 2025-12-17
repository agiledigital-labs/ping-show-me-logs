use crate::errors::ShowMeErrors;
use crate::trees::nodes::{NodeConfig, NodeData, node_id_to_script_config};
use crate::{JourneyId, NodeOutcomeEdge};
use futures::future::JoinAll;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum NodeType {
  AccountLockoutNode,
  AgentDataStoreDecisionNode,
  AttributeCollectorNode,
  AttributePresentDecisionNode,
  ConfigProviderNode,
  CookiePresenceDecisionNode,
  CreateObjectNode,
  DataStoreDecisionNode,
  DebugNode,
  DeviceMatchNode,
  DeviceProfileCollectorNode,
  DeviceSaveNode,
  DisplayUserNameNode,
  EmailSuspendNode,
  IdentifyExistingUserNode,
  IdentityStoreDecisionNode,
  IncrementLoginCountNode,
  InnerTreeEvaluatorNode,
  LoginCountDecisionNode,
  MessageNode,
  OneTimePasswordGeneratorNode,
  PageNode,
  PatchObjectNode,
  PollingWaitNode,
  QueryFilterDecisionNode,
  RetryLimitDecisionNode,
  ScriptedDecisionNode,
  SelectIdPNode,
  SessionDataNode,
  SetCustomCookieNode,
  SetFailureUrlNode,
  SetStateNode,
  SetSuccessDetailsNode,
  SetSuccessUrlNode,
  SocialProviderHandlerNodeV2,
  UsernameCollectorNode,
  ZeroPageLoginNode,
  #[serde(rename = "product-PingOneProtectEvaluationNode")]
  ProductPingOneProtectEvaluationNode,
  #[serde(rename = "product-PingOneProtectInitializeNode")]
  ProductPingOneProtectInitializeNode,
  #[serde(rename = "product-PingOneProtectResultNode")]
  ProductPingOneProtectResultNode,
  #[serde(other)]
  NewThing,
}
impl Display for NodeType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    // The match expression will return the Result from the write! macro
    match self {
      NodeType::AccountLockoutNode => write!(f, "AccountLockoutNode"),
      NodeType::AgentDataStoreDecisionNode => write!(f, "AgentDataStoreDecisionNode"),
      NodeType::AttributeCollectorNode => write!(f, "AttributeCollectorNode"),
      NodeType::AttributePresentDecisionNode => write!(f, "AttributePresentDecisionNode"),
      NodeType::ConfigProviderNode => write!(f, "ConfigProviderNode"),
      NodeType::CookiePresenceDecisionNode => write!(f, "CookiePresenceDecisionNode"),
      NodeType::CreateObjectNode => write!(f, "CreateObjectNode"),
      NodeType::DataStoreDecisionNode => write!(f, "DataStoreDecisionNode"),
      NodeType::DebugNode => write!(f, "DebugNode"),
      NodeType::DeviceMatchNode => write!(f, "DeviceMatchNode"),
      NodeType::DeviceProfileCollectorNode => write!(f, "DeviceProfileCollectorNode"),
      NodeType::DeviceSaveNode => write!(f, "DeviceSaveNode"),
      NodeType::DisplayUserNameNode => write!(f, "DisplayUserNameNode"),
      NodeType::EmailSuspendNode => write!(f, "EmailSuspendNode"),
      NodeType::IdentifyExistingUserNode => write!(f, "IdentifyExistingUserNode"),
      NodeType::IdentityStoreDecisionNode => write!(f, "IdentityStoreDecisionNode"),
      NodeType::IncrementLoginCountNode => write!(f, "IncrementLoginCountNode"),
      NodeType::InnerTreeEvaluatorNode => write!(f, "InnerTreeEvaluatorNode"),
      NodeType::LoginCountDecisionNode => write!(f, "LoginCountDecisionNode"),
      NodeType::MessageNode => write!(f, "MessageNode"),
      NodeType::OneTimePasswordGeneratorNode => write!(f, "OneTimePasswordGeneratorNode"),
      NodeType::PageNode => write!(f, "PageNode"),
      NodeType::PatchObjectNode => write!(f, "PatchObjectNode"),
      NodeType::PollingWaitNode => write!(f, "PollingWaitNode"),
      NodeType::QueryFilterDecisionNode => write!(f, "QueryFilterDecisionNode"),
      NodeType::RetryLimitDecisionNode => write!(f, "RetryLimitDecisionNode"),
      NodeType::ScriptedDecisionNode => write!(f, "ScriptedDecisionNode"),
      NodeType::SelectIdPNode => write!(f, "SelectIdPNode"),
      NodeType::SessionDataNode => write!(f, "SessionDataNode"),
      NodeType::SetCustomCookieNode => write!(f, "SetCustomCookieNode"),
      NodeType::SetFailureUrlNode => write!(f, "SetFailureUrlNode"),
      NodeType::SetStateNode => write!(f, "SetStateNode"),
      NodeType::SetSuccessDetailsNode => write!(f, "SetSuccessDetailsNode"),
      NodeType::SetSuccessUrlNode => write!(f, "SetSuccessUrlNode"),
      NodeType::SocialProviderHandlerNodeV2 => write!(f, "SocialProviderHandlerNodeV2"),
      NodeType::UsernameCollectorNode => write!(f, "UsernameCollectorNode"),
      NodeType::ZeroPageLoginNode => write!(f, "ZeroPageLoginNode"),
      // By default, Display uses the variant name, not the serde rename
      NodeType::ProductPingOneProtectEvaluationNode => {
        write!(f, "ProductPingOneProtectEvaluationNode")
      }
      NodeType::ProductPingOneProtectInitializeNode => {
        write!(f, "ProductPingOneProtectInitializeNode")
      }
      NodeType::ProductPingOneProtectResultNode => write!(f, "ProductPingOneProtectResultNode"),
      _ => {
        write!(f, "some new fancy things")
      }
    }
  }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
  connections: HashMap<String, String>,
  display_name: String,
  node_type: NodeType,
  x: Option<f32>,
  y: Option<f32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticNode {
  x: f32,
  y: f32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Tree {
  #[serde(rename = "_id")]
  id: String,
  #[serde(rename = "_rev")]
  rev: String,
  identity_resource: Option<String>,
  entry_node_id: String,
  inner_tree_only: bool,
  no_session: bool,
  must_run: bool,
  enabled: bool,
  transaction_only: Option<bool>,
  ui_config: HashMap<String, String>,
  nodes: HashMap<String, Node>,
  static_nodes: HashMap<String, StaticNode>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationTreeList {
  result: Vec<Tree>,
  result_count: i32,
  paged_results_cookie: Option<String>,
  total_paged_results_policy: String,
  total_paged_results: i16,
  remaining_paged_results: i16,
}

impl AuthenticationTreeList {
  pub fn get_tree_list(&self) -> Vec<JourneyId> {
    self.result.iter().map(|t| t.id.to_owned()).collect()
  }
  pub fn get_tree(&self, name: &str) -> Option<Tree> {
    self.result.iter().find(|t| t.id.eq(name)).cloned()
  }
}

#[derive(Serialize)]
enum EdgeType {
  #[serde(rename = "default")]
  Normal,
  #[serde(rename = "default")]
  Error,
}

#[derive(Serialize)]
struct ReactFlowEdgeStyle {
  stroke: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactFlowEdge {
  id: String,
  #[serde(rename = "type")]
  edge_type: EdgeType,
  source: String,
  target: String,
  source_handle: String,
  style: ReactFlowEdgeStyle,
}

#[derive(Serialize)]
struct Position {
  x: f32,
  y: f32,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "lowercase")]
enum HandlePosition {
  Left,
  #[default]
  Right,
  Top,
  Bottom,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "lowercase")]
enum HandleType {
  #[default]
  Source,
  Target,
}

#[derive(Serialize, Default)]
struct ReactFlowNodeHandle {
  width: Option<f32>,
  height: Option<f32>,
  id: Option<String>,
  x: f32,
  y: f32,
  position: HandlePosition,
  handle_type: HandleType,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactFlowNode {
  id: String,
  position: Position,
  data: HashMap<String, String>,
  handles: Option<Vec<ReactFlowNodeHandle>>,
  source_position: HandlePosition,
  target_position: HandlePosition,
}

impl Tree {
  pub fn get_id_vec(&self) -> Vec<String> {
    self.nodes.iter().map(|t| t.0).cloned().collect()
  }

  pub async fn get_node_info(
    &self,
    dom: &str,
    token_str: &str,
  ) -> Result<HashMap<String, (NodeConfig, NodeData)>, ShowMeErrors> {
    let test = self
      .nodes
      .iter()
      .map(async |t| {
        node_id_to_script_config(&t.1.node_type, t.0, dom, &token_str)
          .await
          .map(|v| (t.0.clone(), v))
          .unwrap_or(("nothing".to_string(), (NodeConfig::None, NodeData::None)))
      })
      .collect::<JoinAll<_>>()
      .await
      .iter()
      .cloned()
      .collect();

    Ok(test)
  }

  pub fn generate_edges(&self, outcomes: &Vec<NodeOutcomeEdge>) -> Vec<ReactFlowEdge> {
    let start_edge = ReactFlowEdge {
      id: "startNode".to_string(),
      edge_type: EdgeType::Normal,
      source: "startNode".to_string(),
      target: self.entry_node_id.clone(),
      source_handle: "ok".to_string(),
      style: ReactFlowEdgeStyle {
        stroke: "green".to_string(),
      },
    };

    let mut rest = self
      .nodes
      .iter()
      .flat_map(|t| {
        t.1.connections.iter().map(move |v| {
          let handle = v.0.to_string();

          let node_outcome = outcomes
            .iter()
            .find(|outcome| outcome.name == t.1.display_name && outcome.outcome == handle);

          ReactFlowEdge {
            id: format!("{}/{}", t.0.to_owned(), v.0.to_owned()),
            edge_type: if v.0.starts_with("error") {
              EdgeType::Error
            } else {
              EdgeType::Normal
            },
            source: t.0.to_owned(),
            target: v.1.to_string(),
            source_handle: v.0.to_string(),
            style: ReactFlowEdgeStyle {
              stroke: match node_outcome {
                Some(outcome) => {
                  if outcome.outcome == "error" {
                    "red".to_string()
                  } else {
                    "green".to_string()
                  }
                }
                None => "grey".to_string(),
              },
            },
          }
        })
      })
      .collect::<Vec<ReactFlowEdge>>();

    rest.push(start_edge);
    rest
  }
  pub fn generate_nodes(&self) -> Vec<ReactFlowNode> {
    let static_nodes = self.static_nodes.iter().map(|t| ReactFlowNode {
      id: t.0.to_owned(),
      position: Position { x: t.1.x, y: t.1.y },
      data: HashMap::from([("name".to_string(), t.0.clone())]),
      handles: Some(vec![ReactFlowNodeHandle {
        width: None,
        height: None,
        id: Some("ok".to_string()),
        x: 0.0,
        y: 0.0,
        position: Default::default(),
        handle_type: Default::default(),
      }]),
      source_position: HandlePosition::Right,
      target_position: HandlePosition::Left,
    });

    let other_nodes = self.nodes.iter().map(|t| {
      let test = t
        .1
        .connections
        .keys()
        .enumerate()
        .map(|(idx, t)| ReactFlowNodeHandle {
          width: None,
          height: None,
          id: Some(t.to_string()),
          x: 0.0,
          y: (idx * 10 + 10) as f32,
          position: Default::default(),
          handle_type: HandleType::Source,
        })
        .collect::<Vec<ReactFlowNodeHandle>>();

      ReactFlowNode {
        id: t.0.to_owned(),
        position: Position {
          x: t.1.x.unwrap_or(0.0),
          y: t.1.y.unwrap_or(0.0),
        },
        data: HashMap::from([
          ("name".to_string(), t.1.display_name.clone()),
          (
            "type".to_string(),
            serde_json::to_string(&t.1.node_type).unwrap_or("ToDo better".to_string()),
          ),
        ]),
        handles: Some(test),
        source_position: HandlePosition::Right,
        target_position: HandlePosition::Left,
      }
    });
    static_nodes.chain(other_nodes).collect()
  }
}
