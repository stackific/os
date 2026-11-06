# shellcheck shell=bash
# Arch x86_64 checks: the Arch checks both architectures pass (tests/vm/arch.sh), then what only the x86_64 image has
# until task 018 brings the installer to aarch64 (`just arch test` sources this file there).
# shellcheck disable=SC2154 # vm, vm_until, screenshot and fail come from boot.sh, which sources this file
# shellcheck source=/dev/null # arch.sh sits next to this file
source "$(dirname "${BASH_SOURCE[0]}")/arch.sh"

# The installer's launcher, Disks with NTFS resizing, and Plasma's GTK integration, which gives Disks Plasma's look
vm_until "grep -cx -e 'Name=Install Stackific OS' -e 'Exec=foot --app-id=stackific-installer --window-size-chars=100x30 \
--font=monospace:size=11 --override=dpi-aware=yes run0 stackific-installer' /usr/share/applications/stackific-installer.desktop" '^2$' 0
vm_until 'test -x /usr/bin/gnome-disks -a -f /usr/share/applications/org.gnome.DiskUtility.desktop -a -x /usr/bin/ntfsresize && echo ok' \
  '^ok$' 0
vm_until 'test -d /usr/share/themes/Breeze/gtk-3.0 && pacman -Q kde-gtk-config >/dev/null && echo ok' '^ok$' 0

# The live user's desktop shortcut to the installer, which Plasma shows on its desktop (a link into the system's
# applications, so Plasma trusts it); the install removes it with /home/live
vm_until 'readlink /home/live/Desktop/stackific-installer.desktop' \
  '^/usr/share/applications/stackific-installer.desktop$' 0
vm_until 'stat -c %U /home/live/Desktop' '^live$' 0
# Plasma with the shortcut: greetd logs `live` into Plasma once its run file is gone (arch.sh left the greeter up)
vm "sed -i 's|^command = \"mango\"|command = \"/usr/lib/plasma-dbus-run-session-if-needed /usr/bin/startplasma-wayland\"|' /etc/greetd/config.toml && rm -f /run/greetd.run && systemctl restart greetd" >/dev/null
vm_until 'pgrep -u live -x plasmashell >/dev/null && echo up' '^up$' 180
sleep 30 # Plasma draws its desktop a while after plasmashell starts, on the VM's software renderer
screenshot plasma
