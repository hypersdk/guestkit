// SPDX-License-Identifier: LGPL-3.0-or-later
//! Users view - System user accounts browser

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, ERROR_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if app.users.is_empty() {
        let empty = Paragraph::new("No user accounts found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" User Accounts ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let filtered_users: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        app.users
            .iter()
            .filter(|user| {
                user.username.to_lowercase().contains(&app.search_query.to_lowercase())
                    || user.uid.contains(&app.search_query)
                    || user.shell.to_lowercase().contains(&app.search_query.to_lowercase())
                    || user.home.to_lowercase().contains(&app.search_query.to_lowercase())
            })
            .collect()
    } else {
        app.users.iter().collect()
    };

    let items: Vec<ListItem> = filtered_users
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|user| {
            // Parse UID to determine user type
            let uid: i32 = user.uid.parse().unwrap_or(99999);

            // Color coding:
            // - System users (UID < 1000): Orange
            // - Normal users (UID >= 1000): Light orange
            // - Root (UID 0): Different handling
            let (username_color, uid_color) = if uid == 0 {
                (ERROR_COLOR, ERROR_COLOR)  // Root in red
            } else if uid < 1000 {
                (WARNING_COLOR, WARNING_COLOR)  // System users in warning color
            } else {
                (SUCCESS_COLOR, LIGHT_ORANGE)  // Normal users in success color
            };

            // Detect potentially problematic shells
            let shell_color = if user.shell.contains("nologin") || user.shell.contains("false") {
                TEXT_COLOR  // Disabled shells
            } else if user.shell.contains("bash") || user.shell.contains("zsh") || user.shell.contains("sh") {
                SUCCESS_COLOR  // Interactive shells
            } else {
                WARNING_COLOR  // Other shells
            };

            ListItem::new(Line::from(vec![
                ratatui::text::Span::styled(
                    format!("{:16}", user.username),
                    Style::default().fg(username_color).add_modifier(Modifier::BOLD)
                ),
                ratatui::text::Span::styled(
                    format!("UID:{:5} ", user.uid),
                    Style::default().fg(uid_color)
                ),
                ratatui::text::Span::styled(
                    format!("GID:{:5} ", user.gid),
                    Style::default().fg(TEXT_COLOR)
                ),
                ratatui::text::Span::styled(
                    format!("{:30} ", user.home),
                    Style::default().fg(LIGHT_ORANGE)
                ),
                ratatui::text::Span::styled(
                    &user.shell,
                    Style::default().fg(shell_color)
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" User Accounts ({} / {} total) ", filtered_users.len(), app.users.len()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
