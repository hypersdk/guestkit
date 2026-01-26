// SPDX-License-Identifier: LGPL-3.0-or-later
//! Databases view - Database installations and configurations

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
    if app.databases.is_empty() {
        let empty = Paragraph::new("⚠️  No database installations found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" 🗄️  Databases ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let filtered_databases: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        app.databases
            .iter()
            .filter(|db| {
                db.name.to_lowercase().contains(&app.search_query.to_lowercase())
                    || db.data_dir.to_lowercase().contains(&app.search_query.to_lowercase())
                    || db.config_path.to_lowercase().contains(&app.search_query.to_lowercase())
            })
            .collect()
    } else {
        app.databases.iter().collect()
    };

    let items: Vec<ListItem> = filtered_databases
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|db| {
            // Determine database icon and color
            let (icon, db_color) = match db.name.to_lowercase().as_str() {
                s if s.contains("postgres") || s.contains("postgresql") => ("🐘", SUCCESS_COLOR),
                s if s.contains("mysql") || s.contains("mariadb") => ("🐬", LIGHT_ORANGE),
                s if s.contains("mongodb") || s.contains("mongo") => ("🍃", SUCCESS_COLOR),
                s if s.contains("redis") => ("💎", WARNING_COLOR),
                s if s.contains("sqlite") => ("📦", TEXT_COLOR),
                _ => ("🗄️", TEXT_COLOR),
            };

            ListItem::new(Line::from(vec![
                ratatui::text::Span::raw(format!("{} ", icon)),
                ratatui::text::Span::styled(
                    format!("{:20} ", db.name),
                    Style::default().fg(db_color).add_modifier(Modifier::BOLD)
                ),
                ratatui::text::Span::styled(
                    format!("data: {:25} ", db.data_dir),
                    Style::default().fg(TEXT_COLOR)
                ),
                ratatui::text::Span::styled(
                    format!("config: {}", db.config_path),
                    Style::default().fg(LIGHT_ORANGE)
                ),
            ]))
        })
        .collect();

    // Calculate scroll position
    let visible_items = area.height.saturating_sub(2) as usize;
    let total_items = filtered_databases.len();
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
            .title(format!(" 🗄️  Databases • {} total{} ",
                filtered_databases.len(), scroll_indicator))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
