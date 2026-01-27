// SPDX-License-Identifier: LGPL-3.0-or-later
//! TUI Configuration system
//!
//! Loads and saves user preferences from ~/.config/guestkit/tui.toml

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// TUI Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// UI settings
    pub ui: UiConfig,

    /// Behavior settings
    pub behavior: BehaviorConfig,

    /// Keybindings (future: allow custom bindings)
    #[serde(default)]
    pub keybindings: KeybindingsConfig,
}

/// UI appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show splash screen on startup
    #[serde(default = "default_true")]
    pub show_splash: bool,

    /// Splash duration in milliseconds
    #[serde(default = "default_splash_duration")]
    pub splash_duration_ms: u64,

    /// Show stats bar at startup
    #[serde(default = "default_true")]
    pub show_stats_bar: bool,

    /// Color theme (currently only "default" supported)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Enable mouse support
    #[serde(default = "default_true")]
    pub mouse_enabled: bool,
}

/// Behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Default view on startup
    #[serde(default = "default_view")]
    pub default_view: String,

    /// Auto-refresh interval in seconds (0 = disabled)
    #[serde(default)]
    pub auto_refresh_seconds: u64,

    /// Search case-sensitive by default
    #[serde(default)]
    pub search_case_sensitive: bool,

    /// Search regex mode by default
    #[serde(default)]
    pub search_regex_mode: bool,

    /// Maximum bookmarks
    #[serde(default = "default_max_bookmarks")]
    pub max_bookmarks: usize,

    /// Scroll amount for page up/down
    #[serde(default = "default_page_scroll")]
    pub page_scroll_lines: usize,
}

/// Keybindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Enable vim-style keybindings
    #[serde(default = "default_true")]
    pub vim_mode: bool,

    /// Enable Ctrl+P quick jump menu
    #[serde(default = "default_true")]
    pub quick_jump_enabled: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            behavior: BehaviorConfig::default(),
            keybindings: KeybindingsConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_splash: true,
            splash_duration_ms: 800,
            show_stats_bar: true,
            theme: "default".to_string(),
            mouse_enabled: true,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            default_view: "dashboard".to_string(),
            auto_refresh_seconds: 0,
            search_case_sensitive: false,
            search_regex_mode: false,
            max_bookmarks: 20,
            page_scroll_lines: 10,
        }
    }
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            vim_mode: true,
            quick_jump_enabled: true,
        }
    }
}

// Default value functions for serde
fn default_true() -> bool { true }
fn default_splash_duration() -> u64 { 800 }
fn default_theme() -> String { "default".to_string() }
fn default_view() -> String { "dashboard".to_string() }
fn default_max_bookmarks() -> usize { 20 }
fn default_page_scroll() -> usize { 10 }

impl TuiConfig {
    /// Get the default config file path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not determine config directory")?;

        Ok(config_dir.join("guestkit").join("tui.toml"))
    }

    /// Load configuration from default path, or return default config
    pub fn load() -> Self {
        match Self::load_from_file() {
            Ok(config) => config,
            Err(_) => {
                // Return default config if file doesn't exist or can't be read
                Self::default()
            }
        }
    }

    /// Load configuration from file
    fn load_from_file() -> Result<Self> {
        let path = Self::default_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .context("Failed to read config file")?;

        let config: TuiConfig = toml::from_str(&contents)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to default path
    #[allow(dead_code)]
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&path, contents)
            .context("Failed to write config file")?;

        Ok(())
    }

    /// Create a default config file if it doesn't exist
    #[allow(dead_code)]
    pub fn init() -> Result<PathBuf> {
        let path = Self::default_path()?;

        if path.exists() {
            return Ok(path);
        }

        let config = Self::default();
        config.save()?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TuiConfig::default();
        assert!(config.ui.show_splash);
        assert!(config.ui.mouse_enabled);
        assert_eq!(config.ui.splash_duration_ms, 800);
        assert_eq!(config.behavior.max_bookmarks, 20);
        assert!(config.keybindings.vim_mode);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = TuiConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: TuiConfig = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.ui.show_splash, deserialized.ui.show_splash);
        assert_eq!(config.behavior.default_view, deserialized.behavior.default_view);
    }
}
