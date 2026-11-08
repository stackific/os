# shellcheck shell=bash
# Fedora checks, the same on x86_64 and aarch64: boot.sh sources this file after the checks every image passes
# (`just fedora test`; on x86_64 through tests/vm/fedora-x86_64.sh).

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
# Terra supplies only what fedora/iso/terra-repo.sh allows (limine only on x86_64, fedora-x86_64.sh)
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
vm_until 'test -f /usr/share/wayland-sessions/mango.desktop -a -f /usr/share/wayland-sessions/gnome.desktop && echo ok' \
  '^ok$' 0

# GNOME's defaults (stackific-desktop): our wallpaper, light and dark and on the lock screen, and the installer first in
# its dash (GNOME skips it where the installer is missing, as on aarch64 and installed systems)
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

# SELinux denied nothing during boot, the session, the logout and the greeter (the kernel writes two spaces)
vm_until "journalctl -b | grep -c 'avc:  denied'" '^0$' 0
