// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plan generator - converts profile findings into fix plans

use super::types::*;
use crate::cli::profiles::{ProfileReport, RiskLevel, ReportSection, Finding};
use anyhow::Result;

/// Generates fix plans from profile reports
pub struct PlanGenerator {
    vm_path: String,
}

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

        let risk = match finding.risk_level {
            Some(RiskLevel::Critical) => Priority::Critical,
            Some(RiskLevel::High) => Priority::High,
            Some(RiskLevel::Medium) => Priority::Medium,
            Some(RiskLevel::Low) => Priority::Low,
            Some(RiskLevel::Info) | None => Priority::Info,
        };

        Ok(Operation {
            id: id.to_string(),
            op_type,
            priority,
            description: finding.item.clone(),
            risk,
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

    /// Generate a fix plan from bootability report blockers/warnings
    pub fn from_boot_report(
        &self,
        boot: &crate::boot::BootabilityReport,
        image: &std::path::Path,
    ) -> Result<FixPlan> {
        let mut plan = FixPlan::new(
            image.display().to_string(),
            "boot-repair".to_string(),
        );
        plan.metadata.description = Some(
            "Automated boot repair plan from doctor analysis".to_string(),
        );
        plan.metadata.tags = vec!["boot".to_string(), "doctor".to_string()];

        let mut op_counter = 1;
        for finding in boot
            .blockers
            .iter()
            .chain(boot.warnings.iter())
        {
            let remediation = finding
                .remediation
                .clone()
                .unwrap_or_else(|| finding.message.clone());
            if let Ok(op_type) = self.parse_remediation(&remediation) {
                plan.add_operation(Operation {
                    id: format!("boot-{:03}", op_counter),
                    op_type,
                    priority: Priority::High,
                    description: finding.title.clone(),
                    risk: Priority::Medium,
                    reversible: true,
                    depends_on: vec![],
                    validation: None,
                    undo: None,
                });
                op_counter += 1;
            }
        }

        plan.estimated_duration = Self::estimate_duration(plan.operations.len());
        self.add_post_apply_actions(&mut plan);
        Ok(plan)
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
    use crate::cli::profiles::FindingStatus;

    fn create_test_finding(risk: RiskLevel, message: &str) -> Finding {
        Finding {
            item: "Test finding".to_string(),
            status: FindingStatus::Fail,
            message: message.to_string(),
            risk_level: Some(risk),
        }
    }

    fn create_test_report() -> ProfileReport {
        ProfileReport {
            profile_name: "security".to_string(),
            overall_risk: Some(RiskLevel::High),
            sections: vec![
                ReportSection {
                    title: "SSH Configuration".to_string(),
                    findings: vec![
                        create_test_finding(RiskLevel::High, "Disable PermitRootLogin in SSH config"),
                        create_test_finding(RiskLevel::Medium, "Enable firewall service"),
                    ],
                },
            ],
            summary: None,
        }
    }

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

    #[test]
    fn test_duration_estimation_boundaries() {
        assert_eq!(PlanGenerator::estimate_duration(1), "1-2 minutes");
        assert_eq!(PlanGenerator::estimate_duration(3), "1-2 minutes");
        assert_eq!(PlanGenerator::estimate_duration(4), "3-5 minutes");
        assert_eq!(PlanGenerator::estimate_duration(8), "3-5 minutes");
        assert_eq!(PlanGenerator::estimate_duration(9), "5-10 minutes");
        assert_eq!(PlanGenerator::estimate_duration(15), "5-10 minutes");
        assert_eq!(PlanGenerator::estimate_duration(16), "10+ minutes");
    }

    #[test]
    fn test_from_security_profile() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let report = create_test_report();
        let plan = generator.from_security_profile(&report).unwrap();

        assert_eq!(plan.vm, "test.qcow2");
        assert_eq!(plan.profile, "security");
        assert_eq!(plan.overall_risk, "high");
        assert!(!plan.operations.is_empty());
    }

    #[test]
    fn test_from_security_profile_risk_levels() {
        let generator = PlanGenerator::new("test.qcow2".to_string());

        let mut report = create_test_report();
        report.overall_risk = Some(RiskLevel::Critical);
        let plan = generator.from_security_profile(&report).unwrap();
        assert_eq!(plan.overall_risk, "critical");

        report.overall_risk = Some(RiskLevel::Medium);
        let plan = generator.from_security_profile(&report).unwrap();
        assert_eq!(plan.overall_risk, "medium");

        report.overall_risk = Some(RiskLevel::Low);
        let plan = generator.from_security_profile(&report).unwrap();
        assert_eq!(plan.overall_risk, "low");

        report.overall_risk = Some(RiskLevel::Info);
        let plan = generator.from_security_profile(&report).unwrap();
        assert_eq!(plan.overall_risk, "info");

        report.overall_risk = None;
        let plan = generator.from_security_profile(&report).unwrap();
        assert_eq!(plan.overall_risk, "unknown");
    }

    #[test]
    fn test_from_security_profile_metadata() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let report = create_test_report();
        let plan = generator.from_security_profile(&report).unwrap();

        assert!(plan.metadata.description.is_some());
        assert!(plan.metadata.tags.contains(&"security".to_string()));
        assert!(plan.metadata.tags.contains(&"automated".to_string()));
    }

    #[test]
    fn test_parse_remediation_ssh() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Disable PermitRootLogin in SSH config").unwrap();

        match op_type {
            OperationType::FileEdit(fe) => {
                assert!(fe.file.contains("sshd_config"));
                assert!(fe.backup);
            }
            _ => panic!("Expected FileEdit operation"),
        }
    }

    #[test]
    fn test_parse_remediation_firewall_install() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Install firewall for security").unwrap();

        match op_type {
            OperationType::PackageInstall(pi) => {
                assert!(pi.packages.contains(&"firewalld".to_string()));
            }
            _ => panic!("Expected PackageInstall operation"),
        }
    }

    #[test]
    fn test_parse_remediation_firewall_enable() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Enable firewall service").unwrap();

        match op_type {
            OperationType::ServiceOperation(so) => {
                assert_eq!(so.service, "firewalld");
                assert_eq!(so.state, Some("enabled".to_string()));
                assert!(so.start);
            }
            _ => panic!("Expected ServiceOperation"),
        }
    }

    #[test]
    fn test_parse_remediation_selinux() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Set SELinux to enforcing mode").unwrap();

        match op_type {
            OperationType::SelinuxMode(sm) => {
                assert!(sm.file.contains("selinux"));
                assert_eq!(sm.target, "enforcing");
                assert!(sm.warning.is_some());
            }
            _ => panic!("Expected SelinuxMode operation"),
        }
    }

    #[test]
    fn test_parse_remediation_fail2ban() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Install fail2ban for brute force protection").unwrap();

        match op_type {
            OperationType::PackageInstall(pi) => {
                assert!(pi.packages.contains(&"fail2ban".to_string()));
            }
            _ => panic!("Expected PackageInstall operation"),
        }
    }

    #[test]
    fn test_parse_remediation_aide() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Install AIDE for file integrity").unwrap();

        match op_type {
            OperationType::PackageInstall(pi) => {
                assert!(pi.packages.contains(&"aide".to_string()));
            }
            _ => panic!("Expected PackageInstall operation"),
        }
    }

    #[test]
    fn test_parse_remediation_default_command() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let op_type = generator.parse_remediation("Run custom security check").unwrap();

        match op_type {
            OperationType::CommandExec(ce) => {
                assert_eq!(ce.command, "Run custom security check");
                assert_eq!(ce.expected_exit, 0);
                assert_eq!(ce.timeout, Some(300));
            }
            _ => panic!("Expected CommandExec operation"),
        }
    }

    #[test]
    fn test_finding_to_operation_priority_mapping() {
        let generator = PlanGenerator::new("test.qcow2".to_string());

        let finding_critical = create_test_finding(RiskLevel::Critical, "Test");
        let op = generator.finding_to_operation("op-1", &finding_critical, "Test").unwrap();
        assert_eq!(op.priority, Priority::Critical);

        let finding_high = create_test_finding(RiskLevel::High, "Test");
        let op = generator.finding_to_operation("op-2", &finding_high, "Test").unwrap();
        assert_eq!(op.priority, Priority::High);

        let finding_medium = create_test_finding(RiskLevel::Medium, "Test");
        let op = generator.finding_to_operation("op-3", &finding_medium, "Test").unwrap();
        assert_eq!(op.priority, Priority::Medium);

        let finding_low = create_test_finding(RiskLevel::Low, "Test");
        let op = generator.finding_to_operation("op-4", &finding_low, "Test").unwrap();
        assert_eq!(op.priority, Priority::Low);

        let finding_info = create_test_finding(RiskLevel::Info, "Test");
        let op = generator.finding_to_operation("op-5", &finding_info, "Test").unwrap();
        assert_eq!(op.priority, Priority::Info);
    }

    #[test]
    fn test_finding_to_operation_structure() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let finding = create_test_finding(RiskLevel::High, "Enable firewall");
        let op = generator.finding_to_operation("op-test", &finding, "Enable firewall").unwrap();

        assert_eq!(op.id, "op-test");
        assert_eq!(op.description, "Test finding");
        assert_eq!(op.risk, Priority::High);
        assert!(op.reversible);
        assert!(op.depends_on.is_empty());
    }

    #[test]
    fn test_add_post_apply_actions_ssh() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let mut plan = FixPlan::new("test.qcow2".to_string(), "security".to_string());

        plan.add_operation(Operation {
            id: "op-ssh".to_string(),
            op_type: OperationType::FileEdit(FileEdit {
                file: "/etc/ssh/sshd_config".to_string(),
                backup: true,
                changes: vec![],
            }),
            priority: Priority::High,
            description: "SSH config".to_string(),
            risk: Priority::Medium,
            reversible: true,
            depends_on: vec![],
            validation: None,
            undo: None,
        });

        generator.add_post_apply_actions(&mut plan);

        assert!(!plan.post_apply.is_empty());
        let has_ssh_restart = plan.post_apply.iter().any(|action| {
            matches!(action, PostApplyAction::ServiceRestart { services } if services.contains(&"sshd".to_string()))
        });
        assert!(has_ssh_restart);
    }

    #[test]
    fn test_add_post_apply_actions_firewall() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let mut plan = FixPlan::new("test.qcow2".to_string(), "security".to_string());

        plan.add_operation(Operation {
            id: "op-fw".to_string(),
            op_type: OperationType::ServiceOperation(ServiceOperation {
                service: "firewalld".to_string(),
                state: Some("enabled".to_string()),
                start: true,
                restart: false,
            }),
            priority: Priority::High,
            description: "Enable firewall".to_string(),
            risk: Priority::Low,
            reversible: true,
            depends_on: vec![],
            validation: None,
            undo: None,
        });

        generator.add_post_apply_actions(&mut plan);

        let has_firewall_validation = plan.post_apply.iter().any(|action| {
            matches!(action, PostApplyAction::Validation { command, .. } if command.contains("firewall-cmd"))
        });
        assert!(has_firewall_validation);
    }

    #[test]
    fn test_add_post_apply_actions_selinux() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let mut plan = FixPlan::new("test.qcow2".to_string(), "security".to_string());

        plan.add_operation(Operation {
            id: "op-sel".to_string(),
            op_type: OperationType::SelinuxMode(SELinuxMode {
                file: "/etc/selinux/config".to_string(),
                current: "permissive".to_string(),
                target: "enforcing".to_string(),
                warning: None,
            }),
            priority: Priority::Critical,
            description: "Set SELinux".to_string(),
            risk: Priority::Medium,
            reversible: true,
            depends_on: vec![],
            validation: None,
            undo: None,
        });

        generator.add_post_apply_actions(&mut plan);

        let has_reboot = plan.post_apply.iter().any(|action| {
            matches!(action, PostApplyAction::RebootRequired { .. })
        });
        assert!(has_reboot);
    }

    #[test]
    fn test_from_security_profile_no_findings() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let report = ProfileReport {
            profile_name: "security".to_string(),
            overall_risk: Some(RiskLevel::Info),
            sections: vec![],
            summary: None,
        };

        let plan = generator.from_security_profile(&report).unwrap();
        assert_eq!(plan.operations.len(), 0);
        assert_eq!(plan.estimated_duration, "0s");
    }

    #[test]
    fn test_from_security_profile_filters_no_risk() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let report = ProfileReport {
            profile_name: "security".to_string(),
            overall_risk: Some(RiskLevel::Medium),
            sections: vec![
                ReportSection {
                    title: "Test Section".to_string(),
                    findings: vec![
                        Finding {
                            item: "Finding without risk".to_string(),
                            status: FindingStatus::Pass,
                            message: "No action needed".to_string(),
                            risk_level: None,
                        },
                    ],
                },
            ],
            summary: None,
        };

        let plan = generator.from_security_profile(&report).unwrap();
        // Should skip findings without risk_level
        assert_eq!(plan.operations.len(), 0);
    }

    #[test]
    fn test_from_security_profile_operation_ids() {
        let generator = PlanGenerator::new("test.qcow2".to_string());
        let report = create_test_report();
        let plan = generator.from_security_profile(&report).unwrap();

        // Check that operation IDs are sequential
        for (i, op) in plan.operations.iter().enumerate() {
            assert_eq!(op.id, format!("sec-{:03}", i + 1));
        }
    }
}
