// SPDX-License-Identifier: LGPL-3.0-or-later
//! Packages view - Installed packages browser

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, INFO_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    // Determine package manager icon
    let manager_icon = match app.packages.manager.to_lowercase().as_str() {
        "rpm" | "dnf" | "yum" => "📦",
        "deb" | "apt" | "dpkg" => "📦",
        "pacman" => "📦",
        "apk" => "📦",
        "zypper" => "📦",
        _ => "📦",
    };

    if app.packages.packages.is_empty() {
        let empty = Paragraph::new(format!("⚠️  No packages found (manager: {})", app.packages.manager))
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(format!(" {} Installed Packages ", manager_icon))
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
        .enumerate()
        .map(|(idx, pkg)| {
            // Alternate colors for better readability
            let name_color = if idx % 2 == 0 { LIGHT_ORANGE } else { ORANGE };

            ListItem::new(Line::from(vec![
                ratatui::text::Span::raw("• "),
                ratatui::text::Span::styled(&pkg.name, Style::default().fg(name_color).add_modifier(Modifier::BOLD)),
                ratatui::text::Span::raw("  "),
                ratatui::text::Span::styled("v", Style::default().fg(INFO_COLOR)),
                ratatui::text::Span::styled(&pkg.version, Style::default().fg(SUCCESS_COLOR)),
            ]))
        })
        .collect();

    // Calculate scroll position percentage
    let visible_items = area.height.saturating_sub(2) as usize;
    let total_items = filtered_packages.len();
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
            .title(format!(" {} Installed Packages • {} showing of {} total • Manager: {}{} ",
                manager_icon, filtered_packages.len(), app.packages.package_count, app.packages.manager, scroll_indicator))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
