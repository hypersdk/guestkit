# guestkit Quick Start Guide

## Project Overview

**guestkit** is a pure Rust library and CLI for offline VM intelligence and **migration assurance**. Features include:

- 🩺 **Doctor / migrate-plan** - Boot probability and hypervisor-aware migration scoring before cutover
- 🖥️ **TUI Assurance** - Same scoring in `guestctl tui` (Security group · `d`/`t`/`p`/`e` keys)
- 🎯 **Killer Summary View** - See OS, version, architecture at a glance
- 🪟 **Windows Registry Parsing** - Full Windows version detection (incl. `windows-migration` profile)
- 🔄 **VM Migration Support** - Universal fstab/crypttab rewriter + fix plans
- 💾 **Smart LVM Cleanup** - Automatic volume group management
- 🔄 **Loop Device Primary** - Built-in support for RAW/IMG/ISO

Designed to work seamlessly with [hyper2kvm](https://github.com/ssahani/hyper2kvm) and VM migration workflows.

## Building

```bash
cd ~/tt/guestkit

# Build the project
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test
```

## Using the CLI

```bash
# Build and run
cargo run -- --help

# Convert VMDK to qcow2
cargo run -- convert \
  --source /path/to/vm.vmdk \
  --output /path/to/vm.qcow2 \
  --format qcow2 \
  --compress

# Detect disk format
cargo run -- detect --image /path/to/disk.img

# Get disk information
cargo run -- info --image /path/to/disk.img

# Verbose logging
cargo run -- -v convert --source vm.vmdk --output vm.qcow2
```

## Using as a Library

### In Your Cargo.toml

```toml
[dependencies]
guestkit = { path = "~/tt/guestkit" }
```

### Example Code

```rust
use guestkit::converters::DiskConverter;
use std::path::Path;

fn main() -> anyhow::Result<()> {
    let converter = DiskConverter::new();

    let result = converter.convert(
        Path::new("/path/to/source.vmdk"),
        Path::new("/path/to/output.qcow2"),
        "qcow2",
        true,  // compress
        true,  // flatten
    )?;

    if result.success {
        println!("✓ Conversion successful!");
        println!("  Source:  {} ({})",
            result.source_path.display(),
            result.source_format.as_str()
        );
        println!("  Output:  {} ({})",
            result.output_path.display(),
            result.output_format.as_str()
        );
        println!("  Size:    {} bytes", result.output_size);
        println!("  Time:    {:.2}s", result.duration_secs);
    }

    Ok(())
}
```

## Running Examples

```bash
# Convert disk
cargo run --example convert_disk

# Detect format
cargo run --example detect_format

# Retry example
cargo run --example retry_example
```

## TUI (interactive dashboard)

```bash
# Carbon-themed multi-view inspector
guestctl tui vm.qcow2

# Fleet of images
guestctl tui vm.qcow2 --fleet ./images/

# Compare second disk on dashboard
guestctl tui vm.qcow2 --compare other.qcow2
```

**Assurance** (Security group): offline `doctor` + `migrate-plan` parity with CLI — `d` run doctor, `t` cycle target (kvm/proxmox/aws), `p` preview fix plan, `e` export YAML. Dashboard **`a`** jumps to Assurance.

See [TUI enhancements](../features/tui-enhancements.md) and [migration assurance](../features/migration-assurance.md).

## Integration with hyper2kvm

To use guestkit in hyper2kvm:

1. **Update hyper2kvm to use guestkit for disk operations**
2. **Replace Python qemu-img calls with guestkit Rust calls**
3. **Benefit from memory safety and performance**

Example integration:

```python
# In hyper2kvm
import subprocess

# Call guestkit from Python
result = subprocess.run([
    "guestkit", "convert",
    "--source", source_path,
    "--output", output_path,
    "--compress"
], capture_output=True, text=True)
```

Or use PyO3 to create Python bindings:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn convert_disk(source: String, output: String) -> PyResult<()> {
    // Call guestkit converter
    Ok(())
}
```

## Development

### Project Structure

```
guestkit/
├── Cargo.toml          # Project configuration
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # CLI entry point
│   ├── core/           # Core utilities
│   ├── converters/     # Disk converters
│   └── ...
├── examples/           # Example programs
└── tests/              # Tests
```

### Adding New Features

1. **Create new module** in `src/`
2. **Export in lib.rs**
3. **Add tests**
4. **Update documentation**

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_disk_format_conversion

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

## Next Steps

1. **Implement guest OS detection** ( FFI)
2. **Add async disk operations**
3. **Create Python bindings** (PyO3)
4. **Integrate with hyper2kvm**
5. **Add more examples**

## Troubleshooting

### Build Errors

```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean && cargo build
```

### Missing qemu-img

```bash
# Fedora/RHEL
sudo dnf install qemu-img

# Ubuntu/Debian
sudo apt install qemu-utils
```

## Resources

- **README.md** - Comprehensive project documentation
- **examples/** - Working code examples
- **Cargo.toml** - Dependencies and configuration
- **hyper2kvm** - Primary integration target

## License

Apache-2.0
