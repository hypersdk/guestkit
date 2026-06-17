# GuestKit

**Offline VM intelligence and migration assurance.**

Inspect **QCOW2, VMDK, and RAW** images without powering them on. Score boot probability, generate hypervisor-aware fix plans, and explore disks from a carbon-themed TUI — pure Rust, no libguestfs appliance.

```text
┌──────────────────────────────────────────────────────────────┐
│  Interfaces   guestkit CLI · guestctl TUI · Python bindings  │
├──────────────────────────────────────────────────────────────┤
│  Engine       Pure Rust disk parser · boot scoring · fixes   │
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

**Pairs with:** [hyper2kvm](https://github.com/ssahani/hyper2kvm-) for VMware → KVM pipelines.

---

## Platform at a Glance

| Layer | What's in the repo |
|-------|-------------------|
| **Core** | Pure Rust disk engine — `crates/`, `src/` |
| **CLI** | `guestkit` + `guestctl` — doctor, migrate-plan, fleet |
| **TUI** | Carbon-themed multi-view dashboard |
| **Python** | `hypersdk-guestkit` on PyPI |
| **K8s** | KubeVirt integration hooks — `k8s/` |
| **Deploy** | Docker, remote deploy scripts — `deploy/` |

---

## Quick Start

```bash
cargo install guestkit   # guestkit + guestctl

guestkit doctor vm.qcow2 --target proxmox --explain
# → boot probability · blockers · root-cause chain

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
| CE vs Enterprise | [docs/ce-vs-enterprise.md](docs/ce-vs-enterprise.md) |

---

## Three Commands Before Cutover

| Command | Outcome |
|---------|---------|
| `guestkit doctor` | Boot probability + blockers |
| `guestkit migrate-plan` | Executable fix plan YAML |
| `guestctl tui` | Interactive assurance workspace |

---

## Documentation

| Goal | Document |
|------|----------|
| Docs index | [docs/README.md](docs/README.md) |
| User stories | [docs/USER_STORIES.md](docs/USER_STORIES.md) |
| Architecture | [docs/architecture/overview.md](docs/architecture/overview.md) |
| Full index | [docs/INDEX.md](docs/INDEX.md) |

→ [zyvor.dev/guestkit](https://zyvor.dev/guestkit) · [Demo video](https://www.youtube.com/watch?v=ZYCz6HN7bXE)

## Zyvor Platform Stack

| Product | Role |
|---------|------|
| **hypercluster** | Bare-metal Kubernetes bootstrap |
| **machina** | Physical hypervisor OS (libvirt/KVM) |
| **zeus-os** | Cloud / KubeVirt control plane |
| **hermes** | Application layer for Kubernetes |
| **forge** | AI infrastructure on Kubernetes |
| **hypersdk / hyper2kvm** | Multi-cloud VM migration |
| **guestkit** | Offline VM migration assurance |
| **packetwolf** | Kernel-native network intelligence |
| **Aether** | Universal runtime portability |
| **Veyron** | KubeVirt VM command center |
| **IronWolf** | Metal3 bare-metal automation |
| **zyvor-fabric** | systemd-native private cloud |

→ [zyvor.dev](https://zyvor.dev)

---

## Development

See project docs for CI, testing, and contribution guidelines. Historical build summaries in the repo root are snapshots — **`docs/` and this README are authoritative.**

---

## License

See [LICENSE](LICENSE) or project-specific licensing files in `docs/legal/`.
