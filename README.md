# GuestKit

### Offline VM intelligence. Migration assurance you can prove.

Score boot readiness **before** power-on. Generate reviewable fix plans. Repair disks offline. Certify cutover with a signed Passport — then hand off to [hyper2kvm](https://github.com/hypersdk/hyper2kvm) / HyperSDK.

[![CI](https://github.com/ssahani/guestkit/actions/workflows/ci.yml/badge.svg)](https://github.com/ssahani/guestkit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/guestkit.svg)](https://crates.io/crates/guestkit)
[![PyPI](https://img.shields.io/pypi/v/hypersdk-guestkit.svg)](https://pypi.org/project/hypersdk-guestkit/)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![GHCR](https://img.shields.io/badge/GHCR-hypersdk-black?logo=github)](https://github.com/orgs/hypersdk/packages)

**[Demos](#see-it-in-action)** · **[Wiki](https://github.com/hypersdk/guestkit/wiki)** · **[Feature guide](docs/guestkit-customer-feature-guide.md)** · **[Quick start](#quick-start)** · **[GHCR](#run-from-ghcr)** · **[Docs](#documentation)** · **[zyvor.dev](https://zyvor.dev/guestkit)**

```text
  disk.qcow2 / .vmdk / .vhdx
           │
           ▼
   ┌───────────────────┐     doctor 0–100     migrate-plan YAML
   │  Pure-Rust engine │ ──────────────────►  passport emit
   │  NBD/loop mount   │     rescue / plan apply (offline)
   └───────────────────┘
           │
     CLI · TUI · Python · Web · Agent
```

**70+** commands · **6** disk formats · **0** libguestfs appliances · **8** migration targets · Apache-2.0

---

## See it in action

<table>
<tr>
<td width="50%" align="center">
<a href="https://www.youtube.com/watch?v=LYoqOye3P3I">
<img src="docs/img/machina-guestkit-demo-thumb.jpg" alt="Machina × GuestKit live guest-agent UX" width="100%">
</a>
<br><b>Machina × GuestKit</b><br>
<sub>Live Linux guest agent — health, TRIM, netplan, services</sub>
</td>
<td width="50%" align="center">
<a href="https://www.youtube.com/watch?v=lLEBQoFceIs">
<img src="https://i.ytimg.com/vi/lLEBQoFceIs/maxresdefault.jpg" alt="GuestKit CLI and TUI demo" width="100%">
</a>
<br><b>CLI &amp; TUI</b><br>
<sub>Offline VM intelligence, explained</sub>
</td>
</tr>
<tr>
<td width="50%" align="center">
<a href="https://www.youtube.com/watch?v=usQX2rQIFM8">
<img src="https://i.ytimg.com/vi/usQX2rQIFM8/maxresdefault.jpg" alt="GuestKit web dashboard tour" width="100%">
</a>
<br><b>Web Dashboard Tour</b><br>
<sub>Server Image Vault, live KubeVirt cluster</sub>
</td>
<td width="50%" align="center">
<a href="https://www.youtube.com/watch?v=icTLVko588A">
<img src="https://i.ytimg.com/vi/icTLVko588A/maxresdefault.jpg" alt="GuestKit web dashboard tutorial" width="100%">
</a>
<br><b>Web Dashboard Tutorial</b><br>
<sub>Sources, live cluster, one-click intelligence</sub>
</td>
</tr>
</table>

Recorded live against real deployments — no staged screenshots.

---

## Why teams use it

| Before GuestKit | With GuestKit |
|-----------------|---------------|
| “Will it boot?” answered at power-on | Offline **doctor** score + root-cause chain |
| guestfish scripts and tribal knowledge | Structured plans, JSON/YAML, CI gates |
| Migration surprises on cutover weekend | Hypervisor-aware **migrate-plan** + day-0 packs |
| No audit trail MTV / virt-v2v can skip | Signed **Cutover Passport** |
| Fleet drift invisible until outage | `fleet analyze`/`watch`, forensic diff, policy-as-code |
| Migration order guessed by hand | `fleet wave-plan` — dependency-aware migration waves |
| Deep inspect needs a running guest | Carbon **TUI** + in-guest agent over QGA |

**Pairs with:** [hyper2kvm](https://github.com/hypersdk/hyper2kvm) for VMware → KVM. GuestKit certifies; hyper2kvm converts.

---

## What you can do

### Assurance before cutover

```bash
guestkit doctor vm.qcow2 --target proxmox --explain
guestkit migrate-plan vm.vmdk --target proxmox --export plan.yaml
guestkit passport emit vm.qcow2 --target kvm -o passport.json
guestkit passport verify passport.json --fail-below 80
```

Targets: `kvm` · `proxmox` · `qemu` · `aws` · `azure` · `gcp` · `cloud` · `hyperv`

**GitHub Actions:** gate conversion on the same score, no CLI install step —
see [`action.yml`](action.yml) and the runbook at
[docs/devops/01-passport-ci-gate.md](docs/devops/01-passport-ci-gate.md).

```yaml
- uses: hypersdk/guestkit@v1
  with:
    disk: vm.qcow2
    target: kvm
    fail-below: '80'
```

### Offline repair (no boot required)

```bash
# Day-0 plans
guestkit plan generate win.qcow2 -p windows-rdp -o rdp.yaml
guestkit plan generate win.qcow2 -p windows-domain-leave --workgroup WORKGROUP
guestkit plan generate disk.qcow2 -p linux-ssh --user ubuntu --key-file ~/.ssh/id_ed25519.pub

# Rescue shortcuts
guestkit rescue disk.qcow2 -o enable-ssh
guestkit rescue disk.qcow2 -o fix-grub --force          # BIOS or UEFI ESP
guestkit rescue win.qcow2 -o reset-password --user Administrator --password '…'
# → AES/RC4 SAM NT-hash when registry-write is built; RunOnce fallback

guestkit plan apply plan.yaml --vm disk.qcow2 --yes     # backups + rollback
```

PackageInstall can stage from cache, host-fetch (`GUESTKIT_PACKAGE_FETCH=1`), or HTTP mirror (`GUESTKIT_PACKAGE_MIRROR`). Service enable/disable and CommandExec stage first-boot oneshots when chroot can’t run them live.

### Live control + platform

- **In-guest agent** (Linux + Windows) over virtio-serial / QGA — inject offline, then `agent-proxy` / `agent-call`
- **AI copilot** (optional, `--features ai`) — read-only tool-calling loop over the offline evidence snapshot: `doctor --explain --ai`, `migrate-plan --ai`. Native tool-calling for OpenAI (rig-core), cross-run memory across repeated runs on the same VM, and an MCP server (`guestkit mcp-serve`, `--features mcp`) for external hosts like Claude Desktop. OpenAI/xAI/Anthropic/Ollama. Not the in-guest agent above — see [AI Guest Agent roadmap](docs/development/ai-guest-agent-roadmap.md)
- **KubeVirt** boot-inspect hooks and Guest Control Fabric
- **Web console** + worker on GHCR; Helm chart under `deploy/helm/zyvor`
- **Python:** `pip install hypersdk-guestkit` → `from guestkit import Guestfs`

Full map: **[Customer Feature Guide](docs/guestkit-customer-feature-guide.md)** ([PDF](docs/guestkit-customer-feature-guide.pdf)) · **[Customer manuals](docs/customer/README.md)**

---

## Quick start

```bash
cargo install guestkit          # installs guestkit + guestctl

guestkit doctor vm.qcow2 --target proxmox --explain
guestkit migrate-plan vm.vmdk --target proxmox --export plan.yaml
guestctl tui vm.qcow2           # Assurance · preview · export
```

| You want… | Go here |
|-----------|---------|
| First hour | [Getting started](docs/user-guides/getting-started.md) |
| Command cheat sheet | [Quick reference](docs/user-guides/quick-reference.md) |
| CLI depth | [CLI guide](docs/user-guides/cli-guide.md) |
| Migration story | [Migration assurance](docs/features/migration-assurance.md) |
| Fix plans / rescue | [Fix plans](docs/features/fix-plans.md) |
| Guest agent | [Guest agent](docs/features/guest-agent.md) |
| CE vs Enterprise | [ce-vs-enterprise](docs/ce-vs-enterprise.md) |

Host needs: Linux with `qemu-img`, `losetup`, and `qemu-nbd` (mount/repair may need root).

---

## Run from GHCR

Public images under **`ghcr.io/hypersdk`** — no `docker login` required.

| Image | Role |
|-------|------|
| `ghcr.io/hypersdk/zyvor-ui` | Web console |
| `ghcr.io/hypersdk/zyvor-api` | API |
| `ghcr.io/hypersdk/guestkit-worker` | Disk-inspection worker |

```bash
docker compose -f deploy/docker-compose.ghcr.yml pull
docker compose -f deploy/docker-compose.ghcr.yml up -d
open http://localhost:8088
```

> **Eval only** — unauthenticated stack. Do not expose beyond localhost.  
> Production: `deploy/docker-compose.prod.example.yml` · [Docker guide](docs/guides/DOCKER.md) · [Helm](deploy/helm/zyvor)

Default console login for packaged installs is documented in [remote deploy](docs/guides/DEPLOY-REMOTE.md#web-console-access) — change it before any network exposure.

---

## Platform layout

| Layer | In this repo |
|-------|----------------|
| **Engine** | Pure-Rust parsers + evidence schema · NBD/loop mount (`src/`, `crates/`) |
| **CLI / TUI** | `guestkit` · `guestctl` — doctor, passport, fleet, rescue, carbon TUI |
| **Agent** | Linux + Windows · protocol 1.3 · `agent-inject` / `agent-proxy` |
| **Python** | [hypersdk-guestkit](https://pypi.org/project/hypersdk-guestkit/) |
| **K8s** | KubeVirt hooks · `k8s/` |
| **Web / worker** | GHCR images · `deploy/` |

```text
┌────────────────────────────────────────────────────────────┐
│  guestkit CLI · guestctl TUI · Python · Web · Agent        │
├────────────────────────────────────────────────────────────┤
│  Rust evidence engine · boot scoring · fix-plan apply      │
├────────────────────────────────────────────────────────────┤
│  JSON · YAML · HTML · PDF · Passport · CI exit codes       │
└────────────────────────────────────────────────────────────┘
```

---

## Documentation

| Goal | Document |
|------|----------|
| Operator wiki | [hypersdk/guestkit/wiki](https://github.com/hypersdk/guestkit/wiki) |
| Docs home | [docs/README.md](docs/README.md) · [INDEX](docs/INDEX.md) |
| **DevOps runbooks** | [docs/devops](docs/devops/README.md) — Passport CI, workers, air-gap, fleet, cutover, triage |
| Feature guide (all areas) | [guestkit-customer-feature-guide.md](docs/guestkit-customer-feature-guide.md) |
| Docker / GHCR | [DOCKER.md](docs/guides/DOCKER.md#published-images-ghcr) |
| Remote deploy | [DEPLOY-REMOTE.md](docs/guides/DEPLOY-REMOTE.md) |
| Architecture | [overview](docs/architecture/overview.md) |
| User stories / industry | [USER_STORIES](docs/USER_STORIES.md) · [INDUSTRY_USE_CASES](docs/INDUSTRY_USE_CASES.md) |
| Changelog / roadmap | [CHANGELOG](docs/development/CHANGELOG.md) · [roadmap](docs/development/roadmap.md) |

→ [zyvor.dev/guestkit](https://zyvor.dev/guestkit) · [Full Zyvor platform](https://zyvor.dev)

---

## Development

```bash
cargo build --release
cargo test
```

See [CONTRIBUTING](docs/development/CONTRIBUTING.md) and CI under `.github/workflows/`. **`docs/` and this README are authoritative** over any historical build notes in the tree.

---

## License

[Apache-2.0](LICENSE) · additional notes in `docs/legal/` where applicable.
