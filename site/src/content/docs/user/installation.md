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

Builds after 0.1.0 have an installer on every ISO, x86_64 and aarch64: "Install Stackific OS" in the app launcher, first in GNOME's dash on the Fedora Remix images, and in the boot menu next to "Try Stackific OS". It installs into a drive's free space, next to what is there, or onto a whole drive, which it erases. To install next to Windows, read [Next to Windows](/docs/user/windows/) first: BitLocker needs its recovery key once Secure Boot is off.

1. Open "Install Stackific OS" and follow its screens. It first asks to authenticate as an administrator. `live` has no password: in MangoWM, press Enter at the empty password prompt; in GNOME, choose Authenticate.
2. At the end, remove the USB stick and restart the computer.
3. The computer starts Stackific OS: its boot menu shows "Stackific OS" for 3 seconds, then it asks for your disk password, which is the password you chose in the installer. Type it and press Enter; the login screen follows. Log in as the user you made.

### Encryption

Every install encrypts the Stackific OS partition (LUKS2), and your password unlocks it at every start. That is why the installer refuses a password that is easy to guess. Encryption protects your files if the computer is lost or stolen. It does not protect against someone who changes the computer while you are away.

- The prompt uses the keyboard layout you chose in the installer, but the installer itself reads your password with the US layout. If you chose another layout, use only letters, digits and symbols that sit on the same keys in both: on a German keyboard, for example, avoid y and z, which swap places.
- If the password is refused at the prompt, check Caps Lock and the layout.
- After three wrong tries, a screen says the system could not start. Press Enter to restart and try again.
- Changing your login password later (`passwd`) does not change the disk password. To change the disk password, run `sudo cryptsetup luksChangeKey /dev/<partition>`, with the partition `lsblk` shows as `crypto_LUKS` (for example `nvme0n1p6`).
- If you forget the disk password, the files on the partition cannot be recovered.

On the Fedora Remix image, the installed system updates with `sudo dnf upgrade`. Each kernel update adds an entry to the boot menu: the newest kernel is "Stackific OS", and the two before it are listed below it by version, to start when the newest has a problem.

Apps that store passwords, such as browsers and git, keep them in your keyring (oo7, the Secret Service), which your login password unlocks when you log in. If you change your password later with `passwd`, the keyring keeps the old one, and apps may then ask you for it.

If the computer starts something else, open the firmware's boot menu (often F12, F11, F8 or Esc at power-on) and pick "Stackific OS". The installer puts it first in the firmware's boot order, but a large Windows update can put Windows first again; [Next to Windows](/docs/user/windows/#putting-stackific-os-first-again) says how to change it back.

## Limits of this preview

- The images are unsigned.
- Version 0.1.0 has no installer. The installer in later builds is new: back up your data before you install next to another system.
- To make free space or set up partitions first, open Disks from the app launcher. Shrinking Windows itself is safest from Windows' own Disk Management.
- NVIDIA's own driver is only in the x86_64 images so far ([Graphics and NVIDIA](/docs/user/graphics/)).

## About the bases

The Fedora Remix images contain modified Fedora® packages and software that the Fedora Project does not provide. They are not provided or supported by the Fedora Project. Official, unmodified Fedora releases are available at https://fedoraproject.org/.

Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat.
