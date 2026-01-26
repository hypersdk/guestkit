// SPDX-License-Identifier: LGPL-3.0-or-later
//! Dashboard view - System overview

use crate::cli::profiles::RiskLevel;
use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, ERROR_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // System info
            Constraint::Length(6),  // Profile risk summary
            Constraint::Length(8),  // Stats
            Constraint::Min(0),     // Quick info
        ])
        .split(area);

    draw_system_info(f, chunks[0], app);
    draw_profile_summary(f, chunks[1], app);
    draw_stats(f, chunks[2], app);
    draw_quick_info(f, chunks[3], app);
}

fn draw_system_info(f: &mut Frame, area: Rect, app: &App) {
    let info_lines = vec![
        Line::from(vec![
            Span::styled("OS:          ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.os_name, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Version:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.os_version, Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Kernel:      ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.kernel_version, Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Architecture:", Style::default().fg(LIGHT_ORANGE)),
            Span::raw(" "),
            Span::styled(&app.architecture, Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Hostname:    ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.hostname, Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Init System: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(&app.init_system, Style::default().fg(TEXT_COLOR)),
        ]),
    ];

    let block = Paragraph::new(info_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" System Information ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(block, area);
}

fn draw_profile_summary(f: &mut Frame, area: Rect, app: &App) {
    let mut profile_items = Vec::new();

    // Security Profile
    if let Some(ref profile) = app.security_profile {
        let (risk_text, risk_color) = if let Some(risk) = profile.overall_risk {
            match risk {
                RiskLevel::Critical => ("CRITICAL", ERROR_COLOR),
                RiskLevel::High => ("HIGH", ERROR_COLOR),
                RiskLevel::Medium => ("MEDIUM", WARNING_COLOR),
                RiskLevel::Low => ("LOW", SUCCESS_COLOR),
                RiskLevel::Info => ("OK", SUCCESS_COLOR),
            }
        } else {
            ("OK", SUCCESS_COLOR)
        };

        profile_items.push(ListItem::new(Line::from(vec![
            Span::styled("Security:    ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(risk_text, Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
        ])));
    }

    // Migration Profile
    if let Some(ref profile) = app.migration_profile {
        let (risk_text, risk_color) = if let Some(risk) = profile.overall_risk {
            match risk {
                RiskLevel::Critical => ("CRITICAL", ERROR_COLOR),
                RiskLevel::High => ("HIGH", ERROR_COLOR),
                RiskLevel::Medium => ("MEDIUM", WARNING_COLOR),
                RiskLevel::Low => ("LOW", SUCCESS_COLOR),
                RiskLevel::Info => ("OK", SUCCESS_COLOR),
            }
        } else {
            ("OK", SUCCESS_COLOR)
        };

        profile_items.push(ListItem::new(Line::from(vec![
            Span::styled("Migration:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(risk_text, Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
        ])));
    }

    // Performance Profile
    if let Some(ref profile) = app.performance_profile {
        let (risk_text, risk_color) = if let Some(risk) = profile.overall_risk {
            match risk {
                RiskLevel::Critical => ("CRITICAL", ERROR_COLOR),
                RiskLevel::High => ("HIGH", ERROR_COLOR),
                RiskLevel::Medium => ("MEDIUM", WARNING_COLOR),
                RiskLevel::Low => ("LOW", SUCCESS_COLOR),
                RiskLevel::Info => ("OK", SUCCESS_COLOR),
            }
        } else {
            ("OK", SUCCESS_COLOR)
        };

        profile_items.push(ListItem::new(Line::from(vec![
            Span::styled("Performance: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(risk_text, Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
        ])));
    }

    // Compliance Profile
    if let Some(ref profile) = app.compliance_profile {
        let (risk_text, risk_color) = if let Some(risk) = profile.overall_risk {
            match risk {
                RiskLevel::Critical => ("CRITICAL", ERROR_COLOR),
                RiskLevel::High => ("HIGH", ERROR_COLOR),
                RiskLevel::Medium => ("MEDIUM", WARNING_COLOR),
                RiskLevel::Low => ("LOW", SUCCESS_COLOR),
                RiskLevel::Info => ("OK", SUCCESS_COLOR),
            }
        } else {
            ("OK", SUCCESS_COLOR)
        };

        profile_items.push(ListItem::new(Line::from(vec![
            Span::styled("Compliance:  ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(risk_text, Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
        ])));
    }

    // Hardening Profile
    if let Some(ref profile) = app.hardening_profile {
        let (risk_text, risk_color) = if let Some(risk) = profile.overall_risk {
            match risk {
                RiskLevel::Critical => ("CRITICAL", ERROR_COLOR),
                RiskLevel::High => ("HIGH", ERROR_COLOR),
                RiskLevel::Medium => ("MEDIUM", WARNING_COLOR),
                RiskLevel::Low => ("LOW", SUCCESS_COLOR),
                RiskLevel::Info => ("OK", SUCCESS_COLOR),
            }
        } else {
            ("OK", SUCCESS_COLOR)
        };

        profile_items.push(ListItem::new(Line::from(vec![
            Span::styled("Hardening:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(risk_text, Style::default().fg(risk_color).add_modifier(Modifier::BOLD)),
        ])));
    }

    // Add helper text
    profile_items.push(ListItem::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(TEXT_COLOR)),
        Span::styled("p", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" for detailed profile reports", Style::default().fg(TEXT_COLOR)),
    ])));

    let list = List::new(profile_items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" Profile Status ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}

fn draw_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)])
        .split(area);

    // Packages gauge
    let pkg_count = app.packages.package_count;
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
            .title(" Packages ")
            .title_style(Style::default().fg(ORANGE)))
        .gauge_style(Style::default().fg(ORANGE))
        .label(pkg_label)
        .ratio(pkg_ratio);

    f.render_widget(packages_gauge, chunks[0]);

    // Services gauge
    let svc_count = app.services.len();
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
            .title(" Services ")
            .title_style(Style::default().fg(ORANGE)))
        .gauge_style(Style::default().fg(SUCCESS_COLOR))
        .label(svc_label)
        .ratio(svc_ratio);

    f.render_widget(services_gauge, chunks[1]);

    // Network gauge
    let net_count = app.network_interfaces.len();
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
            .title(" Network ")
            .title_style(Style::default().fg(ORANGE)))
        .gauge_style(Style::default().fg(WARNING_COLOR))
        .label(net_label)
        .ratio(net_ratio);

    f.render_widget(network_gauge, chunks[2]);
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
            .title(" Security ")
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
            .title(" System Details ")
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
