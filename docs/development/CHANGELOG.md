# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.14] - 2026-07-11

### Added
- **Boot-score trend** (`guestkit-ux.js`) — every boot score is recorded per disk
  in localStorage; a re-scan after a repair toasts the delta (`▲ +N` / `▼ −N`),
  and a new **📈 Boot-score trend** command renders the history as an inline SVG
  sparkline (CSP-safe, no external assets, reduced-motion aware).
- **Zyvor brand footer + logo** — the web console and login page now carry the
  `zyvor.dev` logo (linked) and a `zyvor.dev · HyperSDK · © 2026` credit line,
  matching the PacketWolf branding treatment.

### Documentation
- **Default web console login documented** — the seeded `admin` / `Admin@321`
  (previously only printed at install time by `package-auth-bootstrap.sh`) is now
  in the remote-deploy guide, getting-started, and README, each with a
  change-on-first-login warning. Also surfaced as a first-run hint on the login
  page, shown only when local login/bypass is available.
- **Run the web stack from GHCR** — new `deploy/docker-compose.ghcr.yml` (pulls
  only the public `ghcr.io/hypersdk/{zyvor-ui,zyvor-api,guestkit-worker}` images)
  plus a "Published images (GHCR)" guide covering pull, Compose (eval), and Helm
  (prod), cross-linked from the README and deployment docs.

## [0.3.13] - 2026-07-11

### Added
- **Deep offline inspection panels** — the `guestkit.inspect` worker handler now
  collects and surfaces, from a mounted disk: partitions (device/fstype/UUID) +
  fstab, installed kernels + default, boot-load kernel modules, systemd unit
  inventory, and user accounts (`/etc/passwd` → name/uid/home/shell/login). A
  second wave adds network detail (DNS servers, default gateway), machine-id,
  cloud-init presence, VM guest tools (open-vm-tools/vmware/vbox/hyperv/qemu-ga),
  firewall (ufw/firewalld/iptables) and SSH policy (root-login/password-auth).
  The web report renders each as a token-driven panel (Storage table, Kernels,
  Drivers, Systemd units, Users table, Guest platform), all reduced-motion aware.
- **Premium web-console UX layer** (`guestkit-ux.js`) — ⌘K fuzzy command palette
  (with `>` Ask-Zeus mode), dock cursor-magnify, event-bus rich toasts, activity
  log, cinematic Zeus scan overlay + verdict burst, ambient aurora, theme wipe,
  keyboard-driven fleet nav, click-to-copy, shortcut cheat sheet, skeleton
  loaders, Ask-Zeus starter chips, global drag-to-analyze overlay, first-run
  coach-mark tour, a canvas verdict share-card (PNG export), synthesized Web
  Audio cues, a Konami "storm mode", and a client-side fleet compare view.
- **OVA + cloud-image ingest** and **multi-node CephFS RWX vault** for shared
  image storage across cluster nodes.

### Fixed
- **Windows boot doctor** — Linux-only checks (BOOT-003 Initramfs, BOOT-004 GRUB)
  are now gated as N/A on Windows guests instead of failing as false blockers.
- **Legacy-BIOS Windows BCD/bootmgr detection** — the evidence builder checked
  only EFI paths, falsely flagging `BCD store not found` on legacy installs;
  now also detects `/Boot/BCD` and `/bootmgr` at the boot-volume root.
- **Security: JWT signing key fails closed** — with `AUTH_ENABLED=true`, the API
  refuses to start unless `JWT_SECRET` is a real value (previously fell back to a
  hardcoded, globally-known `change-me-in-production` key → forgeable tokens).
- **Security: DB password out of plaintext env** — `DATABASE_URL` now comes from
  the `zyvor-secrets` Secret via `secretKeyRef` instead of being interpolated
  into the API Deployment env (visible in `kubectl describe`).
- **`delete_vm` no longer 500s on analyzed disks** — the handler tears down
  `job_results → jobs → vm_images` transactionally instead of hitting the
  `jobs_vm_id_fkey` foreign-key constraint.
- **Helm multi-tenant collision** — the KubeVirt ClusterRole/Binding names are
  namespace-scoped (`zyvor-api-kubevirt-<ns>`) so a second install doesn't clash.

## [0.3.12] - 2026-07-10

### Added
- **Offline Windows registry writes** — `registry-write` feature links libhivex (hand-rolled FFI, LGPL-2.1 dynamic link — no copyleft crate) so fix-plan `RegistryEdit` operations mutate offline SOFTWARE/SYSTEM/SAM/SECURITY hives (`HKLM`) instead of being skipped; supports REG_SZ/EXPAND_SZ/DWORD/QWORD/MULTI_SZ/BINARY with whole-disk backup. Build with `--features registry-write` (needs `libhivex-dev`/`hivex-devel`)

## [0.3.11] - 2026-06-15

### Added
- **Guest Control Fabric** — transport-independent guest control with 7-tier ladder (virtio-serial → QGA exec → QGA builtin → push cache → offline disk)
- **New API routes** — `guest/status`, `guest/capabilities`, `guest/doctor`, `guest/readiness`, `guest/install-agent`, `guest/repair-plan`, `guest/file/read|write`, `guest/poll-reconcile`
- **QGA file bootstrap** — airgap agent install via `guest-file-write` + `guest-exec` (no guest network)
- **Agent Doctor** — probe tree, readiness score (0–100), live `guestkit.doctor` via transport ladder
- **Host-mediated polling** — background reconciler for `airgap_live` VMs without push telemetry
- **GuestActionPolicy extensions** — `execAllowlist`, `fileReadAllowlist`, `fileWriteAllowlist`, `freezeAllowed`, `maxExecOutputBytes`
- **UI** — Guest Control panel, Agent Doctor tree, control-state chips, host-mediated exec warning banner
- **Docs** — [guest-control-fabric.md](../features/guest-control-fabric.md)

### Changed
- **Guest intel routes** — `/guest/*` intel endpoints return `GuestControlEnvelope` with legacy fields in `data`
- **Exec policy** — when `GuestActionPolicy` exists, `execAllowlist` is required (no raw shell by default)
- **Repair worker** — honors `inject_qga`, `fix_cloud_init_network`, `validate_fstab`, `enable_systemd` job payload fields
- **Transport ladder** — attempts VirtioSerial (daemon + socket) and InGuestSocket before QGA exec RPC
- **Offline inject** — agent binary path aligned to `/usr/local/bin/zyvor-guest-agent` and `zyvor-guest-agent` systemd unit
- **Worker repair** — honors `inject_zyvor_agent` from job payload

## [0.3.10] - 2026-06-14

### Added
- **`deploy/scripts/e2e-ubuntu-k3s.sh`** — Ubuntu 22.04 k3s E2E: offline inspect/doctor, CDI VM, live guest intel, cluster offline inspect

### Fixed
- **Release CI** — optional `journal-native` feature for musl/static builds; install `libsystemd-dev` for gnu tarballs; fix `ApiError` mapping in guest pull; align VM tools `.sha256` artifact name
- **KubeVirt QGA transport** — virt-launcher pod lookup uses `kubevirt.io/vm` (KubeVirt 1.8 labels) with fallbacks
- **Guest agent install** — KubeVirt 1.8+ virtio guestagent disk (`serial: org.qemu.guest_agent.0`) instead of rejected `devices.channels`
- **Per-VM guest pull** — install/RPC paths use `/usr/local/bin/zyvor-guest-agent`; QGA failures no longer silently fall back to in-cluster `AGENT_PROXY_URL`

## [0.3.9] - 2026-06-13

### Fixed
- `cargo fmt` formatting in security profile score calculation
- Integration tests treat `unknown` OS distro as undetected and fall back to `/etc/debian_version`
- RPM workflow installs binary package when optional Python RPM is absent; verify step uses `command -v`
- Python wheel CI installs locally built artifacts instead of stale PyPI releases
- PyPI publish uses wheel-only upload with version synced from `Cargo.toml`

## [0.3.8] - 2026-06-12

### Fixed
- CI clippy warnings across agent, AI, guestfs, and assurance modules
- Integration test uses disk-to-disk `guestkit copy` with four arguments
- `Cargo.lock` synced for RPM `--locked` builds

## [0.3.7] - 2026-06-12

### Added
- **Abyss web console** — GuestKit deploy UI with deep-navy design system (aurora background, indigo/violet accents, frosted-glass cards), local Inter fonts, and GuestKit-branded brain/dock/mission rail
- **Console Copilot API** — briefing, ask, fleet overview, compare narrative, launch advice, and system status endpoints in `zyvor-api`
- **`zyvor-guest-agent` crate** — in-guest agent daemon for Windows and Linux VM Tools
- **Windows forensic depth** — EVTX parsing, persistence run keys, forensic profile merge in evidence collectors
- **QGA helpers** — KubeVirt guest-agent transport improvements for live inspection

### Changed
- Renamed Abyss UI modules to `guestkit-console.js`, `guestkit-ai.js`, `guestkit-features.js`
- Integration tests use `guestkit copy` (replacing removed `cp` alias)
- TUI view registry includes Assurance, SystemdDeep, and AiInsights (21 views)

### Fixed
- AI agent tool-call parser accepts JSON embedded in prose lines
- Failed-disk UX in web console (deduped job tracker, disk switch guidance)
- RPM spec `%changelog` weekday dates for Fedora builds

## [0.3.6] - 2026-05-27

### Added
- **In-guest agent** — optional `agent` feature: `guestkit agent` (virtio-serial JSON-RPC daemon), `guestkit agent-proxy` (host HTTP bridge), live evidence + fix-plan execution inside running VMs
- **`guestkit-agent-protocol`** — shared length-prefixed JSON-RPC types for agent and proxy
- **`repair --inject-agent` / `migrate-plan --export --inject-agent`** — offline guestfs injection of agent binary + systemd unit
- **Worker jobs** — `guestkit.agent.evidence` and `guestkit.agent.fix` via agent-proxy HTTP
- **TUI LIVE badge** — assurance view when `GUESTKIT_AGENT_SOCKET` responds to ping
- **CI** — `agent-release.yml` musl artifact workflow; integration tests behind `--features agent`
- **TUI fix-plan preview** — read-only modal of migration plan operations (`p` in Assurance, `: plan preview`)
- **TUI Assurance shortcuts** — dashboard `a` opens Assurance; global search indexes boot blockers and migration items
- **TUI Assurance view** — Security-group panel for `doctor` boot gate and `migrate-plan` scoring; `d`/`t`/`e` keys; palette commands `doctor`, `migrate-plan`, `export plan`
- **TUI config** — `[behavior]` `default_migration_target`, `assurance_on_startup`, `show_assurance_hint`
- **TUI UX** — scrollable view tab row (`,` / `.`); compact density on Issues list rows; palette `goto` aliases for all views

### Changed
- **Dashboard** — boot score line when assurance data is loaded
- **Documentation** — pruned CLI guide and CHANGELOG; TUI assurance docs updated

## [0.3.5] - 2026-05-26

### Added
- **Migration assurance platform** — evidence snapshot model (`EvidenceSnapshot`) as the digital twin primitive for scoring engines
- **`guestkit doctor`** — bootability prediction with weighted score, blockers, and `--explain` root-cause analysis
- **`guestkit migrate-plan`** — hypervisor-aware migration scoring (KVM/Proxmox/cloud) with driver injections and downtime estimate
- **`guestkit migrate-plan --export`** — write migration guidance as an executable fix plan (YAML/JSON)
- **`guestkit policy check`** — policy-as-code alias with expression DSL over evidence fields (`bootability.score >= 80`)
- **`guestkit fleet analyze`** — cluster identical VMs, detect snowflakes, flag migration blockers
- **`guestkit forensic-diff`** — security drift scoring between two disk snapshots
- **`guestkit repair --fix boot`** — transactional boot repair via fix plans with post-apply doctor validation
- **`--profile windows-migration`** — BitLocker, domain join, RDP, hypervisor remnants, driver gaps
- **Windows registry depth** — SAM/SECURITY hive parsing, BitLocker detection, pending reboot, domain/RDP audit
- **OSV CVE lookup** — offline cache at `~/.cache/guestkit/cve/` with static fallback database
- **Cloud disk sources** — optional S3/Azure/GCS backends (`--features cloud-s3`, etc.)
- **AI evidence tools** — deterministic bootability and evidence snapshot for the optional AI assistant
- **Assurance integration tests** — CLI and plan-generation coverage for doctor/migrate-plan/repair workflow
- **Documentation** — [migration-assurance.md](../features/migration-assurance.md); VM migration and fix-plans guides updated

### Changed
- **TUI navigation** — two-tier tabs (group + view rows); scrollable jump menu and help; `{`/`}` switch groups ([tui-enhancements.md](../features/tui-enhancements.md))

## [0.3.4] - 2026-05-25

### Added
- **`guestctl` binary** — separate crate binary (alias entry point); install via `cargo install guestkit` or client tarball symlink
- **GitHub Release customer bundles** — full install tarball (`guestkit-<version>-linux-amd64.tar.gz`) matching remote deploy packaging
- **`scripts/package-binary-release.sh`** — local/CI packaging shared with GitHub Actions
- **TUI visual polish** — shared `widgets.rs` (stat chips, severity rail, progress bar, risk donut)
- **Theme variants** — `high-contrast` and `minimal` via `[ui] theme` in `tui.toml`
- **Config** — `show_emoji`, `density` under `[ui]`

### Changed
- CLI entry split into `guestkit::cli` module tree (`entry`, `invocation`, `commands_list`, `welcome`)
- TUI header, stats bar, tabs, footer, loading bar, fleet sidebar, and modal dim layer
- Dashboard and Issues views use carbon gauges, sparklines, and risk summary donut
- GitHub release workflow uploads customer bundles (gnu + musl) instead of bare binaries
- Documentation: [tui-enhancements.md](../features/tui-enhancements.md) updated for carbon theme and visual polish

## [0.3.3] - 2026-05-22

### Added
- **Carbon control-plane TUI theme** — graphite surfaces (`#0B0E12`), orange accent (`#FF7A00`) on focus and risk states only
- **Zyvor branding** on TUI splash (`zyvor.dev` wordmark)
- **Risk-aware header border** — subtle red/amber glow from security issue counts
- **Documentation hub** (`docs/INDEX.md`) with pruned user-facing docs

### Changed
- TUI footer uses muted key hints; orange reserved for primary actions
- Default TUI theme config: `carbon`
- README: open-source branding (removed Community Edition wording)

### Fixed
- TUI dashboard and issues views use consistent `content_block` pane styling

## [0.3.1] - 2026-01-26

### Added
- Killer summary view on inspect; Windows registry-based version detection
- Universal fstab/crypttab rewriter for VM migration; loop-device primary path for RAW/IMG/ISO
- LVM volume group cleanup on shutdown

## [0.3.0] and earlier

Release notes for v0.3.0 (interactive REPL, expanded inspect), v0.2.0 (extended guestfs API coverage), and v0.1.0 (initial toolkit) are in [GitHub Releases](https://github.com/hypersdk/guestkit/releases) and git history.
