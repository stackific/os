# Arch build pins: one Arch Linux Archive date and the builder's base image. Move them forward only through the test
# gate: `just arch iso`, then `just arch test`, must pass on the new pins.

# Arch Linux Archive snapshot every package comes from, for the builder and the image (archive.archlinux.org/repos/<date>)
ala_date := "2026-11-01"

# Builder base image, pinned by digest (hub.docker.com/r/archlinux/archlinux)
builder_base := "docker.io/archlinux/archlinux:base-20261101.0.620341@sha256:837194727013a6e81fb55be11e61d6dcfe7fdd095b3313f8f3cff9bb3f9b182d"

# The builder image `just arch builder` makes: archiso, QEMU, OVMF and socat from the archive date
builder := "localhost/stackific-arch-builder:" + ala_date

# Build output for Arch x86_64 images
out := justfile_directory() / ".build" / "arch" / "x86_64"

# Local repo of our packages (`just arch packages`), read by the ISO build as [stackific]
repo := justfile_directory() / ".build" / "arch" / "repo"
