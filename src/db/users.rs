use sqlx::{query, query_as, SqlitePool, FromRow};
use serde::Serialize;
use crate::cli::ModeratorsAddCommand;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

#[derive(FromRow, Debug, Serialize)]
pub struct User {
  pub id: i64,
  pub site: String,
  pub username: String,
  pub name: String,
  pub password: Option<String>,
  pub password_salt: Option<String>,
  pub moderator: bool,
  pub third_party_id: Option<String>,
  pub avatar: Option<String>
}

/// Returns all moderators for a given site
pub async fn all(db: &SqlitePool, site: &str) -> anyhow::Result<Vec<User>> {
    let users = query_as!(User, "SELECT * FROM users WHERE moderator = ? AND site = ?", true, site)
        .fetch_all(db).await?;
    Ok(users)
}

/// Finds a moderator by a username for a given site
pub async fn find_moderator_by_username(db: &SqlitePool, site: &str, username: &str) -> anyhow::Result<Option<User>> {
    let user = query_as!(
            User,
            "SELECT * FROM users WHERE(moderator = ? AND site = ? AND username = ?)"
            , true
            , site
            , username
        ).fetch_optional(db).await?;
    Ok(user)
}

/// Inserts a new moderator for a site and returns
/// the newly inserted row.
/// Password is hashed with Argon2 before saving
pub async fn insert_moderator(db: &SqlitePool, moderator: ModeratorsAddCommand, site: &str) -> anyhow::Result<Option<User>> {
    let password = moderator.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    query(r#"
            INSERT INTO users (site, username, name, password, password_salt, moderator, avatar)
            VALUES(?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(site)
        .bind(&moderator.username)
        .bind(moderator.name)
        .bind(password_hash)
        .bind(salt.as_str())
        .bind(true)
        .bind(moderator.avatar)
        .fetch_optional(db).await?;
    find_moderator_by_username(db, &site, &moderator.username).await
}
