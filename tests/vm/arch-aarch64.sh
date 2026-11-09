# shellcheck shell=bash
# Arch Linux ARM aarch64 checks: the Arch checks both architectures pass (tests/vm/arch.sh), then NVIDIA's open modules
# from nvidia-open-dkms (task 022), which DKMS built for linux-aarch64 while the image was built (`just arch test`
# sources this file there).
# shellcheck disable=SC2154 # vm, vm_until, screenshot and fail come from boot.sh, which sources this file
# shellcheck source=/dev/null # arch.sh sits next to this file
source "$(dirname "${BASH_SOURCE[0]}")/arch.sh"

vm_until 'pacman -Q linux-aarch64-headers nvidia-open-dkms nvidia-utils stackific-hardware | wc -l' '^4$' 0
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until 'dkms status -k "$(uname -r)" nvidia' '^nvidia/[0-9.]+, [^,]+, aarch64: installed' 0
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until '[ "$(modinfo -F vermagic nvidia | cut -d" " -f1)" = "$(uname -r)" ] && modinfo -F license nvidia' '^Dual MIT/GPL$' 0
