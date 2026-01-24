# Next Priority Enhancements

This document outlines the next tier of enhancements to implement after completing the 5 quick wins.

## Status of Quick Wins

✅ All 5 quick wins completed:
1. Python Context Manager - Done
2. Python Type Hints - Done
3. Shell Completion - Done
4. Progress Bars - Already existed
5. Colorized Output - Done

## Next Tier: High-Impact Enhancements

These enhancements build on the foundation and deliver significant value to users.

---

## 1. PyPI Publication (HIGHEST PRIORITY)

**Impact:** 🟢 Very High - Makes installation effortless
**Effort:** 🟡 Medium (1-2 days)
**Dependencies:** None (ready to go!)

### Why This Matters

Currently users must:
```bash
git clone https://github.com/ssahani/guestctl
cd guestctl
cargo build --release
maturin develop
```

After PyPI publication:
```bash
pip install guestctl
```

That's it! This will **massively** increase adoption.

### Implementation Steps

#### Step 1: Configure Project Metadata

Ensure `pyproject.toml` has all required fields:

```toml
[project]
name = "guestctl"
version = "0.3.0"
description = "Modern Rust-based VM disk inspection and manipulation tool"
authors = [{name = "Susant Sahani", email = "ssahani@gmail.com"}]
license = {text = "LGPL-3.0-or-later"}
readme = "README.md"
requires-python = ">=3.8"
classifiers = [
    "Development Status :: 4 - Beta",
    "Intended Audience :: Developers",
    "Intended Audience :: System Administrators",
    "License :: OSI Approved :: GNU Lesser General Public License v3 or later (LGPLv3+)",
    "Programming Language :: Python :: 3",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
    "Programming Language :: Python :: 3.12",
    "Programming Language :: Rust",
    "Operating System :: POSIX :: Linux",
    "Topic :: System :: Systems Administration",
    "Topic :: Software Development :: Libraries :: Python Modules",
]
keywords = ["libguestfs", "vm", "disk", "qcow2", "inspection", "virtual-machine"]

[project.urls]
Homepage = "https://github.com/ssahani/guestctl"
Documentation = "https://github.com/ssahani/guestctl/tree/main/docs"
Repository = "https://github.com/ssahani/guestctl"
Issues = "https://github.com/ssahani/guestctl/issues"
Changelog = "https://github.com/ssahani/guestctl/blob/main/CHANGELOG.md"

[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
features = ["python-bindings"]
module-name = "guestctl"
include = ["guestctl.pyi"]
python-source = "python"
strip = true
```

#### Step 2: Create GitHub Actions Workflow

Create `.github/workflows/build-wheels.yml`:

```yaml
name: Build and Publish

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

permissions:
  contents: read

jobs:
  linux:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        target: [x86_64, aarch64]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          args: --release --out dist --features python-bindings
          sccache: 'true'
          manylinux: auto
      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels-linux-${{ matrix.target }}
          path: dist

  macos:
    runs-on: macos-latest
    strategy:
      matrix:
        target: [x86_64, aarch64]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'
      - name: Build wheels
        uses: PyO3/maturin-action@v1
        with:
          target: ${{ matrix.target }}
          args: --release --out dist --features python-bindings
          sccache: 'true'
      - name: Upload wheels
        uses: actions/upload-artifact@v4
        with:
          name: wheels-macos-${{ matrix.target }}
          path: dist

  sdist:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build sdist
        uses: PyO3/maturin-action@v1
        with:
          command: sdist
          args: --out dist
      - name: Upload sdist
        uses: actions/upload-artifact@v4
        with:
          name: wheels-sdist
          path: dist

  publish:
    name: Publish to PyPI
    runs-on: ubuntu-latest
    needs: [linux, macos, sdist]
    if: startsWith(github.ref, 'refs/tags/')
    environment:
      name: pypi
      url: https://pypi.org/p/guestctl
    permissions:
      id-token: write
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: wheels-*
          merge-multiple: true
          path: dist
      - name: Publish to PyPI
        uses: PyO3/maturin-action@v1
        with:
          command: upload
          args: --skip-existing dist/*
```

#### Step 3: Test Build Locally

```bash
# Build wheel for current platform
maturin build --release --features python-bindings

# Test in clean environment
python3 -m venv test-env
source test-env/bin/activate
pip install target/wheels/guestctl-*.whl

# Verify it works
python3 -c "from guestctl import Guestfs; print('Success!')"
```

#### Step 4: Publish to TestPyPI First

```bash
# Build
maturin build --release --features python-bindings

# Upload to TestPyPI
maturin upload --repository testpypi target/wheels/*

# Test installation
pip install --index-url https://test.pypi.org/simple/ guestctl
```

#### Step 5: Publish to PyPI

```bash
# Create a tag
git tag v0.3.0
git push origin v0.3.0

# GitHub Actions will automatically build and publish!
```

#### Step 6: Create PyPI Account Setup

1. Create account at https://pypi.org
2. Enable 2FA
3. Create API token
4. Add token to GitHub secrets as `PYPI_API_TOKEN`

### Benefits After Implementation

- ✅ `pip install guestctl` works
- ✅ Works on Linux x86_64 and aarch64
- ✅ Works on macOS x86_64 and aarch64
- ✅ Automatic builds on every release
- ✅ Source distribution available
- ✅ Listed on PyPI with proper metadata

---

## 2. Async Python API

**Impact:** 🟢 High - Modern Python expectations
**Effort:** 🟡 Medium (1-2 days)
**Dependencies:** PyPI publication (recommended)

### Why This Matters

Modern Python applications are async-first. Long-running operations like VM inspection should not block the event loop.

**Current (Blocking):**
```python
def inspect_multiple_vms(disks):
    results = []
    for disk in disks:  # Sequential, slow
        g = Guestfs()
        g.add_drive_ro(disk)
        g.launch()
        results.append(g.inspect_os())
    return results

# Takes 30 seconds for 10 VMs
```

**Enhanced (Async):**
```python
async def inspect_multiple_vms(disks):
    tasks = [inspect_vm(disk) for disk in disks]
    results = await asyncio.gather(*tasks)  # Parallel, fast!
    return results

# Takes 5 seconds for 10 VMs (6x faster!)
```

### Implementation Steps

#### Step 1: Add Dependencies

Add to `Cargo.toml`:
```toml
[dependencies]
pyo3-asyncio = { version = "0.20", features = ["tokio-runtime"] }
tokio = { version = "1", features = ["full"] }
```

#### Step 2: Create Async Wrapper Class

Add to `src/python.rs`:

```rust
use pyo3_asyncio::tokio::future_into_py;
use tokio::runtime::Runtime;

#[pyclass]
pub struct AsyncGuestfs {
    handle: Arc<Mutex<GuestfsHandle>>,
    runtime: Arc<Runtime>,
}

#[pymethods]
impl AsyncGuestfs {
    #[new]
    fn new() -> PyResult<Self> {
        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("Failed to create runtime: {}", e)
            ))?;

        Ok(AsyncGuestfs {
            handle: Arc::new(Mutex::new(GuestfsHandle::new()?)),
            runtime: Arc::new(runtime),
        })
    }

    fn __aenter__<'p>(slf: PyRef<'p, Self>, py: Python<'p>) -> PyResult<&'p PyAny> {
        let slf_copy = slf.into();
        future_into_py(py, async move {
            Ok(slf_copy)
        })
    }

    fn __aexit__<'p>(
        &mut self,
        py: Python<'p>,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_value: Option<&Bound<'_, PyAny>>,
        _traceback: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<&'p PyAny> {
        let handle = self.handle.clone();
        future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.shutdown()?;
            Ok(false)
        })
    }

    fn add_drive_ro<'p>(&self, py: Python<'p>, filename: String) -> PyResult<&'p PyAny> {
        let handle = self.handle.clone();
        future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.add_drive_ro(&filename)?;
            Ok(())
        })
    }

    fn launch<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let handle = self.handle.clone();
        future_into_py(py, async move {
            let mut h = handle.lock().await;
            h.launch()?;
            Ok(())
        })
    }

    fn inspect_os<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let handle = self.handle.clone();
        future_into_py(py, async move {
            let h = handle.lock().await;
            let roots = h.inspect_os()?;
            Ok(roots)
        })
    }

    // Add more async methods...
}

#[pymodule]
fn guestctl(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Guestfs>()?;
    m.add_class::<AsyncGuestfs>()?;  // Add async class
    m.add_class::<DiskConverter>()?;
    Ok(())
}
```

#### Step 3: Update Type Hints

Add to `guestctl.pyi`:

```python
import asyncio
from typing import List, Dict, Optional, Any
from types import TracebackType

class AsyncGuestfs:
    """Async version of Guestfs for non-blocking operations."""

    def __init__(self) -> None: ...

    async def __aenter__(self) -> 'AsyncGuestfs': ...
    async def __aexit__(
        self,
        exc_type: Optional[type[BaseException]],
        exc_value: Optional[BaseException],
        traceback: Optional[TracebackType]
    ) -> bool: ...

    async def add_drive_ro(self, filename: str) -> None: ...
    async def launch(self) -> None: ...
    async def inspect_os(self) -> List[str]: ...
    async def inspect_get_type(self, root: str) -> str: ...
    async def inspect_get_distro(self, root: str) -> str: ...
    # ... all other methods as async
```

#### Step 4: Create Example

Create `examples/python/async_example.py`:

```python
import asyncio
from guestctl import AsyncGuestfs

async def inspect_vm(disk_path: str) -> dict:
    """Inspect a single VM asynchronously."""
    async with AsyncGuestfs() as g:
        await g.add_drive_ro(disk_path)
        await g.launch()

        roots = await g.inspect_os()
        if not roots:
            return {"error": "No OS found"}

        root = roots[0]
        return {
            "disk": disk_path,
            "os_type": await g.inspect_get_type(root),
            "distro": await g.inspect_get_distro(root),
            "version": f"{await g.inspect_get_major_version(root)}.{await g.inspect_get_minor_version(root)}",
        }

async def inspect_multiple_vms(disk_paths: list[str]):
    """Inspect multiple VMs concurrently."""
    tasks = [inspect_vm(disk) for disk in disk_paths]
    results = await asyncio.gather(*tasks, return_exceptions=True)

    for disk, result in zip(disk_paths, results):
        if isinstance(result, Exception):
            print(f"❌ {disk}: {result}")
        else:
            print(f"✅ {disk}: {result}")

if __name__ == "__main__":
    disks = [
        "vm1.qcow2",
        "vm2.qcow2",
        "vm3.qcow2",
    ]

    asyncio.run(inspect_multiple_vms(disks))
```

### Benefits After Implementation

- ✅ Non-blocking operations
- ✅ Concurrent VM inspection
- ✅ Integration with async frameworks (FastAPI, aiohttp)
- ✅ Better resource utilization
- ✅ Modern Python ecosystem compatibility

---

## 3. Interactive CLI Mode (REPL)

**Impact:** 🟢 High - Much better UX for exploration
**Effort:** 🟡 Medium (1-2 days)
**Dependencies:** None

### Why This Matters

Currently, every operation requires a full command:
```bash
guestctl inspect disk.img
guestctl filesystems disk.img
guestctl cat disk.img /etc/hostname
```

With interactive mode:
```bash
$ guestctl interactive disk.img
Loaded disk.img
guestctl> inspect
OS: Ubuntu 22.04 LTS

guestctl> filesystems
/dev/sda1: ext4
/dev/sda2: swap

guestctl> cat /etc/hostname
ubuntu-server

guestctl> help
Available commands: inspect, filesystems, cat, ls, download, ...
```

**Much faster for exploration!**

### Implementation Steps

#### Step 1: Add Dependencies

```toml
[dependencies]
rustyline = "14.0"
rustyline-derive = "0.10"
```

#### Step 2: Create Interactive Module

Create `src/cli/interactive.rs`:

```rust
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use rustyline::history::FileHistory;
use crate::guestfs::GuestfsHandle;
use std::path::Path;

pub struct InteractiveSession {
    handle: GuestfsHandle,
    editor: DefaultEditor,
    disk_path: String,
}

impl InteractiveSession {
    pub fn new(disk_path: &str) -> Result<Self> {
        let mut handle = GuestfsHandle::new()
            .map_err(|e| ReadlineError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create handle: {}", e)
            )))?;

        // Add drive and launch
        handle.add_drive_ro(disk_path)
            .map_err(|e| ReadlineError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to add drive: {}", e)
            )))?;

        println!("Launching appliance...");
        handle.launch()
            .map_err(|e| ReadlineError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to launch: {}", e)
            )))?;

        let editor = DefaultEditor::new()?;

        Ok(InteractiveSession {
            handle,
            editor,
            disk_path: disk_path.to_string(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("GuestCtl Interactive Mode");
        println!("Type 'help' for available commands, 'exit' to quit\n");

        loop {
            let readline = self.editor.readline("guestctl> ");
            match readline {
                Ok(line) => {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    self.editor.add_history_entry(line)?;

                    if line == "exit" || line == "quit" {
                        break;
                    }

                    if let Err(e) = self.execute_command(line) {
                        eprintln!("Error: {}", e);
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn execute_command(&mut self, line: &str) -> anyhow::Result<()> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "help" => self.show_help(),
            "inspect" => self.cmd_inspect(),
            "filesystems" | "fs" => self.cmd_filesystems(),
            "cat" => self.cmd_cat(&parts[1..]),
            "ls" => self.cmd_ls(&parts[1..]),
            "download" => self.cmd_download(&parts[1..]),
            "packages" | "pkg" => self.cmd_packages(),
            "services" | "svc" => self.cmd_services(),
            "users" => self.cmd_users(),
            "mount" => self.cmd_mount(&parts[1..]),
            "umount" => self.cmd_umount(&parts[1..]),
            _ => {
                println!("Unknown command: {}", parts[0]);
                println!("Type 'help' for available commands");
                Ok(())
            }
        }
    }

    fn show_help(&self) -> anyhow::Result<()> {
        println!("Available commands:");
        println!("  inspect              - Show OS information");
        println!("  filesystems, fs      - List filesystems");
        println!("  mount <device> <dir> - Mount a filesystem");
        println!("  umount <dir>         - Unmount a filesystem");
        println!("  ls [path]            - List directory contents");
        println!("  cat <path>           - Display file contents");
        println!("  download <src> <dst> - Download file from disk");
        println!("  packages, pkg        - List installed packages");
        println!("  services, svc        - List system services");
        println!("  users                - List user accounts");
        println!("  help                 - Show this help");
        println!("  exit, quit           - Exit interactive mode");
        Ok(())
    }

    fn cmd_inspect(&mut self) -> anyhow::Result<()> {
        let roots = self.handle.inspect_os()?;
        if roots.is_empty() {
            println!("No operating system found");
            return Ok(());
        }

        let root = &roots[0];
        let os_type = self.handle.inspect_get_type(root)?;
        let distro = self.handle.inspect_get_distro(root)?;
        let major = self.handle.inspect_get_major_version(root)?;
        let minor = self.handle.inspect_get_minor_version(root)?;

        println!("OS Type: {}", os_type);
        println!("Distribution: {}", distro);
        println!("Version: {}.{}", major, minor);

        Ok(())
    }

    fn cmd_filesystems(&mut self) -> anyhow::Result<()> {
        let filesystems = self.handle.list_filesystems()?;
        for (device, fstype) in filesystems {
            println!("{}: {}", device, fstype);
        }
        Ok(())
    }

    fn cmd_cat(&mut self, args: &[&str]) -> anyhow::Result<()> {
        if args.is_empty() {
            println!("Usage: cat <path>");
            return Ok(());
        }

        let content = self.handle.cat(args[0])?;
        println!("{}", content);
        Ok(())
    }

    fn cmd_ls(&mut self, args: &[&str]) -> anyhow::Result<()> {
        let path = if args.is_empty() { "/" } else { args[0] };
        let entries = self.handle.ls(path)?;
        for entry in entries {
            println!("{}", entry);
        }
        Ok(())
    }

    fn cmd_download(&mut self, args: &[&str]) -> anyhow::Result<()> {
        if args.len() < 2 {
            println!("Usage: download <remote_path> <local_path>");
            return Ok(());
        }

        self.handle.download(args[0], args[1])?;
        println!("Downloaded {} to {}", args[0], args[1]);
        Ok(())
    }

    fn cmd_packages(&mut self) -> anyhow::Result<()> {
        let roots = self.handle.inspect_os()?;
        if roots.is_empty() {
            println!("No OS found");
            return Ok(());
        }

        let apps = self.handle.inspect_list_applications(&roots[0])?;
        println!("Installed packages: {}", apps.len());
        for app in apps.iter().take(20) {
            println!("  {} {}", app.app_name, app.app_version);
        }
        if apps.len() > 20 {
            println!("  ... and {} more", apps.len() - 20);
        }
        Ok(())
    }

    fn cmd_services(&mut self) -> anyhow::Result<()> {
        // Implement service listing
        println!("Service listing not yet implemented");
        Ok(())
    }

    fn cmd_users(&mut self) -> anyhow::Result<()> {
        // Implement user listing
        println!("User listing not yet implemented");
        Ok(())
    }

    fn cmd_mount(&mut self, args: &[&str]) -> anyhow::Result<()> {
        if args.len() < 2 {
            println!("Usage: mount <device> <mountpoint>");
            return Ok(());
        }

        self.handle.mount(args[0], args[1])?;
        println!("Mounted {} at {}", args[0], args[1]);
        Ok(())
    }

    fn cmd_umount(&mut self, args: &[&str]) -> anyhow::Result<()> {
        if args.is_empty() {
            println!("Usage: umount <mountpoint>");
            return Ok(());
        }

        self.handle.umount(args[0])?;
        println!("Unmounted {}", args[0]);
        Ok(())
    }
}
```

#### Step 3: Add Interactive Command

In `src/cli/commands.rs`:

```rust
use crate::cli::interactive::InteractiveSession;

#[derive(Args)]
pub struct InteractiveArgs {
    /// Path to disk image
    pub image: String,
}

pub fn interactive(args: InteractiveArgs) -> anyhow::Result<()> {
    let mut session = InteractiveSession::new(&args.image)?;
    session.run()?;
    Ok(())
}
```

Add to `src/cli/mod.rs`:
```rust
pub mod interactive;
```

Add to `src/main.rs`:
```rust
Commands::Interactive(args) => commands::interactive(args),
```

#### Step 4: Add Command Completion

Add tab completion for commands and paths within interactive mode using rustyline's completion features.

### Benefits After Implementation

- ✅ Much faster exploration workflow
- ✅ Persistent session (no re-launching appliance)
- ✅ Command history with arrow keys
- ✅ Tab completion
- ✅ Similar UX to guestfish but with Rust performance

---

## 4. Distribution Packages

**Impact:** 🟢 High - Native installation experience
**Effort:** 🟡 Medium (varies by distro)
**Dependencies:** PyPI publication (for .deb/.rpm Python packages)

### Why This Matters

Users prefer native package managers:

```bash
# Debian/Ubuntu
sudo apt install guestctl

# Fedora/RHEL
sudo dnf install guestctl

# Arch Linux
yay -S guestctl
```

### Implementation Steps

#### Step 1: Debian/Ubuntu Package

Create `debian/` directory structure:

```
debian/
├── changelog
├── control
├── copyright
├── rules
└── source/
    └── format
```

`debian/control`:
```
Source: guestctl
Section: admin
Priority: optional
Maintainer: Susant Sahani <ssahani@gmail.com>
Build-Depends: debhelper-compat (= 13), cargo, rustc (>= 1.70)
Standards-Version: 4.6.0
Homepage: https://github.com/ssahani/guestctl

Package: guestctl
Architecture: any
Depends: ${shlibs:Depends}, ${misc:Depends}
Description: Modern VM disk inspection and manipulation tool
 GuestCtl is a Rust-based tool for inspecting and manipulating
 virtual machine disk images without mounting them. It provides
 a safe, fast alternative to libguestfs.
```

`debian/rules`:
```makefile
#!/usr/bin/make -f

%:
	dh $@

override_dh_auto_build:
	cargo build --release

override_dh_auto_install:
	install -D -m 755 target/release/guestctl $(DESTDIR)/usr/bin/guestctl
```

Build:
```bash
dpkg-buildpackage -us -uc
```

#### Step 2: RPM Package

Create `guestctl.spec`:

```spec
Name:           guestctl
Version:        0.3.0
Release:        1%{?dist}
Summary:        Modern VM disk inspection and manipulation tool

License:        LGPL-3.0-or-later
URL:            https://github.com/ssahani/guestctl
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo

%description
GuestCtl is a Rust-based tool for inspecting and manipulating
virtual machine disk images without mounting them.

%prep
%autosetup

%build
cargo build --release

%install
install -D -m 755 target/release/guestctl %{buildroot}%{_bindir}/guestctl

%files
%license LICENSE
%doc README.md
%{_bindir}/guestctl

%changelog
* Fri Jan 24 2026 Susant Sahani <ssahani@gmail.com> - 0.3.0-1
- Initial package
```

Build:
```bash
rpmbuild -ba guestctl.spec
```

#### Step 3: Arch Linux (AUR)

Create `PKGBUILD`:

```bash
# Maintainer: Susant Sahani <ssahani@gmail.com>
pkgname=guestctl
pkgver=0.3.0
pkgrel=1
pkgdesc="Modern VM disk inspection and manipulation tool"
arch=('x86_64' 'aarch64')
url="https://github.com/ssahani/guestctl"
license=('LGPL3')
depends=()
makedepends=('rust' 'cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/ssahani/guestctl/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
    cd "$pkgname-$pkgver"
    cargo build --release --locked
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 target/release/guestctl "$pkgdir/usr/bin/guestctl"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 README.md "$pkgdir/usr/share/doc/$pkgname/README.md"
}
```

Submit to AUR:
```bash
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Initial commit"
git push
```

#### Step 4: Automate with GitHub Actions

Create `.github/workflows/packages.yml`:

```yaml
name: Build Packages

on:
  release:
    types: [published]

jobs:
  debian:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build deb package
        run: |
          sudo apt-get update
          sudo apt-get install -y debhelper devscripts
          dpkg-buildpackage -us -uc
      - name: Upload deb
        uses: actions/upload-artifact@v4
        with:
          name: debian-package
          path: ../*.deb

  rpm:
    runs-on: fedora-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build RPM
        run: |
          sudo dnf install -y rpm-build rust cargo
          rpmbuild -ba guestctl.spec
      - name: Upload RPM
        uses: actions/upload-artifact@v4
        with:
          name: rpm-package
          path: ~/rpmbuild/RPMS/*/*.rpm
```

### Benefits After Implementation

- ✅ Native installation via system package manager
- ✅ Automatic dependency management
- ✅ System-wide availability
- ✅ Easy updates via package manager
- ✅ Integration with system tools

---

## 5. Enhanced Documentation Site

**Impact:** 🟡 Medium - Better discoverability and onboarding
**Effort:** 🟡 Medium (2-3 days)
**Dependencies:** None

### Why This Matters

Current documentation is in GitHub markdown. A dedicated site provides:
- Better navigation
- Search functionality
- Code examples with syntax highlighting
- API browsing
- Multi-version support

### Implementation Steps

#### Option 1: MkDocs Material (Recommended)

Create `mkdocs.yml`:

```yaml
site_name: GuestCtl Documentation
site_description: Modern VM disk inspection and manipulation
site_author: Susant Sahani
site_url: https://ssahani.github.io/guestctl/

repo_name: ssahani/guestctl
repo_url: https://github.com/ssahani/guestctl

theme:
  name: material
  palette:
    - scheme: default
      primary: indigo
      accent: indigo
      toggle:
        icon: material/brightness-7
        name: Switch to dark mode
    - scheme: slate
      primary: indigo
      accent: indigo
      toggle:
        icon: material/brightness-4
        name: Switch to light mode
  features:
    - navigation.tabs
    - navigation.sections
    - navigation.expand
    - navigation.top
    - search.suggest
    - search.highlight
    - content.code.copy

plugins:
  - search
  - mkdocstrings

markdown_extensions:
  - pymdownx.highlight
  - pymdownx.superfences
  - pymdownx.tabbed
  - pymdownx.emoji
  - admonition
  - tables

nav:
  - Home: index.md
  - Getting Started:
    - Installation: guides/installation.md
    - Quick Start: guides/QUICKSTART.md
  - User Guide:
    - CLI Guide: guides/CLI_GUIDE.md
    - Python Bindings: guides/PYTHON_BINDINGS.md
    - Output Formats: guides/OUTPUT_FORMATS.md
    - Profiles: guides/PROFILES_GUIDE.md
  - API Reference:
    - Python API: api/PYTHON_API_REFERENCE.md
    - Rust API: api/API_REFERENCE.md
  - Architecture:
    - Overview: architecture/ARCHITECTURE.md
    - vs libguestfs: architecture/LIBGUESTFS_COMPARISON.md
  - Development:
    - Contributing: CONTRIBUTING.md
    - Roadmap: development/ROADMAP.md
```

Deploy to GitHub Pages:

```yaml
# .github/workflows/docs.yml
name: Deploy Documentation

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: 3.x
      - run: pip install mkdocs-material mkdocstrings
      - run: mkdocs gh-deploy --force
```

---

## Implementation Priority Order

### Week 1: Distribution Focus
1. **Day 1-2:** PyPI Publication (highest impact)
2. **Day 3-4:** Test PyPI packages, fix issues
3. **Day 5:** Documentation updates for PyPI

### Week 2: Python Enhancements
1. **Day 1-2:** Async Python API
2. **Day 3:** Test async API
3. **Day 4-5:** Examples and documentation

### Week 3: User Experience
1. **Day 1-2:** Interactive CLI mode
2. **Day 3:** Documentation site setup
3. **Day 4-5:** Package building (.deb, .rpm)

### Week 4: Polish and Release
1. **Day 1-2:** Testing all enhancements
2. **Day 3:** Bug fixes
3. **Day 4:** Documentation polish
4. **Day 5:** Release v0.4.0

---

## Success Metrics

### After PyPI Publication
- ✅ Package installable via `pip install guestctl`
- ✅ Works on Linux (x86_64, aarch64)
- ✅ Works on macOS (x86_64, aarch64)
- ✅ Listed on PyPI with complete metadata
- 📊 Target: 100+ downloads in first week

### After Async API
- ✅ All core methods have async versions
- ✅ 6x+ speedup for parallel operations
- ✅ Works with asyncio, FastAPI, aiohttp
- 📊 Target: Used in at least one async web framework example

### After Interactive Mode
- ✅ REPL with command history
- ✅ Tab completion
- ✅ 10+ commands available
- 📊 Target: Positive user feedback

### After Distribution Packages
- ✅ .deb package for Debian/Ubuntu
- ✅ .rpm package for Fedora/RHEL
- ✅ AUR package for Arch
- 📊 Target: Available in at least 2 distro repositories

---

## Questions Before Starting

Before implementing, consider:

1. **PyPI Publication**
   - Do you have a PyPI account?
   - Should we start with TestPyPI first?
   - What version number for first release? (suggest 0.3.0)

2. **Async API**
   - Should async be in same class or separate `AsyncGuestfs`?
   - Which runtime: Tokio or async-std? (recommend Tokio)

3. **Interactive Mode**
   - Should it auto-mount common paths?
   - Should it have command aliases (e.g., `fs` for `filesystems`)?

4. **Packages**
   - Which distros are priority? (suggest Ubuntu, Fedora, Arch)
   - Should we target specific LTS versions?

---

## Ready to Implement?

All 5 enhancements are ready to implement with the guides above. The recommended order is:

1. **PyPI Publication** - Biggest impact, enables easy adoption
2. **Async Python API** - Modern Python expectation
3. **Interactive Mode** - Best UX improvement
4. **Distribution Packages** - Native installation
5. **Documentation Site** - Professional presentation

Each enhancement builds on the solid foundation from the quick wins!
