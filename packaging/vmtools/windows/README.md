# Zeus VM Tools — Windows (Phase 4)

Windows guest tools ship via **virtio-win** today. Native MSI packaging is planned here.

## Current path

1. Open Zeus OS → VM → **Guest Tools**
2. Attach `virtio-win.iso`
3. Install QEMU Guest Agent / virtio-win guest tools inside the VM

## Future MSI

`build-msi.sh` will wrap the GuestKit Windows agent binary with WiX when the
Windows agent service is ready. Deferred until the in-guest Windows service
matches Linux `zyvor-guest-agent` capabilities.
