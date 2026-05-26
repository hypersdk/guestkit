# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.6] - 2026-05-27

### Added
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
