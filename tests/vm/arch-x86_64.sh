# shellcheck shell=bash
# Arch x86_64 checks: boot.sh sources this file after the checks every image passes (`just arch test`).

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
