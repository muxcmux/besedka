use axum::{routing::post, Json, Router};
use super::{ApiRequest, Context, Result, Error, verify_read_permission};

pub fn router() -> Router {
    Router::new().route("/api/preview", post(preview))
}

async fn preview(ctx: Context, Json(req): Json<ApiRequest<String>>) -> Result<String> {
    let (site, user) = req.extract_verified(&ctx.db).await?;

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
