# shellcheck shell=bash
# Checks on an installed system booted from its disk alone: the second boot of `just arch test-install` and
# `just fedora test-install` (boot.sh with `-` for the ISO and the first boot's firmware variables), after
# tests/vm/install.sh's free-space install, which is the one on the disk. The firmware started our entry; Limine started
# the kernel from our ESP with root on @; the boot files, the firmware entries and stackific-boot's hooks (the recipe put
# the image's limine package, and on Fedora its kernel-core package, at the ESP's root); the system the installer set
# up; the keyboard layout in MangoWM and in the second desktop (Plasma, GNOME); on Arch x86_64 the LTS kernel's entry,
# one restart; and, last, the fake Windows install.sh
# left on partition 1 started through both of Limine's Windows entries, one restart each. Leaves limine.png (boot.sh),
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
second=$(sed -n 's/^BootOrder: //p' <<<"$entries" | cut -d, -f2)
grep -q "^Boot$second\\* Windows Boot Manager	" <<<"$entries" || fail "Windows Boot Manager is not second: $entries"
echo "ok: Windows Boot Manager is second"

# Limine started the kernel with root on the @ subvolume inside the LUKS2 container, which the initramfs unlocked with
# the passphrase credential (boot.sh); the btrfs default stays the top level
luks=$(vm "findmnt -no SOURCE / | sed -n 's|^/dev/mapper/luks-\\(.*\\)\\[/@\\]\$|\\1|p'" | tr -d '\r\n')
[ -n "$luks" ] || fail "/ is not on the LUKS2 container: $(vm 'findmnt -no SOURCE /')"
cmdline="rd.luks.uuid=$luks root=/dev/mapper/luks-$luks rootflags=subvol=/@ rw quiet"
[ "$efi" != BOOTAA64.EFI ] || cmdline+=" console=tty0" # the screen, not SPCR's serial port (installer's boot::CONSOLE)
vm_until 'cat /proc/cmdline' "(^| )$cmdline( |\$)" 0
vm_until "blkid -o value -s UUID \$(cryptsetup status luks-$luks | sed -n 's/^  device: *//p')" "^$luks\$" 0
vm_until "cryptsetup status luks-$luks" '^  type: +LUKS2$' 0
vm_until 'findmnt -no FSTYPE,OPTIONS /' '^btrfs .*subvol=/@(,|$)' 0
vm_until 'findmnt -no FSTYPE /boot' '^vfat$' 0
vm_until 'btrfs subvolume get-default /' '^ID 5 \(FS_TREE\)$' 0

# The ESP: the kernel and its initramfs (no fallback or rescue image, no unified kernel image), limine.conf, and
# Limine's program at the firmware entry's path and the fallback path; the initramfs is the base's own, with no live
# image's hooks, with the unlock and the boot-failure screen. Arch: mkinitcpio's preset with Arch's default hooks plus
# the installer's sd-encrypt and stackific-boot-failure. Fedora: kernel-install with Fedora's dracut, the
# kernel's version in the names, no GRUB.
efi_same="cmp /usr/share/limine/$efi /boot/EFI/limine/$efi && cmp /usr/share/limine/$efi /boot/EFI/BOOT/$efi && echo same"
vm_until "$efi_same" '^same$' 0
# (systemd-boot-random-seed.service writes /boot/loader/random-seed on any ESP; Boot Loader Spec entries would be in
# loader/entries)
vm_until 'ls -d /boot/*fallback* /boot/*rescue* /boot/EFI/Linux /boot/loader/entries /boot/grub2 2>/dev/null | wc -l' '^0$' 0
# shellcheck disable=SC2016 # $(…) belongs to the guest shell
vm_until 'ls /usr/lib/modules | grep -cx "$(uname -r)"' '^1$' 0
if [ "$like" = fedora ]; then
  vm_until "cd /boot && ls -d vmlinuz-$version initramfs-$version.img limine.conf EFI/limine/$efi EFI/BOOT/$efi | wc -l" \
    '^5$' 0
  vm_until "lsinitrd /boot/initramfs-$version.img | grep -c dmsquash" '^0$' 0
  vm_until "lsinitrd -m /boot/initramfs-$version.img | grep -cx btrfs" '^1$' 0
  # No NVIDIA or nouveau module in the initramfs (RPM Fusion's dracut drop-in and stackific-hardware's, task 020), and
  # on x86_64 NVIDIA's open modules for the running kernel, with no RPM Fusion repo file (aarch64: task 022)
  if [ "$efi" = BOOTX64.EFI ]; then
    vm_until 'rpm -q "kmod-nvidia-$(uname -r)" >/dev/null && ls /etc/yum.repos.d/ /usr/share/dnf5/repos.d/ 2>/dev/null | grep -ci rpmfusion' \
      '^0$' 0
  fi
  vm_until "lsinitrd /boot/initramfs-$version.img | grep -cE '/(nvidia|nouveau)[^/]*\\.ko'" '^0$' 0
  # The unlock (dracut's systemd-cryptsetup) and stackific-boot's plain screen after a failed start
  vm_until "lsinitrd -m /boot/initramfs-$version.img | grep -cx -e systemd-cryptsetup -e stackific-boot-failure" '^2$' 0
  vm_until "cmp /usr/lib/modules/$version/vmlinuz /boot/vmlinuz-$version && echo same" '^same$' 0
else
  # Arch Linux ARM: linux-aarch64's own /boot/Image, and no copy left by the live image's hook
  kernel=vmlinuz-linux
  [ "$efi" != BOOTAA64.EFI ] || kernel=Image
  vm_until "cd /boot && ls -d $kernel initramfs-linux.img limine.conf EFI/limine/$efi EFI/BOOT/$efi | wc -l" '^5$' 0
  vm_until "grep -cx '    path: boot():/$kernel' /boot/limine.conf" '^1$' 0
  vm_until 'ls /etc/pacman.d/hooks/stackific-vmlinuz.hook 2>/dev/null | wc -l' '^0$' 0
  vm_until 'lsinitcpio /boot/initramfs-linux.img | grep -c archiso' '^0$' 0
  vm_until "lsinitcpio /boot/initramfs-linux.img | grep -c 'usr/lib/systemd/systemd\$'" '^1$' 0
  # The unlock (sd-encrypt) and stackific-boot's plain screen after a failed start
  vm_until "lsinitcpio /boot/initramfs-linux.img | grep -cE 'usr/lib/systemd/systemd-cryptsetup\$|etc/systemd/system/emergency.service.d/stackific-boot-failure.conf\$'" \
    '^2$' 0
  # No NVIDIA or nouveau module in any initramfs (no kms hook, tasks 019 and 022: the driver loads from the root file
  # system), and NVIDIA's open module for the running kernel (aarch64: built by DKMS, task 022)
  vm_until 'for i in /boot/initramfs-linux*.img; do lsinitcpio "$i"; done | grep -cE "/(nvidia|nouveau)[^/]*\.ko"' '^0$' 0
  # shellcheck disable=SC2016 # $(…) belongs to the guest shell
  vm_until '[ "$(modinfo -F vermagic nvidia | cut -d" " -f1)" = "$(uname -r)" ] && echo ok' '^ok$' 0
  if [ "$efi" = BOOTX64.EFI ]; then
    # The fallback kernel (task 019): linux-lts beside linux, from its own Limine entry after the default
    vm_until 'cd /boot && ls -d vmlinuz-linux-lts initramfs-linux-lts.img | wc -l' '^2$' 0
    vm_until "grep -E '^/' /boot/limine.conf | head -2 | paste -sd '|'" "^/$name\\|/$name \\(LTS kernel\\)\$" 0
    vm_until "grep -cx -e '    path: boot():/vmlinuz-linux-lts' -e '    module_path: boot():/initramfs-linux-lts.img' /boot/limine.conf" \
      '^2$' 0
  fi
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
vm ": >/boot/EFI/limine/$efi && : >/boot/EFI/BOOT/$efi" >/dev/null
if [ "$like" = fedora ]; then
  reinstall='rpm -Uvh --replacepkgs /boot/limine-*.rpm >/dev/null && rm /boot/limine-*.rpm && echo reinstalled'
else
  # (Arch Linux ARM's packages are .pkg.tar.xz, Arch's .pkg.tar.zst)
  reinstall='pacman -U --noconfirm /boot/limine-*.pkg.tar.* >/dev/null && rm /boot/limine-*.pkg.tar.* && echo reinstalled'
fi
vm_until "$reinstall" '^reinstalled$' 0
vm_until "$efi_same" '^same$' 0
[ "$(vm efibootmgr)" = "$entries" ] || fail "reinstalling limine changed the firmware entries"
echo "ok: stackific-boot's hook restored Limine's program"

# Fedora: stackific-boot's kernel-install plugin lists every kernel on the ESP in Limine's menu, newest first, and takes
# a removed one out; reinstalling the kernel package (offline, from the ESP) runs it through the kernel's own scriptlet.
# None of it touches the firmware entries.
if [ "$like" = fedora ]; then
  # On aarch64 an EFI program: Fedora's zboot kernel loads its initramfs from initrd= (90-stackific-limine.install)
  if [ "$efi" = BOOTAA64.EFI ]; then
    entry() { printf '%s\n    protocol: efi\n    path: boot():/vmlinuz-%s\n    cmdline: initrd=\\initramfs-%s.img %s' "$1" "$2" "$2" "$cmdline"; }
  else
    entry() { printf '%s\n    protocol: linux\n    path: boot():/vmlinuz-%s\n    module_path: boot():/initramfs-%s.img\n    cmdline: %s' "$1" "$2" "$2" "$cmdline"; }
  fi
  # The Windows entries the installer left for the plugin, after the kernels' each time
  win=$(vm 'cat /etc/kernel/limine-windows.conf')
  grep -qx '    entry: Windows Boot Manager' <<<"$win" || fail "no Windows entries for the plugin: $win"
  one="timeout: 3
interface_branding: $name

/$(entry "$name" "$version")
$win"
  [ "$(vm 'cat /boot/limine.conf')" = "$one" ] || fail "limine.conf after the install: $(vm 'cat /boot/limine.conf')"
  vm_until "kernel-install add 0.0.1-test /boot/vmlinuz-$version /boot/initramfs-$version.img && echo added" '^added$' 0
  [ "$(vm 'cat /boot/limine.conf')" = "${one%"$win"}
/$(entry "$name (0.0.1-test)" 0.0.1-test)
$win" ] || fail "limine.conf with a second kernel: $(vm 'cat /boot/limine.conf')"
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
# Run a command as tester on tester's session bus
as_tester() { printf 'runuser -u tester -- env DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/%s/bus %s' "$uid" "$1"; }
# The Secret Service after a login through the greeter with the password: oo7-daemon owns the name, pam_oo7 unlocked
# (or made) the login keyring with the password, and a secret stored with secret-tool reads back, with no prompt
secrets() {
  vm_until "$(as_tester 'busctl --user status org.freedesktop.secrets') | grep -E '^Comm='" '^Comm=oo7-daemon$' 60
  vm_until "$(as_tester "sh -c 'printf stackific-secret | timeout 20 secret-tool store --label=test stackific test && timeout 20 secret-tool lookup stackific test'")" \
    '^stackific-secret$' 0
  # (oo7 0.7, Fedora's, keeps it under keyrings/v1/)
  vm_until 'find /home/tester/.local/share/keyrings -name "*.keyring"' '\.keyring$' 0
  echo "ok: the keyring is unlocked at login and keeps a secret"
}
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
  secrets
else
  session '/usr/lib/plasma-dbus-run-session-if-needed /usr/bin/startplasma-wayland'
  vm_until 'pgrep -u tester -x plasmashell' '^[0-9]+$' 180
  vm_until "runuser -u tester -- env DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/$uid/bus busctl --user call org.kde.keyboard /Layouts org.kde.KeyboardLayouts getLayoutsList" \
    '^a\(sss\) [0-9]+ "de"' 60
  echo "ok: MangoWM and Plasma start with the installer's layout"
  # oo7, not KWallet's ksecretd, is the Secret Service in Plasma too
  vm_until "$(as_tester 'busctl --user status org.freedesktop.secrets') | grep -E '^Comm='" '^Comm=oo7-daemon$' 60
  # A login through the greeter with a password (stackific-greetd's pam_oo7 gets it), letters the US and German
  # layouts type alike, into the greeter's first session
  vm "sed -i '/^\\[initial_session\\]/,\$d' /etc/greetd/config.toml && printf 'tester:stackific\\n' | chpasswd \
    && loginctl terminate-user tester; systemctl restart greetd" >/dev/null
  vm_until "pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |\$)'" '^[0-9]+$' 60
  sleep 15 # the greeter draws a few seconds after it starts, on the VM's software renderer
  sendkey s t a c k i f i c ret
  vm_until 'pgrep -u tester -x "mango|plasmashell"' '^[0-9]+$' 180
  secrets
fi

# Limine's Windows entries start the fake Windows: a limine.conf beside its loader (Limine looks in its program's
# folder first) starts this system's kernel from our ESP with a marker. "Windows" restarts into the firmware's Windows
# Boot Manager entry (BootNext); "Windows, started directly" starts the loader from Limine. One restart each.
esp_uuid=$(vm 'findmnt -no PARTUUID /boot' | tr -d '\r\n')
vm "mkdir -p /mnt/w && mount /dev/vda1 /mnt/w && { printf 'timeout: 1\n\n/Fake Windows\n'; \
  sed -n '/^    /{p;/^    cmdline: /q}' /boot/limine.conf | sed -e 's|boot():|guid($esp_uuid):|' \
  -e '/^    cmdline: /s/\$/ stackific.test=windows/'; } >/mnt/w/EFI/Microsoft/Boot/limine.conf && umount /mnt/w" >/dev/null
# Restart, wait for the firmware to start an entry, then pick Limine's entry <n> below the first: the first key stops
# its 3 s countdown, so the wait polls often and the keys go a second after the firmware's line
limine_pick() {
  local seen keys=()
  seen=$(grep -ac 'BdsDxe: starting Boot' "$out/serial.log")
  vm 'systemctl reboot' >/dev/null 2>&1 || true
  for _ in $(seq 600); do [ "$(grep -ac 'BdsDxe: starting Boot' "$out/serial.log")" -gt "$seen" ] && break; sleep 0.2; done
  sleep 1
  for _ in $(seq "$1"); do keys+=(down); done
  sendkey "${keys[@]}" ret
}
current() { vm efibootmgr | sed -n 's/^BootCurrent: //p'; }
# Arch x86_64's second entry starts linux-lts, the fallback kernel (task 019)
if [ "$efi" = BOOTX64.EFI ] && [ "$like" != fedora ]; then
  limine_pick 1
  vm_until 'uname -r' 'lts$' 300
  echo "ok: \"$name (LTS kernel)\" starts linux-lts"
fi
# The Windows entries follow the system's entries
win=$(( $(vm "grep '^/' /boot/limine.conf | grep -nx /Windows | cut -d: -f1" | tr -d '\r\n') - 1 ))
limine_pick "$win"
vm_until 'cat /proc/cmdline' '(^| )stackific.test=windows$' 300
grep -aq 'BdsDxe: starting Boot[0-9A-F]\{4\} "Windows Boot Manager"' "$out/serial.log" || fail "the firmware did not start Windows Boot Manager"
vm efibootmgr | grep -q "^Boot$(current)\\* Windows Boot Manager	" || fail "BootCurrent is not Windows Boot Manager: $(vm efibootmgr)"
echo "ok: \"Windows\" restarts into the firmware's Windows Boot Manager"
limine_pick $((win + 1))
vm_until 'cat /proc/cmdline' '(^| )stackific.test=windows$' 300
vm efibootmgr | grep -q "^Boot$(current)\\* $name	" || fail "BootCurrent is not \"$name\": $(vm efibootmgr)"
echo "ok: \"Windows, started directly\" starts Windows' loader from Limine"
