// SPDX-License-Identifier: LGPL-3.0-or-later
//! Packages view - Installed packages browser

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, LIGHT_ORANGE, ORANGE, TEXT_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if app.packages.packages.is_empty() {
        let empty = Paragraph::new(format!("No packages found (manager: {})", app.packages.manager))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" Installed Packages ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let filtered_packages: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        app.packages.packages
            .iter()
            .filter(|pkg| {
                pkg.name.to_lowercase().contains(&app.search_query.to_lowercase())
                    || pkg.version.contains(&app.search_query)
            })
            .collect()
    } else {
        app.packages.packages.iter().collect()
    };

    let items: Vec<ListItem> = filtered_packages
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|pkg| {
            ListItem::new(Line::from(vec![
                ratatui::text::Span::styled(&pkg.name, Style::default().fg(LIGHT_ORANGE)),
                ratatui::text::Span::raw("  "),
                ratatui::text::Span::styled(&pkg.version, Style::default().fg(TEXT_COLOR)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" Packages ({} / {} total) - Manager: {} ",
                filtered_packages.len(), app.packages.package_count, app.packages.manager))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
