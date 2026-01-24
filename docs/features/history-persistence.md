# Command History Persistence Guide

GuestKit's interactive mode now automatically saves your command history across sessions, making it easy to repeat previous operations and explore VMs more efficiently.

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
- **Directory:** `~/.guestkit/history/`
- **Format:** `guestkit-{hash}.history`
- **Encoding:** Plain text, one command per line

## Usage

### Basic Usage

Just use interactive mode normally - history is saved automatically:

```bash
# Start interactive session
guestkit interactive vm.qcow2

guestkit> mount /dev/sda1 /
guestkit> packages
guestkit> services
guestkit> exit

# Start again - your history is preserved!
guestkit interactive vm.qcow2

guestkit> # Press ↑ to see "services"
guestkit> # Press ↑ again to see "packages"
guestkit> # Press ↑ again to see "mount /dev/sda1 /"
```

### Reverse Search (Ctrl+R)

Search through history interactively:

```bash
guestkit> # Press Ctrl+R
(reverse-i-search)`pack': packages

# Type more to narrow search
(reverse-i-search)`packages': packages grep nginx

# Press Enter to execute or Ctrl+R again for previous match
```

### History Per Disk

Each disk image has its own history file:

```bash
# Working with production VM
guestkit interactive prod-web-01.qcow2
guestkit> mount /dev/sda1 /
guestkit> packages grep nginx
guestkit> exit

# Working with staging VM (different history)
guestkit interactive staging-web-01.qcow2
guestkit> mount /dev/vda1 /
guestkit> services
guestkit> exit

# Return to production - original history preserved
guestkit interactive prod-web-01.qcow2
guestkit> # Press ↑ to see "packages grep nginx"
```

## How It Works

### History Files

History files are stored in `~/.guestkit/history/` with unique names based on the disk path:

```
~/.guestkit/history/
├── guestkit-a1b2c3d4e5f6g7h8.history  # prod-web-01.qcow2
├── guestkit-9i8h7g6f5e4d3c2b.history  # staging-web-01.qcow2
└── guestkit-1a2b3c4d5e6f7g8h.history  # dev-db-01.qcow2
```

The hash is generated from the disk path, ensuring:
- Same disk = same history file
- Different disks = different history files
- Disk path changes = new history file

### Automatic Loading

When you start interactive mode:
1. GuestKit computes hash of disk path
2. Checks for existing history file
3. Loads history if file exists
4. Shows message: "→ Loaded command history"

### Automatic Saving

When you exit (via `exit`, `quit`, or Ctrl+D):
1. GuestKit saves all commands from session
2. Merges with existing history
3. Writes to disk-specific history file
4. Silent unless error occurs

## Advanced Usage

### Manual History Management

View history file directly:
```bash
# Find your history file
ls -lah ~/.guestkit/history/

# View contents
cat ~/.guestkit/history/guestkit-*.history

# Count commands
wc -l ~/.guestkit/history/guestkit-*.history
```

### Clear History for Specific Disk

```bash
# Remove specific history file
rm ~/.guestkit/history/guestkit-a1b2c3d4e5f6g7h8.history

# Or clear all history
rm -rf ~/.guestkit/history/
```

### Export History

```bash
# Copy history for backup
cp ~/.guestkit/history/guestkit-*.history ~/backups/

# Share common commands with team
cat ~/.guestkit/history/guestkit-*.history | \
  grep "^mount\|^packages\|^services" > team-workflow.txt
```

## Tips & Tricks

### Efficient Workflows

**1. Build Custom Inspection Sequences:**
```bash
guestkit> mount /dev/sda1 /
guestkit> packages | grep apache
guestkit> services | grep httpd
guestkit> cat /etc/httpd/conf/httpd.conf
guestkit> exit

# Next time: Just use ↑ to replay entire sequence!
```

**2. Debug Iteratively:**
```bash
guestkit> find /var/log
guestkit> cat /var/log/messages | grep error
guestkit> cat /var/log/syslog | grep failed
# Each refinement is saved for next session
```

**3. Create Reusable Patterns:**
```bash
# First session - explore and refine
guestkit> packages | grep -i sec
guestkit> packages | grep -i audit
guestkit> packages | grep -i firewall

# Later sessions - reuse best pattern
guestkit> # Press Ctrl+R, type "packages", select best one
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

Default: `~/.guestkit/history/`

To change (requires code modification):
```rust
// In src/cli/interactive.rs - get_history_dir()
let history_dir = home.join(".config").join("guestkit").join("history");
```

## Troubleshooting

### History Not Saving

**Problem:** Commands don't persist after exit

**Solutions:**
1. Check directory permissions:
   ```bash
   ls -ld ~/.guestkit/history/
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
mv ~/.guestkit/history/guestkit-*.history \
   ~/.guestkit/history/backup/

# Start fresh (history auto-creates new file)
guestkit interactive vm.qcow2
```

### Multiple Disk Paths Point to Same Disk

**Problem:** Symlinks or different paths to same disk create separate histories

**Solution:** Use consistent path:
```bash
# Always use absolute path
guestkit interactive /vms/prod-web-01.qcow2

# Or always use relative path
cd /vms && guestkit interactive prod-web-01.qcow2
```

## Privacy & Security

### Sensitive Commands

History files store commands in plain text. If working with sensitive data:

**Option 1 - Clear After Session:**
```bash
guestkit interactive secure-vm.qcow2
# ... do work ...
guestkit> exit

# Immediately clear
rm ~/.guestkit/history/guestkit-*.history
```

**Option 2 - Disable for Specific Session:**
```bash
# Start with read-only home directory mount
HOME=/tmp/readonly-home guestkit interactive vm.qcow2
# History won't save (directory not writable)
```

**Option 3 - Use Batch Mode Instead:**
```bash
# For sensitive workflows, use script files
guestkit script secure-vm.qcow2 workflow.gk
# No history saved
```

### Credentials in History

**Never type credentials in interactive mode!** Use:
- Environment variables
- Batch scripts with redacted commands
- Alternative authentication methods

## Comparison with Bash History

| Feature | GuestKit | Bash |
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
    guestkit script vm.qcow2 inspect.gk
```

### Team Knowledge Sharing

Extract common patterns:
```bash
# Collect useful commands
cat ~/.guestkit/history/guestkit-*.history | \
  sort | uniq -c | sort -rn | head -20 > common-commands.txt

# Share with team
# Add to project wiki or documentation
```

### Training & Onboarding

New team members can learn from history:
```bash
# Show common inspection workflow
cat ~/.guestkit/history/guestkit-*.history | head -10
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

- [Interactive Mode Guide](./INTERACTIVE_MODE.md)
- [Batch Mode Guide](../examples/batch/README.md)
- [CLI Guide](./CLI_GUIDE.md)

---

**Updated:** 2026-01-24
**GuestKit Version:** 0.3.0+
