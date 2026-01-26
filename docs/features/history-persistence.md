# Command History Persistence Guide

GuestCtl's interactive mode now automatically saves your command history across sessions, making it easy to repeat previous operations and explore VMs more efficiently.

## Features

### 📜 Automatic History Saving
- Commands are automatically saved when you exit
- History persists across interactive sessions
- Per-disk history files for context-specific workflows

### 🔄 History Navigation
- **↑** (Up Arrow) - Previous command
- **↓** (Down Arrow) - Next command
- **Ctrl+R** - Reverse search through history
- **Ctrl+S** - Forward search through history

### 💾 Storage Location
- **Directory:** `~/.guestctl/history/`
- **Format:** `guestctl-{hash}.history`
- **Encoding:** Plain text, one command per line

## Usage

### Basic Usage

Just use interactive mode normally - history is saved automatically:

```bash
# Start interactive session
guestctl interactive vm.qcow2

guestctl> mount /dev/sda1 /
guestctl> packages
guestctl> services
guestctl> exit

# Start again - your history is preserved!
guestctl interactive vm.qcow2

guestctl> # Press ↑ to see "services"
guestctl> # Press ↑ again to see "packages"
guestctl> # Press ↑ again to see "mount /dev/sda1 /"
```

### Reverse Search (Ctrl+R)

Search through history interactively:

```bash
guestctl> # Press Ctrl+R
(reverse-i-search)`pack': packages

# Type more to narrow search
(reverse-i-search)`packages': packages grep nginx

# Press Enter to execute or Ctrl+R again for previous match
```

### History Per Disk

Each disk image has its own history file:

```bash
# Working with production VM
guestctl interactive prod-web-01.qcow2
guestctl> mount /dev/sda1 /
guestctl> packages grep nginx
guestctl> exit

# Working with staging VM (different history)
guestctl interactive staging-web-01.qcow2
guestctl> mount /dev/vda1 /
guestctl> services
guestctl> exit

# Return to production - original history preserved
guestctl interactive prod-web-01.qcow2
guestctl> # Press ↑ to see "packages grep nginx"
```

## How It Works

### History Files

History files are stored in `~/.guestctl/history/` with unique names based on the disk path:

```
~/.guestctl/history/
├── guestctl-a1b2c3d4e5f6g7h8.history  # prod-web-01.qcow2
├── guestctl-9i8h7g6f5e4d3c2b.history  # staging-web-01.qcow2
└── guestctl-1a2b3c4d5e6f7g8h.history  # dev-db-01.qcow2
```

The hash is generated from the disk path, ensuring:
- Same disk = same history file
- Different disks = different history files
- Disk path changes = new history file

### Automatic Loading

When you start interactive mode:
1. GuestCtl computes hash of disk path
2. Checks for existing history file
3. Loads history if file exists
4. Shows message: "→ Loaded command history"

### Automatic Saving

When you exit (via `exit`, `quit`, or Ctrl+D):
1. GuestCtl saves all commands from session
2. Merges with existing history
3. Writes to disk-specific history file
4. Silent unless error occurs

## Advanced Usage

### Manual History Management

View history file directly:
```bash
# Find your history file
ls -lah ~/.guestctl/history/

# View contents
cat ~/.guestctl/history/guestctl-*.history

# Count commands
wc -l ~/.guestctl/history/guestctl-*.history
```

### Clear History for Specific Disk

```bash
# Remove specific history file
rm ~/.guestctl/history/guestctl-a1b2c3d4e5f6g7h8.history

# Or clear all history
rm -rf ~/.guestctl/history/
```

### Export History

```bash
# Copy history for backup
cp ~/.guestctl/history/guestctl-*.history ~/backups/

# Share common commands with team
cat ~/.guestctl/history/guestctl-*.history | \
  grep "^mount\|^packages\|^services" > team-workflow.txt
```

## Tips & Tricks

### Efficient Workflows

**1. Build Custom Inspection Sequences:**
```bash
guestctl> mount /dev/sda1 /
guestctl> packages | grep apache
guestctl> services | grep httpd
guestctl> cat /etc/httpd/conf/httpd.conf
guestctl> exit

# Next time: Just use ↑ to replay entire sequence!
```

**2. Debug Iteratively:**
```bash
guestctl> find /var/log
guestctl> cat /var/log/messages | grep error
guestctl> cat /var/log/syslog | grep failed
# Each refinement is saved for next session
```

**3. Create Reusable Patterns:**
```bash
# First session - explore and refine
guestctl> packages | grep -i sec
guestctl> packages | grep -i audit
guestctl> packages | grep -i firewall

# Later sessions - reuse best pattern
guestctl> # Press Ctrl+R, type "packages", select best one
```

### Search Shortcuts

- **Partial Match:** `Ctrl+R` then type partial command
- **Navigate Matches:** `Ctrl+R` repeatedly to cycle through matches
- **Edit Before Execute:** Arrow keys to edit recalled command
- **Cancel Search:** `Ctrl+G` to cancel without executing

## Configuration

### History Size

Rustyline default: 100 commands

To change (requires code modification):
```rust
editor.set_max_history_size(500)?;  // Keep 500 commands
```

### History File Location

Default: `~/.guestctl/history/`

To change (requires code modification):
```rust
// In src/cli/interactive.rs - get_history_dir()
let history_dir = home.join(".config").join("guestctl").join("history");
```

## Troubleshooting

### History Not Saving

**Problem:** Commands don't persist after exit

**Solutions:**
1. Check directory permissions:
   ```bash
   ls -ld ~/.guestctl/history/
   # Should be writable by your user
   ```

2. Check disk space:
   ```bash
   df -h ~
   ```

3. Check for error messages when exiting:
   - Warning messages indicate write failures

### History File Corruption

**Problem:** Interactive mode fails to load history

**Solution:**
```bash
# Backup corrupted file
mv ~/.guestctl/history/guestctl-*.history \
   ~/.guestctl/history/backup/

# Start fresh (history auto-creates new file)
guestctl interactive vm.qcow2
```

### Multiple Disk Paths Point to Same Disk

**Problem:** Symlinks or different paths to same disk create separate histories

**Solution:** Use consistent path:
```bash
# Always use absolute path
guestctl interactive /vms/prod-web-01.qcow2

# Or always use relative path
cd /vms && guestctl interactive prod-web-01.qcow2
```

## Privacy & Security

### Sensitive Commands

History files store commands in plain text. If working with sensitive data:

**Option 1 - Clear After Session:**
```bash
guestctl interactive secure-vm.qcow2
# ... do work ...
guestctl> exit

# Immediately clear
rm ~/.guestctl/history/guestctl-*.history
```

**Option 2 - Disable for Specific Session:**
```bash
# Start with read-only home directory mount
HOME=/tmp/readonly-home guestctl interactive vm.qcow2
# History won't save (directory not writable)
```

**Option 3 - Use Batch Mode Instead:**
```bash
# For sensitive workflows, use script files
guestctl script secure-vm.qcow2 workflow.gk
# No history saved
```

### Credentials in History

**Never type credentials in interactive mode!** Use:
- Environment variables
- Batch scripts with redacted commands
- Alternative authentication methods

## Comparison with Bash History

| Feature | GuestCtl | Bash |
|---------|----------|------|
| Per-context history | ✅ Yes (per-disk) | ❌ No (global) |
| Auto-save on exit | ✅ Yes | ⚠️ Optional |
| Search (Ctrl+R) | ✅ Yes | ✅ Yes |
| Navigation (↑/↓) | ✅ Yes | ✅ Yes |
| Storage format | Plain text | Plain text |
| Max size | 100 (default) | 500+ (default) |

## Integration with Workflows

### CI/CD Pipelines

History is useful for development but not CI/CD:
```yaml
# GitHub Actions - no history needed
- name: Inspect VM
  run: |
    # Use batch mode for deterministic execution
    guestctl script vm.qcow2 inspect.gk
```

### Team Knowledge Sharing

Extract common patterns:
```bash
# Collect useful commands
cat ~/.guestctl/history/guestctl-*.history | \
  sort | uniq -c | sort -rn | head -20 > common-commands.txt

# Share with team
# Add to project wiki or documentation
```

### Training & Onboarding

New team members can learn from history:
```bash
# Show common inspection workflow
cat ~/.guestctl/history/guestctl-*.history | head -10
# Example output:
# mount /dev/sda1 /
# filesystems
# packages | grep kernel
# services | grep running
# users
```

## Future Enhancements

Planned improvements:
- Configurable history size
- History statistics (most used commands)
- Shared team history (optional)
- History export/import
- Command timestamps
- Search with regex patterns

## See Also

- [Interactive Mode Guide](../user-guides/interactive-mode.md)
- [CLI Guide](../user-guides/cli-guide.md)

---

**Updated:** 2026-01-24
**GuestCtl Version:** 0.3.0+
