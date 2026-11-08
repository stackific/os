# Stackific OS desktop defaults for Fedora, the same files at the same paths as arch/packages/stackific-desktop/PKGBUILD.
# The sources are base-neutral files from desktop/ and branding/, which `just fedora packages` stages as this spec's
# sources by name. Noctalia reads no system-wide config, so its defaults reach each user through an include file in
# /etc/skel. noctalia-greeter reads /var/lib/noctalia-greeter/greeter.toml, which a tmpfiles line links to our defaults,
# and takes its logo from the asset folder that stackific-greeter.sh names: ours, with stackific-release's icon.
# GNOME, the second desktop on Fedora, takes our wallpaper and dash from a gschema override, which sits next to this
# spec because it is Fedora-only (Plasma's counterpart is in the Arch package). mangowm comes from Terra at install
# time; nothing here needs it to build. The Secret Service in every session (task 017): oo7-daemon starts with each
# user's systemd manager, and greetd's own PAM file already calls pam_oo7, which unlocks its keyring at login.

Name: stackific-desktop
Version: 0.2.0
Release: 4%{?dist}
Summary: Stackific OS desktop defaults: Noctalia starts in MangoWM sessions with our defaults, the greeter shows our look
License: Apache-2.0
URL: https://os.stackific.com
BuildArch: noarch
Source0: stackific-noctalia.service
Source1: stackific-wlroots.sh
Source2: stackific-noctalia.toml
Source3: 00-stackific.toml
Source4: branding.env
Source5: stackific-greeter.toml.in
Source6: stackific-greeter.conf
Source7: stackific-greeter.sh
Source8: 20_stackific.gschema.override
BuildRequires: gettext-envsubst
# The tmpfiles directory macro (rpm expands macros even in comments, so it is not named here)
BuildRequires: systemd-rpm-macros
Requires: mangowm
Requires: noctalia
Requires: noctalia-greeter
Requires: oo7-daemon
Requires: pam_oo7
Requires: systemd
Requires: stackific-artwork
Requires: stackific-release

%description
Starts Noctalia in MangoWM sessions through a user unit wanted by mango-session.target, lets wlroots compositors start
on software rendering, and ships Noctalia's Stackific OS defaults with the per-user file that includes them, and
noctalia-greeter's defaults (our logo, palette and wallpaper), GNOME's wallpaper and dash, and starts oo7, the Secret
Service, for every user.

%build
# Render the greeter's defaults with the accent from branding.env
set -a
. %{SOURCE4}
set +a
envsubst '$ACCENT' <%{SOURCE5} >stackific-greeter.toml

%install
# The unit, wanted by mango-session.target for every user, the profile.d script, Noctalia's defaults with the per-user
# file that includes them, the greeter's defaults with the tmpfiles line that links them, and the greeter's asset
# folder: our logo (stackific-release's icon) and the greeter's own fonts, both as links
install -Dm644 -t %{buildroot}%{_prefix}/lib/systemd/user %{SOURCE0}
install -d %{buildroot}%{_prefix}/lib/systemd/user/mango-session.target.wants
ln -s ../stackific-noctalia.service %{buildroot}%{_prefix}/lib/systemd/user/mango-session.target.wants/stackific-noctalia.service
install -Dm644 -t %{buildroot}%{_sysconfdir}/profile.d %{SOURCE1}
install -Dm644 %{SOURCE2} %{buildroot}%{_datadir}/stackific/noctalia.toml
install -Dm644 -t %{buildroot}%{_sysconfdir}/skel/.config/noctalia %{SOURCE3}
install -Dm644 stackific-greeter.toml %{buildroot}%{_datadir}/stackific/greeter.toml
install -Dm644 -t %{buildroot}%{_tmpfilesdir} %{SOURCE6}
install -Dm644 -t %{buildroot}%{_sysconfdir}/profile.d %{SOURCE7}
install -d %{buildroot}%{_datadir}/stackific/greeter-assets
ln -s %{_datadir}/icons/hicolor/scalable/apps/stackific.svg %{buildroot}%{_datadir}/stackific/greeter-assets/noctalia.svg
ln -s %{_datadir}/noctalia-greeter/assets/fonts %{buildroot}%{_datadir}/stackific/greeter-assets/fonts
# GNOME's defaults; glib2's file trigger compiles the schemas
install -Dm644 -t %{buildroot}%{_datadir}/glib-2.0/schemas %{SOURCE8}
# The Secret Service: oo7-daemon wanted by every user's manager (Fedora's own preset is not in our images)
install -d %{buildroot}%{_prefix}/lib/systemd/user/default.target.wants
ln -s ../oo7-daemon.service %{buildroot}%{_prefix}/lib/systemd/user/default.target.wants/oo7-daemon.service

%files
%{_prefix}/lib/systemd/user/stackific-noctalia.service
%dir %{_prefix}/lib/systemd/user/mango-session.target.wants
%{_prefix}/lib/systemd/user/mango-session.target.wants/stackific-noctalia.service
%config(noreplace) %{_sysconfdir}/profile.d/stackific-wlroots.sh
%config(noreplace) %{_sysconfdir}/profile.d/stackific-greeter.sh
%dir %{_datadir}/stackific
%{_datadir}/stackific/noctalia.toml
%{_datadir}/stackific/greeter.toml
%{_tmpfilesdir}/stackific-greeter.conf
%dir %{_datadir}/stackific/greeter-assets
%{_datadir}/stackific/greeter-assets/noctalia.svg
%{_datadir}/stackific/greeter-assets/fonts
%dir %{_sysconfdir}/skel/.config
%dir %{_sysconfdir}/skel/.config/noctalia
%config(noreplace) %{_sysconfdir}/skel/.config/noctalia/00-stackific.toml
%{_datadir}/glib-2.0/schemas/20_stackific.gschema.override
%{_prefix}/lib/systemd/user/default.target.wants/oo7-daemon.service
