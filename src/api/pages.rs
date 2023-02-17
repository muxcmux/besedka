use crate::{api::{ApiRequest, AppState, Result}, db::pages::{create_or_find_by_site_and_path, self}};
use axum::{routing::patch, Json, Router, extract::State};
use sqlx::SqlitePool;

use super::{PageConfig, require_moderator};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/pages", patch(toggle_lock))
}

async fn toggle_lock(
    State(db): State<SqlitePool>,
    Json(req): Json<ApiRequest<()>>
) -> Result<Json<PageConfig>> {
    let (site, user) = req.extract_verified(&db).await?;

    require_moderator(&user)?;

    let page = create_or_find_by_site_and_path(&db, &req.site, &req.path, &req.title).await?;

    pages::toggle_lock(&db, page.id).await?;

    Ok(Json(PageConfig {
        anonymous: site.anonymous,
        moderated: site.moderated,
        locked: !page.locked,
    }))
}
