use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{SqlitePool, FromRow, query_as, query};

use crate::api::{Base64, Result, Cursor};

use super::UTC_DATETIME_FORMAT;

#[derive(FromRow, Clone, Debug, Serialize)]
pub struct Comment {
    pub id: i64,
    #[serde(skip_serializing)]
    pub page_id: i64,
    #[serde(skip_serializing)]
    pub parent_id: Option<i64>,
    pub name: String,
    pub body: String,
    pub avatar: Option<String>,
    pub replies_count: i64,
    #[serde(skip_serializing)]
    pub reviewed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub token: Base64,
}

pub async fn find(db: &SqlitePool, id: i64) -> sqlx::Result<Comment> {
    Ok(
        query_as!(
            Comment,
            r#"
                SELECT
                id, page_id, parent_id, name, body, avatar, replies_count, reviewed,
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>",
                token as "token: Base64"
                FROM comments WHERE id = ?
            "#,
            id
        ).fetch_one(db).await?
    )
}

pub async fn approve(db: &SqlitePool, id: i64) -> sqlx::Result<Comment> {
    let mut tx = db.begin().await?;

    let comment = query_as::<_, Comment>(
        "UPDATE comments SET reviewed = 1 WHERE id = ? AND reviewed = 0 RETURNING * "
    ).bind(id).fetch_one(&mut tx).await?;

    query!("UPDATE pages SET comments_count = comments_count + 1 WHERE id = ?", comment.page_id)
        .execute(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(comment)
}

pub async fn delete(db: &SqlitePool, id: i64) -> sqlx::Result<sqlx::sqlite::SqliteQueryResult> {
    Ok(
        query!("DELETE FROM comments WHERE id = ?", id)
            .execute(db)
            .await?
    )
}

pub async fn find_root(db: &SqlitePool, id: i64) -> sqlx::Result<Comment> {
    Ok(
        query_as!(
            Comment,
            r#"
                SELECT
                id, page_id, parent_id, name, body, avatar, replies_count, reviewed,
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>",
                token as "token: Base64"
                FROM comments WHERE parent_id IS NULL AND id = ?
            "#,
            id
        ).fetch_one(db).await?
    )
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
                    SELECT id, page_id, parent_id, name, body, avatar, replies_count,
                    reviewed, created_at, updated_at, token
                    FROM comments
                    WHERE page_id = ?
                    AND reviewed = 1
                    AND parent_id IS NULL
                    AND (created_at < ? OR (created_at = ? AND id < ?))
                    ORDER BY created_at DESC, id DESC
                    LIMIT ?
                "#,
             )
             .bind(page_id)
             .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
             .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
             .bind(cur.id)
             .bind(limit)
        },
        None => {
            query_as::<_, Comment>(
                r#"
                    SELECT id, page_id, parent_id, name, body, avatar, replies_count,
                    reviewed, created_at, updated_at, token
                    FROM comments
                    WHERE page_id = ?
                    AND reviewed = 1
                    AND parent_id IS NULL
                    ORDER BY created_at DESC, id DESC
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
            id, page_id, parent_id, name, body, avatar, replies_count,
            reviewed, created_at, updated_at, token
        FROM (
            SELECT
                r.id AS id,
                r.page_id AS page_id,
                r.parent_id AS parent_id,
                r.name AS name,
                r.body AS body,
                r.avatar AS avatar,
                r.replies_count AS replies_count,
                r.reviewed AS reviewed,
                r.created_at AS created_at,
                r.updated_at AS updated_at,
                r.token AS token,
                row_number() OVER (PARTITION BY c.id ORDER BY r.created_at, r.id) AS rn
            FROM comments c
            LEFT JOIN comments r
            ON r.parent_id = c.id
            WHERE r.reviewed = 1
        )
        WHERE parent_id IN({ids})
        AND id NOT NULL
        AND rn <= {limit}
        ORDER BY created_at, id;
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
                    SELECT id, page_id, parent_id, name, body, avatar, replies_count,
                    reviewed, created_at, updated_at, token
                    FROM comments
                    WHERE parent_id = ?
                    AND reviewed = 1
                    AND (created_at > ? OR (created_at = ? AND id > ?))
                    ORDER BY created_at, id
                    LIMIT ?
                 "#,
             )
             .bind(parent_id)
             .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
             .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
             .bind(cur.id)
             .bind(limit)
        },
        None => {
            query_as::<_, Comment>(
               r#"
                    SELECT id, page_id, parent_id, name, body, avatar, replies_count,
                    reviewed, created_at, updated_at, token
                    FROM comments
                    WHERE parent_id = ?
                    AND reviewed = 1
                    ORDER BY created_at, id
                    LIMIT ?
                "#,
            )
            .bind(parent_id)
            .bind(limit)
        }
    };

    Ok(query.fetch_all(db).await?)
}

pub async fn create(
    db: &SqlitePool,
    page_id: i64,
    parent_id: Option<i64>,
    name: &str,
    body: &str,
    avatar: &Option<&String>,
    reviewed: bool,
    token: &Base64,
) -> sqlx::Result<Comment> {
    let mut tx = db.begin().await?;

    let comment = query_as::<_, Comment>(
            r#"
                INSERT INTO comments (page_id, parent_id, name, body, avatar, reviewed, token)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                RETURNING *
            "#
        )
        .bind(page_id)
        .bind(parent_id)
        .bind(name)
        .bind(body)
        .bind(avatar)
        .bind(reviewed)
        .bind(token)
        .fetch_one(&mut tx)
        .await?;

    if reviewed {
        query!("UPDATE pages SET comments_count = comments_count + 1 WHERE id = ?", page_id)
            .execute(&mut tx)
            .await?;
        if let Some(pid) = parent_id {
            query!("UPDATE comments SET replies_count = replies_count + 1 WHERE id = ?", pid)
                .execute(&mut tx)
                .await?;
        }
    }

    tx.commit().await?;

    Ok(comment)
}
