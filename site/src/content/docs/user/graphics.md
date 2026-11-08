---
title: Graphics and NVIDIA
description: Which driver each graphics card gets, how older NVIDIA cards run, and the LTS kernel entry in the boot menu.
sidebar:
  order: 4
---

Stackific OS picks the graphics driver every time it starts, live and installed, so there is nothing to choose or install. Today this covers the Arch Linux x86_64 image; the other images follow.

| Graphics card                                                         | Driver                                                                                                                         |
| --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| AMD and Intel                                                         | The open drivers in the kernel, with Mesa.                                                                                     |
| NVIDIA GeForce GTX 16, RTX 20 and newer (Turing and later)            | NVIDIA's open kernel modules, already in the image for both kernels. No download and no compiling.                             |
| NVIDIA GeForce GTX 10 and older (Pascal, Maxwell, Volta, and earlier) | nouveau, the open driver in the kernel. NVIDIA's current driver no longer supports these cards. 2D and video work; 3D is slow. |

Laptops with two GPUs get the right driver for each of them.

## Older NVIDIA cards

nouveau cannot raise the clock speed of GTX 900 and GTX 10 series cards (and of the MX150 to MX350 in laptops), so games and other 3D apps run slowly. Where such a card draws the desktop, MangoWM starts with shorter animations. To bring the longer ones back, add your own lines after the `source_optional` line in `~/.config/mango/config.conf`, for example:

```ini
animation_duration_open=400
animation_duration_close=800
```

If you move your disk to a computer with a different graphics card, Stackific OS picks the right driver at the next start.

## The LTS kernel

Installed systems carry two kernels: the current Linux kernel, which starts by default, and the long-term support (LTS) kernel, which changes less often. If the system stops starting after an update, pick "Stackific OS (LTS kernel)" in the boot menu, which shows for three seconds at power-on. Both kernels have the NVIDIA driver. The LTS kernel is older, so very new hardware may work only with the default kernel.

The screen that asks for the disk's password uses the firmware's display mode on this image; the graphics driver starts after the disk is unlocked.
