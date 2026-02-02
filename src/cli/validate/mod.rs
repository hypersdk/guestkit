// SPDX-License-Identifier: LGPL-3.0-or-later
//! Policy-based validation module

pub mod policy;
pub mod rules;
pub mod benchmarks;

use anyhow::Result;
use guestkit::Guestfs;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use policy::{Policy, PolicyRule, RuleType};
pub use benchmarks::Benchmark;

/// Validation result for a single rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule_id: String,
    pub rule_name: String,
    pub status: ValidationStatus,
    pub message: String,
    pub severity: String,
    pub remediation: Option<String>,
}

/// Validation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationStatus {
    Pass,
    Fail,
    Warning,
    Skip,
    Error,
}

impl ValidationStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Warning => "WARN",
            Self::Skip => "SKIP",
            Self::Error => "ERROR",
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            Self::Pass => "✅",
            Self::Fail => "❌",
            Self::Warning => "⚠️",
            Self::Skip => "⏭️",
            Self::Error => "🔥",
        }
    }
}

/// Complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub image_path: String,
    pub policy_name: String,
    pub timestamp: String,
    pub results: Vec<ValidationResult>,
    pub summary: ValidationSummary,
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_rules: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub skipped: usize,
    pub errors: usize,
    pub compliance_score: f64,
}

impl ValidationSummary {
    pub fn new(results: &[ValidationResult]) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.status == ValidationStatus::Pass).count();
        let failed = results.iter().filter(|r| r.status == ValidationStatus::Fail).count();
        let warnings = results.iter().filter(|r| r.status == ValidationStatus::Warning).count();
        let skipped = results.iter().filter(|r| r.status == ValidationStatus::Skip).count();
        let errors = results.iter().filter(|r| r.status == ValidationStatus::Error).count();

        let compliance_score = if total > 0 {
            (passed as f64 / (total - skipped) as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total_rules: total,
            passed,
            failed,
            warnings,
            skipped,
            errors,
            compliance_score,
        }
    }
}

/// Validate disk image against policy
pub fn validate_image<P: AsRef<Path>>(
    image_path: P,
    policy: &Policy,
    verbose: bool,
) -> Result<ValidationReport> {
    let image_path_str = image_path.as_ref().display().to_string();

    if verbose {
        println!("🔍 Validating: {}", image_path_str);
        println!("📋 Policy: {}", policy.name);
    }

    // Initialize guestfs
    let mut g = Guestfs::new()?;
    g.add_drive_opts(&image_path, true, None)?;
    g.launch()?;

    // Inspect OS
    let roots = g.inspect_os()?;
    if roots.is_empty() {
        anyhow::bail!("No operating systems found in disk image");
    }

    let root = &roots[0];

    // Mount filesystems
    let mountpoints = g.inspect_get_mountpoints(root)?;
    for (mp, dev) in mountpoints {
        let _ = g.mount(&dev, &mp);
    }

    // Run validation rules
    let mut results = Vec::new();

    for rule in &policy.rules {
        if verbose {
            println!("  Checking: {}", rule.name);
        }

        let result = validate_rule(&mut g, root, rule)?;
        results.push(result);
    }

    // Shutdown guestfs
    g.shutdown()?;

    // Calculate summary
    let summary = ValidationSummary::new(&results);

    Ok(ValidationReport {
        image_path: image_path_str,
        policy_name: policy.name.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        results,
        summary,
    })
}

/// Validate a single rule
fn validate_rule(
    g: &mut Guestfs,
    root: &str,
    rule: &PolicyRule,
) -> Result<ValidationResult> {
    let status = match &rule.rule_type {
        RuleType::PackageInstalled { package } => {
            check_package_installed(g, root, package)?
        }
        RuleType::PackageForbidden { package } => {
            check_package_forbidden(g, root, package)?
        }
        RuleType::FileExists { path } => {
            check_file_exists(g, path)?
        }
        RuleType::FileNotExists { path } => {
            check_file_not_exists(g, path)?
        }
        RuleType::FileContains { path, pattern } => {
            check_file_contains(g, path, pattern)?
        }
        RuleType::FilePermissions { path, mode } => {
            check_file_permissions(g, path, mode)?
        }
        RuleType::ServiceEnabled { service } => {
            check_service_enabled(g, service)?
        }
        RuleType::ServiceDisabled { service } => {
            check_service_disabled(g, service)?
        }
        RuleType::UserExists { username } => {
            check_user_exists(g, username)?
        }
        RuleType::UserNotExists { username } => {
            check_user_not_exists(g, username)?
        }
        RuleType::PortClosed { port: _ } => {
            // Port checking requires more complex parsing
            ValidationStatus::Skip
        }
        RuleType::Custom { check: _ } => {
            // Custom checks would be implemented here
            ValidationStatus::Skip
        }
    };

    let message = if status == ValidationStatus::Pass {
        format!("{} - Check passed", rule.name)
    } else {
        format!("{} - Check failed", rule.name)
    };

    Ok(ValidationResult {
        rule_id: rule.id.clone(),
        rule_name: rule.name.clone(),
        status,
        message,
        severity: rule.severity.clone(),
        remediation: rule.remediation.clone(),
    })
}

// Rule check implementations

fn check_package_installed(g: &mut Guestfs, root: &str, package: &str) -> Result<ValidationStatus> {
    let apps = g.inspect_list_applications2(root)?;
    let installed = apps.iter().any(|(name, _, _)| name == package);
    Ok(if installed { ValidationStatus::Pass } else { ValidationStatus::Fail })
}

fn check_package_forbidden(g: &mut Guestfs, root: &str, package: &str) -> Result<ValidationStatus> {
    let apps = g.inspect_list_applications2(root)?;
    let installed = apps.iter().any(|(name, _, _)| name == package);
    Ok(if installed { ValidationStatus::Fail } else { ValidationStatus::Pass })
}

fn check_file_exists(g: &mut Guestfs, path: &str) -> Result<ValidationStatus> {
    let exists = g.exists(path)?;
    Ok(if exists { ValidationStatus::Pass } else { ValidationStatus::Fail })
}

fn check_file_not_exists(g: &mut Guestfs, path: &str) -> Result<ValidationStatus> {
    let exists = g.exists(path)?;
    Ok(if exists { ValidationStatus::Fail } else { ValidationStatus::Pass })
}

fn check_file_contains(g: &mut Guestfs, path: &str, pattern: &str) -> Result<ValidationStatus> {
    if !g.exists(path)? {
        return Ok(ValidationStatus::Fail);
    }

    let content = g.read_file(path)?;
    let content_str = String::from_utf8_lossy(&content);
    Ok(if content_str.contains(pattern) {
        ValidationStatus::Pass
    } else {
        ValidationStatus::Fail
    })
}

fn check_file_permissions(g: &mut Guestfs, path: &str, expected_mode: &str) -> Result<ValidationStatus> {
    if !g.exists(path)? {
        return Ok(ValidationStatus::Fail);
    }

    let stat = g.stat(path)?;
    let actual_mode = format!("{:o}", stat.mode & 0o777);

    Ok(if actual_mode == expected_mode {
        ValidationStatus::Pass
    } else {
        ValidationStatus::Fail
    })
}

fn check_service_enabled(g: &mut Guestfs, service: &str) -> Result<ValidationStatus> {
    // Check if systemd unit is enabled
    let service_path = format!("/etc/systemd/system/multi-user.target.wants/{}.service", service);
    let enabled = g.exists(&service_path)?;

    Ok(if enabled { ValidationStatus::Pass } else { ValidationStatus::Fail })
}

fn check_service_disabled(g: &mut Guestfs, service: &str) -> Result<ValidationStatus> {
    let service_path = format!("/etc/systemd/system/multi-user.target.wants/{}.service", service);
    let enabled = g.exists(&service_path)?;

    Ok(if enabled { ValidationStatus::Fail } else { ValidationStatus::Pass })
}

fn check_user_exists(g: &mut Guestfs, username: &str) -> Result<ValidationStatus> {
    if !g.exists("/etc/passwd")? {
        return Ok(ValidationStatus::Error);
    }

    let passwd = g.read_file("/etc/passwd")?;
    let passwd_str = String::from_utf8_lossy(&passwd);

    let exists = passwd_str.lines().any(|line| {
        line.split(':').next().map(|u| u == username).unwrap_or(false)
    });

    Ok(if exists { ValidationStatus::Pass } else { ValidationStatus::Fail })
}

fn check_user_not_exists(g: &mut Guestfs, username: &str) -> Result<ValidationStatus> {
    if !g.exists("/etc/passwd")? {
        return Ok(ValidationStatus::Error);
    }

    let passwd = g.read_file("/etc/passwd")?;
    let passwd_str = String::from_utf8_lossy(&passwd);

    let exists = passwd_str.lines().any(|line| {
        line.split(':').next().map(|u| u == username).unwrap_or(false)
    });

    Ok(if exists { ValidationStatus::Fail } else { ValidationStatus::Pass })
}


/// Format validation report as text
pub fn format_report(report: &ValidationReport) -> String {
    let mut output = String::new();

    output.push_str(&format!("🔍 Policy Validation Report\n"));
    output.push_str(&format!("==========================\n\n"));
    output.push_str(&format!("Image: {}\n", report.image_path));
    output.push_str(&format!("Policy: {}\n", report.policy_name));
    output.push_str(&format!("Time: {}\n\n", report.timestamp));

    output.push_str(&format!("📊 Summary\n"));
    output.push_str(&format!("----------\n"));
    output.push_str(&format!("Total Rules: {}\n", report.summary.total_rules));
    output.push_str(&format!("✅ Passed: {}\n", report.summary.passed));
    output.push_str(&format!("❌ Failed: {}\n", report.summary.failed));
    output.push_str(&format!("⚠️  Warnings: {}\n", report.summary.warnings));
    output.push_str(&format!("⏭️  Skipped: {}\n", report.summary.skipped));
    output.push_str(&format!("\n📈 Compliance Score: {:.1}%\n\n", report.summary.compliance_score));

    if report.summary.failed > 0 {
        output.push_str(&format!("❌ Failed Checks\n"));
        output.push_str(&format!("---------------\n"));
        for result in &report.results {
            if result.status == ValidationStatus::Fail {
                output.push_str(&format!("  {} [{}] {}\n",
                    result.status.emoji(),
                    result.severity,
                    result.rule_name
                ));
                if let Some(remediation) = &result.remediation {
                    output.push_str(&format!("    💡 {}\n", remediation));
                }
            }
        }
        output.push('\n');
    }

    if report.summary.warnings > 0 {
        output.push_str(&format!("⚠️  Warnings\n"));
        output.push_str(&format!("-----------\n"));
        for result in &report.results {
            if result.status == ValidationStatus::Warning {
                output.push_str(&format!("  {} [{}] {}\n",
                    result.status.emoji(),
                    result.severity,
                    result.rule_name
                ));
            }
        }
        output.push('\n');
    }

    if report.summary.compliance_score >= 90.0 {
        output.push_str("✅ Excellent compliance!\n");
    } else if report.summary.compliance_score >= 75.0 {
        output.push_str("⚠️  Good compliance, but improvements needed\n");
    } else if report.summary.compliance_score >= 50.0 {
        output.push_str("❌ Poor compliance - significant issues found\n");
    } else {
        output.push_str("🔥 Critical compliance failure!\n");
    }

    output
}
