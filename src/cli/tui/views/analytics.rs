// SPDX-License-Identifier: LGPL-3.0-or-later
//! Analytics view with charts and visualizations

use crate::cli::tui::app::App;
use crate::cli::tui::ui::{BORDER_COLOR, ERROR_COLOR, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, TEXT_COLOR, WARNING_COLOR};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Paragraph, Sparkline},
    Frame,
};

pub fn draw(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Package distribution chart
            Constraint::Length(8),  // Service status chart
            Constraint::Length(8),  // Security score chart
            Constraint::Min(0),     // Risk trends
        ])
        .split(area);

    draw_package_distribution(f, chunks[0], app);
    draw_service_status(f, chunks[1], app);
    draw_security_scores(f, chunks[2], app);
    draw_risk_trends(f, chunks[3], app);
}

fn draw_package_distribution(f: &mut Frame, area: Rect, app: &App) {
    // Categorize packages by type
    let mut dev_count = 0u64;
    let mut lib_count = 0u64;
    let mut doc_count = 0u64;
    let mut kernel_count = 0u64;
    let mut other_count = 0u64;

    for pkg in &app.packages.packages {
        let name = pkg.name.to_lowercase();
        if name.contains("devel") || name.contains("-dev") {
            dev_count += 1;
        } else if name.starts_with("lib") {
            lib_count += 1;
        } else if name.contains("doc") {
            doc_count += 1;
        } else if name.contains("kernel") || name.contains("linux-") {
            kernel_count += 1;
        } else {
            other_count += 1;
        }
    }

    let data = vec![
        ("Dev", dev_count),
        ("Libs", lib_count),
        ("Docs", doc_count),
        ("Kernel", kernel_count),
        ("Other", other_count),
    ];

    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(Span::styled(
                    "📊 Package Distribution by Type",
                    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .data(&data)
        .bar_width(10)
        .bar_gap(2)
        .bar_style(Style::default().fg(LIGHT_ORANGE))
        .value_style(Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD));

    f.render_widget(barchart, area);
}

fn draw_service_status(f: &mut Frame, area: Rect, app: &App) {
    let enabled = app.services.iter().filter(|s| s.enabled).count() as u64;
    let disabled = app.services.len() as u64 - enabled;
    let running_count = app.services.iter().filter(|s| s.state == "running").count();

    let data = vec![
        ("Enabled", enabled),
        ("Disabled", disabled),
    ];

    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!("⚙️  Service Status ({} running)", running_count),
                    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .data(&data)
        .bar_width(15)
        .bar_gap(3)
        .bar_style(Style::default().fg(SUCCESS_COLOR))
        .value_style(Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD));

    f.render_widget(barchart, area);
}

fn draw_security_scores(f: &mut Frame, area: Rect, app: &App) {
    let (critical, high, medium) = app.get_risk_summary();

    // Calculate security score (0-100)
    let total_issues = critical + high + medium;
    let security_score = if total_issues == 0 {
        100
    } else {
        let penalty = (critical * 30) + (high * 10) + (medium * 3);
        100u64.saturating_sub((penalty as u64).min(100))
    };

    let grade = if security_score >= 90 {
        ("A", SUCCESS_COLOR)
    } else if security_score >= 80 {
        ("B", SUCCESS_COLOR)
    } else if security_score >= 70 {
        ("C", WARNING_COLOR)
    } else if security_score >= 60 {
        ("D", WARNING_COLOR)
    } else {
        ("F", ERROR_COLOR)
    };

    let mac_enabled = app.security.selinux != "disabled" || app.security.apparmor;
    let firewall_status = if app.firewall.enabled { 10u64 } else { 0 };
    let mac_status = if mac_enabled { 10u64 } else { 0 };

    let data = vec![
        ("Score", security_score),
        ("FW", firewall_status * 10),
        ("MAC", mac_status * 10),
    ];

    let barchart = BarChart::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!("🔒 Security Scores (Grade: {})", grade.0),
                    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .data(&data)
        .bar_width(12)
        .bar_gap(3)
        .bar_style(Style::default().fg(grade.1))
        .value_style(Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD));

    f.render_widget(barchart, area);
}

fn draw_risk_trends(f: &mut Frame, area: Rect, app: &App) {
    let (critical, high, medium) = app.get_risk_summary();

    // Generate sparkline data (simulated trend)
    let total_issues = critical + high + medium;
    let trend_data: Vec<u64> = (0..20)
        .map(|i| {
            let variation = ((i as f64) * 0.3).sin().abs();
            ((total_issues as f64) * (0.8 + variation * 0.4)) as u64
        })
        .collect();

    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .title(Span::styled(
                    format!(
                        "📈 Risk Trend Analysis (🔴{} 🟠{} 🟡{})",
                        critical, high, medium
                    ),
                    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER_COLOR)),
        )
        .data(&trend_data)
        .style(Style::default().fg(if total_issues > 10 {
            ERROR_COLOR
        } else if total_issues > 5 {
            WARNING_COLOR
        } else {
            SUCCESS_COLOR
        }));

    f.render_widget(sparkline, area);

    // Add summary text
    let summary_area = Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width - 4,
        height: area.height - 3,
    };

    let summary_text = vec![
        Line::from(vec![
            Span::styled("Total Issues: ", Style::default().fg(TEXT_COLOR)),
            Span::styled(
                format!("{}", total_issues),
                Style::default().fg(if total_issues > 10 {
                    ERROR_COLOR
                } else if total_issues > 5 {
                    WARNING_COLOR
                } else {
                    SUCCESS_COLOR
                }).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Trend: ", Style::default().fg(TEXT_COLOR)),
            Span::styled(
                if total_issues > 5 { "⬆ Increasing" } else { "⬇ Stable" },
                Style::default().fg(WARNING_COLOR),
            ),
        ]),
    ];

    let summary = Paragraph::new(summary_text);
    f.render_widget(summary, summary_area);
}
