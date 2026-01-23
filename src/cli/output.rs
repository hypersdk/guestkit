// SPDX-License-Identifier: LGPL-3.0-or-later
//! Output formatting utilities for CLI

use serde::Serialize;
use std::fmt;

/// Output format options
#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Human,
    Json,
    Yaml,
}

impl OutputFormat {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "yaml" => OutputFormat::Yaml,
            _ => OutputFormat::Human,
        }
    }
}

/// Format output based on format type
pub fn format_output<T: Serialize>(data: &T, format: OutputFormat) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
        OutputFormat::Human => Ok(format!("{:#?}", data)),
    }
}

/// Pretty print size in human readable format
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format duration in human readable format
pub fn format_duration(secs: f64) -> String {
    if secs >= 60.0 {
        let mins = (secs / 60.0).floor();
        let remaining_secs = secs % 60.0;
        format!("{}m {:.2}s", mins, remaining_secs)
    } else {
        format!("{:.2}s", secs)
    }
}

/// Table formatter for aligned output
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn print(&self) {
        if self.headers.is_empty() {
            return;
        }

        // Calculate column widths
        let mut widths: Vec<usize> = self.headers.iter().map(|h| h.len()).collect();

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // Print header
        for (i, header) in self.headers.iter().enumerate() {
            print!("{:width$}  ", header, width = widths[i]);
        }
        println!();

        // Print separator
        for width in &widths {
            print!("{}  ", "-".repeat(*width));
        }
        println!();

        // Print rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    print!("{:width$}  ", cell, width = widths[i]);
                }
            }
            println!();
        }
    }
}

/// Progress indicator for long operations
pub struct ProgressBar {
    total: u64,
    current: u64,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: 0,
            width: 50,
        }
    }

    pub fn update(&mut self, current: u64) {
        self.current = current;
        self.draw();
    }

    pub fn finish(&mut self) {
        self.current = self.total;
        self.draw();
        println!();
    }

    fn draw(&self) {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u8
        } else {
            0
        };

        let filled = if self.total > 0 {
            ((self.current as f64 / self.total as f64) * self.width as f64) as usize
        } else {
            0
        };

        let empty = self.width - filled;

        print!("\r[{}{}] {}%",
               "=".repeat(filled),
               " ".repeat(empty),
               percentage);

        use std::io::Write;
        std::io::stdout().flush().ok();
    }
}

impl fmt::Display for ProgressBar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Progress: {}/{}", self.current, self.total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30.5), "30.50s");
        assert_eq!(format_duration(90.0), "1m 30.00s");
        assert_eq!(format_duration(150.75), "2m 30.75s");
    }
}
