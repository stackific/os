---
title: NVIDIA on Fedora x86_64, its open modules built into the image
description: RPM Fusion's NVIDIA driver on the Fedora x86_64 image with NVIDIA's open modules built by mock for the image's kernel, no compiler in the image, and the same boot-time pick of NVIDIA or nouveau as on Arch.
sidebar:
  order: 20
  label: 020 NVIDIA on Fedora x86_64
pr: https://github.com/stackific/os/pull/20
task: dev/tasks/020-nvidia-fedora-x86-64.md
---

## Goal

The Fedora x86_64 live ISO and every install from it carry RPM Fusion's NVIDIA driver with NVIDIA's open kernel modules for the image's kernel, and no compiler or kernel headers. `stackific-hardware` picks NVIDIA or nouveau at every boot, as on Arch, and also falls back to nouveau when the running kernel has no NVIDIA module.

## Context

Task 019 put NVIDIA on the Arch x86_64 image, where Arch ships prebuilt modules. RPM Fusion ships none for Fedora: its `akmod-nvidia` compiles the modules on each machine, which a live ISO cannot wait for. This step builds them once, while the image is built, and settles the operator's open decision 6 (prebuilt in the image).

## Expectations

Must:

- `just hardware test` passes, including: a Turing desktop without an NVIDIA module for the running kernel gets the modprobe redirect and the greetd drop-in but no MangoWM link; every task 019 set gives the same files as before.
- `just fedora packages` builds `stackific-hardware`; `just arch packages` still builds.
- `just fedora iso` builds the x86_64 ISO and prints its size, at most 2048 MiB. If it is larger, stop and report the sizes to the operator before going on. The manifest lists `xorg-x11-drv-nvidia` 615.78.08, `kmod-nvidia-<the image's kernel>` and `stackific-hardware`, and no `gcc`, `kernel-devel`, `akmod-nvidia`, `akmods` or `xorg-x11-drv-nvidia-kmodsrc`.
- On the live ISO in QEMU (`just fedora test`): `modinfo -F vermagic nvidia` starts with `uname -r`; `modinfo -F license nvidia` prints `Dual MIT/GPL` (the open modules); `/usr/lib/modprobe.d/nvidia-utils.conf` blacklists nouveau; the generator wrote nothing on QEMU's display; no `nvidia` or `nouveau` module is loaded; `/home/live/.config/mango/config.conf` has the three lines Arch's live user has, and Super+Space opens the launcher; every existing check passes.
- On the installed system (`just fedora test-install`): `kmod-nvidia-$(uname -r)` is installed; the initramfs holds no `nvidia` or `nouveau` module (`lsinitrd`); `rpm -qa 'grub2*' grubby` prints nothing; no `/etc/yum.repos.d` or `/usr/share/dnf5/repos.d` file names RPM Fusion; every existing install check passes.
- `just installer test` passes.

Must not:

- leave a compiler, kernel headers, `akmod-nvidia` or NVIDIA's module sources in the image;
- leave an RPM Fusion repo file in the image, or take any package from RPM Fusion outside `includepkgs`;
- put NVIDIA or nouveau modules in any initramfs;
- change the aarch64 Fedora image's package list (task 022);
- touch files outside the Changes list.

## Prompt

```text title="Prompt" wrap
Make NVIDIA graphics work on the Fedora x86_64 image, live and installed, as task 019 did on Arch x86_64. Take NVIDIA's driver from RPM Fusion nonfree, its packages and metadata checked against RPM Fusion's key committed in the repo and the repository limited to the NVIDIA packages, as Terra is. Build NVIDIA's open kernel modules for the image's kernel in `just fedora packages`, with mock from the source RPM inside RPM Fusion's akmod-nvidia (its packages checked against the committed key, kernel-devel added to mock's root, the open variant forced), so the module RPM lands in our local repo and leave the compiler and the kernel headers out of the finished image, so the live session and installs need no compiler, no network and no wait. Turing and newer cards run NVIDIA's open modules; older cards run nouveau, decided at every boot by the same generator as on Arch (stackific-hardware, from hardware/), packaged as an RPM. RPM Fusion blacklists nouveau through grubby on GRUB's command line, which Limine systems never read, so ship the nouveau blacklist as a modprobe.d file; and when the running kernel has no NVIDIA module (a later kernel update), let nouveau drive the card instead of leaving a black screen. Keep NVIDIA and nouveau modules out of the initramfs. Give the live user the same MangoWM config as on Arch. Fedora has no chwd: decide whether the planned Fedora detection step is still needed. Report the ISO size and keep it within 2 GiB. RPM Fusion's driver requires grubby only while it installs, and grubby keeps grub2-tools: the installer removes it with the live image's GRUB.
```

## Outcome

- **The modules.** On x86_64, `just fedora packages` downloads `akmod-nvidia` and `xorg-x11-drv-nvidia-kmodsrc` from RPM Fusion and checks both against RPM Fusion's key, committed as `fedora/iso/rpmfusion-nonfree-45.asc` (fingerprint `79BD B88F 9BBF 7391 0FD4 095B 6A2A F961 9484 3C65`). Mock then rebuilds the source RPM inside `akmod-nvidia` for the newest Fedora kernel, with `--with kmod_nvidia_open`, in about two minutes. `kmod-nvidia-7.2.15-300.fc45.x86_64` lands in our local repo next to our RPMs. The first try stopped in `%prep` ("source/Makefile not found"), because the source RPM names no kernel headers once `kernels` is set; mock now adds `kernel-devel-<kernel>` itself.
- **The image.** kiwi takes the driver (`xorg-x11-drv-nvidia`, `-libs`, `-cuda-libs`, `nvidia-modprobe`, `nvidia-settings`) from RPM Fusion, limited by `includepkgs` and checked against the same key, and `nvidia-kmod` from our repo. It also adds `stackific-hardware`, now an RPM too. No compiler, kernel headers, akmods or RPM Fusion repo file end up in the image.
- **The pick.** It is the same generator as on Arch. It now also gives NVIDIA graphics to nouveau when the running kernel has no `nvidia.ko`, which covers a Fedora kernel update before our repos ship its kmod (task 028). Fedora's `stackific-hardware` blacklists nouveau in `nvidia-utils.conf`, because RPM Fusion does it only on GRUB's command line, and a dracut drop-in keeps nouveau out of the initramfs.
- **The live user** gets the same MangoWM config as on Arch: the stock config, the low-effects include and Super+Space for the launcher.
- **GRUB on installs.** RPM Fusion's driver requires `grubby` while it installs, and `grubby` keeps `grub2-tools`. The installer now removes `grubby` with the live image's GRUB.
- **Size.** 1921 MiB (target 2048 MiB).

### Screenshots

No UI.

## Review

- **Round 1** found nothing. One worry was checked and dropped before it was reported: whether the signature check passes an unsigned package. rpm 6.1 refuses one, so the build stops; a comment in the recipe now says so.

## Manual changes

None.

## Result

- `just hardware test`: 13 PCI sets, three of them without an NVIDIA module for the running kernel, give exactly the expected files.
- `just installer lint` and `just installer test` pass.
- `just fedora packages` builds `stackific-hardware` and `kmod-nvidia-7.2.15-300.fc45.x86_64` (615.78.08); `just arch packages` still builds.
- `just fedora iso`: 1921 MiB (target 2048 MiB); the manifest has the driver, the kmod for kernel 7.2.15-300.fc45 and `stackific-hardware`, and no gcc, kernel-devel or akmods.
- `just fedora test` passes, with the new checks: the driver packages and the kmod for the running kernel, no build tools, `nvidia`'s vermagic matching `uname -r` and its license `Dual MIT/GPL` (the open modules), no RPM Fusion repo file, the modprobe and dracut files, the generator idle on QEMU's std VGA, and `live`'s MangoWM config.
- `just fedora test-install` passes: the installed system has `kmod-nvidia-$(uname -r)`, no GPU module in its initramfs, and no GRUB or grubby. The first run failed only on GRUB's three leftover packages, before the installer removed `grubby`.
- `just lefthook all` passes.
- Docs: `docs/hardware.md`, `docs/fedora-iso.md`, `docs/installer.md`, `site/src/content/docs/user/graphics.md`, `site/src/content/docs/user/system-requirements.md`, `site/src/content/docs/user/installation.md`, `release/notes.md`, `AGENTS.md`.

## Deviations

- The modules are built by `just fedora packages` with mock, not inside kiwi's `config.sh` with akmods, so the image never holds a compiler and no kiwi uninstall step is needed. The task file was updated.
- `tests/vm/fedora.sh` checks the live user's new MangoWM config (both Fedora images, which share `config.sh`).
- The Fedora detection step from the earlier design is not needed for NVIDIA: every image and install carries the driver, and the generator picks at boot.
