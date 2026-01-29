// SPDX-License-Identifier: LGPL-3.0-or-later
//! Services view - Systemd services viewer

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, ERROR_COLOR, INFO_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
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

    // Split area into summary and list sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Summary gauges
            Constraint::Min(0),     // Service list
        ])
        .split(area);

    draw_service_summary(f, chunks[0], app);
    draw_service_list(f, chunks[1], app);
}

fn draw_service_summary(f: &mut Frame, area: Rect, app: &App) {
    let enabled_count = app.services.iter().filter(|s| s.enabled).count();
    let disabled_count = app.services.len() - enabled_count;
    let running_count = app.services.iter().filter(|s| s.state == "running" || s.state == "active").count();
    let stopped_count = app.services.len() - running_count;

    let enabled_pct = if app.services.len() > 0 {
        (enabled_count as f64 / app.services.len() as f64 * 100.0) as u16
    } else {
        0
    };

    let running_pct = if app.services.len() > 0 {
        (running_count as f64 / app.services.len() as f64 * 100.0) as u16
    } else {
        0
    };

    // Split into two gauge sections
    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(3),  // Enabled gauge
            Constraint::Length(3),  // Running gauge
            Constraint::Length(1),  // Padding
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" 📊 Service Status Overview", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
    ]));
    f.render_widget(header, gauge_chunks[0]);

    // Enabled/Disabled gauge
    let enabled_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" Enabled Services "))
        .gauge_style(Style::default().fg(SUCCESS_COLOR))
        .percent(enabled_pct)
        .label(format!("{} enabled • {} disabled ({}%)", enabled_count, disabled_count, enabled_pct));

    f.render_widget(enabled_gauge, gauge_chunks[1]);

    // Running/Stopped gauge
    let running_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" Running Services "))
        .gauge_style(Style::default().fg(INFO_COLOR))
        .percent(running_pct)
        .label(format!("{} running • {} stopped ({}%)", running_count, stopped_count, running_pct));

    f.render_widget(running_gauge, gauge_chunks[2]);
}

fn draw_service_list(f: &mut Frame, area: Rect, app: &App) {

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
        .enumerate()
        .map(|(idx, svc)| {
            let actual_idx = app.scroll_offset + idx;

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

            // Multi-select checkbox
            let checkbox = if app.multi_select_mode {
                if app.is_item_selected(actual_idx) {
                    "☑ "
                } else {
                    "☐ "
                }
            } else {
                ""
            };

            ListItem::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::raw(checkbox),
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

    // Multi-select indicator
    let multiselect_indicator = if app.multi_select_mode {
        format!(" [{}  selected] ", app.get_selected_count())
    } else {
        String::new()
    };

    // Filter indicator
    let filter_indicator = if let Some(label) = app.get_filter_label() {
        format!(" [{}] ", label)
    } else {
        String::new()
    };

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" ⚙️  Systemd Services • {} showing • {} enabled • {} running{}{}{} ",
                filtered_services.len(), enabled_count, running_count, scroll_indicator, multiselect_indicator, filter_indicator))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
