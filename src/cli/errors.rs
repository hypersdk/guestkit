// SPDX-License-Identifier: LGPL-3.0-or-later
//! Enhanced error messages with suggestions

use owo_colors::OwoColorize;
use std::fmt;

/// Enhanced error with helpful suggestions
pub struct EnhancedError {
    pub message: String,
    pub suggestion: Option<String>,
    pub examples: Vec<String>,
}

impl EnhancedError {
    /// Create a new enhanced error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            suggestion: None,
            examples: Vec::new(),
        }
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Add an example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Add multiple examples
    pub fn with_examples(mut self, examples: Vec<String>) -> Self {
        self.examples = examples;
        self
    }

    /// Display the error with formatting
    pub fn display(&self) {
        // Error message
        eprintln!("{} {}", "Error:".red().bold(), self.message.red());

        // Suggestion
        if let Some(ref suggestion) = self.suggestion {
            eprintln!();
            eprintln!("{} {}", "Suggestion:".yellow().bold(), suggestion.yellow());
        }

        // Examples
        if !self.examples.is_empty() {
            eprintln!();
            eprintln!("{}", "Examples:".cyan().bold());
            for example in &self.examples {
                eprintln!("  {}", example.dimmed());
            }
        }
    }
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EnhancedError {}

/// Common error builders
pub mod errors {
    use super::*;

    /// Invalid command usage
    pub fn invalid_usage(command: &str, usage: &str) -> EnhancedError {
        EnhancedError::new(format!("Invalid usage of '{}'", command))
            .with_suggestion(format!("Usage: {}", usage))
    }

    /// Unknown command
    pub fn unknown_command(command: &str, available: &[&str]) -> EnhancedError {
        let mut err = EnhancedError::new(format!("Unknown command: '{}'", command))
            .with_suggestion("Type 'help' to see all available commands");

        // Find similar commands (simple prefix matching)
        let similar: Vec<String> = available
            .iter()
            .filter(|&&cmd| {
                command.starts_with(&cmd[..1.min(cmd.len())])
                    || cmd.starts_with(&command[..1.min(command.len())])
            })
            .take(3)
            .map(|&s| s.to_string())
            .collect();

        if !similar.is_empty() {
            err = err.with_suggestion(format!("Did you mean: {}?", similar.join(", ")));
        }

        err
    }

    /// File not found
    #[allow(dead_code)]
    pub fn file_not_found(path: &str) -> EnhancedError {
        EnhancedError::new(format!("File not found: {}", path))
            .with_suggestion("Check that the file path is correct and the file exists")
            .with_example("ls /path/to/verify")
    }

    /// Mount required
    #[allow(dead_code)]
    pub fn mount_required() -> EnhancedError {
        EnhancedError::new("No filesystem is mounted")
            .with_suggestion("Mount a filesystem first before accessing files")
            .with_examples(vec![
                "mount /dev/sda1 /".to_string(),
                "mount /dev/vda1 /".to_string(),
            ])
    }

    /// OS detection failed
    pub fn os_detection_failed() -> EnhancedError {
        EnhancedError::new("No operating system detected in the disk image")
            .with_suggestion(
                "This command requires OS detection. The disk may be empty or corrupted",
            )
            .with_examples(vec![
                "# Try mounting manually:".to_string(),
                "filesystems".to_string(),
                "mount /dev/sda1 /".to_string(),
            ])
    }

    /// Permission denied
    #[allow(dead_code)]
    pub fn permission_denied(operation: &str) -> EnhancedError {
        EnhancedError::new(format!("Permission denied: {}", operation))
            .with_suggestion("Try running with elevated privileges or check file permissions")
            .with_example("sudo guestkit ...")
    }

    /// Disk image not found
    #[allow(dead_code)]
    pub fn disk_not_found(path: &str) -> EnhancedError {
        EnhancedError::new(format!("Disk image not found: {}", path))
            .with_suggestion("Verify the disk image path exists and is accessible")
            .with_examples(vec![
                "# Check if file exists:".to_string(),
                format!("ls -lh {}", path),
                "# Common locations:".to_string(),
                "ls /var/lib/libvirt/images/".to_string(),
                "ls ~/VirtualBox\\ VMs/".to_string(),
            ])
    }

    /// Invalid disk format
    #[allow(dead_code)]
    pub fn invalid_format(path: &str) -> EnhancedError {
        EnhancedError::new(format!("Unable to recognize disk image format: {}", path))
            .with_suggestion("Supported formats: qcow2, raw, vmdk, vhd, vdi")
            .with_examples(vec![
                "# Check file format:".to_string(),
                "file disk.qcow2".to_string(),
                "qemu-img info disk.qcow2".to_string(),
            ])
    }

    /// Cache error
    #[allow(dead_code)]
    pub fn cache_error(message: &str) -> EnhancedError {
        EnhancedError::new(format!("Cache error: {}", message))
            .with_suggestion("Try clearing the cache or running without --cache")
            .with_examples(vec![
                "guestkit cache-clear".to_string(),
                "guestkit inspect vm.qcow2  # without --cache".to_string(),
            ])
    }

    /// Export error
    #[allow(dead_code)]
    pub fn export_error(format: &str, message: &str) -> EnhancedError {
        EnhancedError::new(format!("Export to {} failed: {}", format, message))
            .with_suggestion("Check that the output path is writable".to_string())
            .with_examples(vec![
                "# Verify output directory:".to_string(),
                "ls -ld $(dirname output.html)".to_string(),
                "# Try different output location:".to_string(),
                "guestkit inspect vm.qcow2 --export html --export-output ~/report.html".to_string(),
            ])
    }

    /// Network error
    #[allow(dead_code)]
    pub fn network_error(message: &str) -> EnhancedError {
        EnhancedError::new(format!("Network error: {}", message))
            .with_suggestion("Check your internet connection or try again later")
    }

    /// Timeout error
    #[allow(dead_code)]
    pub fn timeout_error(operation: &str) -> EnhancedError {
        EnhancedError::new(format!("Operation timed out: {}", operation)).with_suggestion(
            "The operation took too long. Try with a smaller disk or increase timeout",
        )
    }

    /// Insufficient space
    #[allow(dead_code)]
    pub fn insufficient_space(required: &str) -> EnhancedError {
        EnhancedError::new("Insufficient disk space")
            .with_suggestion(format!("At least {} of free space is required", required))
            .with_examples(vec![
                "# Check available space:".to_string(),
                "df -h /tmp".to_string(),
                "df -h ~".to_string(),
            ])
    }

    /// Dependency missing
    #[allow(dead_code)]
    pub fn dependency_missing(dependency: &str) -> EnhancedError {
        EnhancedError::new(format!("Required dependency not found: {}", dependency))
            .with_suggestion(format!("Install {} to use this feature", dependency))
            .with_examples(vec![
                format!("# On Ubuntu/Debian:"),
                format!("sudo apt-get install {}", dependency),
                format!("# On Fedora/RHEL:"),
                format!("sudo dnf install {}", dependency),
            ])
    }

    /// Invalid argument
    #[allow(dead_code)]
    pub fn invalid_argument(arg: &str, expected: &str) -> EnhancedError {
        EnhancedError::new(format!("Invalid argument: {}", arg))
            .with_suggestion(format!("Expected: {}", expected))
    }

    /// Feature not available
    #[allow(dead_code)]
    pub fn feature_not_available(feature: &str, reason: &str) -> EnhancedError {
        EnhancedError::new(format!("Feature not available: {}", feature))
            .with_suggestion(reason.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_error() {
        let err = EnhancedError::new("Test error")
            .with_suggestion("Try this instead")
            .with_example("example command");

        assert_eq!(err.message, "Test error");
        assert_eq!(err.suggestion, Some("Try this instead".to_string()));
        assert_eq!(err.examples.len(), 1);
    }

    #[test]
    fn test_unknown_command() {
        let err = errors::unknown_command("pac", &["packages", "pkg", "services"]);
        assert!(err.message.contains("Unknown command"));
    }
}
