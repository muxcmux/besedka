use crate::{api::{ApiRequest, Context, Result}, db::pages::find_by_site_and_path};
use axum::{routing::post, Json, Router};
use super::PageConfig;

pub fn router() -> Router {
    Router::new().route("/api/config", post(page_config))
}

async fn page_config(ctx: Context, Json(req): Json<ApiRequest<()>>) -> Result<Json<PageConfig>> {
    let (site, _) = req.extract_verified(&ctx.db).await?;

    let locked = match find_by_site_and_path(&ctx.db, &req.site, &req.path).await {
        Err(_) => false,
        Ok(page) => page.locked
    };

    Ok(Json(PageConfig {
        anonymous: site.anonymous,
        moderated: site.moderated,
        locked,
        theme: site.theme,
    }))
}
