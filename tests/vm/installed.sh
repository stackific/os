# shellcheck shell=bash
# Checks on an installed system booted from its disk alone: the second boot of `just arch test-install` and
# `just fedora test-install` (boot.sh with `-` for the ISO and the first boot's firmware variables), after
# tests/vm/install.sh's free-space install, which is the one on the disk. The firmware started our entry; Limine started
# the kernel from our ESP with root on @; the boot files, the firmware entries and stackific-boot's hooks (the recipe put
# the image's limine package, and on Fedora its kernel-core package, at the ESP's root); the system the installer set
# up; and the keyboard layout in MangoWM and in the second desktop (Plasma, GNOME). Leaves limine.png (boot.sh),
# installed.png (the login screen) and efibootmgr.txt.
# shellcheck disable=SC2154 # out, vm, vm_until, fail and screenshot come from boot.sh, which sources this file

# shellcheck disable=SC2016 # $NAME belongs to the guest shell
name=$(vm '. /etc/os-release && echo "$NAME"' | tr -d '\r\n')
[ -n "$name" ] || fail "no NAME in /etc/os-release"
# shellcheck disable=SC2016 # $ID_LIKE belongs to the guest shell
like=$(vm '. /etc/os-release && echo "$ID_LIKE"' | tr -d '\r\n')
version=$(vm 'uname -r' | tr -d '\r\n')

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

# The ESP: the kernel and its initramfs (no fallback or rescue image, no unified kernel image), limine.conf, and
# Limine's program at the firmware entry's path and the fallback path; the initramfs is the base's own, with no live
# image's hooks. Arch: mkinitcpio's preset with Arch's default hooks. Fedora: kernel-install with Fedora's dracut, the
# kernel's version in the names, no GRUB.
efi_same='cmp /usr/share/limine/BOOTX64.EFI /boot/EFI/limine/BOOTX64.EFI && cmp /usr/share/limine/BOOTX64.EFI /boot/EFI/BOOT/BOOTX64.EFI && echo same'
vm_until "$efi_same" '^same$' 0
# (systemd-boot-random-seed.service writes /boot/loader/random-seed on any ESP; Boot Loader Spec entries would be in
# loader/entries)
vm_until 'ls -d /boot/*fallback* /boot/*rescue* /boot/EFI/Linux /boot/loader/entries /boot/grub2 2>/dev/null | wc -l' '^0$' 0
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until '[ "$(ls /usr/lib/modules)" = "$(uname -r)" ] && echo match' '^match$' 0
if [ "$like" = fedora ]; then
  vm_until "cd /boot && ls -d vmlinuz-$version initramfs-$version.img limine.conf EFI/limine/BOOTX64.EFI EFI/BOOT/BOOTX64.EFI | wc -l" \
    '^5$' 0
  vm_until "lsinitrd /boot/initramfs-$version.img | grep -c dmsquash" '^0$' 0
  vm_until "lsinitrd -m /boot/initramfs-$version.img | grep -cx btrfs" '^1$' 0
  vm_until "cmp /usr/lib/modules/$version/vmlinuz /boot/vmlinuz-$version && echo same" '^same$' 0
else
  vm_until 'cd /boot && ls -d vmlinuz-linux initramfs-linux.img limine.conf EFI/limine/BOOTX64.EFI EFI/BOOT/BOOTX64.EFI | wc -l' \
    '^5$' 0
  vm_until 'lsinitcpio /boot/initramfs-linux.img | grep -c archiso' '^0$' 0
  vm_until "lsinitcpio /boot/initramfs-linux.img | grep -c 'usr/lib/systemd/systemd\$'" '^1$' 0
fi

# Packages: the boot files stay, the installer and what only it needed are gone (on Fedora also the live image's GRUB);
# Fedora keeps SELinux enforcing, with the policy's labels on what the installer wrote
if [ "$like" = fedora ]; then
  vm_until 'rpm -q stackific-boot limine efibootmgr dnf5 | grep -vc "not installed"' '^4$' 0
  vm_until 'rpm -q stackific-installer' '^package stackific-installer is not installed$' 0
  vm_until "rpm -qa 'grub2*' grubby | wc -l" '^0$' 0
  vm_until 'getenforce' '^Enforcing$' 0
  vm_until 'ls -Zd /home/tester' ':user_home_dir_t:' 0
  # (noctalia-greeter's state folder holds files the greeter writes at runtime, labelled by its own domain)
  vm_until 'restorecon -nRv -e /var/lib/noctalia-greeter /etc /home /var/lib | wc -l' '^0$' 0
else
  vm_until 'pacman -Q stackific-boot limine efibootmgr | wc -l' '^3$' 0
  vm_until 'pacman -Q stackific-installer arch-install-scripts 2>/dev/null | wc -l' '^0$' 0
fi

# The system the installer set up, and its login screen
vm_until 'hostnamectl hostname' '^stackific-test$' 0
vm_until 'timedatectl show -P Timezone' '^Europe/Berlin$' 0
vm_until 'localectl status' 'X11 Layout: de$' 0
vm_until 'localectl status' 'VC Keymap: de$' 0
vm_until 'id -nG tester' '(^| )wheel( |$)' 0
vm_until 'systemctl is-active greetd' '^active$' 0
greeter=greeter
[ "$like" != fedora ] || greeter=greetd # Fedora's greetd package names its user greetd
vm_until "pgrep -u $greeter -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
sleep 10 # the greeter draws a few seconds after it starts, on the VM's software renderer
screenshot installed

# stackific-boot's hook (Arch: a pacman hook; Fedora: an RPM trigger): reinstalling limine restores both copies of its
# program, and leaves the firmware entries alone
vm ': >/boot/EFI/limine/BOOTX64.EFI && : >/boot/EFI/BOOT/BOOTX64.EFI' >/dev/null
if [ "$like" = fedora ]; then
  reinstall='rpm -Uvh --replacepkgs /boot/limine-*.rpm >/dev/null && rm /boot/limine-*.rpm && echo reinstalled'
else
  reinstall='pacman -U --noconfirm /boot/limine-*.pkg.tar.zst >/dev/null && rm /boot/limine-*.pkg.tar.zst && echo reinstalled'
fi
vm_until "$reinstall" '^reinstalled$' 0
vm_until "$efi_same" '^same$' 0
[ "$(vm efibootmgr)" = "$entries" ] || fail "reinstalling limine changed the firmware entries"
echo "ok: stackific-boot's hook restored Limine's program"

# Fedora: stackific-boot's kernel-install plugin lists every kernel on the ESP in Limine's menu, newest first, and takes
# a removed one out; reinstalling the kernel package (offline, from the ESP) runs it through the kernel's own scriptlet.
# None of it touches the firmware entries.
if [ "$like" = fedora ]; then
  entry() { printf '%s\n    protocol: linux\n    path: boot():/vmlinuz-%s\n    module_path: boot():/initramfs-%s.img\n    cmdline: root=UUID=%s rootflags=subvol=/@ rw' "$1" "$2" "$2" "$uuid"; }
  one="timeout: 3
interface_branding: $name

/$(entry "$name" "$version")"
  [ "$(vm 'cat /boot/limine.conf')" = "$one" ] || fail "limine.conf after the install: $(vm 'cat /boot/limine.conf')"
  vm_until "kernel-install add 0.0.1-test /boot/vmlinuz-$version /boot/initramfs-$version.img && echo added" '^added$' 0
  [ "$(vm 'cat /boot/limine.conf')" = "$one

/$(entry "$name (0.0.1-test)" 0.0.1-test)" ] || fail "limine.conf with a second kernel: $(vm 'cat /boot/limine.conf')"
  vm_until 'kernel-install remove 0.0.1-test && ls /boot/*0.0.1-test* 2>/dev/null | wc -l' '^0$' 0
  [ "$(vm 'cat /boot/limine.conf')" = "$one" ] || fail "limine.conf after the removal: $(vm 'cat /boot/limine.conf')"
  vm "rm /boot/vmlinuz-$version" >/dev/null
  vm_until "dnf5 reinstall -y --disable-repo='*' /boot/kernel-core-*.rpm >/dev/null && rm /boot/kernel-core-*.rpm && cmp /usr/lib/modules/$version/vmlinuz /boot/vmlinuz-$version && echo same" \
    '^same$' 0
  [ "$(vm 'cat /boot/limine.conf')" = "$one" ] || fail "limine.conf after the kernel's reinstall: $(vm 'cat /boot/limine.conf')"
  [ "$(vm efibootmgr)" = "$entries" ] || fail "kernel-install changed the firmware entries"
  echo "ok: kernel-install keeps Limine's menu current"
fi

# The keyboard layout the installer set, in each desktop: greetd logs `tester` in through an initial session, as the
# live test does (the image ships none), and into GNOME through the greeter. MangoWM reads the user's config.conf, which sources the stock one and sets the
# layout; Plasma takes localed's X11 layout while the user has no kxkbrc layout, and GNOME while the user has no input
# sources.
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
if [ "$like" = fedora ]; then
  # GNOME starts only in a wayland logind session, which the greeter asks greetd for, so `tester` logs in there (GNOME
  # is first in its session list), with a password of letters the US and German layouts type alike
  vm "printf 'tester:stackific\\n' | chpasswd && loginctl terminate-user tester; systemctl restart greetd" >/dev/null
  vm_until "pgrep -u greetd -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
  sleep 15 # the greeter draws a few seconds after it starts, on the VM's software renderer
  sendkey s t a c k i f i c ret
  vm_until 'pgrep -u tester -x gnome-shell' '^[0-9]+$' 180
  vm_until "runuser -u tester -- gsettings get org.gnome.desktop.input-sources sources" "^\\[\\('xkb', 'de'\\)\\]\$" 60
  echo "ok: MangoWM and GNOME start with the installer's layout"
else
  session '/usr/lib/plasma-dbus-run-session-if-needed /usr/bin/startplasma-wayland'
  vm_until 'pgrep -u tester -x plasmashell' '^[0-9]+$' 180
  vm_until "runuser -u tester -- env DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$uid/bus busctl --user call org.kde.keyboard /Layouts org.kde.KeyboardLayouts getLayoutsList" \
    '^a\(sss\) [0-9]+ "de"' 60
  echo "ok: MangoWM and Plasma start with the installer's layout"
fi
