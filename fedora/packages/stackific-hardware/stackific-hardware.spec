# Stackific OS hardware support for Fedora: picks the NVIDIA driver at every boot, as arch/packages/stackific-hardware
# does on Arch. The generator (hardware/) reads the display controllers from sysfs and, where all NVIDIA graphics are
# older than Turing or the running kernel has no NVIDIA module, loads nouveau instead of NVIDIA's open module, and gives
# MangoWM lighter effects where an older card is the only GPU. Two Fedora-only files sit next to this spec: RPM
# Fusion blacklists nouveau only on GRUB's command line (grubby), which Limine never reads, so nvidia-utils.conf (the
# name of Arch's file, which the generator's /run file shadows on both bases) blacklists it and loads nvidia_drm with
# nvidia; and a dracut drop-in keeps nouveau out of every initramfs (RPM Fusion's own drop-in omits NVIDIA's modules).

Name: stackific-hardware
Version: 0.1.0
Release: 1%{?dist}
Summary: Stackific OS hardware support: picks NVIDIA open modules or nouveau at every boot
License: Apache-2.0
URL: https://os.stackific.com
BuildArch: noarch
Source0: stackific-gpu-generator
Source1: nvidia-legacy.ids
Source2: mango-low-effects.conf
Source3: nvidia-utils.conf
Source4: 99-stackific-gpu.conf
Requires: bash
Requires: systemd

%description
A systemd generator that picks NVIDIA's open kernel modules or nouveau at every boot from NVIDIA's legacy device list,
the modprobe and dracut settings it relies on, and MangoWM's lighter effects for older NVIDIA graphics.

%install
install -Dm755 %{SOURCE0} %{buildroot}%{_prefix}/lib/systemd/system-generators/stackific-gpu-generator
install -Dm644 -t %{buildroot}%{_datadir}/stackific %{SOURCE1} %{SOURCE2}
install -Dm644 %{SOURCE3} %{buildroot}%{_prefix}/lib/modprobe.d/nvidia-utils.conf
install -Dm644 %{SOURCE4} %{buildroot}%{_prefix}/lib/dracut/dracut.conf.d/99-stackific-gpu.conf

%files
%{_prefix}/lib/systemd/system-generators/stackific-gpu-generator
%{_datadir}/stackific/nvidia-legacy.ids
%{_datadir}/stackific/mango-low-effects.conf
%{_prefix}/lib/modprobe.d/nvidia-utils.conf
%{_prefix}/lib/dracut/dracut.conf.d/99-stackific-gpu.conf
