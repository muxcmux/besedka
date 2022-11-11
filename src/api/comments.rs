use std::collections::HashMap;

use crate::{
    api::{Context, Result},
    db::configs::{find_or_default, Config},
};
use axum::{
    async_trait,
    extract::{FromRequestParts, Path, Query},
    http::{Uri, request::Parts},
    routing::get,
    Json, Router, response::IntoResponse, RequestPartsExt,
};
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{query_as, FromRow};
use tracing::info;

use super::Error;

pub fn router() -> Router {
    Router::new().route("/api/comments/*page", get(comments))
}

#[derive(FromRow, Clone, Debug, Serialize)]
struct Comment {
    id: i64,
    parent_id: Option<i64>,
    name: String,
    body: String,
    avatar: Option<String>,
    replies_count: i64,
    locked_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Serialize)]
struct CommentWithReplies {
    id: i64,
    parent_id: Option<i64>,
    name: String,
    body: String,
    avatar: Option<String>,
    replies_count: i64,
    locked_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    replies: Vec<Comment>,
}

#[derive(Serialize)]
struct Response {
    comments: Vec<CommentWithReplies>,
    cursor: Option<Cursor>,
}

#[derive(Debug, Serialize)]
struct Page {
    id: i64,
    site: String,
    path: String,
    comments_count: i64,
    locked_at: Option<NaiveDateTime>,
}

#[derive(Serialize)]
struct Cursor {
    last_id: i64,
    last_created_at: NaiveDateTime,
}

struct ExtractPageAndConfig {
    page: Page,
    config: Config,
}

#[async_trait]
impl<T: Send + Sync> FromRequestParts<T> for ExtractPageAndConfig {

    type Rejection = axum::response::Response;

    async fn from_request_parts(parts: &mut Parts, state: &T) -> Result<Self, Self::Rejection> {

        fn site_from_page(page: &str) -> Result<String> {
            let uri: anyhow::Result<Uri, _> = format!("https://{}", page).parse();
            match uri {
                Err(_) => Err(Error::NotFound),
                Ok(uri) => match uri.host() {
                    Some(site) => Ok(site.to_string()),
                    None => Err(Error::NotFound),
                },
            }
        }

        let Path(page) = Path::<String>::from_request_parts(parts, state)
            .await
            .map_err(|err| err.into_response())?;

        let ctx = parts.extract::<Context>()
            .await
            .map_err(|err| err.into_response())?;

        let site = site_from_page(&page)
            .map_err(|err| err.into_response())?;

        let config = find_or_default(&ctx.db, &site)
            .await
            .map_err(|err| Error::Anyhow(err).into_response())?;

        let page = query_as!(Page, "SELECT * FROM pages WHERE (site || path = ?)", page)
            .fetch_one(&ctx.db)
            .await
            .map_err(|err| Error::Sqlx(err).into_response())?;

        Ok(ExtractPageAndConfig { page, config })
    }
}

async fn comments(
    ctx: Context,
    ExtractPageAndConfig{page, config}: ExtractPageAndConfig,
    Query(query): Query<HashMap<String, String>>,
) -> Result<Json<Response>> {

    let cursor = Cursor {
        last_id: 10,
        last_created_at: NaiveDateTime::parse_from_str("2015-09-05T23:56:04", "%Y-%m-%dT%H:%M:%S")
            .unwrap(),
    };

    info!("cursor from query: {:?}", query.get("cursor"));

    let limit = config.comments_per_page + 1;

    let parents = query_as::<_, Comment>(
        r#"
            SELECT id, parent_id, name, body, avatar, replies_count,
            locked_at, created_at, updated_at
            FROM comments
            WHERE page_id = ?
            AND parent_id IS NULL
            AND (created_at < ? OR (created_at = ? AND id < ?))
            ORDER BY created_at DESC
            LIMIT ?
        "#,
    )
    .bind(page.id)
    .bind(cursor.last_created_at)
    .bind(cursor.last_created_at)
    .bind(cursor.last_id)
    .bind(limit)
    .fetch_all(&ctx.db)
    .await?;

    let parent_ids: Vec<String> = parents.iter().map(|p| p.id.to_string()).collect();
    let ids = parent_ids.join(",");

    let replies_query = format!(
        r#"
        SELECT
            id, parent_id, name, body, avatar, replies_count,
            locked_at,  created_at, updated_at
        FROM (
            SELECT
                r.id AS id,
                r.parent_id AS parent_id,
                r.name AS name,
                r.body AS body,
                r.avatar AS avatar,
                r.replies_count AS replies_count,
                r.locked_at AS locked_at,
                r.created_at AS created_at,
                r.updated_at AS updated_at,
                row_number() OVER (PARTITION BY c.id ORDER BY r.created_at, r.id) AS rn
            FROM comments c
            LEFT JOIN comments r
            ON r.parent_id = c.id
        )
        WHERE parent_id IN({ids}) AND id NOT NULL AND rn <= {limit}
        ORDER BY created_at ASC;
    "#,
        ids = ids,
        limit = config.replies_per_comment
    );

    let replies = query_as::<_, Comment>(&replies_query)
        .fetch_all(&ctx.db)
        .await?;

    let mut comments = vec![];

    for parent in parents {
        comments.push(CommentWithReplies {
            id: parent.id,
            parent_id: parent.parent_id,
            name: parent.name,
            body: parent.body,
            avatar: parent.avatar,
            replies_count: parent.replies_count,
            locked_at: parent.locked_at,
            created_at: parent.created_at,
            updated_at: parent.updated_at,
            replies: replies
                .iter()
                .filter(|r| r.parent_id == Some(parent.id))
                .cloned()
                .collect(),
        });
    }

    Ok(Json(Response {
        comments,
        cursor: None,
    }))
}
