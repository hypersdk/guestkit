// SPDX-License-Identifier: Apache-2.0
//! Migration-readiness checks: general (MIG-G), Linux (MIG-L), Windows (MIG-W).

use super::readiness::{
    AssessContext, MigrationCheckResult as Check, ReadinessCategory as Cat, RemediationHint as Hint,
};
use crate::boot::report::CheckSeverity;
use crate::evidence::EvidenceSnapshot;

pub fn run_all_checks(ev: &EvidenceSnapshot, ctx: &AssessContext) -> Vec<Check> {
    let mut results = general_checks(ev, ctx);
    if is_windows(ev) {
        results.extend(windows_checks(ev, ctx));
    } else {
        results.extend(linux_checks(ev, ctx));
    }
    results
}

fn is_windows(ev: &EvidenceSnapshot) -> bool {
    ev.os.os_type.to_lowercase().contains("windows")
}

// --- MIG-G: general ---

fn general_checks(ev: &EvidenceSnapshot, _ctx: &AssessContext) -> Vec<Check> {
    let mut out = Vec::new();

    // MIG-G-001: OS identified
    out.push(if ev.os.os_type.is_empty() {
        Check::fail(
            "MIG-G-001",
            "OS identification",
            Cat::Boot,
            8.0,
            CheckSeverity::Blocker,
            "operating system could not be identified",
            None,
        )
    } else {
        Check::pass(
            "MIG-G-001",
            "OS identification",
            Cat::Boot,
            8.0,
            format!("{} {} ({})", ev.os.distribution, ev.os.version, ev.os.os_type),
        )
    });

    // MIG-G-002: firmware / Secure Boot
    let fw = &ev.boot.firmware;
    if ev.boot.secure_boot == Some(true) {
        out.push(Check::fail(
            "MIG-G-002",
            "Firmware / Secure Boot",
            Cat::Security,
            4.0,
            CheckSeverity::Warning,
            "Secure Boot is enabled — destination must provide a matching UEFI+SecureBoot configuration",
            Some(Hint::Manual {
                instructions: "provision the destination VM with UEFI firmware and Secure Boot, or disable Secure Boot in the guest".into(),
            }),
        ));
    } else if !fw.is_empty() {
        out.push(Check::pass(
            "MIG-G-002",
            "Firmware / Secure Boot",
            Cat::Security,
            4.0,
            format!("{fw} firmware, Secure Boot not enforcing"),
        ));
    }

    // MIG-G-003: free disk space on root
    match ev.storage.free_space_root_mb {
        Some(mb) if mb < 500 => out.push(Check::fail(
            "MIG-G-003",
            "Free disk space",
            Cat::Storage,
            5.0,
            CheckSeverity::Warning,
            format!("only {mb} MB free on root — driver/package installation may fail"),
            Some(Hint::Manual {
                instructions: "free at least 500 MB on the root filesystem before migration".into(),
            }),
        )),
        Some(mb) => out.push(Check::pass(
            "MIG-G-003",
            "Free disk space",
            Cat::Storage,
            5.0,
            format!("{mb} MB free on root"),
        )),
        None => {}
    }

    // MIG-G-004: pending reboot (Windows evidence carries it; Linux comes
    // from the live heartbeat, informational here)
    if let Some(win) = &ev.windows {
        out.push(if win.pending_reboot {
            Check::fail(
                "MIG-G-004",
                "Pending reboot",
                Cat::Boot,
                3.0,
                CheckSeverity::Warning,
                "a reboot is pending — finish it before cutover so first boot on the target is clean",
                Some(Hint::Manual {
                    instructions: "reboot the guest before the migration window".into(),
                }),
            )
        } else {
            Check::pass("MIG-G-004", "Pending reboot", Cat::Boot, 3.0, "no pending reboot")
        });
    }

    // MIG-G-005: competing hypervisor tooling
    let mut remnants: Vec<String> = ev
        .vm_tools
        .detected
        .iter()
        .map(|t| t.to_string())
        .collect();
    if let Some(win) = &ev.windows {
        remnants.extend(win.hypervisor_remnants.iter().cloned());
    }
    remnants.sort();
    remnants.dedup();
    let foreign: Vec<String> = remnants
        .into_iter()
        .filter(|t| {
            let t = t.to_lowercase();
            t.contains("vmware") || t.contains("hyper-v") || t.contains("hyperv") || t.contains("xen")
        })
        .collect();
    out.push(if foreign.is_empty() {
        Check::pass(
            "MIG-G-005",
            "Source hypervisor tooling",
            Cat::Application,
            5.0,
            "no competing hypervisor tools detected",
        )
    } else {
        Check::fail(
            "MIG-G-005",
            "Source hypervisor tooling",
            Cat::Application,
            5.0,
            CheckSeverity::Warning,
            format!("source-hypervisor tooling present: {}", foreign.join(", ")),
            Some(Hint::RemoveHypervisorTools { packages: foreign }),
        )
    });

    out
}

// --- MIG-L: Linux ---

fn linux_checks(ev: &EvidenceSnapshot, ctx: &AssessContext) -> Vec<Check> {
    let mut out = Vec::new();

    // MIG-L-001: virtio modules inside the initramfs (stronger than
    // BOOT-006, which only proves module availability).
    let ram_mods = &ev.boot.initramfs_modules;
    if !ram_mods.is_empty() {
        let has_blk = ram_mods.iter().any(|m| m == "virtio_blk" || m == "virtio_scsi");
        out.push(if has_blk {
            Check::pass(
                "MIG-L-001",
                "VirtIO in initramfs",
                Cat::Driver,
                10.0,
                format!("initramfs bundles: {}", ram_mods.join(", ")),
            )
        } else {
            Check::fail(
                "MIG-L-001",
                "VirtIO in initramfs",
                Cat::Driver,
                10.0,
                CheckSeverity::Blocker,
                "no virtio block/SCSI module inside the initramfs — the guest will not find its root disk on KVM",
                Some(Hint::AddVirtioToInitramfs),
            )
        });
    } else if let Some(wrapped) =
        ctx.boot_check("BOOT-006", "MIG-L-001", Cat::Driver, 10.0, Some(Hint::AddVirtioToInitramfs))
    {
        // Fall back to module availability when initramfs contents unknown.
        out.push(wrapped);
    }

    // Wrapped boot checks (probes already ran; recategorized + repair hints).
    for (boot_id, mig_id, cat, weight, hint) in [
        ("BOOT-001", "MIG-L-002", Cat::Storage, 8.0, Some(Hint::ConvertFstabToUuid)),
        ("BOOT-003", "MIG-L-003", Cat::Boot, 8.0, Some(Hint::AddVirtioToInitramfs)),
        ("BOOT-004", "MIG-L-004", Cat::Boot, 8.0, Some(Hint::RepairBootloader)),
        ("BOOT-005", "MIG-L-005", Cat::Boot, 5.0, Some(Hint::RepairBootloader)),
        ("BOOT-009", "MIG-L-006", Cat::Security, 3.0, Some(Hint::ScheduleSelinuxRelabel)),
    ] {
        if let Some(check) = ctx.boot_check(boot_id, mig_id, cat, weight, hint) {
            out.push(check);
        }
    }

    // MIG-L-007: static IP configs × NIC renaming risk
    if let Some(lm) = &ev.linux_migration {
        if !lm.static_ip_configs.is_empty() {
            let renaming_risk = lm.predictable_nic_names == Some(true);
            out.push(Check::fail(
                "MIG-L-007",
                "Static IP × NIC naming",
                Cat::Network,
                6.0,
                if renaming_risk {
                    CheckSeverity::Warning
                } else {
                    CheckSeverity::Info
                },
                format!(
                    "static IP configuration in {} file(s){}",
                    lm.static_ip_configs.len(),
                    if renaming_risk {
                        " and predictable NIC names may change on new virtual hardware (e.g. ens192 → enp1s0)"
                    } else {
                        ""
                    }
                ),
                Some(Hint::PreserveStaticIp),
            ));
        } else {
            out.push(Check::pass(
                "MIG-L-007",
                "Static IP × NIC naming",
                Cat::Network,
                6.0,
                "no static IP configuration detected",
            ));
        }

        // MIG-L-008: hypervisor-specific kernel modules
        if !lm.hypervisor_modules.is_empty() {
            out.push(Check::fail(
                "MIG-L-008",
                "Hypervisor kernel modules",
                Cat::Driver,
                4.0,
                CheckSeverity::Warning,
                format!("loaded: {}", lm.hypervisor_modules.join(", ")),
                Some(Hint::RemoveHypervisorTools {
                    packages: vec!["open-vm-tools".to_string()],
                }),
            ));
        } else {
            out.push(Check::pass(
                "MIG-L-008",
                "Hypervisor kernel modules",
                Cat::Driver,
                4.0,
                "no foreign hypervisor modules loaded",
            ));
        }
    }

    // MIG-L-009: serial console (rescue access on KVM targets)
    out.push(if ev.boot.serial_console_configured {
        Check::pass(
            "MIG-L-009",
            "Serial console",
            Cat::Boot,
            2.0,
            "console=ttyS* configured",
        )
    } else {
        Check::fail(
            "MIG-L-009",
            "Serial console",
            Cat::Boot,
            2.0,
            CheckSeverity::Info,
            "no serial console on the kernel command line — virsh console will be blind if boot fails",
            Some(Hint::EnableSerialConsole),
        )
    });

    out
}

// --- MIG-W: Windows ---

fn windows_checks(ev: &EvidenceSnapshot, ctx: &AssessContext) -> Vec<Check> {
    let mut out = Vec::new();
    let Some(win) = &ev.windows else {
        return out;
    };

    // MIG-W-001: boot-critical virtio storage driver
    let storage = win
        .virtio_drivers
        .iter()
        .find(|d| (d.name == "viostor" || d.name == "vioscsi") && d.present);
    out.push(match storage {
        Some(d) if d.boot_critical => Check::pass(
            "MIG-W-001",
            "VirtIO storage driver",
            Cat::Driver,
            12.0,
            format!("{} installed, boot-critical", d.name),
        ),
        Some(d) => Check::fail(
            "MIG-W-001",
            "VirtIO storage driver",
            Cat::Driver,
            12.0,
            CheckSeverity::Blocker,
            format!(
                "{} installed but start type is '{}', not Boot — Windows will 0x7B on virtio storage",
                d.name, d.start_type
            ),
            Some(Hint::RegisterBootCritical {
                driver: d.name.clone(),
            }),
        ),
        None if win.virtio_drivers.is_empty() => Check::fail(
            "MIG-W-001",
            "VirtIO storage driver",
            Cat::Driver,
            12.0,
            CheckSeverity::Warning,
            "virtio driver state could not be determined",
            None,
        ),
        None => Check::fail(
            "MIG-W-001",
            "VirtIO storage driver",
            Cat::Driver,
            12.0,
            CheckSeverity::Blocker,
            "no virtio storage driver (viostor/vioscsi) installed",
            Some(Hint::InjectWindowsDriver {
                driver: "vioscsi".to_string(),
            }),
        ),
    });

    // MIG-W-002: virtio network driver
    if !win.virtio_drivers.is_empty() {
        let net = win.virtio_drivers.iter().find(|d| d.name == "netkvm");
        out.push(match net {
            Some(d) if d.present => Check::pass(
                "MIG-W-002",
                "VirtIO network driver",
                Cat::Driver,
                6.0,
                "netkvm installed",
            ),
            _ => Check::fail(
                "MIG-W-002",
                "VirtIO network driver",
                Cat::Driver,
                6.0,
                CheckSeverity::Warning,
                "netkvm not installed — no network after cutover until drivers are added",
                Some(Hint::InjectWindowsDriver {
                    driver: "netkvm".to_string(),
                }),
            ),
        });
    }

    // MIG-W-003/004: BCD + boot chain (wrap boot engine results)
    for (boot_id, mig_id, weight) in [("BOOT-013", "MIG-W-003", 8.0), ("BOOT-012", "MIG-W-004", 6.0)] {
        if let Some(check) =
            ctx.boot_check(boot_id, mig_id, Cat::Boot, weight, Some(Hint::RepairBootloader))
        {
            out.push(check);
        }
    }

    // MIG-W-005: BitLocker
    match &win.bitlocker {
        Some(bl) if bl.any_protected => out.push(Check::fail(
            "MIG-W-005",
            "BitLocker",
            Cat::Security,
            10.0,
            CheckSeverity::Blocker,
            "BitLocker protection is active — hardware change will trigger recovery mode",
            Some(Hint::SuspendBitLocker),
        )),
        Some(_) => out.push(Check::pass(
            "MIG-W-005",
            "BitLocker",
            Cat::Security,
            10.0,
            "no active BitLocker protection",
        )),
        None if win.bitlocker_detected => out.push(Check::fail(
            "MIG-W-005",
            "BitLocker",
            Cat::Security,
            10.0,
            CheckSeverity::Warning,
            "BitLocker artifacts detected; verify protection state and escrow the recovery key",
            Some(Hint::SuspendBitLocker),
        )),
        None => {}
    }

    // MIG-W-006: ghost NICs
    out.push(if win.ghost_nics.is_empty() {
        Check::pass("MIG-W-006", "Ghost NICs", Cat::Network, 3.0, "no disconnected NICs")
    } else {
        Check::fail(
            "MIG-W-006",
            "Ghost NICs",
            Cat::Network,
            3.0,
            CheckSeverity::Warning,
            format!(
                "{} disconnected NIC(s) hold stale configuration",
                win.ghost_nics.len()
            ),
            Some(Hint::RemoveGhostNics {
                instance_ids: win.ghost_nics.iter().map(|g| g.instance_id.clone()).collect(),
            }),
        )
    });

    // MIG-W-007: static NIC configuration preservation
    if !win.static_nic_configs.is_empty() {
        out.push(Check::fail(
            "MIG-W-007",
            "Static NIC configuration",
            Cat::Network,
            6.0,
            CheckSeverity::Warning,
            format!(
                "{} adapter(s) use static IPs bound to source hardware — capture and re-apply on the destination adapter",
                win.static_nic_configs.len()
            ),
            Some(Hint::PreserveStaticIp),
        ));
    }

    // MIG-W-008: activation channel risk
    if let Some(act) = &win.activation {
        let oem = act.channel.to_uppercase().contains("OEM");
        out.push(if oem {
            Check::fail(
                "MIG-W-008",
                "Windows activation",
                Cat::Application,
                3.0,
                CheckSeverity::Warning,
                format!(
                    "OEM-channel license ({}) is tied to source hardware and may deactivate",
                    act.channel
                ),
                Some(Hint::Manual {
                    instructions: "prepare a KMS/MAK key or plan re-activation after migration".into(),
                }),
            )
        } else {
            Check::pass(
                "MIG-W-008",
                "Windows activation",
                Cat::Application,
                3.0,
                format!(
                    "licensed={} channel={}",
                    act.licensed,
                    if act.channel.is_empty() { "unknown" } else { &act.channel }
                ),
            )
        });
    }

    // MIG-W-009: VSS health (needed for consistent cutover snapshots)
    if let Some(vss) = &win.vss {
        out.push(if vss.healthy {
            Check::pass(
                "MIG-W-009",
                "VSS writers",
                Cat::Application,
                4.0,
                format!("{} writers stable", vss.writers_total),
            )
        } else {
            Check::fail(
                "MIG-W-009",
                "VSS writers",
                Cat::Application,
                4.0,
                CheckSeverity::Warning,
                format!("failed writers: {}", vss.writers_failed.join(", ")),
                Some(Hint::Manual {
                    instructions: "restart the failed VSS writer services before taking a consistent snapshot".into(),
                }),
            )
        });
    }

    // MIG-W-010: driver signature enforcement vs virtio signing
    if let Some(enforced) = win.driver_signature_enforcement {
        if !enforced {
            out.push(Check::fail(
                "MIG-W-010",
                "Driver signature enforcement",
                Cat::Security,
                2.0,
                CheckSeverity::Info,
                "test-signing or nointegritychecks is enabled — remember to re-enable enforcement after driver work",
                None,
            ));
        } else {
            out.push(Check::pass(
                "MIG-W-010",
                "Driver signature enforcement",
                Cat::Security,
                2.0,
                "signature enforcement active (signed virtio-win drivers required)",
            ));
        }
    }

    out
}

/// Shared fixtures for migration tests (checks + repair planner).
#[cfg(test)]
pub(crate) mod tests_support {
    use crate::evidence::snapshot::*;

    pub fn linux_evidence() -> EvidenceSnapshot {
        let mut ev = EvidenceSnapshot {
            schema_version: SCHEMA_VERSION,
            image_path: "test".into(),
            collected_at: String::new(),
            root: "/".into(),
            os: OsEvidence {
                os_type: "linux".into(),
                distribution: "rhel".into(),
                version: "9.6".into(),
                ..Default::default()
            },
            storage: StorageEvidence::default(),
            boot: BootEvidence::default(),
            network: NetworkEvidence::default(),
            packages: PackageEvidence::default(),
            security: SecurityEvidence::default(),
            vm_tools: VmToolsEvidence::default(),
            systemd: None,
            windows: None,
            kubevirt: None,
            cloud_init: None,
            network_probes: None,
            snapshot_readiness: None,
            process: None,
            hardware: None,
            linux_migration: Some(LinuxMigrationEvidence::default()),
        };
        ev.boot.initramfs_modules = vec!["virtio_blk".into(), "virtio_pci".into()];
        ev.boot.serial_console_configured = true;
        ev
    }

    pub fn windows_evidence(virtio_boot_critical: bool) -> EvidenceSnapshot {
        let mut ev = linux_evidence();
        ev.os.os_type = "windows".into();
        ev.linux_migration = None;
        ev.windows = Some(WindowsEvidence {
            virtio_drivers: vec![WindowsDriverEntry {
                name: "viostor".into(),
                start_type: if virtio_boot_critical { "boot" } else { "manual" }.into(),
                boot_critical: virtio_boot_critical,
                present: true,
                version: None,
            }],
            ..Default::default()
        });
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::tests_support::{linux_evidence, windows_evidence};
    use super::*;
    use crate::boot::report::BootabilityReport;
    use crate::boot::BootTarget;
    use crate::evidence::snapshot::*;

    fn ctx(report: &BootabilityReport) -> AssessContext<'_> {
        AssessContext {
            target: BootTarget::Kvm,
            target_name: "kvm".to_string(),
            live: false,
            boot_report: report,
        }
    }

    fn empty_report() -> BootabilityReport {
        BootabilityReport {
            score: 100.0,
            confidence: 1.0,
            target: "kvm".to_string(),
            blockers: vec![],
            warnings: vec![],
            checks: vec![],
            summary: String::new(),
        }
    }

    #[test]
    fn healthy_linux_passes_driver_checks() {
        let report = empty_report();
        let ev = linux_evidence();
        let checks = run_all_checks(&ev, &ctx(&report));
        let l1 = checks.iter().find(|c| c.id == "MIG-L-001").unwrap();
        assert!(l1.passed, "{}", l1.message);
        assert!(checks.iter().any(|c| c.id == "MIG-L-009" && c.passed));
    }

    #[test]
    fn missing_initramfs_virtio_is_blocker_with_hint() {
        let report = empty_report();
        let mut ev = linux_evidence();
        ev.boot.initramfs_modules = vec!["nvme".into()];
        let checks = run_all_checks(&ev, &ctx(&report));
        let l1 = checks.iter().find(|c| c.id == "MIG-L-001").unwrap();
        assert!(!l1.passed);
        assert_eq!(l1.severity, CheckSeverity::Blocker);
        assert_eq!(l1.remediation, Some(Hint::AddVirtioToInitramfs));
    }

    #[test]
    fn windows_manual_start_storage_driver_is_blocker() {
        let report = empty_report();
        let ev = windows_evidence(false);
        let checks = run_all_checks(&ev, &ctx(&report));
        let w1 = checks.iter().find(|c| c.id == "MIG-W-001").unwrap();
        assert!(!w1.passed);
        assert_eq!(
            w1.remediation,
            Some(Hint::RegisterBootCritical {
                driver: "viostor".into()
            })
        );
    }

    #[test]
    fn windows_boot_critical_storage_passes() {
        let report = empty_report();
        let ev = windows_evidence(true);
        let checks = run_all_checks(&ev, &ctx(&report));
        assert!(checks.iter().find(|c| c.id == "MIG-W-001").unwrap().passed);
    }

    #[test]
    fn bitlocker_active_is_blocker() {
        let report = empty_report();
        let mut ev = windows_evidence(true);
        ev.windows.as_mut().unwrap().bitlocker = Some(BitLockerState {
            any_protected: true,
            volumes: vec![],
        });
        let checks = run_all_checks(&ev, &ctx(&report));
        let w5 = checks.iter().find(|c| c.id == "MIG-W-005").unwrap();
        assert_eq!(w5.severity, CheckSeverity::Blocker);
        assert_eq!(w5.remediation, Some(Hint::SuspendBitLocker));
    }

    #[test]
    fn vmware_tooling_flagged_for_removal() {
        let report = empty_report();
        let mut ev = linux_evidence();
        ev.vm_tools.detected = vec!["vmware-tools".into()];
        let checks = run_all_checks(&ev, &ctx(&report));
        let g5 = checks.iter().find(|c| c.id == "MIG-G-005").unwrap();
        assert!(!g5.passed);
    }
}
