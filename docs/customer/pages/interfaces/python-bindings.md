# Python Bindings

## Purpose

Python Bindings — Interfaces surface.

## When to use it

- Operate **Python Bindings** when your job matches this surface
- Prefer dry-run / doctor before mutating repairs on disks
- Shut down the guest before write operations

## How to get there

- Doc id: `python-bindings`
- Nav: **Interfaces → Python Bindings**
- Primary interface: `pip install hypersdk-guestkit` or `maturin develop --features python-bindings`

## Operate from CLI / TUI (UX)

1. `pip install hypersdk-guestkit` or `maturin develop --features python-bindings`.
2. `from guestkit import Guestfs`.
3. `add_drive_ro` → `launch` → `inspect_os`.
4. Mount via `inspect_get_mountpoints` + `mount_ro`.
5. `cat`/`ls` as needed.
6. `umount_all` + `shutdown`; see `examples/python/`.
7. **Empty / fail:** Import error → wrong package/feature; launch fail → NBD/sudo.
8. **Success:** Distro/hostname printed; clean shutdown.

Host needs Linux + `qemu-img` / losetup / qemu-nbd; mount/repair often need root. GuestKit does not invent disk contents.

## Related pages

- [Inspect](../inspection/inspect.md)
- [CLI Guide](../onboarding/cli-guide.md)
- [Guest Files](../guest-files/files.md)
- [Getting Started](../../getting-started.md)
- [Page index](../../PAGE_INDEX.md)
