# Stackific OS boot files for Fedora: keeps Limine on our EFI System Partition current, as arch/packages/stackific-boot
# does on Arch. The installer copies /usr/share/limine/BOOTX64.EFI to /boot/EFI/limine/ (the firmware entry's loader)
# and /boot/EFI/BOOT/ (the firmware's fallback path) and writes /etc/kernel/cmdline, then runs kernel-install. Kernels:
# kernel-install runs with layout=other on the ESP at /boot, Fedora's dracut plugin writes the initramfs, and our plugin
# copies the kernel and writes limine.conf, on every kernel install and removal, with the Windows entries the installer
# left in /etc/kernel/limine-windows.conf after the kernels. Limine's program: the trigger copies it
# again after every limine install or upgrade. The files are Fedora-only, so they sit next to this spec, except the
# boot-failure screen (installer/initramfs/), which the dracut module 99stackific-boot-failure puts into installed
# systems' initramfs (task 016). cryptsetup stays installed for changing the disk passphrase.

Name: stackific-boot
Version: 0.1.0
Release: 3%{?dist}
Summary: Stackific OS boot files: keeps Limine and its menu on the EFI System Partition current
License: Apache-2.0
URL: https://os.stackific.com
BuildArch: noarch
Source0: 50-stackific.conf
Source1: 90-stackific-limine.install
Source2: dracut-stackific-boot-failure.sh
Source3: stackific-boot-failure
Source4: stackific-boot-failure.conf
Requires: cryptsetup
Requires: dracut
Requires: efibootmgr
Requires: limine
Requires: systemd-udev

%description
kernel-install settings and a plugin that put each kernel on the EFI System Partition and list every installed kernel in
Limine's menu, a trigger that copies Limine's EFI program to the partition after limine is installed or upgraded, and a
dracut module that shows a plain screen and restarts on Enter when the start fails (a wrong disk password three times).

%install
install -Dm644 %{SOURCE0} %{buildroot}%{_prefix}/lib/kernel/install.conf.d/50-stackific.conf
install -Dm755 %{SOURCE1} %{buildroot}%{_prefix}/lib/kernel/install.d/90-stackific-limine.install
install -Dm755 %{SOURCE2} %{buildroot}%{_prefix}/lib/dracut/modules.d/99stackific-boot-failure/module-setup.sh
install -Dm755 %{SOURCE3} %{buildroot}%{_prefix}/lib/stackific/boot-failure
install -Dm644 %{SOURCE4} %{buildroot}%{_prefix}/lib/stackific/boot-failure.conf

# Where /boot/EFI/limine/ is missing (the live session and its build), nothing to do
%triggerin -- limine
if [ -d /boot/EFI/limine ]; then
  install -Dm644 /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/BOOTX64.EFI
  install -Dm644 /usr/share/limine/BOOTX64.EFI /boot/EFI/BOOT/BOOTX64.EFI
fi

%files
%dir %{_prefix}/lib/kernel/install.conf.d
%{_prefix}/lib/kernel/install.conf.d/50-stackific.conf
%{_prefix}/lib/kernel/install.d/90-stackific-limine.install
%{_prefix}/lib/dracut/modules.d/99stackific-boot-failure
%{_prefix}/lib/stackific
