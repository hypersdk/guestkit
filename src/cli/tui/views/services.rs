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

    let filtered_services: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        app.services
            .iter()
            .filter(|svc| {
                svc.name.to_lowercase().contains(&app.search_query.to_lowercase())
                    || svc.state.to_lowercase().contains(&app.search_query.to_lowercase())
            })
            .collect()
    } else {
        app.services.iter().collect()
    };

    let items: Vec<ListItem> = filtered_services
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
            .title(format!(" Systemd Services ({} / {} total) ", filtered_services.len(), app.services.len()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
