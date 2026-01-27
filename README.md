# 🔧 GuestKit

> **A Pure-Rust VM Disk Toolkit** — inspect & manipulate VM disks **without booting them**  
> **🤖 AI-powered diagnostics** (optional) — ask *“why won’t this boot?”* and get actionable fixes

GuestKit is a production-ready toolkit for VM disk inspection and manipulation with **beautiful emoji-enhanced CLI output** and an **interactive TUI dashboard**. Built in pure Rust for safety and performance, it inspects VM disks in seconds and integrates cleanly with [hyper2kvm](https://github.com/ssahani/hyper2kvm) for migration workflows.

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Crates.io](https://img.shields.io/crates/v/guestkit.svg)](https://crates.io/crates/guestkit)
[![PyPI](https://img.shields.io/pypi/v/guestkit.svg)](https://pypi.org/project/guestkit/)
[![Downloads](https://pepy.tech/badge/guestkit)](https://pepy.tech/project/guestkit)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

---

## ✨ Highlights

- 🦀 **Pure Rust** — memory-safe, fast, and pipeline-friendly
- 🎨 **Beautiful TUI** — interactive dashboard with visual analytics + quick navigation
- 🤖 **AI Diagnostics (optional)** — GPT-powered troubleshooting for boot/storage/config issues
- 🐍 **Python Bindings** — native PyO3 bindings for Python workflows
- 💿 **Multi-format** — QCOW2, VMDK, VDI, VHD/VHDX, RAW/IMG/ISO
- ⚡ **Scale-ready** — caching + parallel batch inspection for fleets
- 🔄 **Migration-ready** — fstab/crypttab rewriting and cross-hypervisor prep (via hyper2kvm)
- 🧰 **REPL shell** — interactive mode with many commands for offline changes

---

## 📖 Table of Contents

- [Quick Start](#-quick-start)
- [TUI Dashboard](#-interactive-tui-terminal-user-interface)
- [CLI Examples](#-cli-examples)
- [AI Diagnostics](#-ai-powered-diagnostics-optional)
- [Python API](#-python-api)
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
````

**Rust (recommended for Rust developers):**

```bash
cargo install guestkit
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

---

## 🎨 Interactive TUI (Terminal User Interface)

A professional dashboard for VM inspection with real-time visual analytics.

### What you get

* **Multi-view dashboard**: storage, OS, services, network, security, users, packages, databases, webservers
* **Quick navigation**: vim keys (j/k/g/G), Ctrl+P fuzzy jump, mouse support
* **Search**: `/` search with optional regex / case toggles
* **Configurable**: `~/.config/guestkit/tui.toml`

**Launch:**

```bash
guestctl tui vm.qcow2
```

**Example config:**

```toml
[ui]
show_splash = true
mouse_enabled = true
theme = "default"

[behavior]
default_view = "dashboard"
search_regex_mode = false

[keybindings]
vim_mode = true
quick_jump_enabled = true
```

---

## 📋 CLI Examples

**Basic inspection:**

```bash
guestctl inspect vm.qcow2
```

**Sample output (illustrative):**

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
```

**JSON for automation:**

```bash
guestctl inspect vm.qcow2 --output json | jq '.operating_systems[0].hostname'
```

**Compare two images:**

```bash
guestctl diff vm-before.qcow2 vm-after.qcow2
```

**Batch inspect in parallel:**

```bash
guestctl inspect-batch *.qcow2 --parallel 4
```

---

## 🤖 AI-Powered Diagnostics (Optional)

GuestKit can integrate with OpenAI (feature-gated) to provide natural-language diagnostics based on what GuestKit discovers inside the disk image.

### Build with AI support

```bash
cargo build --release --features ai
```

### Set API key

```bash
export OPENAI_API_KEY='your-key-here'
```

### Use in interactive mode

```bash
guestctl interactive vm.qcow2
```

Example prompts:

* `ai why won't this boot?`
* `ai what security issues do you see?`
* `ai explain the network configuration and likely issues`

Notes:

* AI is **optional** and **off by default**
* Works best when combined with deterministic inspection output (GuestKit provides the facts; AI helps interpret)

---

## 🐍 Python API

GuestKit provides native Python bindings via PyO3 for Python automation and integration.

**Install:**

```bash
pip install guestkit
```

**Example:**

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

    g.shutdown()
```

---

## 💿 Supported Disk Formats

GuestKit/guestctl auto-detects formats and uses the best available path.

### Loop device (fast path)

**Formats:** RAW, IMG, ISO
**Why:** built into Linux, minimal moving parts.

```bash
guestctl inspect disk.raw
guestctl inspect ubuntu.img
guestctl inspect debian.iso
```

### NBD (fallback for advanced formats)

**Formats:** QCOW2, VMDK, VDI, VHD/VHDX
**Why:** common virtual disk formats need QEMU helpers for block access.

```bash
guestctl inspect vm.qcow2
guestctl inspect windows.vmdk
guestctl inspect virtualbox.vdi
```

**Pro tip (speed for repeated inspections):**

```bash
qemu-img convert -O raw vm.qcow2 vm.raw
guestctl inspect vm.raw
```

---

## 🧠 Design & Principles

1. **Rust-first**: safety, predictability, performance
2. **Fast inspection**: pull facts from the image, don’t boot the guest
3. **Automation-friendly**: JSON/YAML outputs for pipelines
4. **Human-friendly**: readable CLI, TUI for interactive triage
5. **Migration-aware**: built to plug into hyper2kvm-style workflows

---

## 🧱 Project Structure

```text
guestkit/
├── Cargo.toml
├── README.md
├── src/
│   ├── core/              # errors, types, helpers
│   ├── disk/              # pure-rust disk + partition primitives
│   ├── guestfs/           # guestfs-compatible style APIs (where applicable)
│   ├── cli/               # guestctl commands, formatters, caching, exporters
│   └── python.rs          # PyO3 bindings
├── docs/                  # guides, architecture, references
├── examples/              # rust + python examples
└── tests/                 # integration tests
```

---

## 🗺️ Roadmap

**Near-term**

* tighter filesystem-level ops (read/write/edit) with robust safety gates
* richer Windows boot diagnostics (EFI/BCD hints, registry-backed checks)
* more migration fixers (fstab/crypttab, net configs, initramfs hints)

**Mid-term**

* broader “no-kernel-module” workflows where feasible
* richer exporters (HTML/PDF/Markdown) with consistent schemas
* deeper integration patterns for hyper2kvm pipelines

---

## 🤝 Contributing

Contributions are welcome.

* Fork the repo
* Create a feature branch
* Add tests where meaningful
* Run:

  ```bash
  cargo fmt
  cargo clippy
  cargo test
  ```
* Open a PR

---

## 📜 License

Licensed under **LGPL-3.0**. See `LICENSE`.

---

## 🔗 Related Projects

* **[hyper2kvm](https://github.com/ssahani/hyper2kvm)** — production-grade VM migration toolkit
* **[hypersdk](https://github.com/ssahani/hypersdk)** — high-performance hypervisor SDK

---

Made with ❤️ for reliable VM operations.

```

If you want this to exactly match your current “mega README” (with every command list, every module list, every table), paste your repo’s current `README.md` and I’ll refactor it **without deleting content**—just reorganize, de-duplicate, and make it read like a sharp product instead of a novel written by a caffeinated kernel.
```
