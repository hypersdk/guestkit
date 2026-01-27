# 📚 GuestKit Documentation Index

Welcome to the GuestKit documentation! This index helps you find the information you need quickly.

---

## 📖 Quick Navigation

### For New Users
- [README](../README.md) — Project overview and quick start
- [User Guides](user-guides/) — Step-by-step tutorials
- [Examples](../examples/) — Code examples

### For Developers
- [Contributing Guide](../CONTRIBUTING.md) — How to contribute
- [Development Docs](development/) — Development guides
- [API Reference](api/) — API documentation
- [Architecture](architecture/) — System design

### For Security
- [Security Policy](../SECURITY.md) — Security practices and reporting
- [Changelog](../CHANGELOG.md) — Version history and changes

---

## 📂 Documentation Structure

### `/` (Root)
Core documentation files that should be easily accessible:

| File | Description |
|------|-------------|
| [README.md](../README.md) | Project overview, features, installation, quick start |
| [CHANGELOG.md](../CHANGELOG.md) | Version history and release notes |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Contribution guidelines and development setup |
| [SECURITY.md](../SECURITY.md) | Security policy and vulnerability reporting |

### `/docs/` (Documentation Hub)

#### [User Guides](user-guides/)
End-user documentation and tutorials:

| File | Description |
|------|-------------|
| [getting-started.md](user-guides/getting-started.md) | Installation and first steps |
| [cli-usage.md](user-guides/cli-usage.md) | Command-line interface guide |
| [tui-guide.md](user-guides/tui-guide.md) | Interactive TUI dashboard guide |
| [interactive-shell.md](user-guides/interactive-shell.md) | REPL shell usage |
| [python-guide.md](user-guides/python-guide.md) | Python bindings guide |
| [security-profiles.md](user-guides/security-profiles.md) | Security analysis profiles |
| [export-guide.md](user-guides/export-guide.md) | Exporting reports and data |

#### [Development](development/)
Developer documentation and implementation details:

| File | Description |
|------|-------------|
| [COMMANDS_SUMMARY.md](development/COMMANDS_SUMMARY.md) | Summary of all CLI commands |
| [DOCUMENTATION_ENHANCEMENTS.md](development/DOCUMENTATION_ENHANCEMENTS.md) | Documentation improvements history |
| [DOCUMENTATION_UPDATE_SUMMARY.md](development/DOCUMENTATION_UPDATE_SUMMARY.md) | Documentation update summary |
| [INSPECT_ENHANCED_IMPROVEMENTS.md](development/INSPECT_ENHANCED_IMPROVEMENTS.md) | Enhanced inspection features |
| [Q1-2026-IMPLEMENTATION-START.md](development/Q1-2026-IMPLEMENTATION-START.md) | Q1 2026 roadmap and implementation |
| [test-coverage-initiative-complete.md](development/test-coverage-initiative-complete.md) | Test coverage initiative results |
| [complete-session-summary.md](development/complete-session-summary.md) | Complete development session summary |

#### [API Reference](api/)
API documentation for library usage:

| File | Description |
|------|-------------|
| [core-api.md](api/core-api.md) | Core API reference |
| [disk-api.md](api/disk-api.md) | Disk manipulation API |
| [inspection-api.md](api/inspection-api.md) | VM inspection API |
| [python-api.md](api/python-api.md) | Python bindings API |

#### [Architecture](architecture/)
System design and architectural decisions:

| File | Description |
|------|-------------|
| [overview.md](architecture/overview.md) | System architecture overview |
| [disk-formats.md](architecture/disk-formats.md) | Supported disk formats and handling |
| [inspection-engine.md](architecture/inspection-engine.md) | Inspection engine design |
| [security-profiles.md](architecture/security-profiles.md) | Profile system architecture |

#### [Features](features/)
Feature documentation and specifications:

| File | Description |
|------|-------------|
| [tui-dashboard.md](features/tui-dashboard.md) | TUI dashboard features |
| [interactive-shell.md](features/interactive-shell.md) | Interactive shell features |
| [ai-diagnostics.md](features/ai-diagnostics.md) | AI-powered diagnostics |
| [export-formats.md](features/export-formats.md) | Export format specifications |
| [caching.md](features/caching.md) | Inspection caching system |
| [batch-processing.md](features/batch-processing.md) | Batch inspection features |

#### [Marketing](marketing/)
Project marketing and communication materials:

| File | Description |
|------|-------------|
| [announcement.md](marketing/announcement.md) | Project announcement |
| [feature-comparison.md](marketing/feature-comparison.md) | Comparison with alternatives |
| [use-cases.md](marketing/use-cases.md) | Real-world use cases |

---

## 🔧 Examples

### `/examples/` (Code Examples)

#### Rust Examples
Located in `/examples/`:

| File | Description |
|------|-------------|
| [inspect_vm.rs](../examples/inspect_vm.rs) | Basic VM inspection |
| [inspect_os_typed.rs](../examples/inspect_os_typed.rs) | Type-safe OS inspection |
| [system_info.rs](../examples/system_info.rs) | System information extraction |
| [mount_and_explore.rs](../examples/mount_and_explore.rs) | Mount and explore filesystems |
| [disk_forensics.rs](../examples/disk_forensics.rs) | Disk forensics example |
| [fluent_api.rs](../examples/fluent_api.rs) | Fluent API usage |
| [create_disk_fluent.rs](../examples/create_disk_fluent.rs) | Disk creation with fluent API |
| [lvm_luks_demo.rs](../examples/lvm_luks_demo.rs) | LVM and LUKS handling |
| [vm_clone_prep.rs](../examples/vm_clone_prep.rs) | VM cloning preparation |
| [command_and_archive.rs](../examples/command_and_archive.rs) | Command execution and archiving |
| [convert_disk.rs](../examples/convert_disk.rs) | Disk format conversion |
| [detect_format.rs](../examples/detect_format.rs) | Disk format detection |
| [retry_example.rs](../examples/retry_example.rs) | Retry mechanism example |

#### Python Examples
Located in `/examples/python/`:

| File | Description |
|------|-------------|
| [test_all_apis.py](../examples/python/test_all_apis.py) | Comprehensive API test |
| [test_enhancements.py](../examples/python/test_enhancements.py) | Enhanced features test |

#### Batch Examples
Located in `/examples/batch/`:

| Directory | Description |
|-----------|-------------|
| [examples/batch/](../examples/batch/) | Batch processing examples |

---

## 📝 Configuration

### TUI Configuration
- **Config File**: `~/.config/guestkit/tui.toml`
- **Example**: [tui-config-example.toml](tui-config-example.toml)

---

## 🎯 Quick Reference

### Common Tasks

**Get started quickly:**
```bash
# Install
cargo install guestkit

# Inspect a VM
guestctl inspect vm.qcow2

# Launch TUI
guestctl tui vm.qcow2

# Interactive shell
guestctl interactive vm.qcow2
```

**Find documentation:**
- Installation → [User Guides: Getting Started](user-guides/getting-started.md)
- CLI Usage → [User Guides: CLI Usage](user-guides/cli-usage.md)
- TUI Help → [User Guides: TUI Guide](user-guides/tui-guide.md)
- Python → [User Guides: Python Guide](user-guides/python-guide.md)
- API Reference → [API Reference](api/)

**For developers:**
- Contributing → [CONTRIBUTING.md](../CONTRIBUTING.md)
- Architecture → [Architecture](architecture/)
- Development Docs → [Development](development/)

---

## 🔍 Search Tips

### Finding Information

**By Topic:**
- **Installation** → README, User Guides
- **CLI Commands** → CLI Usage Guide, Commands Summary
- **TUI Features** → TUI Guide, Features/TUI Dashboard
- **Python API** → Python Guide, API Reference
- **Security** → Security Profiles, SECURITY.md
- **Development** → CONTRIBUTING.md, Development Docs
- **Architecture** → Architecture Docs

**By Role:**
- **End Users** → README, User Guides, Examples
- **Developers** → CONTRIBUTING, Development, Architecture, API Reference
- **Security Auditors** → SECURITY.md, Security Profiles
- **Contributors** → CONTRIBUTING, Development Docs

---

## 📊 Documentation Stats

- **Total Documentation Files**: 30+
- **User Guides**: 7
- **Development Docs**: 7
- **API References**: 4
- **Architecture Docs**: 4
- **Feature Specs**: 6
- **Examples**: 20+
- **Languages**: English
- **Last Updated**: 2026-01-27

---

## 🔄 Updates

### Recent Changes
- 2026-01-27: Reorganized root directory, moved development docs to docs/development/
- 2026-01-27: Enhanced README with comprehensive documentation
- 2026-01-27: Created INDEX.md for better navigation
- 2026-01-26: Added complete session summary and test coverage docs
- 2026-01-25: Initial documentation structure

### Contributing to Docs

To improve documentation:
1. Read [CONTRIBUTING.md](../CONTRIBUTING.md)
2. Make changes to relevant markdown files
3. Update this index if adding/removing files
4. Submit a PR with clear description

---

## 📧 Help & Support

**Found an issue in the docs?**
- Open an issue: [GitHub Issues](https://github.com/ssahani/guestkit/issues)
- Start a discussion: [GitHub Discussions](https://github.com/ssahani/guestkit/discussions)

**Want to contribute?**
- Read: [CONTRIBUTING.md](../CONTRIBUTING.md)
- Check: [Development Docs](development/)

---

Made with ❤️ by the GuestKit team.
