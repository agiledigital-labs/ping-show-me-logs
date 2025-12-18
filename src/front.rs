use crate::AppMutState;
use crate::errors::ShowMeErrors;
use actix_web::http::header::ContentType;
use actix_web::{HttpRequest, HttpResponse, get, mime, web};
use reqwest::Client;
use std::path::PathBuf;

pub async fn index(req: HttpRequest) -> Result<HttpResponse, ShowMeErrors> {
  let path: PathBuf = req
    .match_info()
    .query("filename")
    .parse()
    .map_err(|_| ShowMeErrors::ParsingUiPath)?;
  Ok(match path.to_str() {
    Some(pp) => {
      println!("{}", pp);
      if pp.eq("") || pp.eq("/") {
        HttpResponse::Ok()
          .content_type(ContentType::html())
          .body(include_str!(concat!("..", "/", "ui/dist/index.html")))
      } else if pp.ends_with("js") {
        HttpResponse::Ok()
          .content_type(mime::APPLICATION_JAVASCRIPT)
          .body(include_str!(concat!("..", "/", env!("JS"))))
      } else if pp.ends_with("css") {
        HttpResponse::Ok()
          .content_type("text/css")
          .body(include_str!(concat!("..", "/", env!("CSS"))))
      } else if pp.ends_with("vite.svg") {
        HttpResponse::Ok()
          .content_type(ContentType::octet_stream())
          .body(include_str!(concat!("..", "/", "ui/dist/vite.svg")))
      } else {
        HttpResponse::NotFound().body("")
      }
    }
    _ => HttpResponse::NotFound().body(""),
  })
}

#[get("/idm")]
pub async fn idm(data: web::Data<AppMutState>) -> Result<String, ShowMeErrors> {
  let client = Client::new();
  let metrics = &*client
    .get(format!("{}/monitoring/prometheus/idm", &data.token.dom,))
    .header("x-api-key", &data.key)
    .header("x-api-secret", &data.sec)
    .send()
    .await?
    .text()
    .await?;

  Ok(metrics.to_string())
}

#[get("/am")]
pub async fn am(data: web::Data<AppMutState>) -> Result<String, ShowMeErrors> {
  let client = Client::new();
  let metrics = client
    .get(format!("{}/monitoring/prometheus/am", &data.token.dom,))
    .header("x-api-key", &data.key)
    .header("x-api-secret", &data.sec)
    .send()
    .await?
    .text()
    .await?;

  Ok(metrics.to_string())
}
