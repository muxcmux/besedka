use axum::{
    body::{boxed, Full},
    http::header,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::NaiveDateTime;
use rust_embed::RustEmbed;

use crate::api::Error;

#[derive(RustEmbed)]
#[folder = "frontend/"]
struct Assets;

struct StaticFile<T>(T);

impl<T: Into<String>> IntoResponse for StaticFile<T> {
    fn into_response(self) -> Response {
        let path = self.0.into();
        match Assets::get(path.as_str()) {
            None => Error::NotFound.into_response(),
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let metadata = content.metadata;
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                let etag = metadata
                    .sha256_hash()
                    .iter()
                    .map(|b| format!("{:02x}", b).to_string())
                    .collect::<Vec<String>>()
                    .join("");

                let mut response = Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .header(header::ETAG, etag);

                if let Some(last_modified) = metadata.last_modified() {
                    let date = NaiveDateTime::from_timestamp_opt(last_modified as i64, 0).unwrap();
                    response = response.header(
                        header::LAST_MODIFIED,
                        date.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
                    );
                }

                response.body(body).unwrap()
            }
        }
    }
}

pub fn router() -> Router {
    Router::new().route("/comments.js", get(comments))
}

async fn comments() -> impl IntoResponse {
    StaticFile("dist/comments.js")
}
