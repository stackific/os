# shellcheck shell=bash
# Arch x86_64 checks: the Arch checks both architectures pass (tests/vm/arch.sh), then Arch's prebuilt NVIDIA modules
# and the fallback kernel, which only the x86_64 image carries (`just arch test` sources this file there).
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
