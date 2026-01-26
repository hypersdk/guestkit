// SPDX-License-Identifier: LGPL-3.0-or-later
//! Output formatting utilities for CLI

use owo_colors::OwoColorize;
use serde::Serialize;
use std::fmt;

/// Output format options
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum OutputFormat {
    Human,
    Json,
    Yaml,
}

impl OutputFormat {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "yaml" => OutputFormat::Yaml,
            _ => OutputFormat::Human,
        }
    }
}

/// Format output based on format type
#[allow(dead_code)]
pub fn format_output<T: Serialize + std::fmt::Debug>(
    data: &T,
    format: OutputFormat,
) -> Result<String, Box<dyn std::error::Error>> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(data)?),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(data)?),
        OutputFormat::Human => Ok(format!("{:#?}", data)),
    }
}

/// Pretty print size in human readable format
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub struct Table {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl Table {
    #[allow(dead_code)]
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    #[allow(dead_code)]
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
#[allow(dead_code)]
pub struct ProgressBar {
    total: u64,
    current: u64,
    width: usize,
}

impl ProgressBar {
    #[allow(dead_code)]
    pub fn new(total: u64) -> Self {
        Self {
            total,
            current: 0,
            width: 50,
        }
    }

    #[allow(dead_code)]
    pub fn update(&mut self, current: u64) {
        self.current = current;
        self.draw();
    }

    #[allow(dead_code)]
    pub fn finish(&mut self) {
        self.current = self.total;
        self.draw();
        println!();
    }

    #[allow(dead_code)]
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

        print!(
            "\r[{}{}] {}%",
            "=".repeat(filled),
            " ".repeat(empty),
            percentage
        );

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

/// Colorized output helpers
#[allow(dead_code)]
pub mod colors {
    use super::*;

    // Coral-Terracotta Orange theme - Pantone 7416 C inspired
    const ORANGE_RGB: (u8, u8, u8) = (222, 115, 86);        // Primary coral orange
    const LIGHT_ORANGE_RGB: (u8, u8, u8) = (255, 145, 115); // Lighter coral
    const DARK_ORANGE_RGB: (u8, u8, u8) = (180, 85, 60);    // Darker terracotta

    /// Print success message with green checkmark
    pub fn success(msg: &str) {
        println!("{} {}", "✓".green(), msg.green());
    }

    /// Print error message with red X
    pub fn error(msg: &str) {
        eprintln!("{} {}", "✗".red(), msg.red());
    }

    /// Print warning message with yellow triangle
    pub fn warning(msg: &str) {
        println!("{} {}", "⚠".yellow(), msg.yellow());
    }

    /// Print info message with coral orange info icon
    pub fn info(msg: &str) {
        println!("{} {}", "ℹ".truecolor(ORANGE_RGB.0, ORANGE_RGB.1, ORANGE_RGB.2), msg);
    }

    /// Print header with bold and underline in coral orange
    pub fn header(msg: &str) {
        println!("{}", msg.truecolor(ORANGE_RGB.0, ORANGE_RGB.1, ORANGE_RGB.2).bold().underline());
    }

    /// Print section header with coral orange color
    pub fn section(msg: &str) {
        println!("\n{}", msg.truecolor(ORANGE_RGB.0, ORANGE_RGB.1, ORANGE_RGB.2).bold());
    }

    /// Print key-value pair with coral orange key
    pub fn kv(key: &str, value: &str) {
        println!("{}: {}", key.truecolor(LIGHT_ORANGE_RGB.0, LIGHT_ORANGE_RGB.1, LIGHT_ORANGE_RGB.2), value);
    }

    /// Print key-value with optional value coloring
    pub fn kv_colored(key: &str, value: &str, color: Color) {
        let colored_value = match color {
            Color::Green => value.green().to_string(),
            Color::Red => value.red().to_string(),
            Color::Yellow => value.yellow().to_string(),
            Color::Blue => value.blue().to_string(),
            Color::Cyan => value.cyan().to_string(),
            Color::Magenta => value.magenta().to_string(),
            Color::White => value.white().to_string(),
        };
        println!("{}: {}", key.cyan(), colored_value);
    }

    /// Print dimmed/muted text
    pub fn dimmed(msg: &str) {
        println!("{}", msg.dimmed());
    }

    /// Print emphasized/bright text
    pub fn emphasis(msg: &str) {
        println!("{}", msg.bright_white().bold());
    }

    /// Color enum for output
    #[derive(Debug, Clone, Copy)]
    pub enum Color {
        Green,
        Red,
        Yellow,
        Blue,
        Cyan,
        Magenta,
        White,
    }

    /// Status indicator with colored icon
    pub fn status(label: &str, status: Status) {
        let label_colored = label.truecolor(LIGHT_ORANGE_RGB.0, LIGHT_ORANGE_RGB.1, LIGHT_ORANGE_RGB.2);
        match status {
            Status::Enabled => println!("{} {}: {}", "✓".bold(), label_colored, "enabled".green()),
            Status::Disabled => println!("{} {}: {}", "✗".bold(), label_colored, "disabled".red()),
            Status::Unknown => println!("{} {}: {}", "?".bold(), label_colored, "unknown".yellow()),
            Status::Running => println!("{} {}: {}", "▶".bold(), label_colored, "running".green()),
            Status::Stopped => println!("{} {}: {}", "■".bold(), label_colored, "stopped".red()),
            Status::Warning => println!("{} {}: {}", "⚠".bold(), label_colored, "warning".yellow()),
        }
    }

    /// Status types
    #[derive(Debug, Clone, Copy)]
    pub enum Status {
        Enabled,
        Disabled,
        Unknown,
        Running,
        Stopped,
        Warning,
    }

    /// Print a separator line
    pub fn separator() {
        println!("{}", "─".repeat(80).dimmed());
    }

    /// Print a thick separator
    pub fn thick_separator() {
        println!("{}", "═".repeat(80).bold());
    }

    /// Print bullet point item
    pub fn bullet(msg: &str) {
        println!("  {} {}", "•".truecolor(ORANGE_RGB.0, ORANGE_RGB.1, ORANGE_RGB.2), msg);
    }

    /// Print numbered item
    pub fn numbered(num: usize, msg: &str) {
        println!("  {}. {}", num.to_string().truecolor(ORANGE_RGB.0, ORANGE_RGB.1, ORANGE_RGB.2).bold(), msg);
    }

    /// Print progress indicator
    pub fn progress(current: usize, total: usize, msg: &str) {
        println!("{} {}", format!("[{}/{}]", current, total).truecolor(ORANGE_RGB.0, ORANGE_RGB.1, ORANGE_RGB.2), msg);
    }
}

#[cfg(test)]
mod color_tests {
    use super::colors::*;

    #[test]
    fn test_colors_compile() {
        // Just ensure functions compile and can be called
        success("test");
        error("test");
        warning("test");
        info("test");
        header("test");
        section("test");
        kv("key", "value");
        dimmed("test");
        emphasis("test");
        separator();
        bullet("test");
        numbered(1, "test");
        progress(1, 10, "test");
    }
}
