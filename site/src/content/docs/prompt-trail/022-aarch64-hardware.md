---
title: NVIDIA and regular hardware on both aarch64 images
description: NVIDIA's open modules or nouveau, picked at every boot, on the Arch Linux ARM image, with the regular-hardware set of the x86_64 images on Arch Linux ARM.
sidebar:
  order: 22
  label: 022 NVIDIA and regular hardware on aarch64
pr: https://github.com/stackific/os/pull/22
task: dev/tasks/022-aarch64-hardware.md
---

## Goal

Both aarch64 images and their installs carry NVIDIA's driver, with the same pick at every boot as on x86_64, and Arch Linux ARM carries the regular hardware the other three images have.

## Context

Tasks 019 to 021 gave the two x86_64 images NVIDIA's driver, `stackific-gpu-generator` and matched firmware, Thunderbolt, fingerprint and webcam support. The aarch64 images had none of the NVIDIA part, and Arch Linux ARM none of the regular-hardware part. Arch Linux ARM has no prebuilt NVIDIA module package; RPM Fusion has the same NVIDIA packages for aarch64 as for x86_64.

## Expectations

Must:

- `just arch test` on aarch64 passes with `arch-aarch64.sh`: the 14 new packages installed, `dkms status` lists `nvidia/615.78.08` installed for `uname -r`, `modinfo -F vermagic nvidia` starts with `uname -r`, no `/usr/lib/firmware/mrvl/prestera`, the generator present and writing nothing on QEMU, no `nvidia` or `nouveau` module loaded.
- `just fedora test` on aarch64 passes the NVIDIA checks `fedora-x86_64.sh` had, now in `fedora.sh`: `kmod-nvidia-$(uname -r)` installed, no gcc, kernel-devel or akmods, no RPM Fusion repo file.
- `just arch test-install` and `just fedora test-install` on aarch64 pass `installed.sh` with the module present for the installed kernel and no NVIDIA or nouveau module in the initramfs; the unlock test still shows the passphrase prompt on Arch aarch64 without `kms`.
- Both x86_64 images still pass their boot and install tests (the moved checks unchanged).
- Each image task reports its ISO size; the `images` workflow builds all four.
- On the Mac, `limactl list --format '{{.Memory}}' stackific-runner` prints `17179869184` after `just runner vm`.
- `just lefthook all` and `cargo test` in `installer/` pass.

Must not:

- Add `linux-firmware-qcom` or `qcom-firmware`.
- Put a compiler, kernel headers or akmods on any image other than Arch aarch64.
- Change the generator or `nvidia-legacy.ids`.

## Prompt

```text title="Prompt" wrap
Bring both aarch64 images to the hardware support the x86_64 images have, live and installed. NVIDIA: Turing and newer cards get NVIDIA's open modules, older ones nouveau, picked at every boot by stackific-hardware's generator as on x86_64. On Arch Linux ARM, which has no prebuilt module package, use nvidia-open-dkms with nvidia-utils and linux-aarch64-headers: the module builds once while the image is built, and DKMS rebuilds it when an installed system updates its kernel; the compiler and kernel headers this puts in the image are accepted on this one image. On Fedora aarch64, do what Fedora x86_64 does: RPM Fusion's driver with kmod-nvidia built for the image's kernel by `just fedora packages`, no compiler in the image. Arch Linux ARM also gets task 021's regular hardware: linux-firmware-marvell (without Prestera), sof-firmware, alsa-firmware, wireless-regdb, bolt, fprintd (no PAM change), and libcamera with PipeWire's libcamera and V4L2 plugins; Fedora aarch64 has these already. Leave Qualcomm SoC firmware out of both aarch64 images: Snapdragon machines need device trees, which our UEFI-only Limine boot does not load, and it costs about 183 MiB. Installed Arch aarch64 systems drop the kms hook as x86_64 did, so no GPU module goes into the initramfs. Best effort for real hardware: QEMU shows no NVIDIA device, so the VM tests prove the packages and modules are in place and nothing loads. Also give the Mac's runner VM 16 GiB of memory, including a VM made earlier with less. Land it in the same pull request as task 021 (#22).
```

## Outcome

- **Arch Linux ARM.** `just arch iso` adds ALARM's `[alarm]` repo on aarch64, where `libpisp` (which `libcamera` needs) lives. `packages.aarch64` gains `linux-aarch64-headers`, `stackific-hardware`, `nvidia-open-dkms`, `nvidia-utils` and task 021's ten packages, so it differs from `packages.x86_64` only in its kernel, headers, keyring and module package. DKMS's pacman hook builds `nvidia.ko` for `linux-aarch64` while mkarchiso installs the packages, so the live image compiles nothing and an installed system rebuilds the module when its kernel updates. `NoExtract` in the shared `pacman.conf` keeps Marvell's Prestera firmware out, as on x86_64.
- **Fedora aarch64.** No NVIDIA yet: RPM Fusion's aarch64 `xorg-x11-drv-nvidia-libs` 615.78.08 requires `libnvidia-rmapi-tegra.so.615.78.08`, which no package in RPM Fusion's release or updates repos ships (CI run 42546467105 and `dnf5 repoquery --forcearch=aarch64 --whatprovides`, 2026-11-08). The four NVIDIA lines stay `arch="x86_64"` and the `kmod-nvidia` build stays x86_64; `docs/hardware.md` lists the two-line change under Later. kiwi's tmpfs takes up to 75% of the RAM.
- **Installer.** Installed Arch systems drop the `kms` hook on aarch64 too, so no GPU module goes into any initramfs; `linux-aarch64` has `CONFIG_FB_EFI=y`, so the passphrase prompt draws on the firmware's framebuffer.
- **Tests.** The generator and task 021 checks move from `arch-x86_64.sh` to `arch.sh`, so both Arch images run them; Fedora's NVIDIA checks stay in `fedora-x86_64.sh`. The new `arch-aarch64.sh` checks the DKMS module (`dkms status`, vermagic, licence). `installed.sh` checks the module for the running kernel and an initramfs without NVIDIA or nouveau on every install.
- **Runner VM.** `runner_vm_memory` (16 GiB) sizes the Mac's VM, and `just runner vm` resizes a VM made with other memory.
- Qualcomm SoC firmware stays out of both aarch64 images.

### Screenshots

No UI.

## Review

- Round 1 found three things, all fixed. kiwi's tmpfs was capped at half the RAM, 8 GiB in the Mac's VM, and NVIDIA would have pushed the Fedora aarch64 build past it; it is now `size=75%`. The hardware notes still said `kms` was dropped only on x86_64. Two moved test comments were too long, and one named x86_64's display.
- Round 2 found nothing.
- Round 3, from the first `images` run with 022 (run 42546467105): both aarch64 builds failed. Arch Linux ARM could not resolve `libpisp`, which lives in ALARM's `[alarm]` repo, now added on aarch64. Fedora aarch64 could not install RPM Fusion's driver, which requires a Tegra library no package ships, so Fedora aarch64 keeps NVIDIA to x86_64 for now.

## Manual changes

None.

## Result

- `just lefthook all` passes, `just installer test` passes 66 tests, and shellcheck passes on `tests/vm/arch-aarch64.sh`.
- The four images build and are tested by the `images` workflow, run by hand on this branch: the x86_64 pair on the PC, the aarch64 pair on the Mac's runner VM.
- Docs: `docs/hardware.md` (NVIDIA on aarch64, Qualcomm firmware under Later), `docs/arch-iso.md`, `docs/fedora-iso.md`, `docs/installer.md`, `docs/setup.md` (the VM's memory), and the user page `user/graphics.md`, which now covers all four images and the rebuild after a kernel update on Arch Linux ARM.

## Deviations

- Fedora aarch64 gets no NVIDIA driver, against the ask: RPM Fusion's aarch64 package cannot be installed (Review, round 3). The task file was updated.

- Fedora aarch64 uses task 020's prebuilt `kmod-nvidia`, not akmods on the machine as the operator's plan said: task 020 settled prebuilt modules for Fedora (no compiler in the image). The task says so.
- `just release changed` keeps naming `tests/vm/fedora-x86_64.sh`, as it names `fedora-aarch64.sh`, which does not exist either; the task file was updated.
