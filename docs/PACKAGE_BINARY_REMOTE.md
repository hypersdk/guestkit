# Package GuestKit as a Linux binary (remote build)

Ship a **client tarball** (`guestkit` + install/test scripts) without giving customers deploy scripts.

## What you get

```
guestkit-0.3.3-linux-amd64/
  guestkit              # CLI binary
  install.sh            # one-command install
  install-client-deps.sh
  test-host.sh          # libguestfs / qemu / nbd checks
  test-selftest.sh      # full GuestKit selftest
  test-package.sh
  uninstall.sh
  HOST_SETUP.txt
  PREREQUISITES.txt
  guestkit.env.example
```

## Build

**GitHub Release** (tag `v*`): CI builds the same customer tarball and attaches it to the release:

- `guestkit-<version>-linux-amd64.tar.gz` (+ `.sha256`)
- `guestkit-<version>-linux-amd64-musl.tar.gz` (+ `.sha256`)

**Remote Linux host:**

```bash
./scripts/package-binary-remote.sh 212.8.252.194 sus --fetch
./scripts/package-binary-remote.sh 212.8.252.194 sus --reuse-build --skip-deps
```

**Local / CI** (after `cargo build --release` on Linux):

```bash
./scripts/package-binary-release.sh --build
./scripts/package-binary-release.sh --build --target x86_64-unknown-linux-musl
```

Environment:

| Variable | Purpose |
|----------|---------|
| `GUESTKIT_PACKAGE_DIR` | Remote output dir (default `~/guestkit-dist`) |
| `GUESTKIT_PACKAGE_VERSION` | Override version string |
| `GUESTKIT_REMOTE_SKIP_SSH_CHECK=1` | Skip SSH preflight |

## Customer install

```bash
tar xzf guestkit-*-linux-amd64.tar.gz && cd guestkit-*-linux-amd64
./install.sh
./test-host.sh
./test-selftest.sh --quick
./guestkit inspect /path/to/disk.qcow2
```

## Customer uninstall

```bash
./uninstall.sh --yes
./uninstall.sh --yes --remove-dir
```

## Host requirements (not Kubernetes)

- libguestfs-tools, qemu-img, nbd kernel module
- Disk image files (QCOW2, VMDK, RAW, …)
- See `HOST_SETUP.txt` in the tarball

For full remote deploy from source, use `./scripts/deploy-remote.sh`.
