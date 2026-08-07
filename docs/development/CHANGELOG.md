# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Offline GRUB repair (`fix-grub`)** — `rescue -o fix-grub` bind-mounts
  proc/sys/dev and runs chroot `grub2-mkconfig` / `grub-mkconfig` /
  `update-grub`; `--force` also attempts `grub-install` onto the NBD device;
  if chroot mkconfig fails, stages `guestkit-firstboot-grub.service`.
  `--export-plan` writes the first-boot FileWrite/Symlink ops. `check-grub`
  remains diagnose-only.
- **System Reserved / ESP detection** — offline Windows evidence probes
  non-OS NTFS/FAT volumes for `bootmgr` + BCD (legacy System Reserved) or
  EFI Microsoft Boot (ESP). Surfaces on `windows.system_reserved`, promotes
  `bcd_store_found` / `bootmgr_found`, fixes `esp_present` (no longer aliased
  to bootmgr). Boot check **BOOT-014**, migration **MIG-W-011**, Passport
  flags `system_reserved_layout` + `bcd_store_found`.
- **Windows driver/hotfix migration diagnostics** — offline HotFix registry +
  `$NtUninstall*` / `$hf_mig$` / CBS.log tail; VirtIO `.sys` presence on
  `WindowsDriverEntry.sys_present`; BCD UTF-16 probe for testsigning /
  nointegritychecks. Migration **MIG-W-012** (hotfixes/servicing),
  **MIG-W-013** (VirtIO files); Passport `hotfix_count` /
  `hf_mig_present` / `driver_signature_enforcement`. Hive paths resolve via
  guestfs mount root.
- **Offline activation / ghost-NIC depth** — SOFTWARE ProductId/EditionID/
  DigitalProductId + `oeminfo.ini` → `windows.activation` (OEM/Retail/Volume);
  SYSTEM `Enum\PCI` remnant/problem NICs → `ghost_nics`; Tcpip static
  interfaces → `static_nic_configs`. Enriches **MIG-W-006/007/008**; Passport
  `activation_channel`, `ghost_nic_count`, `static_nic_count`.
- **Offline BitLocker / VSS enrichment** — `BitLockerStatus\BootStatus` (On →
  hard block), FVE/`$BitLocker`/fvevol artifacts (`offline_uncertain` warning),
  VSS+swprv services + System Volume Information inference. Fills
  `windows.bitlocker` / `windows.vss` for **MIG-W-005/009**; Passport
  `bitlocker_uncertain`.
- **Day-0 plan/rescue depth** — `windows-dhcp` / `windows-dns` / `linux-hostname`
  profiles; rescue `enable-rdp` / `enable-winrm` / `set-timezone`; Windows
  `set-hostname` applies registry day-0 plan (was Linux `/etc/hostname`).
- **Cutover Passport signed-enterprise workflows** — `passport keygen` (Ed25519
  seed + pubkey); emit `--issuer` / `--expires-hours`; verify `--trust-keys`
  allowlist + `--max-age-hours` freshness gate (signing/verify need `agent`).
- **Production Helm** — `values-prod.yaml`: PVC-backed Postgres/Redis/MinIO
  (eval still `emptyDir`); Ingress TLS + cert-manager annotations; pinned
  GHCR `v0.3.19` images; nightly image-vault backup CronJob + backup PVC.
- **Guest Control Fabric poll telemetry** — airgap reconciler records per-method
  latency + transport attempts; Redis fleet rollup; `GET .../guest/poll-telemetry`
  (VM + fleet); `guest/status` exposes `lastPoll` / `telemetryMode`.
- **Fleet analyze performance** — parallel `--jobs` / `GUESTKIT_FLEET_JOBS`
  (default min(4, CPUs)); evidence-cache hit skips remount.
- **Cloud disk source depth** — persistent `~/.cache/guestkit/cloud` pulls;
  S3 `GUESTKIT_S3_ENDPOINT`/`AWS_ENDPOINT_URL`; `azure://` URIs; GCS
  `gcloud storage` fallback; CI recipe `scripts/ci-cloud-disk-sources.sh`.
- **Offline heuristic remediations + linux-grub** — `systemctl enable/disable`
  → Symlink/FileDelete; fail2ban/auditd/chrony/apparmor/sshd enable offline;
  ufw default deny FileEdit; day-0 `linux-grub` (`--grub-timeout` /
  `--grub-cmdline`) for `/etc/default/grub`.
- **Offline PackageInstall staging** — when `GUESTKIT_PACKAGE_CACHE` (or
  `host_cache`) holds matching `.rpm`/`.deb`, offline `plan apply` stages
  packages + a first-boot systemd oneshot instead of skipping; live install
  unchanged.
- **Windows offline password set** — `rescue -o reset-password --password`
  blanks SAM then stages HKLM RunOnce `net user` for first boot (SYSKEY AES
  hash write still avoided); omit `--password` to blank only.

## [0.3.19] - 2026-08-06

### Added
- **Cutover Passport** — `guestkit passport emit|verify`: versioned CI-gateable
  assurance artifact (evidence digest, boot/migration scores, FixPlan digest,
  Windows BitLocker hard-block + `windows_offline_ready`, optional live
  attestation via agent-proxy, optional Ed25519 sign). Suite handoff points to
  HyperSDK (export) + hyper2kvm (convert/deploy). Web: `POST /vms/:id/passport`
  + dock download. Worker op `guestkit.passport`.
- **`plan generate -p windows-domain-leave`** — offline domain→workgroup
  markers (`--workgroup`, default `WORKGROUP`).
- **`plan generate -p windows-timezone`** — offline `TimeZoneKeyName`
  (`--timezone`).
- **`plan generate -p windows-static-ip`** — offline static IPv4 on a known
  interface GUID (`--interface-guid --ip --mask [--gateway] [--dns]`).

## [0.3.18] - 2026-08-06

### Added
- **`plan generate --profile windows-hostname`** — offline ComputerName + Tcpip
  Hostname / NV Hostname (`--hostname` required). Apply with `--skip-backup`.
- **`plan generate --profile windows-winrm`** — WinRM Automatic +
  `WINRM-HTTP-In-TCP` firewall rule. Apply with `--skip-backup`.
- **`Symlink` / `FileDelete` plan ops** — offline guestfs `ln_sf` / `rm`
  (used by hardened `linux-ssh`).
- **`plan generate -p linux-ssh --user` + `--key` / `--key-file`** — inject
  `authorized_keys` into the enable plan.
- **Windows `rescue -o reset-password`** — offline SAM blank (chntpw-style)
  via `registry-write` / libhivex; clears password so interactive logon works.
- **`rescue --export-plan PLAN.yaml`** — emit a reviewable FixPlan for
  enable-ssh / inject-ssh-key / set-hostname / reset-password / fix-fstab.
- **Offline `DriverInject`** — apply uses `host_dir` / `GUESTKIT_VIRTIO_WIN`
  + `inject_windows_driver_dir` when built with `registry-write,agent`.
- **`migrate-repair --virtio-win DIR`** — wires VirtIO host tree into
  migration repair `DriverInject` (same as `$GUESTKIT_VIRTIO_WIN`).
- **Heuristic offline remediations** — firewalld enable → `Symlink`, ufw →
  conf edit, more sshd FileEdits; preview tags live-only ops as offline-skip.

### Fixed
- **`linux-ssh` plan fidelity** — wants enable via `Symlink` (not `CommandExec
  ln`), removes `/etc/ssh/sshd_not_to_be_run`, matches rescue enable path.
- **`from_security_profile` naming** — plan profile/tags follow the inspect
  profile name (not hardcoded `"security"`).
- **`rescue check-grub`** — diagnose-only rename (`fix-grub` kept as alias).

## [0.3.17] - 2026-08-06

### Added
- **`plan generate --profile linux-ssh`** — offline Linux SSH enablement
  (systemd `ssh`/`sshd` wants symlink + `/etc/ssh/sshd_config.d/99-guestkit.conf`
  with `PubkeyAuthentication yes`). Apply with `--skip-backup`.
- **`plan` `FileWrite` operation** — create/overwrite a guest file offline.
- **`rescue inject-ssh-key`** — `--user` + `--key` / `--key-file` appends to
  `authorized_keys`.
- **`rescue set-hostname`** — `--hostname` writes `/etc/hostname` and patches
  `/etc/hosts`.

### Fixed
- **`rescue enable-ssh`** — actually creates the systemd wants symlink and
  writes an sshd drop-in (previously only printed a manual `systemctl` note);
  write drop-in before unit enable; prefer real wants dirs / relative
  symlinks / `ln -sfn` when guestfs `ln_sf` rejects unit paths.
- **Windows `guest-fsfreeze-freeze`/`thaw`** — route to VSS marker shadows
  instead of the Linux `fsfreeze` binary so KubeVirt quiesced snapshots work
  on Windows guests.

## [0.3.16] - 2026-08-06

### Added
- **`plan apply --skip-backup`** — skip the mandatory full-image qcow2/raw copy
  before apply. For low-risk registry-only plans (enable RDP, etc.) a 30–40 GiB
  Windows golden backup is slower than the edit; default remains refuse-without-backup.
- **`plan generate --profile windows-rdp`** — offline Windows Remote Desktop
  enablement plan (fDenyTSConnections, NLA, TermService/UmRdpService Automatic,
  port 3389, inbound TCP/UDP firewall Active=TRUE). Apply with `--skip-backup`.

### Fixed
- **`plan apply` dirty NTFS** — run `ntfsfix` on NTFS devices before mount
  (same path as `agent-inject --windows`) so force-off / fast-startup disks
  mount read-write and hive edits are not silently skipped (0 operations).

## [0.3.15] - 2026-07-29

### Added
- **In-guest Windows agent, fully offline install** — `guestkit agent-inject --windows`
  provisions a Windows guest with no boot required: registers the `GuestKitAgent`
  service in the `SYSTEM` hive via hivex, and installs the virtio-serial (`vioser`)
  driver the QGA channel needs (driver files, `DevicePath`, service key, and
  `CriticalDeviceDatabase` entries parsed from the INF, including the KMDF binding).
- **Stock `qemu-guest-agent` takeover** — any `QEMU-GA`/`qemu-ga`/`QEMUGuestAgent`
  service found during Windows injection is disabled (`Start=4`) so GuestKit answers
  the virtio-serial channel uncontended, while remaining QGA-compatible so
  KubeVirt/libvirt see no difference.
- **Converted-image driver fix** — deletes the stale cached
  `SYSTEM\...\Enum\PCI\VEN_1AF4&DEV_1043` device key on converted images (e.g.
  VirtualBox eval → qcow2) so the PCI bus re-detects the virtio-serial device and
  runs a full driver install on next boot instead of staying stuck on "no driver."
- **Generic `guestkit-rpc` QGA passthrough** — every in-guest agent RPC method is
  now reachable through the standard QGA channel, so host automation only needs
  `virsh qemu-agent-command`.
- **Windows agent default channel** — the Windows service now defaults to the
  virtio QGA port, matching the Linux agent's channel selection.
- Fall back to `systemctl restart` when the D-Bus `RestartUnit` call fails.

### Documentation
- Page-by-page customer manual (`docs/customer/`) with per-page PDFs, linked from
  the README.
- `docs/features/guest-agent.md` documents the Windows offline install path
  end-to-end, including the stock-QGA disable step.
- README now surfaces the in-guest agent (previously undocumented at the top
  level) with a dedicated "What's New" section, plus CI/crates.io/PyPI/license
  badges.

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
