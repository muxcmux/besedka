use crate::db::pages::Page;
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{query_as, SqlitePool};

#[derive(Debug, Serialize)]
pub struct Comment {
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

pub async fn comments(db: &SqlitePool, page: String) -> Result<Vec<Comment>, sqlx::Error> {
    let page = query_as!(Page, "SELECT * FROM pages WHERE (site || path = ?)", page)
        .fetch_optional(db)
        .await
        .unwrap();

    match page {
        None => Ok(vec![]),
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
            .fetch_all(db)
            .await;
            parents
        }
    }
}
