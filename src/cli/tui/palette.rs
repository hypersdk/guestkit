// SPDX-License-Identifier: LGPL-3.0-or-later
//! Command palette (`:`) for quick TUI actions.

use super::app::View;

#[derive(Debug, Clone)]
pub struct PaletteCommand {
    pub name: &'static str,
    pub description: &'static str,
}

pub const COMMANDS: &[PaletteCommand] = &[
    PaletteCommand { name: "goto dashboard", description: "Open dashboard" },
    PaletteCommand { name: "goto issues", description: "Security issues" },
    PaletteCommand { name: "goto files", description: "File browser" },
    PaletteCommand { name: "goto packages", description: "Installed packages" },
    PaletteCommand { name: "goto profiles", description: "Profile reports" },
    PaletteCommand { name: "export json", description: "Export current view as JSON" },
    PaletteCommand { name: "export html", description: "Export security report HTML" },
    PaletteCommand { name: "refresh", description: "Reload current view data" },
    PaletteCommand { name: "refresh full", description: "Full re-inspect of image" },
    PaletteCommand { name: "compare toggle", description: "Toggle comparison mode" },
    PaletteCommand { name: "pin view", description: "Pin current tab" },
    PaletteCommand { name: "help", description: "Toggle help overlay" },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    Goto(View),
    ExportJson,
    ExportHtml,
    Refresh,
    RefreshFull,
    CompareToggle,
    PinView,
    Help,
    Unknown,
}

pub fn parse_command(input: &str) -> PaletteAction {
    let cmd = input.trim().to_lowercase();
    match cmd.as_str() {
        "goto dashboard" | "dashboard" => PaletteAction::Goto(View::Dashboard),
        "goto issues" | "issues" => PaletteAction::Goto(View::Issues),
        "goto files" | "files" => PaletteAction::Goto(View::Files),
        "goto packages" | "packages" => PaletteAction::Goto(View::Packages),
        "goto profiles" | "profiles" => PaletteAction::Goto(View::Profiles),
        "goto network" | "network" => PaletteAction::Goto(View::Network),
        "goto services" | "services" => PaletteAction::Goto(View::Services),
        "goto security" | "security" => PaletteAction::Goto(View::Security),
        "export json" => PaletteAction::ExportJson,
        "export html" => PaletteAction::ExportHtml,
        "refresh" => PaletteAction::Refresh,
        "refresh full" => PaletteAction::RefreshFull,
        "compare toggle" | "compare" => PaletteAction::CompareToggle,
        "pin view" | "pin" => PaletteAction::PinView,
        "help" => PaletteAction::Help,
        _ if cmd.starts_with("goto ") => {
            let target = cmd.trim_start_matches("goto ");
            View::from_name(target).map(PaletteAction::Goto).unwrap_or(PaletteAction::Unknown)
        }
        _ => PaletteAction::Unknown,
    }
}

pub fn filtered_commands(query: &str) -> Vec<(&'static str, &'static str)> {
    let q = query.trim().to_lowercase();
    let iter = COMMANDS.iter().filter(|c| {
        q.is_empty()
            || c.name.contains(&q)
            || c.description.to_lowercase().contains(&q)
    });
    iter.map(|c| (c.name, c.description)).collect()
}
