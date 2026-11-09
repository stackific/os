#!/usr/bin/env bash
# kiwi runs this script in the image root after the packages are installed and fedora/iso/root/ is copied in.
set -euo pipefail

# root has an empty password on the live ISO, as on the Arch one
passwd -d root

# The live user: empty password, in wheel, home from /etc/skel (our Noctalia include among it), as on the Arch image
useradd --create-home --groups wheel --comment 'Live user' live
passwd -d live
# live's MangoWM config, as on the Arch image (arch/iso/airootfs/home/live/.config/mango/config.conf): MangoWM's stock
# config, lighter effects where older NVIDIA graphics draw the desktop (stackific-hardware links the file only there),
# then Super+Space for the shell's launcher. After useradd, which copies /etc/skel only into a home it creates.
install -d -o live -g live /home/live/.config/mango
printf '%s\n' 'source=/etc/mango/config.conf' 'source_optional=/run/stackific/mango-gpu.conf' \
  'bind=SUPER,space,spawn,noctalia msg panel-toggle launcher' >/home/live/.config/mango/config.conf
chown live:live /home/live/.config/mango/config.conf

# Bazaar, the app store (task 023), from Flathub, which stackific-desktop's remotes.d file adds on first use: Fedora has
# no package of it. Installed now, so the live session and every install have it without network.
flatpak install --system --noninteractive flathub io.github.kolunmi.Bazaar

# greetd is the display manager. gdm is here only because gnome-shell requires it, and Fedora's preset enables it.
systemctl disable gdm.service
systemctl enable greetd.service
