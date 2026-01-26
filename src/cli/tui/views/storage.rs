// SPDX-License-Identifier: LGPL-3.0-or-later
//! Storage view - Disk, LVM, and mount points

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, LIGHT_ORANGE, ORANGE, TEXT_COLOR};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    if app.fstab.is_empty() {
        let empty = Paragraph::new("No fstab entries found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" Storage / Mount Points ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app.fstab
        .iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .map(|(device, mountpoint, fstype)| {
            ListItem::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(format!("{:20} ", device), Style::default().fg(LIGHT_ORANGE)),
                ratatui::text::Span::raw("→ "),
                ratatui::text::Span::styled(format!("{:20} ", mountpoint), Style::default().fg(TEXT_COLOR)),
                ratatui::text::Span::styled(format!("({})", fstype), Style::default().fg(ORANGE)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" /etc/fstab Entries ({}) ", app.fstab.len()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
