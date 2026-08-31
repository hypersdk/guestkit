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
    /// Resolve hostDisk / PVC name from a VM/VMI (does not mount PVCs)
    Resolve {
        vm: String,
        #[arg(short, long)]
        namespace: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Run `guestkit gate` against --image or a resolved hostDisk
    Gate {
        vm: Option<String>,
        #[arg(short, long)]
        namespace: Option<String>,
        #[arg(long)]
        image: Option<PathBuf>,
        #[arg(long)]
        passport: Option<PathBuf>,
        #[arg(long, default_value_t = 80.0)]
        fail_below: f64,
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
            let image = resolve_image(image, vm.as_deref(), namespace.as_deref())?;
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
        PluginCmd::Resolve { vm, namespace, json } => {
            let info = inspect_vm(&vm, namespace.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&info)?);
            } else {
                println!("kind={} name={}", info.kind, info.name);
                for d in &info.disks {
                    println!("  {} {:?}", d.kind, d.path);
                }
            }
            Ok(())
        }
        PluginCmd::Gate {
            vm,
            namespace,
            image,
            passport,
            fail_below,
        } => {
            let mut args = vec!["gate".into(), "--fail-below".into(), fail_below.to_string()];
            if let Some(p) = passport {
                args.push("--passport".into());
                args.push(p.display().to_string());
            } else {
                let img = resolve_image(image, vm.as_deref(), namespace.as_deref())?;
                args.push("--image".into());
                args.push(img.display().to_string());
            }
            exec_guestkit(&args)
        }
    }
}

fn hint_vm(name: &str, namespace: Option<&str>) {
    let ns = namespace.unwrap_or("default");
    eprintln!("# VM {ns}/{name} — fetch YAML with: kubectl get vm {name} -n {ns} -o yaml");
}

#[derive(serde::Serialize)]
struct VmDisks {
    kind: String,
    name: String,
    disks: Vec<ResolvedDisk>,
}

#[derive(serde::Serialize)]
struct ResolvedDisk {
    kind: String,
    path: Option<String>,
}

fn resolve_image(
    explicit: Option<PathBuf>,
    vm: Option<&str>,
    namespace: Option<&str>,
) -> Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    let name = vm.ok_or_else(|| anyhow::anyhow!("pass --image PATH or a VM name"))?;
    let info = inspect_vm(name, namespace)?;
    if let Some(p) = info
        .disks
        .iter()
        .find(|d| d.kind == "hostDisk" && d.path.is_some())
        .and_then(|d| d.path.clone())
    {
        return Ok(PathBuf::from(p));
    }
    let pvcs: Vec<_> = info
        .disks
        .iter()
        .filter(|d| d.kind == "pvc")
        .filter_map(|d| d.path.clone())
        .collect();
    anyhow::bail!(
        "no hostDisk on {name}; PVCs {:?} are not mounted by this plugin — copy out or pass --image",
        pvcs
    )
}

fn inspect_vm(name: &str, namespace: Option<&str>) -> Result<VmDisks> {
    let ns = namespace.unwrap_or("default");
    let raw = kubectl_json(&["get", "vm", name, "-n", ns, "-o", "json"])
        .or_else(|_| kubectl_json(&["get", "vmi", name, "-n", ns, "-o", "json"]))?;
    let v: serde_json::Value = serde_json::from_str(&raw)?;
    let kind = v
        .get("kind")
        .and_then(|k| k.as_str())
        .unwrap_or("VirtualMachine")
        .to_string();
    let volumes = v
        .pointer("/spec/template/spec/volumes")
        .or_else(|| v.pointer("/spec/volumes"))
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let mut disks = Vec::new();
    for vol in volumes {
        if let Some(p) = vol.pointer("/hostDisk/path").and_then(|x| x.as_str()) {
            disks.push(ResolvedDisk {
                kind: "hostDisk".into(),
                path: Some(p.to_string()),
            });
        } else if let Some(c) = vol
            .pointer("/persistentVolumeClaim/claimName")
            .and_then(|x| x.as_str())
        {
            disks.push(ResolvedDisk {
                kind: "pvc".into(),
                path: Some(c.to_string()),
            });
        }
    }
    Ok(VmDisks {
        kind,
        name: name.into(),
        disks,
    })
}

fn kubectl_json(args: &[&str]) -> Result<String> {
    let out = Command::new("kubectl").args(args).output()?;
    if !out.status.success() {
        anyhow::bail!(
            "kubectl {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(String::from_utf8(out.stdout)?)
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
