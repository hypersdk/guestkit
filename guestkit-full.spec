%global debug_package %{nil}

Name:           guestkit
Version:        0.3.1
Release:        1%{?dist}
Summary:        Pure-Rust VM disk inspection and manipulation toolkit

License:        LGPL-3.0-or-later
URL:            https://github.com/ssahani/guestkit
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

# Rust/Cargo requirements
BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  make

# System libraries
BuildRequires:  pkgconfig
BuildRequires:  systemd-devel

# Python bindings (optional)
%bcond_without python
%if %{with python}
BuildRequires:  python3-devel
BuildRequires:  python3-setuptools
BuildRequires:  python3-pip
BuildRequires:  python3-maturin
%endif

# Runtime dependencies
Requires:       qemu-img
Requires:       nbd
Requires:       util-linux

# Optional dependencies for full functionality
Recommends:     lvm2
Recommends:     mdadm
Recommends:     cryptsetup
Recommends:     device-mapper

# Architecture restrictions (Rust availability)
ExclusiveArch:  x86_64 aarch64 ppc64le s390x

%description
GuestKit is a production-ready toolkit for VM disk inspection and manipulation
with beautiful emoji-enhanced CLI output and an interactive TUI dashboard.
Built in pure Rust for safety and performance, it inspects VM disks in seconds
and integrates cleanly with hyper2kvm for migration workflows.

Features:
- Pure Rust implementation for memory safety and performance
- Interactive TUI dashboard with visual analytics and fuzzy navigation
- Multi-format support: QCOW2, VMDK, VDI, VHD/VHDX, RAW/IMG/ISO
- Security, compliance, hardening, and performance analysis profiles
- Export to JSON, YAML, HTML, PDF formats
- Python bindings for automation (PyO3)
- Interactive REPL shell with 20+ commands
- Batch processing with parallel inspection
- Zero-trust approach - read-only by default

%if %{with python}
%package -n python3-%{name}
Summary:        Python 3 bindings for %{name}
Requires:       %{name}%{?_isa} = %{version}-%{release}

%description -n python3-%{name}
Python 3 bindings for GuestKit, providing native Python API for VM disk
inspection and manipulation.
%endif

%package devel
Summary:        Development files for %{name}
Requires:       %{name}%{?_isa} = %{version}-%{release}

%description devel
Development files, examples, and documentation for %{name}.

%prep
%autosetup -n %{name}-%{version}

%build
# Set Rust optimization flags
export CARGO_TARGET_DIR=target
export RUSTFLAGS="%{?rustflags}"

# Build Rust binary with release profile
cargo build --release --locked --all-features

%if %{with python}
# Build Python bindings
export PYO3_PYTHON=%{__python3}
maturin build --release --strip
%endif

%install
# Install Rust binary
install -Dm755 target/release/guestkit %{buildroot}%{_bindir}/guestkit

%if %{with python}
# Install Python bindings
%{__python3} -m pip install --root %{buildroot} --prefix %{_prefix} --no-deps target/wheels/*.whl
%endif

# Install documentation
install -dm755 %{buildroot}%{_docdir}/%{name}
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 CHANGELOG.md %{buildroot}%{_docdir}/%{name}/CHANGELOG.md
install -Dm644 CONTRIBUTING.md %{buildroot}%{_docdir}/%{name}/CONTRIBUTING.md
install -Dm644 SECURITY.md %{buildroot}%{_docdir}/%{name}/SECURITY.md

# Install docs directory
cp -r docs %{buildroot}%{_docdir}/%{name}/

# Install examples
install -dm755 %{buildroot}%{_docdir}/%{name}/examples
cp -r examples/* %{buildroot}%{_docdir}/%{name}/examples/

# Install license
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE

# Install bash completion (if available)
# install -Dm644 completions/guestkit.bash %{buildroot}%{_datadir}/bash-completion/completions/guestkit

# Install zsh completion (if available)
# install -Dm644 completions/_guestkit %{buildroot}%{_datadir}/zsh/site-functions/_guestkit

# Install fish completion (if available)
# install -Dm644 completions/guestkit.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/guestkit.fish

%check
# Run basic binary check
%{buildroot}%{_bindir}/guestkit --version

# Run Rust tests (may require test fixtures)
# cargo test --release --locked

%if %{with python}
# Test Python bindings
# %{__python3} -c "import guestkit; print(guestkit.__version__)"
%endif

%files
%license LICENSE
%doc README.md CHANGELOG.md CONTRIBUTING.md SECURITY.md
%{_bindir}/guestkit
%dir %{_docdir}/%{name}
%{_docdir}/%{name}/README.md
%{_docdir}/%{name}/CHANGELOG.md
%{_docdir}/%{name}/CONTRIBUTING.md
%{_docdir}/%{name}/SECURITY.md
%{_docdir}/%{name}/docs/
# %{_datadir}/bash-completion/completions/guestkit
# %{_datadir}/zsh/site-functions/_guestkit
# %{_datadir}/fish/vendor_completions.d/guestkit.fish

%if %{with python}
%files -n python3-%{name}
%license LICENSE
%{python3_sitearch}/%{name}/
%{python3_sitearch}/%{name}-%{version}.dist-info/
%endif

%files devel
%{_docdir}/%{name}/examples/

%changelog
* Mon Jan 27 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.3.1-1
- Initial RPM package for Fedora/RHEL
- Interactive TUI dashboard with fuzzy jump navigation (Ctrl+P)
- Security, compliance, hardening, performance profiles
- Export to JSON, YAML, HTML, PDF formats
- Interactive shell with 20+ commands for disk exploration
- Python bindings via PyO3 (optional)
- Support for QCOW2, VMDK, VDI, VHD/VHDX, RAW, IMG, ISO formats
- LVM, RAID, fstab inspection
- Network, services, databases, web servers detection
- Batch processing with parallel inspection
- Caching system for performance
- Clean build with zero compiler warnings
- Reorganized documentation structure
- Removed libguestfs references (pure Rust implementation)

* Sun Jan 26 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.3.0-1
- Add comprehensive TUI dashboard with multiple views
- Add security analysis profiles (5 types)
- Add enhanced inspection APIs
- Improve documentation with examples
- Add export formats (JSON, YAML, HTML, PDF)

* Fri Jan 24 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.2.0-1
- Add Python bindings via PyO3
- Add batch processing support with parallelization
- Add inspection caching system
- Improve error handling and reporting
- Add retry mechanisms for operations

* Thu Jan 23 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.1.0-1
- Initial release
- Basic VM disk inspection functionality
- Support for multiple disk formats (QCOW2, VMDK, etc.)
- CLI interface with emoji-enhanced output
- Read-only operations by default
