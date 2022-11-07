mod error;
pub mod comments;
use std::sync::Arc;

use axum::Extension;
use sqlx::SqlitePool;

pub struct AppContext {
    pub db: SqlitePool
}

pub type Context = Extension<Arc<AppContext>>;

pub use error::{Error, ResultExt};
pub type Result<T, E = Error> = std::result::Result<T, E>;
