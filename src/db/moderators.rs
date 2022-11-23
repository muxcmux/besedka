use sqlx::{query_as, SqlitePool, FromRow, query};
use serde::Serialize;
use crate::{cli::ModeratorsAddCommandArgs, api::Result};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2
};

#[derive(FromRow, Debug, Serialize)]
pub struct Moderator {
  pub name: String,
  pub password: String,
  pub password_salt: String,
  pub avatar: Option<String>,
  pub sid: Option<Vec<u8>>,
}

/// Returns all moderators for a given site
pub async fn all(db: &SqlitePool) -> anyhow::Result<Vec<Moderator>> {
    let users = query_as!(Moderator, "SELECT * FROM moderators")
        .fetch_all(db).await?;
    Ok(users)
}

/// Inserts a moderator and returns the newly inserted row.
/// Password is hashed with Argon2 before saving
pub async fn insert_moderator(db: &SqlitePool, moderator: ModeratorsAddCommandArgs) -> sqlx::Result<Moderator> {
    let password = moderator.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    Ok(
        query_as::<_, Moderator>(
            r#"
                INSERT INTO moderators (name, password, password_salt, avatar)
                VALUES(?, ?, ?, ?);
                SELECT * FROM moderators WHERE name = ? LIMIT 1
            "#,
        )
        .bind(&moderator.name)
        .bind(&password_hash)
        .bind(salt.as_str())
        .bind(&moderator.avatar)
        .bind(&moderator.name)
        .fetch_one(db)
        .await?
    )
}

pub async fn find_by_sid(db: &SqlitePool, sid: &Vec<u8>) -> Result<Moderator> {
    Ok(
        query_as!(Moderator, "SELECT * FROM moderators WHERE sid = ? LIMIT 1", sid)
            .fetch_one(db)
            .await?
    )
}

pub async fn find_by_name(db: &SqlitePool, name: &str) -> Result<Moderator> {
    Ok(
        query_as!(Moderator, "SELECT * FROM moderators WHERE name = ? LIMIT 1", name)
            .fetch_one(db)
            .await?
    )
}

pub async fn delete(db: &SqlitePool, name: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    query!("DELETE FROM moderators WHERE name = ?", name)
        .execute(db)
        .await
}
