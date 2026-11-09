# shellcheck shell=bash
# Arch x86_64 checks: the Arch checks both architectures pass (tests/vm/arch.sh), then NVIDIA and the fallback kernel,
# which only the x86_64 image carries until task 022 (`just arch test` sources this file there).
# shellcheck disable=SC2154 # vm, vm_until, screenshot and fail come from boot.sh, which sources this file
# shellcheck source=/dev/null # arch.sh sits next to this file
source "$(dirname "${BASH_SOURCE[0]}")/arch.sh"

# NVIDIA's open modules for both kernels (task 019), from the same archive date: each module's vermagic starts with
# its kernel's version
vm_until 'pacman -Q linux-lts nvidia-open nvidia-open-lts nvidia-utils stackific-hardware | wc -l' '^5$' 0
# shellcheck disable=SC2016 # $v and $(…) belong to the guest shell
vm_until 'for v in $(ls /usr/lib/modules); do [ "$(modinfo -k "$v" -F vermagic nvidia | cut -d" " -f1)" = "$v" ] && echo "$v"; done | wc -l' \
  '^2$' 0
# The live medium boots linux only: no linux-lts files on it (the live image's stackific-live-lts.hook)
vm_until 'ls /run/archiso/bootmnt/arch/boot/x86_64 | grep -c lts' '^0$' 0
# The driver pick on QEMU's std VGA (no NVIDIA): the generator is installed and wrote nothing, so nvidia-utils keeps
# nouveau blacklisted and greetd starts without waiting for udev
vm_until 'test -x /usr/lib/systemd/system-generators/stackific-gpu-generator && echo ok' '^ok$' 0
vm_until 'ls /run/modprobe.d/nvidia-utils.conf /run/stackific/mango-gpu.conf /run/systemd/generator/greetd.service.d 2>/dev/null | wc -l' \
  '^0$' 0
vm_until "lsmod | grep -cE '^(nvidia|nouveau) '" '^0$' 0

# Regular hardware (task 021): the firmware families Fedora's image carries too, Thunderbolt, fingerprint readers and
# MIPI webcams, with no PAM change
vm_until 'pacman -Q linux-firmware-marvell sof-firmware alsa-firmware wireless-regdb bolt fprintd libcamera libcamera-ipa pipewire-libcamera pipewire-v4l2 | wc -l' \
  '^10$' 0
# Sentinel files of each firmware family (any compression suffix), Marvell's Prestera switch firmware left out, and the
# services answering; no PAM file uses the fingerprint reader
# shellcheck disable=SC2016 # $f belongs to the guest shell
vm_until 'for f in regulatory.db mrvl/pcie8897_uapsta.bin ctefx.bin dvb-usb-dib0700-1.20.fw yamaha/yss225_registers.bin ea/3g_asic.fw; do ls /usr/lib/firmware/"$f"* >/dev/null 2>&1 && echo "$f"; done | wc -l' \
  '^6$' 0
vm_until 'ls -d /usr/lib/firmware/mrvl/prestera 2>/dev/null | wc -l' '^0$' 0
vm_until 'ls -d /usr/lib/firmware/intel/sof* | wc -l' '^[1-9]' 0
vm_until 'boltctl list >/dev/null && echo ok' '^ok$' 0
vm_until 'busctl --system introspect net.reactivated.Fprint /net/reactivated/Fprint/Manager >/dev/null && echo ok' '^ok$' 0
vm_until 'ls /usr/lib*/spa-0.2/libcamera/libspa-libcamera.so | wc -l' '^[1-9]' 0 # /usr/lib64 is a link to /usr/lib on Arch
# (the desktops' own fingerprint services excepted: Plasma's lock screen, kscreenlocker's kde-fingerprint, and GDM's
# gdm-fingerprint, which nothing here starts)
vm_until 'grep -rl pam_fprintd /etc/pam.d /usr/lib/pam.d 2>/dev/null | grep -vcE "/(kde|gdm)-fingerprint$"' '^0$' 0
