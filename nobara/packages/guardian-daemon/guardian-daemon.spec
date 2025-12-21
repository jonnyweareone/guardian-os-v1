Name:           guardian-daemon
Version:        1.1.0
Release:        1%{?dist}
Summary:        Guardian OS Safety Daemon
License:        GPL-3.0-or-later
URL:            https://gameguardian.ai

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  openssl-devel
BuildRequires:  sqlite-devel
BuildRequires:  dbus-devel
BuildRequires:  systemd-rpm-macros

Requires:       openssl
Requires:       sqlite
Requires:       dbus

%description
Core safety enforcement daemon for Guardian OS.
Monitors device activity, enforces parental controls,
manages content filtering, and syncs with the Guardian cloud.

%prep
# Source is in the guardian-os repo
cd %{_builddir}
cp -r %{_sourcedir}/guardian-components/guardian-daemon/* .

%build
cargo build --release

%install
install -D -m 755 target/release/guardian-daemon %{buildroot}%{_bindir}/guardian-daemon
install -D -m 644 %{_sourcedir}/guardian-daemon.service %{buildroot}%{_unitdir}/guardian-daemon.service
install -d -m 700 %{buildroot}%{_sysconfdir}/guardian

%post
%systemd_post guardian-daemon.service

%preun
%systemd_preun guardian-daemon.service

%postun
%systemd_postun_with_restart guardian-daemon.service

%files
%{_bindir}/guardian-daemon
%{_unitdir}/guardian-daemon.service
%dir %attr(700, root, root) %{_sysconfdir}/guardian

%changelog
* Sun Dec 21 2024 Guardian OS Team <support@gameguardian.ai> - 1.1.0-1
- Initial Nobara/Fedora package
- Core safety daemon with DNS filtering
- Activity monitoring and reporting
- Cloud sync support
