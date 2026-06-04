# Roadmap

High-level direction for GuestKit / GuestCtl. Shipped work is tracked in [CHANGELOG.md](CHANGELOG.md).

## Shipped (recent)

- **v0.3.7 (unreleased)** — In-guest agent (`guestkit agent`, agent-proxy), live evidence, offline `--inject-agent`
- **v0.3.5–0.3.6** — Migration assurance (`doctor`, `migrate-plan`, fix plans), TUI Assurance view, fix-plan preview, fleet/policy/forensic-diff
- **v0.3.3–0.3.4** — Carbon TUI theme, `guestctl` binary, release tarballs, two-tier navigation
- **v0.3.1+** — VM migration (fstab/crypttab), Windows registry inspect, interactive shell

## In progress / next

| Area | Goal |
|------|------|
| **AI Guest Agent Phase 0** | Richer systemd + Windows evidence in schema v2 — see [ai-guest-agent-roadmap.md](ai-guest-agent-roadmap.md) |
| **TUI plan apply** | Apply fix plans from TUI with write mount, backups, progress (preview-only today) |
| **Performance** | Parallel inspect, faster fleet scans, cache improvements |
| **Windows boot** | Deeper EFI/BCD diagnostics for migration |
| **Cloud** | Broader S3/Azure/GCS pull paths and CI recipes |

## Not planned (open source)

- Hosted control plane (see [zyvor-enterprise.md](../zyvor-enterprise.md))
- Automatic apply without dry-run/backup guardrails

## How to contribute

Pick an item from [GitHub Issues](https://github.com/hypersdk/guestkit/issues) or propose a small PR with tests. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Docs

- User guides: [docs/INDEX.md](../INDEX.md)
- CLI cheat sheet: [quick-reference.md](../user-guides/quick-reference.md)
- TUI: [tui-enhancements.md](../features/tui-enhancements.md)
