# TUI Enhancements - January 2026

## Overview

The GuestKit TUI has been enhanced with several new features to improve usability and user experience.

## New Features

### 1. Splash Screen Integration ✨
- Beautiful ASCII art logo displayed on startup
- Shows "GuestKit" branding
- 800ms display duration before loading
- Smooth transition to main UI

### 2. Vim-Style Keybindings ⌨️
Now supports vim-style navigation for power users:
- `j` / `k` - Scroll down/up (same as ↑/↓)
- `g` / `G` - Jump to top/bottom (same as Home/End)
- `Ctrl-u` / `Ctrl-d` - Page up/down (same as PgUp/PgDn)

All vim bindings work alongside traditional navigation keys, so both styles are supported.

### 3. View Counts in Tabs 📊
Tabs now show item counts for better context:
- **Network (5)** - 5 network interfaces
- **Packages (1247)** - 1247 packages
- **Services (42)** - 42 services
- **Databases (3)** - 3 databases installed
- **WebServers (2)** - 2 web servers
- **Issues (12)** - 12 security issues
- **Storage (8)** - 8 mount points
- **Users (23)** - 23 user accounts
- **Kernel (156)** - 156 kernel modules

Views without counts (Dashboard, Security, Profiles) show plain names.

### 4. Updated Help System 📖
- Help overlay now documents vim keybindings
- Clearer descriptions of keyboard shortcuts
- Better organization of command categories

## Technical Details

### Files Modified
- `src/cli/tui/mod.rs` - Splash screen integration, vim keybindings
- `src/cli/tui/ui.rs` - View counts in tabs, help overlay updates
- `src/cli/tui/splash.rs` - Removed dead_code attribute

### Keyboard Event Handling
Vim keybindings are context-aware and only activate when:
- Not in search mode
- Not entering a filename
- Not in other input modes

This prevents conflicts with text input.

### Color Theme
Consistent coral-terracotta orange theme (Pantone 7416 C):
- Primary: RGB(222, 115, 86)
- Dark: RGB(180, 85, 60)
- Light: RGB(255, 145, 115)

## User Experience

### Before
- No splash screen (directly to loading spinner)
- Arrow keys only for navigation
- Tabs showed only view names
- Help didn't mention vim-style controls

### After
- Polished startup with splash screen
- Vim users can use familiar j/k/g/G navigation
- Tabs show helpful counts at a glance
- Comprehensive help documentation

## Performance

- Splash screen adds only 800ms to startup
- No performance impact during normal operation
- Tab count calculation is O(1) - just reading existing data

## Future Enhancements

See `docs/development/tui-development-plan.md` for:
- Real-time updates
- Enhanced search (regex, case-insensitive)
- Charts and graphs
- Mouse support
- Comparison mode
- And more...

## Usage

Run the TUI with:
```bash
guestctl tui vm.qcow2
```

Or install and run:
```bash
cargo install guestkit
guestctl tui vm.qcow2
```

## Screenshots

*(Screenshots to be added after testing with real VM images)*

## Compatibility

- Works on all terminals that support:
  - UTF-8 characters
  - 256 colors or true color
  - Crossterm backend

- Tested on:
  - Linux (primary platform)
  - Terminal emulators: gnome-terminal, alacritty, kitty, wezterm

## Configuration System ⚙️

### Overview
The TUI now supports user configuration through a TOML file located at `~/.config/guestkit/tui.toml`.

### Configuration File
Create a config file to customize your TUI experience:

```toml
[ui]
show_splash = true              # Show splash screen on startup
splash_duration_ms = 800        # Splash duration in milliseconds
show_stats_bar = true           # Show statistics bar
theme = "default"               # Color theme (currently only "default")
mouse_enabled = true            # Enable mouse support

[behavior]
default_view = "dashboard"      # Initial view on startup
auto_refresh_seconds = 0        # Auto-refresh interval (0 = disabled)
search_case_sensitive = false   # Search case-sensitive by default
search_regex_mode = false       # Search regex mode by default
max_bookmarks = 20              # Maximum bookmarks
page_scroll_lines = 10          # Lines to scroll with Page Up/Down

[keybindings]
vim_mode = true                 # Enable vim-style keybindings
quick_jump_enabled = true       # Enable Ctrl+P quick jump menu
```

### Configuration Options

#### UI Settings (`[ui]`)
- **show_splash**: Enable/disable splash screen (default: `true`)
- **splash_duration_ms**: How long to show splash in milliseconds (default: `800`)
- **show_stats_bar**: Show/hide the statistics bar (default: `true`)
- **theme**: Color theme selection (default: `"default"`)
- **mouse_enabled**: Enable/disable mouse support (default: `true`)

#### Behavior Settings (`[behavior]`)
- **default_view**: Which view to show on startup (default: `"dashboard"`)
  - Options: `"dashboard"`, `"network"`, `"packages"`, `"services"`, `"databases"`, `"webservers"`, `"security"`, `"issues"`, `"storage"`, `"users"`, `"kernel"`, `"profiles"`
- **auto_refresh_seconds**: Auto-refresh interval in seconds (default: `0` = disabled)
- **search_case_sensitive**: Search case-sensitive by default (default: `false`)
- **search_regex_mode**: Enable regex search by default (default: `false`)
- **max_bookmarks**: Maximum number of bookmarks (default: `20`)
- **page_scroll_lines**: Lines to scroll with Page Up/Down (default: `10`)

#### Keybindings Settings (`[keybindings]`)
- **vim_mode**: Enable vim-style navigation (default: `true`)
- **quick_jump_enabled**: Enable Ctrl+P quick jump menu (default: `true`)

### Example Configurations

**Minimal Splash, Start at Services:**
```toml
[ui]
show_splash = false

[behavior]
default_view = "services"
```

**Power User Setup:**
```toml
[ui]
show_splash = false
mouse_enabled = false

[behavior]
default_view = "issues"
search_regex_mode = true
page_scroll_lines = 20
```

**Accessibility-Focused:**
```toml
[ui]
splash_duration_ms = 2000
show_stats_bar = true

[behavior]
default_view = "dashboard"
auto_refresh_seconds = 60
```

### Configuration File Location
The configuration file is automatically loaded from:
- **Linux/macOS**: `~/.config/guestkit/tui.toml`
- **Windows**: `%APPDATA%\guestkit\tui.toml`

### Creating Configuration
1. Create the directory:
   ```bash
   mkdir -p ~/.config/guestkit
   ```

2. Copy the example config:
   ```bash
   cp docs/tui-config-example.toml ~/.config/guestkit/tui.toml
   ```

3. Edit to your preferences:
   ```bash
   nano ~/.config/guestkit/tui.toml
   ```

### Default Behavior
If no configuration file exists, the TUI uses built-in defaults:
- Splash screen: enabled (800ms)
- Mouse support: enabled
- Vim keybindings: enabled
- Default view: Dashboard
- Auto-refresh: disabled
- All settings match the defaults listed above

### Configuration Validation
- Invalid configuration files fall back to defaults
- Unknown fields are ignored
- Type mismatches use defaults for that setting

## Credits

- ASCII art logo created for GuestKit branding
- Color scheme inspired by Pantone 7416 C (Coral/Terracotta)
- Vim keybindings follow standard vim conventions
- Configuration system uses TOML for human-friendly editing
