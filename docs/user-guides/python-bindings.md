# GuestCtl Python Bindings

Python bindings for GuestCtl - Pure Rust toolkit for VM disk image inspection and manipulation.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Advanced Usage](#advanced-usage)
- [Error Handling](#error-handling)

## Installation

### Prerequisites

- Python 3.7 or later
- Rust toolchain (for building from source)
- System dependencies:
  - `qemu-img`, `qemu-nbd` (QEMU tools)
  - `lvm2` (for LVM support)
  - Root/sudo access (required for mounting operations)

### Build from Source

```bash
# Clone repository
git clone https://github.com/ssahani/guestctl
cd guestctl

# Install maturin (PyO3 build tool)
pip install maturin

# Build and install Python package
maturin develop --features python-bindings

# Or build wheel for distribution
maturin build --release --features python-bindings
pip install target/wheels/guestctl-*.whl
```

### Verify Installation

```python
import guestctl
print(guestctl.__version__)
```

## Quick Start

### Basic Inspection

```python
from guestctl import Guestfs

# Create handle
g = Guestfs()

# Add disk image (read-only)
g.add_drive_ro("/path/to/disk.qcow2")

# Launch appliance
g.launch()

# Inspect OS
roots = g.inspect_os()
if roots:
    root = roots[0]
    print(f"OS Type: {g.inspect_get_type(root)}")
    print(f"Distribution: {g.inspect_get_distro(root)}")
    print(f"Hostname: {g.inspect_get_hostname(root)}")

    # Mount filesystems
    mountpoints = g.inspect_get_mountpoints(root)
    for mp, device in sorted(mountpoints.items(), key=lambda x: len(x[0])):
        g.mount_ro(device, mp)

    # Read files
    if g.is_file("/etc/hostname"):
        hostname = g.cat("/etc/hostname")
        print(f"Hostname from file: {hostname}")

# Cleanup
g.umount_all()
g.shutdown()
```

## API Reference

See comprehensive examples in `examples/python/` directory.

**Full documentation:** 100+ Python bindings methods covering all GuestCtl functionality.

---

**Built with GuestCtl** 🐍
