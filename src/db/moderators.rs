use sqlx::{query_as, SqlitePool, FromRow, query};
use serde::Serialize;
use crate::{cli::ModeratorsAddCommandArgs, api::{Result, Base64}};
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
    #[serde(skip_serializing)]
    pub password: String,
    pub op: bool,
    pub avatar: Option<String>,
    pub sid: Option<Base64>,
}

/// Returns all moderators for a given site
pub async fn all(db: &SqlitePool) -> anyhow::Result<Vec<Moderator>> {
    let users = query_as!(Moderator, r#"SELECT name, password, avatar, op, sid as "sid: Base64" FROM moderators"#)
        .fetch_all(db).await?;
    Ok(users)
}

impl Moderator {
    pub async fn set_sid(&mut self, db: &SqlitePool, sid: Base64) {
        let mut tx = db.begin().await.unwrap();
        let name_ref = &self.name;
        let sid_ref = &sid;
        query!("UPDATE moderators SET sid = ? WHERE name = ?", sid_ref, name_ref)
            .execute(&mut tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        self.sid = Some(sid);
    }
}

/// Inserts a moderator and returns the newly inserted row.
/// Password is hashed with Argon2 before saving
pub async fn insert_moderator(
    db: &SqlitePool,
    moderator: ModeratorsAddCommandArgs,
) -> sqlx::Result<Moderator> {
    let password = moderator.password.as_bytes();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password, &salt)
        .unwrap()
        .to_string();

    Ok(
        query_as::<_, Moderator>(
            r#"
                INSERT INTO moderators (name, password, avatar, op)
                VALUES(?, ?, ?, ?);
                SELECT * FROM moderators WHERE name = ? LIMIT 1
            "#,
        )
        .bind(&moderator.name)
        .bind(&password_hash)
        .bind(&moderator.avatar)
        .bind(&moderator.op.unwrap_or(false))
        .bind(&moderator.name)
        .fetch_one(db)
        .await?
    )
}

pub async fn find_by_sid(db: &SqlitePool, sid: &Base64) -> Result<Moderator> {
    Ok(
        query_as!(Moderator, r#"SELECT name, password, avatar, op, sid as "sid: Base64" FROM moderators WHERE sid = ? LIMIT 1"#, sid)
            .fetch_one(db)
            .await?
    )
}

pub async fn find_by_name(db: &SqlitePool, name: &str) -> Result<Moderator> {
    Ok(
        query_as!(Moderator, r#"SELECT name, password, avatar, op, sid as "sid: Base64" FROM moderators WHERE name = ? LIMIT 1"#, name)
            .fetch_one(db)
            .await?
    )
}

pub async fn delete(db: &SqlitePool, name: &str) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error> {
    query!("DELETE FROM moderators WHERE name = ?", name)
        .execute(db)
        .await
}

pub async fn update(db: &SqlitePool, name: &str, op: Option<bool>, avatar: Option<String>, password: Option<String>) -> Result<Moderator> {
    let mut q = String::from("UPDATE moderators SET name = ?");

    if op.is_some() { q.push_str(", op = ?") }
    if avatar.is_some() { q.push_str(", avatar = ?") }
    if password.is_some() { q.push_str(", password = ?") }

    q.push_str(" WHERE name = ?; SELECT * FROM moderators WHERE name = ? LIMIT 1");

    let mut query = query_as::<_, Moderator>(&q).bind(name);

    if op.is_some() { query = query.bind(op.unwrap()) }
    if avatar.is_some() { query = query.bind(avatar.unwrap()) }

    if password.is_some() {
        let pass = password.unwrap();
        let pwd = pass.as_bytes();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(pwd, &salt)
            .unwrap()
            .to_string();

        query = query.bind(password_hash);

        let _ = query!("UPDATE moderators SET sid = NULL WHERE name = ?", name).execute(db).await?;
    }

    query = query.bind(name);
    query = query.bind(name);

    Ok(
        query.fetch_one(db).await?
    )
}
