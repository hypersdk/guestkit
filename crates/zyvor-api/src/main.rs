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

    // guestkit (linked in for offline disk operations like provision's
    // run_migrate_plan) logs through the plain `log` facade, not `tracing`
    // — without this bridge those records have no installed logger at all
    // and go nowhere, silently, regardless of RUST_LOG.
    tracing_log::LogTracer::init()
        .map_err(|e| anyhow::anyhow!("failed to install log-to-tracing bridge: {e}"))?;

    // EnvFilter::from_default_env() alone yields "nothing enabled" when
    // RUST_LOG is unset, which combined with the "zyvor_api=info" directive
    // meant every other target — including anything bridged from `log`,
    // e.g. guestkit's own diagnostics — was silently dropped by default.
    // Falls back to "warn" globally (plus zyvor_api at info) only when the
    // operator hasn't set RUST_LOG themselves.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("warn,zyvor_api=info")),
        )
        .init();

    let config = Config::from_env()?;
    serve(config).await
}
