# CLI reference (v0.3.6)

`guestkit` and `guestctl` share the same command surface. Use **`guestkit --help`** and **`guestkit <command> --help`** for flags; this page links the curated docs.

## Quick start

```bash
guestkit inspect disk.qcow2
guestkit doctor disk.qcow2 --target proxmox --explain
guestkit doctor disk.qcow2 --target proxmox -o json --fail-below 80
guestkit migrate-plan disk.qcow2 --target kvm --export plan.yaml
guestctl tui disk.qcow2
```

## Where to look

| Topic | Doc |
|-------|-----|
| Cheat sheet | [quick-reference.md](quick-reference.md) |
| Install & build | [getting-started.md](getting-started.md) |
| Migration assurance | [migration-assurance.md](../features/migration-assurance.md) |
| VM migration workflows | [vm-migration.md](vm-migration.md) |
| TUI keys & Assurance | [tui-enhancements.md](../features/tui-enhancements.md) |
| Fix plans | [fix-plans.md](../features/fix-plans.md) |
| Profiles | [profiles.md](profiles.md) |
| Interactive REPL | [interactive-mode.md](interactive-mode.md) |
| File explorer | [EXPLORE-QUICKSTART.md](../features/explore/EXPLORE-QUICKSTART.md) |
| Python API | [python-bindings.md](python-bindings.md) |
| FAQ | [faq.md](faq.md) |
| Troubleshooting | [troubleshooting.md](troubleshooting.md) |

## Command groups

| Group | Examples |
|-------|----------|
| Inspect | `inspect`, `filesystems`, `packages`, `services`, `users`, `network` |
| Files | `ls`, `cat`, `cp`, `download`, `upload`, `find` |
| Assurance | `doctor`, `migrate-plan`, `policy`, `fleet`, `forensic-diff`, `repair --fix boot` |
| Plans | `plan preview`, `plan apply`, `plan rollback` |
| Profiles | `profile security`, `profile windows-migration` |
| TUI | `guestctl tui`, `guestkit tui` |
| Shell | `guestkit shell`, `guestkit interactive` |

List all commands: **`guestkit commands`** (or **`guestkit command-catalog`**).

## Disk formats

QCOW2, VMDK, VDI, VHD, RAW, IMG — auto-detected. RAW/IMG often use loop devices; QCOW2/VMDK use NBD. Use **`--trace`** to see which path is used.

## JSON output

Most inspect commands accept **`-o json`** or **`--json`** for scripting. See [quick-reference.md](quick-reference.md) for examples.

## See also

- [Documentation index](../INDEX.md)
- [Changelog](../development/CHANGELOG.md)
- [zyvor.dev/guestkit](https://zyvor.dev/guestkit)
