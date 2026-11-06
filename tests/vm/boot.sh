#!/usr/bin/env bash
# Boot test for a live ISO under QEMU with KVM and UEFI firmware, for the host's architecture: x86_64 (q35, OVMF) or
# aarch64 (virt, AAVMF). Runs inside a builder container (`just <base> test`).
# Test access comes only from one-boot SMBIOS credentials, never from the image: they add a token-gated vsock shell
# (stackific-test-shell.service next to this script), which waits for the end of boot and runs the checks. systemd's
# vmm.notify_socket is not used: over vsock it opens one connection per message and drops some, READY=1 included.
# Usage: boot.sh <iso> <out-dir> <checks>. It runs the checks every image passes, then sources <checks>, the image's own
# check file (tests/vm/<base>.sh, or tests/vm/<base>-<arch>.sh for checks only one architecture passes), which uses vm,
# vm_until and screenshot from here. Leaves <out-dir>/serial.log, and the screenshots the check file takes.
# OVMF_CODE and OVMF_VARS name the raw firmware pair in the builder (AAVMF's on aarch64); the defaults are the Arch
# builder's.
set -euo pipefail

iso=$1
out=$2
checks=$3
here=$(dirname "$(readlink -f "$0")")
shell_port=$(sed -n 's/.*--listen=vsock::\([0-9]*\) .*/\1/p' "$here/stackific-test-shell.service")

# The machine and its default firmware for this architecture
case "$(uname -m)" in
  x86_64)
    machine=(qemu-system-x86_64 -machine "q35,accel=kvm" -vga std)
    ovmf_code=${OVMF_CODE:-/usr/share/edk2/x64/OVMF_CODE.4m.fd}
    ovmf_vars=${OVMF_VARS:-/usr/share/edk2/x64/OVMF_VARS.4m.fd}
    ;;
  aarch64)
    # virtio-gpu-pci is the display that works with KVM on virt; tty1 shows once its driver loads
    machine=(qemu-system-aarch64 -machine "virt,accel=kvm" -device virtio-gpu-pci)
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

cp "$ovmf_vars" "$tmp/vars.fd"
rm -f "$out/screen.png" "$out/greeter.png" "$out/serial.log"
"${machine[@]}" -cpu host -smp 2 -m 4096 \
  -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code" \
  -drive "if=pflash,format=raw,file=$tmp/vars.fd" \
  -cdrom "$iso" -nic user,model=virtio-net-pci \
  -device "vhost-vsock-pci,guest-cid=$cid" "${smbios[@]}" \
  -serial "file:$out/serial.log" -display none \
  -monitor "unix:$tmp/monitor,server=on,wait=off" &
qemu=$!

# The shell listens from multi-user.target on; --wait returns once boot has finished, in whatever state it reached
vm_until 'systemctl is-system-running --wait' '^[a-z]+$' 300
vm_until 'systemctl is-system-running' '^running$' 0
vm_until 'nmcli -t -f RUNNING general' '^running$' 30

# A wrong token must get no shell: no output, connection closed
got=$(printf 'bad\nid\n' | socat -t 10 - "VSOCK-CONNECT:$cid:$shell_port" 2>/dev/null) || true
[ -z "$got" ] || fail "wrong token got a shell: '$got'"
echo "ok: wrong token rejected"

# The image's own checks
# shellcheck source=/dev/null
source "$checks"

vm 'systemctl poweroff' >/dev/null 2>&1 || true
for _ in $(seq 60); do kill -0 "$qemu" 2>/dev/null || break; sleep 1; done
kill -0 "$qemu" 2>/dev/null && fail "VM still running 60 s after poweroff"
echo "PASS: $iso"
