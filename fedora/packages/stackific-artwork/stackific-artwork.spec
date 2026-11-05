# Stackific OS artwork for Fedora, the same files at the same paths as arch/packages/stackific-artwork/PKGBUILD: the
# wallpapers with their credits, and fastfetch's logo. The sources are base-neutral files from branding/, which
# `just fedora packages` stages as this spec's sources by name. The paintings are public domain (nine CC0, one without
# a named licence); wallpapers.toml credits each one and serves as their licence file.

Name: stackific-artwork
Version: 0.1.0
Release: 1%{?dist}
Summary: Stackific OS artwork: the wallpapers and the fastfetch logo
License: Apache-2.0 AND CC0-1.0 AND LicenseRef-PublicDomain
URL: https://os.stackific.com
BuildArch: noarch
Source0: branding.env
Source1: fastfetch.jsonc.in
Source2: logo.txt
Source3: wallpapers.toml
Source4: an-eruption-of-vesuvius-johan-christian-dahl.jpg
Source5: festoon-of-fruits-johannes-borman.jpg
Source6: fog-effect-camille-pissarro.jpg
Source7: pines-along-the-shore-henri-edmond-cross.jpg
Source8: still-life-with-a-gilt-cup-willem-claesz-heda.jpg
Source9: sunset-after-a-storm-andreas-achenbach.jpg
Source10: the-north-cape-by-moonlight-peder-balke.jpg
Source11: twilight-in-the-wilderness-frederic-edwin-church.jpg
Source12: view-from-mount-holyoke-thomas-cole.jpg
Source13: wheat-field-with-cypresses-vincent-van-gogh.jpg
BuildRequires: gettext-envsubst
Requires: fastfetch

%description
The Stackific OS wallpapers, ten public-domain paintings with their credits, and a system-wide fastfetch config that
shows the Stackific OS logo in the accent colour.

%build
# Render fastfetch's config with the accent from branding.env; copy the credits here for the license tag
set -a
. %{SOURCE0}
set +a
envsubst '$ACCENT' <%{SOURCE1} >fastfetch.jsonc
cp %{SOURCE3} .

%install
# The sources directory holds only this package's sources, so its JPEGs are the ten wallpapers
install -Dm644 -t %{buildroot}%{_datadir}/backgrounds/stackific %{_sourcedir}/*.jpg
install -Dm644 -t %{buildroot}%{_datadir}/stackific %{SOURCE2}
install -Dm644 fastfetch.jsonc %{buildroot}%{_sysconfdir}/xdg/fastfetch/config.jsonc

%files
%license wallpapers.toml
%dir %{_datadir}/backgrounds/stackific
%{_datadir}/backgrounds/stackific/*.jpg
%dir %{_datadir}/stackific
%{_datadir}/stackific/logo.txt
%dir %{_sysconfdir}/xdg/fastfetch
%config(noreplace) %{_sysconfdir}/xdg/fastfetch/config.jsonc
