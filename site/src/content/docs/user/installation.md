---
title: Installation
description: Download a Stackific OS live ISO, check it, write it to a USB stick, boot it and use the live session.
sidebar:
  order: 2
---

Stackific OS 0.1.0 is a live preview: there is no installer yet. Each ISO boots from a USB stick to MangoWM, with a second desktop to switch to. Nothing is written to your disk, and changes are gone after a reboot. Check the [system requirements](/docs/user/system-requirements/) first.

## Pick an ISO

Every release has four ISOs, named `stackific-<version>-<base>-<arch>.iso`. Pick the architecture from the [system requirements](/docs/user/system-requirements/), then the base:

| ISO                                      | Built from              | Second desktop |
| ---------------------------------------- | ----------------------- | -------------- |
| `stackific-<version>-arch-x86_64.iso`    | Based on Arch Linux     | KDE Plasma     |
| `stackific-<version>-arch-aarch64.iso`   | Based on Arch Linux ARM | KDE Plasma     |
| `stackific-<version>-fedora-x86_64.iso`  | Fedora Remix            | GNOME          |
| `stackific-<version>-fedora-aarch64.iso` | Fedora Remix            | GNOME          |

## Download and check it

1. Open the latest release: https://github.com/stackific/os/releases/latest.
2. Download your ISO and `SHA256SUMS` into one folder.
3. In that folder, check the download:
   - Linux: `sha256sum -c SHA256SUMS --ignore-missing`
   - macOS: `shasum -a 256 -c SHA256SUMS --ignore-missing`

   Either prints `<iso>: OK`. Anything else means the download is broken: download it again.

The images are unsigned: `SHA256SUMS` shows that a download is intact, not who built it. Signed releases come later.

## Write it to a USB stick

:::caution
Writing the ISO erases everything on the USB stick.
:::

1. Plug in the stick and run `lsblk`. Find the stick by its size, for example `/dev/sdb`. Use the whole device, not a partition such as `/dev/sdb1`.
2. Write the ISO, with `/dev/sdX` replaced by the stick:

   ```sh
   sudo dd if=stackific-<version>-<base>-<arch>.iso of=/dev/sdX bs=4M status=progress oflag=sync
   ```

3. Remove the stick once `dd` returns.

On macOS, find the stick with `diskutil list` (for example `/dev/disk4`), run `diskutil unmountDisk /dev/disk4`, then write the ISO to the raw device: `sudo dd if=stackific-<version>-<base>-<arch>.iso of=/dev/rdisk4 bs=4m`, with `4` replaced by the stick's number.

## Boot it

1. In the PC's firmware setup, turn Secure Boot off. The images boot in UEFI mode only, not in legacy BIOS (CSM) mode.
2. Boot from the USB stick, through the firmware's boot menu (often F12, F11, F8 or Esc at power-on).

## The live session

The live ISO logs you in as the user `live` with no prompt, and starts MangoWM. In MangoWM, `Alt+Return` opens a terminal, `Super+Space` opens the desktop's launcher, and `Alt+Space` opens rofi, the older one.

### Switch to the second desktop

Each image has a second desktop: KDE Plasma on the images based on Arch Linux and Arch Linux ARM, GNOME on the Fedora Remix images.

1. Log out: run `loginctl terminate-user live` in a terminal.
2. At the login screen, pick the user `live` and the session (Plasma or GNOME) in the list below the password field.
3. Type any character as the password, and press Enter. `live` has no password, but the login screen does not accept an empty field.

## Install it

Builds after 0.1.0 have an installer on the `-arch-x86_64.iso`: "Install Stackific OS" in the app launcher, and in the boot menu next to "Try Stackific OS". It installs into a drive's free space, next to what is there, or onto a whole drive, which it erases.

1. Open "Install Stackific OS" and follow its screens.
2. At the end, remove the USB stick and restart the computer.
3. The computer starts Stackific OS: its boot menu shows "Stackific OS" for 3 seconds, then the login screen. Log in as the user you made.

If the computer starts something else, open the firmware's boot menu (often F12, F11, F8 or Esc at power-on) and pick "Stackific OS". The installer puts it first in the firmware's boot order, but a large Windows update can put Windows first again.

## Limits of this preview

- The images are unsigned.
- Version 0.1.0 has no installer. The installer in later builds is new: back up your data before you install next to another system.
- To make free space or set up partitions first, open Disks from the app launcher. Shrinking Windows itself is safest from Windows' own Disk Management.
- NVIDIA's own driver is not in the images yet.

## About the bases

The Fedora Remix images contain modified Fedora® packages and software that the Fedora Project does not provide. They are not provided or supported by the Fedora Project. Official, unmodified Fedora releases are available at https://fedoraproject.org/.

Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat.
