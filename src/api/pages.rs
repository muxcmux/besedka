use crate::{api::{ApiRequest, Context, Result}, db::pages::{create_or_find_by_site_and_path, self}};
use axum::{routing::patch, Json, Router};

use super::{PageConfig, require_moderator};

pub fn router() -> Router {
    Router::new().route("/api/pages", patch(toggle_lock))
}

async fn toggle_lock(ctx: Context, Json(req): Json<ApiRequest<()>>) -> Result<Json<PageConfig>> {
    let (site, user) = req.extract_verified(&ctx.db).await?;

    require_moderator(&user)?;

    let page = create_or_find_by_site_and_path(&ctx.db, &req.site, &req.path).await?;

    pages::toggle_lock(&ctx.db, page.id).await?;

    Ok(Json(PageConfig {
        anonymous: site.anonymous,
        moderated: site.moderated,
        locked: !page.locked,
    }))
}
