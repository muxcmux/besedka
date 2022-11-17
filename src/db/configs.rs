// Some db related operations shared between cli and api

use anyhow::bail;
use sqlx::{SqlitePool, FromRow, query_as, query};
use serde::Serialize;

use crate::cli::ConfigSetCommand;

#[derive(FromRow, Debug, Serialize)]
pub struct Config {
    pub site: String,
    pub secret: Vec<u8>,
    pub private: bool,
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

/// Creates a new configration for a site from
/// command line arguments and returns the result
pub async fn insert(db: &SqlitePool, args: ConfigSetCommand) -> anyhow::Result<Config> {
    fn append<T>(value: &Option<T>, attribute: &str, query: &mut String, values: &mut String) {
        if let Some(_) = value {
            query.push_str(&format!(", {}", attribute));
            values.push_str(", ?");
        }
    }

    let mut insert = String::from("INSERT INTO configs (site");
    let mut values = String::from("VALUES (?");

    append(&args.private, "private", &mut insert, &mut values);
    append(&args.anonymous_comments, "anonymous_comments", &mut insert, &mut values);
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
    if let Some(a) = args.anonymous_comments { result = result.bind(a) }
    if let Some(a) = args.moderated { result = result.bind(a) }
    if let Some(a) = args.comments_per_page { result = result.bind(a) }
    if let Some(a) = args.replies_per_comment { result = result.bind(a) }
    if let Some(a) = args.minutes_to_edit { result = result.bind(a) }
    if let Some(a) = args.theme { result = result.bind(a) }

    result = result.bind(&args.site);

    result.execute(db).await?;

    Ok(find(db, &args.site).await?.unwrap())
}

/// Updates a configuration for a given site from
/// command line arguments and returns the updated row
pub async fn update(db: &SqlitePool, existing: Config, args: ConfigSetCommand) -> anyhow::Result<Config> {
    let mut update = String::from("UPDATE configs SET");

    if let Some(_) = args.private { update.push_str(" private = ?") };
    if let Some(_) = args.anonymous_comments { update.push_str(" anonymous_comments = ?") };
    if let Some(_) = args.moderated { update.push_str(" moderated = ?") };
    if let Some(_) = args.comments_per_page { update.push_str(" comments_per_page = ?") };
    if let Some(_) = args.replies_per_comment { update.push_str(" replies_per_comment = ?") };
    if let Some(_) = args.minutes_to_edit { update.push_str(" minutes_to_edit = ?") };
    if let Some(_) = args.theme { update.push_str(" theme = ?") };

    update.push_str(" WHERE site = ?");

    let mut result = query(&update);

    if let Some(a) = args.private { result = result.bind(a) }
    if let Some(a) = args.anonymous_comments { result = result.bind(a) }
    if let Some(a) = args.moderated { result = result.bind(a) }
    if let Some(a) = args.comments_per_page { result = result.bind(a) }
    if let Some(a) = args.replies_per_comment { result = result.bind(a) }
    if let Some(a) = args.minutes_to_edit { result = result.bind(a) }
    if let Some(a) = args.theme { result = result.bind(a) }

    result = result.bind(existing.site);

    result.execute(db).await?;

    Ok(find(db, &args.site).await?.unwrap())
}
