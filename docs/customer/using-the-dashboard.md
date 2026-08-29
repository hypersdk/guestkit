# Using the Dashboard

GuestKit is primarily **CLI + TUI**, with an optional web Image Vault.

## Surfaces

| Surface | How to open |
|---------|-------------|
| **CLI** | `guestkit <cmd>` / `guestctl <cmd>` |
| **TUI** | `guestctl tui IMAGE` (aliases: `guestkit tui`, `guestkit ui`) |
| **Interactive REPL** | `guestkit interactive IMAGE` |
| **File explorer** | `guestkit explore IMAGE [/path]` |
| **Optional web** | GHCR compose → `http://<host>:8088` |

## TUI keys (Assurance)

| Key | Action |
|-----|--------|
| `d` | Doctor |
| `t` | Cycle migration target |
| `p` | Preview plan |
| `e` | Export plan |
| `:` | Command palette |
| `h` / `?` | Help |

## Browse vs act

Inspect / doctor / export are safe. **Repair**, **plan apply**, and **rescue** mutate disks — shut down the guest, dry-run first, keep a backup.

## Related

- [Getting Started](getting-started.md)
- [TUI](pages/interfaces/tui.md)
- [Common workflows](workflows.md)
