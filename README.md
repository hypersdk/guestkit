# 🔧 GuestKit

> **A Pure-Rust VM Disk Toolkit** — inspect & manipulate VM disks **without booting them**
> **🤖 AI-powered diagnostics** (optional) — ask *"why won't this boot?"* and get actionable fixes

GuestKit is a production-ready toolkit for VM disk inspection and manipulation with **beautiful emoji-enhanced CLI output** and an **interactive TUI dashboard**. Built in pure Rust for safety and performance, it inspects VM disks in seconds and integrates cleanly with [hyper2kvm](https://github.com/ssahani/hyper2kvm) for migration workflows.

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/ssahani/guestkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ssahani/guestkit/actions/workflows/ci.yml)
[![RPM Build](https://github.com/ssahani/guestkit/actions/workflows/rpm.yml/badge.svg)](https://github.com/ssahani/guestkit/actions/workflows/rpm.yml)
[![Crates.io](https://img.shields.io/crates/v/guestkit.svg)](https://crates.io/crates/guestkit)
[![PyPI](https://img.shields.io/pypi/v/guestkit.svg)](https://pypi.org/project/guestkit/)
[![Downloads](https://pepy.tech/badge/guestkit)](https://pepy.tech/project/guestkit)

---

## ✨ Highlights

- 🦀 **Pure Rust** — memory-safe, fast, and pipeline-friendly
- 🎨 **Beautiful TUI** — interactive dashboard with visual analytics + quick navigation (Ctrl+P fuzzy jump!)
- 🤖 **AI Diagnostics (optional)** — GPT-powered troubleshooting for boot/storage/config issues
- 🐍 **Python Bindings** — native PyO3 bindings for Python workflows
- 💿 **Multi-format** — QCOW2, VMDK, VDI, VHD/VHDX, RAW/IMG/ISO
- ⚡ **Scale-ready** — caching + parallel batch inspection for fleets
- 🔄 **Migration-ready** — fstab/crypttab rewriting and cross-hypervisor prep (via hyper2kvm)
- 🧰 **REPL shell** — interactive mode with 20+ commands for offline changes
- 📊 **Security Profiles** — built-in security, compliance, hardening, and performance analysis
- 🔧 **Fix Plans** — offline patch preview with bash/ansible export (inspect → plan → review → execute)
- 📤 **Export Anywhere** — JSON, YAML, HTML, PDF reports for automation

---

## 📖 Table of Contents

- [Quick Start](#-quick-start)
- [TUI Dashboard](#-interactive-tui-terminal-user-interface)
- [CLI Examples](#-cli-examples)
- [Interactive Shell](#-interactive-shell)
- [Security Profiles](#-security-profiles)
- [Fix Plans](#-offline-patch--fix-plans)
- [AI Diagnostics](#-ai-powered-diagnostics-optional)
- [Python API](#-python-api)
- [Export Formats](#-export-formats)
- [Supported Disk Formats](#-supported-disk-formats)
- [Design & Principles](#-design--principles)
- [Project Structure](#-project-structure)
- [Roadmap](#-roadmap)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🚀 Quick Start

### Installation

**Python (recommended for Python users):**
```bash
pip install guestkit
```

**Rust (recommended for Rust developers):**
```bash
cargo install guestkit
```

**RPM (Fedora/RHEL/CentOS):**
```bash
# Download latest RPM from releases
sudo dnf install guestkit-*.rpm

# Or build from source (see RPM-BUILD.md)
```

**From source:**
```bash
git clone https://github.com/ssahani/guestkit
cd guestkit
cargo build --release
```

### One-liners

**🎨 TUI dashboard:**
```bash
guestctl tui vm.qcow2
```

**🔍 Inspect quickly:**
```bash
guestctl inspect vm.qcow2
```

**🎮 Interactive shell:**
```bash
guestctl interactive vm.qcow2
```

**📊 Run security profile:**
```bash
guestctl profile security vm.qcow2
```

**🔄 Batch process VMs:**
```bash
guestctl inspect-batch *.qcow2 --parallel 4 --output json
```

---

## 🎨 Interactive TUI (Terminal User Interface)

A professional dashboard for VM inspection with real-time visual analytics.

### Features

* **📊 Multi-view dashboard**:
  - **Dashboard** — System overview with health score
  - **Network** — Interfaces, DNS, firewall rules
  - **Packages** — Installed software, version tracking
  - **Services** — systemd services, status
  - **Databases** — PostgreSQL, MySQL, MongoDB, Redis, SQLite
  - **Web Servers** — nginx, Apache, Caddy, lighttpd
  - **Security** — SELinux, AppArmor, fail2ban, SSH keys
  - **Issues** — Critical/high/medium findings from profiles
  - **Storage** — LVM, RAID, fstab/mount points
  - **Users** — User accounts, sudo access
  - **Kernel** — Modules, parameters
  - **Profiles** — Security, migration, performance, compliance, hardening

* **⚡ Quick navigation**:
  - Vim keys (j/k/g/G/Ctrl+d/Ctrl+u)
  - Ctrl+P fuzzy jump menu
  - Tab/Shift+Tab for views
  - Number keys (1-9) to jump to views
  - Mouse support (click, scroll)

* **🔍 Search**:
  - `/` to search current view
  - Regex mode toggle
  - Case-sensitive toggle
  - Search history

* **📤 Export**:
  - Press 'e' to open export menu
  - Export to JSON, YAML (HTML/PDF coming soon)
  - Export current view or full report

* **⚙️ Configurable**:
  - Config file: `~/.config/guestkit/tui.toml`
  - Customize colors, keybindings, default view
  - Enable/disable splash screen, stats bar

### Launch

```bash
guestctl tui vm.qcow2
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next/previous view |
| `1-9` | Jump to specific view |
| `Ctrl+P` | Fuzzy jump menu |
| `j/k` | Scroll down/up |
| `g/G` | Jump to top/bottom |
| `Ctrl+d/u` | Page down/up |
| `/` | Start search |
| `Esc` | Cancel search/close menus |
| `e` | Export menu |
| `s` | Cycle sort mode |
| `t` | Toggle stats bar |
| `r` | Refresh data |
| `?` | Help screen |
| `q` | Quit |

### Example Config

```toml
[ui]
show_splash = true
splash_duration_ms = 800
show_stats_bar = true
theme = "default"
mouse_enabled = true

[behavior]
default_view = "dashboard"
auto_refresh_seconds = 0
search_case_sensitive = false
search_regex_mode = false
max_bookmarks = 20
page_scroll_lines = 10

[keybindings]
vim_mode = true
quick_jump_enabled = true
```

---

## 📋 CLI Examples

### Basic Inspection

**Inspect VM disk:**
```bash
guestctl inspect vm.qcow2
```

**Sample output:**
```
┌────────────────────────────────────────────────────────┐
│ Ubuntu 22.04 LTS                                        │
│ Type: linux | Arch: x86_64 | Hostname: webserver-prod  │
└────────────────────────────────────────────────────────┘

💾 Block Devices
────────────────────────────────────────────────────────────
  ▪ /dev/sda 8589934592 bytes (8.59 GB)

🖥️  Operating Systems
────────────────────────────────────────────────────────────
    🐧 Type:         linux
    📦 Distribution: ubuntu
    🏷️ Product:      Ubuntu 22.04 LTS
    🏠 Hostname:     webserver-prod
    🔴 Packages:     deb
    ⚡ Init system:  systemd

🌐 Network Interfaces
────────────────────────────────────────────────────────────
    eth0: 192.168.1.100/24 (up)
    lo: 127.0.0.1/8 (up)

📦 Packages: 1,234 installed
🔧 Services: 45 systemd services
🔐 Security: SELinux enforcing, firewalld active
```

### JSON Output for Automation

```bash
# Get hostname
guestctl inspect vm.qcow2 --output json | jq '.operating_systems[0].hostname'

# Get all network interfaces
guestctl inspect vm.qcow2 --output json | jq '.network_interfaces[].name'

# Check if firewall is enabled
guestctl inspect vm.qcow2 --output json | jq '.firewall.enabled'

# List all databases
guestctl inspect vm.qcow2 --output json | jq '.databases[].name'
```

### Export to Files

```bash
# Export as JSON
guestctl inspect vm.qcow2 --export report.json

# Export as YAML
guestctl inspect vm.qcow2 --export report.yaml

# Export as HTML (requires --features html)
guestctl inspect vm.qcow2 --export report.html

# Export as PDF (requires --features pdf)
guestctl inspect vm.qcow2 --export report.pdf
```

### Comparison

```bash
# Compare two VM images
guestctl diff vm-before.qcow2 vm-after.qcow2

# Output differences in JSON
guestctl diff vm-before.qcow2 vm-after.qcow2 --output json
```

### Batch Operations

```bash
# Inspect multiple VMs in parallel
guestctl inspect-batch *.qcow2 --parallel 4

# Batch with JSON output
guestctl inspect-batch *.qcow2 --parallel 4 --output json > results.json

# Batch with caching (faster for repeated inspections)
guestctl inspect-batch *.qcow2 --parallel 4 --cache
```

---

## 🧰 Interactive Shell

The interactive shell provides a REPL environment for exploring and modifying VM disks.

### Launch

```bash
guestctl interactive vm.qcow2
```

### Available Commands

| Command | Description |
|---------|-------------|
| `ls [path]` | List files and directories |
| `cat <file>` | Display file contents |
| `head <file> [n]` | Show first n lines of file |
| `tail <file> [n]` | Show last n lines of file |
| `stat <path>` | Show file/directory information |
| `find <path> <name>` | Search for files |
| `grep <pattern> <file>` | Search file contents |
| `download <src> <dest>` | Download file from VM disk |
| `upload <src> <dest>` | Upload file to VM disk |
| `mkdir <path>` | Create directory |
| `rm <path>` | Remove file |
| `rmdir <path>` | Remove directory |
| `touch <path>` | Create empty file |
| `chmod <mode> <path>` | Change file permissions |
| `chown <owner> <path>` | Change file ownership |
| `mount` | List mounted filesystems |
| `df` | Show disk space usage |
| `inspect` | Run full inspection |
| `packages` | List installed packages |
| `services` | List systemd services |
| `users` | List user accounts |
| `network` | Show network configuration |
| `security` | Show security info |
| `ai <prompt>` | Ask AI for help (if enabled) |
| `history` | Show command history |
| `clear` | Clear screen |
| `exit` or `quit` | Exit shell |

### Example Session

```
guestctl> ls /etc
total 1024 items
drwxr-xr-x  2 root root  4096 fstab
drwxr-xr-x  2 root root  4096 hostname
...

guestctl> cat /etc/hostname
webserver-prod

guestctl> grep -r "database" /etc
/etc/my.cnf: database=prod
/etc/postgresql/postgresql.conf: database_dir=/var/lib/postgresql

guestctl> packages | grep postgresql
postgresql-14.5-1.el8
postgresql-client-14.5-1.el8

guestctl> download /var/log/syslog ./syslog.txt
Downloaded /var/log/syslog to ./syslog.txt (2.4 MB)

guestctl> ai why is the database service failing?
Analyzing system configuration...

Based on the inspection:
1. PostgreSQL is installed but the systemd service is 'failed'
2. The data directory /var/lib/postgresql is not mounted
3. /etc/fstab shows the database volume is commented out

Recommended fix:
1. Uncomment the database volume in /etc/fstab
2. Or ensure the LVM volume group is available at boot
```

### Shell Features

- **Tab completion** — command and path completion
- **Command history** — up/down arrows, searchable with Ctrl+R
- **Bookmarks** — save frequently used paths
- **Timing** — see how long each command takes
- **Colorized output** — syntax highlighting for files
- **Progress indicators** — for long-running operations

---

## 📊 Security Profiles

GuestKit includes built-in security analysis profiles that scan VM disks for security, compliance, and performance issues.

### Available Profiles

| Profile | Purpose |
|---------|---------|
| **Security** | Identify security vulnerabilities and misconfigurations |
| **Compliance** | Check adherence to standards (PCI-DSS, HIPAA, etc.) |
| **Hardening** | Find hardening opportunities |
| **Performance** | Detect performance bottlenecks |
| **Migration** | Assess migration readiness |

### Run a Profile

```bash
# Run security profile
guestctl profile security vm.qcow2

# Run with JSON output
guestctl profile security vm.qcow2 --output json

# Run all profiles
guestctl profile all vm.qcow2
```

### Example Output

```
🔐 Security Profile Report
═══════════════════════════════════════════════════════════

Overall Risk: HIGH ⚠️

Critical Issues (3):
  🔴 Root login via SSH is enabled
  🔴 Firewall is disabled
  🔴 SELinux is in permissive mode

High Priority (5):
  🟠 Weak password hashing algorithm (MD5)
  🟠 Unpatched kernel vulnerabilities (CVE-2023-1234)
  🟠 World-writable directories found
  🟠 SSH allows password authentication
  🟠 No intrusion detection system (AIDE/fail2ban)

Medium Priority (8):
  🟡 Default SSH port (22) in use
  🟡 IPv6 is enabled but not configured
  ...

Recommendations:
  1. Disable root SSH login (PermitRootLogin no)
  2. Enable and configure firewalld
  3. Set SELinux to enforcing mode
  4. Update password hashing to SHA512
  5. Apply latest kernel security patches
```

### Profile Findings

Each profile generates findings with:
- **Risk Level** — Critical, High, Medium, Low, Info
- **Title** — Short description
- **Description** — Detailed explanation
- **Remediation** — How to fix the issue
- **References** — Links to documentation/CVEs

---

## 🔧 Offline Patch & Fix Plans

GuestKit can generate, preview, and apply fix plans for security hardening, compliance, and migration preparation. This workflow enables safe, reviewable changes with complete separation of concerns.

### Workflow

```
Inspect → Diagnose → Generate Plan → Review → Approve → Execute
```

### Generate a Fix Plan

```bash
# From a security profile (Phase 2 - coming soon)
guestctl profile security vm.qcow2 --plan security-fixes.yaml
```

### Preview the Plan

```bash
# Human-readable preview
guestctl plan preview security-fixes.yaml

# Show as unified diff
guestctl plan preview security-fixes.yaml --diff

# Summary only
guestctl plan preview security-fixes.yaml --summary
```

### Export to Scripts

```bash
# Export as bash script
guestctl plan export security-fixes.yaml --format bash --output fixes.sh

# Export as Ansible playbook
guestctl plan export security-fixes.yaml --format ansible --output fixes.yml

# Export as JSON or YAML
guestctl plan export security-fixes.yaml --format json --output fixes.json
```

### Validate and Apply

```bash
# Validate plan (dry-run simulation)
guestctl plan validate security-fixes.yaml

# Apply with interactive prompts
guestctl plan apply security-fixes.yaml --interactive

# Apply with backup
guestctl plan apply security-fixes.yaml --backup /backup/vm-state

# Rollback if needed
guestctl plan rollback /backup/vm-state --vm vm.qcow2
```

### Show Statistics

```bash
guestctl plan stats security-fixes.yaml
```

### Key Features

- **Safety First**: Preview changes, validate plans, create backups
- **Auditability**: Plans are version-controllable YAML/JSON artifacts
- **Scriptability**: Export to bash/ansible for integration
- **Reversibility**: Rollback capabilities for safe recovery
- **Collaboration**: Security team generates, ops team applies

For detailed documentation, see [Fix Plans Documentation](docs/features/fix-plans.md).

---

## 🤖 AI-Powered Diagnostics (Optional)

GuestKit can integrate with OpenAI (feature-gated) to provide natural-language diagnostics based on what GuestKit discovers inside the disk image.

### Build with AI Support

```bash
cargo build --release --features ai
```

### Set API Key

```bash
export OPENAI_API_KEY='your-key-here'
```

### Use in Interactive Mode

```bash
guestctl interactive vm.qcow2
```

Example prompts:
```
ai why won't this boot?
ai what security issues do you see?
ai explain the network configuration and likely issues
ai how can I improve database performance?
ai is this VM ready for migration to KVM?
```

### Use in CLI

```bash
guestctl inspect vm.qcow2 --ai-analyze
```

### Notes

- AI is **optional** and **off by default**
- Requires OpenAI API key
- Works best when combined with deterministic inspection output (GuestKit provides the facts; AI helps interpret)
- Sends inspection data to OpenAI API (be mindful of sensitive data)

---

## 🐍 Python API

GuestKit provides native Python bindings via PyO3 for Python automation and integration.

### Install

```bash
pip install guestkit
```

### Basic Example

```python
from guestkit import Guestfs

with Guestfs() as g:
    g.add_drive_ro("disk.qcow2")
    g.launch()

    roots = g.inspect_os()
    for root in roots:
        print("Type:", g.inspect_get_type(root))
        print("Distro:", g.inspect_get_distro(root))
        print("Hostname:", g.inspect_get_hostname(root))
        print("Packages:", g.inspect_get_package_format(root))

    g.shutdown()
```

### Enhanced Inspection

```python
from guestkit import Guestfs

with Guestfs() as g:
    g.add_drive_ro("vm.qcow2")
    g.launch()

    roots = g.inspect_os()
    root = roots[0]

    # Network configuration
    interfaces = g.inspect_network(root)
    for iface in interfaces:
        print(f"{iface.name}: {iface.address}/{iface.netmask}")

    # Installed packages
    packages = g.inspect_packages(root)
    print(f"Package manager: {packages.manager}")
    print(f"Total packages: {packages.package_count}")

    # Services
    services = g.inspect_systemd_services(root)
    for svc in services:
        print(f"{svc.name}: {svc.state}")

    # Databases
    databases = g.inspect_databases(root)
    for db in databases:
        print(f"{db.name}: {db.data_dir}")

    # Security
    security = g.inspect_security(root)
    print(f"SELinux: {security.selinux}")
    print(f"AppArmor: {security.apparmor}")
    print(f"fail2ban: {security.fail2ban}")

    g.shutdown()
```

### Batch Processing

```python
from guestkit import Guestfs
import glob
import json

results = []

for vm_path in glob.glob("vms/*.qcow2"):
    with Guestfs() as g:
        g.add_drive_ro(vm_path)
        g.launch()

        roots = g.inspect_os()
        if roots:
            root = roots[0]
            results.append({
                "vm": vm_path,
                "hostname": g.inspect_get_hostname(root),
                "os": g.inspect_get_product_name(root),
                "packages": g.inspect_packages(root).package_count,
            })

        g.shutdown()

# Save results
with open("vm_inventory.json", "w") as f:
    json.dump(results, f, indent=2)
```

---

## 📤 Export Formats

GuestKit supports multiple export formats for reports and automation.

### Supported Formats

| Format | Extension | Use Case |
|--------|-----------|----------|
| **JSON** | `.json` | Automation, APIs, parsing |
| **YAML** | `.yaml` | Configuration, human-readable |
| **HTML** | `.html` | Web viewing, documentation |
| **PDF** | `.pdf` | Reports, archival |

### Export from CLI

```bash
# JSON (default)
guestctl inspect vm.qcow2 --export report.json

# YAML
guestctl inspect vm.qcow2 --export report.yaml

# HTML (requires --features html)
guestctl inspect vm.qcow2 --export report.html

# PDF (requires --features pdf)
guestctl inspect vm.qcow2 --export report.pdf
```

### Export from TUI

1. Press `e` to open export menu
2. Select format (JSON, YAML, HTML, PDF)
3. Enter filename
4. Press Enter to export

### Export from Interactive Shell

```
guestctl> export json report.json
Exported current inspection to report.json

guestctl> export yaml report.yaml
Exported current inspection to report.yaml
```

### Export Format Details

**JSON:**
- Machine-readable
- Complete data structure
- Ideal for automation pipelines
- Can be queried with `jq`

**YAML:**
- Human-readable
- Configuration-friendly
- Preserves structure
- Comments supported

**HTML:**
- Rich formatting
- Interactive tables
- Charts and graphs
- View in browser

**PDF:**
- Portable documents
- Professional reports
- Print-friendly
- Archival quality

---

## 💿 Supported Disk Formats

GuestKit auto-detects formats and uses the best available path.

### Loop Device (Fast Path)

**Formats:** RAW, IMG, ISO
**Why:** Built into Linux, minimal moving parts.

```bash
guestctl inspect disk.raw
guestctl inspect ubuntu.img
guestctl inspect debian.iso
```

### NBD (Fallback for Advanced Formats)

**Formats:** QCOW2, VMDK, VDI, VHD/VHDX
**Why:** Common virtual disk formats need QEMU helpers for block access.

```bash
guestctl inspect vm.qcow2
guestctl inspect windows.vmdk
guestctl inspect virtualbox.vdi
guestctl inspect hyperv.vhdx
```

### Performance Tips

**For repeated inspections, convert to RAW:**
```bash
qemu-img convert -O raw vm.qcow2 vm.raw
guestctl inspect vm.raw
```

**Use caching for batch operations:**
```bash
guestctl inspect-batch *.qcow2 --cache
```

**Parallel processing:**
```bash
guestctl inspect-batch *.qcow2 --parallel 8
```

---

## 🧠 Design & Principles

1. **Rust-first** — Safety, predictability, performance
2. **Fast inspection** — Pull facts from the image, don't boot the guest
3. **Automation-friendly** — JSON/YAML outputs for pipelines
4. **Human-friendly** — Readable CLI, TUI for interactive triage
5. **Migration-aware** — Built to plug into hyper2kvm-style workflows
6. **Zero-trust** — Never execute guest code, always read-only by default
7. **Comprehensive** — OS, network, packages, services, security, storage, users

---

## 🧱 Project Structure

```text
guestkit/
├── Cargo.toml              # Rust dependencies and features
├── README.md               # This file
├── LICENSE                 # LGPL-3.0 license
├── src/
│   ├── core/               # Errors, types, helpers
│   ├── disk/               # Pure-Rust disk + partition primitives
│   ├── guestfs/            # VM inspection and operations APIs
│   │   ├── inspect.rs      # Basic OS inspection
│   │   ├── inspect_enhanced.rs  # Enhanced inspection (network, services, etc.)
│   │   └── operations.rs   # File operations (read, write, download, upload)
│   ├── cli/                # CLI application
│   │   ├── commands/       # Command implementations
│   │   ├── tui/            # Terminal UI (ratatui)
│   │   │   ├── views/      # TUI views (dashboard, network, security, etc.)
│   │   │   ├── app.rs      # TUI application state
│   │   │   ├── ui.rs       # UI rendering
│   │   │   └── config.rs   # TUI configuration
│   │   ├── shell/          # Interactive shell
│   │   ├── profiles/       # Security/compliance profiles
│   │   ├── formatters/     # Output formatters (JSON, YAML, etc.)
│   │   ├── exporters/      # Export engines (HTML, PDF)
│   │   └── cache.rs        # Inspection caching
│   ├── python.rs           # PyO3 Python bindings
│   └── lib.rs              # Library entry point
├── docs/                   # Documentation
│   ├── architecture.md     # Architecture overview
│   ├── profiles.md         # Profile system documentation
│   └── examples.md         # Usage examples
├── examples/               # Rust and Python examples
│   ├── basic_inspection.rs
│   ├── batch_processing.rs
│   └── python_example.py
└── tests/                  # Integration tests
    ├── test_inspection.rs
    └── test_operations.rs
```

---

## 🗺️ Roadmap

### Near-term

- ✅ Interactive TUI dashboard with fuzzy jump navigation
- ✅ Security, compliance, hardening, performance profiles
- ✅ Export to JSON, YAML, HTML, PDF
- ✅ Interactive shell with 20+ commands
- ✅ Python bindings via PyO3
- 🔄 Tighter filesystem-level ops (read/write/edit) with robust safety gates
- 🔄 Richer Windows boot diagnostics (EFI/BCD hints, registry-backed checks)
- 🔄 More migration fixers (fstab/crypttab, net configs, initramfs hints)

### Mid-term

- 🔮 Broader "no-kernel-module" workflows where feasible
- 🔮 Plugin system for custom profiles and exporters
- 🔮 Cloud integration (inspect VMs in AWS/Azure/GCP)
- 🔮 Real-time monitoring integration (Prometheus metrics)
- 🔮 GUI application (GTK/Qt)

### Long-term

- 🔮 Distributed inspection (cluster mode)
- 🔮 Machine learning for anomaly detection
- 🔮 Container image inspection (Docker, OCI)
- 🔮 Bootloader repair automation
- 🔮 Snapshot and rollback capabilities

---

## 🤝 Contributing

Contributions are welcome! Here's how to get started:

### Development Setup

```bash
# Clone the repository
git clone https://github.com/ssahani/guestkit
cd guestkit

# Build
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- inspect test.qcow2
```

### Code Quality

Before submitting a PR:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Run tests
cargo test

# Run with all features
cargo test --all-features

# Check documentation
cargo doc --no-deps --open
```

### Contribution Guidelines

1. **Fork the repo** and create a feature branch
2. **Write tests** for new features
3. **Update documentation** if changing APIs
4. **Follow Rust conventions** (use `cargo fmt` and `cargo clippy`)
5. **Keep commits atomic** and write clear commit messages
6. **Open a PR** with a clear description of changes

### Areas for Contribution

- 🐛 Bug fixes and error handling improvements
- 📚 Documentation and examples
- 🧪 Test coverage expansion
- 🎨 TUI enhancements and new views
- 🔌 New export formats
- 🔍 Additional security profiles
- 🌍 Internationalization
- 🪟 Windows guest support improvements

---

## 📜 License

Licensed under **LGPL-3.0-or-later**. See [LICENSE](LICENSE) for details.

This allows:
- ✅ Use in commercial products
- ✅ Modification and redistribution
- ✅ Private use
- ⚠️ Must disclose source for modifications
- ⚠️ Must use same license for derivatives

---

## 🔗 Related Projects

- **[hyper2kvm](https://github.com/ssahani/hyper2kvm)** — Production-grade VM migration toolkit (Hyper-V to KVM)
- **[hypersdk](https://github.com/ssahani/hypersdk)** — High-performance hypervisor SDK (Go)

---

## 📚 Additional Resources

- **Documentation:** [docs/](docs/)
- **Examples:** [examples/](examples/)
- **Issue Tracker:** [GitHub Issues](https://github.com/ssahani/guestkit/issues)
- **Discussions:** [GitHub Discussions](https://github.com/ssahani/guestkit/discussions)

---

## 🙏 Acknowledgments

GuestKit builds on the shoulders of giants:

- **QEMU** — NBD support for disk formats
- **ratatui** — Beautiful terminal UI framework
- **PyO3** — Seamless Rust-Python integration
- The Rust community for amazing libraries and tools

---

## 📊 Project Stats

- **Language:** Rust 🦀
- **Lines of Code:** ~15,000+
- **Dependencies:** Minimal (lean dependency tree)
- **Test Coverage:** Comprehensive
- **Build Time:** Fast (parallel builds)
- **Binary Size:** Small (optimized release builds)

---

Made with ❤️ for reliable VM operations.

**Questions?** Open an [issue](https://github.com/ssahani/guestkit/issues) or start a [discussion](https://github.com/ssahani/guestkit/discussions).
