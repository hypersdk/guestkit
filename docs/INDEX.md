# 📚 GuestKit Documentation Index

Welcome to the GuestKit documentation! This index helps you find the information you need quickly.

---

## 📖 Quick Navigation

### For New Users
- **[README](../README.md)** — Project overview and quick start
- **[Worker Quickstart](guides/quickstart.md)** — Get the worker running in 5 minutes
- **[CLI Guide](CLI-GUIDE.md)** — Complete CLI reference for guestkit-worker

### For Guestctl Users
- **[User Guides](user-guides/)** — Step-by-step tutorials for guestctl
- **[Examples](../examples/)** — Code examples

### For Worker System
- **[Worker Index](WORKER-INDEX.md)** — Complete worker system documentation
- **[REST API Reference](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)** — API documentation
- **[Prometheus Metrics](phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)** — Metrics guide

### For Developers
- **[Contributing Guide](development/CONTRIBUTING.md)** — How to contribute
- **[Development Docs](development/)** — Development guides
- **[Complete System Summary](development/COMPLETE-SYSTEM-SUMMARY.md)** — Full implementation overview

### For Security
- **[Security Policy](../SECURITY.md)** — Security practices and reporting
- **[Changelog](development/CHANGELOG.md)** — Version history and changes

---

## 📂 Documentation Structure

### `/` (Root)
Core project files:

| File | Description |
|------|-------------|
| [README.md](../README.md) | Project overview, features, installation |
| [SECURITY.md](../SECURITY.md) | Security policy and vulnerability reporting |
| [RELEASE-0.1.0.md](../RELEASE-0.1.0.md) | Release notes for v0.1.0 |
| [CRATES-PUBLISHED.md](../CRATES-PUBLISHED.md) | Published crates information |
| [CLI-DEVELOPMENT-COMPLETE.md](../CLI-DEVELOPMENT-COMPLETE.md) | CLI development summary |
| [DOCS-ORGANIZATION-SUMMARY.md](../DOCS-ORGANIZATION-SUMMARY.md) | Documentation organization notes |
| [install.sh](../install.sh) | Installation script |

### `/docs/` (Documentation Hub)

Main navigation:
- **[WORKER-INDEX.md](WORKER-INDEX.md)** — Worker system documentation index
- **[INDEX.md](INDEX.md)** — This file (complete navigation)
- **[README.md](README.md)** — Main docs index (guestctl)

---

## 🚀 Worker System Documentation

### Quick Start
| File | Description |
|------|-------------|
| [Worker Quickstart](guides/quickstart.md) | Get started in 5 minutes |
| [CLI Guide](CLI-GUIDE.md) | Complete CLI reference |
| [Docker Quickstart](guides/DOCKER-QUICKSTART.md) | Run in containers |
| [Kubernetes Deployment](guides/K8S-DEPLOYMENT.md) | Deploy at scale |

### Phase Documentation

#### Phase 1: Foundation
- **[Phase 1 Complete](phases/phase-1/PHASE-1-COMPLETE.md)** — Job Protocol v1.0, Worker daemon, File transport

#### Phase 2: Handlers
- **[Phase 2 Complete](phases/phase-2/PHASE-2-COMPLETE.md)** — Echo, Inspect, Profile handlers

#### Phase 3: Integration
- **[Phase 3 Complete](phases/phase-3/PHASE-3-COMPLETE.md)** — Guestkit library integration
- **[Integration Summary](phases/phase-3/PHASE-3-INTEGRATION-SUMMARY.md)** — Integration details

#### Phase 4: Production Features
- **[Phase 4 Overview](phases/phase-4/PHASE-4-OVERVIEW.md)** — Security, observability, scalability

**Phase 4.1: SHA256 Checksum Verification** ✅
- **[Feature Guide](phases/phase-4/PHASE-4.1-CHECKSUM-VERIFICATION.md)**
- **[Session Summary](phases/phase-4/PHASE-4.1-SESSION-SUMMARY.md)**

**Phase 4.2: Prometheus Metrics** ✅
- **[Feature Guide](phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)** — 13 comprehensive metrics
- **[Session Summary](phases/phase-4/PHASE-4.2-SESSION-SUMMARY.md)**

**Phase 4.3: REST API** ✅
- **[Feature Guide](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)** — 6 REST endpoints
- **[Session Summary](phases/phase-4/PHASE-4.3-SESSION-SUMMARY.md)**

### Features Documentation

**Worker System:**
- **[Worker Implementation](features/worker/WORKER-IMPLEMENTATION-COMPLETE.md)** — Distributed job processing

**Explore Command:**
- **[Explore Quickstart](features/explore/EXPLORE-QUICKSTART.md)**
- **[Command Reference](features/explore/EXPLORE-COMMAND.md)**
- **[Development Summary](features/explore/EXPLORE-DEVELOPMENT-SUMMARY.md)**
- **[Complete Summary](features/explore/EXPLORE-COMPLETE-SUMMARY.md)**

**TUI File Browser:**
- **[Files View](features/tui/TUI-FILES-VIEW.md)**
- **[Navigation](features/tui/TUI-FILES-NAVIGATION.md)**
- **[Preview](features/tui/TUI-FILES-PREVIEW-INFO.md)**
- **[Filtering](features/tui/TUI-FILES-FILTER.md)**

### Development Documentation

| File | Description |
|------|-------------|
| [Complete System Summary](development/COMPLETE-SYSTEM-SUMMARY.md) | Overall status |
| [CLI Development Summary](development/CLI-DEVELOPMENT-SUMMARY.md) | CLI implementation |
| [Session Logs](development/SESSION-CONTINUATION-2026-01-30.md) | Latest work |
| [Session Summary](development/SESSION-CONTINUATION-SUMMARY.md) | Session continuation notes |
| [Final Session Summary](development/FINAL-SESSION-SUMMARY.md) | Final session notes |
| [RPM Build](development/RPM-BUILD.md) | RPM packaging |
| [Docker Build](development/DOCKER-BUILD-FIX-SUMMARY.md) | Docker build fixes |
| [Docker Fix Session](development/DOCKER-FIX-SESSION.md) | Docker troubleshooting |
| [Contributing Guide](development/CONTRIBUTING.md) | How to contribute |
| [Changelog](development/CHANGELOG.md) | Version history |
| [Commands Summary](development/COMMANDS_SUMMARY.md) | Command reference |
| [Implementations](development/implementations.md) | Implementation notes |

### Guides

| File | Description |
|------|-------------|
| [Worker Quickstart](guides/quickstart.md) | Get started quickly |
| [Docker](guides/DOCKER.md) | Docker deployment |
| [Docker Quickstart](guides/DOCKER-QUICKSTART.md) | Quick Docker setup |
| [Docker Test Results](guides/DOCKER-TEST-RESULTS.md) | Docker testing |
| [Kubernetes Deployment](guides/K8S-DEPLOYMENT.md) | K8s deployment |
| [K8s Success](guides/K8S-DEPLOYMENT-SUCCESS.md) | K8s deployment success |

### Other Documentation

| File | Description |
|------|-------------|
| [Job Protocol](job-protocol-readme.md) | Job Protocol specification |

---

## 📚 Guestctl CLI Documentation

### User Guides
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

### API Reference
API documentation for library usage:

| File | Description |
|------|-------------|
| [core-api.md](api/core-api.md) | Core API reference |
| [disk-api.md](api/disk-api.md) | Disk manipulation API |
| [inspection-api.md](api/inspection-api.md) | VM inspection API |
| [python-api.md](api/python-api.md) | Python bindings API |

### Architecture
System design and architectural decisions:

| File | Description |
|------|-------------|
| [overview.md](architecture/overview.md) | System architecture overview |
| [disk-formats.md](architecture/disk-formats.md) | Supported disk formats and handling |
| [inspection-engine.md](architecture/inspection-engine.md) | Inspection engine design |
| [security-profiles.md](architecture/security-profiles.md) | Profile system architecture |

### Features
Feature documentation and specifications:

| File | Description |
|------|-------------|
| [tui-dashboard.md](features/tui-dashboard.md) | TUI dashboard features |
| [interactive-shell.md](features/interactive-shell.md) | Interactive shell features |
| [ai-diagnostics.md](features/ai-diagnostics.md) | AI-powered diagnostics |
| [export-formats.md](features/export-formats.md) | Export format specifications |
| [caching.md](features/caching.md) | Inspection caching system |
| [batch-processing.md](features/batch-processing.md) | Batch inspection features |

### Marketing
Project marketing and communication materials:

| File | Description |
|------|-------------|
| [announcement.md](marketing/announcement.md) | Project announcement |
| [feature-comparison.md](marketing/feature-comparison.md) | Comparison with alternatives |
| [use-cases.md](marketing/use-cases.md) | Real-world use cases |

---

## 🔧 Examples

### Rust Examples
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
| [job-with-checksum.json](../examples/job-with-checksum.json) | Job with checksum verification |

### Python Examples
Located in `/examples/python/`:

| File | Description |
|------|-------------|
| [test_all_apis.py](../examples/python/test_all_apis.py) | Comprehensive API test |
| [test_enhancements.py](../examples/python/test_enhancements.py) | Enhanced features test |

---

## 🎯 Quick Reference

### Common Tasks

**Guestkit Worker:**
```bash
# Install
cargo install guestkit-worker

# Start daemon
guestkit-worker daemon --transport http

# Submit job
guestkit-worker submit -o guestkit.inspect -i vm.qcow2

# Check status
guestkit-worker list
```

**Guestctl CLI:**
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

### Find Documentation

**Worker System:**
- Installation → [Worker Quickstart](guides/quickstart.md)
- CLI Usage → [CLI Guide](CLI-GUIDE.md)
- REST API → [REST API Reference](phases/phase-4/PHASE-4.3-REST-API-TRANSPORT.md)
- Metrics → [Prometheus Metrics](phases/phase-4/PHASE-4.2-PROMETHEUS-METRICS.md)

**Guestctl:**
- Installation → [User Guides: Getting Started](user-guides/getting-started.md)
- CLI Usage → [User Guides: CLI Usage](user-guides/cli-usage.md)
- TUI Help → [User Guides: TUI Guide](user-guides/tui-guide.md)
- Python → [User Guides: Python Guide](user-guides/python-guide.md)

**For developers:**
- Contributing → [CONTRIBUTING.md](development/CONTRIBUTING.md)
- Architecture → [Architecture](architecture/)
- Development Docs → [Development](development/)

---

## 🔍 Search Tips

### Finding Information

**By Topic:**
- **Worker System** → Worker Index, CLI Guide, Phase Documentation
- **Installation** → README, Quickstart Guides
- **CLI Commands** → CLI Guide, Commands Summary
- **TUI Features** → TUI Guide, Features/TUI
- **Python API** → Python Guide, API Reference
- **Security** → Security Profiles, SECURITY.md
- **Development** → CONTRIBUTING, Development Docs
- **Architecture** → Architecture Docs

**By Role:**
- **End Users** → README, User Guides, CLI Guide
- **DevOps** → Docker Guides, Kubernetes Deployment, Worker Quickstart
- **Developers** → CONTRIBUTING, Development, Architecture, API Reference
- **Security Auditors** → SECURITY.md, Checksum Verification, Security Profiles
- **Contributors** → CONTRIBUTING, Development Docs

---

## 📊 Documentation Stats

- **Total Documentation Files**: 70+
- **Worker Documentation**: 25+
- **User Guides**: 7
- **Development Docs**: 12+
- **API References**: 4
- **Architecture Docs**: 4
- **Feature Specs**: 12+
- **Examples**: 20+
- **Languages**: English
- **Last Updated**: 2026-01-31

---

## 🔄 Updates

### Recent Changes
- **2026-01-31**: Published crates to crates.io (v0.1.0)
- **2026-01-31**: Added comprehensive CLI (7 commands)
- **2026-01-31**: Reorganized all documentation into clean structure
- **2026-01-30**: Completed Phase 4.3 (REST API)
- **2026-01-30**: Completed Phase 4.2 (Prometheus Metrics)
- **2026-01-30**: Completed Phase 4.1 (SHA256 Checksum Verification)
- **2026-01-27**: Enhanced README with comprehensive documentation
- **2026-01-27**: Created INDEX.md for better navigation

### Contributing to Docs

To improve documentation:
1. Read [CONTRIBUTING.md](development/CONTRIBUTING.md)
2. Make changes to relevant markdown files
3. Update this index if adding/removing files
4. Submit a PR with clear description

---

## 📧 Help & Support

**Found an issue in the docs?**
- Open an issue: [GitHub Issues](https://github.com/ssahani/guestkit/issues)
- Start a discussion: [GitHub Discussions](https://github.com/ssahani/guestkit/discussions)

**Want to contribute?**
- Read: [CONTRIBUTING.md](development/CONTRIBUTING.md)
- Check: [Development Docs](development/)

**Published Crates:**
- [guestkit on crates.io](https://crates.io/crates/guestkit)
- [guestkit-worker on crates.io](https://crates.io/crates/guestkit-worker)
- [guestkit-job-spec on crates.io](https://crates.io/crates/guestkit-job-spec)

---

Made with ❤️ by the GuestKit team.

**Status**: Production Ready v0.1.0 🚀
