// SPDX-License-Identifier: LGPL-3.0-or-later
//! CLI tests for guestctl alias and UX entry points.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn guestctl_version() {
    Command::cargo_bin("guestctl")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("guestctl"));
}

#[test]
fn guestctl_welcome_without_subcommand() {
    Command::cargo_bin("guestctl")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("Common workflows"))
        .stdout(predicate::str::contains("guestctl inspect"));
}

#[test]
fn guestkit_welcome_uses_guestkit_name() {
    Command::cargo_bin("guestkit")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("guestkit — Guest VM"))
        .stdout(predicate::str::contains("guestkit inspect"));
}

#[test]
fn guestctl_commands_subcommand() {
    Command::cargo_bin("guestctl")
        .unwrap()
        .arg("commands")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inspect & report"))
        .stdout(predicate::str::contains("inspect"));
}

#[test]
fn guestctl_disk_shorthand_preprocess() {
    use guestkit::cli::invocation;

    let args = vec![
        "guestctl".to_string(),
        "/tmp/test-shorthand.qcow2".to_string(),
    ];
    let out = invocation::preprocess_args(args);
    assert_eq!(
        out,
        vec![
            "guestctl".to_string(),
            "inspect".to_string(),
            "/tmp/test-shorthand.qcow2".to_string(),
        ]
    );
}

#[test]
fn guestctl_completion_supports_all_flag() {
    Command::cargo_bin("guestctl")
        .unwrap()
        .arg("completion")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"));
}

#[test]
#[cfg(not(debug_assertions))]
fn guestctl_completion_all_generates_both_names() {
    Command::cargo_bin("guestctl")
        .unwrap()
        .arg("completion")
        .arg("bash")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("guestkit"))
        .stdout(predicate::str::contains("guestctl"));
}
