Name:           zyvor-vm-tools
Version:        0.1.0
Release:        1%{?dist}
Summary:        Zeus VM Tools — Zyvor in-guest agent for KubeVirt VMs

License:        Apache-2.0
URL:            https://zyvor.dev/guestkit
BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  musl-gcc

Requires:       systemd

%description
Zyvor VM Tools (Zeus Guest Tools) provides an in-guest agent for live VM
assurance, migration health, and cluster-native guest operations on KubeVirt.

%prep
%autosetup -n guestkit-%{version} -p1

%build
export CARGO_TARGET_DIR=target
rustup target add x86_64-unknown-linux-musl 2>/dev/null || true
cargo build --release --features agent --no-default-features \
  --target x86_64-unknown-linux-musl --bin zyvor-guest-agent

%install
install -Dm755 target/x86_64-unknown-linux-musl/release/zyvor-guest-agent \
  %{buildroot}%{_bindir}/zyvor-guest-agent
install -Dm644 templates/agent/zyvor-guest-agent.service \
  %{buildroot}%{_unitdir}/zyvor-guest-agent.service

%post
%systemd_post zyvor-guest-agent.service

%preun
%systemd_preun zyvor-guest-agent.service

%postun
%systemd_postun_with_restart zyvor-guest-agent.service

%files
%{_bindir}/zyvor-guest-agent
%{_unitdir}/zyvor-guest-agent.service

%changelog
* Fri Jun 12 2026 ZyvorAI Labs <info@zyvor.dev> - 0.1.0-1
- Initial Zeus VM Tools Linux package
