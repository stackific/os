# Host facts shared by the toolchain modules. Supported hosts: Linux on x86_64 or aarch64, and macOS on Apple silicon
# (image builds need Linux). Any other host fails at the first checksum.

# Checks `<sha256>  <file>` lines on stdin with the host's checksum tool, printing only failures
sha256_check := if os() == "macos" { "shasum -a 256 -c --quiet -" } else { "sha256sum -c --quiet -" }
