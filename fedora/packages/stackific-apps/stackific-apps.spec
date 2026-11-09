# Stackific OS apps offered outside the image, for Fedora, as arch/packages/stackific-apps does on Arch: the "Install
# creative apps" launcher and its script, which downloads the creative apps from upstream's signed GitHub releases on
# the user's machine (task 025; we never redistribute them). The sources are base-neutral files from apps/.

Name: stackific-apps
Version: 0.1.0
Release: 1%{?dist}
Summary: Stackific OS: one-click install of the creative apps from their upstream releases
License: Apache-2.0
URL: https://os.stackific.com
BuildArch: noarch
Source0: install-creative-apps
Source1: creative-apps.sha256
Source2: creative-apps-key.asc
Source3: stackific-creative-apps.desktop
Requires: bash
Requires: coreutils
Requires: curl
Requires: dnf5
Requires: foot
Requires: gnupg2
Requires: grep
Requires: systemd

%description
The "Install creative apps" launcher and its script, which downloads the creative apps at pinned versions from their
upstream GitHub releases, checks each package's signature and SHA-256, and installs them with dnf.

%install
install -Dm755 %{SOURCE0} %{buildroot}%{_prefix}/lib/stackific/install-creative-apps
install -Dm644 -t %{buildroot}%{_datadir}/stackific %{SOURCE1} %{SOURCE2}
install -Dm644 -t %{buildroot}%{_datadir}/applications %{SOURCE3}

%files
%{_prefix}/lib/stackific/install-creative-apps
%{_datadir}/stackific/creative-apps.sha256
%{_datadir}/stackific/creative-apps-key.asc
%{_datadir}/applications/stackific-creative-apps.desktop
