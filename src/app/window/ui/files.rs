
use rust_embed::RustEmbed;
use axum::{
  http::{header, StatusCode, Uri},
  response::{Html, IntoResponse, Response},
};

use crate::app::api::get_api_port;

#[derive(RustEmbed)]
#[folder = "UI/build"]
pub struct UiAssets;

static INDEX_HTML: &str = "index.html";


pub async fn static_handler(uri: Uri) -> impl IntoResponse {
  let path = uri.path().trim_start_matches('/');

  if path.is_empty() || path == INDEX_HTML {
    return index_html().await;
  }

  match UiAssets::get(path) {
    Some(content) => {
      let mime = mime_guess::from_path(path).first_or_octet_stream();

      ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
    }
    None => {
      if path.contains('.') {
        return not_found().await;
      }

      index_html().await
    }
  }
}

pub async fn index_html() -> Response {
  match UiAssets::get(INDEX_HTML) {
    Some(content) => {
        let html = String::from_utf8_lossy(content.data.as_ref())
          .replace("%SISR_API_PORT%", &get_api_port().unwrap_or(0).to_string());

        Html(html).into_response()
    }
    None => not_found().await,
  }
}

pub async fn not_found() -> Response {
  (StatusCode::NOT_FOUND, "404").into_response()
}