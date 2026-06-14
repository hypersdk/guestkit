// SPDX-License-Identifier: Apache-2.0
//! Standalone Zeus VM Tools guest agent entry (Linux + Windows).

use anyhow::{bail, Context, Result};
use std::env;

#[cfg(windows)]
mod service;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "rpc" || a == "--rpc") {
        #[cfg(unix)]
        {
            let socket = parse_flag(&args, "--socket");
            return guestkit::agent::local_client::run_rpc_stdio(socket.as_deref());
        }
        #[cfg(not(unix))]
        bail!("rpc requires Unix local socket");
    }

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
        let check = guestkit::agent::updater::check_update().await?;
        if check.update_available {
            println!(
                "update available: {} -> {} (channel {})",
                check.current_version,
                check.remote_version.unwrap_or_default(),
                check.channel
            );
            if let Some(url) = &check.artifact_url {
                println!("artifact: {url}");
            }
            if let Some(sha) = &check.artifact_sha256 {
                println!("sha256: {sha}");
            }
        } else {
            println!(
                "zyvor-guest-agent {} is current (channel {})",
                check.current_version, check.channel
            );
        }
        return Ok(());
    }

    if args.iter().any(|a| a == "--apply-update") {
        let msg = guestkit::agent::updater::stage_update(true).await?;
        println!("{msg}");
        return Ok(());
    }

    if args.iter().any(|a| a == "--scheduled-update") {
        let msg = guestkit::agent::updater::run_scheduled_update().await?;
        println!("{msg}");
        return Ok(());
    }

    if args.iter().any(|a| a == "sign-manifest") {
        let json = args
            .iter()
            .position(|a| a == "sign-manifest")
            .and_then(|i| args.get(i + 1))
            .map(String::as_str)
            .context("usage: zyvor-guest-agent sign-manifest '<json>'")?;
        let sig = guestkit::agent::updater::sign_manifest_cli(json)?;
        println!("{sig}");
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
