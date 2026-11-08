---
title: System requirements
description: What a machine needs to run the Stackific OS live ISOs, and the hardware left out.
sidebar:
  order: 1
---

Each Stackific OS ISO runs from a USB stick, so trying it needs no disk space.

| Part      | Needs                                                                                                                                                                                                                                                                                                     |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Processor | x86_64 ISOs: a 64-bit Intel or AMD processor. aarch64 ISOs: a 64-bit ARM machine with UEFI firmware; they are tested in QEMU virtual machines.                                                                                                                                                            |
| Firmware  | UEFI, with Secure Boot off. The images do not boot in legacy BIOS (CSM) mode.                                                                                                                                                                                                                             |
| Memory    | 4 GiB of RAM, which the images are tested with.                                                                                                                                                                                                                                                           |
| USB stick | 4 GB or more. Writing the ISO erases it.                                                                                                                                                                                                                                                                  |
| Graphics  | AMD and Intel GPUs. NVIDIA: on the Arch Linux x86_64 image, NVIDIA's open driver for GTX 16, RTX 20 and newer, and nouveau for older cards ([Graphics and NVIDIA](/docs/user/graphics/)). Without a working GPU driver, as in a virtual machine without 3D, the desktop falls back to software rendering. |

## Left out

Stackific OS does not support:

- server and datacenter network cards;
- hardware that needs a vendor kernel: Microsoft Surface devices, Apple Macs with the T2 chip, Apple silicon Macs and Raspberry Pi boards.
