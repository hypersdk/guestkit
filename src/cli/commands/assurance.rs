// SPDX-License-Identifier: LGPL-3.0-or-later
//! Migration assurance CLI commands (doctor, policy, fleet, migrate-plan, forensic diff).

use anyhow::{Context, Result};
use colored::Colorize;
use crate::boot::{analyze_bootability, BootTarget};
use crate::evidence::build_evidence;
use crate::fleet::analyze_fleet;
use crate::inference::infer_root_cause;
use std::path::{Path, PathBuf};

use super::{init_guestfs_ro, mount_all_ro, validate_command};

/// Shared helper: mount guestfs and collect evidence + boot report.
pub fn collect_assurance_data(
    image: &Path,
    target: BootTarget,
    verbose: bool,
) -> Result<(crate::evidence::EvidenceSnapshot, crate::boot::BootabilityReport)> {
    let resolved = crate::storage::resolve_to_local_path(
        image.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
    )
    .unwrap_or_else(|_| image.to_path_buf());

    let mut g = init_guestfs_ro(&resolved, verbose)?;
    let root = mount_all_ro(&mut g).context("No operating system found in disk image")?;

    let evidence = build_evidence(&mut g, &root, &resolved)?;
    let boot_report = analyze_bootability(&evidence, target);
    let _ = g.shutdown();
    Ok((evidence, boot_report))
}

/// Bootability prediction: `guestkit doctor`
pub fn doctor_command(
    image: &Path,
    target: &str,
    explain: bool,
    output_format: &str,
    verbose: bool,
) -> Result<()> {
    use crate::core::ProgressReporter;

    let progress = ProgressReporter::spinner("Analyzing bootability...");
    let boot_target = BootTarget::parse(target);
    let (evidence, boot_report) = collect_assurance_data(image, boot_target, verbose)?;

    // Cache evidence snapshot
    if let Ok(cache) = crate::cli::cache::EvidenceCache::new() {
        let _ = cache.store(image, &evidence);
    }

    progress.finish_and_clear();

    if output_format == "json" {
        let mut out = serde_json::json!({
            "bootability": boot_report,
            "evidence_schema": evidence.schema_version,
        });
        if explain {
            let root_cause = infer_root_cause(&evidence, &boot_report);
            out["root_cause"] = serde_json::to_value(&root_cause)?;
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!();
    println!("{}", "Bootability Assessment".bold().cyan());
    println!("{}", "═".repeat(50));
    println!();
    println!("  {}", boot_report.boot_probability_message().green().bold());
    println!();

    if !boot_report.blockers.is_empty() {
        println!("{}", "Blockers:".red().bold());
        for b in &boot_report.blockers {
            println!("  {} {} — {}", "✗".red(), b.title.bold(), b.message);
            if let Some(r) = &b.remediation {
                println!("    → {}", r.dimmed());
            }
        }
        println!();
    }

    if !boot_report.warnings.is_empty() {
        println!("{}", "Warnings:".yellow().bold());
        for w in &boot_report.warnings {
            println!("  {} {} — {}", "⚠".yellow(), w.title, w.message);
        }
        println!();
    }

    println!("{}", "Checks:".bold());
    for check in &boot_report.checks {
        if check.weight <= 0.0 {
            continue;
        }
        let icon = if check.passed { "✓".green() } else { "✗".red() };
        println!("  {} [{}] {}", icon, check.id, check.message);
    }

    if explain {
        let root_cause = infer_root_cause(&evidence, &boot_report);
        println!();
        println!("{}", "Root Cause Analysis".bold().cyan());
        println!("  {}", root_cause.summary.yellow());
        for step in &root_cause.chain {
            println!("  {}. {}", step.step, step.description);
        }
    }

    Ok(())
}

/// Policy check alias wrapper
pub fn policy_check_command(
    image: &Path,
    policy: Option<&Path>,
    benchmark: Option<String>,
    example_policy: bool,
    format: &str,
    output: Option<&Path>,
    strict: bool,
    verbose: bool,
) -> Result<()> {
    validate_command(
        image,
        policy,
        benchmark,
        example_policy,
        format,
        output,
        strict,
        verbose,
    )
}

/// Fleet analysis: `guestkit fleet analyze`
pub fn fleet_analyze_command(dir: &Path, output_format: &str, verbose: bool) -> Result<()> {
    use crate::core::ProgressReporter;

    let mut images: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if matches!(
                    ext.to_lowercase().as_str(),
                    "qcow2" | "vmdk" | "raw" | "img" | "vhd" | "vdi"
                ) {
                    images.push(path);
                }
            }
        }
    }

    if images.is_empty() {
        anyhow::bail!("No disk images found in {}", dir.display());
    }

    let msg = format!("Analyzing {} VMs...", images.len());
    let progress = ProgressReporter::spinner(&msg);
    let mut snapshots = Vec::new();

    for image in &images {
        if verbose {
            eprintln!("  → {}", image.display());
        }
        if let Ok((evidence, boot)) = collect_assurance_data(image, BootTarget::Kvm, false) {
            snapshots.push((image.display().to_string(), evidence, boot.score));
        }
    }
    progress.finish_and_clear();

    let report = analyze_fleet(&snapshots);

    if output_format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!();
    println!("{}", "Fleet Analysis".bold().cyan());
    println!("  Total VMs: {}", report.total_vms);
    println!();

    for cluster in &report.clusters {
        if cluster.count > 1 {
            println!(
                "  {} {} identical {} nodes",
                "●".green(),
                cluster.count,
                cluster.os
            );
            for m in &cluster.members {
                println!("      - {}", m);
            }
        }
    }

    if !report.snowflakes.is_empty() {
        println!();
        println!("{}", "Anomalous VMs:".yellow().bold());
        for s in &report.snowflakes {
            println!("  {} {} — {}", "◆".yellow(), s.image, s.reason);
        }
    }

    if !report.migration_blockers.is_empty() {
        println!();
        println!("{}", "Migration blockers:".red().bold());
        for b in &report.migration_blockers {
            println!("  {} {} (boot score: {:.0}%)", "✗".red(), b.image, b.boot_score);
        }
    }

    if !report.golden_image_candidates.is_empty() {
        println!();
        println!("{}", "Golden image candidates:".green().bold());
        for g in &report.golden_image_candidates {
            println!("  {} {}", "★".green(), g);
        }
    }

    Ok(())
}

/// Migration plan: `guestkit migrate-plan`
pub fn migrate_plan_command(
    image: &Path,
    target: &str,
    explain: bool,
    output_format: &str,
    verbose: bool,
) -> Result<()> {
    let boot_target = match target.to_lowercase().as_str() {
        "proxmox" | "kvm" | "qemu" => BootTarget::Proxmox,
        "hyperv" | "hyper-v" => BootTarget::HyperV,
        "aws" | "azure" | "gcp" | "cloud" => BootTarget::Cloud,
        _ => BootTarget::Kvm,
    };

    let (evidence, boot_report) = collect_assurance_data(image, boot_target, verbose)?;
    let migration_score = crate::cli::migrate::plan::compute_migration_score(
        &evidence,
        &boot_report,
        target,
    );

    if output_format == "json" {
        let mut out = serde_json::json!({
            "target": target,
            "migration_score": migration_score.score,
            "bootability": boot_report,
            "driver_injections": migration_score.driver_injections,
            "required_changes": migration_score.required_changes,
            "licensing_warnings": migration_score.licensing_warnings,
            "estimated_downtime_minutes": migration_score.estimated_downtime_minutes,
        });
        if explain {
            out["root_cause"] = serde_json::to_value(infer_root_cause(&evidence, &boot_report))?;
        }
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    println!();
    println!("{}", format!("Migration Plan → {}", target).bold().cyan());
    println!("  Migration score: {:.0}%", migration_score.score);
    println!("  Boot score: {:.0}%", boot_report.score);
    println!(
        "  Est. downtime: {} min",
        migration_score.estimated_downtime_minutes
    );
    println!();

    if !migration_score.driver_injections.is_empty() {
        println!("{}", "Driver injections required:".yellow());
        for d in &migration_score.driver_injections {
            println!("  • {}", d);
        }
        println!();
    }

    if !migration_score.required_changes.is_empty() {
        println!("{}", "Required changes:".bold());
        for c in &migration_score.required_changes {
            println!("  • {}", c);
        }
        println!();
    }

    if !migration_score.licensing_warnings.is_empty() {
        println!("{}", "Licensing warnings:".red());
        for w in &migration_score.licensing_warnings {
            println!("  • {}", w);
        }
    }

    if explain {
        let rc = infer_root_cause(&evidence, &boot_report);
        println!();
        println!("  {}", rc.summary);
    }

    Ok(())
}

/// Forensic diff: `guestkit forensic-diff`
pub fn forensic_diff_command(
    old_image: &Path,
    new_image: &Path,
    output_format: &str,
    verbose: bool,
) -> Result<()> {
    use super::collect_inspection_data;
    use crate::cli::forensic_diff::compute_forensic_diff;

    let mut g1 = init_guestfs_ro(old_image, verbose)?;
    let root1 = mount_all_ro(&mut g1).context("No OS in old image")?;
    let report1 = collect_inspection_data(&mut g1, &root1, verbose)?;
    let _ = g1.shutdown();

    let mut g2 = init_guestfs_ro(new_image, verbose)?;
    let root2 = mount_all_ro(&mut g2).context("No OS in new image")?;
    let report2 = collect_inspection_data(&mut g2, &root2, verbose)?;
    let _ = g2.shutdown();

    let forensic = compute_forensic_diff(&report1, &report2);

    if output_format == "json" {
        println!("{}", serde_json::to_string_pretty(&forensic)?);
        return Ok(());
    }

    println!();
    println!("{}", "Forensic Diff Report".bold().cyan());
    println!("  {}", forensic.summary);
    println!("  Security drift score: {:.0}%", forensic.security_drift_score);
    println!("  Config drift items: {}", forensic.config_drift_count);

    if !forensic.suspicious_persistence.is_empty() {
        println!();
        println!("{}", "Suspicious persistence:".yellow());
        for s in &forensic.suspicious_persistence {
            println!("  • {}", s);
        }
    }

    if !forensic.ransomware_indicators.is_empty() {
        println!();
        println!("{}", "Ransomware indicators:".red().bold());
        for r in &forensic.ransomware_indicators {
            println!("  • {}", r);
        }
    }

    Ok(())
}

/// Transactional boot repair: `guestkit repair --fix boot`
pub fn repair_boot_command(image: &Path, dry_run: bool, verbose: bool) -> Result<()> {
    use crate::cli::plan::{PlanApplicator, PlanGenerator};

    let (_, boot_report) = collect_assurance_data(image, BootTarget::Kvm, verbose)?;

    if boot_report.blockers.is_empty() && boot_report.warnings.is_empty() {
        println!("{}", "No boot issues detected — nothing to repair.".green());
        return Ok(());
    }

    println!("{}", "Boot Repair Plan".bold().cyan());
    for b in boot_report
        .blockers
        .iter()
        .chain(boot_report.warnings.iter())
    {
        println!("  • [{}] {}", b.check_id, b.title);
    }

    let generator = PlanGenerator::new(image.display().to_string());
    let plan = generator.from_boot_report(&boot_report, image)?;

    if dry_run {
        println!();
        println!("{}", "Dry run — no changes applied.".yellow());
        for op in &plan.operations {
            println!("  → {}: {}", op.id, op.description);
        }
        return Ok(());
    }

    let before_score = boot_report.score;
    let applicator = PlanApplicator::new(image.to_str().unwrap().to_string(), false);
    let result = applicator.apply(&plan)?;

    if !result.success {
        anyhow::bail!("Repair failed: {}", result.message);
    }

    let (_, after_report) = collect_assurance_data(image, BootTarget::Kvm, verbose)?;
    println!(
        "{}",
        format!(
            "Repair complete. Boot score: {:.0}% → {:.0}%",
            before_score, after_report.score
        )
        .green()
    );

    if after_report.score < before_score {
        println!(
            "{}",
            "Warning: boot score decreased after repair — review changes".yellow()
        );
    }

    Ok(())
}
