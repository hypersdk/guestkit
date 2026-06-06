// SPDX-License-Identifier: Apache-2.0
//! Library-level migration assurance APIs (doctor, migrate-plan, repair).
//!
//! Used by CLI, worker handlers, and zyvor-api. Does not write to stdout.

use crate::boot::{analyze_bootability, BootTarget, BootabilityReport};
use crate::cli::migrate::plan::{compute_migration_score, MigrationScoreReport};
use crate::cli::plan::{FixPlan, PlanGenerator};
use crate::evidence::build_evidence;
use crate::evidence::EvidenceSnapshot;
use crate::inference::{infer_root_cause, RootCauseReport};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Resolve migration target string to boot analysis target.
pub fn boot_target_from_str(target: &str) -> BootTarget {
    match target.to_lowercase().as_str() {
        "kubevirt" => BootTarget::KubeVirt,
        "proxmox" => BootTarget::Proxmox,
        "kvm" | "qemu" => BootTarget::Kvm,
        "hyperv" | "hyper-v" => BootTarget::HyperV,
        "aws" | "azure" | "gcp" | "cloud" => BootTarget::Cloud,
        _ => BootTarget::Generic,
    }
}

/// Mount guestfs and collect evidence + boot report.
pub fn collect_assurance_data(
    image: &Path,
    target: BootTarget,
    verbose: bool,
) -> Result<(EvidenceSnapshot, BootabilityReport)> {
    let resolved = crate::storage::resolve_to_local_path(
        image
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path"))?,
    )
    .unwrap_or_else(|_| image.to_path_buf());

    let mut g = crate::cli::commands::init_guestfs_ro(&resolved, verbose)?;
    let root = crate::cli::commands::mount_all_ro(&mut g)
        .context("No operating system found in disk image")?;

    let evidence = build_evidence(&mut g, &root, &resolved)?;
    let boot_report = analyze_bootability(&evidence, target);
    let _ = g.shutdown();
    Ok((evidence, boot_report))
}

/// Result of `run_doctor`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorResult {
    pub target: String,
    pub bootability: BootabilityReport,
    pub evidence_schema: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<RootCauseReport>,
}

/// Run bootability doctor on an offline disk image.
pub fn run_doctor(image: &Path, target: &str, explain: bool, verbose: bool) -> Result<DoctorResult> {
    let boot_target = boot_target_from_str(target);
    let (evidence, boot_report) = collect_assurance_data(image, boot_target, verbose)?;

    if let Ok(cache) = crate::cli::cache::EvidenceCache::new() {
        let _ = cache.store(image, &evidence);
    }

    let root_cause = if explain {
        Some(infer_root_cause(&evidence, &boot_report))
    } else {
        None
    };

    Ok(DoctorResult {
        target: target.to_string(),
        bootability: boot_report,
        evidence_schema: evidence.schema_version.to_string(),
        root_cause,
    })
}

/// Result of `run_migrate_plan`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlanResult {
    pub target: String,
    pub migration_score: MigrationScoreReport,
    pub bootability: BootabilityReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<RootCauseReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_plan: Option<FixPlan>,
}

/// Options for migration plan generation.
#[derive(Debug, Clone, Default)]
pub struct MigratePlanOptions {
    pub explain: bool,
    pub verbose: bool,
    pub export_fix_plan: bool,
    pub inject_agent: bool,
    #[cfg(feature = "agent")]
    pub agent_binary: Option<std::path::PathBuf>,
    #[cfg(feature = "agent")]
    pub agent_unit: Option<std::path::PathBuf>,
}

/// Generate a hypervisor-aware migration plan for an offline disk image.
pub fn run_migrate_plan(
    image: &Path,
    target: &str,
    options: &MigratePlanOptions,
) -> Result<MigrationPlanResult> {
    let boot_target = boot_target_from_str(target);
    let (evidence, boot_report) =
        collect_assurance_data(image, boot_target, options.verbose)?;
    let migration_score =
        compute_migration_score(&evidence, &boot_report, target);

    let mut fix_plan = None;
    if options.export_fix_plan {
        let generator = PlanGenerator::new(image.display().to_string());
        #[cfg(feature = "agent")]
        let mut plan =
            generator.from_migration_report(&migration_score, &boot_report, target, image)?;
        #[cfg(not(feature = "agent"))]
        let plan =
            generator.from_migration_report(&migration_score, &boot_report, target, image)?;

        #[cfg(feature = "agent")]
        if options.inject_agent {
            let binary = crate::agent::inject::resolve_agent_binary(
                options.agent_binary.as_deref(),
            )?;
            let unit =
                crate::agent::inject::resolve_agent_unit(options.agent_unit.as_deref())?;
            crate::agent::inject::append_agent_ops(&mut plan, &binary, &unit)?;
        }

        #[cfg(not(feature = "agent"))]
        if options.inject_agent {
            anyhow::bail!("inject_agent requires guestkit built with --features agent");
        }

        fix_plan = Some(plan);
    }

    let root_cause = if options.explain {
        Some(infer_root_cause(&evidence, &boot_report))
    } else {
        None
    };

    Ok(MigrationPlanResult {
        target: target.to_string(),
        migration_score,
        bootability: boot_report,
        root_cause,
        fix_plan,
    })
}

/// Result of `run_repair_plan` (dry-run or applied).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepairPlanResult {
    pub dry_run: bool,
    pub before_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_score: Option<f64>,
    pub fix_plan: FixPlan,
    pub applied: bool,
    pub message: String,
}

/// Options for boot repair.
#[derive(Debug, Clone, Default)]
pub struct RepairOptions {
    pub dry_run: bool,
    pub verbose: bool,
    pub inject_agent: bool,
    #[cfg(feature = "agent")]
    pub agent_binary: Option<std::path::PathBuf>,
    #[cfg(feature = "agent")]
    pub agent_unit: Option<std::path::PathBuf>,
}

/// Generate or apply a boot repair plan (`--fix boot`).
pub fn run_repair_plan(image: &Path, options: &RepairOptions) -> Result<RepairPlanResult> {
    use crate::cli::plan::PlanApplicator;

    let (_, boot_report) = collect_assurance_data(image, BootTarget::Kvm, options.verbose)?;

    let has_boot_issues = !boot_report.blockers.is_empty() || !boot_report.warnings.is_empty();

    if !has_boot_issues && !options.inject_agent {
        return Ok(RepairPlanResult {
            dry_run: options.dry_run,
            before_score: boot_report.score,
            after_score: None,
            fix_plan: FixPlan::new(image.display().to_string(), "boot".to_string()),
            applied: false,
            message: "No boot issues detected — nothing to repair.".to_string(),
        });
    }

    let generator = PlanGenerator::new(image.display().to_string());
    #[cfg(feature = "agent")]
    let mut plan = if has_boot_issues {
        generator.from_boot_report(&boot_report, image)?
    } else {
        FixPlan::new(image.display().to_string(), "boot".to_string())
    };
    #[cfg(not(feature = "agent"))]
    let plan = if has_boot_issues {
        generator.from_boot_report(&boot_report, image)?
    } else {
        FixPlan::new(image.display().to_string(), "boot".to_string())
    };

    #[cfg(feature = "agent")]
    if options.inject_agent {
        let binary =
            crate::agent::inject::resolve_agent_binary(options.agent_binary.as_deref())?;
        let unit = crate::agent::inject::resolve_agent_unit(options.agent_unit.as_deref())?;
        crate::agent::inject::append_agent_ops(&mut plan, &binary, &unit)?;
    }

    #[cfg(not(feature = "agent"))]
    if options.inject_agent {
        anyhow::bail!("inject_agent requires guestkit built with --features agent");
    }

    let before_score = boot_report.score;

    if options.dry_run {
        let op_count = plan.operations.len();
        return Ok(RepairPlanResult {
            dry_run: true,
            before_score,
            after_score: None,
            fix_plan: plan,
            applied: false,
            message: format!("Dry run — {op_count} operation(s) would be applied"),
        });
    }

    if has_boot_issues {
        let applicator = PlanApplicator::new(image.to_str().unwrap().to_string(), false);
        let result = applicator.apply(&plan)?;

        if !result.success {
            anyhow::bail!("Repair failed: {}", result.message);
        }

        let (_, after_report) = collect_assurance_data(image, BootTarget::Kvm, options.verbose)?;

        #[cfg(feature = "agent")]
        if options.inject_agent {
            let binary =
                crate::agent::inject::resolve_agent_binary(options.agent_binary.as_deref())?;
            let unit = crate::agent::inject::resolve_agent_unit(options.agent_unit.as_deref())?;
            crate::agent::inject::inject_agent_into_image(
                image,
                &binary,
                &unit,
                false,
                options.verbose,
            )?;
        }

        return Ok(RepairPlanResult {
            dry_run: false,
            before_score,
            after_score: Some(after_report.score),
            fix_plan: plan,
            applied: true,
            message: format!(
                "Repair complete. Boot score: {:.0}% → {:.0}%",
                before_score, after_report.score
            ),
        });
    }

    #[cfg(feature = "agent")]
    if options.inject_agent {
        let binary =
            crate::agent::inject::resolve_agent_binary(options.agent_binary.as_deref())?;
        let unit = crate::agent::inject::resolve_agent_unit(options.agent_unit.as_deref())?;
        crate::agent::inject::inject_agent_into_image(
            image,
            &binary,
            &unit,
            false,
            options.verbose,
        )?;
    }

    Ok(RepairPlanResult {
        dry_run: false,
        before_score,
        after_score: None,
        fix_plan: plan,
        applied: options.inject_agent,
        message: if options.inject_agent {
            "GuestKit agent injected.".to_string()
        } else {
            "No changes applied.".to_string()
        },
    })
}
