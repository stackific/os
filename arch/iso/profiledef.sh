#!/usr/bin/env bash
# archiso profile for the Stackific OS Arch x86_64 live ISO, sourced by mkarchiso (archiso v92); `just arch iso` runs it.
# Starts from archiso's baseline profile: UEFI only (systemd-boot), squashfs with releng's options, no sshd.
# STACKIFIC_VERSION comes from the recipe, so the image is stackific-<version>-arch-x86_64.iso.
# shellcheck disable=SC2034

iso_name="stackific"
iso_label="STACKIFIC_$(date --date="@${SOURCE_DATE_EPOCH:-$(date +%s)}" +%Y%m)"
iso_publisher="Stackific OS <https://os.stackific.com>"
iso_application="Stackific OS Live (Arch Linux base)"
iso_version="${STACKIFIC_VERSION:?set by just arch iso}-arch"
install_dir="arch"
buildmodes=('iso')
bootmodes=('uefi.systemd-boot')
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'xz' '-Xbcj' 'x86' '-b' '1M' '-Xdict-size' '1M')
file_permissions=(
  ["/etc/shadow"]="0:0:400"
)
