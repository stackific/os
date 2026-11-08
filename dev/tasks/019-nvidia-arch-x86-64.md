# 019: NVIDIA on Arch x86_64: open modules for linux and linux-lts, the driver picked at every boot, nouveau for older cards

- [ ] Done

Base: arch

## Ask

```text title="Prompt" wrap
Make NVIDIA graphics work on the Arch x86_64 image, live and installed. Ship linux-lts as a fallback kernel next to linux, with Arch's prebuilt nvidia-open and nvidia-open-lts, so no compiler and no network are needed; give the installed system a second Limine entry for the LTS kernel. Turing and newer cards (GTX 16, RTX 20 and later) run NVIDIA's open modules; older cards run nouveau, decided from a committed copy of NVIDIA's own legacy-branch device list read through sysfs. Pick the driver at every boot with a small systemd generator, so the live session starts on any GPU without asking, and an installed system follows a GPU swap. Keep NVIDIA and nouveau modules out of the initramfs. Where an older NVIDIA card running nouveau draws the desktop, MangoWM starts with lighter effects through an include file, without touching MangoWM's own config. chwd, CachyOS's hardware-detection tool, is the one CachyOS tool we accept; use it only if the design needs it. Report the ISO size and keep it within 2 GiB.
```

## Goal
The Arch x86_64 live ISO and every install from it carry `linux` and `linux-lts` with `nvidia-open`, `nvidia-open-lts` and `nvidia-utils`; a boot-time generator from a new package `stackific-hardware` loads nouveau instead of NVIDIA's module on pre-Turing cards, waits for the GPU driver before the login screen, and gives MangoWM low effects where nouveau draws the desktop; installs get a Limine entry for the LTS kernel.

## Context
What exists on the branch of task 018 (checked 2026-11-08):
- `arch/iso/packages.x86_64` lists `linux`, `linux-firmware` (the meta package, which pulls `linux-firmware-nvidia`, nouveau's GSP and GR firmware), Mesa through the desktops, no NVIDIA package and no second kernel. The ISO is 1554 MiB (`.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.iso`, 1629427712 bytes).
- The live initramfs uses `arch/iso/airootfs/etc/mkinitcpio.conf.d/archiso.conf`: `HOOKS=(base udev modconf archiso block filesystems)`, no `kms`, so no GPU module is in it today; every GPU driver loads after switch-root.
- The installer copies the live root to the target (`installer/src/copy.rs`), so installed systems get exactly the live image's packages. Arch's kernel step (`installer/src/base.rs`, `arch_kernel`) pipes every `usr/lib/modules/*/vmlinuz` to mkinitcpio's pacman-hook script in `arch-chroot`, which creates a missing preset from `/usr/share/mkinitcpio/hook.preset` (`PRESETS=('default')`, no fallback image) and writes `/boot/vmlinuz-<pkgbase>` and `/boot/initramfs-<pkgbase>.img`. Its HOOKS drop-in (`ARCH_HOOKS_CONF`) is mkinitcpio 42's default with `autodetect` and `kms`, plus `sd-encrypt` and `stackific-boot-failure`.
- `Base::kernel_files` returns one kernel and initramfs; `boot::limine_conf` writes one entry, `/<NAME>`.
- `live`'s MangoWM config is `arch/iso/airootfs/home/live/.config/mango/config.conf` (shared with Arch aarch64); installed users get theirs from `installer/src/system.rs` (`mango_config`, both bases): `source=/etc/mango/config.conf`, the launcher bind, `xkb_rules_layout`.
- greetd is the display manager (`arch/iso/airootfs/etc/systemd/system/display-manager.service` links `greetd.service`).

Upstream facts (Arch Linux Archive 2026-11-01, read in the pinned builder `localhost/stackific-arch-builder:2026-11-01` on 2026-11-08):
- `linux` 7.2.11.arch1-1 (161 MiB download); `linux-lts` 6.18.60-1 (155 MiB); `nvidia-open` 615.78.08-3, modules in `/usr/lib/modules/7.2.11-arch1-1/extramodules/` (`nvidia`, `nvidia-modeset`, `nvidia-drm`, `nvidia-uvm`, `nvidia-peermem`), depends `linux nvidia-utils=615.78.08`; `nvidia-open-lts` 1:615.78.08-10, depends `linux-lts`; `nvidia-utils` 615.78.08-1, 315 MiB download, 948 MiB installed.
- `nvidia-utils` ships `/usr/lib/modprobe.d/nvidia-utils.conf`: `blacklist nouveau`, `blacklist nova_core`, `blacklist nova_drm`, `softdep nvidia post: nvidia-uvm nvidia-drm`, `options nvidia NVreg_TemporaryFilePath=/var/tmp`; its GSP firmware in `/usr/lib/firmware/nvidia/615.78.08/`; `60-nvidia.rules`; the suspend units. So on a pre-Turing card nothing drives the display: the open `nvidia` module refuses the card and nouveau is blacklisted.
- kmod reads `/run/modprobe.d/` before `/usr/lib/modprobe.d/`, and a file there shadows the same name in `/usr/lib/modprobe.d/` (`modprobe.d(5)`).
- mkinitcpio 42.2's `kms` hook adds modules under `drivers/gpu/drm/` only, filtered by `autodetect`, which resolves the machine's modaliases with `modprobe -qaR`. NVIDIA's modules live in `extramodules/`, so `kms` never adds them; whether `-R` drops blacklisted nouveau is checked in the build (Tests).
- mkarchiso (archiso 92) copies every `/boot/vmlinuz-*` and `/boot/initramfs-*.img` of the live root to the ISO and to its boot image (`/usr/bin/mkarchiso` lines 496–497, 571–572).
- MangoWM 0.18.4: `source_optional=<path>` includes a file and ignores it when missing (`docs/configuration/basics.md` at tag 0.18.4, https://github.com/mangowm/mango). The stock `/etc/mango/config.conf` already has `blur=0` and `shadows=0`; its animations last 350–800 ms.
- NVIDIA's legacy list: the 580.xx and older tables of `/usr/share/doc/nvidia/html/supportedchips.html` in `nvidia-utils` 615.78.08 hold 879 device IDs; MX150 `1d10` is among them, MX450 `1f97` and GTX 1650 Mobile `1f99` are not (checked 2026-11-08).
- chwd is not in Arch's repositories (`pacman -Si chwd`: not found, 2026-11-01 archive).

## Inputs
Decisions from the operator's earlier design work, restated:
- A fallback kernel: `linux-lts` beside `linux`, with Arch's prebuilt `nvidia-open` and `nvidia-open-lts`, never `nvidia-open-dkms`; both kernels and module packages from the same archive date, checked by comparing each module's vermagic with its kernel; Limine gets an entry for `linux-lts`; no fallback initramfs.
- NVIDIA classification: a committed copy of NVIDIA's legacy-branch device list, not regenerated on driver bumps (the pre-Turing set is closed); device IDs read from `/sys/bus/pci/devices/*/{vendor,device,class}`, never through lspci, so a runtime-suspended dGPU stays asleep.
- The live image picks the driver at boot with a systemd generator and a `/run/modprobe.d` redirect (the pattern CachyOS ships, written by us, not CachyOS's package); no extra boot entries.
- No NVIDIA or nouveau modules in the initramfs: the screen at the LUKS prompt is the firmware's framebuffer; the driver loads from the root file system.
- Pre-Turing NVIDIA (Maxwell, Pascal, Volta) runs nouveau with Mesa; `linux-firmware-nvidia` stays. Where such a card draws the desktop (no other GPU), MangoWM gets lighter effects from a separate include file; a hybrid laptop whose iGPU draws keeps full effects.
- The GPU driver is re-checked at every boot, so a disk moved to a machine with another GPU still starts.

Copied into the tree while writing this task:
- `hardware/nvidia-legacy.ids`: the legacy list, its header rewritten with the source and the 2026-11-08 check.
- `hardware/mango-low-effects.conf`: the low-effects include (blur, layer blur, shadows and layer shadows off; open, close, move and tag animations 150 ms).

Where this task deviates from the earlier design, and why:
- **No chwd in this task.** Every image and every install carries all GPU packages (the installer copies the live root), and the generator picks the module at each boot, so nothing is left for chwd to install or remove for GPUs; packaging it means building CachyOS's Rust tool ourselves. Broadcom STA and fingerprint readers (task 021) are where package installs depend on hardware; chwd is decided there.
- **No "graphics hardware changed" notice.** The packages are always installed, so a GPU swap needs no package change; the generator handles it at boot.
- **The installer's pre-Turing warning screen is left for later** (logged in `docs/hardware.md`): nouveau starts the desktop, only 3D is slow.

## Design
- New Arch package `stackific-hardware` (`arch=('any')`), sources in `hardware/`: `/usr/lib/systemd/system-generators/stackific-gpu-generator` (bash, `set -euo pipefail`), `/usr/share/stackific/nvidia-legacy.ids`, `/usr/share/stackific/mango-low-effects.conf`. It depends on `bash` only; the image's package list carries the NVIDIA packages. Why our own glue: no packaged Arch tool picks between NVIDIA and nouveau at boot (CachyOS's `cachyos-drm-awaiter` is CachyOS's).
- The generator, on every boot (generators run again after switch-root, before udev's coldplug loads GPU modules), reads display controllers (PCI class `0x03xxxx`) from sysfs:
  - an NVIDIA device whose ID is in the legacy list, and no other NVIDIA device: writes `/run/modprobe.d/nvidia-utils.conf` (shadowing the package's file: `blacklist nvidia`, `blacklist nvidia_drm`, `blacklist nvidia_modeset`, `blacklist nvidia_uvm`, and nova's two blacklist lines), so udev loads nouveau;
  - when also no non-NVIDIA display controller exists: links `/run/stackific/mango-gpu.conf` to `/usr/share/stackific/mango-low-effects.conf`;
  - any NVIDIA display device: a drop-in in the generator's output dir orders `greetd.service` after `systemd-udev-settle.service` (and wants it), so the login screen and the autologin start after the module has loaded, not on the firmware framebuffer it replaces.
  - nothing for other machines.
- Every user's MangoWM config gets `source_optional=/run/stackific/mango-gpu.conf` right after `source=/etc/mango/config.conf` (the live user's file and `installer/src/system.rs`, so installs on both bases; on images without the generator the file never exists).
- `arch/iso/packages.x86_64`: add `linux-lts`, `nvidia-open`, `nvidia-open-lts`, `nvidia-utils`, `stackific-hardware`.
- The live ISO boots `linux` only. A live-only pacman hook, `arch/iso/airootfs/etc/pacman.d/hooks/stackific-live-lts.hook`, removes `linux-lts`'s kernel and initramfs from the live root's `/boot` after mkinitcpio's hook made them, so mkarchiso, which copies every `/boot/vmlinuz-*` and `initramfs-*.img`, does not carry them twice on the ISO. The installer removes the hook with the live state, and the kernel step builds `vmlinuz-linux-lts` and `initramfs-linux-lts.img` on the ESP from mkinitcpio's template preset. (Built first as a `linux-lts.preset` with `PRESETS=()`: mkinitcpio's hook script still copied the kernel, 15.7 MiB twice.)
- The live initramfs is compressed with xz (`COMPRESSION="xz"`, `-9e`, as archiso's releng profile), because the ISO carries it twice; the first build was 2079 MiB.
- Limine: `Base::kernel_files` returns every kernel the base boots, first the default; Arch x86_64 returns `linux`, then `linux-lts`; Arch aarch64 still `linux-aarch64` alone; Fedora still writes its own. `boot::limine_conf` writes `/<NAME>` for the first and `/<NAME> (LTS kernel)` for the second, with the same command line.
- Initramfs: the installer's HOOKS stay as they are when the build confirms that `autodetect` leaves blacklisted nouveau out; otherwise its drop-in drops `kms` (Changes, `installer/src/base.rs`).

## Changes
- `hardware/stackific-gpu-generator` (new), `hardware/nvidia-legacy.ids` and `hardware/mango-low-effects.conf` (copied), `hardware/justfile` (new: `just hardware test`, the generator's container test), `hardware/tests/` (fake sysfs trees and expected outputs, if the test needs files).
- `justfile`: `mod hardware`.
- `arch/packages/stackific-hardware/PKGBUILD` (new); `arch/justfile`: stage `hardware/` sources like `desktop/`.
- `arch/iso/packages.x86_64`; `arch/iso/airootfs/etc/pacman.d/hooks/stackific-live-lts.hook` (new); `arch/iso/airootfs/etc/mkinitcpio.conf.d/archiso.conf` (xz); `arch/iso/airootfs/home/live/.config/mango/config.conf`.
- `installer/src/base.rs` (the live-state preset removal, `kernel_files` with both kernels, the HOOKS drop-in only if needed), `installer/src/boot.rs` (the LTS entry), `installer/src/install.rs` (the call site and its tests), `installer/src/system.rs` (the `source_optional` line and its test).
- `tests/vm/arch.sh` (x86_64 checks behind `$efi`/`uname -m`, or `tests/vm/arch-x86_64.sh` if they cannot share), `tests/vm/install.sh` (the user config line), `tests/vm/installed.sh` (both kernels on the ESP, both Limine entries, no GPU module in either initramfs, both module vermagics).
- `release/justfile` (`just release changed`): `hardware/` belongs to the Arch images.
- `docs/hardware.md` (new: how the driver is picked, the legacy list and how to regenerate it, the improvement log), `docs/arch-iso.md`, `docs/installer.md`, `site/src/content/docs/user/hardware.md` (new user page: NVIDIA, older cards, the LTS entry) and its sidebar entry if the User Guide group lists pages, `AGENTS.md`, `release/notes.md`.
- The prompt-trail entry `site/src/content/docs/prompt-trail/arch/019-nvidia-arch-x86-64.md` and the review record `dev/reviews/019-nvidia-arch-x86-64.md`.

## Acceptance criteria

Must:
- `just arch packages` builds `stackific-hardware`; `just arch iso` builds the x86_64 ISO and prints its size, at most 2048 MiB. If it is larger, stop and report the sizes to the operator before going on.
- `just hardware test` passes: for recorded PCI sets (Turing desktop `10de:1e04`; Pascal desktop `10de:1b80`; MX150 `10de:1d10` with Intel `8086:9b41`; AMD `1002:73bf` alone; Intel alone; Turing `10de:1f99` with AMD; a Pascal plus a Turing card; an NVIDIA HDMI audio function `10de:10f0` class `0x040300` alone; no PCI display device) the generator writes exactly the expected files: the modprobe redirect only for the pre-Turing-only sets, the MangoWM link only for the Pascal desktop, the greetd drop-in for every set with an NVIDIA display controller.
- On the live ISO in QEMU (`just arch test`): `pacman -Q` lists `linux-lts`, `nvidia-open`, `nvidia-open-lts`, `nvidia-utils` and `stackific-hardware`; `modinfo -F vermagic` of `nvidia` for `linux` and for `linux-lts` starts with each kernel's version; the generator ran and wrote nothing (no `/run/modprobe.d/nvidia-utils.conf`, no `/run/stackific/mango-gpu.conf`, no greetd drop-in); the session checks still pass; the ISO's `arch/boot/x86_64/` holds no `linux-lts` file.
- On the installed system (`just arch test-install`): `/boot` holds `vmlinuz-linux`, `initramfs-linux.img`, `vmlinuz-linux-lts`, `initramfs-linux-lts.img` and no fallback image; `limine.conf` has the entries `/Stackific OS` and `/Stackific OS (LTS kernel)`; neither initramfs contains `nvidia` or `nouveau` modules (`lsinitcpio`); the user's MangoWM config has the `source_optional` line; every existing install check passes.
- The installer's unit tests pass (`just installer test`), with tests for the two Limine entries and the user config.
- Every existing boot and install test passes on Arch x86_64.

Must not:
- add `nvidia-open-dkms`, a compiler or headers to the image, or chwd;
- put NVIDIA or nouveau modules in any initramfs;
- change MangoWM's own `/etc/mango/config.conf` or the Fedora and aarch64 images' package lists;
- add a boot menu entry to the live ISO;
- touch files outside the Changes list.

## Tests
- `just hardware test`; `just installer test`.
- `just arch packages`, `just arch iso`, `just arch test`, `just arch test-install`.
- The build also confirms, in the builder container, that `modprobe -qaR` with `nvidia-utils`' blacklist installed prints nothing for a pre-Turing NVIDIA modalias (the `autodetect` question); the result goes into `docs/hardware.md`.
- `just lefthook all`.
- Real NVIDIA hardware cannot be emulated in QEMU. For the operator, once the ISO is built (task 030 collects the results):
  1. Write the ISO to a USB stick and boot a PC with an RTX (Turing or newer) card. The desktop appears; in foot, `lsmod | grep -c '^nvidia_drm'` prints 1 and `cat /run/modprobe.d/nvidia-utils.conf` says no such file.
  2. Boot a PC with a GTX 10 series (Pascal) card. The desktop appears with short animations; `lsmod | grep -c '^nouveau'` prints 1 and `readlink /run/stackific/mango-gpu.conf` prints `/usr/share/stackific/mango-low-effects.conf`.
  3. Install on the RTX PC, reboot, choose "Stackific OS (LTS kernel)" in Limine's menu: the desktop appears and `uname -r` ends in `-lts`.
