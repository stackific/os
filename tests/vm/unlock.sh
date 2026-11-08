# shellcheck shell=bash
# The passphrase prompt of an installed, encrypted system, typed as a person would: boot.sh sources this file as
# PREBOOT when it boots the installed disk alone without the passphrase credential (the last boot of
# `just arch test-install` and `just fedora test-install`, after tests/vm/install.sh's free-space install). Three wrong
# passphrases must leave the plain boot-failure screen (stackific-boot) with no restart, Enter must restart, and the
# right passphrase, typed on the German layout the install set, must start the system, which boot.sh then sees through
# its test shell. Leaves luks-prompt.png and boot-failure.png.
# shellcheck disable=SC2154 # out, fail, screenshot and sendkey come from boot.sh, which sources this file

# The firmware's entry starts, Limine waits 3 s, then the kernel and the initramfs reach the prompt: on KVM well
# within the wait below, and keys typed before the prompt would be lost
prompt_wait=30
starts() { grep -ac 'BdsDxe: starting Boot' "$out/serial.log"; }
# Type a passphrase and Enter: QEMU names keys by their US positions, so on the German layout the z key types y
type_passphrase() {
  local keys=() ch
  while IFS= read -r -n1 ch; do
    case "$ch" in
      ' ') keys+=(spc) ;;
      y) keys+=(z) ;;
      z) keys+=(y) ;;
      *) keys+=("$ch") ;;
    esac
  done < <(printf %s "$1")
  sendkey "${keys[@]}" ret
}

sleep "$prompt_wait"
screenshot luks-prompt
for _ in 1 2 3; do
  type_passphrase wrong
  sleep 6 # each try runs argon2id for about 2 s before the next prompt
done
sleep 10
screenshot boot-failure
seen=$(starts)
sleep 10
[ "$(starts)" = "$seen" ] || fail "the system restarted by itself after three wrong passphrases"
echo "ok: three wrong passphrases leave the boot-failure screen, with no restart"
sendkey ret
for _ in $(seq 60); do [ "$(starts)" -gt "$seen" ] && break; sleep 1; done
[ "$(starts)" -gt "$seen" ] || fail "Enter on the boot-failure screen did not restart"
echo "ok: Enter restarts"
sleep "$prompt_wait"
type_passphrase "$STACKIFIC_TEST_PASSWORD"
echo "ok: typed the passphrase on the German layout"
