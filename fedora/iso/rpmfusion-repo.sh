#!/usr/bin/env bash
# kiwi's customize script for the RPM Fusion nonfree repositories (config.kiwi), as terra-repo.sh is for Terra: kiwi
# writes the repo file, then runs this script with the file's path as $1.
# includepkgs: RPM Fusion supplies only NVIDIA's driver and what it requires from there; akmod-nvidia, which would
# pull a compiler and kernel headers, is left out, so nvidia-kmod resolves to our kmod-nvidia-<kernel>.
# gpgkey: the key packages are checked against, imported into the image's rpmdb from config.kiwi's <signing>.
set -euo pipefail

printf 'includepkgs=xorg-x11-drv-nvidia,xorg-x11-drv-nvidia-libs,xorg-x11-drv-nvidia-cuda-libs,nvidia-modprobe,nvidia-settings\ngpgkey=file:///desc/rpmfusion-nonfree-45.asc\n' >>"$1"
