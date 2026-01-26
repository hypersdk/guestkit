// SPDX-License-Identifier: LGPL-3.0-or-later
//! Issues view - Aggregated security findings and recommendations

use crate::cli::profiles::RiskLevel;
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(14), // Summary with gauges
            Constraint::Min(0),     // Issues list
        ])
        .split(area);

    draw_summary(f, chunks[0], app);
    draw_issues_list(f, chunks[1], app);
}

fn draw_summary(f: &mut Frame, area: Rect, app: &App) {
    let (critical, high, medium) = app.get_risk_summary();
    let total_issues = critical + high + medium;

    let overall_status = if critical > 0 {
        ("🔴 CRITICAL", ERROR_COLOR)
    } else if high > 0 {
        ("🟠 HIGH RISK", WARNING_COLOR)
    } else if medium > 0 {
        ("🟡 MEDIUM RISK", WARNING_COLOR)
    } else {
        ("🟢 HEALTHY", SUCCESS_COLOR)
    };

    // Split into header and gauges
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header info
            Constraint::Length(3),  // Critical gauge
            Constraint::Length(3),  // High gauge
            Constraint::Length(3),  // Medium gauge
        ])
        .split(area);

    // Header with overall status
    let summary_lines = vec![
        Line::from(vec![
            Span::styled("Overall Status: ", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled(overall_status.0, Style::default().fg(overall_status.1).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Total Issues: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", total_issues), Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw("  │  "),
            Span::styled("🔴 Critical: ", Style::default().fg(ERROR_COLOR)),
            Span::styled(format!("{}", critical), Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("🟠 High: ", Style::default().fg(WARNING_COLOR)),
            Span::styled(format!("{}", high), Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("🟡 Medium: ", Style::default().fg(WARNING_COLOR)),
            Span::styled(format!("{}", medium), Style::default().fg(WARNING_COLOR)),
        ]),
    ];

    let summary = Paragraph::new(summary_lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" ⚠️  Security & Compliance Issues ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(summary, chunks[0]);

    // Calculate percentages for gauges
    let critical_pct = if total_issues > 0 {
        (critical as f64 / total_issues as f64 * 100.0) as u16
    } else {
        0
    };

    let high_pct = if total_issues > 0 {
        (high as f64 / total_issues as f64 * 100.0) as u16
    } else {
        0
    };

    let medium_pct = if total_issues > 0 {
        (medium as f64 / total_issues as f64 * 100.0) as u16
    } else {
        0
    };

    // Critical issues gauge
    let critical_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🔴 Critical Issues "))
        .gauge_style(Style::default().fg(ERROR_COLOR))
        .percent(critical_pct)
        .label(format!("{} critical ({}% of total)", critical, critical_pct));

    f.render_widget(critical_gauge, chunks[1]);

    // High risk gauge
    let high_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🟠 High Risk Issues "))
        .gauge_style(Style::default().fg(WARNING_COLOR))
        .percent(high_pct)
        .label(format!("{} high ({}% of total)", high, high_pct));

    f.render_widget(high_gauge, chunks[2]);

    // Medium risk gauge
    let medium_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 🟡 Medium Risk Issues "))
        .gauge_style(Style::default().fg(INFO_COLOR))
        .percent(medium_pct)
        .label(format!("{} medium ({}% of total)", medium, medium_pct));

    f.render_widget(medium_gauge, chunks[3]);
}

fn draw_issues_list(f: &mut Frame, area: Rect, app: &App) {
    let mut issues: Vec<ListItem> = Vec::new();

    // Collect issues from security profile
    if let Some(security_profile) = &app.security_profile {
        for section in &security_profile.sections {
            for finding in &section.findings {
                let (icon, color) = match finding.risk_level {
                    Some(RiskLevel::Critical) => ("🔴", ERROR_COLOR),
                    Some(RiskLevel::High) => ("🟠", WARNING_COLOR),
                    Some(RiskLevel::Medium) => ("🟡", WARNING_COLOR),
                    Some(RiskLevel::Low) => ("🔵", INFO_COLOR),
                    Some(RiskLevel::Info) | None => ("ℹ️ ", TEXT_COLOR),
                };

                issues.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(icon, Style::default().fg(color)),
                        Span::raw(" "),
                        Span::styled(&section.title, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                        Span::raw(" • "),
                        Span::styled(&finding.item, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(&finding.message, Style::default().fg(LIGHT_ORANGE)),
                    ]),
                    Line::from(""),
                ]));
            }
        }
    }

    // Collect issues from hardening profile
    if let Some(hardening_profile) = &app.hardening_profile {
        for section in &hardening_profile.sections {
            for finding in &section.findings {
                let (icon, color) = match finding.risk_level {
                    Some(RiskLevel::Critical) => ("🔴", ERROR_COLOR),
                    Some(RiskLevel::High) => ("🟠", WARNING_COLOR),
                    Some(RiskLevel::Medium) => ("🟡", WARNING_COLOR),
                    Some(RiskLevel::Low) => ("🔵", INFO_COLOR),
                    Some(RiskLevel::Info) | None => ("ℹ️ ", TEXT_COLOR),
                };

                issues.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(icon, Style::default().fg(color)),
                        Span::raw(" "),
                        Span::styled(&section.title, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                        Span::raw(" [Hardening] • "),
                        Span::styled(&finding.item, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(&finding.message, Style::default().fg(LIGHT_ORANGE)),
                    ]),
                    Line::from(""),
                ]));
            }
        }
    }

    // Collect issues from compliance profile
    if let Some(compliance_profile) = &app.compliance_profile {
        for section in &compliance_profile.sections {
            for finding in &section.findings {
                let (icon, color) = match finding.risk_level {
                    Some(RiskLevel::Critical) => ("🔴", ERROR_COLOR),
                    Some(RiskLevel::High) => ("🟠", WARNING_COLOR),
                    Some(RiskLevel::Medium) => ("🟡", WARNING_COLOR),
                    Some(RiskLevel::Low) => ("🔵", INFO_COLOR),
                    Some(RiskLevel::Info) | None => ("ℹ️ ", TEXT_COLOR),
                };

                issues.push(ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(icon, Style::default().fg(color)),
                        Span::raw(" "),
                        Span::styled(&section.title, Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                        Span::raw(" [Compliance] • "),
                        Span::styled(&finding.item, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(&finding.message, Style::default().fg(LIGHT_ORANGE)),
                    ]),
                    Line::from(""),
                ]));
            }
        }
    }

    // Add basic security checks
    if &app.security.selinux == "disabled" {
        issues.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled("🟠", Style::default().fg(WARNING_COLOR)),
                Span::raw(" "),
                Span::styled("Security", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::raw(" • "),
                Span::styled("SELinux is disabled", Style::default().fg(TEXT_COLOR)),
            ]),
            Line::from(vec![
                Span::raw("   💡 "),
                Span::styled("Enable SELinux for enhanced security", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::ITALIC)),
            ]),
            Line::from(""),
        ]));
    }

    if !app.firewall.enabled {
        issues.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled("🔴", Style::default().fg(ERROR_COLOR)),
                Span::raw(" "),
                Span::styled("Firewall", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::raw(" • "),
                Span::styled("Firewall is not enabled", Style::default().fg(TEXT_COLOR)),
            ]),
            Line::from(vec![
                Span::raw("   💡 "),
                Span::styled("Enable and configure firewall to protect the system", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::ITALIC)),
            ]),
            Line::from(""),
        ]));
    }

    if !app.security.auditd {
        issues.push(ListItem::new(vec![
            Line::from(vec![
                Span::styled("🟡", Style::default().fg(WARNING_COLOR)),
                Span::raw(" "),
                Span::styled("Auditing", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::raw(" • "),
                Span::styled("auditd is not running", Style::default().fg(TEXT_COLOR)),
            ]),
            Line::from(vec![
                Span::raw("   💡 "),
                Span::styled("Enable auditd for security event logging", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::ITALIC)),
            ]),
            Line::from(""),
        ]));
    }

    let filtered_issues: Vec<_> = if app.is_searching() && !app.search_query.is_empty() {
        let query = app.search_query.to_lowercase();
        issues.into_iter()
            .filter(|item| {
                // Convert item to string representation and search
                format!("{:?}", item).to_lowercase().contains(&query)
            })
            .collect()
    } else {
        issues
    };

    let items: Vec<ListItem> = filtered_issues
        .into_iter()
        .skip(app.scroll_offset)
        .take(area.height.saturating_sub(2) as usize)
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" 📋 Detailed Findings ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)));

    f.render_widget(list, area);
}
