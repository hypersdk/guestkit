# GuestKit documentation

## Start here

| Guide | Description |
|-------|-------------|
| [Getting started](user-guides/getting-started.md) | Build, install, first commands |
| [CLI reference](user-guides/cli-guide.md) | Command index → topic guides & `guestkit --help` |
| [Quick reference](user-guides/quick-reference.md) | Cheat sheet |
| [FAQ](user-guides/faq.md) | Common questions |
| [Troubleshooting](user-guides/troubleshooting.md) | Fixes for common issues |

## Features

| Guide | Description |
|-------|-------------|
| [TUI dashboard](features/tui-enhancements.md) | Two-tier tabs, **Assurance** (doctor/migrate-plan), fix-plan preview — [zyvor.dev/guestkit](https://zyvor.dev/guestkit) |
| [File explorer](features/explore/EXPLORE-QUICKSTART.md) | `guestkit explore` |
| [Interactive shell](user-guides/interactive-mode.md) | REPL mode |
| [Security profiles](user-guides/profiles.md) | Security, compliance, migration profiles |
| [Migration assurance](features/migration-assurance.md) | Doctor, migrate-plan, fleet, policy, forensic diff |
| [Industry use cases](INDUSTRY_USE_CASES.md) | Real-world scenarios, PM/TA view, Zyvor product stack |
| [Fix plans](features/fix-plans.md) | Offline patch workflow |
| [Export formats](features/export-formats.md) | JSON, YAML, HTML, PDF |
| [Python bindings](user-guides/python-bindings.md) | PyO3 API |
| [VM migration](user-guides/vm-migration.md) | hyper2kvm integration |
| [KubeVirt + Zeus OS](features/kubevirt-integration.md) | In-cluster boot inspect API (pure Rust, not libguestfs) |
| [Guest Control Fabric](features/guest-control-fabric.md) | Transport ladder, airgap QGA install, Agent Doctor, capability contract |

## Deployment

| Guide | Description |
|-------|-------------|
| [Run from GHCR](guides/DOCKER.md#published-images-ghcr) | Pull `ghcr.io/hypersdk/*` images, `docker compose up`, or Helm |
| [Docker](guides/DOCKER.md) | Container usage (web stack + CLI) |
| [Remote deploy](guides/DEPLOY-REMOTE.md) | SSH deploy to Linux hosts + web console access |
| [RPM build](development/RPM-BUILD.md) | Fedora/RHEL packages |

## Architecture & project

| Guide | Description |
|-------|-------------|
| [Architecture](architecture/overview.md) | How GuestKit is structured |
| [Roadmap](development/roadmap.md) | Planned work |
| [Changelog](development/CHANGELOG.md) | Version history |
| [Contributing](development/CONTRIBUTING.md) | How to contribute |

## Zyvor / Enterprise

| Guide | Description |
|-------|-------------|
| [Open source vs Enterprise](ce-vs-enterprise.md) | GitHub vs production platform |
| [Enterprise guide](zyvor-enterprise.md) | Sales, SLAs, full HyperSDK suite |

## Examples

See [`examples/`](../examples/) in the repository root.
