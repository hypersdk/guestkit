// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI (Terminal User Interface) module for interactive VM inspection

pub mod app;
pub mod config;
pub mod events;
pub mod splash;
pub mod ui;
pub mod views;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

pub use app::App;

/// Run the TUI application
pub fn run_tui<P: AsRef<Path>>(image_path: P) -> Result<()> {
    // Load configuration first
    let config = config::TuiConfig::load();

    // Setup terminal first for splash screen
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();

    // Enable mouse capture based on config
    if config.ui.mouse_enabled {
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to enter alternate screen")?;
    } else {
        execute!(stdout, EnterAlternateScreen)
            .context("Failed to enter alternate screen")?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Show splash screen if enabled
    if config.ui.show_splash {
        terminal.draw(|f| splash::draw_splash(f))?;
        std::thread::sleep(Duration::from_millis(config.ui.splash_duration_ms));
    }

    // Show loading spinner during inspection with coral-terracotta orange theme
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.rgb(222,115,86)} {msg:.rgb(222,115,86)}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    spinner.set_message("🔍 Inspecting disk image and analyzing system...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    // Create app state (this is the slow part)
    let app = App::new(image_path.as_ref());

    spinner.finish_and_clear();

    let mut app = app?;

    // Run the event loop
    let result = run_app(&mut terminal, &mut app);

    // Cleanup guestfs handle
    let _ = app.cleanup();

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;

    // Disable mouse capture if it was enabled
    if config.ui.mouse_enabled {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .context("Failed to leave alternate screen")?;
    } else {
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )
        .context("Failed to leave alternate screen")?;
    }

    terminal.show_cursor().context("Failed to show cursor")?;

    result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Mouse(_) => {
                    // Mouse support disabled
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        // Close file preview/info overlays first
                        if app.show_file_preview {
                            app.close_file_preview();
                        } else if app.show_file_info {
                            app.close_file_info();
                        } else if app.file_filtering {
                            // Cancel file filter and clear it
                            app.cancel_file_filter();
                        } else if app.show_jump_menu {
                            app.toggle_jump_menu();
                        } else if app.is_searching() {
                            app.cancel_search();
                        } else if app.is_exporting() {
                            app.cancel_export();
                        } else if app.show_export_menu {
                            app.toggle_export_menu();
                        } else if app.show_help {
                            app.toggle_help();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        app.toggle_jump_menu();
                    }
                    KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) && app.is_searching() => {
                        app.toggle_case_sensitive();
                    }
                    KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) && app.is_searching() => {
                        app.toggle_regex_mode();
                    }
                    KeyCode::Tab => app.next_view(),
                    KeyCode::BackTab => app.previous_view(),
                    KeyCode::Char('h') | KeyCode::F(1) => app.toggle_help(),
                    KeyCode::Char('p') => {
                        app.current_view = app::View::Profiles;
                        app.scroll_offset = 0;
                    }
                    KeyCode::Char('e') => app.toggle_export_menu(),
                    KeyCode::Char('s') => app.cycle_sort_mode(),
                    KeyCode::Char('v') if app.current_view == app::View::Files && !app.is_searching() => {
                        // View file preview in Files view
                        app.show_file_preview();
                    }
                    KeyCode::Char('i') if app.current_view == app::View::Files && !app.is_searching() => {
                        // Show file info in Files view
                        app.show_file_information();
                    }
                    KeyCode::Char('i') => app.toggle_stats_bar(),
                    KeyCode::Char('t') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => {
                        app.toggle_table_mode();
                    }
                    KeyCode::Char('c') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => {
                        app.toggle_comparison_mode();
                    }
                    KeyCode::Char('m') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => {
                        app.toggle_multi_select();
                    }
                    KeyCode::Char('f') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => {
                        app.cycle_filter();
                    }
                    KeyCode::Char('l') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => {
                        app.toggle_live_filter();
                    }
                    KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) && !app.is_searching() => {
                        app.select_all_items();
                    }
                    KeyCode::Char(' ') if app.multi_select_mode && !app.is_searching() => {
                        app.toggle_item_selection();
                    }
                    KeyCode::Char('r') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => {
                        // Trigger refresh
                        app.start_refresh();
                        // Note: In a real implementation, this would spawn a background task
                        // For now, just update the timestamp
                        app.complete_refresh();
                    }
                    KeyCode::Char('b') => {
                        // Bookmark current view
                        let bookmark = format!("{} view", app.current_view.title());
                        app.add_bookmark(bookmark);
                    }
                    KeyCode::Char('/') => {
                        if app.current_view == app::View::Files && !app.is_searching() {
                            // Start file filter in Files view
                            app.start_file_filter();
                        } else {
                            // Start search in other views
                            app.start_search();
                        }
                    }
                    KeyCode::Left => {
                        if app.current_view == app::View::Profiles {
                            app.previous_profile_tab();
                        }
                    }
                    KeyCode::Right => {
                        if app.current_view == app::View::Profiles {
                            app.next_profile_tab();
                        }
                    }
                    KeyCode::Up => {
                        if app.show_jump_menu {
                            app.jump_menu_previous();
                        } else {
                            app.scroll_up();
                        }
                    }
                    KeyCode::Down => {
                        if app.show_jump_menu {
                            app.jump_menu_next();
                        } else {
                            app.scroll_down();
                        }
                    }
                    KeyCode::PageUp => app.page_up(),
                    KeyCode::PageDown => app.page_down(),
                    KeyCode::Home => app.scroll_top(),
                    KeyCode::End => app.scroll_bottom(),
                    // Vim-style navigation
                    KeyCode::Char('k') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => app.scroll_up(),
                    KeyCode::Char('j') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => app.scroll_down(),
                    KeyCode::Char('g') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => app.scroll_top(),
                    KeyCode::Char('G') if !app.is_searching() && !matches!(app.export_mode, Some(app::ExportMode::EnteringFilename)) => app.scroll_bottom(),
                    KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) && !app.is_searching() => app.page_up(),
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) && !app.is_searching() => app.page_down(),
                    KeyCode::Enter => {
                        use app::ExportMode;

                        if app.show_jump_menu {
                            app.jump_menu_select();
                        } else if matches!(app.export_mode, Some(ExportMode::EnteringFilename)) {
                            let _ = app.execute_export();
                        } else if app.file_filtering {
                            // Finish file filter
                            app.finish_file_filter();
                        } else if app.current_view == app::View::Files && !app.is_searching() {
                            // Enter directory in Files view
                            app.file_browser_enter();
                        } else if !app.is_searching() && !app.show_export_menu {
                            app.toggle_detail();
                        } else {
                            app.select_item();
                        }
                    }
                    KeyCode::Char(c) => {
                        use app::{ExportFormat, ExportMode};

                        if app.show_jump_menu {
                            app.jump_menu_input(c);
                        } else if matches!(app.export_mode, Some(ExportMode::Selecting)) {
                            // Handle format selection
                            match c {
                                '1' => app.select_export_format(ExportFormat::Json),
                                '2' => app.select_export_format(ExportFormat::Yaml),
                                '3' => app.select_export_format(ExportFormat::Html),
                                '4' => app.select_export_format(ExportFormat::Pdf),
                                _ => {}
                            }
                        } else if matches!(app.export_mode, Some(ExportMode::EnteringFilename)) {
                            app.export_input(c);
                        } else if app.file_filtering {
                            // Add character to file filter
                            app.file_filter_input_char(c);
                        } else if app.is_searching() {
                            app.search_input(c);
                        } else if app.current_view == app::View::Files && c == '.' {
                            // Toggle hidden files in Files view
                            app.file_browser_toggle_hidden();
                        } else if c.is_ascii_digit() {
                            // Quick jump to views with number keys 1-9
                            if let Some(digit) = c.to_digit(10) {
                                if digit > 0 {
                                    app.jump_to_view((digit - 1) as usize);
                                }
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        use app::ExportMode;

                        if app.show_jump_menu {
                            app.jump_menu_backspace();
                        } else if matches!(app.export_mode, Some(ExportMode::EnteringFilename)) {
                            app.export_backspace();
                        } else if app.file_filtering {
                            // Remove character from file filter
                            app.file_filter_backspace();
                        } else if app.is_searching() {
                            app.search_backspace();
                        } else if app.current_view == app::View::Files {
                            // Go to parent directory in Files view
                            app.file_browser_go_up();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}
