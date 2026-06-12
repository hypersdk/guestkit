// SPDX-License-Identifier: Apache-2.0
//! Migration assurance CLI commands (doctor, policy, fleet, migrate-plan, forensic diff).

use crate::assurance::{
    run_doctor, run_migrate_plan, run_repair_plan, MigratePlanOptions, RepairOptions,
};
use crate::boot::BootTarget;
use crate::fleet::analyze_fleet;
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

use super::{init_guestfs_ro, mount_all_ro, validate_command};

pub use crate::assurance::collect_assurance_data;

/// Bootability prediction: `guestkit doctor`
pub fn doctor_command(
    image: &Path,
    target: &str,
    explain: bool,
    ai: bool,
    output_format: &str,
    verbose: bool,
) -> Result<()> {
    use crate::core::ProgressReporter;

    let progress = ProgressReporter::spinner("Analyzing bootability...");
    let result = run_doctor(image, target, explain, verbose)?;
    progress.finish_and_clear();

    let boot_report = &result.bootability;

    if output_format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    println!();
    println!("{}", "Bootability Assessment".bold().cyan());
    println!("{}", "═".repeat(50));
    println!();
    println!(
        "  {}",
        boot_report.boot_probability_message().green().bold()
    );
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
        let icon = if check.passed {
            "✓".green()
        } else {
            "✗".red()
        };
        println!("  {} [{}] {}", icon, check.id, check.message);
    }

    if let Some(root_cause) = &result.root_cause {
        println!();
        println!("{}", "Root Cause Analysis".bold().cyan());
        println!("  {}", root_cause.summary.yellow());
        for step in &root_cause.chain {
            println!("  {}. {}", step.step, step.description);
        }
    }

    if explain || ai {
        print_guest_intelligence(image, target, verbose, ai, result.copilot.clone())?;
    }

    Ok(())
}

fn print_guest_intelligence(
    image: &Path,
    target: &str,
    verbose: bool,
    ai: bool,
    copilot: Option<crate::assurance::MigrationBriefing>,
) -> Result<()> {
    use crate::ai::build_intelligence;
    use crate::assurance::{boot_target_from_str, collect_assurance_data};
    use colored::Colorize;

    let (evidence, boot) = collect_assurance_data(image, boot_target_from_str(target), verbose)?;
    let intel = build_intelligence(&evidence, Some(&boot), copilot);

    println!();
    println!("{}", "Guest Intelligence".bold().cyan());
    println!("{}", intel.narrative.executive_summary);

    if !intel.recommendations.is_empty() {
        println!();
        println!("{}", "Top recommendations:".bold());
        for rec in intel.recommendations.iter().take(5) {
            println!(
                "  • [{}] {} — {}",
                format!("{:?}", rec.category),
                rec.title,
                rec.citation
            );
        }
    }

    println!("  CIS-lite score: {}/100", intel.security_profile.score);

    #[cfg(feature = "ai")]
    if ai {
        use crate::ai::{run_agent_on_evidence, AgentConfig};
        let rt = tokio::runtime::Runtime::new()?;
        let config = AgentConfig {
            boot_target: target.to_string(),
            ..Default::default()
        };
        match rt.block_on(run_agent_on_evidence(
            &evidence,
            "Summarize migration readiness, cite evidence, and list the top 3 actions.",
            &config,
        )) {
            Ok(agent) => {
                println!();
                println!("{}", "AI analysis".bold().magenta());
                println!("{}", agent.answer);
            }
            Err(e) => eprintln!("AI agent skipped: {e:#}"),
        }
    }

    #[cfg(not(feature = "ai"))]
    if ai {
        eprintln!("AI requires rebuild with --features ai");
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
            println!(
                "  {} {} (boot score: {:.0}%)",
                "✗".red(),
                b.image,
                b.boot_score
            );
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
#[cfg(feature = "agent")]
pub fn migrate_plan_command(
    image: &Path,
    target: &str,
    explain: bool,
    ai: bool,
    output_format: &str,
    export: Option<&Path>,
    verbose: bool,
    inject_agent: bool,
    agent_binary: Option<&Path>,
    agent_unit: Option<&Path>,
) -> Result<()> {
    migrate_plan_command_impl(
        image,
        target,
        explain,
        ai,
        output_format,
        export,
        verbose,
        inject_agent,
        Some((agent_binary, agent_unit)),
    )
}

#[cfg(not(feature = "agent"))]
pub fn migrate_plan_command(
    image: &Path,
    target: &str,
    explain: bool,
    ai: bool,
    output_format: &str,
    export: Option<&Path>,
    verbose: bool,
    inject_agent: bool,
) -> Result<()> {
    migrate_plan_command_impl(
        image,
        target,
        explain,
        ai,
        output_format,
        export,
        verbose,
        inject_agent,
        None,
    )
}

fn migrate_plan_command_impl(
    image: &Path,
    target: &str,
    explain: bool,
    ai: bool,
    output_format: &str,
    export: Option<&Path>,
    verbose: bool,
    inject_agent: bool,
    #[cfg(feature = "agent")] agent_paths: Option<(Option<&Path>, Option<&Path>)>,
    #[cfg(not(feature = "agent"))] _agent_paths: Option<()>,
) -> Result<()> {
    let mut options = MigratePlanOptions {
        explain,
        verbose,
        export_fix_plan: export.is_some(),
        inject_agent,
        ..Default::default()
    };
    #[cfg(feature = "agent")]
    if let Some((agent_binary, agent_unit)) = agent_paths {
        options.agent_binary = agent_binary.map(|p| p.to_path_buf());
        options.agent_unit = agent_unit.map(|p| p.to_path_buf());
    }

    let result = run_migrate_plan(image, target, &options)?;
    let migration_score = &result.migration_score;
    let boot_report = &result.bootability;

    if let Some(export_path) = export {
        use crate::cli::plan::PlanExporter;
        use std::fs;

        let plan = result.fix_plan.as_ref().context("fix plan not generated")?;

        let content = if export_path.extension().is_some_and(|e| e == "json") {
            PlanExporter::to_json(plan)?
        } else {
            PlanExporter::to_yaml(plan)?
        };
        fs::write(export_path, content)
            .with_context(|| format!("Failed to write fix plan to {}", export_path.display()))?;
        println!(
            "{}",
            format!(
                "Fix plan written to {} ({} operations)",
                export_path.display(),
                plan.operations.len()
            )
            .green()
        );
    }

    if output_format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
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

    if let Some(rc) = &result.root_cause {
        println!();
        println!("  {}", rc.summary);
    }

    if explain || ai {
        print_guest_intelligence(image, target, verbose, ai, result.copilot.clone())?;
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
    println!(
        "  Security drift score: {:.0}%",
        forensic.security_drift_score
    );
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
#[cfg(feature = "agent")]
pub fn repair_boot_command(
    image: &Path,
    dry_run: bool,
    verbose: bool,
    inject_agent: bool,
    agent_binary: Option<&Path>,
    agent_unit: Option<&Path>,
) -> Result<()> {
    repair_boot_command_impl(
        image,
        dry_run,
        verbose,
        inject_agent,
        Some((agent_binary, agent_unit)),
    )
}

#[cfg(not(feature = "agent"))]
pub fn repair_boot_command(
    image: &Path,
    dry_run: bool,
    verbose: bool,
    inject_agent: bool,
) -> Result<()> {
    repair_boot_command_impl(image, dry_run, verbose, inject_agent, None)
}

fn repair_boot_command_impl(
    image: &Path,
    dry_run: bool,
    verbose: bool,
    inject_agent: bool,
    #[cfg(feature = "agent")] agent_paths: Option<(Option<&Path>, Option<&Path>)>,
    #[cfg(not(feature = "agent"))] _agent_paths: Option<()>,
) -> Result<()> {
    let mut options = RepairOptions {
        dry_run,
        verbose,
        inject_agent,
        ..Default::default()
    };
    #[cfg(feature = "agent")]
    if let Some((agent_binary, agent_unit)) = agent_paths {
        options.agent_binary = agent_binary.map(|p| p.to_path_buf());
        options.agent_unit = agent_unit.map(|p| p.to_path_buf());
    }

    let result = run_repair_plan(image, &options)?;

    if !result.applied && result.fix_plan.operations.is_empty() {
        println!("{}", result.message.green());
        return Ok(());
    }

    if dry_run {
        println!("{}", "Boot Repair Plan".bold().cyan());
        for op in &result.fix_plan.operations {
            println!("  → {}: {}", op.id, op.description);
        }
        println!();
        println!("{}", result.message.yellow());
        return Ok(());
    }

    if result.applied {
        println!("{}", result.message.green());
        if let (before, Some(after)) = (result.before_score, result.after_score) {
            if after < before {
                println!(
                    "{}",
                    "Warning: boot score decreased after repair — review changes".yellow()
                );
            }
        }
    }

    Ok(())
}
