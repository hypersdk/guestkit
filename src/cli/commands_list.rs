// SPDX-License-Identifier: LGPL-3.0-or-later
//! Grouped listing of CLI subcommands (`commands` subcommand).

use colored::Colorize;

/// (category title, command names)
const GROUPS: &[(&str, &[&str])] = &[
    (
        "Inspect & report",
        &[
            "inspect",
            "inspect-batch",
            "doctor",
            "migrate-plan",
            "diff",
            "forensic-diff",
            "compare",
            "inventory",
            "sbom",
            "cve",
            "licenses",
        ],
    ),
    (
        "Files & disk",
        &[
            "list",
            "extract",
            "inject",
            "search",
            "grep",
            "cat",
            "checksum",
            "du",
            "find-large",
            "tree",
            "archive",
            "convert",
            "info",
            "fsck",
            "df",
            "filesystems",
            "packages",
            "snapshots",
        ],
    ),
    (
        "Security & compliance",
        &[
            "scan",
            "secrets",
            "rescue",
            "cleanup",
            "network-audit",
            "compliance",
            "malware",
            "health",
            "audit",
            "repair",
            "policy",
            "harden",
            "anomaly",
            "recommend",
            "predict",
            "threat-intel",
            "hunt",
            "reconstruct",
            "evolve",
            "verify",
        ],
    ),
    (
        "Migrate & plan",
        &[
            "migrate",
            "migrate-plan",
            "blueprint",
            "plan",
            "cost",
            "dependencies",
            "risk",
            "fleet",
            "agent",
            "agent-proxy",
        ],
    ),
    (
        "Systemd",
        &["systemd-journal", "systemd-services", "systemd-boot"],
    ),
    (
        "Interactive",
        &[
            "tui",
            "shell",
            "interactive",
            "explore",
            "script",
            "ai",
        ],
    ),
    (
        "Utilities",
        &[
            "cache-clear",
            "cache-stats",
            "completion",
            "commands",
            "version",
            "lvm-clone",
        ],
    ),
];

/// Print grouped subcommands; `bin` is the invoked program name.
pub fn print_grouped_commands(bin: &str) {
    println!();
    println!("{} subcommands (by category):", bin.bold());
    println!();

    for (title, cmds) in GROUPS {
        println!("  {}", title.truecolor(222, 115, 86).bold());
        for cmd in *cmds {
            println!("    {cmd}");
        }
        println!();
    }

    println!(
        "  Run {} for per-command help.",
        format!("{bin} <command> --help").dimmed()
    );
    println!();
}
