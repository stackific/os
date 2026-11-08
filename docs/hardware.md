# Hardware

`hardware/` holds what picks drivers by hardware: today the NVIDIA driver on both x86_64 images (Arch: task 019; Fedora: task 020), packaged as `stackific-hardware` (`arch/packages/stackific-hardware/PKGBUILD`, `fedora/packages/stackific-hardware/stackific-hardware.spec`). The user page is `site/src/content/docs/user/graphics.md`.

## Test

```sh
just hardware test   # the generator against fake sysfs trees of recorded PCI sets
```

Each set is a list of `vendor:device:class`; the recipe writes them as `sys/bus/pci/devices/*/{vendor,device,class}` under a temporary root, runs the generator with `STACKIFIC_ROOT` set to it, and compares the files it wrote. The boot checks are in `tests/vm/arch-x86_64.sh` (live) and `tests/vm/installed.sh` (installed); QEMU cannot show an NVIDIA device, so the VM tests see only the "no NVIDIA" case. The install test also starts the installed system through Limine's `(LTS kernel)` entry and checks `uname -r`.

## The NVIDIA driver on Arch x86_64

- **Packages.** The image carries `linux` and `linux-lts`, Arch's prebuilt `nvidia-open` and `nvidia-open-lts`, and `nvidia-utils`, all from the archive date in `arch/vars.just`, so each module matches its kernel (the boot test compares `modinfo -F vermagic` with each kernel). No DKMS, no compiler, no headers. The installer copies the live root, so every install carries them too, on any hardware: a GPU swap needs no package change.
- **The pick, at every boot.** `nvidia-utils` blacklists nouveau (`/usr/lib/modprobe.d/nvidia-utils.conf`), and NVIDIA's open module drives only Turing and newer, so a GTX 10 card would get no driver. `stackific-gpu-generator`, a systemd generator (`/usr/lib/systemd/system-generators/`), runs again after switch-root and before udev's coldplug, and reads the display controllers (PCI class `0x03xxxx`) from sysfs, never through lspci, so a sleeping dGPU stays asleep:
  - all NVIDIA display controllers in `nvidia-legacy.ids`: `/run/modprobe.d/nvidia-utils.conf`, which shadows the package's file (kmod reads `/run` first, one file per name), blacklists `nvidia`, `nvidia_drm`, `nvidia_modeset`, `nvidia_uvm` and nova; udev then loads nouveau;
  - and no other display controller: `/run/stackific/mango-gpu.conf` links `mango-low-effects.conf` (blur and shadows off, 150 ms animations). Every user's `~/.config/mango/config.conf` includes it with `source_optional=`, which MangoWM skips when the file is missing; later lines in the user's file win;
  - any NVIDIA display controller: `greetd.service` gets `Wants=` and `After=systemd-udev-settle.service` (a drop-in in the generator's output), so the session starts on the loaded driver, not on the firmware's framebuffer it takes over.
- **Initramfs.** No GPU module in it: the live image's hooks have no `kms`, and the installer leaves `kms` out of installed systems' hooks on x86_64. mkinitcpio's `autodetect` resolves an NVIDIA card's modalias with `modprobe -qaR`, which ignores blacklists (checked 2026-11-08 in the builder, `linux` 7.2.11 with `nvidia-utils` 615.78.08: it prints `nouveau` for a Pascal modalias either way), so `kms` would pull nouveau and its firmware into the initramfs of every NVIDIA machine. AMD and Intel lose early KMS with it: the passphrase prompt draws on the firmware's framebuffer.
- **The fallback kernel.** The live ISO boots `linux` only: mkarchiso copies every `/boot/vmlinuz-*` and `initramfs-*.img` of the live root to the ISO and to its boot image, so the live-only pacman hook `arch/iso/airootfs/etc/pacman.d/hooks/stackific-live-lts.hook` removes `linux-lts`'s two files after mkinitcpio's own hook has made them (a preset with `PRESETS=()` is not enough: mkinitcpio's hook script copies the kernel anyway). The installer removes that hook; the kernel step then builds both kernels' files on the ESP, and Limine gets `/<NAME> (LTS kernel)` after the default entry.

## The NVIDIA driver on Fedora x86_64

The same generator and the same picks, with RPM Fusion's driver (`xorg-x11-drv-nvidia` and its libraries) and NVIDIA's open modules built into the image by `just fedora packages` (`docs/fedora-iso.md`, "Repositories"): no compiler, kernel headers or akmods on the image. The generator also hands NVIDIA graphics to nouveau when the running kernel has no `nvidia.ko` (it looks under `/usr/lib/modules/$(uname -r)`): after a Fedora kernel update, until our repos publish `kmod-nvidia` for each kernel (task 028), the new kernel starts on nouveau, and Limine still lists the previous kernels, which have the module. Fedora's own `nvidia-utils.conf` (in `stackific-hardware`) blacklists nouveau, because RPM Fusion does it only on GRUB's command line; a dracut drop-in keeps nouveau out of the initramfs, while AMD and Intel keep early KMS (Fedora's dracut adds only the machine's driver).

SPEC open decision 6 (operator's design notes: prebuilt kmods in the image, or akmods on first boot) is settled as prebuilt: akmods on the live ISO would compile for minutes on every boot, and keeping `akmod-nvidia` for later kernels would put gcc, kernel headers and 90 MiB of module sources into every image.

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
- The Arch x86_64 ISO is 2041 MiB with NVIDIA and `linux-lts`, 7 MiB under the 2 GiB target: the next additions (task 021's firmware) need room, or the R2 path in `AGENTS.md` (CI and releases).
- Broadcom STA and fingerprint readers on detection (task 021); whether chwd joins for them.
- `kmod-nvidia` for each new Fedora kernel from our repos (task 028); until then later kernels start on nouveau.
- NVIDIA on aarch64 (task 022).
- Real NVIDIA hardware: a Turing desktop, a Pascal desktop, a Pascal Optimus laptop, and an RTX to Pascal swap on one disk (task 030).
