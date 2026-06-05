// SPDX-License-Identifier: Apache-2.0
//! In-guest fix plan executor (native, no guestfs).

use super::apply::ApplyResult;
use super::topo_sort::topological_sort;
use super::types::*;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ROLLBACK_ROOT: &str = "/var/lib/guestkit/rollback";

pub struct LivePlanExecutor {
    dry_run: bool,
}

impl LivePlanExecutor {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    pub fn apply(&self, plan: &FixPlan) -> Result<ApplyResult> {
        if self.dry_run {
            return Ok(ApplyResult {
                success: true,
                operations_applied: 0,
                operations_failed: 0,
                operations_skipped: plan.operations.len(),
                message: "Dry run completed - no changes made".to_string(),
            });
        }

        let plan_id = plan_id_for(plan);
        let rollback_dir = PathBuf::from(ROLLBACK_ROOT).join(&plan_id);
        fs::create_dir_all(&rollback_dir).context("create rollback directory")?;

        let sorted = topological_sort(plan);
        let mut applied = 0usize;
        let mut failed = 0usize;
        let mut skipped = 0usize;

        for op in &sorted {
            match self.apply_operation(op, &rollback_dir) {
                Ok(true) => applied += 1,
                Ok(false) => skipped += 1,
                Err(e) => {
                    log::error!("Operation {} failed: {}", op.id, e);
                    failed += 1;
                }
            }
        }

        if failed == 0 {
            self.run_post_apply(&plan.post_apply)?;
        }

        Ok(ApplyResult {
            success: failed == 0,
            operations_applied: applied,
            operations_failed: failed,
            operations_skipped: skipped,
            message: if failed == 0 {
                format!("Live plan applied ({applied} operations, rollback: {rollback_dir:?})")
            } else {
                format!("{applied} applied, {failed} failed")
            },
        })
    }

    pub fn rollback(&self, plan_id: &str) -> Result<String> {
        let rollback_dir = PathBuf::from(ROLLBACK_ROOT).join(plan_id);
        if !rollback_dir.is_dir() {
            anyhow::bail!("rollback snapshot not found: {plan_id}");
        }

        let manifest_path = rollback_dir.join("manifest.json");
        let manifest: RollbackManifest = if manifest_path.exists() {
            serde_json::from_str(&fs::read_to_string(&manifest_path)?)?
        } else {
            RollbackManifest { files: Vec::new() }
        };

        for entry in &manifest.files {
            if entry.backup_path.exists() {
                if let Some(parent) = Path::new(&entry.original_path).parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&entry.backup_path, &entry.original_path)?;
            } else if entry.removed {
                let _ = fs::remove_file(&entry.original_path);
            }
        }

        Ok(format!("Rollback complete for plan {plan_id}"))
    }

    fn apply_operation(&self, op: &Operation, rollback_dir: &Path) -> Result<bool> {
        match &op.op_type {
            OperationType::FileEdit(fe) => {
                self.snapshot_file(&fe.file, rollback_dir)?;
                let content =
                    fs::read_to_string(&fe.file).with_context(|| format!("read {}", fe.file))?;
                let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                for change in &fe.changes {
                    let mut found = false;
                    for line in lines.iter_mut() {
                        if line.trim() == change.before.trim() {
                            *line = change.after.clone();
                            found = true;
                            break;
                        }
                    }
                    if !found && change.line > 0 && change.line <= lines.len() {
                        lines[change.line - 1] = change.after.clone();
                    }
                }
                fs::write(&fe.file, lines.join("\n") + "\n")?;
                Ok(true)
            }
            OperationType::CommandExec(ce) => {
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(&ce.command)
                    .status()
                    .with_context(|| format!("exec {}", ce.command))?;
                if status.code().unwrap_or(-1) != ce.expected_exit {
                    anyhow::bail!(
                        "command '{}' exited with {:?}, expected {}",
                        ce.command,
                        status.code(),
                        ce.expected_exit
                    );
                }
                Ok(true)
            }
            OperationType::FilePermissions(fp) => {
                self.snapshot_file(&fp.path, rollback_dir)?;
                let mode = u32::from_str_radix(if fp.mode.is_empty() { "0" } else { &fp.mode }, 8)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = fs::metadata(&fp.path)?.permissions();
                    perms.set_mode(mode);
                    fs::set_permissions(&fp.path, perms)?;
                }
                #[cfg(not(unix))]
                {
                    let _ = mode;
                    anyhow::bail!("chmod not supported on this platform");
                }
                Ok(true)
            }
            OperationType::DirectoryCreate(dc) => {
                fs::create_dir_all(&dc.path)?;
                Ok(true)
            }
            OperationType::FileCopy(fc) => {
                if fc.backup && Path::new(&fc.destination).exists() {
                    self.snapshot_file(&fc.destination, rollback_dir)?;
                }
                if let Some(parent) = Path::new(&fc.destination).parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&fc.source, &fc.destination)?;
                Ok(true)
            }
            OperationType::SelinuxMode(sm) => {
                self.snapshot_file(&sm.file, rollback_dir)?;
                if let Ok(content) = fs::read_to_string(&sm.file) {
                    let new_content: String = content
                        .lines()
                        .map(|line| {
                            let trimmed = line.trim_start();
                            if !trimmed.starts_with('#')
                                && trimmed.starts_with(&format!("SELINUX={}", sm.current))
                            {
                                format!("SELINUX={}", sm.target)
                            } else {
                                line.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        + "\n";
                    fs::write(&sm.file, new_content)?;
                }
                let _ = Command::new("setenforce")
                    .arg(if sm.target == "enforcing" { "1" } else { "0" })
                    .status();
                Ok(true)
            }
            OperationType::PackageInstall(pi) => {
                self.install_packages(&pi.packages)?;
                Ok(true)
            }
            OperationType::ServiceOperation(so) => {
                if so.restart {
                    let _ = Command::new("systemctl")
                        .args(["restart", &so.service])
                        .status();
                } else if so.start {
                    let _ = Command::new("systemctl")
                        .args(["start", &so.service])
                        .status();
                }
                if let Some(state) = &so.state {
                    let action = if state == "enabled" {
                        "enable"
                    } else {
                        "disable"
                    };
                    let _ = Command::new("systemctl")
                        .args([action, &so.service])
                        .status();
                }
                Ok(true)
            }
            OperationType::RegistryEdit(re) => {
                log::warn!("Registry edit ({}) skipped on Linux", re.key);
                Ok(false)
            }
        }
    }

    fn install_packages(&self, packages: &[String]) -> Result<()> {
        if packages.is_empty() {
            return Ok(());
        }
        let status = if Path::new("/usr/bin/apt-get").exists() {
            Command::new("apt-get")
                .args(["install", "-y"])
                .args(packages)
                .status()
        } else if Path::new("/usr/bin/dnf").exists() {
            Command::new("dnf")
                .args(["install", "-y"])
                .args(packages)
                .status()
        } else if Path::new("/usr/bin/zypper").exists() {
            Command::new("zypper")
                .args(["install", "-y"])
                .args(packages)
                .status()
        } else {
            anyhow::bail!("no supported package manager found");
        }?;
        if !status.success() {
            anyhow::bail!("package install failed");
        }
        Ok(())
    }

    fn snapshot_file(&self, path: &str, rollback_dir: &Path) -> Result<()> {
        let src = Path::new(path);
        if !src.exists() {
            return Ok(());
        }
        let backup_name = path.trim_start_matches('/').replace('/', "_");
        let backup_path = rollback_dir.join(&backup_name);
        fs::copy(src, &backup_path)?;

        let manifest_path = rollback_dir.join("manifest.json");
        let mut manifest: RollbackManifest = if manifest_path.exists() {
            serde_json::from_str(&fs::read_to_string(&manifest_path)?)?
        } else {
            RollbackManifest { files: Vec::new() }
        };
        if !manifest.files.iter().any(|f| f.original_path == path) {
            manifest.files.push(RollbackFileEntry {
                original_path: path.to_string(),
                backup_path,
                removed: false,
            });
            fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
        }
        Ok(())
    }

    fn run_post_apply(&self, actions: &[PostApplyAction]) -> Result<()> {
        for action in actions {
            match action {
                PostApplyAction::ServiceRestart { services } => {
                    for svc in services {
                        let _ = Command::new("systemctl").args(["restart", svc]).status();
                    }
                }
                PostApplyAction::Validation {
                    command,
                    expected_output,
                } => {
                    let out = Command::new("sh")
                        .arg("-c")
                        .arg(command)
                        .output()
                        .with_context(|| format!("validation: {command}"))?;
                    if !out.status.success() {
                        log::warn!("validation command failed: {command}");
                    }
                    if let Some(expected) = expected_output {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        if !stdout.contains(expected) {
                            log::warn!(
                                "validation output mismatch for {command}: expected {expected}"
                            );
                        }
                    }
                }
                PostApplyAction::Message { message } => {
                    log::info!("post-apply: {message}");
                }
                PostApplyAction::RebootRequired { reason } => {
                    log::warn!("reboot required: {reason}");
                }
            }
        }
        Ok(())
    }
}

fn plan_id_for(plan: &FixPlan) -> String {
    format!(
        "{}-{}",
        plan.vm.replace('/', "_"),
        plan.generated.timestamp()
    )
}

#[derive(Debug, Serialize, Deserialize)]
struct RollbackManifest {
    files: Vec<RollbackFileEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RollbackFileEntry {
    original_path: String,
    backup_path: PathBuf,
    removed: bool,
}
