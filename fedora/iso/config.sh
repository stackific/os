#!/usr/bin/env bash
# kiwi runs this script in the image root after the packages are installed and fedora/iso/root/ is copied in.
set -euo pipefail

# root has an empty password on the live ISO, as on the Arch one
passwd -d root

# The live user: empty password, in wheel, home from /etc/skel (our Noctalia include among it), as on the Arch image
useradd --create-home --groups wheel --comment 'Live user' live
passwd -d live

# greetd is the display manager. gdm is here only because gnome-shell requires it, and Fedora's preset enables it.
systemctl disable gdm.service
systemctl enable greetd.service
