// SPDX-License-Identifier: LGPL-3.0-or-later
//! Network view - Network configuration details

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, LIGHT_ORANGE, ORANGE, TEXT_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Interfaces table
            Constraint::Length(10), // DNS servers
        ])
        .split(area);

    draw_interfaces(f, chunks[0], app);
    draw_dns_servers(f, chunks[1], app);
}

fn draw_interfaces(f: &mut Frame, area: Rect, app: &App) {
    if app.network_interfaces.is_empty() {
        let empty = Paragraph::new("⚠️  No network interfaces found")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" 🌐 Network Interfaces ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let header = Row::new(vec!["Interface", "IP Address", "MAC Address", "DHCP", "DNS Servers"])
        .style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let mut rows = Vec::new();
    let filtered_interfaces: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        app.network_interfaces
            .iter()
            .filter(|iface| {
                iface.name.to_lowercase().contains(&app.search_query.to_lowercase())
                    || iface.ip_address.iter().any(|ip| ip.contains(&app.search_query))
                    || iface.mac_address.to_lowercase().contains(&app.search_query.to_lowercase())
            })
            .collect()
    } else {
        app.network_interfaces.iter().collect()
    };

    for (idx, iface) in filtered_interfaces.iter().enumerate() {
        let is_selected = idx == app.selected_index;
        let style = if is_selected {
            Style::default().bg(ORANGE).fg(ratatui::style::Color::Black).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(TEXT_COLOR)
        };

        let ip_addrs = if iface.ip_address.is_empty() {
            "—".to_string()
        } else {
            iface.ip_address.join(", ")
        };

        let mac_addr = if iface.mac_address.is_empty() {
            "—".to_string()
        } else {
            iface.mac_address.clone()
        };

        let dns_count = if iface.dns_servers.is_empty() {
            "—".to_string()
        } else {
            format!("{} configured", iface.dns_servers.len())
        };

        rows.push(Row::new(vec![
            iface.name.clone(),
            ip_addrs,
            mac_addr,
            format!("{}", if iface.dhcp { "Yes" } else { "No" }),
            dns_count,
        ]).style(style));
    }

    let widths = [
        Constraint::Length(15),
        Constraint::Length(25),
        Constraint::Length(20),
        Constraint::Length(8),
        Constraint::Min(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" 🌐 Network Interfaces • {} configured ", filtered_interfaces.len()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
        .column_spacing(2);

    f.render_widget(table, area);
}

fn draw_dns_servers(f: &mut Frame, area: Rect, app: &App) {
    if app.dns_servers.is_empty() {
        let empty = Paragraph::new("⚠️  No DNS servers configured")
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR))
                .title(" 🌍 DNS Servers ")
                .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(TEXT_COLOR));
        f.render_widget(empty, area);
        return;
    }

    let dns_items: Vec<ListItem> = app.dns_servers
        .iter()
        .enumerate()
        .map(|(idx, server)| {
            let bullet = if idx == 0 { "◆" } else { "◇" };
            let bullet_color = if idx == 0 { ORANGE } else { LIGHT_ORANGE };

            ListItem::new(Line::from(vec![
                Span::styled(bullet, Style::default().fg(bullet_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled(server, Style::default().fg(TEXT_COLOR)),
            ]))
        })
        .collect();

    let dns_list = List::new(dns_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(format!(" 🌍 DNS Servers • {} configured ", app.dns_servers.len()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(dns_list, area);
}
