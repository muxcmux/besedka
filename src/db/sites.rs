use ring::hmac;
use sqlx::{SqlitePool, FromRow, query_as, query};
use serde::Serialize;

use crate::cli::SitesCommandArgs;

#[derive(FromRow, Debug, Serialize)]
pub struct Site {
    pub site: String,
    #[serde(skip_serializing)]
    pub secret: Vec<u8>,
    pub private: bool,
    pub anonymous: bool,
    pub moderated: bool,
    pub comments_per_page: i64,
    pub replies_per_comment: i64,
    pub minutes_to_edit: i64,
    pub theme: String,
}

impl Site {
    pub fn secret(&self) -> String {
        base64::encode(&self.secret)
    }

    pub fn key(&self) -> hmac::Key {
        hmac::Key::new(hmac::HMAC_SHA256, &self.secret)
    }
}

pub async fn all(db: &SqlitePool) -> sqlx::Result<Vec<Site>> {
    let sites = query_as!(Site, "SELECT * FROM sites")
        .fetch_all(db)
        .await?;
    Ok(sites)
}

/// Finds a config for a given site
pub async fn find(db: &SqlitePool, site: &str) -> sqlx::Result<Site> {
    Ok(query_as!(Site, "SELECT * FROM sites WHERE site = ? LIMIT 1", site)
        .fetch_one(db).await?)
}

/// Deletes a config for a site
pub async fn delete(db: &SqlitePool, site: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    query!("DELETE FROM sites WHERE site = ?", site)
        .execute(db)
        .await
}

/// Creates a new configration for a site from
/// command line arguments and returns the result
pub async fn insert(db: &SqlitePool, args: &SitesCommandArgs) -> sqlx::Result<Site> {
    fn append<T>(value: &Option<T>, attribute: &str, query: &mut String, values: &mut String) {
        if let Some(_) = value {
            query.push_str(&format!(", {}", attribute));
            values.push_str(", ?");
        }
    }

    let mut insert = String::from("INSERT INTO sites (site");
    let mut values = String::from("VALUES (?");

    append(&args.private, "private", &mut insert, &mut values);
    append(&args.anonymous, "anonymous", &mut insert, &mut values);
    append(&args.moderated, "moderated", &mut insert, &mut values);
    append(&args.comments_per_page, "comments_per_page", &mut insert, &mut values);
    append(&args.replies_per_comment, "replies_per_comment", &mut insert, &mut values);
    append(&args.minutes_to_edit, "minutes_to_edit", &mut insert, &mut values);
    append(&args.theme, "theme", &mut insert, &mut values);

    insert.push_str(") ");
    values.push_str(")");
    insert.push_str(&values);

    let mut result = query(&insert);

    result = result.bind(&args.site);

    if let Some(a) = args.private { result = result.bind(a) }
    if let Some(a) = args.anonymous { result = result.bind(a) }
    if let Some(a) = args.moderated { result = result.bind(a) }
    if let Some(a) = args.comments_per_page { result = result.bind(a) }
    if let Some(a) = args.replies_per_comment { result = result.bind(a) }
    if let Some(a) = args.minutes_to_edit { result = result.bind(a) }
    if let Some(ref a) = args.theme { result = result.bind(a) }

    result = result.bind(&args.site);

    result.execute(db).await?;

    Ok(find(db, &args.site).await?)
}

/// Updates a configuration for a given site from
/// command line arguments and returns the updated row
pub async fn update(db: &SqlitePool, existing: Site, args: SitesCommandArgs) -> sqlx::Result<Site> {
    let mut update = String::from("UPDATE sites SET site = ?");

    if let Some(_) = args.private { update.push_str(", private = ?") };
    if let Some(_) = args.anonymous { update.push_str(", anonymous = ?") };
    if let Some(_) = args.moderated { update.push_str(", moderated = ?") };
    if let Some(_) = args.comments_per_page { update.push_str(", comments_per_page = ?") };
    if let Some(_) = args.replies_per_comment { update.push_str(", replies_per_comment = ?") };
    if let Some(_) = args.minutes_to_edit { update.push_str(", minutes_to_edit = ?") };
    if let Some(_) = args.theme { update.push_str(", theme = ?") };

    update.push_str(" WHERE site = ?");

    let mut result = query(&update);

    result = result.bind(&args.site);

    if let Some(a) = args.private { result = result.bind(a) }
    if let Some(a) = args.anonymous { result = result.bind(a) }
    if let Some(a) = args.moderated { result = result.bind(a) }
    if let Some(a) = args.comments_per_page { result = result.bind(a) }
    if let Some(a) = args.replies_per_comment { result = result.bind(a) }
    if let Some(a) = args.minutes_to_edit { result = result.bind(a) }
    if let Some(a) = args.theme { result = result.bind(a) }

    result = result.bind(&existing.site);

    result.execute(db).await?;

    Ok(find(db, &existing.site).await?)
}
