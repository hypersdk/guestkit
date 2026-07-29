# GuestKit

**Offline VM intelligence and migration assurance.**

[![CI](https://github.com/ssahani/guestkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ssahani/guestkit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/guestkit.svg)](https://crates.io/crates/guestkit)
[![PyPI](https://img.shields.io/pypi/v/hypersdk-guestkit.svg)](https://pypi.org/project/hypersdk-guestkit/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## 📖 Feature Guide

**[GuestKit — Customer Feature Guide](docs/guestkit-customer-feature-guide.md)** — a complete, customer-facing reference covering all **63 features** across **10 areas**, grounded in the product's actual capabilities. Also available as a print-ready **[PDF](docs/guestkit-customer-feature-guide.pdf)**.

**[Customer manual (page-by-page)](docs/customer/README.md)** — getting started, admin basics, and a guide for every product surface (PDFs under `docs/customer/pdf/`).

Inspect **QCOW2, VMDK, and RAW** images without powering them on. Score boot readiness, generate hypervisor-aware fix plans, and explore disks from a carbon-themed TUI — **Rust control plane, no libguestfs appliance** (uses host NBD/loop for mount).

```text
┌──────────────────────────────────────────────────────────────┐
│  Interfaces   guestkit CLI · guestctl TUI · Python bindings  │
├──────────────────────────────────────────────────────────────┤
│  Engine       Rust parsers + boot scoring · NBD/loop mount layer   │
├──────────────────────────────────────────────────────────────┤
│  Outputs      JSON · YAML · HTML · PDF · CI gate artifacts   │
└──────────────────────────────────────────────────────────────┘
```

---

## Why GuestKit

| Problem | GuestKit answer |
|---------|-----------------|
| "Will it boot?" answered at power-on | Offline doctor score before cutover |
| guestfish scripts don't scale | Structured assurance APIs + exports |
| Migration surprises cost weekends | Fix plans with driver injections |
| Fleet drift is invisible | `fleet analyze` and forensic diff |
| No VM boot for deep inspection | Carbon TUI explores partitions in place |

**Pairs with:** [hyper2kvm](https://github.com/hypersdk/hyper2kvm) for VMware → KVM pipelines.

---

## 🆕 What's New — In-Guest Agent

GuestKit now runs **inside** the guest too, not just offline against the disk — same evidence schema, same fix-plan format, over the existing virtio-serial QGA channel.

- **Offline Windows install, no boot required** — `agent-inject --windows` writes the `GuestKitAgent` service and the virtio-serial (`vioser`) driver straight into the `SYSTEM` hive via hivex.
- **Stock `qemu-guest-agent` gets out of the way automatically** — any `QEMU-GA`/`qemu-ga`/`QEMUGuestAgent` service found is disabled so GuestKit answers the channel uncontended, while still speaking QGA-compatible commands so KubeVirt/libvirt notice nothing.
- **Converted-image driver fix** — deletes the stale cached PCI devnode so Windows re-detects the virtio-serial device and runs a full driver install on next boot, instead of staying stuck on "no driver."
- **Generic QGA passthrough** — `guestkit-rpc` exposes every agent RPC method through the standard QGA channel, so host-side automation needs only `virsh qemu-agent-command`.

Details, protocol reference, and the Linux path: [docs/features/guest-agent.md](docs/features/guest-agent.md) · [Protocol 1.3](docs/features/guestkit-agent-protocol-1.3.md)

---

## Platform at a Glance

| Layer | What's in the repo |
|-------|-------------------|
| **Core** | Rust disk engine + assurance APIs — `crates/`, `src/` |
| **CLI** | `guestkit` + `guestctl` — doctor, migrate-plan, fleet |
| **In-guest agent** | Linux + Windows, protocol 1.3 — `agent-inject`, `agent-proxy`, `agent-call` |
| **TUI** | Carbon-themed multi-view dashboard |
| **Python** | `hypersdk-guestkit` on PyPI |
| **K8s** | KubeVirt integration hooks — `k8s/` |
| **Web stack** | Prebuilt GHCR images — `ghcr.io/hypersdk/{zyvor-ui,zyvor-api,guestkit-worker}` |
| **Deploy** | Docker/Helm, remote deploy scripts — `deploy/` |

---

## Quick Start

```bash
cargo install guestkit   # guestkit + guestctl

guestkit doctor vm.qcow2 --target proxmox --explain
# → boot assurance score · blockers · root-cause chain

guestkit migrate-plan vm.vmdk --target proxmox --export plan.yaml
# → migration score · driver injections · fix plan

guestctl tui vm.qcow2
# → carbon TUI · Assurance · fix-plan preview
```

| Scenario | Path |
|----------|------|
| Getting started | [docs/user-guides/getting-started.md](docs/user-guides/getting-started.md) |
| CLI reference | [docs/user-guides/cli-guide.md](docs/user-guides/cli-guide.md) |
| Migration assurance | [docs/features/migration-assurance.md](docs/features/migration-assurance.md) |
| In-guest agent (Linux + Windows) | [docs/features/guest-agent.md](docs/features/guest-agent.md) |
| CE vs Enterprise | [docs/ce-vs-enterprise.md](docs/ce-vs-enterprise.md) |

**Web console:** self-hosted via GHCR or Helm. First-login credentials for packaged installs are documented in [remote deploy](docs/guides/DEPLOY-REMOTE.md#web-console-access) — change defaults before exposing to a network.

---

## Run from GHCR (prebuilt images)

The web stack is published to the GitHub Container Registry under **`ghcr.io/hypersdk`** — **public images, no `docker login` required.**

| Image | Role |
|-------|------|
| `ghcr.io/hypersdk/zyvor-ui` | Web console + login |
| `ghcr.io/hypersdk/zyvor-api` | API backend |
| `ghcr.io/hypersdk/guestkit-worker` | Disk-inspection worker |

Tags: `latest`, `vX.Y.Z` (e.g. `v0.3.13`), per-commit SHA. Bring the whole stack up straight from GHCR:

```bash
docker compose -f deploy/docker-compose.ghcr.yml pull
docker compose -f deploy/docker-compose.ghcr.yml up -d
open http://localhost:8088          # web console
```

> **Eval only:** this stack runs without authentication. Do not expose it beyond localhost.
> For production, use `deploy/docker-compose.prod.example.yml` — see [Docker guide](docs/guides/DOCKER.md#production-checklist).

For clusters, use the [Helm chart](deploy/helm/zyvor). Full details: [docs/guides/DOCKER.md → Published images](docs/guides/DOCKER.md#published-images-ghcr).

---

## Three Commands Before Cutover

| Command | Outcome |
|---------|---------|
| `guestkit doctor` | Boot assurance score + blockers |
| `guestkit migrate-plan` | Executable fix plan YAML |
| `guestctl tui` | Interactive assurance workspace |

---

## Documentation

| Goal | Document |
|------|----------|
| Docs index | [docs/README.md](docs/README.md) |
| Run from GHCR / Docker | [docs/guides/DOCKER.md](docs/guides/DOCKER.md#published-images-ghcr) |
| Remote deploy | [docs/guides/DEPLOY-REMOTE.md](docs/guides/DEPLOY-REMOTE.md) |
| User stories | [docs/USER_STORIES.md](docs/USER_STORIES.md) |
| Industry use cases | [docs/INDUSTRY_USE_CASES.md](docs/INDUSTRY_USE_CASES.md) |
| Architecture | [docs/architecture/overview.md](docs/architecture/overview.md) |
| Full index | [docs/INDEX.md](docs/INDEX.md) |

→ [zyvor.dev/guestkit](https://zyvor.dev/guestkit) · [Demo video](https://www.youtube.com/watch?v=ZYCz6HN7bXE) · [Full Zyvor platform](https://zyvor.dev)

---

## Development

See project docs for CI, testing, and contribution guidelines. Historical build summaries in the repo root are snapshots — **`docs/` and this README are authoritative.**

---

## License

See [LICENSE](LICENSE) or project-specific licensing files in `docs/legal/`.
