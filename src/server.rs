use std::sync::Arc;

use anyhow::Context;

use axum::{
    routing::get,
    Router, response::IntoResponse, Extension
};

use sqlx::SqlitePool;
use tower_http::{trace::TraceLayer, compression::CompressionLayer};
use crate::api;

use super::cli::Server;
use axum_server::tls_rustls::RustlsConfig;

impl Server {
    pub fn ssl(&self) -> bool {
        self.ssl_key.is_some() && self.ssl_cert.is_some()
    }
}

/// Runs the server, blocking the main thread
pub async fn run(config: Server, db: SqlitePool) -> anyhow::Result<()> {
    tracing::debug!("{:#?}", config);
    tracing::info!("Listening on {}", config.bind);

    let app = router(build_context(db));

    if config.ssl() {
        let ssl_config = RustlsConfig::from_pem_file(
            config.ssl_key.unwrap(),
            config.ssl_cert.unwrap()
        ).await.unwrap();
        axum_server::bind_rustls(config.bind, ssl_config)
            .serve(app.into_make_service())
            .await
            .context("Failed running HTTPs server")
    } else {
        axum_server::bind(config.bind)
            .serve(app.into_make_service())
            .await
            .context("Failed running HTTP server")
    }
}

fn router(context: Arc<api::AppContext>) -> Router {
    Router::new()
        .route("/", get(root))
        .merge(api::comments::router())
        .layer(Extension(context))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

fn build_context(db: SqlitePool) -> Arc<api::AppContext> {
    Arc::new(api::AppContext { db })
}

async fn root() -> impl IntoResponse {
    String::from("<!DOCTYPE html><html><body>\
        <p>Hello besedka!</p>\
    </body></html>")
}
