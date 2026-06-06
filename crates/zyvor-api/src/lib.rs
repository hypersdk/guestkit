// SPDX-License-Identifier: Apache-2.0
//! Zyvor VM Services API library

pub mod config;
pub mod db;
pub mod error;
pub mod jobs;
pub mod models;
pub mod routes;
pub mod state;

use anyhow::Result;
use axum::Router;
use config::Config;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub async fn serve(config: Config) -> Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    db::migrate(&pool).await?;

    let redis = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis).await?;

    tokio::fs::create_dir_all(&config.storage_path).await?;

    let state = AppState {
        pool,
        redis: redis_conn,
        config: config.clone(),
    };

    let app = Router::new()
        .merge(routes::api_router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = config.bind_addr.parse()?;
    tracing::info!("zyvor-api listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
