Name:           guestkit
Version:        0.3.6
Release:        1%{?dist}
Summary:        Pure-Rust VM disk inspection and manipulation toolkit

License:        Apache-2.0
URL:            https://github.com/hypersdk/guestkit
Source0:        %{name}-%{version}.tar.gz

# Build dependencies
BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  pkgconfig
BuildRequires:  systemd-devel

# Runtime dependencies
Requires:       qemu-img
Requires:       nbd
Requires:       util-linux

# Optional dependencies for full functionality
Recommends:     lvm2
Recommends:     mdadm
Recommends:     cryptsetup

%description
GuestKit is a production-ready toolkit for VM disk inspection and manipulation
with beautiful emoji-enhanced CLI output and an interactive TUI dashboard.
Built in pure Rust for safety and performance, it inspects VM disks in seconds
and integrates cleanly with hyper2kvm for migration workflows.

Features:
- Pure Rust implementation for memory safety and performance
- Interactive TUI dashboard with visual analytics
- Multi-format support: QCOW2, VMDK, VDI, VHD/VHDX, RAW/IMG/ISO
- Security profiles for compliance and hardening analysis
- Export to JSON, YAML, HTML, PDF formats
- Python bindings for automation
- Interactive REPL shell for disk exploration
- Batch processing with parallel inspection

%package devel
Summary:        Development files for %{name}
Requires:       %{name}%{?_isa} = %{version}-%{release}

%description devel
Development files and headers for %{name}.

%prep
%autosetup -n %{name}-%{version}

%build
# Build with release profile
export CARGO_TARGET_DIR=target
cargo build --release --locked

%install
# Install binary
install -Dm755 target/release/guestkit %{buildroot}%{_bindir}/guestkit

# Install documentation
install -Dm644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dm644 docs/development/CHANGELOG.md %{buildroot}%{_docdir}/%{name}/CHANGELOG.md
install -Dm644 docs/development/CONTRIBUTING.md %{buildroot}%{_docdir}/%{name}/CONTRIBUTING.md
install -Dm644 SECURITY.md %{buildroot}%{_docdir}/%{name}/SECURITY.md

# Install docs directory
cp -r docs %{buildroot}%{_docdir}/%{name}/

# Install examples
mkdir -p %{buildroot}%{_docdir}/%{name}/examples
cp -r examples/* %{buildroot}%{_docdir}/%{name}/examples/

# Install license
install -Dm644 LICENSE %{buildroot}%{_licensedir}/%{name}/LICENSE

# Install man page (if we create one)
# install -Dm644 doc/guestkit.1 %{buildroot}%{_mandir}/man1/guestkit.1

%check
# Run tests (currently skipped due to test requirements)
# cargo test --release --locked

%files
%license LICENSE
%doc README.md CHANGELOG.md CONTRIBUTING.md SECURITY.md
%{_bindir}/guestkit
%{_docdir}/%{name}/
%exclude %{_docdir}/%{name}/examples/

%files devel
%{_docdir}/%{name}/examples/

%changelog
* Wed Jun 04 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.3.6-1
- Add in-guest agent mode for live VM assurance over virtio-serial
- Sync RPM spec version with Cargo.toml

* Mon Jan 27 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.3.1-1
- Initial RPM package
- Add interactive TUI dashboard with fuzzy jump navigation
- Add security, compliance, hardening, performance profiles
- Add export to JSON, YAML, HTML, PDF
- Add interactive shell with 20+ commands
- Add Python bindings via PyO3
- Remove libguestfs references
- Reorganize documentation structure
- Fix compiler warnings

* Sun Jan 26 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.3.0-1
- Add comprehensive TUI dashboard
- Add security analysis profiles
- Add enhanced inspection APIs
- Improve documentation

* Fri Jan 24 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.2.0-1
- Add Python bindings
- Add batch processing support
- Add caching system
- Improve error handling

* Thu Jan 23 2026 ZyvorAI Labs Private Limited <ssahani@redhat.com> - 0.1.0-1
- Initial release
- Basic VM disk inspection
- Support for multiple disk formats
- CLI interface
