# Quick Enhancements Guide

This guide shows how to implement the most impactful enhancements with minimal effort.

## 🚀 Top 5 Quick Wins (1-2 Hours Each)

### 1. Python Context Manager (1 hour)

**File:** `src/python.rs`

Add these methods to the `Guestfs` impl block:

```rust
#[pymethods]
impl Guestfs {
    // ... existing methods ...

    fn __enter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exc_type: &PyAny,
        _exc_value: &PyAny,
        _traceback: &PyAny,
    ) -> PyResult<bool> {
        self.shutdown()?;
        Ok(false)
    }
}
```

**Test:**
```python
with Guestfs() as g:
    g.add_drive_ro("disk.img")
    g.launch()
    roots = g.inspect_os()
    print(roots)
# Automatic cleanup!
```

---

### 2. Python Type Hints (1-2 hours)

**File:** `guestctl.pyi` (create new)

```python
"""Type stubs for guestctl"""
from typing import List, Dict, Any, Optional

__version__: str

class Guestfs:
    def __init__(self) -> None: ...
    def __enter__(self) -> 'Guestfs': ...
    def __exit__(self, exc_type: Any, exc_value: Any, traceback: Any) -> bool: ...

    # Drive operations
    def add_drive(self, filename: str) -> None: ...
    def add_drive_ro(self, filename: str) -> None: ...
    def launch(self) -> None: ...
    def shutdown(self) -> None: ...
    def set_verbose(self, verbose: bool) -> None: ...

    # Inspection
    def inspect_os(self) -> List[str]: ...
    def inspect_get_type(self, root: str) -> str: ...
    def inspect_get_distro(self, root: str) -> str: ...
    def inspect_get_hostname(self, root: str) -> str: ...
    def inspect_get_arch(self, root: str) -> str: ...
    def inspect_get_major_version(self, root: str) -> int: ...
    def inspect_get_minor_version(self, root: str) -> int: ...
    def inspect_get_product_name(self, root: str) -> str: ...
    def inspect_get_package_format(self, root: str) -> str: ...
    def inspect_get_package_management(self, root: str) -> str: ...
    def inspect_get_mountpoints(self, root: str) -> Dict[str, str]: ...
    def inspect_list_applications(self, root: str) -> List[Dict[str, Any]]: ...

    # Device operations
    def list_devices(self) -> List[str]: ...
    def list_partitions(self) -> List[str]: ...
    def blockdev_getsize64(self, device: str) -> int: ...

    # Filesystem operations
    def vfs_type(self, device: str) -> str: ...
    def vfs_label(self, device: str) -> str: ...
    def vfs_uuid(self, device: str) -> str: ...
    def mount(self, device: str, mountpoint: str) -> None: ...
    def mount_ro(self, device: str, mountpoint: str) -> None: ...
    def umount(self, mountpoint: str) -> None: ...
    def umount_all(self) -> None: ...
    def sync(self) -> None: ...

    # File operations
    def read_file(self, path: str) -> bytes: ...
    def cat(self, path: str) -> str: ...
    def write(self, path: str, content: bytes) -> None: ...
    def exists(self, path: str) -> bool: ...
    def is_file(self, path: str) -> bool: ...
    def is_dir(self, path: str) -> bool: ...
    def ls(self, directory: str) -> List[str]: ...
    def download(self, remotefilename: str, filename: str) -> None: ...
    def upload(self, filename: str, remotefilename: str) -> None: ...

    # Directory operations
    def mkdir(self, path: str) -> None: ...
    def mkdir_p(self, path: str) -> None: ...
    def rm(self, path: str) -> None: ...
    def rmdir(self, path: str) -> None: ...
    def rm_rf(self, path: str) -> None: ...

    # Permissions
    def chmod(self, mode: int, path: str) -> None: ...
    def chown(self, owner: int, group: int, path: str) -> None: ...

    # Stat
    def stat(self, path: str) -> Dict[str, int]: ...
    def statvfs(self, path: str) -> Dict[str, int]: ...

    # Command execution
    def command(self, arguments: List[str]) -> str: ...
    def sh(self, command: str) -> str: ...
    def sh_lines(self, command: str) -> List[str]: ...

    # LVM
    def vgscan(self) -> None: ...
    def vgs(self) -> List[str]: ...
    def pvs(self) -> List[str]: ...
    def lvs(self) -> List[str]: ...

    # Archives
    def tar_in(self, tarfile: str, directory: str) -> None: ...
    def tar_out(self, directory: str, tarfile: str) -> None: ...
    def tgz_in(self, tarfile: str, directory: str) -> None: ...
    def tgz_out(self, directory: str, tarfile: str) -> None: ...

    # Checksum
    def checksum(self, csumtype: str, path: str) -> str: ...

class DiskConverter:
    def __init__(self) -> None: ...
    def convert(
        self,
        source: str,
        output: str,
        format: str = "qcow2",
        compress: bool = False,
        flatten: bool = True
    ) -> Dict[str, Any]: ...
    def detect_format(self, image: str) -> str: ...
    def get_info(self, image: str) -> Dict[str, Any]: ...
```

**Add to pyproject.toml:**
```toml
[tool.maturin]
include = ["guestctl.pyi"]
```

---

### 3. Shell Completion (1 hour)

**File:** `src/cli/mod.rs`

Add completion command:

```rust
use clap::CommandFactory;
use clap_complete::{generate, shells::*};

#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Generate shell completion scripts
    Completion {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(ValueEnum, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

// In main command handler
Commands::Completion { shell } => {
    let mut cmd = Cli::command();
    match shell {
        Shell::Bash => generate(Bash, &mut cmd, "guestctl", &mut io::stdout()),
        Shell::Zsh => generate(Zsh, &mut cmd, "guestctl", &mut io::stdout()),
        Shell::Fish => generate(Fish, &mut cmd, "guestctl", &mut io::stdout()),
        Shell::PowerShell => generate(PowerShell, &mut cmd, "guestctl", &mut io::stdout()),
        Shell::Elvish => generate(Elvish, &mut cmd, "guestctl", &mut io::stdout()),
    }
}
```

**Add to Cargo.toml:**
```toml
clap_complete = "4.5"
```

**Usage:**
```bash
# Generate and install
guestctl completion bash > /etc/bash_completion.d/guestctl
guestctl completion zsh > ~/.zsh/completion/_guestctl
guestctl completion fish > ~/.config/fish/completions/guestctl.fish
```

---

### 4. Progress Bars (1 hour)

**File:** `src/cli/commands.rs`

Already have `indicatif`, just use it:

```rust
use indicatif::{ProgressBar, ProgressStyle};

// In inspect command
pub fn inspect_with_progress(disk_path: &Path) -> Result<()> {
    let pb = ProgressBar::new(5);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
    );

    pb.set_message("Adding disk...");
    g.add_drive_ro(disk_path)?;
    pb.inc(1);

    pb.set_message("Launching appliance...");
    g.launch()?;
    pb.inc(1);

    pb.set_message("Detecting OS...");
    let roots = g.inspect_os()?;
    pb.inc(1);

    pb.set_message("Mounting filesystems...");
    mount_filesystems(&mut g, &roots[0])?;
    pb.inc(1);

    pb.set_message("Gathering information...");
    let info = gather_info(&mut g, &roots[0])?;
    pb.inc(1);

    pb.finish_with_message("Complete!");
    Ok(())
}
```

---

### 5. Color Output (1 hour)

**File:** `src/cli/mod.rs`

Already have `owo-colors`, apply consistently:

```rust
use owo_colors::OwoColorize;

// Create helper module
pub mod output {
    use owo_colors::OwoColorize;

    pub fn success(msg: &str) {
        println!("{} {}", "✓".green(), msg);
    }

    pub fn error(msg: &str) {
        eprintln!("{} {}", "✗".red(), msg.red());
    }

    pub fn warning(msg: &str) {
        println!("{} {}", "⚠".yellow(), msg.yellow());
    }

    pub fn info(msg: &str) {
        println!("{} {}", "ℹ".blue(), msg);
    }

    pub fn header(msg: &str) {
        println!("{}", msg.bold().underline());
    }

    pub fn kv(key: &str, value: &str) {
        println!("{}: {}", key.cyan(), value);
    }
}

// Usage
output::success("Disk inspected successfully");
output::header("OS Information");
output::kv("Distribution", "Ubuntu");
output::kv("Version", "22.04");
```

---

## 🎯 Medium Effort, High Impact (1 Day Each)

### 6. PyPI Publication

**Step 1:** Create `MANIFEST.in`
```
include README.md
include LICENSE
include pyproject.toml
include Cargo.toml
include Cargo.lock
recursive-include src *.rs
recursive-include docs *.md
```

**Step 2:** Build wheels
```bash
# Install dependencies
pip install maturin twine

# Build wheels
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin build --release --features python-bindings

# Test upload to TestPyPI
twine upload --repository testpypi target/wheels/*

# Upload to PyPI
twine upload target/wheels/*
```

**Step 3:** CI/CD (GitHub Actions)

Create `.github/workflows/python-release.yml`:

```yaml
name: Python Release

on:
  release:
    types: [created]

jobs:
  build-wheels:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        python-version: ['3.8', '3.9', '3.10', '3.11', '3.12']

    steps:
      - uses: actions/checkout@v4

      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python-version }}

      - name: Install maturin
        run: pip install maturin

      - name: Build wheel
        run: |
          maturin build --release --features python-bindings
        env:
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY: 1

      - name: Upload wheel
        uses: actions/upload-artifact@v3
        with:
          name: wheels
          path: target/wheels/*.whl

  publish:
    needs: build-wheels
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v3
        with:
          name: wheels
          path: wheels

      - name: Publish to PyPI
        uses: pypa/gh-action-pypi-publish@release/v1
        with:
          packages-dir: wheels/
          password: ${{ secrets.PYPI_API_TOKEN }}
```

---

### 7. Async Python API

**File:** `src/python_async.rs` (new file)

```rust
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use tokio::runtime::Runtime;

#[pyclass]
pub struct AsyncGuestfs {
    handle: crate::guestfs::Guestfs,
    runtime: Runtime,
}

#[pymethods]
impl AsyncGuestfs {
    #[new]
    fn new() -> PyResult<Self> {
        let handle = crate::guestfs::Guestfs::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let runtime = Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        Ok(Self { handle, runtime })
    }

    fn add_drive_ro<'py>(&self, py: Python<'py>, filename: String) -> PyResult<&'py PyAny> {
        let handle = self.handle.clone();
        future_into_py(py, async move {
            tokio::task::spawn_blocking(move || {
                handle.add_drive_ro(&filename)
            }).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn launch<'py>(&self, py: Python<'py>) -> PyResult<&'py PyAny> {
        let handle = self.handle.clone();
        future_into_py(py, async move {
            tokio::task::spawn_blocking(move || {
                handle.launch()
            }).await
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    // ... other async methods
}

// In lib.rs or python.rs module registration
#[pymodule]
fn guestctl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Guestfs>()?;
    m.add_class::<AsyncGuestfs>()?;  // Add async class
    m.add_class::<DiskConverter>()?;
    Ok(())
}
```

**Add to Cargo.toml:**
```toml
[dependencies]
pyo3-asyncio = { version = "0.20", features = ["tokio-runtime"] }
```

**Usage:**
```python
import asyncio
from guestctl import AsyncGuestfs

async def inspect_vm(disk_path):
    async with AsyncGuestfs() as g:
        await g.add_drive_ro(disk_path)
        await g.launch()
        roots = await g.inspect_os()
        return roots

# Run multiple in parallel
results = await asyncio.gather(
    inspect_vm("disk1.img"),
    inspect_vm("disk2.img"),
    inspect_vm("disk3.img")
)
```

---

## 📊 Testing Your Enhancements

### Quick Test Script

```bash
#!/bin/bash
# test_enhancements.sh

echo "Testing Python enhancements..."

# Test context manager
python3 << 'EOF'
from guestctl import Guestfs

with Guestfs() as g:
    print("✓ Context manager works")
EOF

# Test type hints (requires mypy)
echo "
from guestctl import Guestfs
g = Guestfs()
g.add_drive_ro('/path/to/disk.img')
" > test_types.py

if command -v mypy &> /dev/null; then
    mypy test_types.py && echo "✓ Type hints work"
fi

# Test completion
./target/release/guestctl completion bash > /tmp/completion.bash
[ -s /tmp/completion.bash ] && echo "✓ Shell completion works"

echo "All tests passed!"
```

---

## 📦 Quick Deployment Checklist

- [ ] Context manager implemented
- [ ] Type hints added
- [ ] Shell completion working
- [ ] Progress bars shown
- [ ] Colors applied
- [ ] Tests passing
- [ ] Documentation updated
- [ ] Examples updated
- [ ] Changelog updated
- [ ] Version bumped
- [ ] Git tagged
- [ ] Released on GitHub
- [ ] Published to PyPI
- [ ] Announced on social media

---

## 🎉 After Implementation

### Announce the Enhancements

**GitHub Release Notes:**
```markdown
# v0.4.0 - Enhanced Python API

## New Features
✨ Python context manager support - cleaner code!
📝 Full type hints for better IDE support
🐚 Shell completion for bash, zsh, fish
🎨 Colorful CLI output
⏳ Progress bars for long operations

## Installation
pip install --upgrade guestctl

## Breaking Changes
None - fully backward compatible!

## Examples
[Show code examples]
```

**Social Media:**
- Tweet the release
- Post on Reddit (r/rust, r/python)
- Blog post
- LinkedIn update

---

## Resources

- [PyO3 Guide](https://pyo3.rs/)
- [Clap Documentation](https://docs.rs/clap/)
- [Indicatif Documentation](https://docs.rs/indicatif/)
- [Python Type Hints PEP 484](https://peps.python.org/pep-0484/)

---

**Start with enhancement #1 (context manager) - it takes only 1 hour and provides immediate value!**
