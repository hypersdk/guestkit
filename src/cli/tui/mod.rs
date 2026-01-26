// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI (Terminal User Interface) module for interactive VM inspection

pub mod app;
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
    // Setup terminal first for splash screen
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

    // Show splash screen
    terminal.draw(|f| splash::draw_splash(f))?;

    // Small delay to show splash
    std::thread::sleep(Duration::from_millis(800));

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

    // Restore terminal
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
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
                Event::Mouse(mouse) => {
                    use event::MouseEventKind;
                    match mouse.kind {
                        MouseEventKind::ScrollDown => app.scroll_down(),
                        MouseEventKind::ScrollUp => app.scroll_up(),
                        MouseEventKind::Down(event::MouseButton::Left) => {
                            // Handle tab clicks - tabs are in row 4-6 (0-indexed row 3-5)
                            if mouse.row >= 4 && mouse.row <= 6 {
                                // Calculate which tab was clicked based on column
                                // Each tab is approximately width/12 (12 views)
                                // This is a rough approximation
                                let tab_index = ((mouse.column as f32 / terminal.size()?.width as f32) * 12.0) as usize;
                                app.jump_to_view(tab_index);
                            }
                        }
                        _ => {}
                    }
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.is_searching() {
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
                    KeyCode::Char('i') => app.toggle_stats_bar(),
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
                    KeyCode::Char('/') => app.start_search(),
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
                    KeyCode::Up => app.scroll_up(),
                    KeyCode::Down => app.scroll_down(),
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

                        if matches!(app.export_mode, Some(ExportMode::EnteringFilename)) {
                            let _ = app.execute_export();
                        } else if !app.is_searching() && !app.show_export_menu {
                            app.toggle_detail();
                        } else {
                            app.select_item();
                        }
                    }
                    KeyCode::Char(c) => {
                        use app::{ExportFormat, ExportMode};

                        if matches!(app.export_mode, Some(ExportMode::Selecting)) {
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
                        } else if app.is_searching() {
                            app.search_input(c);
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

                        if matches!(app.export_mode, Some(ExportMode::EnteringFilename)) {
                            app.export_backspace();
                        } else if app.is_searching() {
                            app.search_backspace();
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
