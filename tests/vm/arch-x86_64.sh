# shellcheck shell=bash
# Arch x86_64 checks: boot.sh sources this file after the checks every image passes (`just arch test`).

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

# After a logout greetd shows noctalia-greeter. Its comm name is cut to 15 characters, so match the command line.
vm 'loginctl terminate-user live' >/dev/null
vm_until "pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
# greetd sourced /etc/profile, so the greeter reads our asset folder (stackific-greeter.sh)
vm_until "tr '\\0' '\\n' </proc/\$(pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)' | head -1)/environ | grep -cx 'NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets'" '^1$' 0
sleep 10
screenshot greeter
