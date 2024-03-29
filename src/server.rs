mod assets;

use std::time::Duration;

use anyhow::Context;

use axum::{
    routing::get,
    Router, response::IntoResponse, body::Bytes
};

use sqlx::SqlitePool;
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
    LatencyUnit, timeout::TimeoutLayer, compression::CompressionLayer, cors::CorsLayer,
};

use crate::api::{self, AppState};
use super::cli::ServerArgs;

use axum_server::tls_rustls::RustlsConfig;

impl ServerArgs {
    pub fn ssl(&self) -> bool {
        self.ssl_key.is_some() && self.ssl_cert.is_some()
    }
}

/// Runs the server, blocking the main thread
pub async fn run(config: ServerArgs, db: SqlitePool) -> anyhow::Result<()> {
    tracing::debug!("{:#?}", config);
    tracing::info!("Listening on {}", config.bind);

    let app = router(db);

    if config.ssl() {
        let ssl_config = RustlsConfig::from_pem_file(
            config.ssl_cert.unwrap(),
            config.ssl_key.unwrap()
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

fn router(db: SqlitePool) -> Router {
    let state = AppState { db };

    let middleware = ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
                    tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
                })
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        )
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .layer(CorsLayer::permissive());

    Router::new()
        .route("/", get(root))
        .merge(api::login::router())
        .merge(api::comments::router())
        .merge(api::preview::router())
        .merge(api::sites::router())
        .merge(api::pages::router())
        .merge(assets::router())
        .layer(middleware)
        .with_state(state)
}

async fn root() -> impl IntoResponse {
    String::from("Hello from Besedka!")
}
