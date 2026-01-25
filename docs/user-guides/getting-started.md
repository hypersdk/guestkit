# guestctl Quick Start Guide

## Project Overview

**guestctl** is a Rust library and CLI tool for guest VM disk manipulation, designed to work with [hyper2kvm](https://github.com/ssahani/hyper2kvm).

### Location
```
~/tt/guestctl/
```

## Building

```bash
cd ~/tt/guestctl

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
guestctl = { path = "~/tt/guestctl" }
```

### Example Code

```rust
use guestctl::converters::DiskConverter;
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

## Integration with hyper2kvm

To use guestctl in hyper2kvm:

1. **Update hyper2kvm to use guestctl for disk operations**
2. **Replace Python qemu-img calls with guestctl Rust calls**
3. **Benefit from memory safety and performance**

Example integration:

```python
# In hyper2kvm
import subprocess

# Call guestctl from Python
result = subprocess.run([
    "guestctl", "convert",
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
    // Call guestctl converter
    Ok(())
}
```

## Development

### Project Structure

```
guestctl/
├── Cargo.toml          # Project configuration
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # CLI entry point
│   ├── core/           # Core utilities
│   ├── converters/     # Disk converters
│   ├── orchestrator/   # Pipeline
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

LGPL-3.0-or-later
