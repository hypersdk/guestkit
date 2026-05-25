// SPDX-License-Identifier: LGPL-3.0-or-later
//! No-subcommand welcome banner.

use colored::Colorize;
use crate::cli::invocation;

const ORANGE: u8 = 222;
const ORANGE_G: u8 = 115;
const ORANGE_B: u8 = 86;

/// Print quick-start help when invoked with no subcommand.
pub fn print_welcome() {
    let bin = invocation::name();
    let orange = |s: &str| s.truecolor(ORANGE, ORANGE_G, ORANGE_B);

    println!();
    println!(
        "{}",
        "╭──────────────────────────────────────────────────────────╮"
            .truecolor(ORANGE, ORANGE_G, ORANGE_B)
    );
    println!(
        "│  {} — Guest VM disk inspection & manipulation          │",
        orange(bin).bold()
    );
    println!(
        "{}",
        "╰──────────────────────────────────────────────────────────╯"
            .truecolor(ORANGE, ORANGE_G, ORANGE_B)
    );
    println!();
    println!("{}", "Common workflows:".bold());
    println!(
        "  {}  Inspect a disk image",
        orange(&invocation::example("inspect disk.qcow2")).dimmed()
    );
    println!(
        "  {}  Interactive dashboard",
        orange(&invocation::example("tui disk.qcow2")).dimmed()
    );
    println!(
        "  {}  REPL shell",
        orange(&invocation::example("shell disk.qcow2")).dimmed()
    );
    println!(
        "  {}  Compare two images",
        orange(&invocation::example("diff before.qcow2 after.qcow2")).dimmed()
    );
    println!(
        "  {}  Remediation plans",
        orange(&invocation::example("plan preview plan.json")).dimmed()
    );
    println!();
    println!(
        "  {}  Shorthand: pass a disk image path to run inspect",
        "Tip:".yellow()
    );
    println!(
        "  {}",
        orange(&invocation::example("disk.qcow2")).dimmed()
    );
    println!();
    println!(
        "  {}  Full command list",
        orange(&invocation::example("commands")).dimmed()
    );
    println!(
        "  {}  All flags and subcommands",
        orange(&format!("{bin} --help")).dimmed()
    );
    println!();
}
