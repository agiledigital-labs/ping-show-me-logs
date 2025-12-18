use crate::errors::ShowMeErrors;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct ScriptListRoot {
  #[serde(rename = "type")]
  pub type_field: String,
  pub properties: Properties,
  pub required: Vec<String>,
}

#[derive(Deserialize)]
pub struct Properties {
  pub outputs: Outputs,
  pub outcomes: Outcomes,
  pub script: Script,
  pub inputs: Inputs,
}

#[derive(Deserialize)]
pub struct Outputs {
  pub title: String,
  pub description: String,
  #[serde(rename = "propertyOrder")]
  pub property_order: i64,
  pub items: Items,
  #[serde(rename = "type")]
  pub type_field: String,
  #[serde(rename = "exampleValue")]
  pub example_value: String,
  pub default: Vec<String>,
}

#[derive(Deserialize)]
pub struct Items {
  #[serde(rename = "type")]
  pub type_field: String,
}

#[derive(Deserialize)]
pub struct Outcomes {
  pub title: String,
  pub description: String,
  #[serde(rename = "propertyOrder")]
  pub property_order: i64,
  pub items: Items,
  #[serde(rename = "type")]
  pub type_field: String,
  #[serde(rename = "exampleValue")]
  pub example_value: String,
  pub default: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Script {
  pub title: String,
  pub description: String,
  #[serde(rename = "propertyOrder")]
  pub property_order: i64,
  #[serde(rename = "enum")]
  pub enum_field: Vec<String>,
  pub options: Options,
  #[serde(rename = "enumNames")]
  pub enum_names: Vec<String>,
  #[serde(rename = "type")]
  pub type_field: String,
  #[serde(rename = "exampleValue")]
  pub example_value: String,
  pub default: String,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct Options {
  pub enum_titles: Vec<String>,
}

#[derive(Deserialize)]
pub struct Inputs {
  pub title: String,
  pub description: String,
  #[serde(rename = "propertyOrder")]
  pub property_order: i64,
  pub items: Items,
  #[serde(rename = "type")]
  pub type_field: String,
  #[serde(rename = "exampleValue")]
  pub example_value: String,
  pub default: Vec<String>,
}

#[derive(Serialize, Clone)]
pub struct ScriptConfig {
  name: String,
  id: String,
  title: String,
}

impl From<(String, String, String)> for ScriptConfig {
  fn from((name, title, id): (String, String, String)) -> Self {
    Self { name, title, id }
  }
}

#[derive(Serialize, Clone)]
pub struct RichScript {
  name: String,
  id: String,
  title: String,
  script: Script,
}

pub async fn get_rich_script(
  client: &Client,
  dom: &str,
  token_str: &str,
  script_config: &ScriptConfig,
) -> Result<RichScript, ShowMeErrors> {
  let script_txt = &client
    .get(format!("{dom}/am/json/alpha/scripts/{}", script_config.id))
    .header("authorization", format!("Bearer {}", token_str))
    .send()
    .await?
    .bytes()
    .await?;

  let script_data: Script = serde_json::from_slice(script_txt)?;
  Ok(RichScript {
    name: script_config.name.clone(),
    id: script_config.id.clone(),
    title: script_config.title.clone(),
    script: script_data,
  })
}

pub async fn list_scripts(
  client: &Client,
  dom: &str,
  token_str: &str,
) -> Result<HashMap<String, ScriptConfig>, ShowMeErrors> {
  let script_list = &client
    .post(format!("{dom}/am/json/realms/root/realms/alpha/realm-config/authentication/authenticationtrees/nodes/ScriptedDecisionNode?_action=schema"))
    .body("{}")
    .header("authorization", format!("Bearer {}", token_str))
    .header("Accept-API-Version", "protocol=2.1,resource=1.0")
    .send()
    .await?
    .bytes()
    .await?;

  let config: ScriptListRoot = serde_json::from_slice(script_list)?;

  let combined = config
    .properties
    .script
    .enum_names
    .into_iter()
    .zip(config.properties.script.options.enum_titles.into_iter())
    .zip(config.properties.script.enum_field.into_iter())
    .map(|((a, b), c)| (a.clone(), ScriptConfig::from((a, b, c))))
    .collect::<HashMap<String, ScriptConfig>>();

  Ok(combined)
}
