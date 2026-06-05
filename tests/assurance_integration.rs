// SPDX-License-Identifier: LGPL-3.0-or-later
//! Integration tests for migration assurance commands.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn guestkit() -> Command {
    Command::new(env!("CARGO_BIN_EXE_guestkit"))
}

fn test_image_path() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("GUESTKIT_TEST_IMAGE") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(path) = std::env::var("GUESTKIT_TEST_UBUNTU_22_04") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Some(p);
        }
    }
    let default = PathBuf::from("test-images/ubuntu-22.04.qcow2");
    if default.exists() {
        return Some(default);
    }
    None
}

fn require_test_image() -> PathBuf {
    test_image_path().unwrap_or_else(|| {
        panic!(
            "Set GUESTKIT_TEST_IMAGE or GUESTKIT_TEST_UBUNTU_22_04, or place test-images/ubuntu-22.04.qcow2"
        )
    })
}

#[test]
fn migrate_plan_help_lists_export_flag() {
    guestkit()
        .arg("migrate-plan")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--export"));
}

#[test]
fn doctor_help_lists_target_and_explain() {
    guestkit()
        .arg("doctor")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--target"))
        .stdout(predicate::str::contains("--explain"));
}

#[test]
fn doctor_fails_on_missing_image() {
    guestkit()
        .arg("doctor")
        .arg("/nonexistent/guestkit-test.qcow2")
        .arg("--target")
        .arg("kvm")
        .assert()
        .failure();
}

#[test]
fn migrate_plan_fails_on_missing_image() {
    guestkit()
        .arg("migrate-plan")
        .arg("/nonexistent/guestkit-test.qcow2")
        .arg("--target")
        .arg("proxmox")
        .assert()
        .failure();
}

#[test]
fn repair_boot_dry_run_on_missing_image_fails() {
    guestkit()
        .arg("repair")
        .arg("/nonexistent/guestkit-test.qcow2")
        .arg("--fix")
        .arg("boot")
        .arg("--dry-run")
        .assert()
        .failure();
}

#[test]
#[ignore = "requires guestfs and a test disk image"]
fn doctor_json_on_test_image() {
    let image = require_test_image();
    guestkit()
        .arg("doctor")
        .arg(&image)
        .arg("--target")
        .arg("kvm")
        .arg("-o")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("bootability"));
}

#[test]
#[ignore = "requires guestfs and a test disk image"]
fn migrate_plan_json_on_test_image() {
    let image = require_test_image();
    guestkit()
        .arg("migrate-plan")
        .arg(&image)
        .arg("--target")
        .arg("proxmox")
        .arg("-o")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("migration_score"))
        .stdout(predicate::str::contains("required_changes"));
}

#[test]
#[ignore = "requires guestfs and a test disk image"]
fn migrate_plan_export_writes_fix_plan() {
    let image = require_test_image();
    let dir = TempDir::new().expect("temp dir");
    let plan_path = dir.path().join("migration-plan.yaml");

    guestkit()
        .arg("migrate-plan")
        .arg(&image)
        .arg("--target")
        .arg("proxmox")
        .arg("--export")
        .arg(&plan_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix plan written"));

    let content = std::fs::read_to_string(&plan_path).expect("plan file");
    assert!(content.contains("profile: migration"));
    assert!(content.contains("operations:"));
}

#[test]
#[ignore = "requires guestfs and a test disk image"]
fn repair_boot_dry_run_on_test_image() {
    let image = require_test_image();
    guestkit()
        .arg("repair")
        .arg(&image)
        .arg("--fix")
        .arg("boot")
        .arg("--dry-run")
        .assert()
        .success();
}

#[test]
fn from_migration_report_unit_pipeline() {
    use guestkit::boot::BootabilityReport;
    use guestkit::cli::migrate::plan::compute_migration_score;
    use guestkit::cli::plan::PlanGenerator;
    use guestkit::evidence::snapshot::{
        BootEvidence, EvidenceSnapshot, OsEvidence, PackageEvidence, SecurityEvidence,
        StorageEvidence, VmToolsEvidence, SCHEMA_VERSION,
    };

    let evidence = EvidenceSnapshot {
        schema_version: SCHEMA_VERSION,
        image_path: "vm.qcow2".to_string(),
        collected_at: "2026-01-01".to_string(),
        root: "/".to_string(),
        os: OsEvidence::default(),
        storage: StorageEvidence::default(),
        boot: BootEvidence {
            loaded_modules: vec![],
            cloud_init_present: false,
            ..Default::default()
        },
        network: Default::default(),
        packages: PackageEvidence::default(),
        security: SecurityEvidence::default(),
        vm_tools: VmToolsEvidence {
            detected: vec!["vmware-tools".to_string()],
        },
        systemd: None,
        windows: None,
        kubevirt: None,
        cloud_init: None,
        network_probes: None,
        snapshot_readiness: None,
    };
    let boot = BootabilityReport {
        score: 70.0,
        confidence: 0.85,
        target: "proxmox".to_string(),
        blockers: vec![],
        warnings: vec![],
        checks: vec![],
        summary: "warnings".to_string(),
    };
    let migration = compute_migration_score(&evidence, &boot, "proxmox");
    assert!(!migration.required_changes.is_empty());

    let generator = PlanGenerator::new("vm.qcow2".to_string());
    let plan = generator
        .from_migration_report(&migration, &boot, "proxmox", Path::new("vm.qcow2"))
        .expect("plan generation");
    assert_eq!(plan.profile, "migration");
    assert!(!plan.operations.is_empty());
}
