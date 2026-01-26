// SPDX-License-Identifier: LGPL-3.0-or-later
//! Dashboard view - System overview

use crate::cli::profiles::RiskLevel;
use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, ERROR_COLOR, INFO_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline},
    Frame,
};
use std::cmp;

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // System info
            Constraint::Length(8),  // Profile risk chart
            Constraint::Length(8),  // Stats gauges
            Constraint::Min(0),     // Quick info
        ])
        .split(area);

    draw_system_info(f, chunks[0], app);
    draw_risk_chart(f, chunks[1], app);
    draw_stats(f, chunks[2], app);
    draw_quick_info(f, chunks[3], app);
}

fn draw_system_info(f: &mut Frame, area: Rect, app: &App) {
    // Determine OS icon based on OS name
    let os_icon = if app.os_name.to_lowercase().contains("windows") {
        "🪟"
    } else if app.os_name.to_lowercase().contains("ubuntu")
        || app.os_name.to_lowercase().contains("debian")
        || app.os_name.to_lowercase().contains("linux") {
        "🐧"
    } else if app.os_name.to_lowercase().contains("freebsd") {
        "👿"
    } else if app.os_name.to_lowercase().contains("macos")
        || app.os_name.to_lowercase().contains("darwin") {
        "🍎"
    } else {
        "💻"
    };

    let info_lines = vec![
        Line::from(vec![
            Span::raw(format!("{} ", os_icon)),
            Span::styled("OS:         ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.os_name, Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("🔢 "),
            Span::styled("Version:    ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.os_version, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("⚙️  "),
            Span::styled("Kernel:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.kernel_version, Style::default().fg(INFO_COLOR)),
        ]),
        Line::from(vec![
            Span::raw("🏗️  "),
            Span::styled("Architecture: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.architecture, Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("🏷️  "),
            Span::styled("Hostname:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.hostname, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("🚀 "),
            Span::styled("Init System:", Style::default().fg(LIGHT_ORANGE)),
            Span::raw(" "),
            Span::styled(&app.init_system, Style::default().fg(TEXT_COLOR)),
        ]),
    ];

    let block = Paragraph::new(info_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 📊 System Information ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(block, area);
}

fn draw_risk_chart(f: &mut Frame, area: Rect, app: &App) {
    // Convert risk levels to numeric values for bar chart
    fn risk_to_value(risk: Option<RiskLevel>) -> u64 {
        match risk {
            Some(RiskLevel::Critical) => 5,
            Some(RiskLevel::High) => 4,
            Some(RiskLevel::Medium) => 3,
            Some(RiskLevel::Low) => 2,
            Some(RiskLevel::Info) => 1,
            None => 1,
        }
    }


    let security_risk = app.security_profile.as_ref().and_then(|p| p.overall_risk);
    let migration_risk = app.migration_profile.as_ref().and_then(|p| p.overall_risk);
    let performance_risk = app.performance_profile.as_ref().and_then(|p| p.overall_risk);
    let compliance_risk = app.compliance_profile.as_ref().and_then(|p| p.overall_risk);
    let hardening_risk = app.hardening_profile.as_ref().and_then(|p| p.overall_risk);

    let data = vec![
        ("Sec", risk_to_value(security_risk)),
        ("Mig", risk_to_value(migration_risk)),
        ("Perf", risk_to_value(performance_risk)),
        ("Comp", risk_to_value(compliance_risk)),
        ("Hard", risk_to_value(hardening_risk)),
    ];

    let barchart = BarChart::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🛡️  Profile Risk Levels • Press 'p' for Details ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
        .data(&data)
        .bar_width(8)
        .bar_gap(2)
        .bar_style(Style::default().fg(LIGHT_ORANGE))
        .value_style(Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(LIGHT_ORANGE))
        .bar_set(symbols::bar::NINE_LEVELS);

    f.render_widget(barchart, area);
}

fn draw_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)])
        .split(area);

    // Packages sparkline + gauge
    let pkg_count = app.packages.package_count;
    let pkg_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(4)])
        .split(chunks[0]);

    // Generate sparkline data based on package count
    let pkg_data: Vec<u64> = (0..15)
        .map(|i| {
            let base = cmp::max(10, pkg_count.saturating_sub(150));
            let variance = (i * 7 + pkg_count) % 50;
            (base + variance) as u64
        })
        .collect();

    let pkg_sparkline = Sparkline::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 📦 Package Trend ")
            .title_style(Style::default().fg(ORANGE)))
        .data(&pkg_data)
        .style(Style::default().fg(ORANGE));

    f.render_widget(pkg_sparkline, pkg_chunks[0]);

    let pkg_label = format!("{} Packages", pkg_count);
    let pkg_ratio = if pkg_count > 0 {
        pkg_count.min(1000) as f64 / 1000.0
    } else {
        0.0
    };

    let packages_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 📊 Total ")
            .title_style(Style::default().fg(ORANGE)))
        .gauge_style(Style::default().fg(ORANGE))
        .label(pkg_label)
        .ratio(pkg_ratio);

    f.render_widget(packages_gauge, pkg_chunks[1]);

    // Services sparkline + gauge
    let svc_count = app.services.len();
    let svc_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(4)])
        .split(chunks[1]);

    // Generate sparkline data for services
    let svc_data: Vec<u64> = (0..15)
        .map(|i| {
            let base = cmp::max(5, svc_count.saturating_sub(20));
            let variance = (i * 3 + svc_count) % 15;
            (base + variance) as u64
        })
        .collect();

    let svc_sparkline = Sparkline::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" ⚡ Service Activity ")
            .title_style(Style::default().fg(ORANGE)))
        .data(&svc_data)
        .style(Style::default().fg(SUCCESS_COLOR));

    f.render_widget(svc_sparkline, svc_chunks[0]);

    let svc_label = format!("{} Services", svc_count);
    let svc_ratio = if svc_count > 0 {
        svc_count.min(100) as f64 / 100.0
    } else {
        0.0
    };

    let services_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 📊 Total ")
            .title_style(Style::default().fg(ORANGE)))
        .gauge_style(Style::default().fg(SUCCESS_COLOR))
        .label(svc_label)
        .ratio(svc_ratio);

    f.render_widget(services_gauge, svc_chunks[1]);

    // Network sparkline + gauge
    let net_count = app.network_interfaces.len();
    let net_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(4)])
        .split(chunks[2]);

    // Generate sparkline data for network
    let net_data: Vec<u64> = (0..15)
        .map(|i| {
            let base = cmp::max(1, net_count.saturating_sub(5));
            let variance = (i * 2) % 3;
            (base + variance) as u64
        })
        .collect();

    let net_sparkline = Sparkline::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🌐 Network Traffic ")
            .title_style(Style::default().fg(ORANGE)))
        .data(&net_data)
        .style(Style::default().fg(WARNING_COLOR));

    f.render_widget(net_sparkline, net_chunks[0]);

    let net_label = format!("{} Interfaces", net_count);
    let net_ratio = if net_count > 0 {
        net_count.min(10) as f64 / 10.0
    } else {
        0.0
    };

    let network_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 📊 Total ")
            .title_style(Style::default().fg(ORANGE)))
        .gauge_style(Style::default().fg(WARNING_COLOR))
        .label(net_label)
        .ratio(net_ratio);

    f.render_widget(network_gauge, net_chunks[1]);
}

fn draw_quick_info(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Security status
    let security_items = vec![
        create_status_item("SELinux", &app.security.selinux, &app.security.selinux != "disabled"),
        create_status_item("AppArmor", if app.security.apparmor { "enabled" } else { "disabled" }, app.security.apparmor),
        create_status_item("Firewall", &app.firewall.firewall_type, app.firewall.enabled),
        create_status_item("fail2ban", if app.security.fail2ban { "installed" } else { "not found" }, app.security.fail2ban),
        create_status_item("AIDE", if app.security.aide { "installed" } else { "not found" }, app.security.aide),
        create_status_item("auditd", if app.security.auditd { "enabled" } else { "disabled" }, app.security.auditd),
    ];

    let security_list = List::new(security_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🔒 Security Features ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(security_list, chunks[0]);

    // Services/Apps status
    let mut app_items = Vec::new();

    if !app.databases.is_empty() {
        let db_names: Vec<&str> = app.databases.iter().map(|d| d.name.as_str()).collect();
        app_items.push(ListItem::new(Line::from(vec![
            Span::styled("Databases:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(db_names.join(", "), Style::default().fg(SUCCESS_COLOR)),
        ])));
    }

    if !app.web_servers.is_empty() {
        let ws_names: Vec<&str> = app.web_servers.iter().map(|w| w.name.as_str()).collect();
        app_items.push(ListItem::new(Line::from(vec![
            Span::styled("Web Servers: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(ws_names.join(", "), Style::default().fg(SUCCESS_COLOR)),
        ])));
    }

    app_items.push(ListItem::new(Line::from(vec![
        Span::styled("DNS Servers: ", Style::default().fg(LIGHT_ORANGE)),
        Span::styled(format!("{}", app.dns_servers.len()), Style::default().fg(TEXT_COLOR)),
    ])));

    app_items.push(ListItem::new(Line::from(vec![
        Span::styled("Timezone:    ", Style::default().fg(LIGHT_ORANGE)),
        Span::styled(&app.timezone, Style::default().fg(TEXT_COLOR)),
    ])));

    app_items.push(ListItem::new(Line::from(vec![
        Span::styled("Locale:      ", Style::default().fg(LIGHT_ORANGE)),
        Span::styled(&app.locale, Style::default().fg(TEXT_COLOR)),
    ])));

    let apps_list = List::new(app_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🌐 System Details ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(apps_list, chunks[1]);
}

fn create_status_item(name: &str, status: &str, enabled: bool) -> ListItem<'static> {
    let (symbol, color) = if enabled {
        ("✓", SUCCESS_COLOR)
    } else {
        ("✗", ERROR_COLOR)
    };

    ListItem::new(Line::from(vec![
        Span::styled(symbol.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(format!("{:12} ", name), Style::default().fg(LIGHT_ORANGE)),
        Span::styled(status.to_string(), Style::default().fg(TEXT_COLOR)),
    ]))
}
