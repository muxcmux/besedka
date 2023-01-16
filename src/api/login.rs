use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, Router, routing::post, extract::State};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    db::moderators::{find_by_name, Moderator},
    api::{Error, AppState, Result},
};

use super::generate_random_token;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/login", post(login))
}

#[derive(Deserialize)]
struct LoginRequest {
    name: String,
    password: String,
}

async fn login(
    State(db): State<SqlitePool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<Moderator>> {
    let mut moderator = find_by_name(&db, &req.name)
        .await
        .map_err(|_| Error::Unauthorized)?;

    let hash = PasswordHash::new(&moderator.password)
        .map_err(|_| Error::Unauthorized)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &hash)
        .map_err(|_| Error::Unauthorized)?;

    let sid = generate_random_token();
    moderator.set_sid(&db, sid).await;

    Ok(Json(moderator))
}
