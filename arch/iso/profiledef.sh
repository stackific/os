#!/usr/bin/env bash
# archiso profile for the Stackific OS Arch live ISO, sourced by mkarchiso (archiso v92); `just arch iso` runs it.
# One profile for both architectures: the builder's own (x86_64, or aarch64 from Arch Linux ARM), with the package
# list packages.<arch>. Starts from archiso's baseline profile: UEFI only (systemd-boot), squashfs, no sshd.
# STACKIFIC_VERSION comes from the recipe, so the image is stackific-<version>-arch-<arch>.iso.
# shellcheck disable=SC2034

iso_name="stackific"
iso_label="STACKIFIC_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Stackific OS <https://os.stackific.com>"
iso_application="Stackific OS Live (Arch Linux base)"
iso_version="${STACKIFIC_VERSION:?set by just arch iso}-arch"
install_dir="arch"
buildmodes=('iso')
bootmodes=('uefi.systemd-boot')
arch="$(uname -m)"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
# releng's xz options with only this architecture's BCJ filter: Arch Linux ARM's kernel cannot decode x86 BCJ blocks
case "$arch" in aarch64) bcj="arm64" ;; *) bcj="x86" ;; esac
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' "$bcj" '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
  ["/etc/gshadow"]="0:0:400"
)
