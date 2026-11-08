#!/usr/bin/env bash
# kiwi's editbootconfig script (config.kiwi): kiwi runs it on the build host after it writes the live ISO's grub.cfg,
# with `iso:<media dir>` as $1. It renames kiwi's first entry "Try <title> (<arch>, UEFI)", still the default, and adds
# "Install <title> (<arch>, UEFI)" after it, <arch> being the build host's (x86_64 or aarch64, the image's): the same
# kernel line plus systemd.unit=stackific-installer.target, which starts the installer on tty1 (installer/systemd/).
# kiwi has no setting for an extra entry, and its own "Install" entry is for OEM images.
set -euo pipefail

cfg=${1#iso:}/boot/grub2/grub.cfg
awk -v arch="$(uname -m)" '
  !done && /^menuentry "/ {
    title = $0; sub(/^menuentry "/, "", title); sub(/".*/, "", title)
    sub(/^menuentry "[^"]*"/, "menuentry \"Try " title " (" arch ", UEFI)\"")
    inblock = 1; block = ""
  }
  inblock {
    print; block = block $0 "\n"
    if ($0 ~ /^}/) {
      inblock = 0; done = 1
      sub(/"Try /, "\"Install ", block)
      sub(/\n[[:space:]]*linux [^\n]*/, "& systemd.unit=stackific-installer.target", block)
      printf "%s", block
    }
    next
  }
  { print }
' "$cfg" >"$cfg.new"
grep -q 'systemd.unit=stackific-installer.target' "$cfg.new" || { echo "edit-boot.sh: no entry to copy in $cfg" >&2; exit 1; }
mv "$cfg.new" "$cfg"
