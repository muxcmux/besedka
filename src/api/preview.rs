use axum::{routing::post, Json, Router, extract::State};
use sqlx::SqlitePool;
use super::{ApiRequest, AppState, Result, Error, verify_read_permission};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/preview", post(preview))
}

async fn preview(
    State(db): State<SqlitePool>,
    Json(req): Json<ApiRequest<String>>,
) -> Result<String> {
    let (site, user) = req.extract_verified(&db).await?;

    verify_read_permission(&site, &user, None)?;

    match req.payload {
        None => Err(Error::UnprocessableEntity("Missing body")),
        Some(b) => {
            let md_body = markdown::to_html_with_options(&b, &markdown::Options::gfm())
                .map_err(|e| {
                    tracing::error!("{}", e);
                    Error::UnprocessableEntity("Your comment contains invalid markdown")
                })?;

            Ok(md_body)
        }
    }
}
