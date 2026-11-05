# Fedora build pins. Fedora builds from its live repos, so each image's package manifest is the record of what it got.
# Move the builder digest forward only through the test gate: `just fedora iso`, then `just fedora test`, must pass.

# Builder base image, pinned by digest (registry.fedoraproject.org/fedora; the tag is rebuilt often)
builder_base := "registry.fedoraproject.org/fedora:45@sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd"

# The builder image `just fedora builder` makes: kiwi, QEMU, OVMF and socat
builder := "localhost/stackific-fedora-builder:45"

# Build output for Fedora x86_64 images
out := justfile_directory() / ".build" / "fedora" / "x86_64"
