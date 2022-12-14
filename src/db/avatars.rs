use ring::digest::{Context, SHA256};
use serde::Serialize;
use sqlx::{query_as, SqlitePool, FromRow};
use crate::api::Result;

#[derive(Serialize, FromRow, Debug)]
    pub struct Avatar {
    pub id: i64,
    pub data: String,
    #[serde(skip_serializing)]
    pub sha: Vec<u8>,
}

pub async fn find_all_by_id(db: &SqlitePool, ids: Vec<i64>) -> Result<Vec<Avatar>> {
    let ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    let ids = ids.join(",");
    let q = format!("SELECT * FROM avatars WHERE id IN({})", ids);

    Ok(
        query_as::<_, Avatar>(&q)
            .fetch_all(db)
            .await?
    )
}

pub async fn find(db: &SqlitePool, id: i64) -> Result<Option<Avatar>> {
    Ok(
        query_as!(Avatar, "SELECT * FROM avatars WHERE id = ? LIMIT 1", id)
            .fetch_optional(db)
            .await?
    )
}

pub async fn find_or_create(db: &SqlitePool, data: &str) -> sqlx::Result<Avatar> {
    let mut context = Context::new(&SHA256);
    context.update(data.as_bytes());
    let digest = context.finish();
    let sha = digest.as_ref();

    let existing = query_as!(Avatar, "SELECT * FROM avatars WHERE sha = ? LIMIT 1", sha)
        .fetch_optional(db)
        .await?;

    if existing.is_some() { return Ok(existing.unwrap()) }

    Ok(
        query_as!(Avatar, "INSERT INTO avatars (sha, data) VALUES (?, ?); SELECT * FROM avatars WHERE sha = ? LIMIT 1", sha, data, sha)
            .fetch_one(db)
            .await?
    )
}
