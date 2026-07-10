# Roadmap

High-level direction for GuestKit / GuestCtl. Full release notes are in [CHANGELOG.md](CHANGELOG.md).

## Shipped (recent)

- **Unreleased** — Offline Windows registry writes in fix-plan apply (`registry-write` feature, libhivex FFI) — `RegistryEdit` operations now mutate SOFTWARE/SYSTEM/SAM/SECURITY hives with backup instead of being skipped
- **v0.3.11** — Guest Control Fabric: transport-independent guest control with a 7-tier ladder (virtio-serial → QGA exec → QGA builtin → push cache → offline disk), `guest/*` API routes (`status`, `capabilities`, `doctor`, `readiness`, `install-agent`, `repair-plan`, `file/read|write`, `poll-reconcile`), QGA airgap file bootstrap, Agent Doctor (probe tree + 0–100 readiness score), host-mediated polling for `airgap_live` VMs, and `GuestActionPolicy` exec/file allowlists
- **v0.3.7–0.3.10** — `zyvor-guest-agent` crate (Windows/Linux VM Tools daemon), Windows forensic depth (EVTX parsing, persistence run keys, forensic profile merge), KubeVirt QGA transport hardening, web console (Copilot API: briefing/ask/fleet/compare/launch advice), Ubuntu k3s E2E harness
- **v0.3.6** — In-guest agent (`guestkit agent`, `agent-proxy`, `guestkit-agent-protocol`), offline `--inject-agent`, worker jobs (`agent.evidence`, `agent.fix`), TUI LIVE badge + fix-plan preview + Assurance shortcuts
- **v0.3.5** — Migration assurance platform: `EvidenceSnapshot` digital twin, `doctor` (bootability score + `--explain`), `migrate-plan` (hypervisor-aware scoring + `--export` fix plans), `policy check` DSL, `fleet analyze`, `forensic-diff`, `repair --fix boot`, `--profile windows-migration`, OSV CVE lookup, S3/Azure/GCS disk sources
- **v0.3.3–0.3.4** — `guestctl` binary, customer release tarballs (gnu + musl), TUI theming and two-tier navigation, shared widgets
- **v0.3.1** — VM migration (fstab/crypttab rewriter), Windows registry-based detection, LVM cleanup, loop-device paths

### AI Guest Agent (all phases shipped)

Phases 0–4 of the optional AI layer are complete — richer systemd/Windows evidence, semantic analysis, the agentic loop, local Ollama + what-if simulation, and platform integration (CIS-lite profiles, Machina export, full `.evtx` forensics). See [ai-guest-agent-roadmap.md](ai-guest-agent-roadmap.md).

## In progress / next

| Area | Goal |
|------|------|
| **Guest Control Fabric depth** | Broaden transport ladder coverage and reconciler telemetry for `airgap_live` fleets |
| **Windows boot** | Deeper EFI/BCD diagnostics for migration |
| **Performance** | Faster fleet scans, cache improvements on parallel inspect |
| **Cloud** | Wider S3/Azure/GCS pull paths and CI recipes |

## Not planned (open source)

- Hosted control plane (see [zyvor-enterprise.md](../zyvor-enterprise.md))
- Automatic apply without dry-run/backup guardrails

## How to contribute

Pick an item from [GitHub Issues](https://github.com/hypersdk/guestkit/issues) or propose a small PR with tests. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Docs

- User guides: [docs/INDEX.md](../INDEX.md)
- CLI cheat sheet: [quick-reference.md](../user-guides/quick-reference.md)
- TUI: [tui-enhancements.md](../features/tui-enhancements.md)
