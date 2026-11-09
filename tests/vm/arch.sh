# shellcheck shell=bash
# Arch checks, for both architectures (x86_64 and Arch Linux ARM aarch64): boot.sh sources this file after the checks
# every image passes (`just arch test`).

# Identity: stackific-release owns /etc/os-release, Arch's files stay as shipped
# shellcheck disable=SC2016 # $ID and friends belong to the guest shell
vm_until '. /etc/os-release && echo "$ID/$ID_LIKE/$BUILD_ID/$PRETTY_NAME/$LOGO/$IMAGE_ID"' \
  '^stackific/arch/rolling/Stackific OS/stackific/stackific$' 0
vm_until 'hostnamectl' 'Operating System: Stackific OS' 0
vm_until "pacman -Qqo /etc/os-release /etc/arch-release /usr/lib/os-release | paste -sd ' '" \
  '^stackific-release filesystem filesystem$' 0
vm_until 'grep -c IMAGE_ /usr/lib/os-release' '^0$' 0
vm_until "grep -c 'fill=\"#007a55\"' /usr/share/icons/hicolor/scalable/apps/stackific.svg" '^1$' 0

# The live session: greetd logs `live` into MangoWM, which starts Noctalia through stackific-noctalia.service
vm_until 'systemctl is-active greetd' '^active$' 0
vm_until 'systemctl --user -M live@ is-active stackific-noctalia.service' '^active$' 120
vm_until 'pgrep -u live -x mango' '^[0-9]+$' 0
# The Secret Service: oo7-daemon, started by live's systemd manager (stackific-desktop), owns org.freedesktop.secrets
vm_until "runuser -u live -- env DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus busctl --user status org.freedesktop.secrets | grep -E '^Comm='" \
  '^Comm=oo7-daemon$' 60
vm_until 'test -f /usr/share/wayland-sessions/mango.desktop -a -f /usr/share/wayland-sessions/plasma.desktop && echo ok' \
  '^ok$' 0

# Artwork and defaults: the shipped wallpapers, Noctalia's defaults through the /etc/skel include, fastfetch's logo
vm_until 'ls /usr/share/backgrounds/stackific/*.jpg | wc -l' '^10$' 0
vm_until 'cmp /etc/skel/.config/noctalia/00-stackific.toml /home/live/.config/noctalia/00-stackific.toml && echo ok' \
  '^ok$' 0
vm_until "runuser -u live -- noctalia config export merged | grep -cxE 'setup_wizard_enabled = false|directory = \"/usr/share/backgrounds/stackific\"|\
 *path = \"/usr/share/backgrounds/stackific/an-eruption-of-vesuvius-johan-christian-dahl.jpg\"'" '^3$' 0
# The greeter's defaults (our palette and wallpaper), linked by tmpfiles, and its asset folder with our logo
vm_until 'readlink /var/lib/noctalia-greeter/greeter.toml' '^/usr/share/stackific/greeter.toml$' 0
vm_until "grep -cx 'primary = \"#007a55\"' /usr/share/stackific/greeter.toml" '^1$' 0
vm_until 'readlink -f /usr/share/stackific/greeter-assets/noctalia.svg' '^/usr/share/icons/hicolor/scalable/apps/stackific.svg$' 0
# The logo's last line, in the accent (fastfetch drops colours when piped unless told otherwise)
vm_until "runuser -u live -- fastfetch --pipe false | grep -F '▄▄▄▄▄█████████████████████████████████' | grep -c '38;2;0;122;85'" \
  '^1$' 0
# Our config lists fastfetch's default modules: a config without a modules list prints only the logo
vm_until "runuser -u live -- fastfetch --pipe true | grep -c 'OS: Stackific OS'" '^1$' 0
sleep 10 # Noctalia draws a few seconds after its unit is active, on the VM's software renderer
screenshot screen
# Super+Space opens the shell's launcher, from `live`'s MangoWM config, which sources the stock one and, where present,
# stackific-hardware's lighter effects
vm_until "grep -cx -e 'source=/etc/mango/config.conf' -e 'source_optional=/run/stackific/mango-gpu.conf' \
-e 'bind=SUPER,space,spawn,noctalia msg panel-toggle launcher' /home/live/.config/mango/config.conf" '^3$' 0
sendkey meta_l-spc
sleep 5
screenshot launcher
sendkey esc

# After a logout greetd shows noctalia-greeter. Its comm name is cut to 15 characters, so match the command line.
vm 'loginctl terminate-user live' >/dev/null
vm_until "pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
# greetd sourced /etc/profile, so the greeter reads our asset folder (stackific-greeter.sh)
vm_until "tr '\\0' '\\n' </proc/\$(pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)' | head -1)/environ | grep -cx 'NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets'" '^1$' 0
sleep 10
screenshot greeter

# The installer's launcher, Disks with NTFS resizing, and Plasma's GTK integration, which gives Disks Plasma's look
vm_until "grep -cx -e 'Name=Install Stackific OS' -e 'Exec=foot --app-id=stackific-installer --window-size-chars=100x30 \
--font=monospace:size=11 --override=dpi-aware=yes run0 stackific-installer' /usr/share/applications/stackific-installer.desktop" '^2$' 0
vm_until 'test -x /usr/bin/gnome-disks -a -f /usr/share/applications/org.gnome.DiskUtility.desktop -a -x /usr/bin/ntfsresize && echo ok' \
  '^ok$' 0
vm_until 'test -d /usr/share/themes/Breeze/gtk-3.0 && pacman -Q kde-gtk-config >/dev/null && echo ok' '^ok$' 0
# The apps (task 023): kitty, btop, Thunar with File Roller in its menu and thumbnails, and Bazaar with Flathub, which
# flatpak's own remotes.d file adds to the system installation on first use
vm_until 'pacman -Q kitty btop thunar thunar-archive-plugin tumbler gvfs file-roller flatpak bazaar | wc -l' '^9$' 0
vm_until 'flatpak remotes --system --columns=name' '^flathub$' 0

# The live user's desktop shortcut to the installer, which Plasma shows on its desktop (a link into the system's
# applications, so Plasma trusts it); the install removes it with /home/live
vm_until 'readlink /home/live/Desktop/stackific-installer.desktop' \
  '^/usr/share/applications/stackific-installer.desktop$' 0
vm_until 'stat -c %U /home/live/Desktop' '^live$' 0
# Plasma's look and feel: Breeze with our wallpaper (stackific-desktop), selected for every user in /etc/xdg/kdeglobals
vm_until 'runuser -u live -- kreadconfig6 --file kdeglobals --group KDE --key LookAndFeelPackage' '^org\.stackific\.desktop$' 0
vm_until "grep -A1 -x '\\[Wallpaper\\]' /usr/share/plasma/look-and-feel/org.stackific.desktop/contents/defaults | tail -1" '^Image=stackific$' 0
vm_until 'readlink -f /usr/share/wallpapers/stackific/contents/images/3872x2592.jpg' '^/usr/share/backgrounds/stackific/an-eruption-of-vesuvius-johan-christian-dahl\.jpg$' 0
# Plasma with the shortcut and our wallpaper: greetd logs `live` into Plasma once its run file is gone (the greeter is up from above)
vm "sed -i 's|^command = \"mango\"|command = \"/usr/lib/plasma-dbus-run-session-if-needed /usr/bin/startplasma-wayland\"|' /etc/greetd/config.toml && rm -f /run/greetd.run && systemctl restart greetd" >/dev/null
vm_until 'pgrep -u live -x plasmashell >/dev/null && echo up' '^up$' 180
sleep 30 # Plasma draws its desktop a while after plasmashell starts, on the VM's software renderer
screenshot plasma

# The driver pick (task 019, on aarch64 task 022) on QEMU's display (no NVIDIA): the generator is installed and wrote
# nothing, so nvidia-utils keeps nouveau blacklisted and greetd starts without waiting for udev
vm_until 'test -x /usr/lib/systemd/system-generators/stackific-gpu-generator && echo ok' '^ok$' 0
vm_until 'ls /run/modprobe.d/nvidia-utils.conf /run/stackific/mango-gpu.conf /run/systemd/generator/greetd.service.d 2>/dev/null | wc -l' \
  '^0$' 0
vm_until "lsmod | grep -cE '^(nvidia|nouveau) '" '^0$' 0

# Regular hardware (task 021, on aarch64 task 022): the firmware families Fedora's image carries too, Thunderbolt,
# fingerprint readers and MIPI webcams, with no PAM change
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
