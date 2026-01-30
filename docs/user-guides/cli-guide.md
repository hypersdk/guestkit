# GuestCtl CLI Guide (v0.3.1)

`guestctl` is a command-line tool for inspecting and manipulating virtual machine disk images without mounting them.

## What's New in v0.3.1

- **🎯 Killer Summary View**: Quick boxed summary showing OS, architecture, hostname at a glance
- **🪟 Windows Registry Parsing**: Full Windows version detection via direct registry access
- **🔄 VM Migration Support**: Universal fstab/crypttab rewriter for cross-platform migration
- **💾 Smart LVM Cleanup**: Automatic LVM volume group cleanup during shutdown
- **🔄 Loop Device Primary**: Built-in support for RAW/IMG/ISO formats without kernel modules

## Installation

```bash
# Build from source
cargo build --release

# Binary location
./target/release/guestctl

# Install globally (optional)
cargo install --path .
```

## Quick Start

```bash
# Inspect a disk image
sudo guestctl inspect ubuntu.qcow2

# List filesystems
sudo guestctl filesystems ubuntu.qcow2

# List packages
sudo guestctl packages ubuntu.qcow2

# Copy a file
sudo guestctl cp ubuntu.qcow2:/etc/passwd ./passwd

# List directory
sudo guestctl ls ubuntu.qcow2 /etc

# Read file
sudo guestctl cat ubuntu.qcow2 /etc/hostname
```

## Supported Disk Formats

guestctl automatically detects disk image formats and uses the optimal mounting method:

### Loop Device (Primary) - Default for Common Formats

**Formats:** RAW, IMG, ISO
**Performance:** Fast (~100ms setup)
**Dependencies:** None (built into Linux kernel)

```bash
# These use loop device automatically (fast path)
guestctl inspect disk.raw
guestctl inspect ubuntu-22.04.img
guestctl inspect debian.iso
```

### NBD Device (Fallback) - For Advanced Formats

**Formats:** QCOW2, VMDK, VDI, VHD
**Performance:** Slower (~500ms setup)
**Dependencies:** NBD module (auto-loaded), qemu-nbd

```bash
# These use NBD device automatically (advanced formats)
guestctl inspect vm.qcow2
guestctl inspect windows.vmdk
guestctl inspect virtualbox.vdi
```

### Format Conversion Tips

For better performance with repeated inspections, convert QCOW2 to RAW:

```bash
# Convert once
qemu-img convert -O raw vm.qcow2 vm.raw

# Inspect multiple times (fast)
guestctl inspect vm.raw
guestctl packages vm.raw
guestctl filesystems vm.raw
```

### Verbose Mode

Use `--trace` to see which method is being used:

```bash
guestctl --trace inspect disk.raw
# Output: "guestfs: using loop device for raw disk format"

guestctl --trace inspect disk.qcow2
# Output: "guestfs: using NBD for qcow2/vmdk/vdi/vhd disk format"
```

## Commands

### `inspect` - OS Information

Detect and display operating system information from a disk image.

**Usage:**
```bash
guestctl inspect [OPTIONS] <DISK>
```

**Options:**
- `-j, --json` - Output as JSON for scripting

**Examples:**
```bash
# Human-readable output
sudo guestctl inspect ubuntu.qcow2

# JSON output for jq processing
sudo guestctl inspect --json ubuntu.qcow2 | jq '.operating_systems[0].distro'
```

**Output:**
```
=== Disk Image: ubuntu.qcow2 ===

Found 1 operating system(s):

OS #1
  Root device: /dev/sda2
  Type: linux
  Distribution: ubuntu
  Version: 22.4
  Product: Ubuntu 22.04 LTS
  Hostname: webserver-01
  Architecture: x86_64
  Package format: deb
  Package management: apt
```

**JSON Output:**
```json
{
  "disk": "ubuntu.qcow2",
  "os_count": 1,
  "operating_systems": [
    {
      "root": "/dev/sda2",
      "type": "linux",
      "distro": "ubuntu",
      "major_version": 22,
      "minor_version": 4,
      "product_name": "Ubuntu 22.04 LTS",
      "hostname": "webserver-01",
      "arch": "x86_64",
      "package_format": "deb",
      "package_management": "apt"
    }
  ]
}
```

---

### `diff` - Compare Two Disk Images

Compare two disk images to identify configuration changes, package differences, service changes, and more.

**Usage:**
```bash
guestctl diff [OPTIONS] <IMAGE1> <IMAGE2>
```

**Options:**
- `-o, --output <FORMAT>` - Output format: text (default), json, yaml
- `-v, --verbose` - Verbose output

**Examples:**
```bash
# Compare two versions of a VM
sudo guestctl diff vm-before.qcow2 vm-after.qcow2

# JSON output for automation
sudo guestctl diff vm-before.qcow2 vm-after.qcow2 --output json

# YAML output
sudo guestctl diff vm-before.qcow2 vm-after.qcow2 --output yaml
```

**Output:**
```
=== OS Differences ===
  hostname: fedora-server → fedora-prod
  package_count: 1247 → 1250

=== Package Differences ===
  Added (3):
    + kernel: 6.6.8-200.fc39
    + nginx-1.24.0
    + certbot-2.7.4

  Removed (1):
    - apache2-2.4.57

=== Service Differences ===
  Enabled:
    + nginx.service
  Disabled:
    - apache2.service

=== User Differences ===
  Added: webadmin

=== Network Differences ===
  eth0_ip: 192.168.1.100 → 192.168.1.150

=== Configuration Differences ===
  timezone: America/New_York → Europe/London
```

**JSON Output:**
```json
{
  "os_changes": [
    {
      "field": "hostname",
      "old_value": "fedora-server",
      "new_value": "fedora-prod"
    }
  ],
  "package_changes": {
    "added": ["kernel: 6.6.8-200.fc39", "nginx-1.24.0"],
    "removed": ["apache2-2.4.57"],
    "updated": []
  },
  "service_changes": {
    "enabled": ["nginx.service"],
    "disabled": ["apache2.service"]
  },
  "user_changes": {
    "added": ["webadmin"],
    "removed": []
  },
  "network_changes": [
    {
      "field": "eth0_ip",
      "old_value": "192.168.1.100",
      "new_value": "192.168.1.150"
    }
  ],
  "config_changes": []
}
```

**Use Cases:**
- Track configuration drift between VM snapshots
- Verify migration changes
- Audit updates and modifications
- Generate change reports for compliance

---

### `compare` - Batch VM Comparison

Compare multiple VMs against a baseline to identify deviations and ensure consistency.

**Usage:**
```bash
guestctl compare <BASELINE> <IMAGES>...
```

**Options:**
- `-v, --verbose` - Verbose output

**Examples:**
```bash
# Compare production VMs against golden image
sudo guestctl compare golden-image.qcow2 prod-vm1.qcow2 prod-vm2.qcow2 prod-vm3.qcow2

# Compare all VMs in a directory
sudo guestctl compare baseline.qcow2 /var/lib/libvirt/images/*.qcow2
```

**Output:**
```
=== Comparison Report ===
Baseline: golden-image.qcow2

Comparing 3 VM(s):
  prod-vm1.qcow2
  prod-vm2.qcow2
  prod-vm3.qcow2

=== Summary ===
                    Baseline    prod-vm1    prod-vm2    prod-vm3
OS Version          39.0        39.0        40.0 ⚠     39.0
Hostname            baseline    web1        web2        db1
Package Count       1247        1250        1248        1189 ⚠
SSH Root Login      no          no          YES ⚠       no
Firewall            enabled     enabled     DISABLED ⚠  enabled

=== Compliance Issues ===
prod-vm2:
  ⚠ OS Version: 40.0 (expected: 39.0)
  ⚠ SSH root login: enabled (should be disabled)
  ⚠ Firewall: disabled (should be enabled)

prod-vm3:
  ⚠ Package count: 1189 (58 fewer than baseline)

=== Recommendations ===
- Review prod-vm2 configuration (3 issues detected)
- Investigate missing packages on prod-vm3
```

**Use Cases:**
- Validate VM fleet consistency
- Detect configuration drift across multiple VMs
- Compliance auditing against golden images
- Quality assurance for VM templates

---

### Inspection Profiles

Use specialized profiles for focused inspection:

**Available Profiles:**
- `security` - Security audit and hardening recommendations
- `migration` - Migration planning and compatibility analysis
- `performance` - Performance tuning opportunities

**Usage:**
```bash
guestctl inspect [OPTIONS] --profile <PROFILE> <IMAGE>
```

**Security Profile Example:**
```bash
sudo guestctl inspect --profile security webserver.qcow2
```

**Output:**
```
Profile: Security Audit

━━━ SSH Configuration ━━━
  ✗ PermitRootLogin: yes (CRITICAL - should be 'no')
  ✓ PasswordAuthentication: no
  ✓ SSH Port: 22

━━━ User Security ━━━
  ⚠ Multiple UID 0 Users: 2 users with UID 0 detected
  ✓ Disabled Logins: 15 system accounts properly disabled

━━━ Firewall ━━━
  ✗ Firewall Status: disabled (CRITICAL)

━━━ Mandatory Access Control ━━━
  ⚠ SELinux: permissive (should be enforcing)

━━━ Services ━━━
  ⚠ Risky Services: telnet detected (HIGH RISK)
  ✓ SSH: properly configured

━━━ SSL/TLS Certificates ━━━
  ℹ Certificates: 3 certificate(s) found in /etc/ssl/certs

Overall Risk Level: HIGH
Critical Issues: 2
Warnings: 3
```

**Migration Profile Example:**
```bash
sudo guestctl inspect --profile migration old-server.qcow2 --output json > migration-plan.json
```

**Performance Profile Example:**
```bash
sudo guestctl inspect --profile performance database.qcow2
```

**Output:**
```
Profile: Performance Tuning

━━━ Kernel Parameters ━━━
  ℹ Kernel Parameters: 127 parameters configured
  ⚠ vm.swappiness: Not configured (consider setting to 10)
  ℹ net.core.somaxconn: 4096

━━━ Swap Configuration ━━━
  ℹ Swap Devices: 1 swap device(s): /dev/sda3
  ✓ Swappiness: vm.swappiness = 10

━━━ Disk I/O Configuration ━━━
  ℹ Filesystem Mounts: 4 mount points in fstab
  ℹ /: /dev/mapper/vg0-root (ext4) - check for noatime, nodiratime options

━━━ Network Tuning ━━━
  ⚠ net.core.rmem_max: Not tuned (consider optimizing for high-throughput)
  ℹ eth0: DHCP configuration

━━━ Services & Resource Usage ━━━
  ℹ Enabled Services: 42 services enabled (review for unnecessary services)
  ℹ postgresql: Resource-intensive service detected - ensure proper allocation

Review tuning opportunities to optimize system performance.
```

---

### `filesystems` - List Storage

List all devices, partitions, and filesystems in a disk image.

**Usage:**
```bash
guestctl filesystems [OPTIONS] <DISK>
```

**Options:**
- `-d, --detailed` - Show detailed information (UUIDs, partition numbers)

**Examples:**
```bash
# Basic listing
sudo guestctl filesystems ubuntu.qcow2

# Detailed information
sudo guestctl filesystems --detailed ubuntu.qcow2
```

**Output:**
```
=== Devices ===
/dev/sda
  Size: 21474836480 bytes (21.47 GB)
  Partition table: gpt

=== Partitions ===
/dev/sda1
  Size: 536870912 bytes (0.54 GB)
  Filesystem: vfat
  Label: EFI

/dev/sda2
  Size: 20937965568 bytes (20.94 GB)
  Filesystem: ext4
  Label: rootfs
  UUID: a1b2c3d4-e5f6-7890-abcd-ef1234567890

=== LVM Volume Groups ===
(none)

=== LVM Logical Volumes ===
(none)
```

---

### `packages` - List Installed Software

List all installed packages from a disk image.

**Usage:**
```bash
guestctl packages [OPTIONS] <DISK>
```

**Options:**
- `-f, --filter <NAME>` - Filter packages by name
- `-l, --limit <N>` - Limit number of results
- `-j, --json` - Output as JSON

**Examples:**
```bash
# List all packages
sudo guestctl packages ubuntu.qcow2

# Find nginx packages
sudo guestctl packages --filter nginx ubuntu.qcow2

# Show first 20 packages
sudo guestctl packages --limit 20 ubuntu.qcow2

# JSON output
sudo guestctl packages --json ubuntu.qcow2 | jq '.packages[] | select(.name | contains("kernel"))'
```

**Output:**
```
Found 1847 package(s)

Package                                  Version              Release
----------------------------------------------------------------------------------
accountsservice                          0.6.55               0ubuntu12
adduser                                  3.118                ubuntu2
apache2                                  2.4.52               1ubuntu4
...
```

---

### `cp` - Copy Files

Copy files from a disk image to the local filesystem.

**Usage:**
```bash
guestctl cp <SOURCE> <DEST>
```

**Source Format:** `disk.img:/path/to/file`

**Examples:**
```bash
# Copy passwd file
sudo guestctl cp ubuntu.qcow2:/etc/passwd ./passwd

# Copy nginx config
sudo guestctl cp ubuntu.qcow2:/etc/nginx/nginx.conf ./nginx.conf

# Copy log file
sudo guestctl cp ubuntu.qcow2:/var/log/syslog ./syslog
```

**Output:**
```
✓ Copied ubuntu.qcow2:/etc/passwd -> ./passwd
```

---

### `ls` - List Directory

List files and directories inside a disk image.

**Usage:**
```bash
guestctl ls [OPTIONS] <DISK> [PATH]
```

**Options:**
- `-l, --long` - Long listing format (like `ls -l`)

**Examples:**
```bash
# List root directory
sudo guestctl ls ubuntu.qcow2 /

# List /etc
sudo guestctl ls ubuntu.qcow2 /etc

# Long format
sudo guestctl ls --long ubuntu.qcow2 /etc
```

**Output (basic):**
```
bin
boot
dev
etc
home
lib
...
```

**Output (long format):**
```
drwxr-xr-x   2 root root     4096 Jan 15 10:30 bin
drwxr-xr-x   3 root root     4096 Jan 15 10:31 boot
drwxr-xr-x  16 root root     3200 Jan 22 08:15 dev
drwxr-xr-x  95 root root     4096 Jan 22 09:22 etc
...
```

---

### `cat` - Read Files

Display the contents of a file from a disk image.

**Usage:**
```bash
guestctl cat <DISK> <PATH>
```

**Examples:**
```bash
# Read hostname
sudo guestctl cat ubuntu.qcow2 /etc/hostname

# Read OS release
sudo guestctl cat ubuntu.qcow2 /etc/os-release

# Read systemd service
sudo guestctl cat ubuntu.qcow2 /etc/systemd/system/myapp.service
```

**Output:**
```
$ sudo guestctl cat ubuntu.qcow2 /etc/hostname
webserver-01

$ sudo guestctl cat ubuntu.qcow2 /etc/os-release
NAME="Ubuntu"
VERSION="22.04 LTS (Jammy Jellyfish)"
ID=ubuntu
ID_LIKE=debian
...
```

---

### `explore` - Interactive File Browser 🆕

Launch a visual, interactive file browser for exploring VM filesystems with rich features.

**Usage:**
```bash
guestctl explore [OPTIONS] <DISK> [PATH]
```

**Options:**
- None - the explorer has its own interactive keyboard controls

**Examples:**
```bash
# Launch explorer at root directory
sudo guestctl explore ubuntu.qcow2

# Start at specific directory
sudo guestctl explore ubuntu.qcow2 /var/log

# Explore from interactive shell
guestctl interactive ubuntu.qcow2
guestctl> explore /etc
```

**Features:**
```
┌─────────────────────────────────────────────────────────┐
│ 📍 Path: /var/log  📊 Items: 42                         │
├─────────────────────────────────────────────────────────┤
│ ▸ 📁 ..                                           <DIR> │
│   📁 audit                                        <DIR> │
│   📁 journal                                      <DIR> │
│ ▸ 📄 syslog                                    12.4 MB │
│   📄 syslog.1                                   8.2 MB │
│   📦 syslog.2.gz                                2.1 MB │
│                                                         │
│ [v] Preview  [i] Info  [/] Filter  [q] Quit           │
└─────────────────────────────────────────────────────────┘
```

**Interactive Features:**
- **Visual navigation** - Color-coded files with emoji icons for file types
- **File preview** (press `v`) - View file contents with line numbers
- **File information** (press `i`) - Detailed metadata, size, permissions
- **Real-time filtering** (press `/`) - Live search as you type
- **Hidden files toggle** (press `.`) - Show/hide dotfiles
- **Smart sorting** (press `s`) - By name, size, or modification time
- **Vim-like navigation** - j/k or arrow keys

**Keyboard Shortcuts:**

| Key | Action |
|-----|--------|
| `↑↓` / `j k` | Navigate up/down |
| `Enter` | Enter directory or open file |
| `Backspace` | Go to parent directory |
| `v` | Preview file contents (up to 1MB) |
| `i` | Show file information |
| `/` | Start real-time filter |
| `.` | Toggle hidden files |
| `s` | Cycle sort modes (name/size/date) |
| `?` | Show help overlay |
| `q` | Quit explorer |

**Output:**
The explorer provides an interactive terminal UI. File type icons:
- 📁 Directories (blue)
- 💻 Source code (yellow) - .rs, .py, .js, .ts, etc.
- ⚙️ Config files (cyan) - .json, .yaml, .toml, .xml
- 🔧 Scripts (green) - .sh, .bash
- 📦 Archives (red) - .zip, .tar, .gz
- 🖼️ Images (magenta) - .jpg, .png, .gif
- 📄 Documents (white) - .txt, .md, .log

**Availability:**
- Direct CLI: `guestctl explore <disk> [path]`
- Interactive shell: `explore` or `ex` command
- TUI mode: Files view (accessible via Tab/number keys)

**Documentation:**
- Quick start: EXPLORE-QUICKSTART.md
- Complete guide: EXPLORE-COMMAND.md
- Full overview: EXPLORE-COMPLETE-SUMMARY.md

---

## Advanced Analysis Commands

GuestKit includes 28 advanced commands for forensics, security, threat hunting, compliance, and AI-powered analysis.

### `timeline` - Forensic Timeline Analysis

Build a comprehensive forensic timeline from multiple data sources including files, packages, logs, and system events.

**Usage:**
```bash
guestctl timeline [OPTIONS] <IMAGE>
```

**Options:**
- `--start-time <TIME>` - Start time filter (ISO 8601 format)
- `--end-time <TIME>` - End time filter (ISO 8601 format)
- `-s, --sources <SOURCES>` - Comma-separated data sources (files, packages, logs, registry)
- `-f, --format <FORMAT>` - Output format: text, json, csv (default: text)

**Examples:**
```bash
# Build full timeline
sudo guestctl timeline disk.img

# Timeline for specific time range
sudo guestctl timeline --start-time 2024-01-01T00:00:00Z --end-time 2024-01-31T23:59:59Z disk.img

# Timeline from specific sources
sudo guestctl timeline -s files,packages,logs disk.img -f json

# Export to CSV for analysis
sudo guestctl timeline disk.img -f csv > timeline.csv
```

**Output:**
```
=== Forensic Timeline ===
Image: disk.img
Time Range: 2024-01-01 to 2024-01-31
Sources: files, packages, logs

2024-01-15 10:23:45  FILE_MODIFIED    /etc/ssh/sshd_config
2024-01-15 10:24:12  PACKAGE_INSTALL  nginx-1.24.0
2024-01-15 10:25:03  SERVICE_START    nginx.service
2024-01-15 14:32:10  USER_LOGIN       admin (192.168.1.50)
2024-01-16 02:15:22  FILE_CREATED     /var/www/html/index.html
2024-01-16 09:45:33  PACKAGE_UPDATE   kernel: 6.6.8 → 6.6.9

Total Events: 1,247
```

**Use Cases:**
- Incident response and forensic investigation
- Change tracking and auditing
- Compliance evidence collection
- Root cause analysis

---

### `fingerprint` - Disk Image Fingerprinting

Generate unique cryptographic fingerprints of disk images for integrity verification and deduplication.

**Usage:**
```bash
guestctl fingerprint [OPTIONS] <IMAGE>
```

**Options:**
- `-a, --algorithm <ALGO>` - Hash algorithm: sha256 (default), sha512, blake3
- `-d, --deep` - Deep fingerprinting (includes file content hashes)
- `-o, --output <FILE>` - Save fingerprint to file

**Examples:**
```bash
# Generate basic fingerprint
sudo guestctl fingerprint disk.img

# Deep fingerprint with SHA-512
sudo guestctl fingerprint --deep --algorithm sha512 disk.img

# Save fingerprint for later verification
sudo guestctl fingerprint disk.img -o disk.fingerprint
```

**Output:**
```
=== Disk Fingerprint ===
Image: disk.img
Algorithm: SHA-256

Disk Hash: 7a8f3e2b1c9d4f6a8e5b3c7d9f2a4e6b8c1d3f5a7e9b2c4d6f8a1e3c5b7d9f2a
Boot Sector: 2c4d6f8a1e3c5b7d9f2a4e6b8c1d3f5a7e9b2c4d6f8a1e3c5b7d9f2a4e6b8c1
Partition Table: 9f2a4e6b8c1d3f5a7e9b2c4d6f8a1e3c5b7d9f2a4e6b8c1d3f5a7e9b2c4d6f8

File Count: 12,847
Total Size: 21.5 GB
Fingerprint Generated: 2024-01-28 10:30:15 UTC
```

**Use Cases:**
- Verify image integrity after transfer
- Detect unauthorized modifications
- Deduplication in backup systems
- Chain of custody for forensics

---

### `drift` - Configuration Drift Detection

Detect configuration drift from a known baseline or policy.

**Usage:**
```bash
guestctl drift <BASELINE> <IMAGE>
```

**Options:**
- `-s, --severity <LEVEL>` - Minimum severity to report: low, medium, high, critical
- `-c, --categories <CATS>` - Categories to check: packages, services, users, network, files
- `--json` - Output as JSON

**Examples:**
```bash
# Detect all drift
sudo guestctl drift baseline.img production.img

# Only critical drift
sudo guestctl drift --severity critical baseline.img production.img

# Specific categories
sudo guestctl drift -c packages,services baseline.img production.img --json
```

**Output:**
```
=== Configuration Drift Report ===
Baseline: baseline.img
Target: production.img

CRITICAL (2):
  ⚠ SSH root login enabled (should be disabled)
  ⚠ Firewall disabled (should be enabled)

HIGH (3):
  ⚠ Unauthorized package: netcat-openbsd
  ⚠ User 'backdoor' added (not in baseline)
  ⚠ SELinux set to permissive (should be enforcing)

MEDIUM (7):
  ⚠ Package version drift: nginx 1.24.0 → 1.22.1 (downgrade)
  ⚠ Timezone changed: UTC → America/New_York
  ⚠ Service 'telnet' enabled

LOW (12):
  ℹ Hostname changed
  ℹ 58 package updates available

Total Drift Items: 24
Compliance Score: 67/100
```

---

### `intelligence` - Threat Intelligence Analysis

Scan disk images for Indicators of Compromise (IOCs) using threat intelligence feeds.

**Usage:**
```bash
guestctl intelligence [OPTIONS] <IMAGE>
```

**Options:**
- `--ioc-file <FILE>` - Custom IOC file (JSON/YAML)
- `--feed <FEED>` - Threat feed: alienvault, misp, custom
- `-t, --types <TYPES>` - IOC types: hash, ip, domain, url, registry
- `--output <FORMAT>` - Output format: text, json, stix

**Examples:**
```bash
# Scan with default feeds
sudo guestctl intelligence disk.img

# Use custom IOC file
sudo guestctl intelligence --ioc-file threats.json disk.img

# Check specific IOC types
sudo guestctl intelligence -t hash,ip disk.img --output json
```

**Output:**
```
=== Threat Intelligence Analysis ===
Image: disk.img
IOC Sources: AlienVault OTX, MISP
Scan Date: 2024-01-28 10:45:22 UTC

CRITICAL THREATS (2):
  🔴 File Hash Match: /tmp/malware.exe
     Hash: 5f8a3e2b1c9d4f6a8e5b3c7d9f2a4e6b
     Threat: Ransomware.WannaCry
     Confidence: 98%
     Source: AlienVault OTX

  🔴 IP Connection: 192.168.1.100 → 45.33.32.156
     Threat: C2 Server (APT28)
     Confidence: 95%
     Source: MISP Feed

HIGH RISK (5):
  🟠 Suspicious Domain: evil-domain.ru in browser history
  🟠 Registry Key: HKLM\SOFTWARE\Malware\Persistence

Total IOCs Found: 7
Risk Level: CRITICAL
Recommended Action: Immediate quarantine
```

---

### `secrets` - Exposed Secrets Scanner

Scan for exposed credentials, API keys, passwords, and other secrets in disk images.

**Usage:**
```bash
guestctl secrets [OPTIONS] <IMAGE>
```

**Options:**
- `-p, --patterns <FILE>` - Custom regex patterns file
- `-e, --exclude <PATHS>` - Exclude paths (comma-separated)
- `--mask` - Mask secrets in output
- `-f, --format <FORMAT>` - Output format: text, json

**Examples:**
```bash
# Scan for all secrets
sudo guestctl secrets disk.img

# Use custom patterns
sudo guestctl secrets --patterns custom-secrets.txt disk.img

# Mask secrets in output
sudo guestctl secrets --mask disk.img

# Exclude certain paths
sudo guestctl secrets -e /var/log,/tmp disk.img
```

**Output:**
```
=== Exposed Secrets Report ===
Image: disk.img

CRITICAL FINDINGS (4):

  🔴 AWS Access Key
     Location: /home/user/.bash_history
     Line: export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
     Risk: CRITICAL - Full AWS account access

  🔴 Private SSH Key
     Location: /var/www/.ssh/id_rsa
     Permissions: 644 (readable by all!)
     Risk: CRITICAL - Unauthorized access

  🔴 Database Password
     Location: /etc/app/config.yaml
     Context: db_password: SuperSecret123!
     Risk: HIGH - Database compromise

  🔴 API Token
     Location: /root/.env
     Context: GITHUB_TOKEN=ghp_1234567890abcdef
     Risk: HIGH - Code repository access

Total Secrets Found: 12
Critical: 4, High: 5, Medium: 3
Recommendation: Rotate all exposed credentials immediately
```

---

### `malware` - Malware Detection

Detect malware, rootkits, and suspicious files using signature and behavior-based analysis.

**Usage:**
```bash
guestctl malware [OPTIONS] <IMAGE>
```

**Options:**
- `-s, --signatures <FILE>` - Custom signature database
- `-d, --deep` - Deep scan (slower, more thorough)
- `--yara <RULES>` - YARA rules file
- `-q, --quarantine <DIR>` - Quarantine infected files

**Examples:**
```bash
# Quick malware scan
sudo guestctl malware disk.img

# Deep scan with YARA rules
sudo guestctl malware --deep --yara malware.yar disk.img

# Quarantine detected malware
sudo guestctl malware -q ./quarantine disk.img
```

**Output:**
```
=== Malware Scan Report ===
Image: disk.img
Scan Type: Deep
Signatures: 150,000+ (updated 2024-01-28)

INFECTED FILES (3):

  🦠 /tmp/cryptominer
     Type: Cryptocurrency Miner (XMRig variant)
     Severity: HIGH
     Hash: 3e5b2a8f9c1d4e6b7a8c3f5d2e9b4a6c
     Detection: Behavioral + Signature

  🦠 /usr/bin/.hidden/rootkit.ko
     Type: Kernel Rootkit
     Severity: CRITICAL
     Detection: Hidden file in system directory

  🦠 /home/user/Downloads/invoice.pdf.exe
     Type: Trojan Downloader
     Severity: HIGH
     Detection: Double extension obfuscation

SUSPICIOUS FILES (7):
  ⚠ /var/tmp/suspicious_script.sh (modified system file)
  ⚠ /etc/ld.so.preload (rootkit persistence method)

Total Files Scanned: 124,847
Infected: 3
Suspicious: 7
Clean: 124,837
Scan Duration: 45 seconds
```

---

### `hunt` - MITRE ATT&CK Threat Hunting

Hunt for evidence of tactics, techniques, and procedures (TTPs) based on the MITRE ATT&CK framework.

**Usage:**
```bash
guestctl hunt [OPTIONS] <IMAGE>
```

**Options:**
- `-t, --tactics <TACTICS>` - MITRE tactics to hunt (e.g., persistence, privilege-escalation)
- `-i, --techniques <IDS>` - Specific technique IDs (e.g., T1053, T1548)
- `--severity <LEVEL>` - Minimum severity: low, medium, high, critical
- `-f, --format <FORMAT>` - Output format: text, json, mitre-navigator

**Examples:**
```bash
# Hunt for all TTPs
sudo guestctl hunt disk.img

# Hunt for persistence techniques
sudo guestctl hunt -t persistence disk.img

# Hunt for specific techniques
sudo guestctl hunt -i T1053,T1548 disk.img

# Export to MITRE ATT&CK Navigator
sudo guestctl hunt disk.img -f mitre-navigator > attack-layer.json
```

**Output:**
```
=== MITRE ATT&CK Threat Hunt ===
Image: disk.img

DETECTIONS:

[CRITICAL] T1053 - Scheduled Task/Job
  Tactic: Persistence, Privilege Escalation
  Evidence: Suspicious cron job found
  Location: /etc/cron.d/malicious
  Command: */5 * * * * root /tmp/.hidden/backdoor.sh
  Confidence: 95%

[HIGH] T1548 - Abuse Elevation Control
  Tactic: Privilege Escalation, Defense Evasion
  Evidence: SUID binary in unusual location
  Location: /tmp/su
  Permissions: -rwsr-xr-x
  Confidence: 88%

[HIGH] T1070.004 - File Deletion
  Tactic: Defense Evasion
  Evidence: Log file tampering detected
  Modified: /var/log/auth.log (timestamps altered)
  Confidence: 92%

[MEDIUM] T1136 - Create Account
  Tactic: Persistence
  Evidence: New user account created
  Account: backdoor (UID: 1001)
  Created: 2024-01-15 02:34:12
  Confidence: 85%

Detections by Tactic:
  Persistence: 3
  Privilege Escalation: 2
  Defense Evasion: 2

Total Detections: 7
Risk Level: CRITICAL
```

---

### `reconstruct` - Forensic Incident Reconstruction

Reconstruct the sequence of events during a security incident or system failure.

**Usage:**
```bash
guestctl reconstruct [OPTIONS] <IMAGE>
```

**Options:**
- `--incident-time <TIME>` - Incident timestamp (ISO 8601)
- `--window <HOURS>` - Time window around incident (default: 24 hours)
- `--focus <AREAS>` - Focus areas: files, users, network, processes
- `-o, --output <FILE>` - Save reconstruction report

**Examples:**
```bash
# Reconstruct incident
sudo guestctl reconstruct --incident-time 2024-01-15T14:30:00Z disk.img

# 48-hour window with focus on network
sudo guestctl reconstruct --window 48 --focus network disk.img

# Save detailed report
sudo guestctl reconstruct disk.img -o incident-report.json
```

**Output:**
```
=== Incident Reconstruction ===
Image: disk.img
Incident Time: 2024-01-15 14:30:00 UTC
Window: ±24 hours

TIMELINE OF EVENTS:

2024-01-15 02:15:22  Initial Compromise
  → SSH brute force attack detected (192.168.1.100)
  → 1,247 failed login attempts for 'root'

2024-01-15 02:17:45  Successful Login
  → User 'admin' logged in from 45.33.32.156
  → Session established (PID: 8472)

2024-01-15 02:18:33  Privilege Escalation
  → Exploited CVE-2024-1234 (sudo vulnerability)
  → Gained root access

2024-01-15 02:22:10  Persistence Established
  → Created backdoor user 'sysupdate'
  → Added SSH key to /root/.ssh/authorized_keys
  → Installed cron job: /etc/cron.d/update

2024-01-15 14:25:00  Malicious Activity
  → Cryptocurrency miner started (PID: 9823)
  → CPU usage spiked to 100%

2024-01-15 14:30:00  Detection/Incident
  → System became unresponsive
  → Admin noticed in monitoring

2024-01-15 14:45:00  Attacker Actions
  → Log files cleared (/var/log/auth.log)
  → Miner process hidden (renamed to 'kworker')

ATTACK SUMMARY:
  Initial Access: SSH Brute Force
  Privilege Escalation: Sudo Vulnerability (CVE-2024-1234)
  Persistence: Backdoor account + SSH key + Cron job
  Impact: Cryptomining, Resource exhaustion
  Dwell Time: 12 hours 15 minutes

INDICATORS OF COMPROMISE:
  IP: 45.33.32.156
  User: sysupdate
  File: /tmp/.hidden/xmrig
  Cron: /etc/cron.d/update
```

---

### `verify` - Zero-Trust Verification

Perform continuous verification checks based on zero-trust security principles.

**Usage:**
```bash
guestctl verify [OPTIONS] <IMAGE>
```

**Options:**
- `-p, --policy <FILE>` - Verification policy file
- `-l, --level <LEVEL>` - Verification level: basic, standard, strict, paranoid
- `--trust-boundary <ZONES>` - Define trust boundaries
- `-f, --format <FORMAT>` - Output format: text, json, sarif

**Examples:**
```bash
# Standard verification
sudo guestctl verify disk.img

# Strict verification with custom policy
sudo guestctl verify --level strict --policy security-policy.yaml disk.img

# Paranoid mode (maximum checks)
sudo guestctl verify -l paranoid disk.img
```

**Output:**
```
=== Zero-Trust Verification ===
Image: disk.img
Policy: Default Security Policy
Level: Strict

IDENTITY VERIFICATION:
  ✓ All users have unique UIDs
  ✗ User 'backdoor' not in approved list [FAIL]
  ✓ No shared accounts detected
  ✗ 2 users with UID 0 (root equivalents) [WARN]

DEVICE VERIFICATION:
  ✓ Boot integrity maintained
  ✓ No unknown kernel modules
  ✗ Unsigned driver detected: /lib/modules/custom.ko [FAIL]

NETWORK VERIFICATION:
  ✓ No unauthorized listening ports
  ✗ Firewall disabled [CRITICAL]
  ✗ Outbound connection to untrusted IP: 45.33.32.156 [FAIL]

APPLICATION VERIFICATION:
  ✓ All packages from trusted repositories
  ✗ Unsigned binary: /tmp/suspicious [FAIL]
  ✓ No modified system binaries
  ✗ Unknown service: /etc/systemd/system/miner.service [FAIL]

DATA VERIFICATION:
  ✓ Sensitive files have correct permissions
  ✗ World-readable SSH key: /var/www/.ssh/id_rsa [CRITICAL]
  ✓ No unauthorized SUID binaries

ACCESS VERIFICATION:
  ✗ SSH permits root login [FAIL]
  ✓ Password aging policy enforced
  ✗ Sudo configured with NOPASSWD [WARN]

VERIFICATION RESULTS:
  Total Checks: 42
  Passed: 28 (67%)
  Failed: 10 (24%)
  Warnings: 4 (9%)

Trust Score: 67/100 (FAIL - Below 85% threshold)
Critical Issues: 2
Recommendation: System does not meet zero-trust requirements
```

---

### `audit` - Security Audit & Compliance

Comprehensive security audit and compliance checking against industry standards.

**Usage:**
```bash
guestctl audit [OPTIONS] <IMAGE>
```

**Options:**
- `-f, --framework <NAME>` - Compliance framework: cis, pci-dss, hipaa, soc2
- `-s, --severity <LEVEL>` - Minimum severity: info, low, medium, high, critical
- `--remediate` - Generate remediation script
- `-o, --output <FILE>` - Save audit report

**Examples:**
```bash
# General security audit
sudo guestctl audit disk.img

# CIS benchmark compliance
sudo guestctl audit -f cis disk.img

# PCI-DSS with remediation
sudo guestctl audit -f pci-dss --remediate disk.img -o audit-report.json
```

**Output:**
```
=== Security Audit Report ===
Image: disk.img
Framework: CIS Benchmark (Ubuntu 22.04)
Audit Date: 2024-01-28 10:50:00 UTC

SECTION 1: Initial Setup
  [PASS] 1.1.1 Disable unused filesystems
  [FAIL] 1.1.2 Ensure /tmp is configured [CRITICAL]
  [PASS] 1.1.3 Ensure nodev option on /tmp
  [WARN] 1.1.4 Ensure nosuid option on /tmp

SECTION 2: Services
  [PASS] 2.1.1 Ensure xinetd is not installed
  [FAIL] 2.1.2 Ensure telnet server is not enabled [HIGH]
  [PASS] 2.2.1 Ensure NIS is not installed

SECTION 3: Network Configuration
  [FAIL] 3.1.1 Disable IP forwarding [MEDIUM]
  [PASS] 3.1.2 Disable send packet redirects
  [FAIL] 3.2.1 Ensure firewall is enabled [CRITICAL]

SECTION 4: Logging and Auditing
  [PASS] 4.1.1 Ensure auditd is installed
  [WARN] 4.1.2 Ensure auditd service is enabled
  [PASS] 4.2.1 Configure rsyslog

SECTION 5: Access Control
  [FAIL] 5.1.1 Ensure cron daemon is enabled [MEDIUM]
  [FAIL] 5.2.1 SSH root login permitted [CRITICAL]
  [PASS] 5.2.2 SSH protocol version 2

SECTION 6: User Accounts
  [PASS] 6.1.1 Password expiration configured
  [FAIL] 6.1.2 Weak password detected for user 'test' [HIGH]
  [PASS] 6.2.1 Password hashing algorithm: SHA-512

COMPLIANCE SCORE:
  Total Checks: 187
  Passed: 142 (76%)
  Failed: 28 (15%)
  Warnings: 17 (9%)

Compliance Level: NON-COMPLIANT
Critical Issues: 4
High Issues: 8
Remediation Script: Available (--remediate flag)
```

---

### `compliance` - Multi-Framework Compliance Check

Check compliance against multiple regulatory frameworks simultaneously.

**Usage:**
```bash
guestctl compliance [OPTIONS] <IMAGE>
```

**Options:**
- `-f, --frameworks <LIST>` - Frameworks: cis, pci-dss, hipaa, soc2, gdpr, nist
- `--baseline <FILE>` - Custom compliance baseline
- `-r, --report <FORMAT>` - Report format: text, json, html, pdf
- `--evidence` - Collect compliance evidence

**Examples:**
```bash
# Check all frameworks
sudo guestctl compliance disk.img

# Specific frameworks
sudo guestctl compliance -f pci-dss,hipaa disk.img

# Generate HTML report with evidence
sudo guestctl compliance -f soc2 --evidence -r html disk.img > compliance.html
```

**Output:**
```
=== Multi-Framework Compliance Report ===
Image: disk.img
Frameworks: CIS, PCI-DSS, HIPAA, SOC2

CIS Benchmark (Ubuntu 22.04):
  Score: 76% (142/187 checks passed)
  Status: ⚠ NON-COMPLIANT (requires 85%)
  Critical Issues: 4

PCI-DSS v4.0:
  Score: 68% (89/131 requirements met)
  Status: ✗ NON-COMPLIANT
  Critical Issues:
    - Req 2.2: Firewall disabled
    - Req 8.2: Weak password policy
    - Req 10.2: Insufficient logging

HIPAA Security Rule:
  Score: 72% (54/75 controls implemented)
  Status: ⚠ NON-COMPLIANT
  Issues:
    - §164.308(a)(1): Access controls insufficient
    - §164.312(a)(2): Encryption not enforced

SOC 2 Type II:
  Score: 81% (47/58 controls)
  Status: ⚠ PARTIALLY COMPLIANT
  Trust Service Criteria:
    Security: 78% ⚠
    Availability: 85% ✓
    Confidentiality: 75% ⚠
    Processing Integrity: 88% ✓
    Privacy: 80% ⚠

OVERALL COMPLIANCE:
  Average Score: 74%
  Status: NON-COMPLIANT
  Frameworks Passed: 0/4
  Total Issues: 47 (12 critical, 18 high, 17 medium)

Recommendation: Address critical and high-priority issues before deployment
```

---

### `anomaly` - ML-Based Anomaly Detection

Detect anomalies and unusual patterns using machine learning models.

**Usage:**
```bash
guestctl anomaly [OPTIONS] <IMAGE>
```

**Options:**
- `-m, --model <TYPE>` - ML model: isolation-forest, autoencoder, statistical
- `-s, --sensitivity <LEVEL>` - Sensitivity: low, medium, high
- `--baseline <IMAGES>` - Baseline images for training
- `-c, --categories <CATS>` - Categories: files, processes, network, users

**Examples:**
```bash
# Detect anomalies with default model
sudo guestctl anomaly disk.img

# Use isolation forest with high sensitivity
sudo guestctl anomaly -m isolation-forest -s high disk.img

# Train on baseline images
sudo guestctl anomaly --baseline baseline1.img,baseline2.img disk.img
```

**Output:**
```
=== Anomaly Detection Report ===
Image: disk.img
Model: Isolation Forest
Sensitivity: Medium
Baseline: 3 reference images

DETECTED ANOMALIES:

FILE SYSTEM ANOMALIES (Confidence: 92%):
  🔴 Unusual file location: /tmp/.hidden/xmrig
     Reason: Binary in temp directory, hidden name
     Risk: HIGH

  🔴 Abnormal file size: /var/log/syslog (12 KB)
     Reason: Expected ~5 MB, actual 12 KB
     Risk: MEDIUM - Possible log tampering

PROCESS ANOMALIES (Confidence: 88%):
  🔴 Unusual binary name: /usr/bin/kworker
     Reason: Mimics kernel process name
     Risk: HIGH - Possible malware camouflage

USER ACCOUNT ANOMALIES (Confidence: 95%):
  🔴 Anomalous account: sysupdate
     Reason: Recent creation, no login history
     Risk: HIGH - Possible backdoor account

NETWORK ANOMALIES (Confidence: 85%):
  🔴 Unusual connection: 45.33.32.156:8443
     Reason: Non-standard port, foreign IP
     Risk: MEDIUM - Possible C2 communication

CONFIGURATION ANOMALIES (Confidence: 78%):
  🟡 Atypical service: /etc/systemd/system/miner.service
     Reason: Not found in baseline images
     Risk: MEDIUM

Total Anomalies: 12 (6 high-risk, 4 medium, 2 low)
Anomaly Score: 7.8/10
Recommendation: Investigate high-risk anomalies immediately
```

---

### `recommend` - AI-Powered Recommendations

Get AI-powered recommendations for security, performance, and configuration optimization.

**Usage:**
```bash
guestctl recommend [OPTIONS] <IMAGE>
```

**Options:**
- `-f, --focus <AREA>` - Focus area: security, performance, cost, reliability
- `-p, --priority <LEVEL>` - Show only: critical, high, all
- `--actionable` - Only show actionable recommendations
- `-o, --output <FORMAT>` - Output format: text, json, markdown

**Examples:**
```bash
# Get all recommendations
sudo guestctl recommend disk.img

# Focus on security
sudo guestctl recommend -f security disk.img

# Only critical and actionable
sudo guestctl recommend -p critical --actionable disk.img
```

**Output:**
```
=== AI-Powered Recommendations ===
Image: disk.img
Analysis Date: 2024-01-28

SECURITY RECOMMENDATIONS:

  [CRITICAL] Enable Firewall
    Current: Firewall disabled
    Impact: System exposed to network attacks
    Action: systemctl enable firewalld && systemctl start firewalld
    Effort: 5 minutes
    Risk Reduction: 85%

  [CRITICAL] Disable SSH Root Login
    Current: PermitRootLogin yes
    Impact: Direct root access via SSH
    Action: sed -i 's/PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config
    Effort: 2 minutes
    Risk Reduction: 70%

  [HIGH] Update Vulnerable Packages
    Current: 12 packages with known CVEs
    Impact: Exploitable vulnerabilities present
    Action: apt update && apt upgrade openssh-server apache2 ...
    Effort: 15 minutes
    Risk Reduction: 60%

PERFORMANCE RECOMMENDATIONS:

  [MEDIUM] Optimize Swap Usage
    Current: vm.swappiness = 60
    Impact: Excessive swapping reduces performance
    Action: echo "vm.swappiness=10" >> /etc/sysctl.conf
    Effort: 1 minute
    Performance Gain: 15-20%

  [MEDIUM] Enable Disk I/O Optimizations
    Current: /dev/sda mounted without noatime
    Impact: Unnecessary disk writes
    Action: Add noatime,nodiratime to /etc/fstab
    Effort: 5 minutes
    Performance Gain: 10%

COST RECOMMENDATIONS:

  [LOW] Remove Unused Packages
    Current: 247 unused packages (1.2 GB)
    Impact: Wasted storage space
    Action: apt autoremove && apt autoclean
    Effort: 10 minutes
    Savings: 1.2 GB disk space

RELIABILITY RECOMMENDATIONS:

  [HIGH] Configure Automatic Updates
    Current: Unattended upgrades disabled
    Impact: Security patches not applied
    Action: apt install unattended-upgrades && dpkg-reconfigure -plow unattended-upgrades
    Effort: 10 minutes
    Benefit: Automated security patching

SUMMARY:
  Total Recommendations: 23
  Critical: 2 | High: 5 | Medium: 10 | Low: 6
  Estimated Total Effort: 2.5 hours
  Expected Impact: 75% security improvement, 25% performance gain
```

---

### `predict` - Predictive Analytics

Use machine learning to predict future system states, failures, and resource needs.

**Usage:**
```bash
guestctl predict [OPTIONS] <IMAGE>
```

**Options:**
- `-t, --target <METRIC>` - Prediction target: failures, capacity, performance
- `-w, --window <DAYS>` - Prediction window (default: 30 days)
- `--confidence <LEVEL>` - Confidence threshold: 0.0-1.0 (default: 0.8)
- `-m, --model <TYPE>` - Model: lstm, arima, prophet

**Examples:**
```bash
# Predict potential failures
sudo guestctl predict -t failures disk.img

# Predict capacity needs for 90 days
sudo guestctl predict -t capacity -w 90 disk.img

# Performance predictions with high confidence
sudo guestctl predict -t performance --confidence 0.95 disk.img
```

**Output:**
```
=== Predictive Analytics Report ===
Image: disk.img
Target: Failures & Issues
Prediction Window: 30 days
Model: LSTM Neural Network
Confidence Threshold: 80%

FAILURE PREDICTIONS:

  [CRITICAL] Disk Space Exhaustion
    Probability: 95%
    Estimated Time: 14 days
    Current: 78% used (/dev/sda2)
    Trend: +3.2% per day
    Predicted: 100% by 2024-02-11
    Recommendation: Add storage or cleanup within 10 days

  [HIGH] Package Update Failure
    Probability: 87%
    Estimated Time: 7 days
    Reason: Dependency conflicts detected
    Affected: kernel, systemd (18 packages)
    Recommendation: Resolve dependencies before next update

  [MEDIUM] Service Failure (nginx)
    Probability: 72%
    Estimated Time: 21 days
    Reason: Memory leak pattern detected
    Current: 450 MB (increasing 5 MB/day)
    Predicted: OOM killer at 512 MB
    Recommendation: Schedule nginx restart, investigate leak

CAPACITY PREDICTIONS:

  Disk Space (/dev/sda2):
    Current: 16.8 GB / 21.5 GB (78%)
    Day 7: 19.2 GB (89%)
    Day 14: 21.5 GB (100%) ⚠
    Day 30: 26.3 GB (122% - EXCEEDED) 🔴

  Memory Usage:
    Current: 3.2 GB / 4.0 GB (80%)
    Day 7: 3.4 GB (85%)
    Day 30: 3.9 GB (97%)
    Recommendation: Monitor for memory leaks

PERFORMANCE PREDICTIONS:

  CPU Usage Trend: Increasing
    Current avg: 45%
    Predicted (30d): 62%
    Bottleneck Risk: MEDIUM

  I/O Wait Time: Stable
    Current avg: 8%
    Predicted (30d): 9%
    Bottleneck Risk: LOW

RECOMMENDED ACTIONS:
  1. [URGENT] Address disk space within 10 days
  2. Resolve package dependency conflicts
  3. Investigate nginx memory leak
  4. Consider memory upgrade (future)

Confidence Score: 88%
Model Accuracy: 92% (based on historical data)
```

---

### `health` - System Health Diagnostics

Comprehensive system health check and diagnostics.

**Usage:**
```bash
guestctl health [OPTIONS] <IMAGE>
```

**Options:**
- `-c, --categories <CATS>` - Categories: disk, memory, network, services, security
- `-d, --deep` - Deep health check (slower, more thorough)
- `--baseline <IMAGE>` - Compare against baseline
- `-f, --format <FORMAT>` - Output format: text, json, prometheus

**Examples:**
```bash
# Quick health check
sudo guestctl health disk.img

# Deep health check with all categories
sudo guestctl health --deep disk.img

# Focus on specific categories
sudo guestctl health -c disk,services disk.img
```

**Output:**
```
=== System Health Report ===
Image: disk.img
Health Check: Deep Scan

OVERALL HEALTH: 72/100 (FAIR)

━━━ DISK HEALTH (68/100) ━━━
  ⚠ Space: 78% used - approaching capacity
  ✓ Inodes: 12% used - healthy
  ⚠ Fragmentation: 18% - moderate
  ✗ SMART Status: 2 warnings
    - Reallocated sectors: 8
    - Temperature: 48°C (warn at 45°C)
  ✓ Filesystem: No errors detected

━━━ MEMORY HEALTH (85/100) ━━━
  ✓ Usage: 3.2 GB / 4.0 GB (80%) - acceptable
  ✓ Swap: 512 MB / 2.0 GB (25%) - healthy
  ⚠ Memory leaks: 1 potential leak detected (nginx)
  ✓ OOM events: None

━━━ NETWORK HEALTH (90/100) ━━━
  ✓ Interfaces: All up
  ✓ Connectivity: Configured properly
  ✓ DNS: Resolving correctly
  ⚠ Listening ports: 1 unusual port (8888)

━━━ SERVICE HEALTH (65/100) ━━━
  ✓ Running: 38/42 services (90%)
  ✗ Failed: 4 services
    - apache2.service (failed to start)
    - mysql.service (dependency failure)
  ⚠ Degraded: 2 services
    - systemd-timesyncd (time sync issues)
  ✓ Enabled: All critical services enabled

━━━ SECURITY HEALTH (58/100) ━━━
  ✗ Firewall: Disabled [CRITICAL]
  ✗ SSH: Root login permitted [CRITICAL]
  ⚠ Updates: 47 updates available (12 security)
  ⚠ SELinux: Permissive mode
  ✓ Users: No suspicious accounts
  ✗ Passwords: 2 weak passwords detected

━━━ PERFORMANCE HEALTH (78/100) ━━━
  ✓ CPU: Average load 0.45 (healthy)
  ✓ I/O Wait: 8% (acceptable)
  ⚠ Swap usage: Occasional swapping detected
  ✓ Boot time: 12 seconds (fast)

CRITICAL ISSUES: 4
  - Firewall disabled
  - SSH root login enabled
  - 4 failed services
  - Weak passwords

WARNINGS: 8

RECOMMENDATIONS:
  1. Enable firewall immediately
  2. Fix failed services (apache2, mysql)
  3. Apply 12 security updates
  4. Enforce strong password policy
  5. Monitor disk space (approaching limit)

Next Health Check: 2024-02-04 (7 days)
```

---

### `optimize` - Performance Optimization

Analyze and optimize system performance with automated recommendations.

**Usage:**
```bash
guestctl optimize [OPTIONS] <IMAGE>
```

**Options:**
- `-t, --target <GOAL>` - Optimization target: performance, latency, throughput, resources
- `-a, --apply` - Apply optimizations automatically (requires confirmation)
- `--profile <WORKLOAD>` - Workload profile: web-server, database, desktop, compute
- `-o, --output <FILE>` - Save optimization script

**Examples:**
```bash
# Analyze optimization opportunities
sudo guestctl optimize disk.img

# Optimize for web server workload
sudo guestctl optimize --profile web-server disk.img

# Generate optimization script
sudo guestctl optimize disk.img -o optimize.sh
```

**Output:**
```
=== Performance Optimization Report ===
Image: disk.img
Workload Profile: General Purpose

IDENTIFIED OPTIMIZATIONS:

KERNEL PARAMETERS (Impact: HIGH):
  Current: vm.swappiness = 60
  Optimized: vm.swappiness = 10
  Benefit: 15-20% faster under memory pressure

  Current: net.core.rmem_max = 212992
  Optimized: net.core.rmem_max = 16777216
  Benefit: 30% better network throughput

  Current: kernel.sched_migration_cost_ns = 500000
  Optimized: kernel.sched_migration_cost_ns = 5000000
  Benefit: Reduced context switching overhead

FILESYSTEM OPTIMIZATIONS (Impact: MEDIUM):
  Current: /dev/sda2 mounted with defaults
  Optimized: noatime,nodiratime,commit=60
  Benefit: 10% reduction in disk writes

  Current: No I/O scheduler tuning
  Optimized: Set mq-deadline for SSD
  Benefit: 15% better I/O latency

SERVICE OPTIMIZATIONS (Impact: HIGH):
  Disable: 15 unnecessary services
  Services: bluetooth, cups, avahi-daemon, ...
  Benefit: 200 MB RAM freed, faster boot (8s → 6s)

APPLICATION TUNING (Impact: MEDIUM):
  nginx:
    worker_processes: auto (currently: 2)
    worker_connections: 4096 (currently: 768)
    Benefit: 3x concurrent connection capacity

  postgresql:
    shared_buffers: 1GB (currently: 128MB)
    effective_cache_size: 3GB (currently: 4GB)
    Benefit: 40% faster query performance

MEMORY OPTIMIZATIONS (Impact: LOW):
  Transparent Huge Pages: Set to 'madvise'
  Benefit: Reduced memory fragmentation

  Remove unused packages: 247 packages (1.2 GB)
  Benefit: Faster updates, cleaner system

ESTIMATED PERFORMANCE GAINS:
  CPU Performance: +8%
  Memory Efficiency: +15%
  Disk I/O: +25%
  Network Throughput: +30%
  Boot Time: -25% (12s → 9s)

TOTAL IMPACT:
  Performance Score: 68 → 89 (+31%)
  Resource Efficiency: 72 → 91 (+26%)

Apply optimizations? Generated script: optimize.sh
```

---

### `network` - Network Topology Analysis

Analyze network configuration, topology, and connections.

**Usage:**
```bash
guestctl network [OPTIONS] <IMAGE>
```

**Options:**
- `-t, --type <ANALYSIS>` - Analysis type: topology, connections, firewall, dns
- `-g, --graph` - Generate network topology graph
- `--export <FORMAT>` - Export format: json, graphviz, svg
- `-d, --deep` - Deep packet inspection (if pcap files available)

**Examples:**
```bash
# Analyze network configuration
sudo guestctl network disk.img

# Generate topology graph
sudo guestctl network -g --export svg disk.img > topology.svg

# Analyze specific aspect
sudo guestctl network -t firewall disk.img
```

**Output:**
```
=== Network Analysis Report ===
Image: disk.img

━━━ NETWORK INTERFACES ━━━
  eth0:
    MAC: 52:54:00:12:34:56
    IPv4: 192.168.1.100/24
    IPv6: fe80::5054:ff:fe12:3456/64
    Gateway: 192.168.1.1
    DNS: 8.8.8.8, 8.8.4.4
    Status: UP

  lo:
    IPv4: 127.0.0.1/8
    Status: UP

━━━ ROUTING TABLE ━━━
  default via 192.168.1.1 dev eth0
  192.168.1.0/24 dev eth0 scope link

━━━ LISTENING SERVICES ━━━
  TCP:
    22    sshd          ✓ Expected
    80    nginx         ✓ Expected
    443   nginx         ✓ Expected
    3306  mysqld        ✓ Expected
    8888  unknown       ⚠ Unexpected

  UDP:
    53    systemd-resolved  ✓ Expected
    123   ntpd             ✓ Expected

━━━ FIREWALL CONFIGURATION ━━━
  Status: DISABLED ⚠
  Zones: N/A
  Rules: N/A
  Recommendation: Enable firewall immediately

━━━ NETWORK CONNECTIONS ━━━
  Established:
    192.168.1.100:43210 → 8.8.8.8:443 (HTTPS)
    192.168.1.100:36754 → 93.184.216.34:80 (HTTP)

  Recent Connections (from logs):
    192.168.1.100 → 45.33.32.156:8443 ⚠ Suspicious

━━━ DNS CONFIGURATION ━━━
  Nameservers: 8.8.8.8, 8.8.4.4
  Search domains: localdomain
  Resolution: Working

━━━ NETWORK SECURITY ━━━
  ✗ Firewall disabled
  ⚠ Unusual listening port: 8888
  ⚠ Suspicious external connection detected
  ✓ No promiscuous interfaces

NETWORK HEALTH: 65/100 (FAIR)
Security Issues: 3
Performance: Good
Connectivity: Healthy
```

---

### `clone` - Intelligent VM Cloning

Create optimized clones of VM images with customization options.

**Usage:**
```bash
guestctl clone [OPTIONS] <SOURCE> <DEST>
```

**Options:**
- `-c, --customize <SCRIPT>` - Customization script
- `--sysprep` - Run sysprep (remove unique identifiers)
- `--shrink` - Shrink disk image (remove unused space)
- `-n, --name <NAME>` - Set hostname for clone
- `--network <CONFIG>` - Network configuration for clone

**Examples:**
```bash
# Basic clone
sudo guestctl clone source.img clone.img

# Clone with sysprep and shrink
sudo guestctl clone --sysprep --shrink source.img clone.img

# Clone with customization
sudo guestctl clone -c customize.sh -n webserver-02 source.img clone.img
```

**Output:**
```
=== Smart VM Clone ===
Source: source.img (21.5 GB)
Destination: clone.img

CLONE OPERATIONS:

[1/5] Copying disk image...
  Progress: ████████████████████ 100% (21.5 GB)
  Duration: 45 seconds
  ✓ Complete

[2/5] Running sysprep...
  - Removed machine-id
  - Regenerated SSH host keys
  - Cleared network MAC addresses
  - Removed command history
  - Cleared log files
  ✓ Complete

[3/5] Applying customizations...
  - Set hostname: webserver-02
  - Updated network config: DHCP → static
  - Installed additional packages: monitoring-agent
  ✓ Complete

[4/5] Shrinking disk image...
  - Zeroing free space
  - Compacting qcow2 image
  - Original: 21.5 GB
  - Optimized: 18.2 GB
  - Saved: 3.3 GB (15%)
  ✓ Complete

[5/5] Finalizing clone...
  - Generated new UUID
  - Updated /etc/machine-id
  - Verified filesystem integrity
  ✓ Complete

CLONE SUMMARY:
  Source Size: 21.5 GB
  Clone Size: 18.2 GB
  Space Saved: 3.3 GB
  Customizations Applied: 4
  Clone Ready: clone.img

Clone is ready to boot!
```

---

### `repair` - Automated System Repair

Automatically detect and repair common system issues.

**Usage:**
```bash
guestctl repair [OPTIONS] <IMAGE>
```

**Options:**
- `-c, --check-only` - Only detect issues, don't repair
- `-a, --auto` - Automatic repair without prompts
- `--categories <CATS>` - Repair categories: filesystem, packages, services, config
- `-b, --backup <DIR>` - Backup before repair

**Examples:**
```bash
# Check for issues
sudo guestctl repair --check-only disk.img

# Repair with backup
sudo guestctl repair --backup ./backup disk.img

# Automatic repair of specific categories
sudo guestctl repair -a --categories filesystem,services disk.img
```

**Output:**
```
=== System Repair ===
Image: disk.img

SCANNING FOR ISSUES...

DETECTED ISSUES:

[CRITICAL] Filesystem Corruption
  Location: /dev/sda2 (ext4)
  Error: Superblock checksum error
  Repair: Run fsck.ext4
  Risk: LOW (read-only check first)

[HIGH] Failed Services (4)
  - apache2.service: dependency failure
  - mysql.service: config error
  - systemd-timesyncd: time sync issues
  - network-online.target: timeout
  Repair: Fix configurations, restart services
  Risk: LOW

[MEDIUM] Broken Package Dependencies (12)
  Packages: kernel, systemd, libc6, ...
  Issue: Unmet dependencies
  Repair: apt --fix-broken install
  Risk: MEDIUM (may require internet)

[LOW] Missing Log Directories
  Missing: /var/log/nginx
  Repair: Create directories with correct permissions
  Risk: NONE

REPAIR PLAN:
  1. Create backup snapshot
  2. Repair filesystem (fsck)
  3. Fix package dependencies
  4. Repair service configurations
  5. Create missing directories
  6. Restart failed services

Proceed with repair? [y/N]: y

EXECUTING REPAIRS...

[1/6] Creating backup...
  Backup: ./repair-backup-2024-01-28.img
  ✓ Complete

[2/6] Repairing filesystem...
  fsck.ext4 /dev/sda2
  Fixed: 3 inodes, 0 blocks
  ✓ Filesystem repaired

[3/6] Fixing package dependencies...
  apt --fix-broken install
  Installing: 3 packages
  Configuring: 12 packages
  ✓ Dependencies resolved

[4/6] Repairing service configurations...
  apache2: Fixed DocumentRoot path
  mysql: Fixed socket path
  systemd-timesyncd: Updated NTP servers
  ✓ Configurations repaired

[5/6] Creating missing directories...
  mkdir -p /var/log/nginx
  chown root:root /var/log/nginx
  chmod 755 /var/log/nginx
  ✓ Directories created

[6/6] Restarting services...
  apache2.service: ✓ Started
  mysql.service: ✓ Started
  systemd-timesyncd.service: ✓ Started
  ✓ All services running

REPAIR COMPLETE:
  Issues Found: 19
  Issues Fixed: 19
  Success Rate: 100%
  Backup: ./repair-backup-2024-01-28.img

System is now healthy!
```

---

### `harden` - Security Hardening

Apply security hardening measures based on industry best practices.

**Usage:**
```bash
guestctl harden [OPTIONS] <IMAGE>
```

**Options:**
- `-p, --profile <PROFILE>` - Hardening profile: basic, standard, strict, paranoid
- `-f, --framework <NAME>` - Framework: cis, stig, nsa
- `--dry-run` - Show what would be changed
- `-b, --backup <DIR>` - Backup before hardening

**Examples:**
```bash
# Standard hardening
sudo guestctl harden disk.img

# CIS benchmark hardening
sudo guestctl harden -f cis disk.img

# Paranoid mode with dry-run
sudo guestctl harden -p paranoid --dry-run disk.img
```

**Output:**
```
=== Security Hardening ===
Image: disk.img
Profile: Standard
Framework: CIS Benchmark

HARDENING OPERATIONS:

━━━ SSH HARDENING ━━━
  ✓ Disable root login (PermitRootLogin no)
  ✓ Disable password authentication (use keys only)
  ✓ Set SSH protocol to 2
  ✓ Disable X11 forwarding
  ✓ Enable strict mode
  ✓ Set login grace time to 60s
  ✓ Limit authentication attempts to 3

━━━ FIREWALL HARDENING ━━━
  ✓ Enable firewall (firewalld)
  ✓ Set default zone to 'public'
  ✓ Allow SSH (22/tcp)
  ✓ Allow HTTP/HTTPS (80,443/tcp)
  ✓ Block all other incoming
  ✓ Enable connection tracking

━━━ USER HARDENING ━━━
  ✓ Enforce strong passwords (min 12 chars, complexity)
  ✓ Set password aging (max 90 days)
  ✓ Lock inactive accounts (30 days)
  ✓ Disable unused user accounts (3 disabled)
  ✓ Remove empty password fields

━━━ FILESYSTEM HARDENING ━━━
  ✓ Set noexec on /tmp
  ✓ Set nodev on /tmp
  ✓ Set nosuid on /tmp
  ✓ Set permissions on sensitive files:
    - /etc/shadow: 000 → 000
    - /etc/ssh/sshd_config: 644 → 600
    - /boot/grub/grub.cfg: 644 → 400

━━━ KERNEL HARDENING ━━━
  ✓ Disable IP forwarding
  ✓ Enable SYN cookies
  ✓ Disable ICMP redirects
  ✓ Enable reverse path filtering
  ✓ Disable source routing
  ✓ Log martian packets

━━━ SERVICE HARDENING ━━━
  ✓ Disable unnecessary services (12 services):
    - telnet, rlogin, rsh, rexec
    - tftp, talk, finger
    - nfs, nis, rpc
  ✓ Enable automatic security updates
  ✓ Configure audit logging (auditd)

━━━ MAC (Mandatory Access Control) ━━━
  ✓ Enable SELinux (enforcing mode)
  ✓ Configure SELinux policies
  ✓ Set file contexts

━━━ ADDITIONAL HARDENING ━━━
  ✓ Disable core dumps
  ✓ Set UMASK to 027
  ✓ Configure password history (remember 5)
  ✓ Set session timeout (15 minutes)
  ✓ Enable process accounting

HARDENING SUMMARY:
  Checks Applied: 58
  Changes Made: 47
  Warnings: 3
  Errors: 0

SECURITY SCORE:
  Before: 58/100 (POOR)
  After: 92/100 (EXCELLENT)
  Improvement: +34 points (+59%)

VERIFICATION:
  ✓ All critical controls implemented
  ✓ System meets CIS Level 1 requirements
  ✓ No functionality broken

System has been hardened successfully!
```

---

### `evolve` - System Evolution Tracking

Track how a system has evolved over time through multiple snapshots.

**Usage:**
```bash
guestctl evolve [OPTIONS] <IMAGES>...
```

**Options:**
- `-t, --timeline` - Show timeline visualization
- `-m, --metrics <METRICS>` - Track metrics: packages, users, services, config
- `-o, --output <FORMAT>` - Output format: text, json, html
- `-g, --graph` - Generate evolution graph

**Examples:**
```bash
# Track evolution across snapshots
sudo guestctl evolve snap1.img snap2.img snap3.img snap4.img

# Timeline with graph
sudo guestctl evolve -t -g snap*.img

# Track specific metrics
sudo guestctl evolve -m packages,services snap*.img
```

**Output:**
```
=== System Evolution Analysis ===
Snapshots: 4 images spanning 30 days

TIMELINE:

2024-01-01 (snap1.img) - Baseline
  OS: Ubuntu 22.04.1
  Packages: 1,247
  Services: 42 enabled
  Users: 12
  Disk Usage: 12.5 GB

2024-01-10 (snap2.img) - Update +9 days
  Changes:
    + Packages: +15 (nginx, certbot, monitoring-agent)
    + Services: +3 (nginx, prometheus-node-exporter)
    + Users: +1 (deploy)
    ~ Disk: +2.1 GB (web content)
    ~ Config: SSL certificates added

2024-01-20 (snap3.img) - Incident +19 days
  Changes:
    + Packages: +5 (security patches)
    - Services: -1 (apache2 removed)
    + Users: +1 (sysupdate) ⚠ Suspicious
    ~ Disk: +0.8 GB
    ! Security: Firewall disabled ⚠
    ! Anomaly: Unusual network activity

2024-01-31 (snap4.img) - Recovery +30 days
  Changes:
    + Packages: +3 (security tools)
    - Users: -1 (sysupdate removed) ✓
    + Security: Firewall re-enabled ✓
    ~ Disk: -0.3 GB (cleanup)
    ~ Config: Hardening applied

PACKAGE EVOLUTION:
  Total Installed: 1,247 → 1,265 → 1,270 → 1,273
  Net Change: +26 packages
  Key Additions:
    - nginx-1.24.0 (day 10)
    - monitoring stack (day 10)
    - security tools (day 31)
  Key Removals:
    - apache2 (day 20)
    - malware artifacts (day 31)

SERVICE EVOLUTION:
  Total Enabled: 42 → 45 → 44 → 45
  Net Change: +3 services
  Added: nginx, prometheus, auditd
  Removed: apache2

USER EVOLUTION:
  Total Users: 12 → 13 → 14 → 13
  Net Change: +1 user
  Added: deploy (day 10), sysupdate (day 20)
  Removed: sysupdate (day 31) - security remediation

CONFIGURATION DRIFT:
  Day 10: Planned changes (web server deployment)
  Day 20: Unauthorized changes detected ⚠
    - Firewall disabled
    - Suspicious user added
    - Network anomalies
  Day 31: Remediation applied ✓
    - Security restored
    - System hardened

SECURITY POSTURE:
  Day 1:  75/100 (Good)
  Day 10: 78/100 (Good) +3
  Day 20: 52/100 (Poor) -26 ⚠ Security incident
  Day 31: 92/100 (Excellent) +40 ✓ Fully remediated

GROWTH METRICS:
  Disk Usage: 12.5 → 14.6 → 15.4 → 15.1 GB (+2.6 GB net)
  Growth Rate: ~87 MB/day
  Projection (90 days): 20.3 GB

KEY INSIGHTS:
  - Security incident occurred between day 10-20
  - System fully recovered and hardened by day 31
  - Steady growth in disk usage (monitoring recommended)
  - Package management healthy (regular updates)
  - User account hygiene improved

RECOMMENDATIONS:
  ✓ Security posture excellent after remediation
  - Monitor disk usage (projected 20GB in 90 days)
  - Continue regular security updates
  - Maintain current hardening configuration
```

---

### `insights` - AI-Generated Insights

Generate comprehensive AI-powered insights about the system state and patterns.

**Usage:**
```bash
guestctl insights [OPTIONS] <IMAGE>
```

**Options:**
- `-c, --context <INFO>` - Additional context for AI analysis
- `-f, --focus <AREA>` - Focus area: security, performance, operations, cost
- `-d, --depth <LEVEL>` - Analysis depth: quick, standard, comprehensive
- `-o, --output <FORMAT>` - Output format: text, json, markdown, html

**Examples:**
```bash
# Generate comprehensive insights
sudo guestctl insights disk.img

# Focus on security with context
sudo guestctl insights -f security -c "production web server" disk.img

# Deep analysis with HTML report
sudo guestctl insights -d comprehensive -o html disk.img > insights.html
```

**Output:**
```
=== AI-Generated System Insights ===
Image: disk.img
Analysis Depth: Comprehensive
Model: GPT-4 Analysis Engine

🔍 EXECUTIVE SUMMARY:
This Ubuntu 22.04 system appears to be a web server that has experienced a security
incident, followed by successful remediation. The system currently shows strong
security posture but has some operational concerns around disk space and potential
memory leaks that require attention.

━━━ SECURITY INSIGHTS ━━━

✅ STRENGTHS:
  - Firewall properly configured with restrictive rules
  - SSH hardened (no root login, key-based auth)
  - Recent security patches applied (12 updates)
  - SELinux in enforcing mode
  - Strong password policy enforced

⚠ CONCERNS:
  - Previous compromise indicators still present in logs
  - One unusual listening port (8888) - purpose unclear
  - 2 certificates expiring within 30 days
  - Audit logging recently enabled (no historical data)

🔴 CRITICAL FINDING:
  Evidence of previous unauthorized access detected:
    - Backdoor user account created and later removed
    - Suspicious network connections to 45.33.32.156
    - Log file tampering (timestamps manipulated)
    - Malware remnants in /tmp (quarantined)

  Analysis: System was compromised ~11 days ago, properly cleaned
  and hardened. However, recommend full forensic review to ensure
  complete remediation.

━━━ PERFORMANCE INSIGHTS ━━━

📊 CURRENT STATE:
  - CPU: Underutilized (avg 12%) - appropriate for current load
  - Memory: 80% used - within normal range but trending up
  - Disk I/O: 8% wait time - acceptable
  - Network: Moderate traffic, no saturation

🔮 PREDICTIONS:
  - Memory leak detected in nginx (5MB/day growth)
    → Will reach limit in ~40 days
  - Disk space trend concerning (78% full, +3.2%/week)
    → Will be full in ~14 days ⚠

🎯 OPTIMIZATION OPPORTUNITIES:
  - Nginx: Increase worker connections (3x capacity gain)
  - Swap: Reduce swappiness (15-20% faster under load)
  - Filesystem: Add noatime mount option (10% less writes)
  - Services: Disable 15 unused services (200MB RAM freed)

━━━ OPERATIONAL INSIGHTS ━━━

📈 SYSTEM HEALTH TREND:
  Overall trajectory: ⬆ IMPROVING
  - Security: ⬆⬆ Significantly improved (52 → 92/100)
  - Reliability: ⬆ Stable (4 failed services → 0)
  - Performance: ➡ Steady (no degradation)
  - Compliance: ⬆ Now meets CIS Level 1

⚙️ MAINTENANCE STATUS:
  ✓ Regular updates being applied
  ✓ Monitoring in place (prometheus)
  ⚠ Backups not detected - recommend implementing
  ⚠ No disaster recovery plan evident

🎯 WORKLOAD ANALYSIS:
  Primary Role: Web application server (nginx + MySQL)
  Traffic Pattern: Business hours (9am-5pm spike)
  Resource Usage: Light-to-moderate
  Criticality: HIGH (production system)

━━━ COST INSIGHTS ━━━

💰 RESOURCE EFFICIENCY:
  - CPU: Over-provisioned (only 12% utilized)
    → Could reduce from 4→2 vCPUs: $20/mo savings
  - Memory: Appropriately sized (80% used)
  - Disk: Under-provisioned (78% full, growing)
    → Need +10GB: $2/mo additional cost
  - Network: Efficient (no bandwidth waste)

💡 COST OPTIMIZATION:
  Total potential savings: $18/mo (25% reduction)
  - Resize CPU: -$20/mo
  - Add disk: +$2/mo
  - Remove unused packages: One-time 1.2GB saved

━━━ COMPLIANCE INSIGHTS ━━━

✅ FRAMEWORKS:
  - CIS Benchmark: 92% compliant (Level 1) ✓
  - PCI-DSS: 68% compliant (needs work)
  - HIPAA: Not applicable (no PHI detected)
  - SOC 2: 81% compliant (partially)

📋 GAP ANALYSIS:
  To achieve full PCI-DSS compliance:
    - Implement stronger encryption (TLS 1.3)
    - Enhanced logging (6-month retention)
    - Network segmentation
    - Regular vulnerability scanning
  Estimated effort: 16-20 hours

━━━ RISK ASSESSMENT ━━━

🎯 TOP RISKS:

  [CRITICAL - 95%] Disk Space Exhaustion
    Impact: Service outage
    Timeline: 14 days
    Mitigation: Add storage or cleanup

  [HIGH - 87%] Memory Leak (nginx)
    Impact: Performance degradation, potential crash
    Timeline: 40 days
    Mitigation: Update nginx, investigate leak

  [MEDIUM - 72%] Certificate Expiration
    Impact: Service disruption, security warnings
    Timeline: 30 days
    Mitigation: Renew certificates

  [MEDIUM - 65%] Incomplete Forensics
    Impact: Possible persistent compromise
    Timeline: N/A
    Mitigation: Full forensic analysis

━━━ RECOMMENDATIONS (Prioritized) ━━━

🔴 IMMEDIATE (This Week):
  1. Address disk space - add 10GB or cleanup
  2. Renew expiring SSL certificates
  3. Complete forensic analysis of security incident
  4. Implement backup solution

🟡 SHORT TERM (This Month):
  5. Investigate and fix nginx memory leak
  6. Apply PCI-DSS gaps if required
  7. Review and document the 8888 port purpose
  8. Implement automated monitoring alerts

🟢 LONG TERM (This Quarter):
  9. Right-size CPU allocation (cost savings)
  10. Develop disaster recovery plan
  11. Implement automated security scanning
  12. Create system documentation

━━━ CONFIDENCE SCORES ━━━
  Security Analysis: 94%
  Performance Analysis: 91%
  Prediction Accuracy: 88%
  Recommendations: 96%

Generated: 2024-01-28 11:00:00 UTC
Next Analysis: 2024-02-04 (7 days)
```

---

### `dependencies` - Dependency Analysis

Analyze package dependencies, conflicts, and dependency chains.

**Usage:**
```bash
guestctl dependencies [OPTIONS] <IMAGE>
```

**Options:**
- `-p, --package <NAME>` - Analyze specific package
- `-t, --type <TYPE>` - Analysis type: tree, graph, conflicts, circular
- `-d, --depth <N>` - Maximum dependency depth
- `--visualize` - Generate dependency graph visualization

**Examples:**
```bash
# Analyze all dependencies
sudo guestctl dependencies disk.img

# Specific package dependency tree
sudo guestctl dependencies -p nginx disk.img

# Find circular dependencies
sudo guestctl dependencies -t circular disk.img

# Visualize dependency graph
sudo guestctl dependencies -p systemd --visualize disk.img > deps.dot
```

**Output:**
```
=== Dependency Analysis ===
Image: disk.img
Total Packages: 1,273

DEPENDENCY STATISTICS:
  Total Dependencies: 4,892
  Average Depth: 3.2
  Max Depth: 8 (kernel → drivers → firmware)
  Circular Dependencies: 3 detected

TOP-LEVEL PACKAGES (23):
  nginx, mysql-server, openssh-server, systemd, ...

LEAF PACKAGES (187):
  No dependencies: libfoo, data-files, fonts, ...

CRITICAL DEPENDENCY CHAINS:

  systemd (depth: 5)
    ├── libc6
    ├── libsystemd0
    │   ├── libgcrypt20
    │   ├── liblz4-1
    │   └── libzstd1
    ├── libcap2
    └── libmount1
        └── libblkid1

  nginx (depth: 4)
    ├── libc6
    ├── libssl3
    │   └── libcrypto3
    └── libpcre3

CIRCULAR DEPENDENCIES DETECTED:

  [1] perl ⟷ perl-base
      perl depends on perl-base
      perl-base depends on perl

  [2] systemd ⟷ udev
      systemd depends on udev
      udev depends on libsystemd0 (provided by systemd)

  [3] network-manager ⟷ wpasupplicant
      network-manager suggests wpasupplicant
      wpasupplicant recommends network-manager

DEPENDENCY CONFLICTS:

  apache2 ⚔ nginx
    Both provide: httpd
    Status: nginx installed, apache2 removed
    Resolution: OK

  mysql-server ⚔ mariadb-server
    Both provide: mysql-server
    Status: mysql-server installed
    Resolution: OK

BROKEN DEPENDENCIES:

  ⚠ libfoo3 (missing)
    Required by: custom-app
    Status: NOT INSTALLED
    Impact: custom-app may not function

  ⚠ python3-bar (>=2.0)
    Required by: automation-script
    Installed: 1.8 (too old)
    Impact: Version conflict

UPDATE IMPACT ANALYSIS:

  If kernel updated:
    - 47 packages would be affected
    - Requires rebuild: nvidia-driver
    - Estimated downtime: 5-10 minutes

  If libc6 updated:
    - 892 packages would be affected ⚠
    - Critical: System-wide impact
    - Recommended: Test in staging first

SECURITY-CRITICAL PATHS:

  openssh-server → libssl3 → libcrypto3
    Risk: SSL vulnerability affects SSH
    Status: All up-to-date ✓

RECOMMENDATIONS:
  1. Fix 2 broken dependencies
  2. Monitor circular dependencies (generally safe)
  3. Test libc6 updates in staging before production
  4. Document custom-app dependency on libfoo3
```

---

### `simulate` - Scenario Simulation

Simulate various scenarios and predict outcomes (updates, failures, attacks).

**Usage:**
```bash
guestctl simulate [OPTIONS] <IMAGE>
```

**Options:**
- `-s, --scenario <TYPE>` - Scenario: update, upgrade, failure, attack, migration
- `-p, --parameters <PARAMS>` - Scenario-specific parameters
- `--iterations <N>` - Number of simulation runs (default: 1000)
- `-o, --output <FORMAT>` - Output format: text, json, report

**Examples:**
```bash
# Simulate system update
sudo guestctl simulate -s update disk.img

# Simulate disk failure
sudo guestctl simulate -s failure -p "disk=/dev/sda" disk.img

# Simulate security attack
sudo guestctl simulate -s attack -p "type=ransomware" disk.img

# Simulate migration
sudo guestctl simulate -s migration -p "target=aws" disk.img
```

**Output:**
```
=== Scenario Simulation: System Update ===
Image: disk.img
Scenario: Major OS Upgrade (22.04 → 24.04)
Iterations: 1000 simulations

SIMULATION RESULTS:

SUCCESS RATE: 87.3% (873/1000 simulations)

FAILURE MODES DETECTED:

  [12.7%] Update Failures
    - Package dependency conflicts: 8.2%
    - Insufficient disk space: 3.1%
    - Network timeout: 1.4%

  [5.2%] Boot Failures (post-update)
    - Kernel panic: 2.8%
    - initramfs errors: 1.6%
    - Bootloader issues: 0.8%

  [2.1%] Service Failures
    - nginx config incompatible: 1.2%
    - mysql upgrade failed: 0.6%
    - Custom services broken: 0.3%

PREDICTED IMPACT:

  Disk Space Required:
    Mean: 3.8 GB
    Range: 3.2 - 4.5 GB
    Current Free: 4.7 GB
    Margin: 0.9 GB ⚠ Tight

  Download Size:
    Mean: 1.2 GB
    Time (100 Mbps): ~2 minutes
    Time (10 Mbps): ~16 minutes

  Downtime:
    Mean: 12 minutes
    Range: 8-25 minutes
    95th percentile: 18 minutes

  Service Impact:
    nginx: 98% starts successfully
    mysql: 94% starts successfully
    custom-app: 76% requires manual fix ⚠

RISK ASSESSMENT:

  🟢 LOW RISK (82.1%):
    - Clean update, all services start
    - No manual intervention required
    - Estimated time: 10-15 minutes

  🟡 MEDIUM RISK (10.6%):
    - Some package conflicts (auto-resolvable)
    - One service needs restart
    - Estimated time: 15-20 minutes

  🔴 HIGH RISK (7.3%):
    - Manual intervention required
    - Possible service configuration needed
    - Estimated time: 30+ minutes

RECOMMENDATIONS:

  BEFORE UPDATE:
    ✓ Free up 2GB disk space (current margin: 0.9GB)
    ✓ Backup system (critical)
    ✓ Test custom-app compatibility (76% success)
    ✓ Schedule during low-traffic window

  DURING UPDATE:
    ✓ Monitor nginx config compatibility
    ✓ Watch for mysql schema changes
    ✓ Have rollback plan ready

  AFTER UPDATE:
    ✓ Verify all services running
    ✓ Test custom-app functionality
    ✓ Check logs for warnings
    ✓ Monitor performance for 24h

SUCCESS PROBABILITY: 87% (95% with prep steps)

SIMULATION CONFIDENCE: 89%

Proceed with update? Recommended: YES (with preparation)
```

---

### `template` - Template Generation

Generate reusable templates from existing disk images.

**Usage:**
```bash
guestctl template [OPTIONS] <IMAGE>
```

**Options:**
- `-t, --type <TYPE>` - Template type: vagrant, docker, cloud-init, packer
- `-n, --name <NAME>` - Template name
- `-c, --customize <OPTS>` - Customization options
- `-o, --output <FILE>` - Output template file

**Examples:**
```bash
# Generate Vagrant template
sudo guestctl template -t vagrant -n webserver disk.img

# Generate cloud-init template
sudo guestctl template -t cloud-init disk.img -o cloud-init.yaml

# Generate Packer template
sudo guestctl template -t packer disk.img -o webserver.pkr.hcl
```

**Output:**
```
=== Template Generation ===
Image: disk.img
Type: Cloud-Init
Output: cloud-init.yaml

ANALYZING IMAGE...
  OS: Ubuntu 22.04 LTS
  Packages: 1,273 detected
  Services: 45 enabled
  Users: 13 (12 will be templated)
  Network: DHCP configuration
  Storage: 21.5 GB

GENERATING TEMPLATE...

#cloud-config Generated at: cloud-init.yaml

━━━ GENERATED CLOUD-INIT TEMPLATE ━━━

```yaml
#cloud-config
# Auto-generated from: disk.img
# OS: Ubuntu 22.04 LTS
# Generated: 2024-01-28

# Hostname (customize per instance)
hostname: ${HOSTNAME}
fqdn: ${HOSTNAME}.${DOMAIN}

# Users
users:
  - name: ${ADMIN_USER}
    groups: sudo, adm, systemd-journal
    shell: /bin/bash
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - ${SSH_PUBLIC_KEY}

# Package updates
package_update: true
package_upgrade: true

# Install required packages
packages:
  - nginx
  - mysql-server
  - certbot
  - python3-certbot-nginx
  - prometheus-node-exporter
  - monitoring-agent

# Network configuration
network:
  version: 2
  ethernets:
    eth0:
      dhcp4: true
      dhcp6: false

# Write configuration files
write_files:
  - path: /etc/nginx/sites-available/default
    content: |
      server {
        listen 80 default_server;
        listen [::]:80 default_server;

        root /var/www/html;
        index index.html index.htm;

        server_name ${HOSTNAME}.${DOMAIN};

        location / {
          try_files $uri $uri/ =404;
        }
      }
    permissions: '0644'
    owner: root:root

  - path: /etc/systemd/system/app.service
    content: |
      [Unit]
      Description=Application Service
      After=network.target mysql.service

      [Service]
      Type=simple
      User=www-data
      WorkingDirectory=/var/www/app
      ExecStart=/usr/bin/python3 /var/www/app/main.py
      Restart=always

      [Install]
      WantedBy=multi-user.target
    permissions: '0644'

# Commands to run
runcmd:
  # Configure MySQL
  - mysql -e "CREATE DATABASE IF NOT EXISTS ${DB_NAME};"
  - mysql -e "CREATE USER IF NOT EXISTS '${DB_USER}'@'localhost' IDENTIFIED BY '${DB_PASS}';"
  - mysql -e "GRANT ALL PRIVILEGES ON ${DB_NAME}.* TO '${DB_USER}'@'localhost';"

  # SSL/TLS Setup
  - certbot --nginx -d ${HOSTNAME}.${DOMAIN} --non-interactive --agree-tos -m ${ADMIN_EMAIL}

  # Enable services
  - systemctl enable nginx mysql app prometheus-node-exporter
  - systemctl start nginx mysql app prometheus-node-exporter

  # Security hardening
  - ufw allow OpenSSH
  - ufw allow 'Nginx Full'
  - ufw --force enable

  # Cleanup
  - apt autoremove -y
  - apt autoclean

# Final message
final_message: |
  Cloud-init deployment complete!
  System: ${HOSTNAME}.${DOMAIN}
  Services: nginx, mysql, app
  Security: Firewall enabled, SSL configured
  Duration: $UPTIME seconds
```

━━━ TEMPLATE VARIABLES ━━━

Required:
  - HOSTNAME: Instance hostname
  - DOMAIN: Domain name
  - ADMIN_USER: Administrative user
  - SSH_PUBLIC_KEY: SSH public key
  - DB_NAME: Database name
  - DB_USER: Database user
  - DB_PASS: Database password
  - ADMIN_EMAIL: Administrator email

━━━ USAGE EXAMPLE ━━━

```bash
# Set variables
export HOSTNAME="web01"
export DOMAIN="example.com"
export ADMIN_USER="admin"
export SSH_PUBLIC_KEY="ssh-rsa AAAA..."
export DB_NAME="webapp"
export DB_USER="webapp_user"
export DB_PASS="SecurePass123!"
export ADMIN_EMAIL="admin@example.com"

# Deploy with cloud provider
aws ec2 run-instances \\
  --image-id ami-xxxxxx \\
  --user-data file://cloud-init.yaml \\
  --instance-type t3.medium

# Or use with multipass
multipass launch --cloud-init cloud-init.yaml --name web01
```

Template generated successfully: cloud-init.yaml
```

---

### `rescue` - Disaster Recovery Operations

Perform disaster recovery operations on failed or damaged systems.

**Usage:**
```bash
guestctl rescue [OPTIONS] <IMAGE>
```

**Options:**
- `-m, --mode <MODE>` - Rescue mode: diagnose, repair, recover, extract
- `-t, --target <PATH>` - Target for recovery operations
- `--force` - Force operations (skip safety checks)
- `-b, --backup <DIR>` - Create backup before rescue operations

**Examples:**
```bash
# Diagnose issues
sudo guestctl rescue -m diagnose disk.img

# Attempt automatic repair
sudo guestctl rescue -m repair disk.img

# Extract critical data
sudo guestctl rescue -m extract -t ./recovered disk.img

# Full recovery with backup
sudo guestctl rescue -m recover --backup ./backup disk.img
```

**Output:**
```
=== Disaster Recovery ===
Image: disk.img
Mode: Full Recovery

PHASE 1: DIAGNOSIS

Checking Image Integrity...
  ✓ Image file readable
  ⚠ QCOW2 corruption detected
  ✓ Attempting repair...
  ✓ Image repaired successfully

Filesystem Status...
  ✗ /dev/sda2 (ext4): CORRUPTED
    - Superblock damaged
    - Inode table errors
    - Journal inconsistent
  ✓ /dev/sda1 (vfat): OK

Boot Sector...
  ✓ MBR intact
  ⚠ GRUB configuration corrupted
  ✓ Kernel present

Criticality: HIGH - System unbootable

PHASE 2: DATA RECOVERY

Mounting Filesystems (read-only)...
  ✓ /dev/sda1 mounted
  ⚠ /dev/sda2 mounted (degraded mode)

Scanning for Recoverable Data...
  Critical Data Found:
    ✓ /etc/passwd, /etc/shadow (user accounts)
    ✓ /var/www/html (web content) - 95% intact
    ✓ /home/* (user data) - 98% intact
    ✗ /var/lib/mysql (database) - 60% intact ⚠
    ✓ /root/.ssh (SSH keys)
    ✓ /etc/nginx (configuration)

  Recovered: 18.2 GB / 19.5 GB (93%)

PHASE 3: REPAIR ATTEMPTS

Filesystem Repair...
  Running: fsck.ext4 -y /dev/sda2
  Pass 1: Checking inodes, blocks, and sizes
  Pass 2: Checking directory structure
  Pass 3: Checking directory connectivity
  Pass 4: Checking reference counts
  Pass 5: Checking group summary information

  Repaired: 847 inodes, 124 blocks
  ✓ Filesystem repaired

Bootloader Repair...
  Reinstalling GRUB...
  Regenerating grub.cfg...
  ✓ Bootloader repaired

Database Recovery...
  MySQL recovery mode...
  InnoDB recovery attempt...
  ⚠ 15 tables corrupted beyond repair
  ✓ 203 tables recovered successfully
  Recovery rate: 93%

PHASE 4: SYSTEM RESTORATION

Rebuilding Critical Services...
  ✓ Restored nginx configuration
  ✓ Restored user accounts
  ⚠ MySQL: 15 tables require manual recovery
  ✓ Restored SSH configuration
  ✓ Restored network configuration

Verifying System Integrity...
  ✓ Boot process: OK
  ✓ Critical services: 95% functional
  ⚠ Database: Partially recovered
  ✓ User data: 98% intact

RECOVERY SUMMARY:

  Overall Success: 93%

  ✓ RECOVERED:
    - Filesystem structure
    - Boot capability
    - Web server (nginx)
    - User accounts and data
    - SSH access
    - Network configuration
    - 203/218 database tables

  ⚠ PARTIAL LOSS:
    - 15 MySQL tables corrupted
    - Some log files incomplete
    - Minor configuration drift

  ✗ NOT RECOVERABLE:
    - Complete MySQL transaction log
    - Some temporary files

NEXT STEPS:

  1. Boot recovered system and verify functionality
  2. Manual database recovery for 15 tables:
     - orders_2024
     - transactions_jan
     - ...
  3. Restore from backup if needed (for lost MySQL tables)
  4. Implement backup strategy to prevent future data loss

Recovered Image: disk-recovered.img
Recovery Log: recovery-2024-01-28.log

System is bootable and 93% functional!
```

---

## Global Options

### Verbose Mode

Enable detailed logging for debugging.

```bash
# Any command with verbose output
sudo guestctl -v inspect ubuntu.qcow2
sudo guestctl --verbose filesystems ubuntu.qcow2
```

**Verbose Output:**
```
Launching appliance...
Mounting filesystems...
  Mounted /dev/sda2 at /
  Mounted /dev/sda1 at /boot
...
```

---

## Scripting with JSON

Many commands support `--json` output for easy parsing with `jq`:

### Extract specific information:

```bash
# Get OS type
sudo guestctl inspect --json disk.img | jq -r '.operating_systems[0].type'

# Get hostname
sudo guestctl inspect --json disk.img | jq -r '.operating_systems[0].hostname'

# Count packages
sudo guestctl packages --json disk.img | jq '.total'

# Filter packages by name
sudo guestctl packages --json disk.img | jq '.packages[] | select(.name | contains("python"))'
```

### Batch processing:

```bash
# Inspect multiple disks
for disk in *.qcow2; do
    echo "=== $disk ==="
    sudo guestctl inspect --json "$disk" | jq -r '.operating_systems[0].distro'
done

# Generate report
sudo guestctl inspect --json disk.img | jq '{
    hostname: .operating_systems[0].hostname,
    os: .operating_systems[0].distro,
    version: .operating_systems[0].version
}'
```

---

## Requirements

- **Root/sudo access** - Most operations require root privileges
- **System tools** - QEMU tools (qemu-img, qemu-nbd) for disk operations
- **Disk formats** - Supports QCOW2, VMDK, RAW, VDI, VHD, and more

---

## Common Use Cases

### 1. Pre-deployment Verification

```bash
# Check OS version before deploying VM
sudo guestctl inspect --json template.qcow2 | jq -r '.operating_systems[0].version'

# Verify hostname is set correctly
sudo guestctl cat template.qcow2 /etc/hostname
```

### 2. Security Audit

```bash
# List all installed packages
sudo guestctl packages disk.img > installed-packages.txt

# Check for specific vulnerable packages
sudo guestctl packages --filter openssl disk.img

# Extract SSH keys
sudo guestctl cp disk.img:/root/.ssh/authorized_keys ./authorized_keys
```

### 3. Troubleshooting

```bash
# Check system logs without booting VM
sudo guestctl cat disk.img /var/log/syslog > syslog.txt

# List running services configuration
sudo guestctl ls --long disk.img /etc/systemd/system

# Extract config files for analysis
sudo guestctl cp disk.img:/etc/nginx/nginx.conf ./nginx.conf
```

### 4. Inventory Management

```bash
# Generate inventory report
for vm in /var/lib/libvirt/images/*.qcow2; do
    hostname=$(sudo guestctl inspect --json "$vm" | jq -r '.operating_systems[0].hostname')
    distro=$(sudo guestctl inspect --json "$vm" | jq -r '.operating_systems[0].distro')
    version=$(sudo guestctl inspect --json "$vm" | jq -r '.operating_systems[0].version')
    echo "$vm,$hostname,$distro,$version"
done > vm-inventory.csv
```

### 5. Backup Verification

```bash
# Verify backup contains expected files
sudo guestctl ls backup.img /home/user/important-data

# Extract specific files from backup
sudo guestctl cp backup.img:/home/user/document.pdf ./recovered-document.pdf
```

---

## Tips & Best Practices

### 1. Always Use Read-Only Operations

The CLI automatically uses read-only mode for all operations except when explicitly writing. This prevents accidental modifications.

### 2. Use JSON for Automation

When scripting, always use `--json` output and parse with `jq`:

```bash
# Good: Robust parsing
distro=$(sudo guestctl inspect --json disk.img | jq -r '.operating_systems[0].distro')

# Bad: Fragile text parsing
distro=$(sudo guestctl inspect disk.img | grep "Distribution:" | cut -d: -f2)
```

### 3. Filter Before Limiting

When working with packages, filter first, then limit:

```bash
# Good: Get all kernel packages
sudo guestctl packages --filter kernel disk.img

# Less useful: Random 20 packages
sudo guestctl packages --limit 20 disk.img
```

### 4. Check for Errors

Always check exit codes in scripts:

```bash
if sudo guestctl inspect disk.img > /dev/null 2>&1; then
    echo "Disk is valid"
else
    echo "Error: Invalid disk image"
    exit 1
fi
```

---

## Troubleshooting

### Permission Denied

**Problem:** `Error: permission denied`

**Solution:** Run with sudo:
```bash
sudo guestctl inspect disk.img
```

### Appliance Failed to Launch

**Problem:** `Error: Failed to launch appliance`

**Solutions:**
1. Check KVM is available:
   ```bash
   ls -l /dev/kvm
   sudo usermod -aG kvm $USER
   ```

2. Enable verbose mode to see details:
   ```bash
   sudo guestctl -v inspect disk.img
   ```

3. Ensure qemu-img is installed:
   ```bash
   sudo apt-get install qemu-utils
   ```

### No OS Detected

**Problem:** `⚠️  No operating systems detected`

**Possible causes:**
- Disk is not bootable
- Disk is encrypted (LUKS)
- Unsupported OS type
- Corrupted disk image

**Check:**
```bash
# Check filesystem types
sudo guestctl filesystems disk.img

# Verify disk is readable
qemu-img info disk.img
```

### File Not Found

**Problem:** `Error: File not found: /path/to/file`

**Solutions:**
1. Verify file exists:
   ```bash
   sudo guestctl ls disk.img /path/to
   ```

2. Check filesystem is mounted:
   ```bash
   sudo guestctl -v cat disk.img /path/to/file
   ```

---

## Performance Tips

### 1. Use JSON Output for Large Results

JSON parsing is faster than human-readable output for large datasets:

```bash
# Faster for 1000+ packages
sudo guestctl packages --json disk.img | jq '.total'
```

### 2. Limit Results Early

Use `--limit` to avoid processing unnecessary data:

```bash
# Only need 10 results
sudo guestctl packages --limit 10 disk.img
```

### 3. Filter Efficiently

Combine filter and limit for best performance:

```bash
sudo guestctl packages --filter nginx --limit 5 disk.img
```

---

## Integration Examples

### Ansible Playbook

```yaml
---
- name: Inspect VM disk image
  hosts: localhost
  tasks:
    - name: Get OS information
      shell: guestctl inspect --json /var/lib/libvirt/images/vm.qcow2
      register: vm_info
      become: yes

    - name: Parse OS details
      set_fact:
        os_type: "{{ (vm_info.stdout | from_json).operating_systems[0].type }}"
        hostname: "{{ (vm_info.stdout | from_json).operating_systems[0].hostname }}"

    - name: Display info
      debug:
        msg: "VM {{ hostname }} is running {{ os_type }}"
```

### Shell Script

```bash
#!/bin/bash
# vm-audit.sh - Audit all VM disks

for disk in /var/lib/libvirt/images/*.qcow2; do
    echo "Auditing: $disk"

    # Get OS info
    info=$(sudo guestctl inspect --json "$disk")

    hostname=$(echo "$info" | jq -r '.operating_systems[0].hostname // "unknown"')
    distro=$(echo "$info" | jq -r '.operating_systems[0].distro // "unknown"')

    # Count packages
    pkg_count=$(sudo guestctl packages --json "$disk" | jq '.total')

    echo "  Hostname: $hostname"
    echo "  OS: $distro"
    echo "  Packages: $pkg_count"
    echo
done
```

---

## See Also

- **[Python Bindings](python-bindings.md)** - Use GuestCtl from Python
- **[Ergonomic API](../api/ergonomic-design.md)** - Type-safe Rust API
- **[Quick Priorities](../development/quick-priorities.md)** - Implementation guide
- **[Enhancement Roadmap](../development/enhancement-roadmap.md)** - Future features

---

**Built with GuestCtl** 🚀
