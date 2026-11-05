#!/usr/bin/env bash
# kiwi runs this script in the image root after the packages are installed and fedora/iso/root/ is copied in.
set -euo pipefail

# root has an empty password on the live ISO, as on the Arch one
passwd -d root
