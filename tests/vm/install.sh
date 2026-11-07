# shellcheck shell=bash
# Install checks, for any base: boot.sh sources this file with STACKIFIC_TEST_DISK attached as /dev/vda, a blank
# 64 GiB raw image (`just arch test-install`). In one boot: the console path, then unattended installs: a wrong erase
# confirmation, an erase, a Windows-like table with a BitLocker refusal, and an install into its free space. Each
# installed system is checked on its partitions. Leaves <out-dir>/esp-offset (the free-space install's ESP, in bytes)
# for the host's FAT check, <out-dir>/windows-like.dump and installer-console.png.
# shellcheck disable=SC2154 # out, vm, vm_until, fail, screenshot and sendkey come from boot.sh, which sources this file

password='stackific-test-password'

# Write the unattended answers to /tmp/<name>.toml in the guest: mode, and for an erase the typed confirmation
answers() {
  vm "cat >/tmp/$1.toml <<'EOF'
[disk]
device = \"/dev/vda\"
mode = \"$2\"
erase = \"$3\"

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

# The installed system on <esp> and <system>: labels, subvolumes, fstab, packages, the user, the system settings, no
# live state left, and the boot files on the ESP
installed_checks() {
  local esp=$1 sys=$2
  vm_until "blkid -o value -s TYPE $esp" '^vfat$' 0
  vm_until "blkid -o value -s LABEL $esp" '^STACKIFIC$' 0
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
  vm_until "grep -cx 'ID=stackific' $r/etc/os-release" '^1$' 0
  local db=$r/var/lib/pacman/local
  vm_until "echo \$(ls -d $db/stackific-release-* | wc -l)/\$(ls -d $db/stackific-installer-* 2>/dev/null | wc -l)" '^1/0$' 0
  vm_until "test ! -e $r/usr/share/applications/stackific-installer.desktop && cat $r/etc/machine-id" '^uninitialized$' 0
  # The user, an administrator through wheel, with a yescrypt hash; root locked
  vm_until "grep -cx 'tester:x:1000:1000:Test User:/home/tester:/usr/bin/bash' $r/etc/passwd" '^1$' 0
  vm_until "grep -E '^wheel:' $r/etc/group | cut -d: -f4 | tr , '\\n' | grep -cx tester" '^1$' 0
  vm_until "grep -c '^tester:[\$]y[\$]' $r/etc/shadow" '^1$' 0
  vm_until "grep -c '^root:!' $r/etc/shadow" '^1$' 0
  vm_until "test -f $h/tester/.config/noctalia/00-stackific.toml && cat $h/tester/.config/mango/config.conf | paste -sd ' '" \
    '^source=/etc/mango/config.conf bind=SUPER,space,spawn,noctalia msg panel-toggle launcher xkb_rules_layout=de$' 0
  vm_until "stat -c %u $h/tester/.config/mango/config.conf $h/tester/.config/mango | paste -sd ' '" '^1000 1000$' 0
  # The system: computer name, timezone, console keymap, X11 layout, the live image's locale
  vm_until "cat $r/etc/hostname" '^stackific-test$' 0
  vm_until "readlink $r/etc/localtime" '^\.\./usr/share/zoneinfo/Europe/Berlin$' 0
  vm_until "grep -cx 'KEYMAP=de' $r/etc/vconsole.conf" '^1$' 0
  vm_until "grep -cx '        Option \"XkbLayout\" \"de\"' $r/etc/X11/xorg.conf.d/00-keyboard.conf" '^1$' 0
  vm_until "grep -cx 'LANG=C.UTF-8' $r/etc/locale.conf" '^1$' 0
  # No live state: no user `live`, no home, no sysusers line, no archiso drop-in, no autologin
  vm_until "cat $r/etc/passwd $r/etc/shadow $r/etc/group | grep -cE '(^|[:,])live(:|,|\$)'" '^0$' 0
  vm_until "test ! -e $h/live -a ! -e $r/etc/sysusers.d/stackific-live.conf -a ! -e $r/etc/mkinitcpio.conf.d/archiso.conf && echo ok" \
    '^ok$' 0
  vm_until "grep -c initial_session $r/etc/greetd/config.toml" '^0$' 0
  vm_until "grep -A2 '^\\[default_session\\]' $r/etc/greetd/config.toml | grep -cx 'command = \"/usr/bin/noctalia-greeter-session\"'" '^1$' 0
  # The ESP: the kernel, its initramfs, Limine's program at both paths, and limine.conf with this install's root
  vm "mkdir -p /mnt/e && mount -o ro $esp /mnt/e" >/dev/null
  vm_until 'cd /mnt/e && ls -d vmlinuz-linux initramfs-linux.img limine.conf EFI/limine/BOOTX64.EFI EFI/BOOT/BOOTX64.EFI | wc -l' \
    '^5$' 0
  vm_until "cmp $r/usr/share/limine/BOOTX64.EFI /mnt/e/EFI/limine/BOOTX64.EFI && cmp $r/usr/share/limine/BOOTX64.EFI /mnt/e/EFI/BOOT/BOOTX64.EFI && echo same" \
    '^same$' 0
  vm_until "grep -cx '    cmdline: root=UUID=$uuid_sys rootflags=subvol=/@ rw' /mnt/e/limine.conf" '^1$' 0
  vm "umount /mnt/e /mnt/t" >/dev/null
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

# 1. A wrong erase confirmation is refused (exit 2) before anything is written
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

# 3. A Windows-like table: an ESP (FAT, SYSTEM), Microsoft reserved, a 2 GiB gap, basic data, a large gap, and a
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
before=$(vm 'sfdisk --dump /dev/vda')
printf '%s\n' "$before" >"$out/windows-like.dump"
[ "$(grep -c '^/dev/vda' <<<"$before")" = 4 ] || fail "the Windows-like table: $before"
echo "ok: the Windows-like table"

# 4. BitLocker on partition 3 (the -FVE-FS- signature at byte 3, after the boot code's jump) refuses the free space
vm "printf '\\353\\122\\220-FVE-FS-' | dd of=/dev/vda3 bs=1 conv=notrunc status=none && udevadm settle" >/dev/null
vm_until 'blkid -p -o value -s TYPE /dev/vda3' '^BitLocker$' 0
answers free free-space ''
vm_until '/usr/bin/stackific-installer --config /tmp/free.toml >/dev/null 2>&1; echo $?' '^2$' 0
[ "$(vm 'sfdisk --dump /dev/vda')" = "$before" ] || fail "the BitLocker refusal changed the table"
vm 'dd if=/dev/zero of=/dev/vda3 bs=11 count=1 conv=notrunc status=none && udevadm settle' >/dev/null
vm_until 'blkid -p -o value -s TYPE /dev/vda3; echo end' '^end$' 0
echo "ok: BitLocker refused the free space"

# 5. Free space: the old entries byte-identical, two new partitions inside the large gap
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
# Nothing written to the Windows-like ESP: it stays as mkfs.fat left it, empty
vm_until 'mkdir -p /mnt/w && mount -o ro /dev/vda1 /mnt/w && ls -A /mnt/w | wc -l; umount /mnt/w' '^0$' 0
installed_checks "$(cut -d' ' -f1 <<<"$esp_line")" "$(cut -d' ' -f1 <<<"$sys_line")"
echo $((esp_start * 512)) >"$out/esp-offset"
