# Stackific OS snapshots and swap for Fedora (task 026), as arch/packages/stackific-storage does on Arch: snapper with
# btrfs-assistant, snapper's hourly cleanup, and swap in zram with zswap off. Fedora has no snap-pac: snapper.actions,
# next to this spec, gives dnf5's actions plugin the pre and post snapshots around every transaction. snapper's root
# config is the installer's (installer/src/snapper.rs). zram's drop-in replaces Fedora's zram-generator-defaults, which
# the image leaves out.

Name: stackific-storage
Version: 0.1.0
Release: 1%{?dist}
Summary: Stackific OS snapshots and swap: snapper, btrfs-assistant and zram
License: Apache-2.0
URL: https://os.stackific.com
BuildArch: noarch
Source0: stackific-zram.conf
Source1: 99-stackific-swap.conf
Source2: stackific-zswap.conf
Source3: snapper.actions
Requires: snapper
Requires: btrfs-assistant
Requires: zram-generator
Requires: libdnf5-plugin-actions

%description
Pre and post snapper snapshots around every dnf5 transaction, snapper's hourly cleanup, btrfs-assistant to restore
snapshots, and swap in zram, as large as the RAM, with zswap off.

%install
install -Dm644 %{SOURCE0} %{buildroot}%{_prefix}/lib/systemd/zram-generator.conf.d/stackific.conf
install -Dm644 %{SOURCE1} %{buildroot}%{_prefix}/lib/sysctl.d/99-stackific-swap.conf
install -Dm644 %{SOURCE2} %{buildroot}%{_prefix}/lib/tmpfiles.d/stackific-zswap.conf
install -Dm644 %{SOURCE3} %{buildroot}%{_sysconfdir}/dnf/libdnf5-plugins/actions.d/snapper.actions
install -d %{buildroot}%{_prefix}/lib/systemd/system/timers.target.wants
ln -s ../snapper-cleanup.timer %{buildroot}%{_prefix}/lib/systemd/system/timers.target.wants/snapper-cleanup.timer

%files
%{_prefix}/lib/systemd/zram-generator.conf.d/stackific.conf
%{_prefix}/lib/sysctl.d/99-stackific-swap.conf
%{_prefix}/lib/tmpfiles.d/stackific-zswap.conf
%config(noreplace) %{_sysconfdir}/dnf/libdnf5-plugins/actions.d/snapper.actions
%{_prefix}/lib/systemd/system/timers.target.wants/snapper-cleanup.timer
