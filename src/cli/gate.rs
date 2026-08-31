// SPDX-License-Identifier: Apache-2.0
//! Combined cutover gate: passport + optional SBOM + BitLocker + score.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateReport {
    pub allowed: bool,
    pub fail_below: f64,
    pub boot: Option<f64>,
    pub migration: Option<f64>,
    pub hard_blocked: bool,
    pub bitlocker_blocker: bool,
    pub sbom_dirty: bool,
    pub denies: Vec<String>,
    pub passport: Option<String>,
}

pub struct GateArgs {
    pub passport: Option<PathBuf>,
    pub image: Option<PathBuf>,
    pub target: String,
    pub fail_below: f64,
    pub sbom_old: Option<PathBuf>,
    pub sbom_new: Option<PathBuf>,
    pub rego: Option<PathBuf>,
    pub fail: bool,
}

pub fn run(args: GateArgs) -> Result<GateReport> {
    let mut denies = Vec::new();
    let mut boot = None;
    let mut migration = None;
    let mut hard_blocked = false;
    let mut bitlocker_blocker = false;
    let mut sbom_dirty = false;
    let mut passport_path = args.passport.clone();

    if passport_path.is_none() {
        if let Some(image) = &args.image {
            let tmp = image.with_extension("gate-passport.json");
            crate::cli::commands::assurance::passport_emit_command(
                image,
                &args.target,
                &tmp,
                false,
                false,
                None,
                None,
                None,
                Some("guestkit-gate"),
                None,
                false,
            )?;
            passport_path = Some(tmp);
        }
    }

    if let Some(pp) = &passport_path {
        let raw = std::fs::read_to_string(pp)
            .with_context(|| format!("read {}", pp.display()))?;
        let v: serde_json::Value = serde_json::from_str(&raw).context("parse passport")?;
        boot = v.pointer("/scores/boot").and_then(|x| x.as_f64());
        migration = v.pointer("/scores/migration").and_then(|x| x.as_f64());
        hard_blocked = v.get("hard_blocked").and_then(|x| x.as_bool()).unwrap_or(false);
        bitlocker_blocker = v
            .pointer("/windows/bitlocker_blocker")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        if hard_blocked {
            denies.push("passport hard_blocked".into());
        }
        if bitlocker_blocker {
            denies.push("BitLocker blocker".into());
        }
        if let Some(s) = boot {
            if s < args.fail_below {
                denies.push(format!("boot {s:.0} < {}", args.fail_below));
            }
        }
        if let Some(s) = migration {
            if s < args.fail_below {
                denies.push(format!("migration {s:.0} < {}", args.fail_below));
            }
        }
        if let Some(rego) = &args.rego {
            let report = crate::cli::validate::rego::eval_file(rego, &v)?;
            denies.extend(report.denies);
        }
    } else {
        denies.push("no passport or image".into());
    }

    if let (Some(a), Some(b)) = (&args.sbom_old, &args.sbom_new) {
        let d = crate::cli::sbom_diff::diff_files(a, b)?;
        sbom_dirty = d.dirty();
        if sbom_dirty {
            denies.push(format!(
                "SBOM drift +{} -{} ~{}",
                d.added.len(),
                d.removed.len(),
                d.updated.len()
            ));
        }
    }

    let allowed = denies.is_empty();
    Ok(GateReport {
        allowed,
        fail_below: args.fail_below,
        boot,
        migration,
        hard_blocked,
        bitlocker_blocker,
        sbom_dirty,
        denies,
        passport: passport_path.map(|p| p.display().to_string()),
    })
}

pub fn print(r: &GateReport) {
    println!(
        "gate allowed={} boot={:?} migration={:?} floor={}",
        r.allowed, r.boot, r.migration, r.fail_below
    );
    for d in &r.denies {
        println!("  deny: {d}");
    }
}

// passport emit is handled inside run() when --image is set.
