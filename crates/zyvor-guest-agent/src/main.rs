// SPDX-License-Identifier: Apache-2.0
//! Standalone Zeus VM Tools guest agent entry (Linux + Windows).

use anyhow::{bail, Result};
use std::env;

#[cfg(windows)]
mod service;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "status") {
        #[cfg(unix)]
        {
            let socket = parse_flag(&args, "--socket");
            return guestkit::agent::local_client::print_local_status(socket.as_deref());
        }
        #[cfg(not(unix))]
        bail!("status requires Unix local socket");
    }

    if args.iter().any(|a| a == "--check-update") {
        log::info!(
            "Zyvor GuestAgent update check (stub) — current version {}",
            guestkit::VERSION
        );
        println!("zyvor-guest-agent {} (update channel: stable, no signed updater yet)", guestkit::VERSION);
        return Ok(());
    }

    if args.iter().any(|a| a == "--service") {
        #[cfg(windows)]
        {
            return service::run_service();
        }
        #[cfg(not(windows))]
        bail!("--service is supported on Windows only");
    }

    let channel = parse_channel(&args);
    let device = parse_flag(&args, "--device");
    let socket = parse_flag(&args, "--socket");

    guestkit::agent::run_agent(guestkit::agent::AgentArgs {
        channel,
        device,
        socket,
        user: parse_flag(&args, "--user"),
    })
    .await
}

fn parse_channel(args: &[String]) -> guestkit::agent::AgentChannel {
    if let Some(i) = args.iter().position(|a| a == "--channel") {
        if let Some(val) = args.get(i + 1) {
            match val.as_str() {
                "stdio" => return guestkit::agent::AgentChannel::Stdio,
                "vsock" => return guestkit::agent::AgentChannel::Vsock,
                "virtio" | _ => return guestkit::agent::AgentChannel::Virtio,
            }
        }
    }
    #[cfg(windows)]
    {
        guestkit::agent::AgentChannel::Stdio
    }
    #[cfg(not(windows))]
    {
        guestkit::agent::AgentChannel::Virtio
    }
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
