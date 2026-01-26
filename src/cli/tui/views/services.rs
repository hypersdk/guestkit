// SPDX-License-Identifier: LGPL-3.0-or-later
//! Services view - Systemd services viewer

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if app.services.is_empty() {
        let empty = Paragraph::new("No systemd services found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" Systemd Services ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app.services
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|svc| {
            let status_color = if svc.enabled { SUCCESS_COLOR } else { TEXT_COLOR };
            ListItem::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled("● ", Style::default().fg(status_color)),
                ratatui::text::Span::styled(&svc.name, Style::default().fg(LIGHT_ORANGE)),
                ratatui::text::Span::raw("  "),
                ratatui::text::Span::styled(&svc.state, Style::default().fg(TEXT_COLOR)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" Systemd Services ({}) ", app.services.len()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
