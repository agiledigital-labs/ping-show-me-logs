use actix_web::error;
use actix_web::http::StatusCode;
use std::env::VarError;
use std::io::ErrorKind;
use tantivy::TantivyError;
use tantivy::query::QueryParserError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShowMeErrors {
  #[error("invalid configuration")]
  Config(#[from] VarError),
  #[error("ping api error")]
  PingApiError(#[from] reqwest::Error),
  #[error("parsing error, check the struct")]
  ParsingUiPath,
  #[error("parsing error, check the struct")]
  Parsing(#[from] serde_json::Error),
  #[error("failed to lock shared state [{0}].")]
  SharedLocking(String),
  #[error("There are no logs for id: [{0}].")]
  NoLogsFound(String),
  #[error("Creation of the api token failed: [{0}].")]
  TokenDefault(String),
  #[error("Failed to create openssl rand but")]
  TokenOpenSsl(#[from] openssl::error::ErrorStack),
  #[error("Failed to create openssl rand but")]
  TokenReadKey(#[from] std::io::Error),
  #[error("Failed to create and encode the token")]
  TokenCreateToken(#[from] jsonwebkey::Error),
  #[error("Failed to create and encode the token")]
  TokenCreateKey(#[from] jsonwebtoken::errors::Error),
  #[error("Actix Web Error")]
  ActixWs(#[from] actix_web::Error),
  #[error("Log Search Error")]
  TrivyError(#[from] TantivyError),
  #[error("db query parsing error")]
  QueryParserError(#[from] QueryParserError),
  #[error("id list lock error")]
  IdLockError(String),
}

impl error::ResponseError for ShowMeErrors {
  fn status_code(&self) -> StatusCode {
    match *self {
      ShowMeErrors::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
      ShowMeErrors::PingApiError(_) => StatusCode::BAD_GATEWAY,
      ShowMeErrors::ParsingUiPath => StatusCode::BAD_REQUEST,
      ShowMeErrors::Parsing(_) => StatusCode::INTERNAL_SERVER_ERROR,
      ShowMeErrors::SharedLocking(_) => StatusCode::INTERNAL_SERVER_ERROR,
      ShowMeErrors::NoLogsFound(_) => StatusCode::NOT_FOUND,
      ShowMeErrors::TokenDefault(_) => StatusCode::INTERNAL_SERVER_ERROR,
      _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

impl From<ShowMeErrors> for std::io::Error {
  fn from(value: ShowMeErrors) -> Self {
    Self::new(ErrorKind::Other, value.to_string())
  }
}
