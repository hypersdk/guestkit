// SPDX-License-Identifier: Apache-2.0
//! virtctl / kubectl plugin: `virtctl guestkit …` and `kubectl guestkit …`.
//!
//! Install:
//!   install -m 0755 target/release/virtctl-guestkit /usr/local/bin/virtctl-guestkit
//!   ln -s virtctl-guestkit /usr/local/bin/kubectl-guestkit
//!
//! Domain lifecycle stays with virtctl. This plugin only runs GuestKit
//! guest/disk commands against `--image` or a hostDisk path from a VM YAML.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "virtctl-guestkit",
    about = "GuestKit plugin for virtctl/kubectl"
)]
struct Cli {
    #[command(subcommand)]
    cmd: PluginCmd,
}

#[derive(Subcommand)]
enum PluginCmd {
    /// Run `guestkit doctor` on a disk image (optionally named after a VM)
    Doctor {
        /// VM / VMI name (informational; pass --image for the disk)
        vm: Option<String>,
        #[arg(short, long)]
        namespace: Option<String>,
        #[arg(long)]
        image: Option<PathBuf>,
        #[arg(long, default_value = "kubevirt")]
        target: String,
        #[arg(long)]
        explain: bool,
    },
    /// Run `guestkit passport emit`
    Passport {
        vm: Option<String>,
        #[arg(long)]
        image: PathBuf,
        #[arg(long, default_value = "kubevirt")]
        target: String,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Run `guestkit passport handoff` after emit/verify
    Handoff {
        #[arg(long)]
        passport: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        fail_below: Option<f64>,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("virtctl-guestkit: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        PluginCmd::Doctor {
            vm,
            namespace,
            image,
            target,
            explain,
        } => {
            if let Some(name) = vm.as_deref() {
                hint_vm(name, namespace.as_deref());
            }
            let image = image.ok_or_else(|| {
                anyhow::anyhow!("pass --image PATH (virtctl does not expose PVC contents)")
            })?;
            let mut args = vec![
                "doctor".into(),
                image.display().to_string(),
                "--target".into(),
                target,
            ];
            if explain {
                args.push("--explain".into());
            }
            exec_guestkit(&args)
        }
        PluginCmd::Passport {
            vm,
            image,
            target,
            output,
        } => {
            if let Some(name) = vm.as_deref() {
                hint_vm(name, None);
            }
            exec_guestkit(&[
                "passport".into(),
                "emit".into(),
                image.display().to_string(),
                "--target".into(),
                target,
                "-o".into(),
                output.display().to_string(),
            ])
        }
        PluginCmd::Handoff {
            passport,
            output,
            fail_below,
        } => {
            let mut args = vec![
                "passport".into(),
                "handoff".into(),
                passport.display().to_string(),
            ];
            if let Some(o) = output {
                args.push("-o".into());
                args.push(o.display().to_string());
            }
            if let Some(n) = fail_below {
                args.push("--fail-below".into());
                args.push(n.to_string());
            }
            exec_guestkit(&args)
        }
    }
}

fn hint_vm(name: &str, namespace: Option<&str>) {
    let ns = namespace.unwrap_or("default");
    eprintln!("# VM {ns}/{name} — fetch YAML with: kubectl get vm {name} -n {ns} -o yaml");
}

fn exec_guestkit(args: &[String]) -> Result<()> {
    let bin = std::env::var("GUESTKIT_BIN").unwrap_or_else(|_| "guestkit".into());
    let status = Command::new(&bin)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("spawn {bin}"))?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
