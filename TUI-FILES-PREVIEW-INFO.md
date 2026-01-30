# TUI Files View - File Preview and Information Display

**Date:** 2026-01-30
**Feature:** File Preview and Information Display in TUI Files View
**Status:** ✅ **COMPLETE**

---

## Overview

Added file preview and information display capabilities to the TUI Files view, enabling users to quickly inspect file contents and metadata without leaving the TUI interface.

---

## What Was Implemented

### 1. File Preview (v key)

**Functionality:**
- Displays file content in a popup overlay
- Shows first 100 lines with line numbers
- Truncates long lines (>120 chars) for readability
- Size limit: 1MB (prevents freezing on large files)

**Visual Design:**
```
╔═══════════════════════════════════════════════════════════════╗
║ 📄 File Preview: /etc/nginx/nginx.conf (47/47 lines)        ║
╠═══════════════════════════════════════════════════════════════╣
║    1 │ user www-data;                                         ║
║    2 │ worker_processes auto;                                 ║
║    3 │ pid /run/nginx.pid;                                    ║
║    4 │ include /etc/nginx/modules-enabled/*.conf;             ║
║    5 │                                                         ║
║    6 │ events {                                               ║
║    7 │     worker_connections 768;                            ║
║    8 │ }                                                       ║
║    9 │                                                         ║
║   10 │ http {                                                 ║
║  ...                                                           ║
╠═══════════════════════════════════════════════════════════════╣
║           Press ESC or q to close                             ║
╚═══════════════════════════════════════════════════════════════╝
```

**Features:**
- Line numbers for easy reference
- Orange header with file path
- Shows lines displayed vs total lines
- Clear close instructions
- Black background for contrast

### 2. File Information (i key)

**Functionality:**
- Displays detailed file metadata
- Shows file type, size, permissions
- Includes UID/GID, mode, blocks
- Auto-detects file type with libguestfs

**Visual Design:**
```
╔═══════════════════════════════════════════════════════════╗
║ ℹ️  File Information                                       ║
╠═══════════════════════════════════════════════════════════╣
║ Path: /etc/nginx/nginx.conf                               ║
║ Type: File                                                 ║
║ Size: 1.45 KB (1486 bytes)                                ║
║ Mode: 100644                                               ║
║ UID: 0                                                     ║
║ GID: 0                                                     ║
║ Blocks: 8                                                  ║
║ File Type: ASCII text                                      ║
╠═══════════════════════════════════════════════════════════╣
║           Press ESC or q to close                          ║
╚═══════════════════════════════════════════════════════════╝
```

**Features:**
- Key-value pairs with color coding
- Orange keys, white values
- Human-readable file sizes
- Comprehensive metadata

---

## Implementation Details

### Code Changes

**Modified: `src/cli/tui/app.rs`**

Added fields to App struct:
```rust
pub show_file_preview: bool,
pub file_preview_content: String,
pub file_preview_path: String,
pub show_file_info: bool,
pub file_info_content: String,
```

Added methods:
```rust
/// Show preview of selected file
pub fn show_file_preview(&mut self)

/// Show information about selected file
pub fn show_file_information(&mut self)

/// Close file preview
pub fn close_file_preview(&mut self)

/// Close file info
pub fn close_file_info(&mut self)
```

Added helper function:
```rust
fn format_file_size(size: i64) -> String
```

**Modified: `src/cli/tui/views/files.rs`**

Added helper functions:
```rust
/// Get the full path of the currently selected file
pub fn get_selected_file_path(browser: &FileBrowserState) -> Option<String>

/// Get information about the currently selected file
pub fn get_selected_file_info(browser: &FileBrowserState) -> Option<&FileEntry>
```

Updated footer to show new shortcuts:
```rust
"↑↓ Navigate  Enter Open  v View  i Info  . Hidden  Backspace Up"
```

**Modified: `src/cli/tui/ui.rs`**

Added overlay rendering:
```rust
if app.show_file_preview {
    draw_file_preview(f, app);
}

if app.show_file_info {
    draw_file_info(f, app);
}
```

Added rendering functions:
```rust
fn draw_file_preview(f: &mut Frame, app: &App)
fn draw_file_info(f: &mut Frame, app: &App)
```

**Modified: `src/cli/tui/mod.rs`**

Updated keyboard handlers:
```rust
KeyCode::Char('v') if app.current_view == app::View::Files => {
    app.show_file_preview();
}

KeyCode::Char('i') if app.current_view == app::View::Files => {
    app.show_file_information();
}

KeyCode::Char('q') | KeyCode::Esc => {
    if app.show_file_preview {
        app.close_file_preview();
    } else if app.show_file_info {
        app.close_file_info();
    } else {
        // ... existing handlers
    }
}
```

---

## Keyboard Controls

### Files View

| Key | Action |
|-----|--------|
| **v** | View file content (preview) |
| **i** | Show file information (metadata) |
| **ESC** / **q** | Close overlay (when preview/info is open) |

### In Preview/Info Overlay

| Key | Action |
|-----|--------|
| **ESC** | Close and return to file list |
| **q** | Close and return to file list |

---

## User Experience

### Viewing File Content

1. **Navigate to file:**
   - Use ↑↓ to select a file (not directory)

2. **Press 'v' to preview:**
   - Popup appears with file content
   - Line numbers shown on left
   - First 100 lines displayed
   - Long lines truncated with "..."

3. **Close preview:**
   - Press ESC or q
   - Returns to file list

### Viewing File Info

1. **Navigate to file:**
   - Use ↑↓ to select a file or directory

2. **Press 'i' for info:**
   - Popup appears with metadata
   - Shows path, type, size, permissions
   - File type auto-detected

3. **Close info:**
   - Press ESC or q
   - Returns to file list

---

## Safety Features

### File Preview Protections

1. **Size Limit:**
   - Maximum 1MB file size
   - Prevents memory issues
   - Shows notification if exceeded

2. **Directory Check:**
   - Cannot preview directories
   - Shows "Cannot preview directory" notification

3. **Line Limit:**
   - Shows first 100 lines only
   - Displays "X/Y lines" indicator
   - Prevents UI freezing on huge files

4. **Line Truncation:**
   - Lines > 120 chars truncated
   - Adds "..." to indicate truncation
   - Keeps UI readable

### Error Handling

1. **Read Errors:**
   - Caught and displayed as notification
   - Doesn't crash TUI
   - User can continue browsing

2. **Permission Errors:**
   - Silent handling
   - Shows error in notification

---

## Technical Details

### File Preview Implementation

```rust
// Check if directory
if guestfs.is_dir(&path)? {
    show_notification("Cannot preview directory");
    return;
}

// Check file size
if guestfs.filesize(&path)? > 1024 * 1024 {
    show_notification("File too large to preview");
    return;
}

// Read content
let content = guestfs.cat(&path)?;

// Render with line numbers
lines.enumerate().take(100).map(|(idx, line)| {
    Line::from(vec![
        Span::styled(format!("{:4} │ ", idx + 1), LIGHT_ORANGE),
        Span::styled(truncate_line(line, 120), TEXT_COLOR),
    ])
})
```

### File Info Implementation

```rust
// Gather metadata
let info = vec![
    format!("Path: {}", path),
    format!("Type: {}", if is_dir { "Directory" } else { "File" }),
    format!("Size: {} ({} bytes)", format_size(size), size),
    format!("Mode: {:o}", stat.mode),
    format!("UID: {}", stat.uid),
    format!("GID: {}", stat.gid),
    format!("File Type: {}", guestfs.file(&path)?),
];

// Display as key-value pairs
info.lines().map(|line| {
    if let Some((key, value)) = line.split_once(": ") {
        Line::from(vec![
            Span::styled(format!("{}: ", key), LIGHT_ORANGE.bold()),
            Span::styled(value, TEXT_COLOR),
        ])
    }
})
```

---

## Benefits

### For Users

- **Quick Inspection**: No need to exit TUI or use separate commands
- **Visual Feedback**: Clear, readable overlays
- **Safety**: Size limits prevent freezing
- **Context**: Stay in current directory while viewing files
- **Efficiency**: Faster than switching between views

### For Workflow

- **Security Audits**: Quickly check config files
- **Log Analysis**: Preview log files before extracting
- **Code Review**: View source files inline
- **Troubleshooting**: Check file permissions and ownership
- **Investigation**: Verify file types and contents

---

## Use Cases

### 1. Configuration Review

```
1. Navigate to /etc/nginx
2. Select nginx.conf
3. Press 'v' to preview
4. Review worker_processes, server blocks
5. Press ESC
6. Select sites-enabled/default
7. Press 'v' to preview
```

### 2. Security Audit

```
1. Navigate to /etc/ssh
2. Select sshd_config
3. Press 'i' for info
4. Check permissions (should be 600)
5. Press ESC
6. Press 'v' to preview
7. Review PermitRootLogin, PasswordAuthentication
```

### 3. Log Investigation

```
1. Navigate to /var/log
2. Sort by size (press 's' twice)
3. Select largest log
4. Press 'i' to check size
5. If < 1MB, press 'v' to preview
6. Review recent entries
```

### 4. Application Analysis

```
1. Navigate to /var/www/html
2. Select index.php
3. Press 'i' for metadata
4. Check owner/group (should be www-data)
5. Press 'v' to preview code
6. Review for issues
```

---

## Future Enhancements

### Immediate Improvements

- [ ] **Scrolling in Preview** - Arrow keys to scroll through file
- [ ] **Syntax Highlighting** - Color code for different file types
- [ ] **Search in Preview** - Find text within preview
- [ ] **Copy Content** - Copy file content to clipboard

### Advanced Features

- [ ] **Hex View** - Toggle between text and hex display
- [ ] **Binary File Detection** - Show hex for binary files
- [ ] **Large File Handling** - Stream large files instead of full load
- [ ] **Diff View** - Compare two files side-by-side
- [ ] **Edit Mode** - Allow editing files directly (advanced)

### Integration

- [ ] **Jump to Line** - Enter line number to jump
- [ ] **Export Preview** - Save preview to file
- [ ] **Share Preview** - Copy preview with line numbers
- [ ] **Recent Files** - Track recently previewed files

---

## Keyboard Reference

### Complete Files View Shortcuts

| Key | Action | Description |
|-----|--------|-------------|
| **↑ / k** | Move up | Navigate file list |
| **↓ / j** | Move down | Navigate file list |
| **Enter** | Open | Enter directory or view file |
| **Backspace** | Parent | Go to parent directory |
| **v** | View | Preview file content |
| **i** | Info | Show file information |
| **.** | Hidden | Toggle hidden files |
| **ESC / q** | Close | Close overlay or exit view |
| **Tab** | Next view | Switch to next TUI view |

---

## Testing Status

### Compilation ✅

```bash
$ cargo check --lib
   Finished `dev` profile [unoptimized + debuginfo] in 0.16s
```

### Manual Testing Required

1. **File Preview:**
   - [ ] Preview small text file (<1KB)
   - [ ] Preview medium file (100KB)
   - [ ] Preview at size limit (1MB)
   - [ ] Try to preview >1MB file (should show error)
   - [ ] Try to preview directory (should show error)
   - [ ] Preview file with long lines (>120 chars)
   - [ ] Preview file with >100 lines

2. **File Information:**
   - [ ] Info on regular file
   - [ ] Info on directory
   - [ ] Info on symlink
   - [ ] Verify permissions display
   - [ ] Verify size formatting (B, KB, MB, GB)
   - [ ] Verify file type detection

3. **Keyboard Controls:**
   - [ ] Press 'v' opens preview
   - [ ] Press 'i' opens info
   - [ ] ESC closes overlay
   - [ ] q closes overlay
   - [ ] ESC works when no overlay open

4. **Edge Cases:**
   - [ ] Empty file
   - [ ] Binary file
   - [ ] File with special characters
   - [ ] File with no permissions
   - [ ] Symlink to missing file

---

## Code Statistics

### Lines Added

- **src/cli/tui/app.rs**: ~130 lines
  - 5 new fields
  - 4 new methods
  - 1 helper function

- **src/cli/tui/views/files.rs**: ~40 lines
  - 2 helper functions
  - Updated footer

- **src/cli/tui/ui.rs**: ~130 lines
  - 2 overlay rendering functions
  - Integration in main draw

- **src/cli/tui/mod.rs**: ~15 lines
  - Keyboard handlers for v, i
  - Updated ESC/q handler

**Total:** ~315 lines of new code

---

## Summary

Successfully implemented file preview and information display for the TUI Files view:

✅ **File Preview** - View file content with line numbers
✅ **File Information** - Display metadata and permissions
✅ **Safety Features** - Size limits and error handling
✅ **Visual Overlays** - Clean, centered popups
✅ **Keyboard Controls** - Intuitive v/i/ESC shortcuts
✅ **Footer Updates** - Clear keyboard hints
✅ **Zero Compilation Errors** - Clean build

The TUI Files view now provides comprehensive file inspection capabilities, making it a powerful tool for VM filesystem exploration and analysis!

---

**Status**: ✅ **FILE PREVIEW AND INFO COMPLETE**

**Features Available:**
1. ✅ Visual file browser with colors and icons
2. ✅ Interactive navigation (Enter, Backspace, arrows)
3. ✅ Hidden files toggle (. key)
4. ✅ **File preview (v key)** - NEW!
5. ✅ **File information (i key)** - NEW!
6. ✅ Persistent guestfs for real-time operations

**Next Steps:**
1. Add scrolling in file preview
2. Add syntax highlighting for code files
3. Add search within preview

---

*Implementation Date: 2026-01-30*
*Total Code: 315+ lines*
*Compilation: Clean ✅*
