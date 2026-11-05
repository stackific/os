# shellcheck shell=bash
# Arch Linux ARM aarch64 checks: boot.sh sources this file after the checks every image passes (`just arch test`).
# A console image until the aarch64 desktop task.

vm_until 'uname -m' '^aarch64$' 0

# Identity and artwork, as on x86_64: stackific-release owns /etc/os-release, Arch Linux ARM's files stay as shipped,
# and stackific-artwork brings the wallpapers and fastfetch with our logo
# shellcheck disable=SC2016 # $ID and friends belong to the guest shell
vm_until '. /etc/os-release && echo "$ID/$ID_LIKE/$BUILD_ID/$PRETTY_NAME/$LOGO/$IMAGE_ID"' \
  '^stackific/arch/rolling/Stackific OS/stackific/stackific$' 0
vm_until "pacman -Qqo /etc/os-release /etc/arch-release /usr/lib/os-release | paste -sd ' '" \
  '^stackific-release filesystem filesystem$' 0
vm_until 'ls /usr/share/backgrounds/stackific/*.jpg | wc -l' '^10$' 0
# The logo's last line, in the accent (fastfetch drops colours when piped unless told otherwise)
vm_until "fastfetch --pipe false | grep -F '▄▄▄▄▄█████████████████████████████████' | grep -c '38;2;0;122;85'" '^1$' 0
vm_until "fastfetch --pipe true | grep -c 'OS: Stackific OS'" '^1$' 0

vm_until 'systemctl is-active getty@tty1' '^active$' 0
# getty draws its login prompt a moment after it starts; /dev/vcs1 holds tty1's text
vm_until "grep -ao 'stackific login:' /dev/vcs1" 'stackific login:' 30
screenshot screen
