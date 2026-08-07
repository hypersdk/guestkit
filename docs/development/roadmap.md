# Roadmap

High-level direction for GuestKit / GuestCtl. Full release notes are in [CHANGELOG.md](CHANGELOG.md).

## Shipped (recent)

- **v0.3.19** — Cutover Passport (`passport emit|verify`, BitLocker hard-block, live attestation, HyperSDK/hyper2kvm suite handoff); Windows day-0 `windows-domain-leave` / `windows-timezone` / `windows-static-ip`
- **v0.3.18** — Day-0 plans (`windows-hostname`, `windows-winrm`, hardened `linux-ssh` + key inject), `Symlink`/`FileDelete` ops, Windows SAM blank rescue, `rescue --export-plan`, offline `DriverInject` via `GUESTKIT_VIRTIO_WIN` / `migrate-repair --virtio-win`, heuristic offline remediations + preview live-only tags
- **Unreleased** — Offline remediations closed out:
  - **Package mirror fetch** (`GUESTKIT_PACKAGE_MIRROR` curl/wget fallback)
  - **Domain-leave RunOnce** + worker performance/migration profiles
  - **Offline ServiceOperation / CommandExec** (wants Symlink + first-boot live oneshot)
  - **UEFI-aware fix-grub** (`--force` → EFI `--no-nvram --removable` when ESP present)
  - **Windows AES/RC4 SAM NT-hash write** (SYSKEY bootkey + hashed bootkey; RunOnce fallback)
  - **PackageInstall host fetch** (`GUESTKIT_PACKAGE_FETCH`) + first-boot staging (`GUESTKIT_PACKAGE_CACHE`)
  - **Offline GRUB repair** (`rescue -o fix-grub`: chroot mkconfig, optional `--force` grub-install, first-boot oneshot)
  - **Day-0 / rescue depth** — `linux-grub`, `linux-hostname`, `windows-dhcp`/`windows-dns`, rescue `enable-rdp`/`enable-winrm`/`set-timezone`
  - **Offline heuristic remediations** — `systemctl` enable/disable → Symlink/FileDelete
  - **Fleet** parallel `--jobs` + evidence-cache reuse
  - **Cloud** disk cache, S3 endpoint, `azure://`, gcloud fallback, CI recipe
  - **Guest Control Fabric** poll telemetry (latency/attempts, fleet rollup, APIs)
  - **Production Helm** (`values-prod`: PVCs, TLS/cert-manager, pinned GHCR, image-vault backup)
  - **Cutover Passport** signed-enterprise (`keygen`, issuer/expiry, trust-keys, max-age)
  - **Windows offline depth** — BitLocker/VSS, activation/OEM, ghost-NIC, driver/hotfix, System Reserved/ESP
  - Web-console UX, security hardening, OVA/cloud ingest, CephFS vault
- **v0.3.12** — Offline Windows registry writes in fix-plan apply (`registry-write` feature, libhivex FFI)
- **v0.3.11** — Guest Control Fabric (7-tier ladder, `guest/*` APIs, Agent Doctor, airgap poll)
- **v0.3.7–0.3.10** — `zyvor-guest-agent`, Windows forensics, KubeVirt QGA, web Copilot API, k3s E2E
- **v0.3.6** — In-guest agent, offline `--inject-agent`, worker jobs, TUI LIVE / Assurance
- **v0.3.5** — Migration assurance platform (`doctor`, `migrate-plan`, `policy`, `fleet`, `forensic-diff`)
- **v0.3.3–0.3.4** — `guestctl` binary, customer release tarballs, TUI theming
- **v0.3.1** — VM migration (fstab/crypttab rewriter), Windows registry detection, LVM cleanup

### AI Guest Agent (all phases shipped)

Phases 0–4 of the optional AI layer are complete — richer systemd/Windows evidence, semantic analysis, the agentic loop, local Ollama + what-if simulation, and platform integration (CIS-lite profiles, Machina export, full `.evtx` forensics). See [ai-guest-agent-roadmap.md](ai-guest-agent-roadmap.md).

## In progress / next

The previously tracked Unreleased slices (Passport signing, day-0/rescue, GCF telemetry, Helm, Windows depth, fleet parallel, cloud pulls, AES SAM, PackageInstall fetch, GRUB repair) are **shipped in Unreleased**. Next work is issue-driven — pick from [GitHub Issues](https://github.com/hypersdk/guestkit/issues) or propose a small PR.

## Parked (later)

None currently.

## Not planned (open source)

- Hosted control plane (see [zyvor-enterprise.md](../zyvor-enterprise.md))
- Automatic apply without dry-run/backup guardrails

## How to contribute

Pick an item from [GitHub Issues](https://github.com/hypersdk/guestkit/issues) or propose a small PR with tests. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Docs

- User guides: [docs/INDEX.md](../INDEX.md)
- Fix plans / rescue: [fix-plans.md](../features/fix-plans.md)
- CLI cheat sheet: [quick-reference.md](../user-guides/quick-reference.md)
- TUI: [tui-enhancements.md](../features/tui-enhancements.md)
