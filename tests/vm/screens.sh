# shellcheck shell=bash
# Screenshots of the installer's screens for the prompt trail: boot.sh sources this file with STACKIFIC_TEST_DISK
# attached as /dev/vda, a blank 64 GiB raw image (`just arch screenshots`). It drives the real installer by key presses
# and saves <out-dir>/screens/<name>.png on each screen:
# 1. on the desktop, from Noctalia's launcher (Super+Space): every screen through a free-space install next to a Windows-like table,
#    the stop dialog, and Done;
# 2. on the desktop again: a terminal too small, then an erase stopped during the copy (Stopped);
# 3. on the console, from the Install target: Display, Keyboard and an erase to Done.
# The waits are for the processes (the kernel keeps 15 characters of a name: stackific-insta) and the log; the screens themselves are checked by eye (and by the snapshot tests).
# shellcheck disable=SC2154 # out, vm, vm_until, fail, screenshot and sendkey come from boot.sh, which sources this file

mkdir -p "$out/screens"
log=/run/stackific-installer.log

# Type text as key presses: letters, digits, spaces, / and -
typ() {
  local keys=() c i
  for ((i = 0; i < ${#1}; i++)); do
    c=${1:i:1}
    case $c in
      [a-z0-9]) keys+=("$c") ;;
      [A-Z]) keys+=("shift-${c,,}") ;;
      ' ') keys+=(spc) ;;
      /) keys+=(slash) ;;
      -) keys+=(minus) ;;
      *) fail "typ: no key for '$c'" ;;
    esac
  done
  sendkey "${keys[@]}"
}

# Save the screen as screens/<name>.png once what was pressed has drawn: when two dumps 2 s apart are the same, or after
# 40 s (the copy's progress bar never stands still). MangoWM on the VM's software renderer can leave stale regions on
# screen after small redraws (docs/arch-iso.md), so on the desktop it first repaints everything: workspace 2 (Ctrl+2,
# MangoWM's own key), then back to 1.
desktop=true
shot() {
  local before="" now end=$((SECONDS + 40))
  if $desktop; then
    sendkey ctrl-2 >/dev/null
    sleep 1
    sendkey ctrl-1 >/dev/null
  fi
  sleep 2
  while :; do
    screenshot "screens/$1" >/dev/null
    now=$(md5sum <"$out/screens/$1.png")
    [ "$now" != "$before" ] && [ "$SECONDS" -lt "$end" ] || break
    before=$now
    rm -f "$out/screens/$1.png"
    sleep 2
  done
  echo "ok: $out/screens/$1.png"
}

# Open "Install Stackific OS" from Noctalia's launcher (Super+Space, `live`'s MangoWM config). run0 asks polkit, which
# lets `live` (in wheel, with an empty password) through without a prompt, so the installer comes up by itself.
open_installer() {
  sendkey meta_l-spc
  sleep 3
  [ -z "$1" ] || screenshot "screens/$1"
  typ install
  sleep 4
  sendkey ret
  vm_until 'pgrep -x stackific-insta >/dev/null && echo up' '^up$' 60
}

# Fill You with a name, a user name from it, a password typed twice, and the default computer name
you() {
  typ "$1"
  sendkey tab tab
  typ "$2"
  sendkey tab
  typ "$2"
}

# Wait for an install's end (the log's last step, then the target unmounted)
wait_install() {
  vm_until "grep -c '^==> Remove the live session' $log" '^1$' 1200
  vm_until 'findmnt /run/stackific-installer/target >/dev/null || echo unmounted' '^unmounted$' 300
}

# The disk: a Windows-like table with a large gap, as in tests/vm/install.sh
vm "sfdisk --quiet /dev/vda <<'EOF'
label: gpt
start=2048, size=204800, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B
start=206848, size=32768, type=E3C9E316-0B5C-4DB8-817D-F92DF00215AE
start=4433920, size=20971520, type=EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
5: start=132118528, size=2097152, type=DE94BBA4-06D1-4D40-A16A-BFD50179D6AC
EOF
udevadm settle && mkfs.fat -F 32 -n SYSTEM /dev/vda1 >/dev/null && mkfs.ntfs --quick --label Windows /dev/vda3 >/dev/null" >/dev/null
vm_until 'blkid -o value -s LABEL /dev/vda3' '^Windows$' 0

# 1. The desktop: every screen, a free-space install
vm_until 'systemctl --user -M live@ is-active stackific-noctalia.service' '^active$' 120
sleep 10
open_installer launcher
shot display
sendkey ret
shot welcome
sendkey ret
typ germ
shot keyboard
sendkey ret
typ berl
shot timezone
sendkey ret
you 'Ada Lovelace' abcdefgh
sendkey backspace
shot you
typ h
sendkey ret
shot drive
sendkey ret
shot space
sendkey ret
shot confirm-free-space
sendkey esc down ret
typ vda
shot confirm-erase
sendkey esc up ret
sendkey ret
vm_until 'pgrep -x rsync >/dev/null && echo copying' '^copying$' 300
sleep 20
shot installing
sendkey ctrl-c
shot installing-stop
sendkey esc
wait_install
shot "done"
sendkey ret
vm_until 'pgrep -x stackific-insta || echo gone' '^gone$' 30

# 2. The desktop again: text too big for the window, then an erase stopped during the copy
open_installer ''
# foot grows the text by half a point per press; forty presses leave fewer than 80 × 24 cells on the VM's screen
mapfile -t bigger < <(yes ctrl-equal | head -n 40)
sendkey "${bigger[@]}"
sleep 2
screenshot screens/small
sendkey ctrl-0
sleep 1
sendkey ret ret ret ret
you 'Test User' secret
sendkey ret ret
shot space-refused
sendkey ret
typ vda
sendkey ret
vm_until 'pgrep -x rsync >/dev/null && echo copying' '^copying$' 300
sleep 5
sendkey ctrl-c ret
vm_until 'pgrep -x rsync || echo stopped' '^stopped$' 60
shot stopped
sendkey ret
vm_until 'pgrep -x stackific-insta || echo gone' '^gone$' 30

# 3. The console: the boot menu's Install entry, Display to Done with an erase
desktop=false
vm 'systemctl isolate stackific-installer.target' >/dev/null || true
vm_until 'systemctl is-active stackific-installer-console.service' '^active$' 60
sleep 5
shot display-console
sendkey ret ret
typ germ
shot keyboard-console
sendkey ret ret
you 'Grace Hopper' secret
sendkey ret ret ret
typ vda
shot confirm-console
sendkey ret
vm_until 'pgrep -x rsync >/dev/null && echo copying' '^copying$' 300
sleep 20
shot installing-console
wait_install
shot done-console
sendkey ret
vm_until 'systemctl is-active graphical.target' '^active$' 120
