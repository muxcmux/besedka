use serde::Serialize;
use sqlx::{query_as, SqlitePool, FromRow};

#[derive(FromRow, Debug, Serialize)]
pub struct Page {
  pub id: i64,
  pub site: String,
  pub path: String,
  pub comments_count: i64,
  pub locked: bool
}

pub async fn find_by_site_and_path(db: &SqlitePool, site: &str, path: &str) -> sqlx::Result<Page> {
    Ok(
        query_as!(
            Page,
            "SELECT * FROM pages WHERE site = ? AND path = ? LIMIT 1",
            site,
            path
        )
        .fetch_one(db)
        .await?
    )
}

pub async fn create_or_find_by_site_and_path(db: &SqlitePool, site: &str, path: &str) -> sqlx::Result<Page> {
    let mut tx = db.begin().await?;

    let page = match query_as::<_, Page>("INSERT INTO pages (site, path) VALUES(?, ?) RETURNING * ")
        .bind(site)
        .bind(path)
        .fetch_one(&mut tx)
        .await {
            Ok(page) => Ok(page),
            Err(err) => match err {
                sqlx::Error::Database(e) if e.message().contains("UNIQUE") => {
                    Ok(
                        query_as!(
                            Page,
                            "SELECT * FROM pages WHERE site = ? AND path = ? LIMIT 1",
                            site,
                            path,
                        )
                        .fetch_one(&mut tx)
                        .await?
                    )
                },
                _ => Err(err),
            }
        };

    tx.commit().await?;

    page
}
