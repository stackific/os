# shellcheck shell=bash
# Fedora x86_64 checks: the Fedora checks both architectures pass (tests/vm/fedora.sh), then what only the x86_64 image
# has until task 018 brings the installer to aarch64 (`just fedora test` sources this file there).
# shellcheck disable=SC2154 # vm, vm_until, screenshot and fail come from boot.sh, which sources this file
# shellcheck source=/dev/null # fedora.sh sits next to this file
source "$(dirname "${BASH_SOURCE[0]}")/fedora.sh"

# The installer with its launcher and console path, and Limine's boot files with limine from Terra
vm_until "grep -cx -e 'Name=Install Stackific OS' -e 'Exec=foot --app-id=stackific-installer --window-size-chars=100x30 \
--font=monospace:size=11 --override=dpi-aware=yes run0 stackific-installer' /usr/share/applications/stackific-installer.desktop" '^2$' 0
vm_until 'systemctl cat stackific-installer.target stackific-installer-console.service >/dev/null && echo ok' '^ok$' 0
vm_until 'test -x /usr/lib/kernel/install.d/90-stackific-limine.install -a -f /usr/share/limine/BOOTX64.EFI && echo ok' '^ok$' 0
vm_until "rpm -q --qf '%{vendor}\n' limine" '^Terra$' 0

# GNOME with our wallpaper and the installer in its dash, logged in through the greeter fedora.sh left up: `live`,
# with GNOME first in the session list, and any character as the password. Not through greetd's [initial_session], as
# for Plasma on Arch: that opens a logind session of type tty, and GNOME Shell starts only in a wayland one ("Failed to
# find any matching session"), which the greeter asks greetd for. GNOME opens on its overview, which shows the dash.
sleep 15 # the greeter draws a few seconds after it starts, on the VM's software renderer
sendkey a ret
vm_until 'pgrep -u live -x gnome-shell >/dev/null && echo up' '^up$' 180
sleep 30 # GNOME draws a while after gnome-shell starts
screenshot gnome
vm_until "journalctl -b | grep -c 'avc:  denied'" '^0$' 0
