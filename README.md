# guestkit

A Guest VM toolkit for disk inspection and manipulation, written in Rust and inspired by libguestfs. Designed to work seamlessly with [hyper2kvm](https://github.com/ssahani/hyper2kvm).

[![License: LGPL v3](https://img.shields.io/badge/License-LGPL_v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## Features

- 🔄 **Disk Format Conversion** - Convert between VMDK, qcow2, RAW, VHD, VDI using qemu-img
- 🔍 **Format Detection** - Automatic disk format detection
- ⚡ **Retry Logic** - Built-in exponential backoff for reliable operations
- 📊 **Pipeline Orchestration** - Multi-stage migration pipelines
- 🦀 **Pure Rust** - Memory safe, high performance
- 🔌 **Extensible** - Modular architecture for easy extension

## Quick Start

### Installation

```bash
# Install system dependencies (Fedora/RHEL)
sudo dnf install qemu-img

# From source
git clone https://github.com/ssahani/guestkit
cd guestkit
cargo build --release
sudo cargo install --path .
```

### Basic Usage

#### CLI

```bash
# Convert VMDK to qcow2
guestkit convert --source vm.vmdk --output vm.qcow2 --compress

# Detect disk format
guestkit detect --image disk.img

# Get detailed disk information
guestkit info --image disk.img
```

#### Library

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
        println!("Conversion successful!");
        println!("Output size: {} bytes", result.output_size);
    }

    Ok(())
}
```

## Project Structure

```
guestkit/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # CLI entry point
│   ├── core/               # Core utilities
│   │   ├── error.rs        # Error types
│   │   ├── retry.rs        # Retry logic
│   │   └── types.rs        # Common types
│   ├── converters/         # Disk format converters
│   │   └── disk_converter.rs
│   ├── detectors/          # Guest OS detection (planned)
│   ├── fixers/             # Guest OS fixers (planned)
│   ├── orchestrator/       # Pipeline orchestration
│   │   └── pipeline.rs
│   └── ffi/                # FFI bindings (optional)
├── examples/               # Example programs
│   ├── convert_disk.rs
│   ├── detect_format.rs
│   └── retry_example.rs
└── tests/                  # Integration tests

```

## Architecture

### Core Modules

#### `core` - Core Utilities
- **error.rs** - Error types using thiserror
- **retry.rs** - Exponential backoff retry logic
- **types.rs** - Common types (DiskFormat, GuestType, etc.)

#### `converters` - Disk Format Conversion
- **disk_converter.rs** - qemu-img wrapper for format conversion

#### `orchestrator` - Pipeline Orchestration
- **pipeline.rs** - Multi-stage migration pipeline

### Design Principles

1. **Memory Safety** - Leveraging Rust's ownership system
2. **Zero-cost Abstractions** - High-level APIs with no runtime overhead
3. **Modularity** - Clean separation of concerns
4. **Testability** - Comprehensive test coverage
5. **Extensibility** - Easy to add new features

## Examples

See the [`examples/`](examples/) directory for complete examples:

- **convert_disk.rs** - Convert disk image formats
- **detect_format.rs** - Detect and inspect disk images
- **retry_example.rs** - Using retry logic with exponential backoff

Run examples with:

```bash
cargo run --example convert_disk
cargo run --example detect_format
```

## Integration with hyper2kvm

guestkit is designed to work with [hyper2kvm](https://github.com/ssahani/hyper2kvm) for production VM migrations:

```rust
// Example: Using guestkit in a migration pipeline
use guestkit::converters::DiskConverter;

let converter = DiskConverter::new();
let result = converter.convert(
    vmdk_path,
    qcow2_path,
    "qcow2",
    true,  // compress
    true,  // flatten
)?;
```

## Dependencies

### System Dependencies

- **qemu-img** - Disk image manipulation (QEMU tools)
- **libguestfs** - Guest filesystem access (optional, for FFI bindings)

```bash
# Fedora/RHEL
sudo dnf install qemu-img libguestfs

# Ubuntu/Debian
sudo apt install qemu-utils libguestfs-tools

# Arch Linux
sudo pacman -S qemu libguestfs
```

### Rust Dependencies

See [`Cargo.toml`](Cargo.toml) for complete list:

- **anyhow** - Error handling
- **thiserror** - Custom error types
- **tokio** - Async runtime
- **serde** - Serialization
- **clap** - CLI argument parsing
- **log** - Logging framework

## Development

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- convert --source test.vmdk --output test.qcow2
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# With coverage
cargo tarpaulin --out Html
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Check documentation
cargo doc --no-deps --open
```

## Features

guestkit uses Cargo features for optional functionality:

- **`disk-ops`** (default) - Disk operation utilities
- **`guest-inspect`** - Guest OS inspection
- **`ffi-bindings`** - FFI bindings to libguestfs

```toml
[dependencies]
guestkit = { version = "0.1", features = ["guest-inspect"] }
```

## Roadmap

### Version 0.1 (Current)
- [x] Project structure
- [x] Core error types
- [x] Retry logic
- [x] Disk format conversion
- [x] CLI interface

### Version 0.2 (Planned)
- [ ] Guest OS detection
- [ ] FFI bindings to libguestfs
- [ ] Async disk operations
- [ ] Progress reporting

### Version 0.3 (Future)
- [ ] Guest OS fixing
- [ ] Network configuration
- [ ] Bootloader repair
- [ ] Full pipeline orchestration

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the **GNU Lesser General Public License v3.0 (LGPL-3.0)**.

See [LICENSE](LICENSE) for full license text.

## Acknowledgments

- **libguestfs** - Inspiration and design patterns
- **hyper2kvm** - Primary use case and integration target
- **QEMU** - Disk format conversion tools

## Support

- **GitHub Issues**: [Report bugs](https://github.com/ssahani/guestkit/issues)
- **Documentation**: [API docs](https://docs.rs/guestkit)
- **Examples**: See [`examples/`](examples/) directory

## Related Projects

- **[hyper2kvm](https://github.com/ssahani/hyper2kvm)** - Production-grade VM migration toolkit
- **[hypersdk](https://github.com/ssahani/hypersdk)** - High-performance hypervisor SDK
- **[libguestfs](https://libguestfs.org/)** - Guest filesystem inspection library

---

Made with ❤️ for reliable VM operations
