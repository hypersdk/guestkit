// SPDX-License-Identifier: LGPL-3.0-or-later
//! CLI module for guestkit

pub mod ai;
pub mod batch;
pub mod commands_list;
pub mod entry;
pub mod invocation;
pub mod welcome;
pub mod blueprint;
pub mod cache;
pub mod commands;
pub mod cost;
pub mod dependencies;
pub mod diff;
pub mod errors;
pub mod exporters;
pub mod forensic_diff;
pub mod formatters;
pub mod interactive;
pub mod inventory;
pub mod license;
pub mod migrate;
pub mod output;
pub mod parallel;
pub mod plan;
pub mod profiles;
pub mod shell;
pub mod tui;
pub mod validate;

pub use batch::*;
pub use interactive::*;

/// Run the CLI (`guestkit` / `guestctl`).
pub fn run() -> anyhow::Result<()> {
    entry::run()
}
