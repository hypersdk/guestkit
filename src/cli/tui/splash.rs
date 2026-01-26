// SPDX-License-Identifier: LGPL-3.0-or-later
//! Splash screen with ASCII art

use crate::cli::tui::ui::{BORDER_COLOR, LIGHT_ORANGE, ORANGE, TEXT_COLOR};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_splash(f: &mut Frame) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area);

    let logo_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(chunks[1]);

    let logo = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("   ██████╗ ██╗   ██╗███████╗███████╗████████╗", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ██╔════╝ ██║   ██║██╔════╝██╔════╝╚══██╔══╝", Style::default().fg(ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("  ██║  ███╗██║   ██║█████╗  ███████╗   ██║   ", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ██║   ██║██║   ██║██╔══╝  ╚════██║   ██║   ", Style::default().fg(LIGHT_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("  ╚██████╔╝╚██████╔╝███████╗███████║   ██║   ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("   ╚═════╝  ╚═════╝ ╚══════╝╚══════╝   ╚═╝   ", Style::default().fg(ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("       ██╗  ██╗██╗████████╗", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("       ██║ ██╔╝██║╚══██╔══╝", Style::default().fg(LIGHT_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("       █████╔╝ ██║   ██║   ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("       ██╔═██╗ ██║   ██║   ", Style::default().fg(ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("       ██║  ██╗██║   ██║   ", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("       ╚═╝  ╚═╝╚═╝   ╚═╝   ", Style::default().fg(LIGHT_ORANGE)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("       VM Inspection & Analysis Tool", Style::default().fg(TEXT_COLOR).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("            Press any key to continue...", Style::default().fg(BORDER_COLOR)),
        ]),
    ];

    let splash = Paragraph::new(logo)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR)))
        .alignment(Alignment::Center);

    f.render_widget(splash, logo_chunks[1]);
}
