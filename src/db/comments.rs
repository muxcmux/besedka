use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{SqlitePool, FromRow, query_as};

use crate::api::{Result, Cursor};

#[derive(FromRow, Clone, Debug, Serialize)]
pub struct Comment {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub body: String,
    pub avatar: Option<String>,
    pub replies_count: i64,
    pub locked: bool,
    pub reviewed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn root_comments(
    db: &SqlitePool,
    page_id: i64,
    limit: i64,
    cursor: Option<Cursor>,
) -> Result<Vec<Comment>> {
    let query = match cursor {
        Some(cur) => {
            query_as::<_, Comment>(
                r#"
                    SELECT id, parent_id, name, body, avatar, replies_count,
                    locked, reviewed_at, created_at, updated_at
                    FROM comments
                    WHERE page_id = ?
                    AND reviewed_at NOT NULL
                    AND parent_id IS NULL
                    AND (datetime(created_at) < datetime(?) OR (datetime(created_at) = datetime(?) AND id < ?))
                    ORDER BY datetime(created_at) DESC
                    LIMIT ?
                "#,
             )
             .bind(page_id)
             .bind(cur.created_at)
             .bind(cur.created_at)
             .bind(cur.id)
             .bind(limit)
        },
        None => {
            query_as::<_, Comment>(
                r#"
                    SELECT id, parent_id, name, body, avatar, replies_count,
                    locked, reviewed_at, created_at, updated_at
                    FROM comments
                    WHERE page_id = ?
                    AND reviewed_at NOT NULL
                    AND parent_id IS NULL
                    ORDER BY datetime(created_at) DESC
                    LIMIT ?
                "#,
            )
            .bind(page_id)
            .bind(limit)
        }
    };

    Ok(query.fetch_all(db).await?)
}

pub async fn nested_replies(
    db: &SqlitePool,
    limit: i64,
    parents: &Vec<Comment>,
) -> Result<Vec<Comment>> {
    let parent_ids: Vec<String> = parents.iter().map(|p| p.id.to_string()).collect();
    let ids = parent_ids.join(",");

    let query = format!(
        r#"
        SELECT
            id, parent_id, name, body, avatar, replies_count,
            locked, reviewed_at, created_at, updated_at
        FROM (
            SELECT
                r.id AS id,
                r.parent_id AS parent_id,
                r.name AS name,
                r.body AS body,
                r.avatar AS avatar,
                r.replies_count AS replies_count,
                r.locked AS locked,
                r.reviewed_at AS reviewed_at,
                r.created_at AS created_at,
                r.updated_at AS updated_at,
                row_number() OVER (PARTITION BY c.id ORDER BY datetime(r.created_at), r.id) AS rn
            FROM comments c
            LEFT JOIN comments r
            ON r.parent_id = c.id
            WHERE r.reviewed_at NOT NULL
        )
        WHERE parent_id IN({ids})
        AND id NOT NULL
        AND rn <= {limit}
        ORDER BY datetime(created_at) ASC;
    "#,
        ids = ids,
        limit = limit
    );

    Ok(query_as::<_, Comment>(&query).fetch_all(db).await?)
}

pub async fn replies(
    db: &SqlitePool,
    parent_id: i64,
    limit: i64,
    cursor: Option<Cursor>,
) -> Result<Vec<Comment>> {
    let query = match cursor {
        Some(cur) => {
            query_as::<_, Comment>(
                r#"
                     SELECT id, parent_id, name, body, avatar, replies_count,
                     locked, reviewed_at, created_at, updated_at
                     FROM comments
                     WHERE parent_id = ?
                     AND reviewed_at NOT NULL
                     AND (datetime(created_at) > datetime(?) OR (datetime(created_at) = datetime(?) AND id > ?))
                     ORDER BY datetime(created_at) ASC
                     LIMIT ?
                 "#,
             )
             .bind(parent_id)
             .bind(cur.created_at)
             .bind(cur.created_at)
             .bind(cur.id)
             .bind(limit)
        },
        None => {
            query_as::<_, Comment>(
               r#"
                    SELECT id, parent_id, name, body, avatar, replies_count,
                    locked, reviewed_at, created_at, updated_at
                    FROM comments
                    WHERE parent_id = ?
                    AND reviewed_at NOT NULL
                    ORDER BY datetime(created_at) ASC
                    LIMIT ?
                "#,
            )
            .bind(parent_id)
            .bind(limit)
        }
    };

    Ok(query.fetch_all(db).await?)
}

