// SPDX-License-Identifier: Apache-2.0
//! Individual boot checks.

use super::report::{CheckResult, CheckSeverity};
use crate::evidence::EvidenceSnapshot;

pub trait BootCheck {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn weight(&self) -> f64;
    fn run(&self, evidence: &EvidenceSnapshot, target: &str) -> CheckResult;
}

pub struct FstabUuidCheck;
pub struct CrypttabCheck;
pub struct InitramfsCheck;
pub struct GrubConsistencyCheck;
pub struct EfiCheck;
pub struct VirtioReadinessCheck;
pub struct NicRenameCheck;
pub struct CloudInitCheck;
pub struct SelinuxRelabelCheck;
pub struct VmToolsRemnantsCheck;
pub struct SystemdStaticCheck;

impl BootCheck for FstabUuidCheck {
    fn id(&self) -> &str {
        "BOOT-001"
    }
    fn name(&self) -> &str {
        "Fstab UUID consistency"
    }
    fn weight(&self) -> f64 {
        15.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        let known_uuids: std::collections::HashSet<_> = evidence
            .storage
            .partition_uuids
            .iter()
            .map(|p| p.uuid.to_lowercase())
            .collect();

        let mut mismatches = Vec::new();
        for entry in &evidence.storage.fstab_entries {
            if entry.device.starts_with("UUID=") {
                let uuid = entry.device.trim_start_matches("UUID=").to_lowercase();
                if !known_uuids.is_empty() && !known_uuids.contains(&uuid) {
                    mismatches.push(entry.device.clone());
                }
            }
        }

        let passed = mismatches.is_empty();
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity: if passed {
                CheckSeverity::Info
            } else {
                CheckSeverity::Blocker
            },
            message: if passed {
                "All fstab UUID references match partition table".to_string()
            } else {
                format!("Fstab UUID mismatches: {}", mismatches.join(", "))
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for CrypttabCheck {
    fn id(&self) -> &str {
        "BOOT-002"
    }
    fn name(&self) -> &str {
        "Crypttab validation"
    }
    fn weight(&self) -> f64 {
        10.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        let issues: Vec<String> = evidence
            .storage
            .crypttab_entries
            .iter()
            .filter(|e| e.device.is_empty() || e.name.is_empty())
            .map(|e| format!("{}:{}", e.name, e.device))
            .collect();

        let passed = issues.is_empty();
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity: if passed {
                CheckSeverity::Info
            } else {
                CheckSeverity::Blocker
            },
            message: if passed {
                "Crypttab entries appear valid".to_string()
            } else {
                format!("Invalid crypttab entries: {}", issues.join(", "))
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for InitramfsCheck {
    fn id(&self) -> &str {
        "BOOT-003"
    }
    fn name(&self) -> &str {
        "Initramfs presence"
    }
    fn weight(&self) -> f64 {
        12.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        let has_initramfs = !evidence.boot.initramfs_paths.is_empty();
        let has_kernel = !evidence.boot.kernel_paths.is_empty();
        let passed = has_initramfs && has_kernel;
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity: if passed {
                CheckSeverity::Info
            } else if !has_kernel {
                CheckSeverity::Blocker
            } else {
                CheckSeverity::Warning
            },
            message: if passed {
                format!(
                    "Found {} kernel(s) and {} initramfs",
                    evidence.boot.kernel_paths.len(),
                    evidence.boot.initramfs_paths.len()
                )
            } else if !has_kernel {
                "No kernel images found in /boot".to_string()
            } else {
                "Kernel found but no initramfs detected".to_string()
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for GrubConsistencyCheck {
    fn id(&self) -> &str {
        "BOOT-004"
    }
    fn name(&self) -> &str {
        "GRUB configuration"
    }
    fn weight(&self) -> f64 {
        10.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        let has_grub = evidence.boot.grub_cfg_path.is_some()
            || evidence.boot.bootloader.to_lowercase().contains("grub");
        let passed = has_grub;
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity: if passed {
                CheckSeverity::Info
            } else {
                CheckSeverity::Warning
            },
            message: if passed {
                format!(
                    "Bootloader: {} (default: {})",
                    evidence.boot.bootloader, evidence.boot.default_entry
                )
            } else {
                "No GRUB configuration detected".to_string()
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for EfiCheck {
    fn id(&self) -> &str {
        "BOOT-005"
    }
    fn name(&self) -> &str {
        "EFI boot support"
    }
    fn weight(&self) -> f64 {
        5.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed: true,
            severity: CheckSeverity::Info,
            message: if evidence.boot.efi_present {
                "EFI system partition detected".to_string()
            } else {
                "Legacy BIOS boot mode (no EFI partition detected)".to_string()
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for VirtioReadinessCheck {
    fn id(&self) -> &str {
        "BOOT-006"
    }
    fn name(&self) -> &str {
        "Virtio driver readiness"
    }
    fn weight(&self) -> f64 {
        15.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, target: &str) -> CheckResult {
        if !matches!(target, "kvm" | "proxmox" | "qemu" | "kubevirt") {
            return CheckResult {
                id: self.id().to_string(),
                name: self.name().to_string(),
                passed: true,
                severity: CheckSeverity::Info,
                message: "Virtio check skipped for non-KVM target".to_string(),
                weight: 0.0,
            };
        }

        let modules = &evidence.boot.loaded_modules;
        let has_virtio_blk = modules.iter().any(|m| m.contains("virtio_blk"));
        let has_virtio_net = modules.iter().any(|m| m.contains("virtio_net"));
        let has_virtio =
            has_virtio_blk || has_virtio_net || modules.iter().any(|m| m.contains("virtio"));

        let vmware_tools = evidence
            .vm_tools
            .detected
            .iter()
            .any(|t| t.contains("vmware"));
        let passed = has_virtio || !vmware_tools;

        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity: if passed {
                CheckSeverity::Info
            } else {
                CheckSeverity::Warning
            },
            message: if has_virtio {
                "Virtio modules available".to_string()
            } else if vmware_tools {
                "VMware tools detected but no virtio modules — driver injection may be required"
                    .to_string()
            } else {
                "Virtio modules not detected in module tree".to_string()
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for NicRenameCheck {
    fn id(&self) -> &str {
        "BOOT-007"
    }
    fn name(&self) -> &str {
        "NIC rename prediction"
    }
    fn weight(&self) -> f64 {
        5.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, target: &str) -> CheckResult {
        if !matches!(target, "kvm" | "proxmox" | "cloud" | "kubevirt") {
            return CheckResult {
                id: self.id().to_string(),
                name: self.name().to_string(),
                passed: true,
                severity: CheckSeverity::Info,
                message: "NIC rename check skipped".to_string(),
                weight: 0.0,
            };
        }

        let has_persistent = !evidence.network.udev_persistent_net.is_empty();
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed: true,
            severity: CheckSeverity::Info,
            message: if has_persistent {
                "Persistent udev net rules present — interface names may change on new hardware"
                    .to_string()
            } else {
                "No persistent net rules — predict ens3/eth0 rename on migration".to_string()
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for CloudInitCheck {
    fn id(&self) -> &str {
        "BOOT-008"
    }
    fn name(&self) -> &str {
        "Cloud-init conflicts"
    }
    fn weight(&self) -> f64 {
        8.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        if !evidence.boot.cloud_init_present {
            return CheckResult {
                id: self.id().to_string(),
                name: self.name().to_string(),
                passed: true,
                severity: CheckSeverity::Info,
                message: "Cloud-init not configured".to_string(),
                weight: self.weight(),
            };
        }
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed: true,
            severity: CheckSeverity::Warning,
            message: "Cloud-init present — verify instance-id and datasource on target platform"
                .to_string(),
            weight: self.weight(),
        }
    }
}

impl BootCheck for SelinuxRelabelCheck {
    fn id(&self) -> &str {
        "BOOT-009"
    }
    fn name(&self) -> &str {
        "SELinux relabel requirement"
    }
    fn weight(&self) -> f64 {
        8.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        let needs_relabel = evidence.boot.pending_relabel
            || evidence.security.selinux.to_lowercase() == "enforcing";
        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed: !evidence.boot.pending_relabel,
            severity: if evidence.boot.pending_relabel {
                CheckSeverity::Warning
            } else {
                CheckSeverity::Info
            },
            message: if evidence.boot.pending_relabel {
                "/.autorelabel present — first boot will trigger SELinux relabel".to_string()
            } else if needs_relabel {
                "SELinux enforcing — disk UUID changes may require relabel".to_string()
            } else {
                "No SELinux relabel pending".to_string()
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for VmToolsRemnantsCheck {
    fn id(&self) -> &str {
        "BOOT-010"
    }
    fn name(&self) -> &str {
        "Hypervisor remnants"
    }
    fn weight(&self) -> f64 {
        7.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, target: &str) -> CheckResult {
        if !matches!(target, "kvm" | "proxmox" | "kubevirt") {
            return CheckResult {
                id: self.id().to_string(),
                name: self.name().to_string(),
                passed: true,
                severity: CheckSeverity::Info,
                message: "Hypervisor remnant check skipped".to_string(),
                weight: 0.0,
            };
        }

        let remnants: Vec<_> = evidence
            .vm_tools
            .detected
            .iter()
            .filter(|t| t.contains("vmware") || t.contains("hyper-v") || t.contains("virtualbox"))
            .cloned()
            .collect();

        let win_remnants = evidence
            .windows
            .as_ref()
            .map(|w| w.hypervisor_remnants.clone())
            .unwrap_or_default();

        let all: Vec<_> = remnants.into_iter().chain(win_remnants).collect();
        let passed = all.is_empty();

        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity: if passed {
                CheckSeverity::Info
            } else {
                CheckSeverity::Warning
            },
            message: if passed {
                "No source hypervisor tools detected".to_string()
            } else {
                format!("Hypervisor remnants: {}", all.join(", "))
            },
            weight: self.weight(),
        }
    }
}

impl BootCheck for SystemdStaticCheck {
    fn id(&self) -> &str {
        "BOOT-011"
    }
    fn name(&self) -> &str {
        "Systemd static analysis"
    }
    fn weight(&self) -> f64 {
        8.0
    }
    fn run(&self, evidence: &EvidenceSnapshot, _target: &str) -> CheckResult {
        let Some(systemd) = evidence.systemd.as_ref() else {
            return CheckResult {
                id: self.id().to_string(),
                name: self.name().to_string(),
                passed: true,
                severity: CheckSeverity::Info,
                message: "No systemd evidence collected".to_string(),
                weight: 0.0,
            };
        };

        let critical: Vec<_> = systemd
            .problem_hints
            .iter()
            .filter(|h| h.severity == crate::evidence::SystemdProblemSeverity::Critical)
            .collect();
        let warnings: Vec<_> = systemd
            .problem_hints
            .iter()
            .filter(|h| h.severity == crate::evidence::SystemdProblemSeverity::Warning)
            .collect();

        let passed = critical.is_empty();
        let severity = if !critical.is_empty() {
            CheckSeverity::Blocker
        } else if !warnings.is_empty() {
            CheckSeverity::Warning
        } else {
            CheckSeverity::Info
        };

        let message = if !critical.is_empty() {
            format!(
                "{} critical systemd issue(s): {}",
                critical.len(),
                critical
                    .iter()
                    .take(3)
                    .map(|h| format!("{} ({})", h.unit, h.code))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else if !warnings.is_empty() {
            format!(
                "{} systemd warning(s): {}",
                warnings.len(),
                warnings
                    .iter()
                    .take(3)
                    .map(|h| format!("{} ({})", h.unit, h.code))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            format!("No systemd static issues in {} units", systemd.unit_count)
        };

        CheckResult {
            id: self.id().to_string(),
            name: self.name().to_string(),
            passed,
            severity,
            message,
            weight: self.weight(),
        }
    }
}

pub fn all_checks() -> Vec<Box<dyn BootCheck>> {
    vec![
        Box::new(FstabUuidCheck),
        Box::new(CrypttabCheck),
        Box::new(InitramfsCheck),
        Box::new(GrubConsistencyCheck),
        Box::new(EfiCheck),
        Box::new(VirtioReadinessCheck),
        Box::new(NicRenameCheck),
        Box::new(CloudInitCheck),
        Box::new(SelinuxRelabelCheck),
        Box::new(VmToolsRemnantsCheck),
        Box::new(SystemdStaticCheck),
    ]
}
