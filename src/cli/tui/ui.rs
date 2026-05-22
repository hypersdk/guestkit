// SPDX-License-Identifier: LGPL-3.0-or-later
//! UI rendering orchestration

use super::app::{App, View};
use super::theme::{self, fill_background, key_muted, key_primary, pane_block, pane_block_with_border, risk_border_color};
use super::views;
pub use super::theme::{
    content_block, ACCENT, ACCENT_SOFT, BG, BG_COLOR, BORDER_COLOR, DARK_ORANGE, ERROR_COLOR,
    focus_style, INFO_COLOR, label_style, LIGHT_ORANGE, ORANGE, SUCCESS_COLOR, SURFACE,
    TEXT_COLOR, TEXT_MUTED, value_style, WARNING_COLOR,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Tabs,
    },
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    f.render_widget(fill_background(), f.area());
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

    if app.show_file_preview {
        draw_file_preview(f, app);
    }

    if app.show_file_info {
        draw_file_info(f, app);
    }

    if app.notification.is_some() {
        draw_notification(f, app);
    }

    if app.show_jump_menu {
        draw_jump_menu(f, app);
    }

    if app.show_palette {
        draw_palette(f, app);
    }

    if app.loading.is_some() {
        draw_loading_banner(f, app);
    }
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    // Get current view icon and description
    let (view_icon, view_desc) = match app.current_view {
        View::Dashboard => ("📊", "System Overview"),
        View::Analytics => ("📈", "Analytics & Charts"),
        View::Timeline => ("⏰", "System Timeline"),
        View::Recommendations => ("💡", "Smart Recommendations"),
        View::Topology => ("🏗️ ", "System Topology"),
        View::Network => ("🌐", "Network Configuration"),
        View::Packages => ("📦", "Installed Packages"),
        View::Services => ("⚙️ ", "System Services"),
        View::Databases => ("🗄️ ", "Database Installations"),
        View::WebServers => ("🌐", "Web Server Installations"),
        View::Security => ("🔒", "Security Features"),
        View::Issues => ("⚠️ ", "Security Issues & Findings"),
        View::Storage => ("💾", "Storage & Filesystems"),
        View::Users => ("👥", "User Accounts"),
        View::Kernel => ("🧩", "Kernel Configuration"),
        View::Logs => ("📋", "System Logs"),
        View::Profiles => ("🛡️ ", "Profile Reports"),
        View::Files => ("📂", "File Browser"),
    };

    let header_text = vec![
        Line::from(vec![
            Span::styled("GuestKit", Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
            Span::styled(" · ", theme::label_style()),
            Span::styled("Zyvor", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("  │  ", theme::label_style()),
            Span::raw(format!("{} ", view_icon)),
            Span::styled(app.current_view.title(), Style::default().fg(TEXT_COLOR).add_modifier(Modifier::BOLD)),
            Span::styled(": ", theme::label_style()),
            Span::styled(view_desc, theme::value_style()),
        ]),
        Line::from(vec![
            Span::styled("Image ", theme::label_style()),
            Span::styled(&app.image_path, theme::value_style()),
        ]),
    ];

    let (critical, high, medium) = app.get_risk_summary();
    let header_border = risk_border_color(critical, high, medium);
    let header = Paragraph::new(header_text)
        .block(pane_block_with_border(" HyperSDK · zyvor.dev · GuestKit ", header_border));

    f.render_widget(header, area);
}

fn draw_stats_bar(f: &mut Frame, area: Rect, app: &App) {
    let (critical, high, medium) = app.get_risk_summary();

    let stats_spans = vec![
        Span::styled("📊 ", Style::default().fg(TEXT_MUTED)),
        Span::styled("Pkgs:", theme::label_style()),
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
        .style(Style::default().bg(SURFACE).fg(TEXT_COLOR));

    f.render_widget(stats, area);
}

fn draw_tabs(f: &mut Frame, area: Rect, app: &App) {
    let ordered = app.tab_titles_ordered();
    let titles: Vec<String> = ordered.iter().map(|(_, t)| t.clone()).collect();
    let select = ordered
        .iter()
        .position(|(v, _)| *v == app.current_view)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(pane_block("Navigation (pinned first)", false))
        .select(select)
        .style(Style::default().fg(TEXT_MUTED))
        .highlight_style(theme::focus_style().add_modifier(Modifier::UNDERLINED));

    f.render_widget(tabs, area);
}

fn draw_loading_banner(f: &mut Frame, app: &App) {
    if let Some(ref loading) = app.loading {
        let area = Rect {
            x: f.area().x + 2,
            y: f.area().y + 1,
            width: f.area().width.saturating_sub(4),
            height: 1,
        };
        let line = Line::from(vec![
            Span::styled("◐ ", Style::default().fg(ACCENT)),
            Span::styled(loading.progress_label(), theme::value_style()),
        ]);
        f.render_widget(Paragraph::new(line), area);
    }
}

fn draw_palette(f: &mut Frame, app: &App) {
    let area = centered_rect(70, 60, f.area());
    f.render_widget(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ACCENT))
            .title(" Command palette ")
            .style(Style::default().bg(SURFACE)),
        area,
    );
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };
    let cmds = super::palette::filtered_commands(&app.palette_query);
    let mut lines = vec![Line::from(vec![
        Span::styled(": ", Style::default().fg(ACCENT)),
        Span::styled(&app.palette_query, theme::value_style()),
    ])];
    for (i, (name, desc)) in cmds.iter().take(inner.height as usize - 2).enumerate() {
        let style = if i == app.palette_selected {
            theme::focus_style()
        } else {
            Style::default().fg(TEXT_COLOR)
        };
        lines.push(Line::from(vec![
            Span::styled(format!("  {}  ", name), style),
            Span::styled(*desc, theme::label_style()),
        ]));
    }
    lines.push(Line::from(Span::styled(
        "Enter run • Esc cancel",
        theme::label_style(),
    )));
    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_view {
        View::Dashboard => views::dashboard::draw(f, area, app),
        View::Analytics => views::analytics::draw(f, area, app),
        View::Timeline => views::timeline::draw(f, area, app),
        View::Recommendations => views::recommendations::draw(f, area, app),
        View::Topology => views::topology::draw(f, area, app),
        View::Network => views::network::draw(f, area, app),
        View::Packages => views::packages::draw(f, area, app),
        View::Services => views::services::draw(f, area, app),
        View::Databases => views::databases::draw(f, area, app),
        View::WebServers => views::webservers::draw(f, area, app),
        View::Security => views::security::draw(f, area, app),
        View::Issues => views::issues::draw(f, area, app),
        View::Storage => views::storage::draw(f, area, app),
        View::Users => views::users::draw(f, area, app),
        View::Kernel => views::kernel::draw(f, area, app),
        View::Logs => views::logs::draw(f, area, app),
        View::Profiles => views::profiles::draw(f, area, app),
        View::Files => views::files::draw(f, area, app),
    }
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = if app.is_searching() {
        let mode_indicator = app.get_search_mode_indicator();
        vec![
            Span::styled("🔍 ", Style::default().fg(ORANGE)),
            Span::styled("Search: ", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
            Span::styled(mode_indicator, Style::default().fg(INFO_COLOR)),
            Span::styled(&app.search_query, Style::default().fg(TEXT_COLOR).add_modifier(Modifier::UNDERLINED)),
            Span::styled("_", Style::default().fg(ORANGE)),
            Span::styled(" │ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Ctrl+I", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(": Case • "),
            Span::styled("Ctrl+R", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(": Regex • "),
            Span::styled("ESC", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(": Cancel • "),
            Span::styled("Enter", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw(": Finish"),
        ]
    } else {
        vec![
            Span::styled("⌨ ", key_muted()),
            Span::styled("Tab", key_primary()),
            Span::styled(": views │ ", key_muted()),
            Span::styled("/", key_primary()),
            Span::styled(": search │ ", key_muted()),
            Span::styled("e", key_primary()),
            Span::styled(": export │ ", key_muted()),
            Span::styled(":", key_primary()),
            Span::styled(": cmd │ ", key_muted()),
            Span::styled("?", key_primary()),
            Span::styled(": help │ ", key_muted()),
            Span::styled("q", Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD)),
            Span::styled(": quit │ ", key_muted()),
            Span::styled(app.get_time_since_update(), theme::value_style()),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .style(Style::default().bg(SURFACE).fg(TEXT_COLOR));

    f.render_widget(footer, area);
}

fn draw_help_overlay(f: &mut Frame, app: &App) {
    let area = centered_rect(75, if app.help_context { 45 } else { 85 }, f.area());

    if app.help_context {
        let ctx = context_help_for_view(app.current_view);
        let help_text: Vec<Line> = ctx
            .into_iter()
            .map(|s| Line::from(Span::raw(s)))
            .collect();
        let block = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(ACCENT))
                    .title(format!(" Help: {} ", app.current_view.title()))
                    .style(Style::default().bg(SURFACE)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(block, area);
        return;
    }

    let help_text = vec![
        Line::from(vec![
            Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                Style::default().fg(ORANGE))
        ]),
        Line::from(vec![
            Span::raw("                    "),
            Span::styled("🔍 GuestKit TUI - Complete Keyboard Reference",
                Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━",
                Style::default().fg(ORANGE))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("NAVIGATION & MOVEMENT", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled(" ─────────────────────────────────────────────────┐", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("1-9          ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Quick jump: 1=Dashboard 2=Network 3=Packages 4=Services     "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("             ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("            5=DBs 6=WebServers 7=Security 8=Issues 9=Storage"),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Tab          ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Next view                                                         "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Shift+Tab    ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Previous view                                                     "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("p            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Jump to Profiles view                                            "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("←/→          ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Switch profile tabs (when in Profiles view)                      "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Ctrl+P       ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Quick jump menu with fuzzy search                                "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(DARK_ORANGE)),
            Span::styled("────────────────────────────────────────────────────────────────────────┘", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("SCROLLING & BROWSING", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled(" ───────────────────────────────────────────────────┐", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("↑/↓ or j/k   ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Scroll up/down one line (vim-style)                              "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("PgUp/PgDn    ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Scroll up/down one page (also Ctrl-u/Ctrl-d)                     "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Home or g    ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Jump to top of list (vim-style)                                  "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("End or G     ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Jump to bottom of list (vim-style)                               "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(DARK_ORANGE)),
            Span::styled("────────────────────────────────────────────────────────────────────────┘", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("DATA OPERATIONS", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled(" ─────────────────────────────────────────────────────┐", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("r            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Refresh data and update timestamp                                "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("s            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Cycle sort mode (Default → Name ↑ → Name ↓)                      "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("/            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Start search/filter (saved to history)                           "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Ctrl+I       ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle case-sensitive search (while searching)                   "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Ctrl+R       ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle regex mode (while searching)                              "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Enter        ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Show detailed view for current item                              "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("e            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Open export menu (JSON, YAML, HTML, PDF)                         "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(DARK_ORANGE)),
            Span::styled("────────────────────────────────────────────────────────────────────────┘", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("MULTI-SELECT & FILTERING", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled(" ──────────────────────────────────────────────┐", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("m            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle multi-select mode (shows checkboxes)                      "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Space        ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle selection for current item (in multi-select mode)         "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Ctrl+A       ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Select all items in current view                                 "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("f            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Cycle through quick filters (critical/enabled/running/etc.)      "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("l            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle live filtering (filter as you type in search)             "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(DARK_ORANGE)),
            Span::styled("────────────────────────────────────────────────────────────────────────┘", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("UI CUSTOMIZATION", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled(" ──────────────────────────────────────────────────────┐", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("i            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle statistics bar (shows pkgs, services, risk summary)       "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("b            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Bookmark current view (max 20 bookmarks)                         "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("t            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle table/list view mode (table has columns)                  "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("c            ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle comparison mode (shows changes since snapshot)            "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("h or F1      ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle this help overlay                                         "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(DARK_ORANGE)),
            Span::styled("────────────────────────────────────────────────────────────────────────┘", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌─ ", Style::default().fg(DARK_ORANGE)),
            Span::styled("EXIT & CONTROL", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)),
            Span::styled(" ────────────────────────────────────────────────────────┐", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("q or ESC     ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Quit application / Close overlay / Cancel action                "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("│  ", Style::default().fg(DARK_ORANGE)),
            Span::styled("Ctrl+C       ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
            Span::raw("Force quit immediately                                           "),
            Span::styled("   │", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(DARK_ORANGE)),
            Span::styled("────────────────────────────────────────────────────────────────────────┘", Style::default().fg(DARK_ORANGE)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("💡 ", Style::default().fg(WARNING_COLOR)),
            Span::styled("Pro Tips:", Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD)),
            Span::raw("  Use number keys for fastest navigation • Search is live and instant")
        ]),
        Line::from(vec![
            Span::raw("             Color coding: "),
            Span::styled("🟢 OK/Running", Style::default().fg(SUCCESS_COLOR)),
            Span::raw(" • "),
            Span::styled("🟡 Warning", Style::default().fg(WARNING_COLOR)),
            Span::raw(" • "),
            Span::styled("🔴 Error/Critical", Style::default().fg(ERROR_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC, h, or F1 to close this help",
                Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
            .title(vec![
                Span::raw(" "),
                Span::styled("📖 Help & Keyboard Shortcuts", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
            ])
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
                    Span::raw("HTML  - Rich formatted report")
                ]),
                Line::from(vec![
                    Span::styled("  4  ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                    Span::raw("PDF   - Portable document (CLI only)")
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
        View::Analytics => generate_analytics_details(app),
        View::Timeline => generate_timeline_details(app),
        View::Recommendations => generate_recommendations_details(app),
        View::Topology => generate_topology_details(app),
        View::Network => generate_network_details(app),
        View::Packages => generate_packages_details(app),
        View::Services => generate_services_details(app),
        View::Databases => generate_databases_details(app),
        View::WebServers => generate_webservers_details(app),
        View::Security => generate_security_details(app),
        View::Issues => generate_issues_details(app),
        View::Storage => generate_storage_details(app),
        View::Users => generate_users_details(app),
        View::Kernel => generate_kernel_details(app),
        View::Logs => generate_logs_details(app),
        View::Profiles => generate_profiles_details(app),
        View::Files => {
            // Files view doesn't use detail overlay - file preview/info overlays are used instead
            vec![Line::from("Use 'v' to preview files and 'i' to view file information.")]
        },
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

fn generate_databases_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Database Installations", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Databases:  ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.databases.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_webservers_details(app: &App) -> Vec<Line<'static>> {
    let enabled_count = app.web_servers.iter().filter(|ws| ws.enabled).count();
    vec![
        Line::from(vec![
            Span::styled("Web Server Installations", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Total Web Servers:", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.web_servers.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Enabled:          ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", enabled_count), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Disabled:         ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", app.web_servers.len() - enabled_count), Style::default().fg(WARNING_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_issues_details(app: &App) -> Vec<Line<'static>> {
    let (critical, high, medium) = app.get_risk_summary();
    vec![
        Line::from(vec![
            Span::styled("Security Issues Summary", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Critical Issues:  ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", critical), Style::default().fg(ERROR_COLOR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("High Risk:        ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", high), Style::default().fg(WARNING_COLOR).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Medium Risk:      ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(format!("{}", medium), Style::default().fg(WARNING_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Sources:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Security Profile"),
        ]),
        Line::from(vec![
            Span::raw("  • Hardening Profile"),
        ]),
        Line::from(vec![
            Span::raw("  • Compliance Profile"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_analytics_details(app: &App) -> Vec<Line<'static>> {
    let (critical, high, medium) = app.get_risk_summary();
    vec![
        Line::from(vec![
            Span::styled("Analytics & Charts", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Package Distribution:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  Total Packages:  ", Style::default().fg(TEXT_COLOR)),
            Span::styled(format!("{}", app.packages.package_count), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Service Status:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  Total Services:  ", Style::default().fg(TEXT_COLOR)),
            Span::styled(format!("{}", app.services.len()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("  Enabled:         ", Style::default().fg(TEXT_COLOR)),
            Span::styled(format!("{}", app.services.iter().filter(|s| s.enabled).count()), Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Security Score:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  Critical:        ", Style::default().fg(TEXT_COLOR)),
            Span::styled(format!("{}", critical), Style::default().fg(ERROR_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("  High:            ", Style::default().fg(TEXT_COLOR)),
            Span::styled(format!("{}", high), Style::default().fg(WARNING_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("  Medium:          ", Style::default().fg(TEXT_COLOR)),
            Span::styled(format!("{}", medium), Style::default().fg(INFO_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_timeline_details(app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("System Timeline", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Operating System: ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.os_name.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(vec![
            Span::styled("Kernel Version:   ", Style::default().fg(LIGHT_ORANGE)),
            Span::styled(app.kernel_version.clone(), Style::default().fg(TEXT_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Timeline Events:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::styled("  Total Events:    ", Style::default().fg(TEXT_COLOR)),
            Span::styled("Multiple", Style::default().fg(SUCCESS_COLOR)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("View timeline for chronological system events"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_logs_details(_app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("System Logs", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Log Categories:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Authentication Logs"),
        ]),
        Line::from(vec![
            Span::raw("  • System Logs"),
        ]),
        Line::from(vec![
            Span::raw("  • Kernel Logs"),
        ]),
        Line::from(vec![
            Span::raw("  • Security Logs"),
        ]),
        Line::from(vec![
            Span::raw("  • Application Logs"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Features:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Filter logs with /"),
        ]),
        Line::from(vec![
            Span::raw("  • Switch categories with Tab"),
        ]),
        Line::from(vec![
            Span::raw("  • Scroll with ↑↓"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_recommendations_details(_app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("Smart Recommendations", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("AI-Powered Analysis:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Security hardening recommendations"),
        ]),
        Line::from(vec![
            Span::raw("  • Performance optimization suggestions"),
        ]),
        Line::from(vec![
            Span::raw("  • Maintenance and compliance guidance"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Features:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Priority-based sorting (Critical → Info)"),
        ]),
        Line::from(vec![
            Span::raw("  • Impact and effort analysis"),
        ]),
        Line::from(vec![
            Span::raw("  • Step-by-step remediation guides"),
        ]),
        Line::from(vec![
            Span::raw("  • Quick wins identification"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn generate_topology_details(_app: &App) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("System Topology", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Architecture Layers:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Applications & Services layer"),
        ]),
        Line::from(vec![
            Span::raw("  • System Services layer"),
        ]),
        Line::from(vec![
            Span::raw("  • Operating System layer"),
        ]),
        Line::from(vec![
            Span::raw("  • Kernel layer"),
        ]),
        Line::from(vec![
            Span::raw("  • Hardware layer"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Network Topology:", Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD))
        ]),
        Line::from(vec![
            Span::raw("  • Internet connectivity flow"),
        ]),
        Line::from(vec![
            Span::raw("  • Firewall configuration"),
        ]),
        Line::from(vec![
            Span::raw("  • Network interface mapping"),
        ]),
        Line::from(vec![
            Span::raw("  • Service dependencies"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC or Enter to close", Style::default().fg(DARK_ORANGE).add_modifier(Modifier::ITALIC))
        ]),
    ]
}

fn draw_notification(f: &mut Frame, app: &App) {
    if let Some((ref message, _)) = app.notification {
        // Calculate notification position (top-right corner)
        let char_width = message.chars().count() as u16;
        let area = Rect {
            x: f.area().width.saturating_sub(char_width + 6),
            y: 1,
            width: char_width + 4,
            height: 3,
        };

        let notification_text = vec![
            Line::from(vec![
                Span::styled(message.clone(), Style::default().fg(TEXT_COLOR))
            ]),
        ];

        let notification = Paragraph::new(notification_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(Color::Black)))
            .style(Style::default().bg(Color::Black).fg(TEXT_COLOR))
            .alignment(Alignment::Center);

        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(notification, area);
    }
}

fn draw_jump_menu(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 60, f.area());

    // Get filtered views
    let filtered_views = app.get_filtered_views();

    // Create list items
    let items: Vec<ListItem> = filtered_views.iter().enumerate().map(|(idx, (view_idx, _view, highlighted))| {
        let is_selected = idx == app.jump_selected_index;

        // Parse highlighted string to create spans
        let mut spans = vec![];
        let mut current_text = String::new();
        let mut chars = highlighted.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '[' {
                // Push accumulated text
                if !current_text.is_empty() {
                    spans.push(Span::raw(current_text.clone()));
                    current_text.clear();
                }
                // Get highlighted char
                if let Some(highlighted_char) = chars.next() {
                    if chars.next() == Some(']') {
                        spans.push(Span::styled(
                            highlighted_char.to_string(),
                            Style::default().fg(ORANGE).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
                        ));
                    }
                }
            } else {
                current_text.push(c);
            }
        }
        if !current_text.is_empty() {
            spans.push(Span::raw(current_text));
        }

        let line = if is_selected {
            Line::from(vec![
                Span::styled(" ▶ ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", view_idx + 1), Style::default().fg(INFO_COLOR)),
            ].into_iter().chain(spans.into_iter().map(|s| {
                s.style(Style::default().add_modifier(Modifier::BOLD))
            })).collect::<Vec<_>>())
        } else {
            Line::from(vec![
                Span::raw("   "),
                Span::styled(format!("{} ", view_idx + 1), Style::default().fg(Color::DarkGray)),
            ].into_iter().chain(spans).collect::<Vec<_>>())
        };

        ListItem::new(line)
    }).collect();

    let list = List::new(items)
        .block(Block::default()
            .title(vec![
                Span::styled(" 🚀 Quick Jump ", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
                Span::styled(&app.jump_query, Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
            ])
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ORANGE)))
        .style(Style::default().bg(Color::Black).fg(TEXT_COLOR));

    // Help footer
    let help_text = vec![
        Span::styled("↑↓", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw(": Navigate  "),
        Span::styled("Enter", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw(": Select  "),
        Span::styled("Esc", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw(": Cancel  "),
        Span::styled("Type", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::raw(": Search"),
    ];

    let help = Paragraph::new(Line::from(help_text))
        .block(Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(DARK_ORANGE)))
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black).fg(TEXT_COLOR));

    // Split area for list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(list, chunks[0]);
    f.render_widget(help, chunks[1]);
}

fn draw_file_preview(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, f.area());

    // Prepare file content with line numbers
    let lines: Vec<Line> = app
        .file_preview_content
        .lines()
        .enumerate()
        .take(100) // Show first 100 lines
        .map(|(idx, line)| {
            Line::from(vec![
                Span::styled(
                    format!("{:4} │ ", idx + 1),
                    Style::default().fg(LIGHT_ORANGE),
                ),
                Span::styled(
                    if line.len() > 120 {
                        let truncated: String = line.chars().take(120).collect();
                        format!("{}...", truncated)
                    } else {
                        line.to_string()
                    },
                    Style::default().fg(TEXT_COLOR),
                ),
            ])
        })
        .collect();

    let total_lines = app.file_preview_content.lines().count();
    let showing_lines = lines.len();

    let title = format!(
        " 📄 File Preview: {} ({}/{} lines) ",
        app.file_preview_path, showing_lines, total_lines
    );

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(vec![Span::styled(
                    title,
                    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                )])
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(paragraph, area);

    // Footer with help
    let footer_area = Rect {
        x: area.x,
        y: area.y + area.height - 2,
        width: area.width,
        height: 2,
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(TEXT_COLOR)),
        Span::styled("ESC", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(TEXT_COLOR)),
        Span::styled("q", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" to close", Style::default().fg(TEXT_COLOR)),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(Color::Black));

    f.render_widget(footer, footer_area);
}

fn draw_file_info(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 40, f.area());

    let lines: Vec<Line> = app
        .file_info_content
        .lines()
        .map(|line| {
            if let Some((key, value)) = line.split_once(": ") {
                Line::from(vec![
                    Span::styled(format!("{}: ", key), Style::default().fg(LIGHT_ORANGE).add_modifier(Modifier::BOLD)),
                    Span::styled(value, Style::default().fg(TEXT_COLOR)),
                ])
            } else {
                Line::from(Span::styled(line, Style::default().fg(TEXT_COLOR)))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(vec![Span::styled(
                    " ℹ️  File Information ",
                    Style::default().fg(ORANGE).add_modifier(Modifier::BOLD),
                )])
                .borders(Borders::ALL)
                .border_style(Style::default().fg(ORANGE)),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(ratatui::widgets::Clear, area);
    f.render_widget(paragraph, area);

    // Footer with help
    let footer_area = Rect {
        x: area.x,
        y: area.y + area.height - 2,
        width: area.width,
        height: 2,
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(TEXT_COLOR)),
        Span::styled("ESC", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(TEXT_COLOR)),
        Span::styled("q", Style::default().fg(ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled(" to close", Style::default().fg(TEXT_COLOR)),
    ]))
    .alignment(Alignment::Center)
    .style(Style::default().bg(Color::Black));

    f.render_widget(footer, footer_area);
}

fn context_help_for_view(view: View) -> Vec<&'static str> {
    match view {
        View::Dashboard => vec![
            "Dashboard — system overview and health meter.",
            "r refresh view • Shift+R full re-inspect • Tab next view",
            "[ ] cycle layout • : command palette • p profiles",
        ],
        View::Issues => vec![
            "Issues — security findings from profiles.",
            "f cycle risk filter • Enter detail • / search",
            "e export report • Ctrl+M migration JSON bundle",
        ],
        View::Files => vec![
            "Files — browse guest filesystem.",
            "Enter open dir • v preview • i file info • . hidden toggle",
            "/ filter path • Backspace parent dir",
        ],
        View::Packages => vec![
            "Packages — installed packages.",
            "s sort • c compare mode • m multi-select",
        ],
        _ => vec![
            "j/k or arrows scroll • Tab switch view • q quit",
            ": command palette • ? context help • h full help",
        ],
    }
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
