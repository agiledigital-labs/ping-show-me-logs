use crate::errors::ShowMeErrors;
use crate::ping_logs::logs::{GenericLog, tail_logs};
use crate::ws_server::LogsServerHandle;
use crate::{AppMutState, add_to_rolling_buffer};
use actix_web::web::Data;
use reqwest::Client;
use std::path::Path;
use std::time::Duration;
use std::{fs, io};
use tantivy::Directory;
use tantivy::directory::MmapDirectory;
use tantivy::schema::{Field, STORED, Schema, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy, TantivyError, doc};
use tokio::time::sleep;

pub struct LogWatcher {
  logs_schema: Schema,
  pub logs_index: Index,
  pub id_field: Field,
  server_tx: LogsServerHandle,
  app_data: Data<AppMutState>,
}

impl LogWatcher {
  pub fn new(
    server_tx: LogsServerHandle,
    app_data: Data<AppMutState>,
  ) -> Result<(Self, IndexReader, Schema), ShowMeErrors> {
    if !fs::exists("./index")? {
      fs::create_dir("./index")?;
    }

    let index_path = Path::new("./index");
    let dir = MmapDirectory::open(index_path)
      .map_err(|_| ShowMeErrors::IdLockError("alsdkjfl".to_string()))?;

    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("transactionId", TEXT | STORED);

    let schema = schema_builder.build();

    let search_index = Index::open_or_create(dir, schema.clone())?;
    let transaction_id_schema = schema.get_field("transactionId")?;

    let reader = search_index
      .clone()
      .reader_builder()
      .reload_policy(ReloadPolicy::OnCommitWithDelay)
      .try_into()?;

    Ok((
      Self {
        logs_schema: schema.clone(),
        logs_index: search_index,
        id_field: transaction_id_schema,
        server_tx,
        app_data,
      },
      reader,
      schema,
    ))
  }

  pub async fn watch(self) -> io::Result<()> {
    let client = Client::new();
    let mut cookie: Option<String> = None;

    println!("1");

    let mut index_writer = self.logs_index.clone().writer(50_000_000).map_err(|t| {
      dbg!(t.clone());
      <TantivyError as Into<ShowMeErrors>>::into(t)
    })?;
    // .map_err(Into::<ShowMeErrors>::into)?;

    println!("2");
    let transaction_id_schema = self
      .logs_schema
      .get_field("transactionId")
      .map_err(Into::<ShowMeErrors>::into)?;

    println!("3");
    loop {
      let logs = tail_logs(&client, &self.app_data, cookie, None).await?;

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
              let is_new = add_to_rolling_buffer(&self.app_data.rolling_id_list, id.clone());
              if is_new.is_some() { Some(id) } else { None }
            }
            _t => None,
          };
          Ok(res)
        })
        .collect::<Result<Vec<Option<String>>, ShowMeErrors>>()?;

      for doc in docs {
        if let Some(t) = doc {
          self
            .server_tx
            .new_transaction_id("".to_string(), t.clone())
            .await;
        }
      }

      let _ = index_writer.commit().map_err(Into::<ShowMeErrors>::into)?;
      sleep(Duration::from_secs(2)).await;
    }

    Ok(())
  }
}
