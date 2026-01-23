// SPDX-License-Identifier: LGPL-3.0-or-later
//! Progress reporting for long-running operations

use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::sync::Arc;
use std::time::Duration;

/// Progress reporter for disk operations
pub struct ProgressReporter {
    bar: Arc<ProgressBar>,
    multi: Option<Arc<MultiProgress>>,
}

impl ProgressReporter {
    /// Create a new progress bar with total size
    pub fn new(total: u64, message: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            bar: Arc::new(bar),
            multi: None,
        }
    }

    /// Create a spinner for operations without known size
    pub fn spinner(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));

        Self {
            bar: Arc::new(bar),
            multi: None,
        }
    }

    /// Update progress to specific position
    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    /// Increment progress by delta
    pub fn inc(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// Update the message
    pub fn set_message(&self, message: impl Into<String>) {
        self.bar.set_message(message.into());
    }

    /// Finish with a success message
    pub fn finish_with_message(&self, message: impl Into<String>) {
        self.bar.finish_with_message(message.into());
    }

    /// Finish and clear the progress bar
    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }

    /// Abandon the progress bar (error case)
    pub fn abandon_with_message(&self, message: impl Into<String>) {
        self.bar.abandon_with_message(message.into());
    }

    /// Get a clone of the underlying progress bar
    pub fn clone_bar(&self) -> Arc<ProgressBar> {
        self.bar.clone()
    }
}

/// Progress tracker for multiple operations
pub struct MultiProgressReporter {
    multi: Arc<MultiProgress>,
}

impl MultiProgressReporter {
    /// Create a new multi-progress tracker
    pub fn new() -> Self {
        Self {
            multi: Arc::new(MultiProgress::new()),
        }
    }

    /// Add a new progress bar
    pub fn add_bar(&self, total: u64, message: &str) -> ProgressReporter {
        let bar = self.multi.add(ProgressBar::new(total));
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(100));

        ProgressReporter {
            bar: Arc::new(bar),
            multi: Some(self.multi.clone()),
        }
    }

    /// Add a new spinner
    pub fn add_spinner(&self, message: &str) -> ProgressReporter {
        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        bar.set_message(message.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));

        ProgressReporter {
            bar: Arc::new(bar),
            multi: Some(self.multi.clone()),
        }
    }
}

impl Default for MultiProgressReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_progress_reporter() {
        let progress = ProgressReporter::new(100, "Testing");
        progress.set_position(50);
        assert_eq!(progress.bar.position(), 50);
        progress.finish_and_clear();
    }

    #[test]
    fn test_spinner() {
        let spinner = ProgressReporter::spinner("Loading...");
        thread::sleep(Duration::from_millis(200));
        spinner.finish_with_message("Done!");
    }

    #[test]
    fn test_multi_progress() {
        let multi = MultiProgressReporter::new();
        let bar1 = multi.add_bar(100, "Task 1");
        let bar2 = multi.add_bar(100, "Task 2");

        bar1.set_position(50);
        bar2.set_position(75);

        bar1.finish_with_message("Task 1 complete");
        bar2.finish_with_message("Task 2 complete");
    }
}
