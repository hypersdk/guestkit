// SPDX-License-Identifier: LGPL-3.0-or-later
//! UI rendering orchestration

use super::app::{App, View};
use super::views;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

// Orange color theme
pub const ORANGE: Color = Color::Rgb(255, 165, 0);
pub const DARK_ORANGE: Color = Color::Rgb(204, 85, 0);
pub const LIGHT_ORANGE: Color = Color::Rgb(255, 200, 100);
pub const BG_COLOR: Color = Color::Reset;
pub const TEXT_COLOR: Color = Color::White;
pub const HIGHLIGHT_COLOR: Color = ORANGE;
pub const BORDER_COLOR: Color = DARK_ORANGE;
pub const SUCCESS_COLOR: Color = Color::Green;
pub const WARNING_COLOR: Color = Color::Yellow;
pub const ERROR_COLOR: Color = Color::Red;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_tabs(f, chunks[1], app);
    draw_content(f, chunks[2], app);
    draw_footer(f, chunks[3], app);

    if app.show_help {
        draw_help_overlay(f, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("GuestKit", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled("VM Inspector", Style::default().fg(LIGHT_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("Image: ", Style::default().fg(TEXT_COLOR)),
            Span::styled(&app.image_path, Style::default().fg(LIGHT_ORANGE)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .style(Style::default().bg(BG_COLOR)));

    f.render_widget(header, area);
}

fn draw_tabs(f: &mut Frame, area: Rect, app: &App) {
    let views = View::all();
    let titles: Vec<String> = views.iter().map(|v| v.title().to_string()).collect();

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" Navigation ")
            .title_style(Style::default().fg(ORANGE)))
        .select(views.iter().position(|v| v == &app.current_view).unwrap_or(0))
        .style(Style::default().fg(TEXT_COLOR))
        .highlight_style(Style::default()
            .fg(ORANGE)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED));

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_view {
        View::Dashboard => views::dashboard::draw(f, area, app),
        View::Network => views::network::draw(f, area, app),
        View::Packages => views::packages::draw(f, area, app),
        View::Services => views::services::draw(f, area, app),
        View::Security => views::security::draw(f, area, app),
        View::Storage => views::storage::draw(f, area, app),
    }
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = if app.is_searching() {
        vec![
            Span::styled("Search: ", Style::default().fg(ORANGE)),
            Span::styled(&app.search_query, Style::default().fg(TEXT_COLOR)),
            Span::styled(" | ", Style::default().fg(DARK_ORANGE)),
            Span::raw("ESC: Cancel"),
        ]
    } else {
        vec![
            Span::styled("Tab", Style::default().fg(ORANGE)),
            Span::raw(": Switch View | "),
            Span::styled("/", Style::default().fg(ORANGE)),
            Span::raw(": Search | "),
            Span::styled("h", Style::default().fg(ORANGE)),
            Span::raw(": Help | "),
            Span::styled("q", Style::default().fg(ORANGE)),
            Span::raw(": Quit"),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .style(Style::default().bg(BG_COLOR).fg(TEXT_COLOR));

    f.render_widget(footer, area);
}

fn draw_help_overlay(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, f.area());

    let help_text = vec![
        Line::from(vec![
            Span::styled("GuestKit TUI - Keyboard Shortcuts",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  Tab/Shift+Tab  ", Style::default().fg(ORANGE)),
            Span::raw("Switch between views")
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓            ", Style::default().fg(ORANGE)),
            Span::raw("Scroll up/down")
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn      ", Style::default().fg(ORANGE)),
            Span::raw("Page up/down")
        ]),
        Line::from(vec![
            Span::styled("  Home/End       ", Style::default().fg(ORANGE)),
            Span::raw("Jump to top/bottom")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  /              ", Style::default().fg(ORANGE)),
            Span::raw("Start search/filter")
        ]),
        Line::from(vec![
            Span::styled("  Enter          ", Style::default().fg(ORANGE)),
            Span::raw("Select/expand item")
        ]),
        Line::from(vec![
            Span::styled("  h or F1        ", Style::default().fg(ORANGE)),
            Span::raw("Toggle this help")
        ]),
        Line::from(vec![
            Span::styled("  q or ESC       ", Style::default().fg(ORANGE)),
            Span::raw("Quit / Go back")
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C         ", Style::default().fg(ORANGE)),
            Span::raw("Force quit")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or h to close this help",
                Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ORANGE))
            .title(" Help ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Color::Black).fg(TEXT_COLOR))
        .alignment(Alignment::Left);

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(help, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
