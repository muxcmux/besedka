use crate::api::{Context, Result};
use axum::{extract::Path, routing::get, Json, Router};
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::query_as;

pub fn router() -> Router {
    Router::new().route("/api/comments/*page", get(comments))
}

#[derive(Debug, Serialize)]
struct Comment {
    id: i64,
    page_id: i64,
    parent_id: Option<i64>,
    user_id: Option<i64>,
    name: String,
    body: String,
    avatar: Option<String>,
    replies_count: i64,
    locked_at: Option<NaiveDateTime>,
    reviewed_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
struct Page {
    id: i64,
    site: String,
    path: String,
    comments_count: i64,
    locked_at: Option<NaiveDateTime>,
}

async fn comments(
    ctx: Context,
    Path(page): Path<String>
) -> Result<Json<Vec<Comment>>> {
    let page_record = query_as!(Page, "SELECT * FROM pages WHERE (site || path = ?)", page)
        .fetch_optional(&ctx.db)
        .await
        .unwrap();

    match page_record {
        None => Ok(Json(vec![])),
        Some(p) => {
            let parents = query_as!(
                Comment,
                r#"
                    SELECT * FROM comments
                    WHERE page_id = ? AND parent_id IS NULL
                    ORDER BY created_at desc
                "#,
                p.id
            )
            .fetch_all(&ctx.db)
            .await?;
            Ok(Json(parents))
        }
    }
}
