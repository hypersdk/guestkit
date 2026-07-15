# GuestKit

**Offline VM intelligence and migration assurance.**

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

## Platform at a Glance

| Layer | What's in the repo |
|-------|-------------------|
| **Core** | Rust disk engine + assurance APIs — `crates/`, `src/` |
| **CLI** | `guestkit` + `guestctl` — doctor, migrate-plan, fleet |
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
| Architecture | [docs/architecture/overview.md](docs/architecture/overview.md) |
| Full index | [docs/INDEX.md](docs/INDEX.md) |

→ [zyvor.dev/guestkit](https://zyvor.dev/guestkit) · [Demo video](https://www.youtube.com/watch?v=ZYCz6HN7bXE) · [Full Zyvor platform](https://zyvor.dev)

---

## Development

See project docs for CI, testing, and contribution guidelines. Historical build summaries in the repo root are snapshots — **`docs/` and this README are authoritative.**

---

## License

See [LICENSE](LICENSE) or project-specific licensing files in `docs/legal/`.
