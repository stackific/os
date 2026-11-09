# shellcheck shell=bash
# Fedora x86_64 checks: the Fedora checks both architectures pass (tests/vm/fedora.sh), then NVIDIA, which only the
# x86_64 image carries: RPM Fusion's aarch64 driver cannot be installed (task 022; `just fedora test` sources this file
# there).
# shellcheck disable=SC2154 # vm, vm_until, screenshot and fail come from boot.sh, which sources this file
# shellcheck source=/dev/null # fedora.sh sits next to this file
source "$(dirname "${BASH_SOURCE[0]}")/fedora.sh"

# NVIDIA (task 020): RPM Fusion's driver with NVIDIA's open modules, built for the image's kernel by
# `just fedora packages`, and no compiler, kernel headers or akmods in the image
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until 'rpm -q xorg-x11-drv-nvidia xorg-x11-drv-nvidia-libs stackific-hardware "kmod-nvidia-$(uname -r)" | grep -vc "not installed"' \
  '^4$' 0
vm_until 'rpm -q gcc kernel-devel akmod-nvidia akmods xorg-x11-drv-nvidia-kmodsrc | grep -c "not installed"' '^5$' 0
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until '[ "$(modinfo -F vermagic nvidia | cut -d" " -f1)" = "$(uname -r)" ] && modinfo -F license nvidia' '^Dual MIT/GPL$' 0
# Images keep no RPM Fusion repo file (installed systems get the driver's updates from our repos, task 028)
vm_until 'ls /etc/yum.repos.d/ /usr/share/dnf5/repos.d/ 2>/dev/null | grep -ci rpmfusion' '^0$' 0
# nouveau stays out of the initramfs and is blacklisted unless the generator says otherwise
vm_until 'grep -cx "omit_drivers+=\" nouveau nova_core nova_drm \"" /usr/lib/dracut/dracut.conf.d/99-stackific-gpu.conf' '^1$' 0
vm_until 'grep -cx -e "blacklist nouveau" -e "softdep nvidia post: nvidia-uvm nvidia-drm" /usr/lib/modprobe.d/nvidia-utils.conf' '^2$' 0
# The driver pick on QEMU's display (no NVIDIA): the generator is installed and wrote nothing
vm_until 'test -x /usr/lib/systemd/system-generators/stackific-gpu-generator && echo ok' '^ok$' 0
vm_until 'ls /run/modprobe.d/nvidia-utils.conf /run/stackific/mango-gpu.conf /run/systemd/generator/greetd.service.d 2>/dev/null | wc -l' \
  '^0$' 0
vm_until "lsmod | grep -cE '^(nvidia|nouveau) '" '^0$' 0
