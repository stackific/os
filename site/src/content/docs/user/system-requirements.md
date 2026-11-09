---
title: System requirements
description: What a machine needs to run the Stackific OS live ISOs, and the hardware left out.
sidebar:
  order: 1
---

Each Stackific OS ISO runs from a USB stick, so trying it needs no disk space.

| Part      | Needs                                                                                                                                                                                                                                                                                           |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Processor | x86_64 ISOs: a 64-bit Intel or AMD processor. aarch64 ISOs: a 64-bit ARM machine with UEFI firmware; they are tested in QEMU virtual machines.                                                                                                                                                  |
| Firmware  | UEFI, with Secure Boot off. The images do not boot in legacy BIOS (CSM) mode.                                                                                                                                                                                                                   |
| Memory    | 4 GiB of RAM, which the images are tested with.                                                                                                                                                                                                                                                 |
| USB stick | 4 GB or more. Writing the ISO erases it.                                                                                                                                                                                                                                                        |
| Graphics  | AMD and Intel GPUs. NVIDIA: on the x86_64 images, NVIDIA's open driver for GTX 16, RTX 20 and newer, and nouveau for older cards ([Graphics and NVIDIA](/docs/user/graphics/)). Without a working GPU driver, as in a virtual machine without 3D, the desktop falls back to software rendering. |

## Other hardware

On the x86_64 images, live and installed:

- **Wi-Fi, Bluetooth and sound** get their firmware from the image, with nothing to download, including Intel, AMD, Realtek, MediaTek, Qualcomm Atheros, Broadcom (`brcmfmac`) and Marvell Wi-Fi, laptop audio (Intel SOF, Cirrus), older sound cards and TV tuners.
- **Thunderbolt and USB4** docks and devices are authorised by `bolt`: `boltctl list` shows them.
- **Fingerprint readers** work with `fprintd`: `fprintd-enroll` saves a finger and `fprintd-verify` checks it. Logging in, `sudo` and other password prompts do not use the reader yet; they always ask for your password. Plasma's own lock screen, on the Arch Linux image, also accepts a saved finger.
- **Webcams**, USB ones and the MIPI cameras of newer Intel laptops (IPU6), reach apps through PipeWire and libcamera. Intel's newest IPU7 cameras are not supported yet.

### Broadcom Wi-Fi that needs the STA driver

Some laptops from about 2008 to 2015, and MacBooks with a BCM4360, have Broadcom Wi-Fi that only Broadcom's own driver (`wl`) runs. Its package has to be built on the computer, so it is not on the images. To check, run `lspci -nn | grep 14e4`: these IDs need it: `4311`, `4312`, `4315`, `4727`, `4328`, `4329`, `432a`, `432b`, `432c`, `432d`, `0576`, `4353`, `4357`, `4358`, `4359`, `4365`, `4331`, `43b1`, `43a0`, `4360`. Connect by Ethernet or USB tethering, then:

- **Arch Linux image:** `sudo pacman -S broadcom-wl-dkms linux-headers linux-lts-headers`, then restart.
- **Fedora Remix image:** install RPM Fusion's nonfree repository (`sudo dnf install https://mirrors.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-45.noarch.rpm`), then `sudo dnf install akmod-wl`, wait a few minutes for the driver to build, and restart.

## Left out

Stackific OS does not support:

- server and datacenter network cards;
- hardware that needs a vendor kernel: Microsoft Surface devices, Apple Macs with the T2 chip, Apple silicon Macs and Raspberry Pi boards.
