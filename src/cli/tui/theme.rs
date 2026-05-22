// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carbon control-plane palette with restrained orange accents (Zellij / k9s inspired).

use super::app::App;
use super::config::UiConfig;
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Sparkline},
};

// ── Carbon (default) ───────────────────────────────────────────────────────────
pub const BG: Color = Color::Rgb(11, 14, 18);
pub const SURFACE: Color = Color::Rgb(17, 21, 27);
pub const SURFACE_RAISED: Color = Color::Rgb(22, 27, 34);
pub const ACCENT: Color = Color::Rgb(255, 122, 0);
pub const ACCENT_SOFT: Color = Color::Rgb(255, 158, 64);
pub const BORDER_MUTED: Color = Color::Rgb(42, 47, 56);
pub const TEXT: Color = Color::Rgb(220, 227, 234);
pub const TEXT_MUTED: Color = Color::Rgb(125, 133, 144);
pub const SUCCESS: Color = Color::Rgb(63, 185, 80);
pub const WARNING: Color = Color::Rgb(210, 153, 34);
pub const ERROR: Color = Color::Rgb(248, 81, 73);
pub const INFO: Color = TEXT_MUTED;
pub const SPARKLINE_MUTED: Color = Color::Rgb(60, 68, 78);
pub const TRACK: Color = BORDER_MUTED;

// Legacy aliases
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

/// Resolved palette for active theme.
#[derive(Clone, Copy)]
pub struct UiPalette {
    pub bg: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub accent: Color,
    pub accent_soft: Color,
    pub border: Color,
    pub text: Color,
    pub text_muted: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl UiPalette {
    pub const CARBON: Self = Self {
        bg: BG,
        surface: SURFACE,
        surface_raised: SURFACE_RAISED,
        accent: ACCENT,
        accent_soft: ACCENT_SOFT,
        border: BORDER_MUTED,
        text: TEXT,
        text_muted: TEXT_MUTED,
        success: SUCCESS,
        warning: WARNING,
        error: ERROR,
    };

    pub const HIGH_CONTRAST: Self = Self {
        bg: Color::Rgb(0, 0, 0),
        surface: Color::Rgb(18, 18, 18),
        surface_raised: Color::Rgb(32, 32, 32),
        accent: Color::Rgb(255, 140, 0),
        accent_soft: Color::Rgb(255, 180, 80),
        border: Color::Rgb(100, 100, 100),
        text: Color::Rgb(240, 240, 240),
        text_muted: Color::Rgb(180, 180, 180),
        success: Color::Rgb(80, 220, 100),
        warning: Color::Rgb(255, 200, 50),
        error: Color::Rgb(255, 90, 90),
    };

    pub const MINIMAL: Self = Self {
        bg: Color::Rgb(8, 10, 12),
        surface: BG,
        surface_raised: SURFACE,
        accent: ACCENT,
        accent_soft: ACCENT_SOFT,
        border: Color::Rgb(35, 40, 48),
        text: TEXT,
        text_muted: TEXT_MUTED,
        success: SUCCESS,
        warning: WARNING,
        error: ERROR,
    };
}

pub fn for_config(cfg: &UiConfig) -> UiPalette {
    match cfg.theme.as_str() {
        "high-contrast" | "high_contrast" => UiPalette::HIGH_CONTRAST,
        "minimal" => UiPalette::MINIMAL,
        _ => UiPalette::CARBON,
    }
}

pub fn for_app(app: &App) -> UiPalette {
    for_config(&app.config.ui)
}

pub fn use_emoji(cfg: &UiConfig) -> bool {
    cfg.show_emoji && cfg.icon_mode != "ascii"
}

pub fn list_row_height(cfg: &UiConfig) -> u16 {
    if cfg.density == "compact" {
        1
    } else {
        1
    }
}

pub fn fill_background() -> Block<'static> {
    Block::default().style(Style::default().bg(BG))
}

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

pub fn content_block(title: &str) -> Block<'_> {
    pane_block(title, false)
}

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

pub fn key_primary() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

pub fn key_muted() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub fn pane_block_with_border(title: &str, border: Color) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(format!(" {title} "))
        .title_style(Style::default().fg(TEXT_MUTED))
        .style(Style::default().bg(SURFACE))
}

pub fn label_style() -> Style {
    Style::default().fg(TEXT_MUTED)
}

pub fn value_style() -> Style {
    Style::default().fg(TEXT).add_modifier(Modifier::BOLD)
}

pub fn focus_style() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(SURFACE_RAISED)
        .add_modifier(Modifier::BOLD)
}

pub fn gauge_block<'a>(title: &'a str) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER_MUTED))
        .title(format!(" {title} "))
        .title_style(Style::default().fg(TEXT_MUTED))
        .style(Style::default().bg(SURFACE))
}

pub fn gauge_widget<'a>(title: &'a str, percent: u16, label: &str, fill: Color) -> Gauge<'a> {
    Gauge::default()
        .block(gauge_block(title))
        .gauge_style(Style::default().fg(fill))
        .percent(percent.min(100))
        .label(label.to_string())
}

pub fn sparkline_block<'a>(title: &'a str) -> Block<'a> {
    gauge_block(title)
}

pub fn sparkline_widget<'a>(title: &'a str, data: &'a [u64], line_color: Color) -> Sparkline<'a> {
    Sparkline::default()
        .block(sparkline_block(title))
        .data(data)
        .style(Style::default().fg(line_color))
}

pub fn glow_error() -> Style {
    Style::default().fg(Color::Rgb(180, 60, 55))
}

pub fn glow_warning() -> Style {
    Style::default().fg(Color::Rgb(160, 110, 40))
}
