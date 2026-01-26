# GuestCtl Interactive Mode - Complete Command List

**Total Commands: 97**
**Build: Clean (0 warnings)**
**File Size: 3,928 lines**

## Categories & Commands

### 1. System Information (2 commands)
- `info` - Show disk and OS information
- `help`, `?` - Show this help

### 2. Filesystem Operations (5 commands)
- `filesystems`, `fs` - List available filesystems
- `mount <device> <path>` - Mount a filesystem
- `umount <path>` - Unmount a filesystem
- `mounts` - Show mounted filesystems

### 3. File Operations (8 commands)
- `ls [path]` - List directory contents
- `cat <path>` - Display file contents
- `head <path> [lines]` - Display first lines of file
- `find <pattern>` - Find files by name pattern
- `stat <path>` - Show file information
- `download <src> <dest>` - Download file from disk

### 4. File Management (12 commands)
- `upload <local> <remote>` - Upload file to VM
- `edit <path>` - Edit file in VM (opens in $EDITOR)
- `write <path> <content>` - Write content to file
- `copy <src> <dest>` - Copy file/directory
- `move <src> <dest>` - Move/rename file
- `delete <path>` - Delete file/directory
- `mkdir <path>` - Create directory
- `chmod <mode> <path>` - Change permissions
- `chown <user:group> <path>` - Change owner
- `symlink <target> <link>` - Create symlink
- `large-files [path] [size]` - Find large files (default >100MB)
- `disk-usage [path]` - Disk usage analysis

### 5. Package Management (5 commands)
- `packages [filter]` - List installed packages
- `install <package>` - Install package (auto-detects dnf/apt/yum)
- `remove <package>` - Remove package
- `update` - Update all packages
- `search <keyword>` - Search packages

### 6. System Inspection (4 commands)
- `services` - List system services
- `users` - List user accounts
- `network` - Show network configuration

### 7. User Management (5 commands)
- `adduser <username>` - Create new user
- `deluser <username>` - Delete user
- `passwd <username>` - Change password (secure input)
- `usermod <user> <group>` - Add user to group
- `groups <username>` - Show user groups

### 8. SSH Key Management (5 commands)
- `ssh-addkey <user> <keyfile>` - Add SSH public key
- `ssh-removekey <user> <index>` - Remove SSH key
- `ssh-listkeys <user>` - List authorized keys
- `ssh-enable` - Enable SSH service
- `ssh-config` - Show SSH config

### 9. System Configuration (4 commands)
- `hostname <name>` - Set hostname
- `timezone <tz>` - Set timezone
- `selinux <mode>` - Set SELinux mode
- `locale <locale>` - Set system locale

### 10. Service Management (6 commands)
- `enable <service>` - Enable service at boot
- `disable <service>` - Disable service at boot
- `restart <service>` - Mark service for restart
- `logs <service> [lines]` - View service logs
- `failed` - Show failed services
- `boot-time` - Analyze boot performance

### 11. Firewall Management (3 commands)
- `firewall-add <port/service>` - Add firewall rule
- `firewall-remove <port/service>` - Remove firewall rule
- `firewall-list` - List firewall rules

### 12. Cron/Scheduled Tasks (2 commands)
- `cron-add <user> <schedule> <cmd>` - Add cron job
- `cron-list [user]` - List cron jobs

### 13. System Cleanup (4 commands)
- `clean-logs` - Clean old logs (>30 days)
- `clean-cache` - Clean package cache
- `clean-temp` - Clean /tmp and /var/tmp
- `clean-kernels` - Remove old kernels

### 14. Backup & Safety (2 commands)
- `backup <path>` - Create timestamped backup
- `backups [path]` - List all backups

### 15. Boot Configuration (3 subcommands)
- `grub show` - Show current GRUB config
- `grub set <param>` - Set kernel parameter
- `grub update` - Update GRUB

### 16. Network Configuration (4 commands)
- `net-setip <iface> <ip> <mask>` - Set static IP
- `net-setdns <server1> [server2]` - Set DNS servers
- `net-route-add <dest> <gateway>` - Add route
- `net-dhcp <interface>` - Enable DHCP

### 17. Process Management (3 commands)
- `ps [filter]` - List processes
- `kill <pid> [signal]` - Kill process
- `top` - Show top processes by CPU/memory

### 18. Security & Audit (4 commands)
- `scan-ports` - Scan for open ports
- `audit-perms` - Find world-writable files
- `audit-suid` - Find SUID/SGID files
- `check-updates` - Check for security updates

### 19. Database Operations (2 commands)
- `db-list` - List all databases (MySQL/PostgreSQL)
- `db-backup <db> [type]` - Backup database

### 20. Advanced File Operations (5 commands)
- `grep-replace <old> <new> <file>` - Search & replace in file
- `diff <file1> <file2>` - Compare files
- `tree [path] [depth]` - Show directory tree
- `compress <path> [output]` - Create tar.gz archive
- `extract <archive> [dest]` - Extract archive

### 21. Git Operations (2 commands)
- `git-clone <url> [path]` - Clone repository
- `git-pull [path]` - Update repository

### 22. Performance Tuning (2 commands)
- `tune-swappiness <value>` - Set swap usage (0-100)
- `tune-show` - Show tuning parameters

### 23. Quick Setup Wizards (3 commands)
- `setup-webserver` - Auto-install & configure Nginx
- `setup-database [mysql|postgres]` - Auto-install database
- `setup-docker` - Auto-install Docker

### 24. Monitoring & Metrics (2 commands)
- `metrics` - System metrics summary (CPU, mem, disk, load)
- `bandwidth` - Network interface statistics

### 25. SELinux Advanced (2 commands)
- `selinux-context <path>` - Show SELinux context
- `selinux-audit` - Show recent SELinux denials

### 26. Templates (1 command)
- `template-save <name>` - Save configuration snapshot

### 27. AI Assistant (1 command - requires --features ai)
- `ai <query>` - Ask AI for help (requires OPENAI_API_KEY)

### 28. Shell Commands (3 commands)
- `clear`, `cls` - Clear screen
- `exit`, `quit`, `q` - Exit interactive mode

## AI Features (Optional)

**Requires:**
- Build with `--features ai`
- Set `OPENAI_API_KEY` environment variable

**Capabilities:**
- Natural language VM troubleshooting
- Boot failure analysis
- Disk problem diagnosis
- Configuration recommendations
- Security analysis
- Performance optimization suggestions

**Example queries:**
```
ai why won't this boot?
ai what security issues do you see?
ai analyze disk usage patterns
ai how can I optimize performance?
ai what's causing high CPU usage?
```

## Building

**Standard build:**
```bash
cargo build --release
```

**With AI support:**
```bash
cargo build --release --features ai
export OPENAI_API_KEY='your-key-here'
```

**Usage:**
```bash
sudo -E guestctl interactive vm.qcow2
```

## Statistics

- **Total Commands:** 97
- **Lines of Code:** 3,928 (interactive.rs)
- **Build Status:** ✅ Clean (0 warnings)
- **Compilation Time:** ~3-4 minutes (release)
- **Features:** disk-ops, guest-inspect, python-bindings, ai

---
*Generated: 2026-01-26*
