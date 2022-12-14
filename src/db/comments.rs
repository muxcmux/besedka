use chrono::{DateTime, Utc};
use sqlx::{SqlitePool, FromRow, query_as, query, Row};

use crate::api::{Base64, Result, Cursor};

use super::UTC_DATETIME_FORMAT;

#[derive(FromRow, Clone, Debug)]
pub struct Comment {
    pub id: i64,
    pub page_id: i64,
    pub parent_id: Option<i64>,
    pub avatar: Option<String>,
    pub name: String,
    pub html_body: String,
    pub body: String,
    pub reviewed: bool,
    pub moderator: bool,
    pub op: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub token: Base64,
}

pub async fn find(db: &SqlitePool, id: i64) -> sqlx::Result<Comment> {
    Ok(
        query_as!(
            Comment,
            r#"
                SELECT
                id, page_id, parent_id, avatar, name,
                html_body, body, reviewed, moderator, op,
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
                id, page_id, parent_id, avatar, name,
                html_body, body, reviewed, moderator, op,
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
        SELECT
        id, page_id, parent_id, avatar, name,
        html_body, body, reviewed, moderator, op,
        created_at, updated_at, token
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

pub async fn replies(
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
                id, page_id, parent_id, avatar, name,
                html_body, body, reviewed, moderator, op,
                created_at, updated_at, token
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

pub async fn create(
    db: &SqlitePool,
    page_id: i64,
    parent_id: Option<i64>,
    avatar: &Option<&String>,
    name: &str,
    html_body: &str,
    body: &str,
    reviewed: bool,
    op: bool,
    moderator: bool,
    token: &Base64,
) -> sqlx::Result<Comment> {
    let mut tx = db.begin().await?;

    let comment = query_as::<_, Comment>(
            r#"
                INSERT INTO comments
                (page_id, parent_id, avatar, name, html_body, body, reviewed, op, moderator, token)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING *
            "#
        )
        .bind(page_id)
        .bind(parent_id)
        .bind(avatar)
        .bind(name)
        .bind(html_body)
        .bind(body)
        .bind(reviewed)
        .bind(op)
        .bind(moderator)
        .bind(token)
        .fetch_one(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(comment)
}

pub async fn update(
    db: &SqlitePool,
    id: i64,
    html_body: &str,
    body: &str,
) -> sqlx::Result<Comment> {
    let mut tx = db.begin().await?;

    let comment = query_as::<_, Comment>(
            r#"
                UPDATE comments
                SET
                html_body = ?,
                body = ?,
                updated_at = (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                WHERE id = ?
                RETURNING *
            "#
        )
        .bind(html_body)
        .bind(body)
        .bind(id)
        .fetch_one(&mut tx)
        .await?;

    tx.commit().await?;

    Ok(comment)
}
