# Visual Output Guide

Complete reference for guestctl's beautiful terminal output system.

## Overview

guestctl v0.3.1+ features a sophisticated visual output system with:
- **Killer Summary View** - Quick boxed OS summary
- **Emoji Icons** - Visual indicators for different components
- **Color Coding** - Semantic colors for easy scanning
- **Hierarchical Display** - Clear section organization
- **Status Indicators** - At-a-glance status information

## Killer Summary View

The summary box appears first, showing critical information at a glance:

```
┌────────────────────────────────────────────────────────┐
│ Ubuntu 22.04.3 LTS                                      │
│ Type: linux | Arch: x86_64 | Hostname: webserver-prod │
│ Packages: deb | Init: systemd                          │
└────────────────────────────────────────────────────────┘
```

### Summary Components

| Component | Description | Example |
|-----------|-------------|---------|
| Product Name | Full OS product name | Ubuntu 22.04.3 LTS, Windows 11 Pro |
| OS Type | Operating system type | linux, windows, freebsd |
| Architecture | CPU architecture | x86_64, aarch64, i386 |
| Hostname | System hostname | webserver-prod, WIN-DESKTOP |
| Package Format | Package system | deb, rpm, pacman, msi |
| Init System | Boot system | systemd, sysvinit, upstart |

### Summary Colors

- **Product Name**: Bright Green - Most important information
- **Type**: Green text
- **Architecture**: Cyan - Technical detail
- **Hostname**: Blue - Identification
- **Packages**: Magenta - System property
- **Init**: Orange/Yellow - System property

## Color Coding System

guestctl uses semantic colors to convey meaning:

### Primary Colors

| Color | RGB | Usage | Meaning |
|-------|-----|-------|---------|
| **Bright Green** | (0, 255, 0) | OS product, active services, secure settings | ✅ Positive, Active, Secure |
| **Bright Red** | (255, 0, 0) | Errors, insecure settings, disabled features | ❌ Issues, Problems, Insecure |
| **Orange** | (255, 165, 0) | Section headers, key information, warnings | ⚠️ Important, Notice |
| **Bright Cyan** | (0, 255, 255) | Architecture, technical details | ℹ️ Technical Info |
| **Bright Blue** | (0, 100, 255) | Hostnames, identifiers | 🔷 Identification |
| **Magenta** | (255, 0, 255) | Package formats, special properties | 💜 Properties |
| **Yellow** | (255, 255, 0) | Init systems, secondary warnings | ⚡ System Info |
| **Bright Black** | (128, 128, 128) | Unknown values, disabled items, separators | ⚫ Unknown, Disabled |
| **Bright White** | (255, 255, 255) | Primary text, values | 📄 Default Text |

### Color Usage Examples

**Security Indicators:**
```
SSH Configuration:
  Port: 22                              (white - neutral)
  PermitRootLogin: no                   (green - secure)
  PasswordAuthentication: yes           (red - insecure)
  PubkeyAuthentication: yes             (green - secure)
```

**Service Status:**
```
Systemd Services:
  ✓ nginx.service                       (green - enabled)
  ✗ apache2.service                     (red - disabled)
  • unknown.service                     (gray - unknown)
```

**Network Interfaces:**
```
Network Configuration:
  📡 Interface: eth0 (UP)               (green - active)
      IP: 192.168.1.100                 (white - value)
      DHCP: disabled                    (orange - notice)
  📡 Interface: eth1 (DOWN)             (red - inactive)
```

## Emoji Reference

### Section Headers

| Emoji | Meaning | Usage |
|-------|---------|-------|
| 💾 | Block Devices | Physical/virtual disks |
| 🗂 | Partitions | Disk partitions |
| ⚙️ | Partition Scheme | GPT, MBR |
| 📁 | Filesystems | ext4, NTFS, XFS, etc. |
| 🖥️ | Operating Systems | OS detection results |
| 👥 | User Accounts | System users |
| 🔐 | SSH Configuration | SSH settings |
| ⚙️ | Systemd Services | System services |
| 💻 | Language Runtimes | Programming languages |
| 🐳 | Container Runtimes | Docker, Podman |
| 💾 | LVM Configuration | Logical volumes |
| 🌐 | Network Configuration | Network interfaces |
| ⚙️ | System Configuration | System settings |

### Component Icons

**Operating Systems:**
| Emoji | OS Type | Example |
|-------|---------|---------|
| 🐧 | Linux | Ubuntu, Fedora, Debian |
| 🪟 | Windows | Windows 10, 11, Server |
| 👹 | FreeBSD | FreeBSD |
| 🍎 | macOS | macOS (when supported) |

**Package Managers:**
| Emoji | Format | Distributions |
|-------|--------|---------------|
| 🔴 | RPM | Fedora, RHEL, CentOS |
| 📦 | DEB | Ubuntu, Debian |
| 📦 | Pacman | Arch Linux |
| 🪟 | MSI | Windows |

**Language Runtimes:**
| Emoji | Runtime | |
|-------|---------|---|
| 🐍 | Python | python, python3 |
| ☕ | Java | java, openjdk |
| 🟢 | Node.js | node, nodejs |
| 💎 | Ruby | ruby |
| 🔷 | Go | go, golang |
| 🐪 | Perl | perl |
| 🟦 | TypeScript | typescript |
| ⚙️ | Rust | rust, cargo |

**Container Runtimes:**
| Emoji | Runtime | |
|-------|---------|---|
| 🐳 | Docker | docker |
| 🦭 | Podman | podman |
| 📦 | containerd | containerd |
| 🔷 | CRI-O | crio |
| ☸️ | Kubernetes | kubectl (when k8s detected) |

**Status Indicators:**
| Emoji | Meaning | Color |
|-------|---------|-------|
| ✓ | Enabled/Active | Green |
| ✗ | Disabled/Inactive | Red |
| ⚠ | Warning | Orange/Yellow |
| ℹ | Information | Cyan |
| ▶ | Running | Green |
| ■ | Stopped | Red |
| • | Neutral/Unknown | Gray |

**Network & Security:**
| Emoji | Component | |
|-------|-----------|---|
| 📡 | Network Interface | |
| 🌐 | DNS Server | |
| 🔒 | HTTPS/SSL | |
| 🔓 | Insecure | |
| 🛡️ | Firewall | |
| 🔑 | SSH Key | |
| 🔐 | Encrypted | |

## Output Sections

### 1. Block Devices Section

```
💾 Block Devices
────────────────────────────────────────────────────────────
  ▪ /dev/sda 8589934592 bytes (8.59 GB)
    • Read-only: yes
```

**Elements:**
- **Section Icon**: 💾 (orange)
- **Separator**: 60 dashes (gray)
- **Device Bullet**: ▪ (orange)
- **Device Name**: /dev/sda (bright white, bold)
- **Size**: bytes + GB (gray for bytes, calculated GB)
- **Properties**: • bullet (gray) for sub-properties

### 2. Partitions Section

```
🗂  Partitions
────────────────────────────────────────────────────────────
  📦 /dev/sda3
    • Size:   8574189056 bytes (8.57 GB)
```

**Elements:**
- **Section Icon**: 🗂 (orange)
- **Partition Icon**: 📦 (orange)
- **Partition Name**: /dev/sda3 (bright white, bold)
- **Properties**: • bullet with property name (gray), value (white)

### 3. Partition Scheme Section

```
⚙️  Partition Scheme
────────────────────────────────────────────────────────────
  🔷 GPT (GUID Partition Table)
```

**Scheme Icons:**
- 🔷 GPT
- 🔶 MBR
- 📋 Other schemes

### 4. Filesystems Section

```
📁 Filesystems
────────────────────────────────────────────────────────────
  🐧 /dev/sda3 ext4
    • UUID:  311182bd-f262-4081-8a2d-56624799dbad
    • Label: rootfs
```

**Filesystem Icons:**
- 🐧 ext2/ext3/ext4 (Linux)
- 🪟 ntfs (Windows)
- 🔷 xfs
- 🌳 btrfs
- 📁 other filesystems

### 5. Operating Systems Section

```
🖥️  Operating Systems
────────────────────────────────────────────────────────────
    🐧 Type:         linux
    📦 Distribution: ubuntu
    🏷️ Product:      Ubuntu 22.04.3 LTS
    🏠 Hostname:     webserver-prod
    🔴 Packages:     rpm
    ⚡ Init system:  systemd
```

**Property Icons:**
- 🐧/🪟/👹 OS Type icon
- 📦 Distribution
- 🏷️ Product name
- 🏠 Hostname
- 🔴/📦 Package format
- ⚡ Init system
- 🐧 Kernel
- 💾 Disk usage

### 6. Subsections

**System Configuration:**
```
    ⚙️  System Configuration
    ────────────────────────────────────────────────────────
      🕐 Timezone:   America/New_York
      🌍 Locale:     en_US.UTF-8
      🛡️ SELinux:    enforcing              (green)
```

**Network Configuration:**
```
    🌐 Network Configuration
    ────────────────────────────────────────────────────────
      📡 Interface: eth0
        • IP: 192.168.1.100/24
        • Gateway: 192.168.1.1
        • DHCP: disabled
      🌐 DNS Servers:
        • 8.8.8.8
        • 8.8.4.4
```

**User Accounts:**
```
    👥 User Accounts
    ────────────────────────────────────────────────────────
      Regular Users:
        • john (UID: 1000)
        • jane (UID: 1001)
      System Users: 15
```

**Language Runtimes:**
```
    💻 Language Runtimes
    ────────────────────────────────────────────────────────
      🐍 Python 3.10.12
      ☕ OpenJDK 11.0.21
      🟢 Node.js v18.19.0
```

**Container Runtimes:**
```
    🐳 Container Runtimes
    ────────────────────────────────────────────────────────
      🐳 Docker 24.0.7
      🦭 Podman 4.3.1
```

## Output Modes

### Pretty Text (Default)

Rich terminal output with emojis and colors:
```bash
guestctl inspect vm.qcow2
```

### JSON Output

Machine-readable structured data:
```bash
guestctl inspect vm.qcow2 --output json
```

**Features:**
- No emojis or color codes
- Valid JSON structure
- Scriptable and parseable
- Progress indicators hidden

### YAML Output

Human-readable structured data:
```bash
guestctl inspect vm.qcow2 --output yaml
```

### CSV Output

Tabular data for spreadsheets:
```bash
guestctl packages vm.qcow2 --output csv
```

## Progress Indicators

### Spinner

During long operations:
```
⠋ Launching appliance...
⠙ Inspecting OS...
⠹ Mounting filesystems...
✓ Complete!
```

**Spinner Frames:** ⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏

### Progress Bar

For operations with known progress:
```
Converting disk format: [████████████████----] 80% (4.2 GB / 5.2 GB)
```

## Customization

### Disable Colors

```bash
# Disable all colors
export NO_COLOR=1
guestctl inspect vm.qcow2

# Or use --no-color flag (planned for v0.4.0)
guestctl inspect vm.qcow2 --no-color
```

### Disable Emojis

```bash
# Use ASCII-only output (planned for v0.4.0)
export GUESTCTL_ASCII=1
guestctl inspect vm.qcow2 --ascii
```

**ASCII Mode Output:**
```
[*] Block Devices
------------------------------------------------------------
  * /dev/sda 8589934592 bytes (8.59 GB)

[*] Operating Systems
------------------------------------------------------------
    [+] Type:         linux
    [+] Distribution: ubuntu
    [+] Product:      Ubuntu 22.04 LTS
```

### Quiet Mode

Minimal output:
```bash
guestctl inspect vm.qcow2 --quiet
# Only shows summary, no details
```

### Verbose Mode

Maximum detail:
```bash
guestctl inspect vm.qcow2 --verbose
# Shows debug information, timing, internal operations
```

## Terminal Compatibility

### Tested Terminals

| Terminal | Emoji Support | Color Support | Notes |
|----------|---------------|---------------|-------|
| GNOME Terminal | ✅ Full | ✅ 256 colors | Recommended |
| Konsole | ✅ Full | ✅ 256 colors | Recommended |
| iTerm2 (macOS) | ✅ Full | ✅ True color | Excellent |
| Windows Terminal | ✅ Full | ✅ True color | Excellent |
| xterm | ⚠️ Limited | ✅ 256 colors | Some emojis missing |
| PuTTY | ⚠️ Limited | ✅ 256 colors | Configure UTF-8 |
| tmux | ✅ Full | ✅ 256 colors | Set `utf8 on` |
| screen | ⚠️ Limited | ✅ 256 colors | Update to latest |

### Enable UTF-8 Support

```bash
# Ensure UTF-8 locale
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# For tmux
tmux set-window-option -g utf8 on

# For PuTTY
# Settings → Translation → UTF-8
```

## Examples

### Security Audit Colors

```bash
guestctl inspect vm.qcow2 --profile security
```

**Color Meanings:**
- **Green SSH settings**: Secure configuration
- **Red SSH settings**: Insecure configuration
- **Green firewall**: Active and configured
- **Red firewall**: Disabled or misconfigured
- **Orange warnings**: Recommendations

### Migration Profile Colors

```bash
guestctl inspect vm.qcow2 --profile migration
```

**Color Highlights:**
- **Orange sections**: Key migration points
- **Cyan technical details**: Device paths, UUIDs
- **White values**: Configuration data
- **Gray notes**: Additional context

## Accessibility

### Screen Reader Support

```bash
# Use --output json for screen reader friendly output
guestctl inspect vm.qcow2 --output json | jq

# Or YAML for structured text
guestctl inspect vm.qcow2 --output yaml
```

### High Contrast Mode

Planned for v0.4.0:
```bash
guestctl inspect vm.qcow2 --high-contrast
```

## Best Practices

1. **Use JSON for automation** - No emojis or colors in JSON output
2. **Pipe to less** - For long output: `guestctl inspect vm.qcow2 | less -R`
3. **Save output** - Use `--output json > report.json` for archival
4. **Check terminal support** - Verify UTF-8 and emoji support before relying on icons
5. **Use verbose mode for debugging** - Get detailed operation information

## Color Palette Reference

### Named Colors (owo-colors)

```rust
// Implementation reference
use owo_colors::OwoColorize;

// Positive/Secure
.green()          // RGB(0, 255, 0)
.bright_green()   // RGB(0, 255, 0)

// Negative/Insecure
.red()            // RGB(255, 0, 0)
.bright_red()     // RGB(255, 0, 0)

// Important/Warnings
.yellow()         // RGB(255, 255, 0)
.truecolor(255, 165, 0)  // Orange

// Information
.cyan()           // RGB(0, 255, 255)
.bright_cyan()    // RGB(0, 255, 255)

// Identification
.blue()           // RGB(0, 100, 255)
.bright_blue()    // RGB(0, 100, 255)

// Properties
.magenta()        // RGB(255, 0, 255)

// Neutral/Unknown
.bright_black()   // RGB(128, 128, 128) - Gray
.dimmed()         // Dim version of current color

// Default
.bright_white()   // RGB(255, 255, 255)
.bold()           // Bold text
```

## Unicode Characters

### Box Drawing

```
┌─┐  Top border
│ │  Sides
└─┘  Bottom border
```

### Bullets

```
▪  Square bullet
•  Circle bullet
▶  Triangle (running)
■  Square (stopped)
```

### Separators

```
────  Horizontal line (60 chars)
│    Vertical line
```

## Future Enhancements (Roadmap)

**v0.4.0:**
- `--no-color` flag
- `--ascii` flag for emoji-free output
- `--high-contrast` mode
- Customizable color themes
- Width detection for responsive layout

**v0.5.0:**
- Interactive color picker for themes
- Export HTML with colors preserved
- Markdown output with GitHub emoji support
- Dark/light mode auto-detection

## Support

For visual output issues:
- GitHub Issues: https://github.com/ssahani/guestkit/issues
- Tag with: `visual`, `colors`, `emojis`, `terminal`
- Include: Terminal name/version, locale settings, screenshot
