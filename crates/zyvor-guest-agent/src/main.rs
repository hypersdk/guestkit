// SPDX-License-Identifier: Apache-2.0
//! Standalone Zeus VM Tools guest agent (cross-compiles to Windows).

mod daemon;
mod handler;

use anyhow::{bail, Context, Result};
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(windows)]
mod service;

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--service") {
        #[cfg(windows)]
        {
            return service::run_service();
        }
        #[cfg(not(windows))]
        bail!("--service is supported on Windows only");
    }

    let channel = args
        .iter()
        .position(|a| a == "--channel")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
        .unwrap_or(default_channel());

    daemon::run(channel)
}

fn default_channel() -> &'static str {
    #[cfg(windows)]
    {
        "stdio"
    }
    #[cfg(not(windows))]
    {
        "virtio"
    }
}
