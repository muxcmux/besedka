mod error;
pub mod comments;
pub mod extractors;
use std::sync::Arc;

use axum::Extension;
use chrono::{NaiveDateTime, DateTime, Utc};
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;

pub struct AppContext {
    pub db: SqlitePool
}

pub type Context = Extension<Arc<AppContext>>;

pub use error::{Error, ResultExt};
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Serialize)]
pub struct Page {
    pub id: i64,
    pub site: String,
    pub path: String,
    pub comments_count: i64,
    pub locked_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursor {
    pub id: i64,
    pub created_at: DateTime<Utc>,
}

impl Cursor {
    fn encode(&self) -> String {
        base64::encode(serde_json::to_vec(&self).unwrap())
    }
}
