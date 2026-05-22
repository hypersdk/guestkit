# Interactive File Explorer - CLI Explore Command

**New Feature:** Interactive TUI file browser for VM filesystem exploration

---

## Overview

The `explore` command provides an intuitive, visual file browser for navigating VM filesystems interactively. It enhances the guestkit shell with a terminal UI (TUI) that makes exploration and file discovery easier and more enjoyable.

##Usage

### Launch from Shell

```bash
# Start guestkit interactive shell
guestkit shell disk.qcow2

# Launch explorer from current directory
explore

# Launch explorer from specific path
explore /etc

# Short alias
ex /var/log
```

### Launch Directly from CLI ⭐ NEW!

```bash
# Launch explorer directly on a disk image
guestkit explore disk.qcow2

# Start from specific path
guestkit explore disk.qcow2 /etc

# Using short alias
guestkit ex vm-image.qcow2 /var/log

# With verbose output
guestkit explore --verbose disk.qcow2 /home
```

---

## Features

### Visual Navigation

- **Directory Listing**: Color-coded files and folders with icons
- **File Information**: Size, type, and metadata displayed
- **Breadcrumb Path**: Always know where you are
- **Selection Highlighting**: Clear visual indicator of current selection

### Keyboard Controls

#### Navigation
```
↑/↓ or k/j    - Move selection up/down
PgUp/PgDn     - Page up/down (fast navigation)
Enter         - Enter directory / View file
Backspace     - Go to parent directory
```

#### Actions
```
v             - View file content (pager-like)
i             - Show detailed file information
/             - Filter files by name
.             - Toggle hidden files (.files)
s             - Cycle sort mode (name → size → type → name)
```

#### General
```
h or ?        - Show help overlay
q or Esc      - Exit explorer
Ctrl+C        - Force exit
```

---

## Interface

### Main View

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║ 📂 GuestKit File Explorer - Ubuntu 22.04.3 LTS                              ║
╠═══════════════════════════════════════════════════════════════════════════════╣
📍 Path: /etc/apache2
📊 Items: 45

├───────────────────────────────────────────────────────────────────────────────┤
  📁 ..
▸ 📁 conf-available                                          <DIR>
  📁 conf-enabled                                            <DIR>
  📁 mods-available                                          <DIR>
  📁 mods-enabled                                            <DIR>
  📁 sites-available                                         <DIR>
  📁 sites-enabled                                           <DIR>
  ⚙️  apache2.conf                                             7.28 KB
  ⚙️  envvars                                                  1.91 KB
  📄 magic                                                   111 B
  🔧 ports.conf                                              1.04 KB
├───────────────────────────────────────────────────────────────────────────────┤
ℹ️  Info: Directory | Size: 0 B
╠═══════════════════════════════════════════════════════════════════════════════╣
║ ↑↓ Navigate  Enter Open  h Help  q Quit                                     ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Color Coding

Files are color-coded by type for easy identification:

| Color | Type | Examples |
|-------|------|----------|
| **Blue** | Directories | All folders |
| **Green** | Executables/Scripts | .sh, .py, .rb |
| **Yellow** | Source Code | .rs, .c, .cpp, .java, .go |
| **Cyan** | Configuration | .conf, .cfg, .yaml, .json, .toml |
| **Red** | Archives | .tar, .gz, .zip, .bz2 |
| **White** | Text Files | .txt, .md, .log |
| **Gray** | Hidden Files | .bashrc, .profile |

### File Icons

Visual indicators for different file types:

- 📁 **Directories**
- 📄 **Text Files** (.txt, .md, .log)
- 💻 **Source Code** (.rs, .py, .js, .java, .c, .cpp, .go)
- ⚙️  **Config Files** (.json, .yaml, .yml, .toml, .xml)
- 🖼️  **Images** (.jpg, .png, .gif, .bmp, .svg)
- 📕 **PDFs**
- 📦 **Archives** (.zip, .tar, .gz, .bz2, .xz)
- 🔧 **Scripts** (.sh, .bash)
- 🔐 **Configs** (.conf, .config, .cfg)
- 📝 **Other Files**

---

## Feature Details

### 1. File Viewing (v)

Press `v` on a file to view its content:

```
╔═ Viewing: /etc/apache2/apache2.conf ═╗
   1 │ # This is the main Apache HTTP server configuration file.
   2 │ # It contains the configuration directives that give the server its instructions.
   3 │ # See <URL:http://httpd.apache.org/docs/2.4/> for detailed information.
   4 │ # In particular, see
   5 │ # <URL:http://httpd.apache.org/docs/2.4/mod/directives.html>
   ...
  98 │ Include conf-enabled/*.conf
  99 │ Include sites-enabled/*.conf
 100 │

... (245 more lines)

Press any key to return...
```

### 2. File Information (i)

Press `i` on a file to see detailed metadata:

```
╔═ File Information: apache2.conf ═╗
Path: /etc/apache2/apache2.conf
Type: File
Size: 7.28 KB
Mode: 100644
UID: 0
GID: 0
File Type: ASCII text

Press any key to return...
```

### 3. Filtering (/)

Press `/` to filter files by name:

```
Enter filter (filename contains):
> apache

# Shows only files containing "apache" in the name
📊 Items: 3 (filter: 'apache')
```

Clear filter: Use `/` again and enter empty string

### 4. Hidden Files Toggle (.)

Press `.` to toggle visibility of hidden files:

```
Hidden Files: OFF  →  ON
📊 Items: 45      →  📊 Items: 67
```

### 5. Sort Modes (s)

Cycle through sort modes:

1. **Name** (default): Alphabetical, directories first
2. **Size**: Largest files first
3. **Type**: Grouped by file extension

### 6. Help Overlay (h or ?)

Press `h` or `?` to see full help:

```
╔════════════════════ Explorer Help ═══════════════════╗
║                                                       ║
║ 📖 Navigation                                        ║
║   ↑/↓ or k/j    - Move selection up/down             ║
║   PgUp/PgDn     - Page up/down                       ║
║   Enter         - Enter directory / view file        ║
║   Backspace     - Go to parent directory             ║
║                                                       ║
║ ⚡ Actions                                            ║
║   v             - View file content                  ║
║   i             - Show file info                     ║
║   /             - Filter files                       ║
║   .             - Toggle hidden files                ║
║   s             - Cycle sort mode                    ║
║                                                       ║
║ 🔧 General                                            ║
║   h or ?        - Show this help                     ║
║   q or Esc      - Exit explorer                      ║
║   Ctrl+C        - Force exit                         ║
║                                                       ║
╚═══════════════════════════════════════════════════════╝

Press any key to continue...
```

---

## Use Cases

### 1. Quick File Discovery

```bash
# Find configuration files
explore /etc
# Navigate with arrows, filter with '/', view with 'v'
```

### 2. Log File Investigation

```bash
# Explore log directory
explore /var/log
# Sort by size (s), view recent logs (v)
```

### 3. Application Analysis

```bash
# Check web server config
explore /etc/nginx
# or
explore /etc/apache2
```

### 4. Home Directory Inspection

```bash
# Explore user home
explore /home/username
# Toggle hidden files (.) to see .bashrc, .ssh, etc.
```

### 5. System Service Configuration

```bash
# Check systemd units
explore /etc/systemd/system
```

---

## Workflow Integration

The explorer integrates seamlessly with other guestkit shell commands:

```bash
# Example workflow
guestkit> cd /var/www
guestkit> explore         # Visual navigation
guestkit> cat html/index.html    # View specific file
guestkit> grep -r "TODO" .       # Search across files
guestkit> tree 2                  # Directory tree view
```

---

## Performance

### Optimized for Large Directories

- **Pagination**: Shows ~20 items at a time
- **Lazy Loading**: Only loads visible entries
- **Fast Sorting**: Efficient in-memory sorting
- **Responsive**: Immediate key response

### Memory Footprint

- Minimal memory usage
- Suitable for directories with 1000+ files
- No disk cache required

---

## Technical Details

### Implementation

- **Language**: Rust
- **TUI Library**: crossterm for terminal control
- **Backend**: guestkit library (libguestfs)
- **Platform**: Linux, works in any terminal

### Terminal Requirements

- **Color Support**: 256-color terminal recommended
- **Minimum Size**: 80x24 characters
- **Unicode**: UTF-8 support for icons

### Compatibility

- ✅ Linux terminals (xterm, gnome-terminal, konsole)
- ✅ macOS Terminal.app, iTerm2
- ✅ Windows Terminal, ConEmu (WSL)
- ✅ SSH sessions
- ✅ tmux, screen

---

## Comparison with Other Commands

| Feature | `ls` | `tree` | `find` | `explore` |
|---------|------|--------|--------|-----------|
| **Interactive** | ❌ | ❌ | ❌ | ✅ |
| **Visual** | Partial | ✅ | ❌ | ✅ |
| **File Preview** | ❌ | ❌ | ❌ | ✅ |
| **Navigation** | ❌ | ❌ | ❌ | ✅ |
| **Filtering** | ❌ | ❌ | ✅ | ✅ |
| **Sorting** | ✅ | ❌ | ✅ | ✅ |
| **Icons** | ❌ | ❌ | ❌ | ✅ |
| **Color Coding** | Partial | Partial | ❌ | ✅ |

---

## Tips & Tricks

### 1. Quick Navigation

```bash
# Jump to common directories
explore /etc          # Config files
explore /var/log      # Logs
explore /var/www      # Web content
explore /home         # User homes
explore /opt          # Optional apps
```

### 2. Find Large Files

```bash
explore /var
# Press 's' twice to sort by size
# Navigate to largest directories
```

### 3. Configuration Discovery

```bash
explore /etc
# Filter for service: '/' then type 'nginx' or 'ssh'
# View configs with 'v'
```

### 4. Security Audit

```bash
explore /home/user
# Press '.' to show hidden files
# Look for .ssh/authorized_keys
# Press 'i' for file permissions
```

### 5. Web Application Analysis

```bash
explore /var/www/html
# Navigate directory structure
# View index.html, .htaccess
# Check permissions with 'i'
```

---

## Future Enhancements

### Planned Features

- [ ] **Copy/Move Operations**: Copy files between locations
- [ ] **Delete Support**: Remove files (with confirmation)
- [ ] **Multi-Selection**: Select multiple files
- [ ] **Bookmarks**: Save favorite locations
- [ ] **Search**: Full-text content search
- [ ] **Diff View**: Compare two files side-by-side
- [ ] **Archive Preview**: Look inside .tar.gz without extracting
- [ ] **Syntax Highlighting**: Color code for source files
- [ ] **Watch Mode**: Auto-refresh on changes
- [ ] **Export**: Save file list to CSV/JSON

### Integration Possibilities

- [x] **Direct launch from main CLI**: `guestkit explore disk.qcow2` ✅ **DONE!**
- [ ] Integration with TUI mode (`guestkit tui`)
- [ ] Bulk operations on selected files
- [ ] Integration with compare/diff commands
- [ ] Quick actions menu (right-click simulation)

---

## Troubleshooting

### Explorer Won't Start

**Issue**: Command not recognized
```bash
guestkit> explore
Unknown command: explore
```

**Solution**: Ensure you're in the interactive shell mode:
```bash
# Start shell first
guestkit shell disk.qcow2

# Then use explore
guestkit> explore
```

### Colors Not Showing

**Issue**: No colors or garbled output

**Solution**:
```bash
# Check terminal color support
echo $TERM

# Should be: xterm-256color or similar
# If not, set it:
export TERM=xterm-256color
```

### Icons Not Displaying

**Issue**: Boxes or question marks instead of icons

**Solution**: Terminal needs UTF-8 support
```bash
# Check locale
locale

# Should include UTF-8
# If not:
export LANG=en_US.UTF-8
```

### Slow Navigation

**Issue**: Lag when navigating large directories

**Solution**: Filter to reduce visible items
```bash
# Press '/' and filter by type
> .conf    # Show only .conf files
> nginx    # Show only files containing "nginx"
```

---

## Examples

### Example 1: Security Configuration Review

```bash
guestkit shell ubuntu.qcow2
guestkit> explore /etc

# Navigate to ssh config
# Press 'v' on sshd_config
# Check for PermitRootLogin, PasswordAuthentication
# Press 'q' to return
# Navigate to other security configs
```

### Example 2: Web Server Troubleshooting

```bash
guestkit> explore /var/log/nginx
# Sort by size with 's'
# View error.log with 'v'
# Check access.log
# Navigate to /etc/nginx
# Review nginx.conf
```

### Example 3: Finding Large Log Files

```bash
guestkit> explore /var/log
# Press 's' twice to sort by size
# Top files are largest
# Press 'i' to see exact sizes
# Press 'v' to check content
```

### Example 4: User Account Audit

```bash
guestkit> explore /home
# Check each user directory
# Press '.' to show hidden files
# Look for .ssh/authorized_keys
# Check .bash_history
# Review .bashrc
```

---

## Integration with Guestkit Ecosystem

The explore command complements existing guestkit features:

### With Shell Commands

```bash
# Explore visually
explore /etc

# Then use shell commands for operations
cat /etc/hosts
grep "server" /etc/nginx/nginx.conf
find /etc -name "*.conf"
```

### With TUI Mode

```bash
# Full-screen TUI
guestkit tui disk.qcow2

# Or shell with explorer
guestkit shell disk.qcow2
guestkit> explore
```

### With Inspection

```bash
# High-level inspection
guestkit inspect disk.qcow2

# Then explore for details
guestkit shell disk.qcow2
guestkit> explore /var/log
```

---

## Keyboard Cheat Sheet

```
╔════════════════════════════════════════╗
║         Quick Reference Card           ║
╠════════════════════════════════════════╣
║ Navigation        │ Actions            ║
║ ─────────────────┼───────────────────  ║
║ ↑↓ / j/k         │ v - View           ║
║ PgUp/PgDn        │ i - Info           ║
║ Enter - Open     │ / - Filter         ║
║ Backspace - Up   │ . - Hidden         ║
║                  │ s - Sort           ║
║ ─────────────────┼───────────────────  ║
║ h/? - Help       │ q/Esc - Quit       ║
╚════════════════════════════════════════╝
```

---

## Summary

The **explore** command brings modern, intuitive file browsing to VM inspection. Its visual interface, keyboard-driven navigation, and integrated actions make filesystem exploration faster and more enjoyable than traditional command-line tools.

**Key Benefits:**
- ✅ Visual, color-coded interface
- ✅ Fast keyboard navigation
- ✅ Integrated file preview
- ✅ Smart filtering and sorting
- ✅ Works in any terminal
- ✅ Zero configuration required

**Get Started:**

**Method 1: Direct Launch** (Fastest!)
```bash
guestkit explore your-vm.qcow2
# Instantly start exploring!
```

**Method 2: From Shell**
```bash
guestkit shell your-vm.qcow2
guestkit> explore
# Start exploring from shell!
```

---

*Happy exploring!* 🚀📂

**Documentation Version:** 1.0
**Last Updated:** 2026-01-30
**Guestkit Version:** 0.3.1+
