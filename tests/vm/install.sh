# shellcheck shell=bash
# Install checks, for any base: boot.sh sources this file with STACKIFIC_TEST_DISK attached as /dev/vda, a blank
# 64 GiB raw image (`just arch test-install`, `just fedora test-install`). In one boot: the console path, then unattended
# installs: a weak password and a wrong erase confirmation refused, an erase, BIOS-mode Windows refused, a Windows-like table with a real BitLocker
# volume (one of cryptsetup's test images, made by Windows, from BITLK_IMAGES_URL and BITLK_IMAGES_SHA256) and a fake
# Windows loader with its firmware entry, the free space refused without the BitLocker confirmation, and an install
# into it with the confirmation. Each installed system is checked on its partitions, its system partition opened with
# the password (LUKS2, btrfs inside). Leaves <out-dir>/esp-offset (the
# free-space install's ESP, in bytes) for the host's FAT check, <out-dir>/windows-like.dump, installer-console.png and
# bitlk-images.tar.xz (kept for the next run).
# shellcheck disable=SC2154 # out, vm, vm_until, fail, screenshot and sendkey come from boot.sh, which sources this file

password=$STACKIFIC_TEST_PASSWORD
# The base, for the few checks that differ: package queries, the kernel's file names, the live image's state
# shellcheck disable=SC2016 # $ID_LIKE belongs to the guest shell
like=$(vm '. /etc/os-release && echo "$ID_LIKE"' | tr -d '\r\n')
# shellcheck disable=SC2016 # $NAME belongs to the guest shell
name=$(vm '. /etc/os-release && echo "$NAME"' | tr -d '\r\n')

# The BitLocker volume: passphrase and recovery passphrase protectors, recovery key ID 64311DEA, 100 MiB
asset "$BITLK_IMAGES_URL" "$BITLK_IMAGES_SHA256" bitlk-images.tar.xz
vm_put "$out/bitlk-images.tar.xz" /tmp/bitlk-images.tar.xz
bitlk='tar -xJOf /tmp/bitlk-images.tar.xz bitlk-images/bitlk-aes-xts-128.img'

# Write the unattended answers to /tmp/<name>.toml in the guest: mode, for an erase the typed confirmation, and the
# BitLocker confirmation
answers() {
  vm "cat >/tmp/$1.toml <<'EOF'
[disk]
device = \"/dev/vda\"
mode = \"$2\"
erase = \"$3\"
bitlocker = \"${4:-}\"

[system]
keyboard = \"de\"
timezone = \"Europe/Berlin\"
hostname = \"stackific-test\"

[user]
full_name = \"Test User\"
name = \"tester\"
password = \"$password\"
EOF" >/dev/null
}

# Run an unattended install as a transient unit (it outlives the test shell's 30 s connection) and wait for its end;
# fail with its journal unless it ends with Result=success
install_run() {
  vm "systemd-run --quiet --unit=stackific-test-$1 --property=RemainAfterExit=yes /usr/bin/stackific-installer --config /tmp/$1.toml" >/dev/null
  vm_until "systemctl show -P SubState stackific-test-$1" '^(exited|failed)$' 1200
  if ! vm "systemctl show -P Result stackific-test-$1" | grep -qx success; then
    vm "journalctl -u stackific-test-$1 --no-pager | tail -n 60" >&2 || true
    fail "the $1 install did not succeed"
  fi
  echo "ok: the $1 install succeeded"
  vm_until "journalctl -u stackific-test-$1 --no-pager | grep -c -- '$password'" '^0$' 0
}

# The installed system on <esp> and <partition>: LUKS2 on the partition with one keyslot, which the password opens,
# btrfs inside it; labels, subvolumes, fstab, packages, the user, the system settings, no live state left, and the boot
# files on the ESP
installed_checks() {
  local esp=$1 part=$2 sys=/dev/mapper/test-root uuid_luks
  vm_until "blkid -o value -s TYPE $esp" '^vfat$' 0
  vm_until "blkid -o value -s LABEL $esp" '^STACKIFIC$' 0
  vm_until "blkid -o value -s TYPE $part" '^crypto_LUKS$' 0
  vm_until "cryptsetup luksDump $part | grep -E '^Version:|: luks2\$' | paste -sd ' '" '^Version:[[:space:]]+2 +0: luks2$' 0
  uuid_luks=$(vm "blkid -o value -s UUID $part" | tr -d '\r\n')
  vm "printf %s '$password' | cryptsetup open --key-file - $part test-root" >/dev/null
  vm_until "blkid -o value -s TYPE $sys" '^btrfs$' 0
  vm_until "blkid -o value -s LABEL $sys" '^stackific$' 0
  vm "mkdir -p /mnt/t && mount -o subvolid=5 $sys /mnt/t" >/dev/null
  local subvols='@ @cache @containers @docker @flatpak @home @libvirt @log @root @srv @swap @tmp'
  # shellcheck disable=SC2016 # $NF belongs to the guest's awk
  vm_until 'btrfs subvolume list /mnt/t | awk '\''$7 == 5 {print $NF}'\'' | sort | paste -sd " "' "^$subvols\$" 0
  vm_until 'btrfs subvolume list /mnt/t | wc -l' '^12$' 0
  vm_until "lsattr -d /mnt/t/@libvirt /mnt/t/@swap | cut -c1-22 | grep -c C" '^2$' 0
  local r=/mnt/t/@ h=/mnt/t/@home
  local uuid_sys uuid_esp
  uuid_sys=$(vm "blkid -o value -s UUID $sys" | tr -d '\r\n')
  uuid_esp=$(vm "blkid -o value -s UUID $esp" | tr -d '\r\n')
  vm_until "grep -c '^UUID=$uuid_sys .* btrfs subvol=/@[a-z]*,noatime,compress=zstd:1 0 0\$' $r/etc/fstab" '^12$' 0
  vm_until "grep -cx 'UUID=$uuid_esp /boot vfat umask=0077 0 2' $r/etc/fstab" '^1$' 0
  vm_until "grep -cx 'luks-$uuid_luks UUID=$uuid_luks none' $r/etc/crypttab" '^1$' 0
  vm_until "grep -cx 'ID=stackific' $r/etc/os-release" '^1$' 0
  # The installer and its launcher are gone; the machine ID is set on first boot (kiwi leaves the file empty)
  if [ "$like" = fedora ]; then
    vm_until "rpm --root=$r -q stackific-release stackific-installer | paste -sd ' '" \
      '^stackific-release-[^ ]+ package stackific-installer is not installed$' 0
    vm_until "rpm --root=$r -qa 'grub2*' | wc -l" '^0$' 0
    vm_until "test ! -e $r/usr/share/applications/stackific-installer.desktop && wc -c <$r/etc/machine-id" '^0$' 0
    vm_until "cat $r/etc/kernel/cmdline" "^rd.luks.uuid=$uuid_luks root=/dev/mapper/luks-$uuid_luks rootflags=subvol=/@ rw quiet\$" 0
  else
    local db=$r/var/lib/pacman/local
    vm_until "echo \$(ls -d $db/stackific-release-* | wc -l)/\$(ls -d $db/stackific-installer-* 2>/dev/null | wc -l)" '^1/0$' 0
    vm_until "test ! -e $r/usr/share/applications/stackific-installer.desktop && cat $r/etc/machine-id" '^uninitialized$' 0
  fi
  # The user, an administrator through wheel, with a yescrypt hash; root locked
  vm_until "grep -cx 'tester:x:1000:1000:Test User:/home/tester:/usr/bin/bash' $r/etc/passwd" '^1$' 0
  vm_until "grep -E '^wheel:' $r/etc/group | cut -d: -f4 | tr , '\\n' | grep -cx tester" '^1$' 0
  vm_until "grep -c '^tester:[\$]y[\$]' $r/etc/shadow" '^1$' 0
  vm_until "grep -c '^root:!' $r/etc/shadow" '^1$' 0
  vm_until "test -f $h/tester/.config/noctalia/00-stackific.toml && cat $h/tester/.config/mango/config.conf | paste -sd ' '" \
    '^source=/etc/mango/config.conf source_optional=/run/stackific/mango-gpu.conf bind=SUPER,space,spawn,noctalia msg panel-toggle launcher xkb_rules_layout=de$' 0
  vm_until "stat -c %u $h/tester/.config/mango/config.conf $h/tester/.config/mango | paste -sd ' '" '^1000 1000$' 0
  # The system: computer name, timezone, console keymap, X11 layout, the live image's locale
  vm_until "cat $r/etc/hostname" '^stackific-test$' 0
  vm_until "readlink $r/etc/localtime" '^\.\./usr/share/zoneinfo/Europe/Berlin$' 0
  vm_until "grep -cx 'KEYMAP=de' $r/etc/vconsole.conf" '^1$' 0
  vm_until "grep -cx '        Option \"XkbLayout\" \"de\"' $r/etc/X11/xorg.conf.d/00-keyboard.conf" '^1$' 0
  vm_until "cmp $r/etc/locale.conf /etc/locale.conf && echo same" '^same$' 0
  # No live state: no user `live`, no home, no sysusers line, no archiso drop-in, no autologin
  vm_until "cat $r/etc/passwd $r/etc/shadow $r/etc/group | grep -cE '(^|[:,])live(:|,|\$)'" '^0$' 0
  vm_until "test ! -e $h/live -a ! -e $r/etc/sysusers.d/stackific-live.conf -a ! -e $r/etc/mkinitcpio.conf.d/archiso.conf && echo ok" \
    '^ok$' 0
  vm_until "grep -c initial_session $r/etc/greetd/config.toml" '^0$' 0
  vm_until "grep -A2 '^\\[default_session\\]' $r/etc/greetd/config.toml | grep -cx 'command = \"/usr/bin/noctalia-greeter-session\"'" '^1$' 0
  # The ESP: the kernel, its initramfs, Limine's program at both paths, and limine.conf with this install's root
  vm "mkdir -p /mnt/e && mount -o ro $esp /mnt/e" >/dev/null
  # Arch x86_64 also boots linux-lts, the fallback kernel, from its own entry with the same command line
  local kernel=vmlinuz-linux initramfs=initramfs-linux.img entries=2
  [ "$efi" != BOOTAA64.EFI ] || kernel=Image entries=1 # Arch Linux ARM's linux-aarch64 owns /boot/Image
  if [ "$like" = fedora ]; then
    local version
    version=$(vm "ls $r/usr/lib/modules" | tr -d '\r\n')
    kernel=vmlinuz-$version initramfs=initramfs-$version.img entries=1
  fi
  vm_until "cd /mnt/e && ls -d $kernel $initramfs limine.conf EFI/limine/$efi EFI/BOOT/$efi | wc -l" \
    '^5$' 0
  vm_until "cmp $r/usr/share/limine/$efi /mnt/e/EFI/limine/$efi && cmp $r/usr/share/limine/$efi /mnt/e/EFI/BOOT/$efi && echo same" \
    '^same$' 0
  vm_until "grep -cx '    cmdline: rd.luks.uuid=$uuid_luks root=/dev/mapper/luks-$uuid_luks rootflags=subvol=/@ rw quiet' /mnt/e/limine.conf" \
    "^$entries\$" 0
  vm "umount /mnt/e /mnt/t && cryptsetup close test-root" >/dev/null
}

# 0. The console path: the Install entry's target runs the installer on tty1 without the greeter; Esc on Display
#    leaves it, and the unit starts the live desktop again
vm 'systemctl isolate stackific-installer.target' >/dev/null || true
vm_until 'systemctl is-active stackific-installer-console.service' '^active$' 60
vm_until 'systemctl is-active greetd' '^inactive$' 30
sleep 5 # the installer draws Display after setfont
screenshot installer-console
sendkey esc
sleep 1
sendkey esc
vm_until 'systemctl is-active stackific-installer-console.service' '^inactive$' 30
vm_until 'systemctl is-active graphical.target' '^active$' 120

# 1. A password too easy to guess and a wrong erase confirmation are refused (exit 2) before anything is written
strong=$password
password=password123
answers weak erase vda
password=$strong
vm_until '/usr/bin/stackific-installer --config /tmp/weak.toml 2>&1 >/dev/null' 'weak.toml: password: Too easy to guess' 0
vm_until '/usr/bin/stackific-installer --config /tmp/weak.toml >/dev/null 2>&1; echo $?' '^2$' 0
answers wrong erase vdb
vm_until '/usr/bin/stackific-installer --config /tmp/wrong.toml >/dev/null 2>&1; echo $?' '^2$' 0
vm_until 'sfdisk --json /dev/vda >/dev/null 2>&1; echo $?' '^1$' 0

# 2. Erase: a new GPT with our two partitions, the ESP at 1 MiB, the system partition to the end
answers erase erase vda
install_run erase
dump=$(vm 'sfdisk --dump /dev/vda')
grep -qx 'label: gpt' <<<"$dump" || fail "no GPT after the erase: $dump"
[ "$(grep -c '^/dev/vda' <<<"$dump")" = 2 ] || fail "not two partitions after the erase: $dump"
grep -qx '/dev/vda1 : start=        2048, size=     8388608, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, uuid=[0-9A-F-]*, name="Stackific OS boot"' <<<"$dump" \
  || fail "partition 1 after the erase: $dump"
grep -qx '/dev/vda2 : start=     8390656, size=   125825024, type=0FC63DAF-8483-4772-8E79-3D69D8477DE4, uuid=[0-9A-F-]*, name="Stackific OS"' <<<"$dump" \
  || fail "partition 2 after the erase: $dump"
echo "ok: the erase made the two partitions"
installed_checks /dev/vda1 /dev/vda2

# 3. BIOS-mode Windows: an MBR table with a BitLocker volume in a type 7 partition refuses the free space (exit 2),
#    naming MBR2GPT.EXE, and changes nothing
vm "wipefs --all --quiet /dev/vda && printf 'label: dos\\nstart=2048, size=2097152, type=7\\n' | sfdisk --quiet /dev/vda \
  && udevadm settle && $bitlk | dd of=/dev/vda1 bs=1M conv=fsync status=none" >/dev/null
vm_until 'udevadm settle; lsblk -no FSTYPE /dev/vda1' '^BitLocker$' 30
mbr=$(vm 'sfdisk --dump /dev/vda')
answers mbr free-space ''
vm_until '/usr/bin/stackific-installer --config /tmp/mbr.toml >/dev/null 2>/tmp/mbr.err; echo $?' '^2$' 0
vm_until 'grep -c MBR2GPT /tmp/mbr.err' '^1$' 0
[ "$(vm 'sfdisk --dump /dev/vda')" = "$mbr" ] || fail "the BIOS-mode refusal changed the table"
echo "ok: BIOS-mode Windows refused the free space"

# 4. A Windows-like table: an ESP (FAT, SYSTEM), Microsoft reserved, a 2 GiB gap, basic data, a large gap, and a
#    recovery partition at the end, numbered 1, 2, 3 and 5
vm "wipefs --all --quiet /dev/vda && sfdisk --quiet /dev/vda <<'EOF'
label: gpt
start=2048, size=204800, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B
start=206848, size=32768, type=E3C9E316-0B5C-4DB8-817D-F92DF00215AE
start=4433920, size=20971520, type=EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
5: start=132118528, size=2097152, type=DE94BBA4-06D1-4D40-A16A-BFD50179D6AC
EOF
udevadm settle && mkfs.fat -F 32 -n SYSTEM /dev/vda1 >/dev/null" >/dev/null
vm_until 'blkid -o value -s LABEL /dev/vda1' '^SYSTEM$' 0
old_esp=$(vm 'blkid -o value -s UUID /dev/vda1' | tr -d '\r\n')
win_uuid=$(vm 'lsblk -no PARTUUID /dev/vda1' | tr -d '\r\n')
# A fake Windows: Limine's program as Windows' loader with an empty BCD beside it, and the firmware's "Windows Boot
# Manager" entry for it, which efibootmgr puts first in the boot order
vm "mkdir -p /mnt/w && mount /dev/vda1 /mnt/w && mkdir -p /mnt/w/EFI/Microsoft/Boot \
  && cp /usr/share/limine/$efi /mnt/w/EFI/Microsoft/Boot/bootmgfw.efi && : >/mnt/w/EFI/Microsoft/Boot/BCD \
  && umount /mnt/w && efibootmgr --quiet --create --disk /dev/vda --part 1 --label 'Windows Boot Manager' \
  --loader '\\EFI\\Microsoft\\Boot\\bootmgfw.efi'" >/dev/null
win_files='mount -o ro /dev/vda1 /mnt/w && (cd /mnt/w && find . -type f -exec sha256sum {} + | sort); umount /mnt/w'
win_before=$(vm "$win_files")
grep -q ' ./EFI/Microsoft/Boot/bootmgfw.efi$' <<<"$win_before" || fail "no fake Windows loader: $win_before"
vm_until 'efibootmgr' '^Boot[0-9A-F]{4}\* Windows Boot Manager	' 0
before=$(vm 'sfdisk --dump /dev/vda')
printf '%s\n' "$before" >"$out/windows-like.dump"
[ "$(grep -c '^/dev/vda' <<<"$before")" = 4 ] || fail "the Windows-like table: $before"
echo "ok: the Windows-like table"

# 5. BitLocker on partition 3: without the typed confirmation the run exits 2, naming the volume and its recovery key,
#    and changes nothing
vm "$bitlk | dd of=/dev/vda3 bs=1M conv=fsync status=none" >/dev/null
vm_until 'blkid -p -o value -s TYPE /dev/vda3' '^BitLocker$' 0
answers free free-space ''
vm_until '/usr/bin/stackific-installer --config /tmp/free.toml >/dev/null 2>/tmp/free.err; echo $?' '^2$' 0
vm_until "grep -c 'vda3 (10.0 GiB): .*recovery key 64311DEA' /tmp/free.err" '^1$' 0
[ "$(vm 'sfdisk --dump /dev/vda')" = "$before" ] || fail "the BitLocker refusal changed the table"
echo "ok: BitLocker needs the typed confirmation"

# 6. Free space, with the BitLocker confirmation: the old entries byte-identical, two new partitions inside the large
#    gap, the BitLocker volume and Windows' ESP untouched, Windows' firmware entry kept after ours, and Limine's menu
#    with both Windows entries
answers free free-space '' 'I have my key'
install_run free
after=$(vm 'sfdisk --dump /dev/vda')
for line in "$(grep '^label-id:' <<<"$before")" $(grep -n '^/dev/vda' <<<"$before" | cut -d: -f1); do
  case "$line" in
    label-id:*) want=$line ;;
    *) want=$(sed -n "${line}p" <<<"$before") ;;
  esac
  grep -qxF -- "$want" <<<"$after" || fail "changed after the free-space install: $want"
done
echo "ok: the old entries are byte-identical"
new=$(grep '^/dev/vda' <<<"$after" | grep -vxF -f <(grep '^/dev/vda' <<<"$before"))
[ "$(wc -l <<<"$new")" = 2 ] || fail "not two new partitions: $new"
field() { sed -n "s/.*$1= *\\([^,]*\\).*/\\1/p" <<<"$2"; }
esp_line=$(grep 'name="Stackific OS boot"' <<<"$new")
sys_line=$(grep 'name="Stackific OS"$' <<<"$new")
esp_start=$(field start "$esp_line")
sys_start=$(field start "$sys_line")
sys_size=$(field size "$sys_line")
if [ $((esp_start % 2048)) != 0 ] || [ $((sys_start % 2048)) != 0 ]; then fail "new partitions off the 1 MiB grid: $new"; fi
if [ "$esp_start" -lt 25405440 ] || [ $((sys_start + sys_size)) != 132118528 ]; then fail "new partitions outside the gap: $new"; fi
if [ "$(field size "$esp_line")" != 8388608 ] || ! grep -q 'type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B' <<<"$esp_line" \
  || ! grep -q 'type=0FC63DAF-8483-4772-8E79-3D69D8477DE4' <<<"$sys_line"; then
  fail "new partitions: $new"
fi
echo "ok: two new partitions fill the gap"
vm_until "blkid -o value -s UUID /dev/vda1" "^$old_esp\$" 0
vm_until 'blkid -o value -s LABEL /dev/vda1' '^SYSTEM$' 0
[ "$(vm "$win_files")" = "$win_before" ] || fail "the install changed Windows' ESP: $(vm "$win_files")"
vm_until "cmp <($bitlk) <(head -c 104857600 /dev/vda3) && echo same" '^same$' 0
echo "ok: Windows' ESP and the BitLocker volume are untouched"
entries=$(vm efibootmgr)
order=$(sed -n 's/^BootOrder: //p' <<<"$entries")
grep -q "^Boot${order%%,*}\\* $name	" <<<"$entries" || fail "\"$name\" is not first: $entries"
grep -q "^Boot$(cut -d, -f2 <<<"$order")\\* Windows Boot Manager	" <<<"$entries" || fail "Windows Boot Manager is not second: $entries"
echo "ok: the firmware starts \"$name\" first and keeps Windows Boot Manager after it"
installed_checks "$(cut -d' ' -f1 <<<"$esp_line")" "$(cut -d' ' -f1 <<<"$sys_line")"
vm "mount -o ro $(cut -d' ' -f1 <<<"$esp_line") /mnt/e" >/dev/null
vm_until "grep -cx -e '/Windows' -e '    entry: Windows Boot Manager' -e '/Windows, started directly' \
  -e '    path: guid($win_uuid):/EFI/Microsoft/Boot/bootmgfw.efi' /mnt/e/limine.conf" '^4$' 0
vm 'umount /mnt/e' >/dev/null
echo $((esp_start * 512)) >"$out/esp-offset"
