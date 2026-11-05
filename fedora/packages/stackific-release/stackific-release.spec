# Stackific OS release files for Fedora: os-release and the logo icon. The sources are base-neutral files from branding/,
# which `just fedora packages` stages as this spec's sources by name. The build section renders them from branding.env, the one
# branding file, with the same lines as arch/packages/stackific-release/PKGBUILD; the per-base fields are appended here.
# It replaces only generic-release, which owns just /usr/lib/os-release. generic-release-common stays: it holds Fedora's
# rpm macros (fedora and dist), dnf5's defaults (pkg_gpgcheck=True among them) and the /etc symlinks, among them
# /etc/os-release, which points at our file. The provides keep what generic-release's dependents need: fedora-repos
# requires system-release(45), and generic-release-common requires generic-release-variant. Without
# system-release(releasever), dnf5 would take $releasever from this package's own version, and every Fedora repo URL
# would break. rpm expands macros in spec sections and comments, so the shell printf doubles each percent sign.

Name: stackific-release
Version: 0.1.0
Release: 1%{?dist}
Summary: Stackific OS release files: os-release and the logo icon
License: Apache-2.0
URL: https://os.stackific.com
BuildArch: noarch
Source0: branding.env
Source1: os-release.in
Source2: logo.svg
BuildRequires: gettext-envsubst
Requires: generic-release-common
Requires: distribution-systemd-presets
Requires: hicolor-icon-theme
Provides: system-release
Provides: system-release(45)
Provides: system-release(releasever) = 45
# shortcut: matches generic-release-common 45-0.4's exact Requires; bump it when Fedora updates generic-release
Provides: generic-release-variant = 45-0.4
Conflicts: generic-release

%description
The os-release that identifies the system as Stackific OS on its Fedora 45 base, and the logo icon its LOGO names.

%build
# Render os-release (ANSI_COLOR is the accent as a 24-bit terminal colour) with Fedora's per-base fields, and the
# accent-filled icon
set -a
. %{SOURCE0}
set +a
printf -v ANSI_COLOR '38;2;%%d;%%d;%%d' "0x${ACCENT:1:2}" "0x${ACCENT:3:2}" "0x${ACCENT:5:2}"
export ANSI_COLOR
envsubst '$NAME $ID $HOME_URL $DOCUMENTATION_URL $SUPPORT_URL $BUG_REPORT_URL $ANSI_COLOR' <%{SOURCE1} >os-release
printf 'ID_LIKE=fedora\nVERSION_ID=45\nSUPPORT_END=2027-12-08\n' >>os-release
sed "s/currentColor/$ACCENT/" %{SOURCE2} >logo-color.svg

%install
# os-release goes where Fedora keeps it (generic-release-common's /etc/os-release links to it), the icon to hicolor's
# scalable `stackific`
install -Dm644 os-release %{buildroot}%{_prefix}/lib/os-release
install -Dm644 logo-color.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/stackific.svg

%files
%{_prefix}/lib/os-release
%{_datadir}/icons/hicolor/scalable/apps/stackific.svg
