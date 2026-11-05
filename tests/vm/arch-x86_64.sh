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
sleep 10 # Noctalia draws a few seconds after its unit is active, on the VM's software renderer
screenshot screen

# After a logout greetd shows noctalia-greeter. Its comm name is cut to 15 characters, so match the command line.
vm 'loginctl terminate-user live' >/dev/null
vm_until "pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
sleep 10
screenshot greeter
