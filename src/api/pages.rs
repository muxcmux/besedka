use crate::{api::{ApiRequest, Context, Result}, db::pages::{create_or_find_by_site_and_path, toggle_lock}};
use axum::{routing::patch, Json, Router};

use super::PageConfig;

pub fn router() -> Router {
    Router::new().route("/api/pages", patch(page))
}

async fn page(ctx: Context, Json(req): Json<ApiRequest<()>>) -> Result<Json<PageConfig>> {
    let (site, _) = req.extract_verified(&ctx.db).await?;

    let page = create_or_find_by_site_and_path(&ctx.db, &req.site, &req.path).await?;

    toggle_lock(&ctx.db, page.id).await?;

    Ok(Json(PageConfig {
        anonymous: site.anonymous,
        moderated: site.moderated,
        locked: !page.locked,
    }))
}
