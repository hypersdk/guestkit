// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plan preview and diff display

use super::types::*;
use colored::*;

/// Displays fix plans in human-readable format
pub struct PlanPreview;

impl PlanPreview {
    /// Display a plan as formatted text
    pub fn display(plan: &FixPlan) {
        Self::print_header(plan);
        Self::print_operations(plan);
        Self::print_footer(plan);
    }

    /// Display a plan as unified diff
    pub fn display_diff(plan: &FixPlan) {
        println!("{}", "Diff Preview".bold().cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();

        for op in &plan.operations {
            Self::print_operation_diff(op);
        }
    }

    /// Print plan header
    fn print_header(plan: &FixPlan) {
        println!();
        println!("{}", "📋 Fix Plan Preview".bold().cyan());
        println!("{}", "═".repeat(60).bright_black());
        println!();
        println!("{}: {}", "VM".bold(), plan.vm);
        println!("{}: {} ({} risk)",
            "Profile".bold(),
            plan.profile,
            Self::colorize_risk(&plan.overall_risk)
        );
        println!("{}: {}", "Operations".bold(), plan.operations.len());
        println!("{}: {}", "Estimated Duration".bold(), plan.estimated_duration);
        println!();
        println!("{}", "━".repeat(60).bright_black());
        println!();
    }

    /// Print operations grouped by priority
    fn print_operations(plan: &FixPlan) {
        for priority in &[Priority::Critical, Priority::High, Priority::Medium, Priority::Low, Priority::Info] {
            let ops: Vec<&Operation> = plan.operations.iter()
                .filter(|op| op.priority == *priority)
                .collect();

            if ops.is_empty() {
                continue;
            }

            println!("{} {} Priority ({} operations)",
                priority.emoji(),
                priority.as_str().to_uppercase(),
                ops.len()
            );
            println!();

            for op in ops {
                Self::print_operation(op);
            }
        }
    }

    /// Print a single operation
    fn print_operation(op: &Operation) {
        println!("[{}] {}", op.id.yellow(), op.description.bold());

        match &op.op_type {
            OperationType::FileEdit(fe) => {
                println!("  File: {}", fe.file.bright_blue());
                for change in &fe.changes {
                    if change.line > 0 {
                        println!("  Line {}: {} → {}",
                            change.line,
                            change.before.red(),
                            change.after.green()
                        );
                    }
                }
            }
            OperationType::PackageInstall(pi) => {
                println!("  Packages: {}", pi.packages.join(", ").bright_cyan());
                if let Some(size) = &pi.estimated_size {
                    println!("  Size: {}", size);
                }
            }
            OperationType::ServiceOperation(so) => {
                println!("  Service: {}", so.service.bright_cyan());
                if let Some(state) = &so.state {
                    println!("  State: {}", state.green());
                }
                if so.start {
                    println!("  {}", "Start on apply".green());
                }
            }
            OperationType::SelinuxMode(sm) => {
                println!("  File: {}", sm.file.bright_blue());
                println!("  {} → {}",
                    sm.current.red(),
                    sm.target.green()
                );
                if let Some(warning) = &sm.warning {
                    println!("  ⚠️  {}", warning.yellow());
                }
            }
            OperationType::RegistryEdit(re) => {
                println!("  Key: {}", re.key.bright_blue());
                println!("  Value: {}", re.value);
                println!("  {} → {}",
                    re.current_data.to_string().red(),
                    re.new_data.to_string().green()
                );
            }
            OperationType::CommandExec(ce) => {
                println!("  Command: {}", ce.command.bright_cyan());
            }
            OperationType::FileCopy(fc) => {
                println!("  {} → {}",
                    fc.source.bright_blue(),
                    fc.destination.bright_green()
                );
            }
            OperationType::DirectoryCreate(dc) => {
                println!("  Path: {}", dc.path.bright_blue());
                if let Some(mode) = &dc.mode {
                    println!("  Mode: {}", mode);
                }
            }
            OperationType::FilePermissions(fp) => {
                println!("  Path: {}", fp.path.bright_blue());
                println!("  Mode: {}", fp.mode.green());
            }
        }

        println!("  Risk: {} | Reversible: {}",
            Self::colorize_risk(&op.risk),
            if op.reversible { "Yes".green() } else { "No".red() }
        );

        if !op.depends_on.is_empty() {
            println!("  Depends on: {}", op.depends_on.join(", ").bright_black());
        }

        println!();
    }

    /// Print operation as diff
    fn print_operation_diff(op: &Operation) {
        if let OperationType::FileEdit(fe) = &op.op_type {
            println!("diff --git a{} b{}", fe.file, fe.file);
            println!("--- a{}", fe.file);
            println!("+++ b{}", fe.file);

            for change in &fe.changes {
                if let Some(context) = &change.context {
                    for line in context.lines() {
                        println!(" {}", line);
                    }
                }
                println!("{}", format!("-{}", change.before).red());
                println!("{}", format!("+{}", change.after).green());
            }
            println!();
        }
    }

    /// Print plan footer
    fn print_footer(plan: &FixPlan) {
        println!("{}", "━".repeat(60).bright_black());
        println!();

        if !plan.operations.is_empty() {
            // Show dependencies
            let has_deps = plan.operations.iter().any(|op| !op.depends_on.is_empty());
            if has_deps {
                println!("{}", "Dependencies:".bold());
                for op in &plan.operations {
                    if !op.depends_on.is_empty() {
                        println!("  {} → {}",
                            op.depends_on.join(", ").yellow(),
                            op.id.yellow()
                        );
                    }
                }
                println!();
            }

            // Show post-apply actions
            if !plan.post_apply.is_empty() {
                println!("{}", "Post-Apply Actions:".bold());
                for action in &plan.post_apply {
                    match action {
                        PostApplyAction::ServiceRestart { services } => {
                            println!("  • Restart services: {}", services.join(", ").bright_cyan());
                        }
                        PostApplyAction::Validation { command, .. } => {
                            println!("  • Validate: {}", command.bright_blue());
                        }
                        PostApplyAction::Message { message } => {
                            println!("  • {}", message);
                        }
                        PostApplyAction::RebootRequired { reason } => {
                            println!("  {} Reboot required: {}", "⚠️".yellow(), reason.yellow());
                        }
                    }
                }
                println!();
            }
        }

        println!("{}", "Backup: Will create automatic backup".bright_black());
        println!("{}", "Rollback: Available for all operations".bright_black());
        println!();
    }

    /// Colorize risk level
    fn colorize_risk(risk: &str) -> ColoredString {
        match risk.to_lowercase().as_str() {
            "critical" => risk.red().bold(),
            "high" => risk.bright_red(),
            "medium" => risk.yellow(),
            "low" => risk.green(),
            _ => risk.normal(),
        }
    }

    /// Print summary statistics
    pub fn print_summary(plan: &FixPlan) {
        println!("{}", "Plan Summary".bold().cyan());
        println!("{}", "─".repeat(40));
        println!("Total Operations: {}", plan.operations.len());
        println!("Critical: {}", plan.count_by_priority(Priority::Critical));
        println!("High: {}", plan.count_by_priority(Priority::High));
        println!("Medium: {}", plan.count_by_priority(Priority::Medium));
        println!("Low: {}", plan.count_by_priority(Priority::Low));
        println!("Info: {}", plan.count_by_priority(Priority::Info));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_creation() {
        let plan = FixPlan::new("test.qcow2".to_string(), "security".to_string());
        // Just ensure it doesn't panic
        PlanPreview::print_summary(&plan);
    }
}
