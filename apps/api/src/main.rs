use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer};
use anyhow::{Context, Result};
use reqwest::Client;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::EnvFilter;

mod config;
mod decide;
mod error;
mod handlers;
mod laya;
mod stats;

use config::Config;
use handlers::AppState;
use laya::LayaClient;
use stats::Stats;

#[actix_web::main]
async fn main() -> Result<()> {
    init_tracing();
    let config = Config::from_env()?;
    let laya = LayaClient::new(&config)?;
    let http = Client::builder()
        .timeout(config.upstream_timeout)
        .build()
        .context("cliente HTTP upstream")?;
    let stats = Arc::new(Stats::new(config.max_events, config.routing_enabled));

    let state = web::Data::new(AppState {
        config: config.clone(),
        laya,
        http,
        stats,
    });

    let bind = config.bind_address.clone();
    tracing::info!(%bind, laya = %config.laya_url, "laya-gateway listening (loopback only)");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .wrap(TracingLogger::default())
            .route("/", web::get().to(handlers::dashboard_page))
            .route("/dashboard", web::get().to(handlers::dashboard_page))
            .route("/img/logo/aios.jpeg", web::get().to(handlers::logo_aios))
            .route("/img/logo/logo-light-full.jpeg", web::get().to(handlers::logo_aios))
            .route("/img/logo/logo-light-streamline.jpeg", web::get().to(handlers::logo_aios))
            .route("/health/live", web::get().to(handlers::health_live))
            .route("/health/ready", web::get().to(handlers::health_ready))
            .route("/v1/health/live", web::get().to(handlers::health_live))
            .route("/v1/health/ready", web::get().to(handlers::health_ready))
            .route("/v1/stats", web::get().to(handlers::get_stats))
            .route("/v1/routing", web::post().to(handlers::set_routing))
            .route("/v1/settings", web::get().to(handlers::get_settings))
            .route("/v1/settings", web::post().to(handlers::set_settings))
            .route("/v1/settings/test", web::post().to(handlers::test_laya))
            .route(
                "/v1/chat/completions",
                web::post().to(handlers::chat_completions),
            )
            // OpenAI-compatible alias (some clients hit /chat/completions on the base URL)
            .route(
                "/chat/completions",
                web::post().to(handlers::chat_completions),
            )
    })
    .bind(&bind)
    .with_context(|| format!("bind em {bind}"))?
    .shutdown_timeout(5)
    .run()
    .await
    .context("servidor HTTP")
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("laya_gateway=info,actix_web=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_target(false)
        .init();
}
