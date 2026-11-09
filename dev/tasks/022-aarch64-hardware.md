# 022: NVIDIA and regular hardware on both aarch64 images

- [x] Done

Base: both

## Ask

```text title="Prompt" wrap
Bring both aarch64 images to the hardware support the x86_64 images have, live and installed. NVIDIA: Turing and newer cards get NVIDIA's open modules, older ones nouveau, picked at every boot by stackific-hardware's generator as on x86_64. On Arch Linux ARM, which has no prebuilt module package, use nvidia-open-dkms with nvidia-utils and linux-aarch64-headers: the module builds once while the image is built, and DKMS rebuilds it when an installed system updates its kernel; the compiler and kernel headers this puts in the image are accepted on this one image. On Fedora aarch64, do what Fedora x86_64 does: RPM Fusion's driver with kmod-nvidia built for the image's kernel by `just fedora packages`, no compiler in the image. Arch Linux ARM also gets task 021's regular hardware: linux-firmware-marvell (without Prestera), sof-firmware, alsa-firmware, wireless-regdb, bolt, fprintd (no PAM change), and libcamera with PipeWire's libcamera and V4L2 plugins; Fedora aarch64 has these already. Leave Qualcomm SoC firmware out of both aarch64 images: Snapdragon machines need device trees, which our UEFI-only Limine boot does not load, and it costs about 183 MiB. Installed Arch aarch64 systems drop the kms hook as x86_64 did, so no GPU module goes into the initramfs. Best effort for real hardware: QEMU shows no NVIDIA device, so the VM tests prove the packages and modules are in place and nothing loads. Also give the Mac's runner VM 16 GiB of memory, including a VM made earlier with less. Land it in the same pull request as task 021 (#22).
```

## Goal
Both aarch64 images and their installs carry NVIDIA's driver (open modules for Turing and newer, nouveau for older cards, picked by `stackific-gpu-generator`) and Arch Linux ARM carries the regular-hardware set task 021 gave the other images; the boot and install tests check it on the Mac runner.

## Context
What exists (branch `task/021-hardware-parity-main`, 2026-11-08):
- `arch/iso/packages.aarch64`: `linux-aarch64`, `linux-firmware` (meta) and the desktop; no NVIDIA, no `stackific-hardware`, none of task 021's packages. `arch/iso/packages.x86_64` has them (`linux-firmware-marvell`, `sof-firmware`, `alsa-firmware`, `wireless-regdb`, `bolt`, `fprintd`, `libcamera`, `libcamera-ipa`, `pipewire-libcamera`, `pipewire-v4l2`, `stackific-hardware`, `nvidia-open`, `nvidia-open-lts`, `nvidia-utils`).
- `arch/iso/pacman.conf` is the profile's, used on both architectures; its `NoExtract = usr/lib/firmware/mrvl/prestera/*` applies to aarch64 as is.
- `fedora/iso/config.kiwi` limits `stackific-hardware`, `xorg-x11-drv-nvidia`, `xorg-x11-drv-nvidia-libs` and `nvidia-kmod` to `arch="x86_64"`; `fedora/justfile` `packages` builds `kmod-nvidia-<kernel>` only inside `if [ "{{ arch() }}" = x86_64 ]`. The RPM Fusion repositories in `config.kiwi` and the recipe use `$basearch` metalinks, so they serve aarch64 unchanged. Fedora's aarch64 image already has task 021's packages (one `config.kiwi`).
- `stackific-hardware` is `arch=('any')` (PKGBUILD) and `BuildArch: noarch` (spec); `hardware/nvidia-legacy.ids` and the generator read only sysfs, so both serve aarch64 unchanged.
- `installer/src/base.rs` `arch_hooks_conf(aarch64)` keeps `kms` in installed Arch systems' mkinitcpio hooks on aarch64 only (task 019 dropped it on x86_64 because `autodetect` resolves an NVIDIA card's modalias to nouveau whatever modprobe.d blacklists, putting nouveau and its firmware in the initramfs).
- Tests: `tests/vm/arch-x86_64.sh` holds the NVIDIA, generator and task 021 checks; `tests/vm/fedora-x86_64.sh` holds Fedora's NVIDIA and generator checks; `tests/vm/installed.sh` checks `kmod-nvidia` on Fedora only for `BOOTX64.EFI` and the initramfs for NVIDIA or nouveau modules only on Arch x86_64. `just release changed` already lists `tests/vm/arch-aarch64.sh` and `tests/vm/fedora-aarch64.sh` for the aarch64 images.
- Last aarch64 sizes: Arch aarch64 ISO 1330 MiB (images run 42536619116, 2026-11-08).

Upstream facts:
- Arch Linux ARM aarch64 repos (`http://mirror.archlinuxarm.org/aarch64/{core,extra}/*.db`, read 2026-11-08): `nvidia-open-dkms` 615.78.08-1 (depends `dkms`, `nvidia-utils=615.78.08`; 17.1 MiB download, 110.7 MiB installed), `nvidia-utils` 615.78.08-1 (282.3 / 867.6 MiB), `dkms` 3.4.4-1 (depends `bash coreutils gawk gcc kmod make patch sed`), `linux-aarch64` and `linux-aarch64-headers` 7.3.3-1 (headers 27.6 / 136.3 MiB), `linux-firmware` 20261019-1 (meta: `-amd -amdgpu -atheros -broadcom -cirrus -intel -mediatek -nvidia -other -radeon -realtek -ti`; no `-marvell`, no `-qcom`), `linux-firmware-marvell` and `linux-firmware-qcom` 20261019-1 (qcom 174.4 / 562.0 MiB), `sof-firmware` 2026.09.2-1, `alsa-firmware` 1.2.4-4, `wireless-regdb` 2026.09.03-1, `bolt` 0.9.11-3, `fprintd` 1.94.5-2, `libcamera` and `libcamera-ipa` 0.7.2-4, `pipewire-libcamera` and `pipewire-v4l2` 1:1.6.10-1. No prebuilt `nvidia-open` module package exists for aarch64.
- `linux-aarch64` 7.3.3-1's config (`linux-aarch64-headers`, `build/.config`, read 2026-11-08): `CONFIG_FB_EFI=y`, `CONFIG_DRM_NOUVEAU=m`, `CONFIG_DRM_VIRTIO_GPU=m`; so without `kms` the passphrase prompt draws on the firmware's framebuffer, as on x86_64.
- DKMS's pacman hook (`70-dkms-install.hook`, from `dkms`) builds every DKMS module for each kernel whose headers are installed, so `mkarchiso`'s pacstrap builds `nvidia.ko` into `/usr/lib/modules/<kernel>/updates/dkms/` while the image is built, and a kernel update on an installed system rebuilds it.
- RPM Fusion nonfree, Fedora 45 aarch64 (`https://download1.rpmfusion.org/nonfree/fedora/releases/45/Everything/aarch64/os/Packages/`, read 2026-11-08): `akmod-nvidia`, `xorg-x11-drv-nvidia` and `xorg-x11-drv-nvidia-kmodsrc` 615.78.08-1.fc45, the same version as x86_64.
- RPM Fusion's aarch64 `xorg-x11-drv-nvidia-libs` 615.78.08 requires `libnvidia-rmapi-tegra.so.615.78.08`, which no package in RPM Fusion's release or updates repos ships (CI run 42546467105 and `dnf5 repoquery --forcearch=aarch64 --whatprovides`, 2026-11-08). So Fedora aarch64 gets no NVIDIA driver in this task (found in the build, 2026-11-08; the operator's shipping bias, AGENTS.md).
- Arch Linux ARM's `libcamera` depends on `libpisp`, which is in ALARM's `[alarm]` repo (`http://mirror.archlinuxarm.org/aarch64/alarm/alarm.db`, `libpisp` 1.5.0-1, 2026-11-08); the profile's `pacman.conf` lists only `[stackific]`, `[core]` and `[extra]`, and Arch's archive has no `alarm` repo, so `just arch iso` appends `[alarm]` on aarch64 only (`arch/justfile` joins the Changes).
- Fedora 45 aarch64 has `qcom-firmware` 20261019-1 (noarch); the image does not install it (only `qcom-wwan-firmware`).

## Inputs
From the operator's plan, row 022: "NVIDIA and firmware on aarch64 (`nvidia-open-dkms` on ALARM, akmods on Fedora); Mac lane; depends on 018 and 021; best effort for real hardware, VM tests only prove the packages install." Deviation: Fedora aarch64 does not use akmods on the machine but task 020's prebuilt `kmod-nvidia`, because task 020 settled prebuilt modules for Fedora (no compiler in the image) and RPM Fusion has the same packages for aarch64. Operator decisions (2026-11-08): `nvidia-open-dkms` on Arch Linux ARM, and Qualcomm SoC firmware left out. No files copied.

## Changes
- `arch/iso/packages.aarch64`: add `linux-aarch64-headers`, `stackific-hardware`, `nvidia-open-dkms`, `nvidia-utils`, and task 021's ten packages.
- `fedora/iso/config.kiwi`: keep the four NVIDIA lines `arch="x86_64"` (see the RPM Fusion fact) and say why in its comment; `fedora/justfile` `iso`: kiwi's tmpfs `size=75%`.
- `fedora/justfile` `packages`: keep the `kmod-nvidia` build to x86_64, with the reason in its comment.
- `installer/src/base.rs`: no `kms` on aarch64 either (drop `arch_hooks_conf`'s parameter and its callers' argument); update its unit test.
- `tests/vm/arch.sh`: take the generator and task 021 checks from `tests/vm/arch-x86_64.sh`, which keeps the linux-lts and `nvidia-open` checks.
- `tests/vm/arch-aarch64.sh` (new): source `arch.sh`, then `nvidia-open-dkms`, `nvidia-utils`, `linux-aarch64-headers` installed, `dkms status` reports `nvidia` installed for the running kernel, and `nvidia.ko`'s vermagic matches `uname -r`.
- `tests/vm/fedora-x86_64.sh` keeps Fedora's NVIDIA checks; `tests/vm/installed.sh` keeps the `kmod-nvidia` check to `BOOTX64.EFI`.
- `tests/vm/installed.sh`: Fedora's `kmod-nvidia` check on both architectures; Arch's no-GPU-module-in-initramfs check on both architectures; on Arch aarch64, the DKMS module for the installed kernel.
- `docs/hardware.md`, `docs/fedora-iso.md`, `docs/arch-iso.md`, `docs/installer.md`: aarch64 NVIDIA and firmware, why DKMS on Arch Linux ARM, why no Qualcomm firmware; Later list updated.
- `site/src/content/docs/user/graphics.md`: all four images.
- `AGENTS.md`: the hardware lines for aarch64.
- `arch/justfile` `iso`: append `[alarm]` to the copied profile's `pacman.conf` on aarch64.
- `just/runner/vars.just`, `just/runner/mod.just` `vm`, `docs/setup.md`: the runner VM's memory as `runner_vm_memory` (16 GiB); `just runner vm` resizes an existing VM with other memory (`limactl stop`, `limactl edit --memory`, `limactl start`) (operator 2026-11-08).
- `site/src/content/docs/prompt-trail/022-aarch64-hardware.md`, `dev/reviews/022-aarch64-hardware.md`.

## Acceptance criteria

Must:
- `just arch test` on aarch64 passes with `arch-aarch64.sh`: the 14 new packages installed, `dkms status` lists `nvidia/615.78.08` installed for `uname -r`, `modinfo -F vermagic nvidia` starts with `uname -r`, no `/usr/lib/firmware/mrvl/prestera`, the generator present and writing nothing on QEMU, no `nvidia` or `nouveau` module loaded.
- `just fedora test` on aarch64 passes, without NVIDIA packages; on x86_64 it passes `fedora-x86_64.sh`'s NVIDIA checks as before.
- `just arch test-install` and `just fedora test-install` on aarch64 pass `installed.sh` with the module present for the installed kernel and no NVIDIA or nouveau module in the initramfs; the unlock test still shows the passphrase prompt on Arch aarch64 without `kms`.
- Both x86_64 images still pass their boot and install tests (the moved checks unchanged).
- Each image task reports its ISO size; the `images` workflow builds all four.
- On the Mac, `limactl list --format '{{.Memory}}' stackific-runner` prints `17179869184` after `just runner vm`.
- `just lefthook all` and `cargo test` in `installer/` pass.

Must not:
- Add `linux-firmware-qcom` or `qcom-firmware`.
- Put a compiler, kernel headers or akmods on any image other than Arch aarch64.
- Change the generator or `nvidia-legacy.ids`.

## Tests
- `just arch test`, `just arch test-install`, `just fedora test`, `just fedora test-install` on the Mac runner through the `images` workflow (`workflow_dispatch` on the branch builds all four).
- The same on the PC for x86_64.
- `just installer test` for the hooks text.
1. On the Mac: `just runner vm`, then `limactl list stackific-runner` shows `16GiB` under MEMORY.
- Real NVIDIA hardware on aarch64 (an Ampere workstation with a Turing card) is not available; recorded under Later in `docs/hardware.md`.
