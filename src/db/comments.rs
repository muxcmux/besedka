use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{SqlitePool, FromRow, query_as, query, Row};

use crate::api::{Base64, Result, Cursor};

use super::UTC_DATETIME_FORMAT;

#[derive(FromRow, Clone, Debug, Serialize)]
pub struct Comment {
    pub id: i64,
    #[serde(skip_serializing)]
    pub page_id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub body: String,
    pub avatar: Option<String>,
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
                id, page_id, parent_id, name, body, avatar, reviewed,
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>",
                token as "token: Base64"
                FROM comments WHERE id = ?
            "#,
            id
        ).fetch_one(db).await?
    )
}

pub async fn approve(db: &SqlitePool, id: i64) -> sqlx::Result<()> {
    let mut tx = db.begin().await?;

    let _ = query(
        "UPDATE comments SET reviewed = 1 WHERE id = ?"
    ).bind(id).execute(&mut tx).await?;

    tx.commit().await?;

    Ok(())
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
                id, page_id, parent_id, name, body, avatar, reviewed,
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
    reviewed_only: bool,
    token: &Option<Base64>,
    cursor: Option<Cursor>,
) -> Result<(i64, Vec<Comment>)> {
    let mut select = String::from(r#"
        SELECT id, page_id, parent_id, name, body, avatar,
        reviewed, created_at, updated_at, token
    "#);

    let mut count = String::from("SELECT count(*)");

    let common = " FROM comments WHERE page_id = ? ";

    select.push_str(common);
    select.push_str(" AND parent_id IS NULL ");
    count.push_str(common);
    count.push_str(" AND parent_id IS NULL ");

    if reviewed_only {
        if token.is_some() {
            select.push_str(" AND (reviewed = 1 OR token = ?) ");
            count.push_str(" AND (reviewed = 1 OR token = ?) ");
        } else {
            select.push_str(" AND reviewed = 1 ");
            count.push_str(" AND reviewed = 1 ");
        }
    }

    let results = match cursor {
        Some(cur) => {
            select.push_str(r#"
                AND (created_at < ? OR (created_at = ? AND id < ?))
                ORDER BY created_at DESC, id DESC
                LIMIT ?
            "#);

            let mut results = query_as::<_, Comment>(&select).bind(page_id);

            if reviewed_only {
                if let Some(t) = token { results = results.bind(t) }
            }

            results
                .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
                .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
                .bind(cur.id)
                .bind(limit)
        },
        None => {
            select.push_str(r#"
                ORDER BY created_at DESC, id DESC
                LIMIT ?
            "#);

            let mut results = query_as::<_, Comment>(&select).bind(page_id);

            if reviewed_only {
                if let Some(t) = token { results = results.bind(t) }
            }

            results.bind(limit)
        }
    };

    let mut total = query(&count).bind(page_id);

    if reviewed_only {
        if let Some(t) = token { total = total.bind(t) }
    }

    Ok((total.fetch_one(db).await?.get(0), results.fetch_all(db).await?))
}

pub async fn nested_replies(
    db: &SqlitePool,
    reviewed_only: bool,
    token: &Option<Base64>,
    parents: &Vec<Comment>,
) -> Result<Vec<Comment>> {
    let parent_ids: Vec<String> = parents.iter().map(|p| p.id.to_string()).collect();
    let ids = parent_ids.join(",");

    let mut condition = "".to_string();
    if reviewed_only {
        if token.is_some() {
            condition.push_str("AND (reviewed = 1 OR token = ?)");
        } else {
            condition.push_str("AND reviewed = 1");
        }
    }

    let query = format!(
        r#"
            SELECT
                id, page_id, parent_id, name, body, avatar,
                reviewed, created_at, updated_at, token
            FROM comments
            WHERE parent_id IN({ids})
            {condition}
            ORDER BY created_at, id
        "#,
        condition = condition,
        ids = ids,
    );

    let mut results = query_as::<_, Comment>(&query);

    if reviewed_only {
        if let Some(t) = token { results = results.bind(t) }
    }

    Ok(results.fetch_all(db).await?)
}

pub async fn replies(
    db: &SqlitePool,
    parent_id: i64,
    limit: i64,
    cursor: Option<Cursor>,
    reviewed_only: bool,
    token: &Option<Base64>,
) -> Result<Vec<Comment>> {

    let mut condition = "".to_string();
    if reviewed_only {
        if token.is_some() {
            condition.push_str("AND (reviewed = 1 OR token = ?)");
        } else {
            condition.push_str("AND reviewed = 1");
        }
    }

    let mut _q = String::from("");

    let mut query = match cursor {
        Some(cur) => {
            _q = format!(
                r#"
                    SELECT id, page_id, parent_id, name, body, avatar,
                    reviewed, created_at, updated_at, token
                    FROM comments
                    WHERE parent_id = ?
                    AND (created_at > ? OR (created_at = ? AND id > ?))
                    {condition}
                    ORDER BY created_at, id
                    LIMIT ?
                "#,
                condition = condition
            );
            query_as::<_, Comment>(&_q)
                .bind(parent_id)
                .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
                .bind(format!("{}", cur.created_at.format(UTC_DATETIME_FORMAT)))
                .bind(cur.id)
        },
        None => {
            _q = format!(
                r#"
                    SELECT id, page_id, parent_id, name, body, avatar,
                    reviewed, created_at, updated_at, token
                    FROM comments
                    WHERE parent_id = ?
                    {condition}
                    ORDER BY created_at, id
                    LIMIT ?
                "#,
                condition = condition
            );
            query_as::<_, Comment>(&_q)
                .bind(parent_id)
        }
    };

    if reviewed_only {
        if let Some(t) = token { query = query.bind(t) }
    }

    Ok(query.bind(limit).fetch_all(db).await?)
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

    tx.commit().await?;

    Ok(comment)
}
