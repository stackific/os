# shellcheck shell=bash
# Checks on an installed system booted from its disk alone: `just arch test-install`'s second boot (boot.sh with `-`
# for the ISO and the first boot's firmware variables), after tests/vm/install.sh's free-space install, which is the
# one on the disk. The firmware started our entry; Limine started the kernel from our ESP with root on @; the boot
# files, the firmware entries and stackific-boot's hook (the recipe put the pinned limine package at the ESP's root);
# the system the installer set up; and the keyboard layout in MangoWM and in Plasma. Arch's kernel paths and packages
# until task 014 brings Fedora. Leaves limine.png (boot.sh), installed.png (the login screen) and efibootmgr.txt.
# shellcheck disable=SC2154 # out, vm, vm_until, fail and screenshot come from boot.sh, which sources this file

# shellcheck disable=SC2016 # $NAME belongs to the guest shell
name=$(vm '. /etc/os-release && echo "$NAME"' | tr -d '\r\n')
[ -n "$name" ] || fail "no NAME in /etc/os-release"

# The firmware started our entry: labelled with the product name, first in the boot order, the only one with that
# label after the erase and the free-space install on this disk
grep -aq "BdsDxe: starting Boot[0-9A-F]\{4\} \"$name\"" "$out/serial.log" || fail "the firmware did not start \"$name\""
echo "ok: the firmware started \"$name\""
entries=$(vm efibootmgr)
printf '%s\n' "$entries" >"$out/efibootmgr.txt"
current=$(sed -n 's/^BootCurrent: //p' <<<"$entries")
grep -q "^Boot$current\\* $name	" <<<"$entries" || fail "BootCurrent $current is not \"$name\": $entries"
[ "$(sed -n 's/^BootOrder: //p' <<<"$entries" | cut -d, -f1)" = "$current" ] || fail "\"$name\" is not first: $entries"
[ "$(grep -c "^Boot[0-9A-F]\\{4\\}\\* $name	" <<<"$entries")" = 1 ] || fail "not one \"$name\" entry: $entries"
echo "ok: one \"$name\" entry, first in the boot order, started this boot"

# Limine started the kernel with root on the @ subvolume; the btrfs default stays the top level
uuid=$(vm 'findmnt -no UUID /' | tr -d '\r\n')
vm_until 'cat /proc/cmdline' "(^| )root=UUID=$uuid rootflags=subvol=/@ rw( |\$)" 0
vm_until 'findmnt -no FSTYPE,OPTIONS /' '^btrfs .*subvol=/@(,|$)' 0
vm_until 'findmnt -no FSTYPE /boot' '^vfat$' 0
vm_until 'btrfs subvolume get-default /' '^ID 5 \(FS_TREE\)$' 0

# The ESP: the kernel and initramfs from mkinitcpio's preset (no fallback image, no unified kernel image), limine.conf,
# and Limine's program at the firmware entry's path and the fallback path; the initramfs has Arch's default hooks
vm_until 'cd /boot && ls -d vmlinuz-linux initramfs-linux.img limine.conf EFI/limine/BOOTX64.EFI EFI/BOOT/BOOTX64.EFI | wc -l' \
  '^5$' 0
vm_until 'ls -d /boot/*fallback* /boot/EFI/Linux 2>/dev/null | wc -l' '^0$' 0
efi_same='cmp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/BOOTX64.EFI && cmp /usr/share/limine/BOOTX64.EFI /boot/EFI/BOOT/BOOTX64.EFI && echo same'
vm_until "$efi_same" '^same$' 0
vm_until 'lsinitcpio /boot/initramfs-linux.img | grep -c archiso' '^0$' 0
vm_until "lsinitcpio /boot/initramfs-linux.img | grep -c 'usr/lib/systemd/systemd\$'" '^1$' 0
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until '[ "$(ls /usr/lib/modules)" = "$(uname -r)" ] && echo match' '^match$' 0

# Packages: the boot files stay, the installer and what only it needed are gone
vm_until 'pacman -Q stackific-boot limine efibootmgr | wc -l' '^3$' 0
vm_until 'pacman -Q stackific-installer arch-install-scripts 2>/dev/null | wc -l' '^0$' 0

# The system the installer set up, and its login screen
vm_until 'hostnamectl hostname' '^stackific-test$' 0
vm_until 'timedatectl show -P Timezone' '^Europe/Berlin$' 0
vm_until 'localectl status' 'X11 Layout: de$' 0
vm_until 'localectl status' 'VC Keymap: de$' 0
vm_until 'id -nG tester' '(^| )wheel( |$)' 0
vm_until 'systemctl is-active greetd' '^active$' 0
vm_until "pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
sleep 10 # the greeter draws a few seconds after it starts, on the VM's software renderer
screenshot installed

# stackific-boot's hook: reinstalling limine (an upgrade to pacman's hooks) restores both copies of its program, and
# leaves the firmware entries alone
vm ': >/boot/EFI/limine/BOOTX64.EFI && : >/boot/EFI/BOOT/BOOTX64.EFI' >/dev/null
vm_until 'pacman -U --noconfirm /boot/limine-*.pkg.tar.zst >/dev/null && rm /boot/limine-*.pkg.tar.zst && echo reinstalled' \
  '^reinstalled$' 0
vm_until "$efi_same" '^same$' 0
[ "$(vm efibootmgr)" = "$entries" ] || fail "reinstalling limine changed the firmware entries"
echo "ok: stackific-boot's hook restored Limine's program"

# The keyboard layout the installer set, in each desktop: greetd logs `tester` in through an initial session, as the
# live test does (the image ships none). MangoWM reads the user's config.conf, which sources the stock one and sets the
# layout; Plasma takes localed's X11 layout while the user has no kxkbrc layout.
uid=$(vm 'id -u tester' | tr -d '\r\n')
session() {
  vm "printf '\\n[initial_session]\\ncommand = \"%s\"\\nuser = \"tester\"\\n' '$1' >>/etc/greetd/config.toml \
    && loginctl terminate-user tester 2>/dev/null; rm -f /run/greetd.run && systemctl restart greetd" >/dev/null
}
session mango
vm_until 'pgrep -u tester -x mango' '^[0-9]+$' 120
vm_until "runuser -u tester -- env MANGO_INSTANCE_SIGNATURE=/run/user/$uid/mango-\$(pgrep -u tester -x mango).sock mmsg get keyboardlayout" \
  '^\{"layout":"German"\}$' 60
vm "sed -i '/^\\[initial_session\\]/,\$d' /etc/greetd/config.toml" >/dev/null
session '/usr/lib/plasma-dbus-run-session-if-needed /usr/bin/startplasma-wayland'
vm_until 'pgrep -u tester -x plasmashell' '^[0-9]+$' 180
vm_until "runuser -u tester -- env DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$uid/bus busctl --user call org.kde.keyboard /Layouts org.kde.KeyboardLayouts getLayoutsList" \
  '^a\(sss\) [0-9]+ "de"' 60
echo "ok: MangoWM and Plasma start with the installer's layout"
