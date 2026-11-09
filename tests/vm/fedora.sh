# shellcheck shell=bash
# Fedora checks, the same on x86_64 and aarch64: boot.sh sources this file after the checks every image passes
# (`just fedora test`).
# shellcheck disable=SC2154 # efi, vm, vm_until, sendkey and screenshot come from boot.sh, which sources this file

# Identity: stackific-release owns /usr/lib/os-release, which generic-release-common's /etc/os-release links to, and no
# Fedora release or logo package is in the image (Fedora's trademark guidelines)
# shellcheck disable=SC2016 # $ID and friends belong to the guest shell
vm_until '. /etc/os-release && echo "$ID/$ID_LIKE/$VERSION_ID/$PRETTY_NAME/$LOGO"' \
  '^stackific/fedora/45/Stackific OS/stackific$' 0
vm_until 'hostnamectl' 'Operating System: Stackific OS' 0
vm_until "rpm -qf --qf '%{name}\n' /etc/os-release /usr/lib/os-release | paste -sd ' '" \
  '^generic-release-common stackific-release$' 0
vm_until "rpm -qa 'fedora-release*' 'fedora-logos*' fedora-release-notes | wc -l" '^0$' 0
vm_until "grep -c 'fill=\"#007a55\"' /usr/share/icons/hicolor/scalable/apps/stackific.svg" '^1$' 0
# Terra supplies only what fedora/iso/terra-repo.sh allows (limine is checked with the installer below)
vm_until "rpm -qa --qf '%{vendor} %{name}\n' | awk '\$1 == \"Terra\" && \$2 != \"limine\" {print \$2}' | sort | paste -sd ' '" \
  '^mangowm noctalia-greeter$' 0
# dnf5, so installed systems can update
vm_until 'rpm -q dnf5' '^dnf5-' 0
vm_until 'getenforce' '^Enforcing$' 0

# The live session: greetd, not gdm, is the display manager; it logs `live` into MangoWM, which starts Noctalia through
# stackific-noctalia.service
vm_until 'systemctl is-active greetd' '^active$' 0
vm_until 'readlink /etc/systemd/system/display-manager.service' '^/usr/lib/systemd/system/greetd.service$' 0
# Not `systemctl --user -M live@` as on Arch: it hands this shell's vsock socket to PID 1 and dbus-broker, which Fedora's
# SELinux policy denies them (silently, through dontaudit rules)
vm_until 'runuser -u live -- env XDG_RUNTIME_DIR=/run/user/1000 systemctl --user is-active stackific-noctalia.service' \
  '^active$' 120
# The Secret Service: oo7-daemon, started by live's systemd manager (stackific-desktop), owns org.freedesktop.secrets
vm_until "runuser -u live -- env DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus busctl --user status org.freedesktop.secrets | grep -E '^Comm='" \
  '^Comm=oo7-daemon$' 60
vm_until 'pgrep -u live -x mango' '^[0-9]+$' 0
# Regular hardware (task 021): the firmware families Arch's image carries too, Thunderbolt, fingerprint readers and
# MIPI webcams, with no PAM change (no fprintd-pam)
vm_until 'rpm -q alsa-firmware dvb-firmware bolt fprintd libcamera libcamera-ipa pipewire-plugin-libcamera pipewire-v4l2 fprintd-pam | grep -c "not installed"' \
  '^1$' 0
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

# live's MangoWM config (fedora/iso/config.sh): the stock config, stackific-hardware's lighter effects where present,
# and Super+Space for the shell's launcher, as on the Arch image
f=/home/live/.config/mango/config.conf
vm_until "{ stat -c %U $f; grep -cx -e 'source=/etc/mango/config.conf' -e 'source_optional=/run/stackific/mango-gpu.conf' \
-e 'bind=SUPER,space,spawn,noctalia msg panel-toggle launcher' $f; } | paste -sd ' '" '^live 3$' 0
vm_until 'test -f /usr/share/wayland-sessions/mango.desktop -a -f /usr/share/wayland-sessions/gnome.desktop && echo ok' \
  '^ok$' 0

# GNOME's defaults (stackific-desktop): our wallpaper, light and dark and on the lock screen, and the installer first in
# its dash (GNOME skips it where the installer is missing, as on installed systems)
vesuvius="'file:///usr/share/backgrounds/stackific/an-eruption-of-vesuvius-johan-christian-dahl.jpg'"
for key in 'desktop.background picture-uri' 'desktop.background picture-uri-dark' 'desktop.screensaver picture-uri'; do
  vm_until "runuser -u live -- gsettings get org.gnome.${key% *} ${key#* }" "^$vesuvius\$" 0
done
vm_until 'runuser -u live -- gsettings get org.gnome.shell favorite-apps' "^\\['stackific-installer.desktop', " 0

# Artwork and defaults: the shipped wallpapers, Noctalia's defaults through the /etc/skel include, fastfetch's logo
vm_until 'ls /usr/share/backgrounds/stackific/*.jpg | wc -l' '^10$' 0
vm_until 'cmp /etc/skel/.config/noctalia/00-stackific.toml /home/live/.config/noctalia/00-stackific.toml && echo ok' \
  '^ok$' 0
vm_until "runuser -u live -- noctalia config export merged | grep -cxE 'setup_wizard_enabled = false|directory = \"/usr/share/backgrounds/stackific\"|\
 *path = \"/usr/share/backgrounds/stackific/an-eruption-of-vesuvius-johan-christian-dahl.jpg\"'" '^3$' 0
# The greeter's defaults (our palette and wallpaper), linked by tmpfiles into its state directory, which belongs to
# Fedora's greeter user, and its asset folder with our logo
vm_until 'readlink /var/lib/noctalia-greeter/greeter.toml' '^/usr/share/stackific/greeter.toml$' 0
vm_until 'stat -c %U /var/lib/noctalia-greeter' '^greetd$' 0
vm_until "grep -cx 'primary = \"#007a55\"' /usr/share/stackific/greeter.toml" '^1$' 0
vm_until 'readlink -f /usr/share/stackific/greeter-assets/noctalia.svg' '^/usr/share/icons/hicolor/scalable/apps/stackific.svg$' 0
# The logo's last line, in the accent (fastfetch drops colours when piped unless told otherwise)
vm_until "runuser -u live -- fastfetch --pipe false | grep -F '▄▄▄▄▄█████████████████████████████████' | grep -c '38;2;0;122;85'" \
  '^1$' 0
# Our config lists fastfetch's default modules: a config without a modules list prints only the logo
vm_until "runuser -u live -- fastfetch --pipe true | grep -c 'OS: Stackific OS'" '^1$' 0
sleep 10 # Noctalia draws a few seconds after its unit is active, on the VM's software renderer
screenshot screen

# After a logout greetd shows noctalia-greeter as `greetd`. Its comm name is cut to 15 characters, so match the command
# line.
vm 'loginctl terminate-user live' >/dev/null
vm_until "pgrep -u greetd -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
# greetd sourced /etc/profile, so the greeter reads our asset folder (stackific-greeter.sh)
vm_until "tr '\\0' '\\n' </proc/\$(pgrep -u greetd -f '^/usr/bin/noctalia-greeter( |\$)' | head -1)/environ | grep -cx 'NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets'" '^1$' 0
sleep 10
screenshot greeter

# The installer with its launcher and console path, Limine's boot files with limine from Terra (the machine's program,
# BOOTX64.EFI or BOOTAA64.EFI), and Disks with NTFS resizing
vm_until "grep -cx -e 'Name=Install Stackific OS' -e 'Exec=foot --app-id=stackific-installer --window-size-chars=100x30 \
--font=monospace:size=11 --override=dpi-aware=yes run0 stackific-installer' /usr/share/applications/stackific-installer.desktop" '^2$' 0
vm_until 'systemctl cat stackific-installer.target stackific-installer-console.service >/dev/null && echo ok' '^ok$' 0
vm_until "test -x /usr/lib/kernel/install.d/90-stackific-limine.install -a -f /usr/share/limine/$efi && echo ok" '^ok$' 0
vm_until "rpm -q --qf '%{vendor}\n' limine" '^Terra$' 0
vm_until 'test -x /usr/bin/gnome-disks -a -f /usr/share/applications/org.gnome.DiskUtility.desktop -a -x /usr/sbin/ntfsresize && echo ok' \
  '^ok$' 0

# GNOME with our wallpaper and the installer in its dash, logged in through the greeter left up above: `live`,
# with GNOME first in the session list, and any character as the password. Not through greetd's [initial_session], as
# for Plasma on Arch: that opens a logind session of type tty, and GNOME Shell starts only in a wayland one ("Failed to
# find any matching session"), which the greeter asks greetd for. GNOME opens on its overview, which shows the dash.
sleep 15 # the greeter draws a few seconds after it starts, on the VM's software renderer
sendkey a ret
vm_until 'pgrep -u live -x gnome-shell >/dev/null && echo up' '^up$' 180
sleep 30 # GNOME draws a while after gnome-shell starts
screenshot gnome
# SELinux denied nothing during boot, the session, the logout, the greeter and GNOME (the kernel writes two spaces)
vm_until "journalctl -b | grep -c 'avc:  denied'" '^0$' 0

# NVIDIA (task 020, on aarch64 task 022): RPM Fusion's driver with NVIDIA's open modules, built for the image's kernel by
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
