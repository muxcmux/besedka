use crate::{api::{ApiRequest, AppState, Result}, db::pages::find_by_site_and_path};
use axum::{routing::post, Json, Router, extract::State};
use sqlx::SqlitePool;
use super::PageConfig;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/config", post(page_config))
}

async fn page_config(
    State(db): State<SqlitePool>,
    Json(req): Json<ApiRequest<()>>
) -> Result<Json<PageConfig>> {
    let (site, _) = req.extract_verified(&db).await?;

    let locked = match find_by_site_and_path(&db, &req.site, &req.path).await {
        Err(_) => false,
        Ok(page) => page.locked
    };

    Ok(Json(PageConfig {
        anonymous: site.anonymous,
        moderated: site.moderated,
        locked,
    }))
}
