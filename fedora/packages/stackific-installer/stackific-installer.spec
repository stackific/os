# Stackific OS installer for Fedora, the same files at the same paths as arch/packages/stackific-installer/PKGBUILD: the
# Rust program from installer/, its launcher (rendered from branding.env) and the console path's target and service.
# `just fedora packages` stages the launcher template, the units and branding.env by name, and packs the crate
# (Cargo.toml, Cargo.lock, src/, fixtures/) into stackific-installer-crate.tar. It builds this spec alone with mock's
# network on, for `cargo fetch` in %%prep, as the PKGBUILD's prepare() fetches; %%build and %%check run offline.
# The installer removes this package (and the dependencies only it needed) from the system it installs; limine and
# efibootmgr stay, because stackific-boot needs them.

Name: stackific-installer
Version: 0.1.0
Release: 2%{?dist}
Summary: Stackific OS installer: into a drive's free space or onto a whole drive, from the live desktop or the boot menu
License: Apache-2.0
URL: https://os.stackific.com
ExclusiveArch: x86_64 aarch64
Source0: stackific-installer-crate.tar
Source1: branding.env
Source2: stackific-installer.desktop.in
Source3: stackific-installer.target
Source4: stackific-installer-console.service
BuildRequires: cargo
BuildRequires: gettext-envsubst
BuildRequires: rust
BuildRequires: systemd-rpm-macros
Requires: btrfs-progs
Requires: cryptsetup
Requires: dnf5
Requires: dosfstools
Requires: e2fsprogs
Requires: efibootmgr
Requires: foot
Requires: kbd
Requires: limine
Requires: policycoreutils
Requires: rsync
Requires: shadow-utils
Requires: stackific-boot
Requires: stackific-release
Requires: systemd
Requires: util-linux
Requires: xkeyboard-config

# The release build carries no debug info to split off, as the PKGBUILD's !debug
%global debug_package %{nil}

%description
Installs Stackific OS from the live image into a drive's largest free space or onto a whole drive: our own EFI System
Partition with Limine, btrfs subvolumes, the system copied from the live image, the keyboard, timezone, computer name
and the first user. A text interface, or unattended from a TOML file.

%prep
%setup -q -c -T
tar -xf %{SOURCE0}
cargo fetch --locked

# Build the installer, and render the launcher with the name and icon from branding.env
%build
CARGO_TARGET_DIR=target cargo build --frozen --release
set -a
. %{SOURCE1}
set +a
envsubst '$NAME $ID' <%{SOURCE2} >stackific-installer.desktop

# The unit, navigation and snapshot tests
%check
CARGO_TARGET_DIR=target cargo test --frozen

%install
install -Dm755 target/release/stackific-installer %{buildroot}%{_bindir}/stackific-installer
install -Dm644 -t %{buildroot}%{_datadir}/applications stackific-installer.desktop
install -Dm644 -t %{buildroot}%{_unitdir} %{SOURCE3} %{SOURCE4}

%files
%{_bindir}/stackific-installer
%{_datadir}/applications/stackific-installer.desktop
%{_unitdir}/stackific-installer.target
%{_unitdir}/stackific-installer-console.service
