// Some db related operations shared between cli and api

use anyhow::bail;
use sqlx::{SqlitePool, FromRow, query_as};
use serde::Serialize;

#[derive(FromRow, Debug, Serialize)]
pub struct Config {
    pub site: String,
    pub secret: Vec<u8>,
    pub anonymous_comments: bool,
    pub moderated: bool,
    pub comments_per_page: i64,
    pub replies_per_comment: i64,
    pub minutes_to_edit: i64,
    pub theme: String,
}

impl Config {
    pub fn secret(&self) -> String {
        base64::encode(&self.secret)
    }
}

/// Finds a config for a given site
pub async fn find(db: &SqlitePool, site: &str) -> anyhow::Result<Option<Config>> {
    let config = query_as!(Config, "SELECT * FROM configs WHERE(site = ?)", site)
        .fetch_optional(db).await?;
    Ok(config)
}

/// Finds a config for a requested site.
/// If it doesn't exist, it returns the default config.
/// Returns an error if the default config is missing
pub async fn find_or_default(db: &SqlitePool, site: &str) -> anyhow::Result<Config> {
    let config = find(db, site).await?;
    if let Some(config) = config { return Ok(config) }
    if let Some(default) = find(db, "default").await? { return Ok(default) }
    bail!("Default config not found")
}
