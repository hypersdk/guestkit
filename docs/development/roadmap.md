# Roadmap

High-level direction for GuestKit / GuestCtl. Full release notes are in [CHANGELOG.md](CHANGELOG.md).

## Shipped (recent)

- **v0.3.19** — Cutover Passport (`passport emit|verify`, BitLocker hard-block, live attestation, HyperSDK/hyper2kvm suite handoff); Windows day-0 `windows-domain-leave` / `windows-timezone` / `windows-static-ip`
- **v0.3.18** — Day-0 plans (`windows-hostname`, `windows-winrm`, hardened `linux-ssh` + key inject), `Symlink`/`FileDelete` ops, Windows SAM blank rescue, `rescue --export-plan`, offline `DriverInject` via `GUESTKIT_VIRTIO_WIN` / `migrate-repair --virtio-win`, heuristic offline remediations + preview live-only tags
- **Unreleased** — Windows offline password set (SAM blank + RunOnce `net user`); Offline PackageInstall first-boot staging (`GUESTKIT_PACKAGE_CACHE`); Offline heuristic remediations (`systemctl` enable/disable Symlink/FileDelete) + `linux-grub` day-0 defaults; Fleet parallel analyze (`--jobs` / evidence cache); Cloud disk source depth (download cache, S3 endpoint, `azure://`, gcloud fallback, CI recipe); Guest Control Fabric poll telemetry (latency/attempts, fleet rollup, poll-telemetry APIs); Production Helm (`values-prod`: PVCs, TLS/cert-manager, pinned GHCR `v0.3.19`, image-vault backup CronJob); Cutover Passport signed-enterprise (`keygen`, issuer/expiry, trust-keys, max-age); Day-0 `windows-dhcp`/`windows-dns`/`linux-hostname` + rescue `enable-rdp`/`enable-winrm`/`set-timezone` and Windows `set-hostname` fix; Offline BitLocker/VSS enrichment (BootStatus hard-block, FVE artifacts, VSS service inference); Offline Windows activation/OEM + ghost-NIC + static IP evidence (MIG-W-006/007/008); Windows driver/hotfix migration diagnostics (HotFix/CBS/`$hf_mig$`, VirtIO `.sys`, BCD signature probe, MIG-W-012/013); System Reserved / ESP multi-partition Windows detection (BOOT-014, Passport flags); deep offline inspection panels (partitions/UUIDs, kernels, drivers, systemd units, users, network/DNS/gateway, cloud-init, VM tools, firewall, SSH policy, machine-id); premium web-console UX layer (⌘K palette, cinematic scan/verdict, drag-to-analyze, coach-mark tour, verdict share-card, fleet compare); Windows fixes (Linux-check gating + legacy-BIOS BCD detection); security hardening (fail-closed JWT, DB password via Secret, transactional `delete_vm`, namespace-scoped KubeVirt RBAC); OVA/cloud-image ingest + multi-node CephFS vault. Windows analysis verified end-to-end on a real Win10 VMDK (37→97 boot score after fixes)
- **v0.3.12** — Offline Windows registry writes in fix-plan apply (`registry-write` feature, libhivex FFI) — `RegistryEdit` operations now mutate SOFTWARE/SYSTEM/SAM/SECURITY hives with backup instead of being skipped
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
| **Cutover Passport** | Signed enterprise depth shipped Unreleased (`keygen`, issuer/expiry, trust-keys, max-age) |
| **Day-0 plan/rescue** | Expanded Unreleased + `linux-grub` defaults; full grub-install remains parked |
| **Guest Control Fabric depth** | Airgap poll telemetry shipped Unreleased (latency/attempts, fleet rollup, poll-telemetry APIs) |
| **Production Helm** | Shipped Unreleased (`values-prod`: PVCs, TLS/cert-manager, pinned GHCR, image-vault backup) |
| **Windows depth** | System-Reserved/ESP through BitLocker/VSS offline + day-0/rescue depth shipped (Unreleased) |
| **Performance** | Fleet parallel `--jobs` + evidence-cache reuse shipped Unreleased |
| **Cloud** | S3/GCS/Azure cache + endpoints + `azure://` + CI recipe shipped Unreleased |

## Parked (later)

| Item | Notes |
|------|-------|
| Full offline GRUB reinstall | `check-grub` diagnose-only + `linux-grub` defaults offline; real grub-install needs chroot |
| Windows AES SAM hash write | Offline set via SAM blank + RunOnce `net user` shipped Unreleased; direct SYSKEY AES hash injection still later |
| PackageInstall live fetch | Offline staging via `GUESTKIT_PACKAGE_CACHE` shipped Unreleased; network package download still live-only |

## Not planned (open source)

- Hosted control plane (see [zyvor-enterprise.md](../zyvor-enterprise.md))
- Automatic apply without dry-run/backup guardrails

## How to contribute

Pick an item from [GitHub Issues](https://github.com/hypersdk/guestkit/issues) or propose a small PR with tests. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Docs

- User guides: [docs/INDEX.md](../INDEX.md)
- CLI cheat sheet: [quick-reference.md](../user-guides/quick-reference.md)
- TUI: [tui-enhancements.md](../features/tui-enhancements.md)
