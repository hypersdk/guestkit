// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plan generator - converts profile findings into fix plans

use super::types::*;
use crate::cli::profiles::{ProfileReport, RiskLevel, ReportSection, Finding};
use anyhow::Result;

/// Generates fix plans from profile reports
#[allow(dead_code)]
pub struct PlanGenerator {
    vm_path: String,
}

#[allow(dead_code)]
impl PlanGenerator {
    /// Create a new plan generator
    pub fn new(vm_path: String) -> Self {
        Self { vm_path }
    }

    /// Generate a fix plan from a security profile report
    pub fn from_security_profile(&self, report: &ProfileReport) -> Result<FixPlan> {
        let mut plan = FixPlan::new(self.vm_path.clone(), "security".to_string());

        plan.overall_risk = match report.overall_risk {
            Some(RiskLevel::Critical) => "critical".to_string(),
            Some(RiskLevel::High) => "high".to_string(),
            Some(RiskLevel::Medium) => "medium".to_string(),
            Some(RiskLevel::Low) => "low".to_string(),
            Some(RiskLevel::Info) => "info".to_string(),
            None => "unknown".to_string(),
        };

        plan.metadata.description = Some(
            "Security hardening plan generated from security profile analysis".to_string()
        );
        plan.metadata.tags = vec!["security".to_string(), "automated".to_string()];

        // Convert findings to operations
        // For now, we use the message as remediation hint
        let mut op_counter = 1;
        for section in &report.sections {
            for finding in &section.findings {
                // Only create operations for findings with risk levels
                if finding.risk_level.is_some() {
                    let remediation = &finding.message;  // Use message as remediation hint
                    let operation = self.finding_to_operation(
                        &format!("sec-{:03}", op_counter),
                        finding,
                        remediation,
                    )?;
                    plan.add_operation(operation);
                    op_counter += 1;
                }
            }
        }

        // Estimate duration based on operation count
        plan.estimated_duration = Self::estimate_duration(plan.operations.len());

        // Add post-apply actions
        self.add_post_apply_actions(&mut plan);

        Ok(plan)
    }

    /// Convert a finding with remediation into an operation
    fn finding_to_operation(
        &self,
        id: &str,
        finding: &Finding,
        remediation: &str,
    ) -> Result<Operation> {
        let priority = match finding.risk_level {
            Some(RiskLevel::Critical) => Priority::Critical,
            Some(RiskLevel::High) => Priority::High,
            Some(RiskLevel::Medium) => Priority::Medium,
            Some(RiskLevel::Low) => Priority::Low,
            Some(RiskLevel::Info) | None => Priority::Info,
        };

        // Parse remediation text to determine operation type
        let op_type = self.parse_remediation(remediation)?;

        let risk_str = match finding.risk_level {
            Some(ref r) => r.to_string().to_lowercase(),
            None => "info".to_string(),
        };

        Ok(Operation {
            id: id.to_string(),
            op_type,
            priority,
            description: finding.item.clone(),
            risk: risk_str,
            reversible: true, // Most operations are reversible
            depends_on: Vec::new(),
            validation: None,
            undo: None,
        })
    }

    /// Parse remediation text to determine operation type
    /// This is a heuristic-based parser that looks for patterns
    fn parse_remediation(&self, remediation: &str) -> Result<OperationType> {
        let lower = remediation.to_lowercase();

        // SSH configuration changes
        if lower.contains("ssh") && lower.contains("permitrootlogin") {
            return Ok(OperationType::FileEdit(FileEdit {
                file: "/etc/ssh/sshd_config".to_string(),
                backup: true,
                changes: vec![FileChange {
                    line: 0, // Will be detected at apply time
                    before: "PermitRootLogin yes".to_string(),
                    after: "PermitRootLogin no".to_string(),
                    context: Some("# Authentication:\nPermitRootLogin no".to_string()),
                }],
            }));
        }

        // Firewall installation/enabling
        if lower.contains("firewall") && (lower.contains("enable") || lower.contains("install")) {
            if lower.contains("install") {
                return Ok(OperationType::PackageInstall(PackageInstall {
                    packages: vec!["firewalld".to_string()],
                    estimated_size: Some("~5MB".to_string()),
                }));
            } else {
                return Ok(OperationType::ServiceOperation(ServiceOperation {
                    service: "firewalld".to_string(),
                    state: Some("enabled".to_string()),
                    start: true,
                    restart: false,
                }));
            }
        }

        // SELinux mode changes
        if lower.contains("selinux") && lower.contains("enforcing") {
            return Ok(OperationType::SelinuxMode(SELinuxMode {
                file: "/etc/selinux/config".to_string(),
                current: "permissive".to_string(),
                target: "enforcing".to_string(),
                warning: Some("Requires reboot to take full effect".to_string()),
            }));
        }

        // fail2ban installation
        if lower.contains("fail2ban") {
            return Ok(OperationType::PackageInstall(PackageInstall {
                packages: vec!["fail2ban".to_string()],
                estimated_size: Some("~15MB".to_string()),
            }));
        }

        // AIDE installation
        if lower.contains("aide") && lower.contains("install") {
            return Ok(OperationType::PackageInstall(PackageInstall {
                packages: vec!["aide".to_string()],
                estimated_size: Some("~10MB".to_string()),
            }));
        }

        // Default: create a command execution operation
        Ok(OperationType::CommandExec(CommandExec {
            command: remediation.to_string(),
            expected_exit: 0,
            timeout: Some(300), // 5 minutes default
        }))
    }

    /// Add common post-apply actions
    fn add_post_apply_actions(&self, plan: &mut FixPlan) {
        // Check if we modified SSH config
        let has_ssh_changes = plan.operations.iter().any(|op| {
            matches!(&op.op_type, OperationType::FileEdit(fe) if fe.file.contains("sshd_config"))
        });

        if has_ssh_changes {
            plan.post_apply.push(PostApplyAction::ServiceRestart {
                services: vec!["sshd".to_string()],
            });
        }

        // Check if we enabled firewall
        let has_firewall = plan.operations.iter().any(|op| {
            matches!(&op.op_type, OperationType::ServiceOperation(so) if so.service == "firewalld")
        });

        if has_firewall {
            plan.post_apply.push(PostApplyAction::Validation {
                command: "firewall-cmd --state".to_string(),
                expected_output: Some("running".to_string()),
            });
        }

        // Check if we modified SELinux
        let has_selinux = plan.operations.iter().any(|op| {
            matches!(&op.op_type, OperationType::SelinuxMode(_))
        });

        if has_selinux {
            plan.post_apply.push(PostApplyAction::RebootRequired {
                reason: "SELinux mode change requires reboot".to_string(),
            });
        }
    }

    /// Estimate duration based on number of operations
    fn estimate_duration(op_count: usize) -> String {
        match op_count {
            0 => "0s".to_string(),
            1..=3 => "1-2 minutes".to_string(),
            4..=8 => "3-5 minutes".to_string(),
            9..=15 => "5-10 minutes".to_string(),
            _ => "10+ minutes".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        assert_eq!(generator.vm_path, "test.qcow2");
    }

    #[test]
    fn test_duration_estimation() {
        assert_eq!(PlanGenerator::estimate_duration(0), "0s");
        assert_eq!(PlanGenerator::estimate_duration(2), "1-2 minutes");
        assert_eq!(PlanGenerator::estimate_duration(5), "3-5 minutes");
        assert_eq!(PlanGenerator::estimate_duration(10), "5-10 minutes");
        assert_eq!(PlanGenerator::estimate_duration(20), "10+ minutes");
    }
}
