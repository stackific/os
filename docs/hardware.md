# Hardware

`hardware/` holds what picks drivers by hardware: the NVIDIA driver on all four images (Arch x86_64: task 019; Fedora x86_64: task 020; both aarch64 images: task 022), packaged as `stackific-hardware` (`arch/packages/stackific-hardware/PKGBUILD`, `fedora/packages/stackific-hardware/stackific-hardware.spec`). The user page is `site/src/content/docs/user/graphics.md`.

## Test

```sh
just hardware test   # the generator against fake sysfs trees of recorded PCI sets
```

Each set is a list of `vendor:device:class`; the recipe writes them as `sys/bus/pci/devices/*/{vendor,device,class}` under a temporary root, runs the generator with `STACKIFIC_ROOT` set to it, and compares the files it wrote. The boot checks are in `tests/vm/arch.sh` and `tests/vm/fedora.sh` (live, both architectures), `tests/vm/arch-x86_64.sh` and `tests/vm/arch-aarch64.sh` (each architecture's modules), and `tests/vm/installed.sh` (installed); QEMU cannot show an NVIDIA device, so the VM tests see only the "no NVIDIA" case. The install test also starts the installed system through Limine's `(LTS kernel)` entry and checks `uname -r`.

## The NVIDIA driver on Arch x86_64

- **Packages.** The image carries `linux` and `linux-lts`, Arch's prebuilt `nvidia-open` and `nvidia-open-lts`, and `nvidia-utils`, all from the archive date in `arch/vars.just`, so each module matches its kernel (the boot test compares `modinfo -F vermagic` with each kernel). No DKMS, no compiler, no headers. The installer copies the live root, so every install carries them too, on any hardware: a GPU swap needs no package change.
- **The pick, at every boot.** `nvidia-utils` blacklists nouveau (`/usr/lib/modprobe.d/nvidia-utils.conf`), and NVIDIA's open module drives only Turing and newer, so a GTX 10 card would get no driver. `stackific-gpu-generator`, a systemd generator (`/usr/lib/systemd/system-generators/`), runs again after switch-root and before udev's coldplug, and reads the display controllers (PCI class `0x03xxxx`) from sysfs, never through lspci, so a sleeping dGPU stays asleep:
  - all NVIDIA display controllers in `nvidia-legacy.ids`: `/run/modprobe.d/nvidia-utils.conf`, which shadows the package's file (kmod reads `/run` first, one file per name), blacklists `nvidia`, `nvidia_drm`, `nvidia_modeset`, `nvidia_uvm` and nova; udev then loads nouveau;
  - and no other display controller: `/run/stackific/mango-gpu.conf` links `mango-low-effects.conf` (blur and shadows off, 150 ms animations). Every user's `~/.config/mango/config.conf` includes it with `source_optional=`, which MangoWM skips when the file is missing; later lines in the user's file win;
  - any NVIDIA display controller: `greetd.service` gets `Wants=` and `After=systemd-udev-settle.service` (a drop-in in the generator's output), so the session starts on the loaded driver, not on the firmware's framebuffer it takes over.
- **Initramfs.** No GPU module in it: the live image's hooks have no `kms`, and the installer leaves `kms` out of installed systems' hooks on both architectures (aarch64 since task 022). mkinitcpio's `autodetect` resolves an NVIDIA card's modalias with `modprobe -qaR`, which ignores blacklists (checked 2026-11-08 in the builder, `linux` 7.2.11 with `nvidia-utils` 615.78.08: it prints `nouveau` for a Pascal modalias either way), so `kms` would pull nouveau and its firmware into the initramfs of every NVIDIA machine. AMD and Intel lose early KMS with it: the passphrase prompt draws on the firmware's framebuffer.
- **The fallback kernel.** The live ISO boots `linux` only: mkarchiso copies every `/boot/vmlinuz-*` and `initramfs-*.img` of the live root to the ISO and to its boot image, so the live-only pacman hook `arch/iso/airootfs/etc/pacman.d/hooks/stackific-live-lts.hook` removes `linux-lts`'s two files after mkinitcpio's own hook has made them (a preset with `PRESETS=()` is not enough: mkinitcpio's hook script copies the kernel anyway). The installer removes that hook; the kernel step then builds both kernels' files on the ESP, and Limine gets `/<NAME> (LTS kernel)` after the default entry.

## The NVIDIA driver on Fedora x86_64

The same generator and the same picks, with RPM Fusion's driver (`xorg-x11-drv-nvidia` and its libraries) and NVIDIA's open modules built into the image by `just fedora packages` (`docs/fedora-iso.md`, "Repositories"): no compiler, kernel headers or akmods on the image. The generator also hands NVIDIA graphics to nouveau when the running kernel has no `nvidia.ko` (it looks under `/usr/lib/modules/$(uname -r)`): after a Fedora kernel update, until our repos publish `kmod-nvidia` for each kernel (task 028), the new kernel starts on nouveau, and Limine still lists the previous kernels, which have the module. Fedora's own `nvidia-utils.conf` (in `stackific-hardware`) blacklists nouveau, because RPM Fusion does it only on GRUB's command line; a dracut drop-in keeps nouveau out of the initramfs, while AMD and Intel keep early KMS (Fedora's dracut adds only the machine's driver).

SPEC open decision 6 (operator's design notes: prebuilt kmods in the image, or akmods on first boot) is settled as prebuilt: akmods on the live ISO would compile for minutes on every boot, and keeping `akmod-nvidia` for later kernels would put gcc, kernel headers and 90 MiB of module sources into every image.

## NVIDIA on aarch64

Task 022 brings the driver and the same pick to both aarch64 images (NVIDIA cards in aarch64 workstations and servers with PCIe slots, such as Ampere Altra machines). Best effort: QEMU shows no NVIDIA device, so the VM tests prove the packages and modules are in place and that nothing loads on QEMU's display.

- **Fedora aarch64** is the same as x86_64: RPM Fusion has the same packages for aarch64 (615.78.08, 2026-11-08), and `just fedora packages` builds `kmod-nvidia-<kernel>` natively in the Mac's runner VM.
- **Arch Linux ARM** has `nvidia-open-dkms` and `nvidia-utils` (615.78.08) but no prebuilt module package, so the image carries `nvidia-open-dkms` with `dkms` and `linux-aarch64-headers` (operator 2026-11-08). DKMS builds the module while the image is built, so the live image compiles nothing; an installed system rebuilds it when `linux-aarch64` updates, so its kernel updates never fall back to nouveau. The price is `gcc`, `make` and the headers in this one image.
- **Initramfs.** Installed Arch systems on aarch64 drop `kms` as x86_64 did, for the same reason. `linux-aarch64` has `CONFIG_FB_EFI=y`, so the passphrase prompt draws on the firmware's framebuffer.

## Regular hardware

Task 021 gives both x86_64 images the same regular hardware (task 022 adds Arch Linux ARM, with the same packages; Fedora aarch64 has shared Fedora's list from the start), installed on every image and install, with no detection step and no chwd:

| | Arch (`arch/iso/packages.x86_64`, `packages.aarch64`) | Fedora (`fedora/iso/config.kiwi`, both architectures) |
| --- | --- | --- |
| Firmware | `linux-firmware` meta, plus `linux-firmware-marvell`, `sof-firmware`, `alsa-firmware`, `wireless-regdb` | its firmware subpackages, plus `alsa-firmware` and `dvb-firmware` |
| Thunderbolt and USB4 | `bolt` | `bolt` |
| Fingerprint readers | `fprintd` (its `pam_fprintd` is in no PAM stack) | `fprintd`, no `fprintd-pam` |
| MIPI webcams (Intel IPU6) | `libcamera`, `libcamera-ipa`, `pipewire-libcamera`, `pipewire-v4l2` | `libcamera`, `libcamera-ipa`, `pipewire-plugin-libcamera`, `pipewire-v4l2` |

- **Parity rule.** Compared by top-level entry under `/usr/lib/firmware` (package file lists, compression suffixes ignored, 2026-11-08), Arch's set had and Fedora's lacked only TV-tuner (DVB, analog) and old sound-card firmware, which Fedora's `alsa-firmware` and `dvb-firmware` carry; Fedora's set had and Arch's lacked only server parts (QLogic, Brocade and others), which both leave out. The boot tests check one sentinel file per family on both images.
- **Prestera.** 71 of `linux-firmware-marvell`'s 78 MiB are Marvell's Prestera Ethernet-switch firmware (`mrvl/prestera/`), a datacenter part. `arch/iso/pacman.conf` keeps it out of the image with `NoExtract`. Installed systems use Arch's own `/etc/pacman.conf`, so a later `linux-firmware-marvell` update writes those files there (disk only, never loaded). Fedora ships Prestera as its own package, `mrvlprestera-firmware`, which the image leaves out.
- **bolt** ships upstream's polkit rule, which lets an active local `wheel` user authorise Thunderbolt devices without a password, as Fedora Workstation does; we keep it unchanged (operator 2026-11-08).
- **fprintd** is D-Bus activated; `fprintd-enroll` and `fprintd-verify` work, but login, `sudo` and polkit still ask only for the password, because none of their PAM stacks calls `pam_fprintd`. The desktops' own fingerprint services stay as packaged: Plasma's lock screen (kscreenlocker's `kde-fingerprint`) offers an enrolled finger next to the password, and GDM's `gdm-fingerprint` is unused (GDM does not run).
- **Broadcom STA** (`wl`, BCM4311 to BCM4360, laptops from about 2008 to 2015): Arch has only `broadcom-wl-dkms` and RPM Fusion only `akmod-wl`, both of which compile on the machine, so the images carry neither; the user page gives the online steps (operator 2026-11-08).

## The legacy list

`hardware/nvidia-legacy.ids` lists the PCI device IDs (vendor `10de`) of NVIDIA's legacy-branch tables (580.xx and older) in `supportedchips.html`, which `nvidia-utils` ships as `/usr/share/doc/nvidia/html/supportedchips.html`: 879 IDs in 615.78.08. MX150 (`1d10`) is in it; MX450 (`1f97`) and GTX 1650 Mobile (`1f99`), Turing chips, are not. The pre-Turing set is closed, so driver updates do not change it. Regenerate it only if NVIDIA moves Turing to a legacy branch: take the four-digit first column of every table row after the "legacy GPUs" heading, lowercased and sorted, for example

```sh
sed -n '/legacy GPUs that are no longer supported/,$p' supportedchips.html \
  | grep -oE '<td>[0-9A-F]{4}( [0-9A-F]{4} [0-9A-F]{4})?</td>' | cut -c5-8 | tr A-F a-f | sort -u
```

and update the count in `just hardware test`.

## Later

- The installer's warning before installing on a pre-Turing NVIDIA card (slow 3D), and the same note once in the live session.
- A free-space guard on the ESP as kernels add up.
- Fingerprint login: wire `pam_fprintd` after the user enrols a finger, with an enrolment page.
- Broadcom STA offline, if a prebuilt `wl` module per kernel becomes worth building (as `kmod-nvidia` on Fedora).
- Qualcomm SoC firmware (Arch's `linux-firmware-qcom`, Fedora's `qcom-firmware`, about 175 MiB compressed) on the aarch64 images, once they boot Snapdragon machines: those need device trees, which our UEFI-only Limine boot does not load (operator 2026-11-08, task 022).
- `kmod-nvidia` for each new Fedora kernel from our repos (task 028); until then later kernels start on nouveau.
- Real NVIDIA hardware on aarch64 (an Ampere workstation with a Turing card).
- Real NVIDIA hardware: a Turing desktop, a Pascal desktop, a Pascal Optimus laptop, and an RTX to Pascal swap on one disk (task 030).
