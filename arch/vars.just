# Arch build pins. The host's architecture picks the image: x86_64 on the PC, aarch64 (Arch Linux ARM) in the Mac's
# runner VM; nothing is cross-built. Move a pin forward only through the test gate: `just arch iso`, then
# `just arch test`, must pass on the new pins.

# Arch Linux Archive snapshot (archive.archlinux.org/repos/<date>). On x86_64 every package, in the builder and in the
# image, comes from it. On aarch64 only the builder's archiso and edk2-aarch64 do; Arch Linux ARM has no archive, so
# the rest comes from its live repos and the image's package manifest is the record.
ala_date := "2026-11-01"

# Builder base image, pinned by digest: archlinux/archlinux on x86_64 (hub.docker.com/r/archlinux/archlinux); on
# aarch64 menci/archlinuxarm, built daily from Arch Linux ARM, which has no official image
# (hub.docker.com/r/menci/archlinuxarm; the arm64 digest)
builder_base := if arch() == "aarch64" {
  "docker.io/menci/archlinuxarm:base-20261105.41979707783@sha256:91ed080fdfec1da7f3d2b5587d7a16a3ce7f3b3d3ee67b6affe51e399fcb619a"
} else {
  "docker.io/archlinux/archlinux:base-20261101.0.620341@sha256:837194727013a6e81fb55be11e61d6dcfe7fdd095b3313f8f3cff9bb3f9b182d"
}

# The builder image `just arch builder` makes: archiso, QEMU, UEFI firmware (OVMF or AAVMF) and socat
builder := "localhost/stackific-arch-builder:" + ala_date

# Build output for this host's architecture
out := justfile_directory() / ".build" / "arch" / arch()

# Local repo of our packages (`just arch packages`), read by the ISO build as [stackific]
repo := justfile_directory() / ".build" / "arch" / "repo"
