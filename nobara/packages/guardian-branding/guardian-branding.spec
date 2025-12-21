Name:           guardian-branding
Version:        1.1.0
Release:        1%{?dist}
Summary:        Guardian OS Branding and Theming
License:        GPL-3.0-or-later
URL:            https://gameguardian.ai
BuildArch:      noarch

Requires:       calamares
Requires:       plymouth
Provides:       system-release
Provides:       system-release(%{version})

%description
Guardian OS branding package including:
- Calamares installer branding and Guardian auth modules
- Plymouth boot splash theme
- Desktop wallpapers
- OS release information

%prep
# Copy branding files from source
cd %{_builddir}
cp -r %{_sourcedir}/branding/* .
cp -r %{_sourcedir}/calamares-modules/* .

%install
# Calamares branding
install -d %{buildroot}%{_datadir}/calamares/branding/guardian
cp -r calamares/* %{buildroot}%{_datadir}/calamares/branding/guardian/

# Calamares modules
install -d %{buildroot}%{_libdir}/calamares/modules
cp -r guardianauth %{buildroot}%{_libdir}/calamares/modules/
cp -r guardianchild %{buildroot}%{_libdir}/calamares/modules/

# Calamares settings (override default)
install -D -m 644 settings.conf %{buildroot}%{_sysconfdir}/calamares/settings.conf

# Plymouth theme
install -d %{buildroot}%{_datadir}/plymouth/themes/guardian
# Plymouth files would be copied here

# Wallpapers
install -d %{buildroot}%{_datadir}/backgrounds/guardian
# Wallpaper files would be copied here

# OS release
install -D -m 644 /dev/stdin %{buildroot}%{_sysconfdir}/os-release << 'EOF'
NAME="Guardian OS"
VERSION="1.1.0 (Nobara Edition)"
ID=guardian
ID_LIKE=fedora nobara
VERSION_ID="1.1.0"
VERSION_CODENAME="Safe Gaming"
PRETTY_NAME="Guardian OS 1.1.0"
ANSI_COLOR="0;38;2;99;102;241"
LOGO=guardian-logo
CPE_NAME="cpe:/o:gameguardian:guardian_os:1.1.0"
HOME_URL="https://gameguardian.ai"
DOCUMENTATION_URL="https://gameguardian.ai/docs"
SUPPORT_URL="https://gameguardian.ai/support"
BUG_REPORT_URL="https://github.com/jonnyweareone/guardian-os-v1/issues"
PRIVACY_POLICY_URL="https://gameguardian.ai/privacy"
VARIANT="Nobara GNOME"
VARIANT_ID=gnome
EOF

# Also create /usr/lib/os-release (canonical location)
install -d %{buildroot}%{_prefix}/lib
ln -sf ../..%{_sysconfdir}/os-release %{buildroot}%{_prefix}/lib/os-release

%files
%{_datadir}/calamares/branding/guardian/
%{_libdir}/calamares/modules/guardianauth/
%{_libdir}/calamares/modules/guardianchild/
%config(noreplace) %{_sysconfdir}/calamares/settings.conf
%{_datadir}/plymouth/themes/guardian/
%{_datadir}/backgrounds/guardian/
%config(noreplace) %{_sysconfdir}/os-release
%{_prefix}/lib/os-release

%changelog
* Sun Dec 22 2024 Guardian OS Team <support@gameguardian.ai> - 1.1.0-1
- Initial Nobara/Fedora branding package
- Calamares Guardian auth and child selection modules
- Guardian installer slideshow
- OS release branding
