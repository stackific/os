#!/usr/bin/env bash
# Boot test for a live ISO under QEMU with KVM and UEFI firmware, for the host's architecture: x86_64 (q35, OVMF) or
# aarch64 (virt, AAVMF). Runs inside a builder container (`just <base> test`).
# Test access comes only from one-boot SMBIOS credentials, never from the image: they add a token-gated vsock shell
# (stackific-test-shell.service next to this script), which waits for the end of boot and runs the checks. systemd's
# vmm.notify_socket is not used: over vsock it opens one connection per message and drops some, READY=1 included.
# Usage: boot.sh <iso> <out-dir> <checks>. It runs the checks every image passes, then sources <checks>, the image's own
# check file (tests/vm/<base>.sh, or tests/vm/<base>-<arch>.sh for checks only one architecture passes), which uses vm,
# vm_until, vm_put, asset, screenshot and sendkey from here. Leaves <out-dir>/serial.log, <out-dir>/vars.fd (the
# firmware's variables after the run, for a later boot through OVMF_VARS) and the screenshots the check file takes.
# OVMF_CODE and OVMF_VARS name the raw firmware pair in the builder (AAVMF's on aarch64); the defaults are the Arch
# builder's. STACKIFIC_TEST_DISK, when it names a raw image file, is attached as a virtio disk (/dev/vda in the guest).
# <iso> `-` boots that disk alone, an installed system: then <out-dir>/limine.png is its boot menu, taken a second after
# the firmware starts an entry (OVMF prints `BdsDxe: starting Boot#### "<label>"` on the serial console).
# STACKIFIC_TEST_PASSPHRASE, when set, is the installed disk's LUKS2 passphrase, passed as systemd-cryptsetup's
# `cryptsetup.passphrase` credential, so the boots unlock with nobody typing. PREBOOT, when set, names a file sourced
# right after that, before the wait for the test shell (tests/vm/unlock.sh types at the passphrase prompt).
set -euo pipefail

iso=$1
out=$2
checks=$3
here=$(dirname "$(readlink -f "$0")")
shell_port=$(sed -n 's/.*--listen=vsock::\([0-9]*\) .*/\1/p' "$here/stackific-test-shell.service")

# The machine and its default firmware for this architecture, and Limine's EFI program for it (UEFI's removable-media
# name, which the installer and stackific-boot use), for the check files
# shellcheck disable=SC2034 # efi is read by the check files this script sources
case "$(uname -m)" in
  x86_64)
    machine=(qemu-system-x86_64 -machine "q35,accel=kvm" -vga std)
    efi=BOOTX64.EFI
    ovmf_code=${OVMF_CODE:-/usr/share/edk2/x64/OVMF_CODE.4m.fd}
    ovmf_vars=${OVMF_VARS:-/usr/share/edk2/x64/OVMF_VARS.4m.fd}
    ;;
  aarch64)
    # virtio-gpu-pci is the display that works with KVM on virt; tty1 shows once its driver loads. virt has no keyboard
    # of its own, so sendkey gets a USB one, which AAVMF (Limine's menu), the initramfs and the desktop all read.
    machine=(qemu-system-aarch64 -machine "virt,accel=kvm" -device virtio-gpu-pci -device qemu-xhci -device usb-kbd)
    efi=BOOTAA64.EFI
    ovmf_code=${OVMF_CODE:-/usr/share/edk2/aarch64/QEMU_EFI.fd}
    ovmf_vars=${OVMF_VARS:-/usr/share/edk2/aarch64/QEMU_VARS.fd}
    ;;
  *) echo "boot.sh: unsupported architecture $(uname -m)" >&2; exit 2 ;;
esac

tmp=$(mktemp -d)
qemu=""
trap 'kill "$qemu" 2>/dev/null || true; rm -rf "$tmp"' EXIT

# Print the reason and the end of the serial log, then exit non-zero
fail() {
  echo "FAIL: $1" >&2
  tail -n 40 "$out/serial.log" >&2 || true
  exit 1
}

# Pass one SMBIOS type 11 credential: name and file whose contents become the value (base64, so any bytes fit)
cred() {
  smbios+=(-smbios "type=11,value=io.systemd.credential.binary:$1=$(base64 -w0 "$2")")
}

# Run a command in the guest through the test shell: the token line first, then the command
vm() {
  printf '%s\n%s\n' "$token" "$1" | socat -t 30 - "VSOCK-CONNECT:$cid:$shell_port"
}

# Copy the host file <file> to <guest path> through the test shell, as base64 in a here-document
vm_put() {
  vm "base64 -d >'$2' <<'EOF'
$(base64 "$1")
EOF
echo put" | grep -qx put || fail "could not copy $1 to the guest"
}

# Download a pinned test asset from <url> to <out-dir>/<file> unless it is there with <sha256>, then check it
asset() {
  local url=$1 sha=$2 file=$out/$3
  echo "$sha  $file" | sha256sum --check --status 2>/dev/null || curl -fsSLo "$file" "$url"
  echo "$sha  $file" | sha256sum --check --status || fail "$3 does not match its pinned SHA-256"
}

# Retry a command in the guest until its output matches an extended regex, for up to <seconds> (one try at least)
vm_until() {
  local cmd=$1 want=$2 end=$((SECONDS + $3)) got=""
  while :; do
    got=$(vm "$cmd" 2>/dev/null) || true
    grep -qE "$want" <<<"$got" && { echo "ok: $cmd"; return 0; }
    kill -0 "$qemu" 2>/dev/null || fail "QEMU exited"
    [ "$SECONDS" -lt "$end" ] || fail "'$cmd' printed '$got', want /$want/"
    sleep 1
  done
}

# Press keys in the VM, one after the other (QEMU key names, such as esc, ret or shift-a), through the QEMU monitor
sendkey() {
  local k
  for k in "$@"; do
    echo "sendkey $k"
    sleep 0.25
  done | socat -t 5 - "UNIX-CONNECT:$tmp/monitor" >/dev/null
  echo "ok: sendkey $*"
}

# Save the VM's display as <out-dir>/<name>.png through the QEMU monitor
screenshot() {
  echo "screendump $out/$1.png -f png" | socat -t 5 - "UNIX-CONNECT:$tmp/monitor" >/dev/null
  for _ in $(seq 10); do [ -s "$out/$1.png" ] && break; sleep 1; done
  [ -s "$out/$1.png" ] || fail "no screenshot $1.png"
  echo "ok: $out/$1.png"
}

token=$(od -An -N32 -tx1 /dev/urandom | tr -d ' \n')
# The token reaches QEMU through a 0600 file (path=), so it never shows on a process command line
printf 'io.systemd.credential:stackific.test.token=%s' "$token" >"$tmp/token"
chmod 600 "$tmp/token"
cid=$((3 + (RANDOM << 15 | RANDOM)))
printf '[Unit]\nWants=stackific-test-shell.service\n' >"$tmp/multi-user.conf"
smbios=(
  -smbios "type=11,path=$tmp/token"
)
cred systemd.extra-unit.stackific-test-shell.service "$here/stackific-test-shell.service"
cred systemd.unit-dropin.multi-user.target "$tmp/multi-user.conf"
if [ -n "${STACKIFIC_TEST_PASSPHRASE:-}" ]; then
  printf %s "$STACKIFIC_TEST_PASSPHRASE" >"$tmp/passphrase"
  cred cryptsetup.passphrase "$tmp/passphrase"
fi

disk=()
[ -z "${STACKIFIC_TEST_DISK:-}" ] || disk=(-drive "file=$STACKIFIC_TEST_DISK,format=raw,if=virtio")
cdrom=(-cdrom "$iso")
# virt has no IDE, so -cdrom would be a virtio disk ahead of the test disk (/dev/vda); a SCSI CD is /dev/sr0, as on q35
[ "$(uname -m)" != aarch64 ] \
  || cdrom=(-device virtio-scsi-pci -drive "if=none,id=cd,media=cdrom,format=raw,file=$iso" -device "scsi-cd,drive=cd")
[ "$iso" != - ] || cdrom=()
cp "$ovmf_vars" "$tmp/vars.fd"
rm -f "$out/screen.png" "$out/greeter.png" "$out/limine.png" "$out/serial.log"
"${machine[@]}" -cpu host -smp 2 -m 4096 \
  -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code" \
  -drive "if=pflash,format=raw,file=$tmp/vars.fd" \
  "${cdrom[@]}" "${disk[@]}" -nic user,model=virtio-net-pci \
  -device "vhost-vsock-pci,guest-cid=$cid" "${smbios[@]}" \
  -serial "file:$out/serial.log" -display none \
  -monitor "unix:$tmp/monitor,server=on,wait=off" &
qemu=$!

# An installed system's boot menu: Limine waits 3 s, so a shot a second after the firmware starts it catches the menu
if [ "$iso" = - ]; then
  for _ in $(seq 60); do grep -aq 'BdsDxe: starting Boot' "$out/serial.log" 2>/dev/null && break; sleep 1; done
  grep -aq 'BdsDxe: starting Boot' "$out/serial.log" || fail "the firmware started no boot entry"
  sleep 1
  screenshot limine
  # shellcheck source=/dev/null
  [ -z "${PREBOOT:-}" ] || source "$PREBOOT"
fi

# The shell listens from multi-user.target on; --wait returns once boot has finished, in whatever state it reached
vm_until 'systemctl is-system-running --wait' '^[a-z]+$' 300
vm_until 'systemctl is-system-running' '^running$' 0
vm_until 'nmcli -t -f RUNNING general' '^running$' 30

# A wrong token must get no shell: no output, connection closed
got=$(printf 'bad\nid\n' | socat -t 10 - "VSOCK-CONNECT:$cid:$shell_port" 2>/dev/null) || true
[ -z "$got" ] || fail "wrong token got a shell: '$got'"
echo "ok: wrong token rejected"

# The creative apps' one-click install (task 025): the launcher and its script, and none of the apps in the image
vm_until 'test -x /usr/lib/stackific/install-creative-apps && grep -x "Name=Install creative apps" /usr/share/applications/stackific-creative-apps.desktop' \
  '^Name=Install creative apps$' 0
vm_until 'ls /usr/share/applications | grep -c storyteller' '^0$' 0

# The image's own checks
# shellcheck source=/dev/null
source "$checks"

vm 'systemctl poweroff' >/dev/null 2>&1 || true
for _ in $(seq 60); do kill -0 "$qemu" 2>/dev/null || break; sleep 1; done
kill -0 "$qemu" 2>/dev/null && fail "VM still running 60 s after poweroff"
cp "$tmp/vars.fd" "$out/vars.fd"
echo "PASS: $iso"
