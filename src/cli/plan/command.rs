// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plan command - manage fix plans

use super::*;
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use colored::*;
use std::fs;
use std::path::Path;

#[derive(Debug, Args)]
pub struct PlanCommand {
    #[command(subcommand)]
    pub action: PlanAction,
}

#[derive(Debug, Subcommand)]
pub enum PlanAction {
    /// Preview a fix plan
    Preview {
        /// Path to plan file (YAML/JSON)
        #[arg(value_name = "PLAN_FILE")]
        plan_file: String,

        /// Show as unified diff
        #[arg(short, long)]
        diff: bool,

        /// Show summary only
        #[arg(short, long)]
        summary: bool,
    },

    /// Validate a fix plan
    Validate {
        /// Path to plan file (YAML/JSON)
        #[arg(value_name = "PLAN_FILE")]
        plan_file: String,

        /// VM disk path (overrides plan)
        #[arg(short, long)]
        vm: Option<String>,
    },

    /// Export a fix plan to different formats
    Export {
        /// Path to plan file (YAML/JSON)
        #[arg(value_name = "PLAN_FILE")]
        plan_file: String,

        /// Output file path
        #[arg(short, long, value_name = "FILE")]
        output: String,

        /// Export format
        #[arg(short, long, value_enum, default_value = "bash")]
        format: ExportFormat,
    },

    /// Apply a fix plan
    Apply {
        /// Path to plan file (YAML/JSON)
        #[arg(value_name = "PLAN_FILE")]
        plan_file: String,

        /// VM disk path (overrides plan)
        #[arg(short, long)]
        vm: Option<String>,

        /// Dry run (don't make changes)
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,

        /// Interactive mode (prompt for each operation)
        #[arg(short, long)]
        interactive: bool,

        /// Backup directory
        #[arg(short, long)]
        backup: Option<String>,
    },

    /// Rollback to a previous state
    Rollback {
        /// Backup directory path
        #[arg(value_name = "BACKUP_DIR")]
        backup_dir: String,

        /// VM disk path
        #[arg(short, long)]
        vm: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Generate a fix plan from a profile
    Generate {
        /// VM disk path
        #[arg(value_name = "VM_DISK")]
        vm_disk: String,

        /// Profile to use (security, compliance, hardening, etc.)
        #[arg(short, long, default_value = "security")]
        profile: String,

        /// Output plan file
        #[arg(short, long)]
        output: String,

        /// Format (yaml or json)
        #[arg(short, long, value_enum, default_value = "yaml")]
        format: PlanFileFormat,
    },

    /// Show plan statistics
    Stats {
        /// Path to plan file (YAML/JSON)
        #[arg(value_name = "PLAN_FILE")]
        plan_file: String,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum ExportFormat {
    Bash,
    Ansible,
    Json,
    Yaml,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PlanFileFormat {
    Yaml,
    Json,
}

impl PlanCommand {
    pub fn execute(&self) -> Result<()> {
        match &self.action {
            PlanAction::Preview { plan_file, diff, summary } => {
                self.preview_plan(plan_file, *diff, *summary)
            }
            PlanAction::Validate { plan_file, vm } => {
                self.validate_plan(plan_file, vm.as_deref())
            }
            PlanAction::Export { plan_file, output, format } => {
                self.export_plan(plan_file, output, format)
            }
            PlanAction::Apply { plan_file, vm, dry_run, yes, interactive, backup } => {
                self.apply_plan(plan_file, vm.as_deref(), *dry_run, *yes, *interactive, backup.as_deref())
            }
            PlanAction::Rollback { backup_dir, vm, yes } => {
                self.rollback(backup_dir, vm, *yes)
            }
            PlanAction::Generate { vm_disk, profile, output, format } => {
                self.generate_plan(vm_disk, profile, output, format)
            }
            PlanAction::Stats { plan_file } => {
                self.show_stats(plan_file)
            }
        }
    }

    fn load_plan(&self, path: &str) -> Result<FixPlan> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read plan file: {}", path))?;

        // Try YAML first, then JSON
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)
                .with_context(|| format!("Failed to parse YAML plan: {}", path))
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)
                .with_context(|| format!("Failed to parse JSON plan: {}", path))
        } else {
            // Auto-detect
            serde_yaml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content))
                .with_context(|| format!("Failed to parse plan file (tried YAML and JSON): {}", path))
        }
    }

    fn preview_plan(&self, plan_file: &str, diff: bool, summary: bool) -> Result<()> {
        let plan = self.load_plan(plan_file)?;

        if summary {
            PlanPreview::print_summary(&plan);
        } else if diff {
            PlanPreview::display_diff(&plan);
        } else {
            PlanPreview::display(&plan);
        }

        Ok(())
    }

    fn validate_plan(&self, plan_file: &str, vm_override: Option<&str>) -> Result<()> {
        let plan = self.load_plan(plan_file)?;
        let vm_path = vm_override.unwrap_or(&plan.vm);

        println!("{}", "Validating plan...".bold().cyan());
        println!();

        let applicator = PlanApplicator::new(vm_path.to_string(), true);
        let result = applicator.validate(&plan)?;

        if result.valid {
            println!("{}", "✓ Plan is valid".green().bold());

            if !result.warnings.is_empty() {
                println!();
                println!("{}", "Warnings:".yellow().bold());
                for warning in &result.warnings {
                    println!("  ⚠️  {}", warning.yellow());
                }
            }
        } else {
            println!("{}", "✗ Plan validation failed".red().bold());
            println!();
            println!("{}", "Errors:".red().bold());
            for error in &result.errors {
                println!("  ✗ {}", error.red());
            }

            if !result.warnings.is_empty() {
                println!();
                println!("{}", "Warnings:".yellow().bold());
                for warning in &result.warnings {
                    println!("  ⚠️  {}", warning.yellow());
                }
            }

            anyhow::bail!("Plan validation failed");
        }

        Ok(())
    }

    fn export_plan(&self, plan_file: &str, output: &str, format: &ExportFormat) -> Result<()> {
        let plan = self.load_plan(plan_file)?;

        println!("Exporting plan to {} format...",
            match format {
                ExportFormat::Bash => "bash",
                ExportFormat::Ansible => "ansible",
                ExportFormat::Json => "JSON",
                ExportFormat::Yaml => "YAML",
            }.cyan()
        );

        let content = match format {
            ExportFormat::Bash => PlanExporter::to_bash(&plan)?,
            ExportFormat::Ansible => PlanExporter::to_ansible(&plan)?,
            ExportFormat::Json => PlanExporter::to_json(&plan)?,
            ExportFormat::Yaml => PlanExporter::to_yaml(&plan)?,
        };

        fs::write(output, content)
            .with_context(|| format!("Failed to write output file: {}", output))?;

        println!("{} Exported to: {}", "✓".green(), output.bright_blue());

        Ok(())
    }

    fn apply_plan(
        &self,
        plan_file: &str,
        vm_override: Option<&str>,
        dry_run: bool,
        yes: bool,
        interactive: bool,
        _backup_dir: Option<&str>,
    ) -> Result<()> {
        let plan = self.load_plan(plan_file)?;
        let vm_path = vm_override.unwrap_or(&plan.vm);

        // Validate first
        let applicator = PlanApplicator::new(vm_path.to_string(), true);
        let validation = applicator.validate(&plan)?;

        if !validation.valid {
            println!("{}", "✗ Plan validation failed".red().bold());
            for error in &validation.errors {
                println!("  ✗ {}", error.red());
            }
            anyhow::bail!("Cannot apply invalid plan");
        }

        // Show preview
        println!();
        PlanPreview::display(&plan);
        println!();

        // Confirm unless --yes or --dry-run
        if !yes && !dry_run && !interactive {
            print!("{}", "Apply this plan? [y/N] ".yellow().bold());
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        // Apply
        let applicator = PlanApplicator::new(vm_path.to_string(), dry_run);

        if dry_run {
            println!();
            println!("{}", "DRY RUN MODE - No changes will be made".yellow().bold());
            println!();
        }

        let result = applicator.apply(&plan)?;

        println!();
        if result.success {
            println!("{}", "✓ Plan applied successfully".green().bold());
            println!("  Operations applied: {}", result.operations_applied);
            println!("  Operations skipped: {}", result.operations_skipped);
        } else {
            println!("{}", "✗ Plan application failed".red().bold());
            println!("  Operations applied: {}", result.operations_applied);
            println!("  Operations failed: {}", result.operations_failed);
            println!("  Message: {}", result.message);
        }

        Ok(())
    }

    fn rollback(&self, backup_dir: &str, vm: &str, yes: bool) -> Result<()> {
        if !Path::new(backup_dir).exists() {
            anyhow::bail!("Backup directory not found: {}", backup_dir);
        }

        println!("{}", "Rollback Operation".bold().red());
        println!("{}", "═".repeat(60).bright_black());
        println!("Backup: {}", backup_dir.bright_blue());
        println!("VM: {}", vm.bright_blue());
        println!("{}", "═".repeat(60).bright_black());
        println!();
        println!("{}", "WARNING: This will restore files from backup.".yellow().bold());
        println!("{}", "Any changes made after the backup will be lost.".yellow());
        println!();

        if !yes {
            print!("{}", "Continue with rollback? [y/N] ".red().bold());
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }

        let applicator = PlanApplicator::new(vm.to_string(), false);
        applicator.rollback(backup_dir)?;

        println!();
        println!("{}", "✓ Rollback completed successfully".green().bold());

        Ok(())
    }

    fn generate_plan(
        &self,
        vm_disk: &str,
        profile: &str,
        output: &str,
        _format: &PlanFileFormat,
    ) -> Result<()> {
        println!("Generating {} plan for {}...", profile.cyan(), vm_disk.bright_blue());

        // TODO: Actually run the profile and generate plan
        // For now, create a placeholder
        anyhow::bail!(
            "Plan generation not yet implemented. Use 'guestctl profile {} {} --plan {}' instead.",
            profile, vm_disk, output
        );
    }

    fn show_stats(&self, plan_file: &str) -> Result<()> {
        let plan = self.load_plan(plan_file)?;

        println!();
        println!("{}", "Plan Statistics".bold().cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();
        println!("File: {}", plan_file.bright_blue());
        println!("VM: {}", plan.vm);
        println!("Profile: {}", plan.profile);
        println!("Generated: {}", plan.generated);
        println!("Risk: {}", match plan.overall_risk.as_str() {
            "critical" => plan.overall_risk.red().bold(),
            "high" => plan.overall_risk.bright_red(),
            "medium" => plan.overall_risk.yellow(),
            "low" => plan.overall_risk.green(),
            _ => plan.overall_risk.normal(),
        });
        println!("Duration: {}", plan.estimated_duration);
        println!();
        println!("{}", "Operations:".bold());
        println!("  Total: {}", plan.operations.len());
        println!("  Critical: {}", plan.count_by_priority(Priority::Critical));
        println!("  High: {}", plan.count_by_priority(Priority::High));
        println!("  Medium: {}", plan.count_by_priority(Priority::Medium));
        println!("  Low: {}", plan.count_by_priority(Priority::Low));
        println!("  Info: {}", plan.count_by_priority(Priority::Info));
        println!();
        println!("Reversible: {}", if plan.metadata.reversible {
            "Yes".green()
        } else {
            "No".red()
        });
        println!("Review Required: {}", if plan.metadata.review_required {
            "Yes".yellow()
        } else {
            "No".green()
        });
        println!();

        if !plan.post_apply.is_empty() {
            println!("{}", "Post-Apply Actions:".bold());
            for (i, action) in plan.post_apply.iter().enumerate() {
                match action {
                    PostApplyAction::ServiceRestart { services } => {
                        println!("  {}. Restart services: {}", i + 1, services.join(", "));
                    }
                    PostApplyAction::Validation { command, .. } => {
                        println!("  {}. Validate: {}", i + 1, command);
                    }
                    PostApplyAction::Message { message } => {
                        println!("  {}. {}", i + 1, message);
                    }
                    PostApplyAction::RebootRequired { reason } => {
                        println!("  {}. Reboot required: {}", i + 1, reason);
                    }
                }
            }
            println!();
        }

        Ok(())
    }
}
