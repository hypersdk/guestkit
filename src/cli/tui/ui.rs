// SPDX-License-Identifier: LGPL-3.0-or-later
//! UI rendering orchestration

use super::app::{App, View};
use super::views;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

// Coral-Terracotta Orange color theme - Pantone 7416 C inspired
pub const ORANGE: Color = Color::Rgb(222, 115, 86);        // Primary coral orange (Pantone 7416 C)
pub const DARK_ORANGE: Color = Color::Rgb(180, 85, 60);    // Darker terracotta
pub const LIGHT_ORANGE: Color = Color::Rgb(255, 145, 115); // Lighter coral
pub const BG_COLOR: Color = Color::Reset;
pub const TEXT_COLOR: Color = Color::Rgb(220, 220, 220);   // Softer white
pub const BORDER_COLOR: Color = DARK_ORANGE;
pub const SUCCESS_COLOR: Color = Color::Rgb(50, 205, 50);  // Brighter green
pub const WARNING_COLOR: Color = Color::Rgb(255, 200, 0);  // Deeper yellow
pub const ERROR_COLOR: Color = Color::Rgb(220, 50, 47);    // Deep red
pub const INFO_COLOR: Color = Color::Rgb(100, 150, 255);   // Soft blue

pub fn draw(f: &mut Frame, app: &App) {
    let constraints = if app.show_stats_bar {
        vec![
            Constraint::Length(3), // Header
            Constraint::Length(2), // Stats bar
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ]
    } else {
        vec![
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    if app.show_stats_bar {
        draw_header(f, chunks[0], app);
        draw_stats_bar(f, chunks[1], app);
        draw_tabs(f, chunks[2], app);
        draw_content(f, chunks[3], app);
        draw_footer(f, chunks[4], app);
    } else {
        draw_header(f, chunks[0], app);
        draw_tabs(f, chunks[1], app);
        draw_content(f, chunks[2], app);
        draw_footer(f, chunks[3], app);
    }

    if app.show_help {
        draw_help_overlay(f, app);
    }

    if app.show_export_menu {
        draw_export_menu(f, app);
    }

    if app.show_detail {
        draw_detail_overlay(f, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let header_text = vec![
        Line::from(vec![
            Span::styled("GuestKit", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(" - "),
            Span::styled("VM Inspector", Style::default().fg(LIGHT_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("Image: ", Style::default().fg(TEXT_COLOR)),
            Span::styled(&app.image_path, Style::default().fg(LIGHT_ORANGE)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .style(Style::default().bg(BG_COLOR)));

    f.render_widget(header, area);
}

fn draw_stats_bar(f: &mut Frame, area: Rect, app: &App) {
    let (critical, high, medium) = app.get_risk_summary();

    let stats_spans = vec![
        Span::styled("📊 ", Style::default().fg(ORANGE)),
        Span::styled("Pkgs:", Style::default().fg(LIGHT_ORANGE)),
        Span::styled(format!(" {} ", app.packages.package_count), Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled("Svcs:", Style::default().fg(LIGHT_ORANGE)),
        Span::styled(format!(" {} ", app.services.len()), Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled("Users:", Style::default().fg(LIGHT_ORANGE)),
        Span::styled(format!(" {} ", app.users.len()), Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD)),
        Span::raw("│ "),
        Span::styled("Risk:", Style::default().fg(LIGHT_ORANGE)),
        Span::raw(" "),
    ];

    let mut risk_spans = stats_spans;

    if critical > 0 {
        risk_spans.push(Span::styled(format!("🔴{} ", critical), Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD)));
    }
    if high > 0 {
        risk_spans.push(Span::styled(format!("🟠{} ", high), Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD)));
    }
    if medium > 0 {
        risk_spans.push(Span::styled(format!("🟡{} ", medium), Style::default().fg(WARNING_COLOR)));
    }
    if critical == 0 && high == 0 && medium == 0 {
        risk_spans.push(Span::styled("✓ OK", Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD)));
    }

    risk_spans.push(Span::raw("│ "));
    risk_spans.push(Span::styled("Bookmarks:", Style::default().fg(LIGHT_ORANGE)));
    risk_spans.push(Span::styled(format!(" {} ", app.bookmarks.len()), Style::default().fg(INFO_COLOR)));

    let stats = Paragraph::new(Line::from(risk_spans))
        .style(Style::default().bg(BG_COLOR).fg(TEXT_COLOR));

    f.render_widget(stats, area);
}

fn draw_tabs(f: &mut Frame, area: Rect, app: &App) {
    let views = View::all();
    let titles: Vec<String> = views.iter().map(|v| v.title().to_string()).collect();

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER_COLOR))
            .title(" Navigation ")
            .title_style(Style::default().fg(ORANGE)))
        .select(views.iter().position(|v| v == &app.current_view).unwrap_or(0))
        .style(Style::default().fg(TEXT_COLOR))
        .highlight_style(Style::default()
            .fg(ORANGE)
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::UNDERLINED));

    f.render_widget(tabs, area);
}

fn draw_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_view {
        View::Dashboard => views::dashboard::draw(f, area, app),
        View::Network => views::network::draw(f, area, app),
        View::Packages => views::packages::draw(f, area, app),
        View::Services => views::services::draw(f, area, app),
        View::Security => views::security::draw(f, area, app),
        View::Storage => views::storage::draw(f, area, app),
        View::Users => views::users::draw(f, area, app),
        View::Kernel => views::kernel::draw(f, area, app),
        View::Profiles => views::profiles::draw(f, area, app),
    }
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = if app.is_searching() {
        vec![
            Span::styled("Search: ", Style::default().fg(ORANGE)),
            Span::styled(&app.search_query, Style::default().fg(TEXT_COLOR)),
            Span::styled(" | ", Style::default().fg(DARK_ORANGE)),
            Span::raw("ESC: Cancel"),
        ]
    } else {
        vec![
            Span::styled("1-9", Style::default().fg(ORANGE)),
            Span::raw(": Jump | "),
            Span::styled("s", Style::default().fg(ORANGE)),
            Span::raw(": Sort ["),
            Span::styled(app.sort_mode.label(), Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("] | "),
            Span::styled("b", Style::default().fg(ORANGE)),
            Span::raw(": Bookmark | "),
            Span::styled("i", Style::default().fg(ORANGE)),
            Span::raw(": Stats | "),
            Span::styled("↵", Style::default().fg(ORANGE)),
            Span::raw(": Detail | "),
            Span::styled("e", Style::default().fg(ORANGE)),
            Span::raw(": Export | "),
            Span::styled("h", Style::default().fg(ORANGE)),
            Span::raw(": Help"),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .style(Style::default().bg(BG_COLOR).fg(TEXT_COLOR));

    f.render_widget(footer, area);
}

fn draw_help_overlay(f: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, f.area());

    let help_text = vec![
        Line::from(vec![
            Span::styled("GuestKit TUI - Keyboard Shortcuts",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Quick Navigation:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  1-9            ", Style::default().fg(ORANGE)),
            Span::raw("Jump directly to view (1=Dashboard, 2=Network, etc.)")
        ]),
        Line::from(vec![
            Span::styled("  Tab/Shift+Tab  ", Style::default().fg(ORANGE)),
            Span::raw("Switch between views")
        ]),
        Line::from(vec![
            Span::styled("  p              ", Style::default().fg(ORANGE)),
            Span::raw("Jump to Profiles view")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Scrolling:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓            ", Style::default().fg(ORANGE)),
            Span::raw("Scroll up/down")
        ]),
        Line::from(vec![
            Span::styled("  PgUp/PgDn      ", Style::default().fg(ORANGE)),
            Span::raw("Page up/down")
        ]),
        Line::from(vec![
            Span::styled("  Home/End       ", Style::default().fg(ORANGE)),
            Span::raw("Jump to top/bottom")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  Enter          ", Style::default().fg(ORANGE)),
            Span::raw("Toggle detail view for selected item")
        ]),
        Line::from(vec![
            Span::styled("  s              ", Style::default().fg(ORANGE)),
            Span::raw("Cycle sort mode (Default → Name ↑ → Name ↓)")
        ]),
        Line::from(vec![
            Span::styled("  b              ", Style::default().fg(ORANGE)),
            Span::raw("Bookmark current view")
        ]),
        Line::from(vec![
            Span::styled("  i              ", Style::default().fg(ORANGE)),
            Span::raw("Toggle statistics bar")
        ]),
        Line::from(vec![
            Span::styled("  /              ", Style::default().fg(ORANGE)),
            Span::raw("Start search/filter (history saved)")
        ]),
        Line::from(vec![
            Span::styled("  e              ", Style::default().fg(ORANGE)),
            Span::raw("Toggle export menu")
        ]),
        Line::from(vec![
            Span::styled("  ←/→            ", Style::default().fg(ORANGE)),
            Span::raw("Switch profile tabs (in Profiles view)")
        ]),
        Line::from(vec![
            Span::styled("  h or F1        ", Style::default().fg(ORANGE)),
            Span::raw("Toggle this help")
        ]),
        Line::from(vec![
            Span::styled("  q or ESC       ", Style::default().fg(ORANGE)),
            Span::raw("Quit / Go back / Cancel")
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C         ", Style::default().fg(ORANGE)),
            Span::raw("Force quit")
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or h to close this help",
                Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ORANGE))
            .title(" Help ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Color::Black).fg(TEXT_COLOR))
        .alignment(Alignment::Left);

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(help, area);
}

fn draw_export_menu(f: &mut Frame, app: &App) {
    use super::app::ExportMode;

    let area = centered_rect(60, 55, f.area());

    let export_text = match &app.export_mode {
        Some(ExportMode::Selecting) => {
            vec![
                Line::from(vec![
                    Span::styled("Export Menu - Select Format",
                        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Exporting: ", Style::default().fg(LIGHT_ORANGE)),
                    Span::styled(app.current_view.title(), Style::default().fg(TEXT_COLOR)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Select export format:",
                        Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  1  ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                    Span::raw("JSON  - Machine-readable data (recommended)")
                ]),
                Line::from(vec![
                    Span::styled("  2  ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                    Span::raw("YAML  - Human-readable data")
                ]),
                Line::from(vec![
                    Span::styled("  3  ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                    Span::raw("HTML  - Rich formatted report (coming soon)")
                ]),
                Line::from(vec![
                    Span::styled("  4  ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                    Span::raw("PDF   - Portable document (coming soon)")
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press 1-4 to select format, ESC to cancel",
                        Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
                ]),
            ]
        }
        Some(ExportMode::EnteringFilename) => {
            vec![
                Line::from(vec![
                    Span::styled("Export Menu - Enter Filename",
                        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Format: ", Style::default().fg(LIGHT_ORANGE)),
                    Span::styled(
                        app.export_format.map(|f| f.name()).unwrap_or("Unknown"),
                        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Filename:",
                        Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&app.export_filename, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::UNDERLINED)),
                    Span::styled("_", Style::default().fg(ORANGE)),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press Enter to export, ESC to go back",
                        Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
                ]),
            ]
        }
        Some(ExportMode::Exporting) => {
            vec![
                Line::from(vec![
                    Span::styled("Exporting...",
                        Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from("Please wait..."),
            ]
        }
        Some(ExportMode::Success(filename)) => {
            vec![
                Line::from(vec![
                    Span::styled("✓ Export Successful!",
                        Style::default().fg(SUCCESS_COLOR).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Saved to: ", Style::default().fg(LIGHT_ORANGE)),
                    Span::styled(filename, Style::default().fg(TEXT_COLOR)),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ESC or e to close",
                        Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
                ]),
            ]
        }
        Some(ExportMode::Error(error)) => {
            vec![
                Line::from(vec![
                    Span::styled("✗ Export Failed",
                        Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD))
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Error: ", Style::default().fg(ERROR_COLOR)),
                    Span::styled(error, Style::default().fg(TEXT_COLOR)),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press ESC or e to close",
                        Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
                ]),
            ]
        }
        None => {
            vec![Line::from("No export state")]
        }
    };

    let export_menu = Paragraph::new(export_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ORANGE))
            .title(" Export ")
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Color::Black).fg(TEXT_COLOR))
        .alignment(Alignment::Left);

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(export_menu, area);
}

fn draw_detail_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 80, f.area());

    let detail_text = match app.current_view {
        View::Dashboard => generate_dashboard_details(app),
        View::Network => generate_network_details(app),
        View::Packages => generate_packages_details(app),
        View::Services => generate_services_details(app),
        View::Security => generate_security_details(app),
        View::Storage => generate_storage_details(app),
        View::Users => generate_users_details(app),
        View::Kernel => generate_kernel_details(app),
        View::Profiles => generate_profiles_details(app),
    };

    let detail = Paragraph::new(detail_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ORANGE))
            .title(format!(" {} - Detailed View ", app.current_view.title()))
            .title_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(Color::Black).fg(TEXT_COLOR))
        .wrap(ratatui::widgets::Wrap { trim: false });

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(detail, area);
}

fn generate_dashboard_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("System Overview", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Operating System: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.os_name.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Version:          ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.os_version.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Kernel:           ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.kernel_version.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Architecture:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.architecture.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Hostname:         ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.hostname.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Init System:      ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.init_system.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Inventory Summary", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Packages:         ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.packages.package_count), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Services:         ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.services.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Network Interfaces:", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.network_interfaces.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("User Accounts:    ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.users.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Kernel Modules:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.kernel_modules.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_network_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Network Configuration", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Interfaces: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.network_interfaces.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("DNS Servers:      ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.dns_servers.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("DNS Server List:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
    ]
    .into_iter()
    .chain(app.dns_servers.iter().take(10).map(|dns| {
        Line::from(vec![
            Span::raw("  • "),
            Span::styled(dns.clone(), Style::default().fg(TEXT_COLOR)),
        ])
    }))
    .chain(std::iter::once(Line::from("")))
    .chain(std::iter::once(Line::from(vec![
        Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
    ])))
    .collect()
}

fn generate_packages_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Package Management", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Package Manager:  ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.packages.manager.clone(), Style::default().fg(ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("Total Packages:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.packages.package_count), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Sort Mode:        ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.sort_mode.label().to_string(), Style::default().fg(ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press 's' to cycle sort modes", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_services_details(app: &App) -> Vec<Line<'static>> {
    let enabled_count = app.services.iter().filter(|s| s.enabled).count();
    vec![
        Line::from(vec![
            Span::styled("System Services", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Services:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.services.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Enabled:          ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", enabled_count), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Disabled:         ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.services.len() - enabled_count), Style::default().fg(WARNING_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_security_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Security Configuration", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("SELinux:    ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.security.selinux.clone(), Style::default().fg(if &app.security.selinux != "disabled" { SUCCESS_COLOR } else { WARNING_COLOR })),
        ]),
        Line::from(vec![
            Span::styled("AppArmor:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(if app.security.apparmor { "enabled" } else { "disabled" }, Style::default().fg(if app.security.apparmor { SUCCESS_COLOR } else { WARNING_COLOR })),
        ]),
        Line::from(vec![
            Span::styled("fail2ban:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(if app.security.fail2ban { "installed" } else { "not installed" }, Style::default().fg(if app.security.fail2ban { SUCCESS_COLOR } else { WARNING_COLOR })),
        ]),
        Line::from(vec![
            Span::styled("AIDE:       ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(if app.security.aide { "installed" } else { "not installed" }, Style::default().fg(if app.security.aide { SUCCESS_COLOR } else { WARNING_COLOR })),
        ]),
        Line::from(vec![
            Span::styled("auditd:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(if app.security.auditd { "enabled" } else { "disabled" }, Style::default().fg(if app.security.auditd { SUCCESS_COLOR } else { WARNING_COLOR })),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Firewall:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.firewall.firewall_type.clone(), Style::default().fg(TEXT_COLOR)),
            Span::raw(" "),
            Span::styled(if app.firewall.enabled { "(enabled)" } else { "(disabled)" }, Style::default().fg(if app.firewall.enabled { SUCCESS_COLOR } else { ERROR_COLOR })),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_storage_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Storage Configuration", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Mount Points:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.fstab.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("LVM Configured:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(if app.lvm_info.is_some() { "Yes" } else { "No" }, Style::default().fg(if app.lvm_info.is_some() { SUCCESS_COLOR } else { TEXT_COLOR })),
        ]),
        Line::from(vec![
            Span::styled("RAID Arrays:      ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.raid_arrays.len()), Style::default().fg(if app.raid_arrays.is_empty() { TEXT_COLOR } else { SUCCESS_COLOR })),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_users_details(app: &App) -> Vec<Line<'static>> {
    let root_users = app.users.iter().filter(|u| u.uid == "0").count();
    let system_users = app.users.iter().filter(|u| {
        if let Ok(uid) = u.uid.parse::<i32>() {
            uid > 0 && uid < 1000
        } else {
            false
        }
    }).count();
    let normal_users = app.users.iter().filter(|u| {
        if let Ok(uid) = u.uid.parse::<i32>() {
            uid >= 1000
        } else {
            false
        }
    }).count();

    vec![
        Line::from(vec![
            Span::styled("User Accounts", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Users:      ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.users.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Root (UID 0):     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", root_users), Style::default().fg(if root_users > 1 { ERROR_COLOR } else { SUCCESS_COLOR })),
        ]),
        Line::from(vec![
            Span::styled("System Users:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", system_users), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Normal Users:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", normal_users), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_kernel_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Kernel Configuration", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Boot Modules:     ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.kernel_modules.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Kernel Parameters:", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.kernel_params.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_profiles_details(app: &App) -> Vec<Line<'static>> {
    let profiles_available = [
        ("Security", app.security_profile.is_some()),
        ("Migration", app.migration_profile.is_some()),
        ("Performance", app.performance_profile.is_some()),
        ("Compliance", app.compliance_profile.is_some()),
        ("Hardening", app.hardening_profile.is_some()),
    ];

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Profile Reports", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
    ];

    for (name, available) in &profiles_available {
        lines.push(Line::from(vec![
            Span::styled(format!("{:12} ", name), Style::default().fg(LIGHT_ORANGE)),
            Span::styled(if *available { "✓ Available" } else { "✗ Not available" }, Style::default().fg(if *available { SUCCESS_COLOR } else { WARNING_COLOR })),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Use ←/→ to switch between profile tabs", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
    ]));
    lines.push(Line::from(vec![
        Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
    ]));

    lines
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
