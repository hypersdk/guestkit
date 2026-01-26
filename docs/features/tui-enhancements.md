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

## Credits

- ASCII art logo created for GuestKit branding
- Color scheme inspired by Pantone 7416 C (Coral/Terracotta)
- Vim keybindings follow standard vim conventions
