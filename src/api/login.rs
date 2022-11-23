use axum::{response::IntoResponse, Router, routing::post};

pub fn router() -> Router {
    Router::new()
        .route("/api/login", post(login))
}

pub async fn login() -> impl IntoResponse {
    todo!()
}
