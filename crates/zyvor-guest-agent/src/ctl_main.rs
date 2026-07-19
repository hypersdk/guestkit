// SPDX-License-Identifier: Apache-2.0
//! guestkitctl — local troubleshooting CLI for the GuestKit agent.
//!
//! Talks to the agent's local socket (`/run/guestkit/agent.sock`, legacy
//! zyvor path as fallback) with one framed request per connection.

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};

#[derive(Parser)]
#[command(
    name = "guestkitctl",
    about = "Local GuestKit agent control",
    version
)]
struct Cli {
    /// Agent socket path override
    #[arg(long, global = true, value_name = "PATH")]
    socket: Option<String>,

    /// Raw JSON output
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Agent + guest health summary
    Status,
    /// Rich heartbeat (agent state, pressure, pending reboot)
    Health,
    /// Performance summary
    Perf {
        /// Tier: fine, medium, coarse
        #[arg(long, default_value = "fine")]
        tier: String,
        /// Window in seconds
        #[arg(long, default_value_t = 900)]
        window: u64,
    },
    /// List failed services
    Services {
        /// Show all units, not only failed
        #[arg(long)]
        all: bool,
    },
    /// Migration readiness assessment
    Assess {
        #[arg(long, default_value = "kvm")]
        target: String,
    },
    /// Freeze filesystems
    Freeze,
    /// Thaw filesystems
    Thaw,
    /// Collect a support bundle
    Bundle {
        /// Output file (default: guestkit-support.tar.zst)
        #[arg(short, long, default_value = "guestkit-support.tar.zst")]
        output: String,
    },
    /// Invoke any agent method directly
    Call {
        method: String,
        /// JSON params
        #[arg(long, default_value = "{}")]
        params: String,
    },
}

fn call(cli: &Cli, method: &str, params: Value) -> Result<Value> {
    guestkit::agent::local_client::call_local(cli.socket.as_deref(), method, params)
}

fn print_result(cli: &Cli, value: &Value) {
    if cli.json {
        println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
    } else {
        print_human(value, 0);
    }
}

fn print_human(value: &Value, indent: usize) {
    let pad = "  ".repeat(indent);
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                match v {
                    Value::Object(_) | Value::Array(_) => {
                        println!("{pad}{k}:");
                        print_human(v, indent + 1);
                    }
                    _ => println!("{pad}{k}: {v}"),
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                match item {
                    Value::Object(_) => {
                        println!("{pad}-");
                        print_human(item, indent + 1);
                    }
                    _ => println!("{pad}- {item}"),
                }
            }
        }
        other => println!("{pad}{other}"),
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    match &cli.command {
        Cmd::Status => {
            let health = call(&cli, "guestkit.getGuestHealth", json!({}))?;
            print_result(&cli, &health);
        }
        Cmd::Health => {
            let hb = call(&cli, "guestkit.getAgentHealth", json!({}))?;
            print_result(&cli, &hb);
        }
        Cmd::Perf { tier, window } => {
            let summary = call(
                &cli,
                "guestkit.getPerformanceSummary",
                json!({ "tier": tier, "window_secs": window }),
            )?;
            print_result(&cli, &summary);
        }
        Cmd::Services { all } => {
            let method = if *all {
                "guestkit.getSystemdUnits"
            } else {
                "guestkit.getFailedUnits"
            };
            let units = call(&cli, method, json!({}))?;
            print_result(&cli, &units);
        }
        Cmd::Assess { target } => {
            let assessment = call(&cli, "guestkit.migration.assess", json!({ "target": target }))?;
            if cli.json {
                print_result(&cli, &assessment);
            } else {
                println!(
                    "Migration Score: {}/100  ({})",
                    assessment["overall_score"],
                    assessment["readiness"].as_str().unwrap_or("?")
                );
                if let Some(subs) = assessment["sub_scores"].as_object() {
                    for (k, v) in subs {
                        println!("  {k:<12} {v}");
                    }
                }
                if let Some(blockers) = assessment["critical_blockers"].as_array() {
                    for b in blockers {
                        println!(
                            "  BLOCKER [{}] {}",
                            b["check_id"].as_str().unwrap_or(""),
                            b["message"].as_str().unwrap_or("")
                        );
                    }
                }
            }
        }
        Cmd::Freeze => {
            let r = call(&cli, "guestkit.freezeFilesystem", json!({}))?;
            print_result(&cli, &r);
        }
        Cmd::Thaw => {
            let r = call(&cli, "guestkit.thawFilesystem", json!({}))?;
            print_result(&cli, &r);
        }
        Cmd::Bundle { output } => {
            let r = call(&cli, "guestkit.collectSupportBundle", json!({}))?;
            let data = r
                .get("data")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow::anyhow!("no bundle data in response"))?;
            use base64::engine::general_purpose::STANDARD;
            use base64::Engine;
            let bytes = STANDARD.decode(data)?;
            std::fs::write(output, &bytes)?;
            println!("wrote {} bytes to {}", bytes.len(), output);
        }
        Cmd::Call { method, params } => {
            let params: Value = serde_json::from_str(params)?;
            let r = call(&cli, method, params)?;
            print_result(&cli, &r);
        }
    }
    Ok(())
}
