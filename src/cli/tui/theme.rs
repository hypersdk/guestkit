// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carbon control-plane palette with restrained orange accents (Zellij / k9s inspired).

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders},
};

// ── Surfaces ─────────────────────────────────────────────────────────────────
pub const BG: Color = Color::Rgb(11, 14, 18); // #0B0E12
pub const SURFACE: Color = Color::Rgb(17, 21, 27); // #11151B
pub const SURFACE_RAISED: Color = Color::Rgb(22, 27, 34); // #161B22

// ── Accent (focus, selection, primary actions only) ──────────────────────────
pub const ACCENT: Color = Color::Rgb(255, 122, 0); // #FF7A00
pub const ACCENT_SOFT: Color = Color::Rgb(255, 158, 64);

// ── Borders & text ───────────────────────────────────────────────────────────
pub const BORDER_MUTED: Color = Color::Rgb(42, 47, 56); // #2A2F38
pub const TEXT: Color = Color::Rgb(220, 227, 234); // #DCE3EA
pub const TEXT_MUTED: Color = Color::Rgb(125, 133, 144); // #7D8590

// ── Semantic (subtle, not rainbow) ─────────────────────────────────────────────
pub const SUCCESS: Color = Color::Rgb(63, 185, 80);
pub const WARNING: Color = Color::Rgb(210, 153, 34);
pub const ERROR: Color = Color::Rgb(248, 81, 73);
pub const INFO: Color = TEXT_MUTED;

// Legacy names used across views
pub const BG_COLOR: Color = BG;
pub const ORANGE: Color = ACCENT;
pub const LIGHT_ORANGE: Color = ACCENT_SOFT;
pub const DARK_ORANGE: Color = BORDER_MUTED;
pub const BORDER_COLOR: Color = BORDER_MUTED;
pub const TEXT_COLOR: Color = TEXT;
pub const SUCCESS_COLOR: Color = SUCCESS;
pub const WARNING_COLOR: Color = WARNING;
pub const ERROR_COLOR: Color = ERROR;
pub const INFO_COLOR: Color = INFO;

/// Full-screen carbon background.
pub fn fill_background() -> Block<'static> {
    Block::default().style(Style::default().bg(BG))
}

/// Zellij-style pane: muted border; orange only when focused.
pub fn pane_block(title: &str, focused: bool) -> Block<'_> {
    let border_color = if focused { ACCENT } else { BORDER_MUTED };
    let title_color = if focused { ACCENT } else { TEXT_MUTED };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(format!(" {title} "))
        .title_style(Style::default().fg(title_color))
        .style(Style::default().bg(SURFACE))
}

/// Standard content pane (muted chrome).
pub fn content_block(title: &str) -> Block<'_> {
    pane_block(title, false)
}

/// Border color from issue severity — context-aware glow, not full-panel orange.
pub fn risk_border_color(critical: usize, high: usize, medium: usize) -> Color {
    if critical > 0 {
        ERROR
    } else if high > 0 {
        ACCENT
    } else if medium > 0 {
        WARNING
    } else {
        BORDER_MUTED
    }
}

/// Keyboard hint: accent only for primary actions.
pub fn key_primary() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Keyboard hint: muted for secondary bindings.
pub fn key_muted() -> Style {
    Style::default().fg(TEXT_MUTED)
}

/// Pane with explicit border color (e.g. risk-aware header).
pub fn pane_block_with_border(title: &str, border: Color) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(format!(" {title} "))
        .title_style(Style::default().fg(TEXT_MUTED))
        .style(Style::default().bg(SURFACE))
}

/// Title style for in-pane headings (muted, not orange).
pub fn label_style() -> Style {
    Style::default().fg(TEXT_MUTED)
}

/// Secondary highlight (paths, values).
pub fn value_style() -> Style {
    Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
}

/// Active list/tab row.
pub fn focus_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(SURFACE_RAISED)
        .add_modifier(Modifier::BOLD)
}

/// Context-aware glow tints for telemetry (faint border emphasis).
pub fn glow_error() -> Style {
    Style::default().fg(Color::Rgb(180, 60, 55))
}

pub fn glow_warning() -> Style {
    Style::default().fg(Color::Rgb(160, 110, 40))
}
