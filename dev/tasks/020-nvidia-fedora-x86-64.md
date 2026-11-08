# 020: NVIDIA on Fedora x86_64: open modules built into the image, the driver picked at every boot, nouveau for older cards

- [ ] Done

Base: fedora

## Ask

```text title="Prompt" wrap
Make NVIDIA graphics work on the Fedora x86_64 image, live and installed, as task 019 did on Arch x86_64. Take NVIDIA's driver from RPM Fusion nonfree, its packages and metadata checked against RPM Fusion's key committed in the repo and the repository limited to the NVIDIA packages, as Terra is. Build NVIDIA's open kernel modules for the image's kernel while the image is built, with RPM Fusion's own akmod sources, and leave the compiler and the kernel headers out of the finished image, so the live session and installs need no compiler, no network and no wait. Turing and newer cards run NVIDIA's open modules; older cards run nouveau, decided at every boot by the same generator as on Arch (stackific-hardware, from hardware/), packaged as an RPM. RPM Fusion blacklists nouveau through grubby on GRUB's command line, which Limine systems never read, so ship the nouveau blacklist as a modprobe.d file; and when the running kernel has no NVIDIA module (a later kernel update), let nouveau drive the card instead of leaving a black screen. Keep NVIDIA and nouveau modules out of the initramfs. Give the live user the same MangoWM config as on Arch. Fedora has no chwd: decide whether the planned Fedora detection step is still needed. Report the ISO size and keep it within 2 GiB.
```

## Goal
The Fedora x86_64 live ISO and every install from it carry RPM Fusion's NVIDIA driver with open kernel modules built for the image's kernel, no compiler and no kernel headers; `stackific-hardware` picks NVIDIA or nouveau at every boot, as on Arch; the generator also falls back to nouveau when the running kernel has no NVIDIA module.

## Context
What exists (task 019's branch, checked 2026-11-08):
- `hardware/stackific-gpu-generator` (task 019): a systemd generator that reads display controllers from sysfs; when every NVIDIA display controller is in `hardware/nvidia-legacy.ids`, it writes `/run/modprobe.d/nvidia-utils.conf` (shadowing Arch's `nvidia-utils` file of that name: `blacklist nvidia`, `nvidia_drm`, `nvidia_modeset`, `nvidia_uvm`, nova's two), and, with no other display controller, links `/run/stackific/mango-gpu.conf` to `/usr/share/stackific/mango-low-effects.conf`; with any NVIDIA display controller it orders `greetd.service` after `systemd-udev-settle.service`. `just hardware test` runs it against fake sysfs trees (`STACKIFIC_ROOT`). Packaged for Arch only (`arch/packages/stackific-hardware/PKGBUILD`); details in `docs/hardware.md`.
- `fedora/iso/config.kiwi` builds both Fedora images from Fedora, Terra (`terra-repo.sh` appends `includepkgs` and `gpgkey`; key committed as `fedora/iso/terra-45.asc`) and our local repo; named packages only, no weak dependencies; `arch="x86_64"` keeps a package to one image. `fedora/iso/config.sh` runs in the image root after the packages are installed: it creates `live` with `useradd --create-home` and enables greetd. Fedora's live user has no `~/.config/mango/config.conf` today, so it gets MangoWM's stock config and no Super+Space launcher binding.
- The kernel is `kernel` 7.2.15-300.fc45 (Fedora 45 updates, `dnf5 repoquery --latest-limit 1 kernel`, 2026-11-08). The image ships `nvidia-gpu-firmware` (nouveau's GSP firmware).
- The installer copies the live root; Fedora's live state removes `grub2-efi-x64`, `-cdboot` and `-modules` with `dnf5 --installroot` (`installer/src/base.rs`, `live_grub`), and `tests/vm/installed.sh` already checks `rpm -qa 'grub2*' grubby` prints nothing on the installed system. Installed systems run kernel-install with `layout=other` and stackific-boot's Limine plugin, which keeps three kernels in `limine.conf`.
- The latest Fedora x86_64 ISO is 1502 MiB (task 018's prompt-trail entry).

RPM Fusion facts (read 2026-11-08 in `localhost/stackific-fedora-builder:45` after installing `rpmfusion-nonfree-release-45`, and https://rpmfusion.org/Howto/NVIDIA):
- `rpmfusion-nonfree-release` 45-2 installs `/usr/share/dnf5/repos.d/rpmfusion-nonfree.repo` (`metalink=https://mirrors.rpmfusion.org/metalink?repo=nonfree-fedora-$releasever&arch=$basearch`, `gpgcheck=1`, `repo_gpgcheck=0`) and `rpmfusion-nonfree-updates.repo` (`repo=nonfree-fedora-updates-released-$releasever`). Its Fedora 45 key `/usr/share/pki/rpm-gpg/RPM-GPG-KEY-rpmfusion-nonfree-fedora-45` links the 2020 key: fingerprint `79BD B88F 9BBF 7391 0FD4 095B 6A2A F961 9484 3C65`, sha256 `7ac65a8dfeced70c8c335862b5f0696d57abac378deda34c4af78cd2220e3de6`; packages are signed with key ID `6a2af96194843c65`. Repository metadata is not signed (`repo_gpgcheck=0`).
- Driver 615.78.08-1.fc45 (nonfree): `xorg-x11-drv-nvidia` (133 MiB download, 185 MiB installed), `xorg-x11-drv-nvidia-libs` (100 MiB, 357 MiB), `xorg-x11-drv-nvidia-cuda-libs` (91 MiB, 487 MiB; required by `-libs`), `nvidia-settings` 4.5 MiB, `nvidia-modprobe`; `akmod-nvidia` (45 KiB) with `xorg-x11-drv-nvidia-kmodsrc` (90 MiB download). `akmods` and `kmodtool` come from Fedora itself. RPM Fusion publishes no prebuilt `kmod-nvidia-<kernel>` packages for Fedora 45: `kmod-nvidia` is only a metapackage that requires `akmod-nvidia`.
- `akmods` requires `gcc`, `make`, `rpm-build`, `kernel-devel-matched` (for `kernel-core`) and more: adding `akmod-nvidia` to the image pulls a compiler and kernel headers (gcc 122 MiB installed, kernel-devel about 90 MiB).
- `nvidia-kmod.spec` (inside `/usr/src/akmods/nvidia-kmod-615.78.08-1.fc45.src.rpm`): `--with kmod_nvidia_open` (macro `_with_kmod_nvidia_open`) builds the open modules unconditionally; without it, `nvidia-kmod-noopen-checks` runs `lspci` on the build machine and builds the closed modules if one of 160 listed GPUs is present. The open modules' `modinfo -F license` is `Dual MIT/GPL`.
- `xorg-x11-drv-nvidia` ships no modprobe.d file: its `%posttrans` runs `grubby --update-kernel=ALL --args='rd.driver.blacklist=nouveau,nova_core modprobe.blacklist=nouveau,nova_core'` (grubby is `Requires(post)` only; `grubby` requires `grub2-tools`, whose `grub2-common` holds GRUB's kernel-install plugin). It ships `/usr/lib/dracut/dracut.conf.d/99-nvidia-dracut.conf` (`omit_drivers+=" nvidia nvidia-drm nvidia-modeset nvidia-uvm "`), `/usr/lib/udev/rules.d/10-nvidia.rules` and `nvidia-fallback.service`, whose `ConditionKernelCommandLine=rd.driver.blacklist=nouveau,nova_core` never holds on our systems, the GSP firmware in `/usr/lib/firmware/nvidia/615.78.08/`, and `Requires: nvidia-kmod >= 3:615.78.08` (provided by `akmod-nvidia` or a built `kmod-nvidia-<kernel>`).
- Neither package loads `nvidia_drm` itself; Arch's `nvidia-utils` does it with `softdep nvidia post: nvidia-uvm nvidia-drm` (task 019).

## Inputs
SPEC open decision 6 (the operator's design notes, restated): NVIDIA on Fedora either as kmods prebuilt in the image (Universal Blue's pattern), with akmods for later kernels, so the driver is "included" with no build wait at first boot and no signing (Secure Boot is off); or RPM Fusion's akmods building on first boot (simpler image build, up to 5 minutes before the driver loads). The operator's earlier hardware design: Fedora has no chwd, so a small detection step of our own would install RPM Fusion packages that match the hardware (NVIDIA akmods, Broadcom `wl`).

Recommendation for decision 6, which this task follows (the operator may overrule it in review):
- **Prebuilt open modules in the image, built while the image is built; no akmods on the image.** The live ISO must work offline and at once, which rules out akmods on first boot (every live boot would compile for minutes). Keeping `akmod-nvidia` for later kernels would ship gcc, kernel headers and the 90 MiB module sources in every ISO and install, against "no compiler", and would likely push the ISO past 2 GiB.
- **Later kernels:** until our package repos are online (task 028, which then publishes `kmod-nvidia` for each new Fedora kernel), a kernel update on an installed system leaves the new kernel without NVIDIA's module. The generator then lets nouveau drive Turing and newer cards (GSP firmware, so they reclock), and Limine still lists the two previous kernels, which have the module. Logged in `docs/hardware.md`.

The detection step: with every GPU package on every image and install and the generator picking at boot (task 019's design), NVIDIA needs no install-time detection on Fedora either. Broadcom `wl` (task 021) is where hardware-dependent installs come back; the step is decided there.

No untracked files are copied: the task reuses `hardware/nvidia-legacy.ids` and `hardware/mango-low-effects.conf` from task 019.

## Design
- **RPM Fusion in kiwi.** `fedora/iso/config.kiwi` adds `rpmfusion-nonfree` and `rpmfusion-nonfree-updates` (the metalinks above, `package_gpgcheck="true"`, `customize="rpmfusion-repo.sh"`), with the key committed as `fedora/iso/rpmfusion-nonfree-45.asc` (the file above, checked against the fingerprint and sha256). `fedora/iso/rpmfusion-repo.sh` appends `includepkgs=xorg-x11-drv-nvidia,xorg-x11-drv-nvidia-libs,xorg-x11-drv-nvidia-cuda-libs,nvidia-modprobe,nvidia-settings` (the five packages the driver pulls from RPM Fusion, checked with a dry-run install on 2026-11-08) and the `gpgkey` line, as `terra-repo.sh` does. `akmod-nvidia` is left out, so `nvidia-kmod` resolves to our module package. No RPM Fusion repo file is left in the image (as with Terra; installed systems get updates from our repos, task 028).
- **Packages** (`arch="x86_64"`; the aarch64 image gets NVIDIA in task 022): `stackific-hardware`, `xorg-x11-drv-nvidia`, `xorg-x11-drv-nvidia-libs`, `nvidia-kmod`.
- **The modules, built by `just fedora packages`, not in the image.** On x86_64 the recipe downloads `akmod-nvidia` and `xorg-x11-drv-nvidia-kmodsrc` from RPM Fusion into a local repo, checks both with `rpmkeys -K` against the committed key, takes `nvidia-kmod-<version>.src.rpm` out of `akmod-nvidia`, and rebuilds it with mock (`--addrepo` the local repo, `--additional-package kernel-devel-<kver>`, `--define "kernels <kver>"`, `--with kmod_nvidia_open`) for the newest `kernel-core` in Fedora's repos, the one kiwi installs. `kmod-nvidia-<kver>` goes into `.build/fedora/repo/` with our RPMs. The image never holds gcc, kernel headers or `akmod-nvidia`, and no kiwi uninstall step is needed. (Prototyped 2026-11-08: two minutes; the RPM provides `nvidia-kmod = 3:615.78.08-1.fc45`, vermagic `7.2.15-300.fc45.x86_64`. The first try failed in `%prep` with "kmodtool error: … source/Makefile not found", because the source RPM names no kernel headers when `kernels` is set, hence `--additional-package`.) This is also how task 028 can publish `kmod-nvidia` for later kernels.
- **`stackific-hardware` RPM** (`fedora/packages/stackific-hardware/stackific-hardware.spec`, sources from `hardware/`, same install paths as the PKGBUILD) plus two Fedora-only files beside the spec:
  - `/usr/lib/modprobe.d/nvidia-utils.conf`: `blacklist nouveau`, `blacklist nova_core`, `blacklist nova_drm`, `softdep nvidia post: nvidia-uvm nvidia-drm`. The same name as Arch's file, so the generator's `/run/modprobe.d/nvidia-utils.conf` shadows it on both bases.
  - `/usr/lib/dracut/dracut.conf.d/99-stackific-gpu.conf`: `omit_drivers+=" nouveau nova_core nova_drm "`, so dracut (hostonly on installs, generic in the live image) leaves nouveau out; RPM Fusion's file already omits NVIDIA's modules. AMD and Intel keep early KMS on Fedora.
- **Generator change (both bases, `hardware/stackific-gpu-generator`):** nouveau also when NVIDIA display controllers are present but the running kernel has no NVIDIA module (`find /usr/lib/modules/$(uname -r) -name 'nvidia.ko*'` finds nothing, under `STACKIFIC_ROOT` in tests). The low-effects link stays for pre-Turing cards only (Turing and newer reclock on nouveau with GSP). Arch's behaviour does not change while `nvidia-open` matches its kernel.
- **Live user's MangoWM config:** `fedora/iso/config.sh` writes `/home/live/.config/mango/config.conf` after `useradd`, owned by `live`, with the same three lines as `arch/iso/airootfs/home/live/.config/mango/config.conf`.
- **grubby:** if `grubby` (and with it `grub2-tools` and `grub2-common`) is still on the installed system after the live state (the existing `rpm -qa 'grub2*' grubby` check fails), the Fedora live state removes it too; it is only a `Requires(post)` of `xorg-x11-drv-nvidia`.

## Changes
- `hardware/stackific-gpu-generator` (the missing-module rule), `hardware/justfile` (fake modules in the NVIDIA sets; new sets: Turing without a module for the running kernel; Pascal without a module).
- `fedora/packages/stackific-hardware/stackific-hardware.spec` (new), `fedora/packages/stackific-hardware/nvidia-utils.conf` (new), `fedora/packages/stackific-hardware/99-stackific-gpu.conf` (new)
- `arch/packages/stackific-hardware/PKGBUILD` (the generator's new sha256, `pkgrel` bump).
- `fedora/iso/config.kiwi`, `fedora/iso/config.sh` (the live user's MangoWM config), `fedora/iso/rpmfusion-repo.sh` (new), `fedora/iso/rpmfusion-nonfree-45.asc` (new, the key above); `fedora/justfile` (`packages`: stage `hardware/`, build `kmod-nvidia-<kver>` on x86_64).
- `installer/src/base.rs` and its tests: Fedora x86_64's live state also removes `grubby` (in the image only as `xorg-x11-drv-nvidia`'s `Requires(post)`; it keeps `grub2-tools`; checked 2026-11-08: `dnf5 remove grubby` succeeds with the driver installed).
- `tests/vm/fedora-x86_64.sh` (new: sources `fedora.sh`, then the NVIDIA checks), `tests/vm/installed.sh` (Fedora x86_64: no `nouveau` or `nvidia` modules in the initramfs).
- `release/justfile` (`just release changed`: `hardware/` belongs to the Fedora images too).
- `docs/hardware.md`, `docs/fedora-iso.md`, `site/src/content/docs/user/graphics.md`, `site/src/content/docs/user/system-requirements.md`, `site/src/content/docs/user/installation.md`, `release/notes.md`, `AGENTS.md`.
- The prompt-trail entry `site/src/content/docs/prompt-trail/fedora/020-nvidia-fedora-x86-64.md` and the review record `dev/reviews/020-nvidia-fedora-x86-64.md`.

## Acceptance criteria

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

## Tests
- `just hardware test`; `just installer test`; `just arch packages`.
- `just fedora packages`, `just fedora iso`, `just fedora test`, `just fedora test-install`.
- `just lefthook all`.
- The Arch images rebuild and boot-test in the `images` workflow on merge (`hardware/` changed); the generator's Arch behaviour is covered by `just hardware test`.
- Real NVIDIA hardware cannot be emulated in QEMU. For the operator, once the ISO is built (task 030 collects the results):
  1. Write the Fedora x86_64 ISO to a USB stick and boot a PC with an RTX (Turing or newer) card. The desktop appears; in foot, `lsmod | grep -c '^nvidia_drm'` prints 1 and `lsmod | grep -c '^nouveau'` prints 0.
  2. Boot a PC with a GTX 10 series (Pascal) card. The desktop appears with short animations; `lsmod | grep -c '^nouveau'` prints 1.
  3. Install on the RTX PC and reboot: `lsmod | grep -c '^nvidia_drm'` prints 1. Then `sudo dnf upgrade` with a new kernel available and reboot: the desktop still appears (nouveau), and Limine's next entry (the previous kernel) starts with `nvidia_drm` loaded.
