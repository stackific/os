# shellcheck shell=bash
# Arch Linux ARM aarch64 checks: boot.sh sources this file after the checks every image passes (`just arch test`).
# A console image until the aarch64 desktop task.

vm_until 'uname -m' '^aarch64$' 0
vm_until 'systemctl is-active getty@tty1' '^active$' 0
# getty draws its login prompt a moment after it starts; /dev/vcs1 holds tty1's text
vm_until "grep -ao 'stackific login:' /dev/vcs1" 'stackific login:' 30
screenshot screen
