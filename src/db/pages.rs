use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Page {
  pub id: i64,
  pub site: String,
  pub path: String,
  pub comments_count: i64,
  pub locked_at: Option<NaiveDateTime>
}

