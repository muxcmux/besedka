use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, Router, routing::post};
use serde::Deserialize;

use crate::{
    db::moderators::find_by_name,
    api::{Error, Context, Result},
};

use super::generate_random_token;

pub fn router() -> Router {
    Router::new()
        .route("/api/login", post(login))
}

#[derive(Deserialize)]
struct LoginRequest {
    name: String,
    password: String,
}

async fn login(
    ctx: Context,
    Json(req): Json<LoginRequest>,
) -> Result<String> {
    let moderator = find_by_name(&ctx.db, &req.name)
        .await
        .map_err(|_| Error::Unauthorized)?;

    let hash = PasswordHash::new(&moderator.password)
        .map_err(|_| Error::Unauthorized)?;

    Argon2::default()
        .verify_password(req.password.as_bytes(), &hash)
        .map_err(|_| Error::Unauthorized)?;

    let sid = generate_random_token();
    moderator.set_sid(&ctx.db, &sid).await;

    Ok(sid.into())
}
