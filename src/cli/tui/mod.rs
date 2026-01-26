// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI (Terminal User Interface) module for interactive VM inspection

pub mod app;
pub mod events;
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

    // Setup terminal
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

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
            if let Event::Key(key) = event::read()? {
                match key.code {
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
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}
