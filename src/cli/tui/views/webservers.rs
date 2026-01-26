// SPDX-License-Identifier: LGPL-3.0-or-later
//! Web servers view - Web server installations and configurations

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if app.web_servers.is_empty() {
        let empty = Paragraph::new("⚠️  No web server installations found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" 🌐 Web Servers ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let filtered_webservers: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        app.web_servers
            .iter()
            .filter(|ws| {
                ws.name.to_lowercase().contains(&app.search_query.to_lowercase())
                    || ws.version.to_lowercase().contains(&app.search_query.to_lowercase())
                    || ws.config_path.to_lowercase().contains(&app.search_query.to_lowercase())
            })
            .collect()
    } else {
        app.web_servers.iter().collect()
    };

    let items: Vec<ListItem> = filtered_webservers
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|ws| {
            // Determine web server icon and color
            let (icon, server_color) = match ws.name.to_lowercase().as_str() {
                s if s.contains("nginx") => ("⚡", SUCCESS_COLOR),
                s if s.contains("apache") || s.contains("httpd") => ("🪶", WARNING_COLOR),
                s if s.contains("caddy") => ("📦", LIGHT_ORANGE),
                s if s.contains("lighttpd") => ("💡", TEXT_COLOR),
                _ => ("🌐", TEXT_COLOR),
            };

            // Determine enabled status
            let status = if ws.enabled {
                ("✓", SUCCESS_COLOR)
            } else {
                ("✗", WARNING_COLOR)
            };

            ListItem::new(Line::from(vec![
                ratatui::text::Span::raw(format!("{} ", icon)),
                ratatui::text::Span::styled(
                    format!("{:15} ", ws.name),
                    Style::default().fg(server_color).add_modifier(Modifier::BOLD)
                ),
                ratatui::text::Span::styled(
                    format!("{} ", status.0),
                    Style::default().fg(status.1).add_modifier(Modifier::BOLD)
                ),
                ratatui::text::Span::styled(
                    format!("v{:10} ", ws.version),
                    Style::default().fg(LIGHT_ORANGE)
                ),
                ratatui::text::Span::styled(
                    format!("config: {}", ws.config_path),
                    Style::default().fg(TEXT_COLOR)
                ),
            ]))
        })
        .collect();

    // Calculate scroll position
    let visible_items = area.height.saturating_sub(2) as usize;
    let total_items = filtered_webservers.len();
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

    let enabled_count = app.web_servers.iter().filter(|ws| ws.enabled).count();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" 🌐 Web Servers • {} total • {} enabled{} ",
                filtered_webservers.len(), enabled_count, scroll_indicator))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
