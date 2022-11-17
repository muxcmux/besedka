use sqlx::{query, query_as, SqlitePool, FromRow};
use serde::Serialize;
use crate::{cli::ModeratorsAddCommand, api::{RequestedUserConfig, extractors::AuthenticatedModerator}};
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
  pub sid: Option<String>,
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

pub async fn find_moderator_by_sid(db: &SqlitePool, sid: &str) -> sqlx::Result<User> {
    Ok(
        query_as!(User, "SELECT * FROM users WHERE moderator = 1 AND sid = ? LIMIT 1", sid)
            .fetch_one(db)
            .await?
    )
}

pub async fn get_commenter(
    db: &SqlitePool,
    site: &str,
    authenticated_user: Option<RequestedUserConfig>,
    moderator: Option<AuthenticatedModerator>,
) -> sqlx::Result<Option<User>> {
    Ok(
        match moderator {
            Some(m) => Some(m),
            None => {
                match authenticated_user {
                    None => None,
                    Some(ref u) => {
                        let mut tx = db.begin().await?;
                        let user = match query_as::<_, User>(
                                r#"
                                INSERT INTO users (site, username, name, moderator, third_party_id, avatar)
                                VALUES(?, ?, ?, ?, ?, ?) RETURNING * "#
                            )
                            .bind(site)
                            .bind(&u.username)
                            .bind(&u.name)
                            .bind(&u.moderator)
                            .bind(&u.id)
                            .bind(&u.avatar)
                            .fetch_optional(&mut tx)
                            .await {
                                Ok(inserted) => Ok(inserted),
                                Err(err) => match err {
                                    sqlx::Error::Database(e) if e.message().contains("UNIQUE") => {
                                        Ok(
                                            query_as!(
                                                User,
                                                r#"
                                                    UPDATE users SET username = ?, name = ?, moderator = ?, avatar = ?
                                                    WHERE site = ? AND third_party_id = ?;
                                                    SELECT * FROM users
                                                    WHERE site = ? AND third_party_id = ?
                                                    LIMIT 1
                                                "#,
                                                u.username,
                                                u.name,
                                                u.moderator,
                                                u.avatar,
                                                site,
                                                u.id,
                                                site,
                                                u.id
                                            )
                                            .fetch_optional(&mut tx)
                                            .await?
                                        )
                                    },
                                    _ => Err(err),
                                }
                            };
                        tx.commit().await?;
                        user?
                    }
                }
            }
        }
    )
}
