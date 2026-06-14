// SPDX-License-Identifier: Apache-2.0
//! Zyvor VM Services API

use anyhow::Result;
use tracing_subscriber::EnvFilter;
use zyvor_api::{config::Config, serve};

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("failed to install rustls crypto provider"))?;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("zyvor_api=info".parse()?))
        .init();

    let config = Config::from_env()?;
    serve(config).await
}
