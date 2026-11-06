# Fedora build pins. Fedora builds from its live repos, so each image's package manifest is the record of what it got.
# Move the builder digest forward only through the test gate: `just fedora iso`, then `just fedora test`, must pass.

# Builder base image, pinned by digest (registry.fedoraproject.org/fedora; the tag is rebuilt often)
builder_base := "registry.fedoraproject.org/fedora:45@sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd"

# The builder image `just fedora builder` makes: kiwi, QEMU, UEFI firmware (OVMF or AAVMF), socat and mock
builder := "localhost/stackific-fedora-builder:45"

# Build output for this host's architecture, which picks the image: x86_64 on the PC, aarch64 in the Mac's runner VM
out := justfile_directory() / ".build" / "fedora" / arch()

# The raw UEFI firmware pair the boot test gives QEMU, from the builder: OVMF on x86_64, AAVMF on aarch64. QEMU's virt
# machine takes only 64 MiB flash images, so aarch64 uses edk2-aarch64's padded -pflash.raw pair, not QEMU_EFI.fd.
uefi_code := if arch() == "aarch64" { "/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw" } else { "/usr/share/edk2/ovmf/OVMF_CODE.fd" }
uefi_vars := if arch() == "aarch64" { "/usr/share/edk2/aarch64/vars-template-pflash.raw" } else { "/usr/share/edk2/ovmf/OVMF_VARS.fd" }

# The local repo `just fedora packages` fills with our RPMs (createrepo_c metadata, unsigned until release signing)
repo := justfile_directory() / ".build" / "fedora" / "repo"
