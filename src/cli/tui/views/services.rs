// SPDX-License-Identifier: LGPL-3.0-or-later
//! Services view - Systemd services viewer

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, ERROR_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if app.services.is_empty() {
        let empty = Paragraph::new("⚠️  No systemd services found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" ⚙️  Systemd Services ")
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
            // Determine status symbol and color based on state and enabled status
            let (status_symbol, _status_color) = match (svc.enabled, svc.state.as_str()) {
                (true, "running") => ("🟢", SUCCESS_COLOR),
                (true, "active") => ("🟢", SUCCESS_COLOR),
                (true, _) => ("🟡", WARNING_COLOR),
                (false, "running") => ("🟠", WARNING_COLOR),
                (false, _) => ("⚫", TEXT_COLOR),
            };

            // Color the state based on its value
            let state_color = match svc.state.as_str() {
                "running" | "active" => SUCCESS_COLOR,
                "stopped" | "inactive" | "failed" => ERROR_COLOR,
                _ => WARNING_COLOR,
            };

            ListItem::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::raw(format!("{} ", status_symbol)),
                ratatui::text::Span::styled(&svc.name, Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
                ratatui::text::Span::raw("  "),
                ratatui::text::Span::styled(&svc.state, Style::default().fg(state_color)),
            ]))
        })
        .collect();

    let enabled_count = app.services.iter().filter(|s| s.enabled).count();
    let running_count = app.services.iter().filter(|s| s.state == "running" || s.state == "active").count();

    // Calculate scroll position
    let visible_items = area.height.saturating_sub(2) as usize;
    let total_items = filtered_services.len();
    let scroll_pct = if total_items > 0 {
        ((app.scroll_offset as f32 / total_items.max(1) as f32) * 100.0) as u16
    } else {
        0
    };

    let scroll_indicator = if total_items > visible_items {
        format!(" 📜 {}% ", scroll_pct)
    } else {
        String::new()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" ⚙️  Systemd Services • {} showing • {} enabled • {} running{} ",
                filtered_services.len(), enabled_count, running_count, scroll_indicator))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
