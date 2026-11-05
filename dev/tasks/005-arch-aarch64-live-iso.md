# 005: Arch Linux ARM aarch64 console live ISO

- [x] Done

Base: arch

## Ask

```text title="Prompt" wrap
Build the aarch64 sibling of the Arch x86_64 live ISO: an Arch Linux ARM (ALARM) aarch64 console live ISO, named stackific-<version>-arch-aarch64.iso. Build it natively on the Mac's arm64 Linux runner VM (`just runner vm`), with the same archiso profile and the same `just arch` recipes, in a pinned builder container. ALARM has no archive, so the image comes from ALARM's live repos and its package manifest is the record. Console only: the desktop follows in a later task that mirrors the x86_64 one. Settle whether archiso can build it or another tool is needed, and whether Limine is in ALARM. Make tests/vm/boot.sh boot it under qemu-system-aarch64 with `-machine virt` and AAVMF from the builder, through the same token-gated vsock shell, and report the ISO size against the 2 GiB target. Add an arch-aarch64 job on [self-hosted, Linux, ARM64] to the images workflow and to the release workflow, each uploading its ISO like the x86_64 jobs. It ships in one PR whose first commit is the chore that readies that VM as a runner.
```

## Goal
On an aarch64 Linux host, `just arch iso` builds `.build/arch/aarch64/stackific-<version>-arch-aarch64.iso` from ALARM's live repos, and `just arch test` boots it under QEMU with KVM and AAVMF and proves a running console. On x86_64 both recipes behave as before.

## Context
Tasks 001 and 002 built the Arch x86_64 live ISO: profile `arch/iso/`, builder `arch/Containerfile`, recipes in `arch/justfile`, pins in `arch/vars.just` (archive date 2026-11-01), boot test `tests/vm/boot.sh`. Today every one of these is x86_64-only: `arch="x86_64"` and `-Xbcj x86` in `profiledef.sh`, `packages.x86_64`, `console=ttyS0` in the boot entry, `qemu-system-x86_64` with q35 and OVMF in `boot.sh`, and the x86_64 file names in `arch/justfile`.

Tasks 003 and 004 merged first:
- `tests/vm/boot.sh <iso> <out-dir> <checks>` runs the checks every image passes. It then sources the image's own check file, `tests/vm/<base>-<arch>.sh`, which uses `vm`, `vm_until` and `screenshot` from `boot.sh`. `tests/vm/arch-x86_64.sh` holds the identity checks from 004 and the desktop checks from 002, and `tests/vm/fedora-x86_64.sh` holds the Fedora console's.
- `OVMF_CODE` and `OVMF_VARS` name the firmware pair in the builder; the defaults are the Arch builder's x86_64 paths, and `just fedora test` passes Fedora's.
- The test shell is `tests/vm/stackific-test-shell.service` (`systemd-socket-activate` on vsock), started by a `multi-user.target` drop-in.
- `images` runs only on push to `main`. When a pull request merged the commit, its jobs upload each image with `just release upload <version> <base> <arch>` to the test release `v0.0.0-pr.<n>`, which `publish` publishes once every job has passed. `release` (by hand, input `version`) runs the same upload steps for a real release. Both list the image jobs in `publish`'s `needs`.
- `stackific-release` (004) is in `packages.x86_64`; the aarch64 image takes it with the desktop, in task 009.

The chore commit at the start of this PR readies the build host. `just runner setup` on the Mac creates the Lima VM `stackific-runner` (`just runner vm`), installs what the runner needs there (git, just, podman, libicu), clones the repo to `~/os` in the VM and runs `just runner setup` inside it (README.md, "Mac (aarch64 images)"). This task's image is built and boot-tested in that VM.

Facts, checked 2026-11-05. "Measured" means run in the runner VM (Lima 2.2.1, Apple M4 Pro, macOS 27.2). The ALARM package facts come from `pacman -Si` / `pacman -Fl` in the pinned base image below.

The runner VM (answers the earlier question whether the Mac's VM can build and boot-test aarch64 images):
- The VM runs Fedora 44 (Lima 2.2.1's `template:fedora`), kernel 6.19.10-300.fc44.aarch64, with SELinux enforcing. Measured.
- `/dev/kvm` and `/dev/vhost-vsock` exist there, mode 0666. `just arch check` prints `arch: ok (podman 5.8.1, /dev/kvm and /dev/vhost-vsock read-write)`. Measured.
- Lima's nested virtualization needs Apple M3 or later, `vmType: vz` and macOS 15 or later. Sources: lima-vm/lima `templates/default.yaml` lines 650-658 and `pkg/driver/vz/vm_darwin.go` lines 347-368 at v2.2.1.
- Rootless `podman run --privileged` works and sees `/dev/loop-control`. Measured.
- Fedora 44's cloud image lacks `libicu`, and the Actions runner's `config.sh` stops without it: `Libicu's dependencies is missing for Dotnet Core 6.0`. Measured on the operator's first `just runner setup`.
- In a rootless container with `--device /dev/kvm --device /dev/vhost-vsock`, `qemu-system-aarch64 -machine virt,accel=kvm -cpu host` boots AAVMF, with `vhost-vsock-pci` attached. Measured, on Fedora 44's QEMU and edk2.
- SELinux keeps containers off `/dev/vhost-vsock` (label `vhost_device_t`). QEMU fails with `Could not open '/dev/vhost-vsock': Permission denied` unless the container runs with `--security-opt label=disable`. `/dev/kvm` works without it. Measured.
- SELinux also hides bind-mounted checkout files from unprivileged containers: `just arch packages` fails with `cp: cannot stat '/packages/*/PKGBUILD'`. `--security-opt label=disable` fixes it without relabelling the checkout, as `:z` would. `--privileged` runs have labelling off already. Measured during /s-build.
- Guest CID 3 is taken in the VM (`unable to set guest cid: Address already in use`); `boot.sh` already picks a random CID. Measured.
- pacman's download sandbox fails in an unprivileged rootless container there: `restricting filesystem access failed because the Landlock ruleset could not be applied: Operation not permitted`. `pacman --disable-sandbox` works, and so does a `--privileged` container. `podman build` runs unprivileged, so the builder's pacman calls need `--disable-sandbox`. Measured.

archiso on aarch64 (answers "archiso or another tool, and is Limine in ALARM"):
- archiso v92 (2026-10-31) is the latest release and supports aarch64 natively. Sources at tag v92, https://gitlab.archlinux.org/archlinux/archiso:
  - `archiso/mkarchiso` maps `['aarch64']='AA64'` (lines 55-60).
  - With `uefi.systemd-boot`, it copies `systemd-bootaa64.efi` to `EFI/BOOT/BOOTAA64.EFI` (lines 852-857).
  - `CHANGELOG.rst`: v87 supports other architectures but says "Cross-architecture building is not supported"; v90 adds xz's arm64 BCJ filter.
- mkarchiso finds the kernel only as `boot/vmlinuz-*` (lines 497, 571, 934; checked). It sizes the ESP with `du -bcs` over those files (line 593; checked). GNU du counts a symlink given on its command line as the link itself, so `vmlinuz-linux` must be a real file, not a symlink.
- On aarch64, mkarchiso turns its stubble step on, then off with a warning when `stubble` is not in the package list (lines 1413-1425; checked). Nothing to do.
- mkarchiso writes `pkglist.<arch>.txt`, so `pkglist.aarch64.txt` here (line 1826).
- ALARM does not package `archiso` or `edk2-aarch64`.
- `pacman -U` installs Arch's `archiso-92-1-any.pkg.tar.zst` and `edk2-aarch64-202608-1-any.pkg.tar.zst` on ALARM. They come from `https://archive.archlinux.org/repos/2026/11/01/extra/os/x86_64/`, signature-checked once `pacman-key --populate archlinuxarm archlinux` has run. Their dependencies come from ALARM. Measured.
- edk2-aarch64 202608-1 ships `/usr/share/edk2/aarch64/QEMU_EFI.fd` and `QEMU_VARS.fd`, each 67108864 bytes, so both fit pflash directly. It also has `/usr/share/AAVMF/AAVMF_CODE.fd` and `AAVMF_VARS.fd` as symlinks to them. Measured.
- ALARM's `linux-aarch64` installs the kernel as `boot/Image` (and `boot/Image.gz`), not `vmlinuz-*` (`pacman -Fl`). Its preset writes `/boot/initramfs-linux.img`. Source: github.com/archlinuxarm/PKGBUILDs `core/linux-aarch64/PKGBUILD` line 97 and `linux.preset`.
- ALARM's `systemd` ships `usr/lib/systemd/boot/efi/systemd-bootaa64.efi` (`pacman -Fl`).
- ALARM's kernel config: `EFI_STUB=y`, `SQUASHFS=m`, `SQUASHFS_XZ=y`, `OVERLAY_FS=m`, `ISO9660_FS=m`, `VIRTIO_VSOCKETS=m`, `DRM_VIRTIO_GPU=m`. Not set: `EROFS_FS`, `SQUASHFS_ZSTD`, `XZ_DEC_X86`. So the airootfs stays squashfs with xz, and only the `arm64` BCJ filter applies. releng's `-Xbcj x86,arm64` keeps whichever filter compresses best per block, which can pick x86. Source: PKGBUILDs `core/linux-aarch64/config` (last commit 2026-11-01). `-Xbcj arm64` needs squashfs-tools 4.7 or later (`squashfs-tools/xz_wrapper.c`).
- Versions in ALARM aarch64 (live repos): `linux-aarch64` 7.3.1-2 (core), `linux-firmware` 20261019-1 (core), `systemd` 262.1-1 (core), `btrfs-progs` 7.3-1 (core), `dosfstools` 4.2-5 (core), `archlinuxarm-keyring` 20240419-2 (core), `pacman` 7.1.0.r9.g54d9411-3 (core). Also `limine` 12.9.6-1 (extra, the same version as Arch's), `mkinitcpio-archiso` 73-1, `networkmanager` 1.58.1-3, `squashfs-tools` 4.7.5-1, `libisoburn` 1.5.8.2-1, `mtools` 1:4.0.50-1, `arch-install-scripts` 31-2, `qemu-system-aarch64` 11.1.3-1 and `socat` 1.8.1.3-1, all in extra. These drift daily.

ALARM repos and the builder base:
- ALARM has no official container image; `archlinux/archlinux` is amd64 only (Docker Hub tags API).
- `docker.io/menci/archlinuxarm` is built daily from ALARM with pacstrap, last pushed 2026-11-05. Tag `base-20261105.41979707783` has arm64 digest `sha256:91ed080fdfec1da7f3d2b5587d7a16a3ce7f3b3d3ee67b6affe51e399fcb619a`; it pulls and runs in the VM (measured). Its README says the pacman lsign key is removed, so run `pacman-key --init` and `pacman-key --populate archlinuxarm` (github.com/Menci/docker-archlinuxarm). Its mirrorlist lists ALARM's regional HTTP mirrors (measured).
- ALARM mirrors serve `http://mirror.archlinuxarm.org/$arch/$repo` over HTTP. HTTPS fails on the main host (its certificate covers only `archlinuxarm.org`). Repos: core, extra, alarm, aur. Repo databases are unsigned and packages are signed, so `SigLevel = Required DatabaseOptional`, as `arch/iso/pacman.conf` already has. Build-system key `68B3537F39A313B3E574D06777193F152BDBE6A6`. Source: https://archlinuxarm.org/about/package-signing.

QEMU on `virt`:
- `virtio-gpu-pci` is the only display device that works correctly with KVM on `virt`. Source: QEMU `docs/system/arm/virt.rst` lines 99-104.
- ALARM's `/etc/makepkg.conf` sets `PKGEXT='.pkg.tar.xz'`, so `just arch packages`, which copied only `*.pkg.tar.zst`, failed with `cp: cannot stat '/home/build/pkg/stackific-desktop/*.pkg.tar.zst'`. Measured during /s-build.
- ALARM's `qemu-system-aarch64` loads devices as modules: without `qemu-hw-display-virtio-gpu` and `qemu-hw-display-virtio-gpu-pci` (both 11.1.3-1, extra), QEMU fails with `'virtio-gpu-pci' is not a valid device model name`. Measured during /s-build.
- Firmware draws nothing Linux can keep, so the console appears once the `virtio_gpu` module loads, after switch_root. Source: edk2 `OvmfPkg/VirtioGpuDxe/Gop.c` line 262.
- HMP `screendump <file> -f png` takes a screenshot (`hmp-commands.hx` line 271).
- QEMU accepts `-cdrom` on `virt`, and also a `scsi-cd` on `virtio-scsi-pci`; both start (measured, Fedora 44's qemu-system-aarch64). Which one AAVMF boots from is for the build to confirm.
- The guest's serial console is the PL011 `ttyAMA0`, not `ttyS0`.

AGENTS.md rules this task applies:
- Arch Linux ARM builds from its live repos, and every image records its full package manifest.
- Builds and tests run in a pinned container; outputs go to `.build/`.
- Every image task reports its ISO size.
- VM test access comes only from one-boot SMBIOS credentials with a token-gated vsock shell; no image ships sshd or test units.
- Self-hosted runners take only `push` to `main` (`images`) and `workflow_dispatch` (`release`).
- No NOPASSWD.

## Inputs
Decisions the operator made for this task (2026-11-05), restated:
- The number 005 is reserved for this task, because tasks 003 (Fedora 45 x86_64 console ISO) and 004 (`stackific-release` on Arch) were in progress on the operator's other machine; both have since merged.
- Scope: a console live ISO, no desktop. A later task brings MangoWM, Noctalia and Plasma to aarch64 by mirroring 002 once this one has merged.
- It answers whether archiso builds the aarch64 ISO (or another tool must) and whether Limine is in ALARM (both answered in Context: archiso v92 from the Arch archive; `limine` 12.9.6-1 in ALARM extra).
- It builds from live repos, so the manifest is the record. It makes `tests/vm/boot.sh` architecture-aware (`qemu-system-aarch64`, `-machine virt`, AAVMF from the builder). It adds an `arch-aarch64` job to `images.yml` on `[self-hosted, Linux, ARM64]`.
- Its image is built and boot-tested on the Mac's runner VM, which mounts nothing from the Mac.
- The PR's first commit is the runner-VM chore; this task's commits follow in the same PR.

Design choices made while writing this task (the operator may change them in review):
- The host's architecture picks the image (`arch()` in just): no cross-building, which archiso does not support. The same `just arch iso` and `just arch test` serve both architectures, and both images jobs run the same steps.
- One profile, `arch/iso/`, for both architectures, so the aarch64 desktop task only grows `packages.aarch64`. Until then, the aarch64 image carries the x86_64 desktop's airootfs files unused: the greetd config, the `live` and `greeter` accounts, and a `display-manager.service` alias whose target is absent. None of them starts anything.
- 003 landed first with per-image check files, so the aarch64 console checks go to `tests/vm/arch-aarch64.sh`. `boot.sh` itself only learns the aarch64 machine and firmware.

No files copied.

## Changes
- `arch/vars.just`:
  - `builder_base` by host architecture: the existing `archlinux/archlinux` digest on x86_64, and `docker.io/menci/archlinuxarm:base-20261105.41979707783@sha256:91ed080fdfec1da7f3d2b5587d7a16a3ce7f3b3d3ee67b6affe51e399fcb619a` on aarch64;
  - `out` becomes `.build/arch/<arch()>`;
  - comments that `ala_date` pins every package on x86_64, but on aarch64 only archiso and edk2-aarch64 (ALARM's live repos give the rest).
- `arch/Containerfile`, one branch per `$(uname -m)`; the `build` user and sub-ID lines stay shared:
  - x86_64 unchanged;
  - aarch64 keeps the base image's ALARM mirrorlist and runs `pacman-key --init` and `pacman-key --populate archlinuxarm archlinux`;
  - then `pacman -Syu --noconfirm --disable-sandbox fakeroot qemu-system-aarch64 qemu-hw-display-virtio-gpu qemu-hw-display-virtio-gpu-pci socat`;
  - then `pacman -U --noconfirm --disable-sandbox` on `archiso-92-1-any.pkg.tar.zst` and `edk2-aarch64-202608-1-any.pkg.tar.zst` from the archive date's `extra/os/x86_64/`.
- `arch/justfile`:
  - `iso` names the image `stackific-<version>-arch-<arch()>` and copies `pkglist.<arch()>.txt` as the manifest;
  - `packages` takes any `*.pkg.tar.*` (ALARM writes `.pkg.tar.xz`);
  - `packages` and `test` add `--security-opt label=disable` (Fedora's SELinux, see Context); `test` boots `stackific-<version>-arch-<arch()>.iso` with the check file `tests/vm/arch-<arch()>.sh`;
  - the module comments name both architectures.
- `arch/iso/profiledef.sh`: `arch="$(uname -m)"`; the BCJ filter is `x86` on x86_64 and `arm64` on aarch64; comments name both architectures.
- `arch/iso/packages.aarch64` (new): `base linux-aarch64 linux-firmware archlinuxarm-keyring mkinitcpio mkinitcpio-archiso networkmanager btrfs-progs dosfstools`.
- `arch/iso/airootfs/etc/pacman.d/hooks/stackific-vmlinuz.hook` (new): after an install or upgrade that touches `boot/Image`, copy `/boot/Image` to `/boot/vmlinuz-linux`. A copy, because mkarchiso sizes the ESP with `du` (see Context). It never triggers on x86_64.
- `arch/iso/efiboot/loader/entries/01-stackific-linux.conf`: the options gain `console=ttyAMA0,115200` before `console=tty0`, so one entry serves both architectures; the kernel ignores the serial console it does not have.
- `tests/vm/boot.sh`: the QEMU command picks its binary, machine, display and default firmware from `uname -m`. x86_64 is unchanged. On aarch64 it uses:
  - `qemu-system-aarch64 -machine virt,accel=kvm -cpu host`;
  - `QEMU_EFI.fd` as read-only pflash, with a copy of `QEMU_VARS.fd`;
  - `virtio-gpu-pci`;
  - the ISO as `-cdrom`.

  `OVMF_CODE` and `OVMF_VARS` still override the firmware.
- `tests/vm/arch-aarch64.sh` (new): the console checks.
  - `uname -m` prints `aarch64`, and `getty@tty1.service` is active.
  - It waits until tty1's text (`/dev/vcs1`) shows `stackific login:`, because getty draws its prompt a moment after it starts. Then it saves `screen.png` of tty1.
- `.github/workflows/images.yml`: an `arch-aarch64` job on `[self-hosted, Linux, ARM64]`, with the same steps as `arch-x86_64` (`just arch iso`, `just arch test`, and the test release upload `just release upload "$VERSION" arch aarch64`); `publish` also needs it. Its comment names the Mac's VM.
- `.github/workflows/release.yml`: the same `arch-aarch64` job, uploading to the release; `publish` also needs it; the header comment no longer defers the Mac runner.
- `docs/releases.md`: the release holds the Arch aarch64 image from the Mac's runner VM too.
- `docs/arch-iso.md`: the aarch64 build, covering:
  - where it runs (the Mac's runner VM);
  - what comes from ALARM's live repos and what from the archive date;
  - the builder base and how to move it (a newer `menci/archlinuxarm:base-<date>` tag with its arm64 digest, then `just arch iso` and `just arch test`);
  - the `vmlinuz-linux` hook;
  - the SELinux and pacman-sandbox notes;
  - the output file names per architecture.
- `README.md`: under Everyday use, `just arch iso` and `just arch test` build and boot the image for the host's architecture (x86_64 on the PC, aarch64 in the Mac's VM).
- `AGENTS.md`, one line each:
  - under Builds: Arch Linux ARM aarch64 builds natively in the Mac's runner VM from the same `arch/Containerfile` and profile. The base is `docker.io/menci/archlinuxarm` by digest (no official image), the packages come from ALARM's live repos, and archiso and edk2-aarch64 come from the archive date;
  - under CI and releases: the release line's "today" names the Arch aarch64 job on the Mac;
  - under Gotchas: Fedora's SELinux keeps unprivileged containers off bind mounts and `/dev/vhost-vsock`, so `just arch packages` and `just arch test` run with `--security-opt label=disable`; pacman's Landlock sandbox fails in unprivileged rootless podman there, so `podman build` steps use `--disable-sandbox`.

## Acceptance criteria

Must:
- In the runner VM, `just arch iso` exits 0. It writes `.build/arch/aarch64/stackific-0.0.0-dev-arch-aarch64.iso` and `.build/arch/aarch64/stackific-0.0.0-dev-arch-aarch64.packages.txt`, and its last line prints the ISO size in MiB against 2048 MiB.
- The manifest lists `linux-aarch64` and `archlinuxarm-keyring`, which proves the image came from ALARM. It lists neither `amd-ucode` nor `intel-ucode`.
- The ISO's ESP holds `EFI/BOOT/BOOTAA64.EFI` (`mdir` on the efiboot image, or `xorriso -indev <iso> -find /EFI`).
- In the runner VM, `just arch test` exits 0. Over the vsock shell its checks print:
  - `running` for `systemctl is-system-running`;
  - `running` for `nmcli -t -f RUNNING general`;
  - `aarch64` for `uname -m`;
  - `active` for `systemctl is-active getty@tty1`.
- A wrong token gets no shell: `echo bad; echo id` prints nothing and the connection closes.
- `just arch test` leaves `.build/arch/aarch64/screen.png`, showing the tty1 login prompt, and `.build/arch/aarch64/serial.log`.
- A second `just arch iso` in a row rebuilds cleanly.
- On the operator's PC, `just arch iso` and `just arch test` pass on this branch, which proves the x86_64 image and its checks unchanged. `images` runs only on `main`, so it cannot prove this before the merge.
- After the merge, the `images` run on `main` passes `arch-aarch64` on the Mac's VM. The test release `v0.0.0-pr.<n>` then holds `stackific-0.0.0-pr.<n>-arch-aarch64.iso`, its `.packages.txt` and `.iso.sha256`.
- `just lefthook all` passes: shellcheck on `tests/vm/boot.sh`, `tests/vm/arch-aarch64.sh` and `arch/iso/profiledef.sh`, actionlint on both workflows, justfiles parse, `just dev check`.
- The task report states the aarch64 ISO size in MiB against the 2048 MiB target.

Must not:
- cross-build, emulate one architecture on another, or use qemu-user or binfmt;
- ship `openssh`, `cloud-init`, VM guest agents, GRUB or syslinux in the image (none in the manifest), or any file from `tests/`;
- use sudo or root on the host for the build or the test;
- change the x86_64 image's package list or its boot checks;
- pin ALARM packages beyond the builder base image digest;
- add `pull_request` or `workflow_dispatch` to `images.yml`, `NOPASSWD`, or a polkit rule;
- touch files outside the Changes list (the review record and prompt-trail entry from /s-build excepted), or add a dependency this task does not name.

## Tests
- In the runner VM: `just arch iso`, then `ls -l .build/arch/aarch64/` and `grep -E '^(linux-aarch64|archlinuxarm-keyring|amd-ucode|intel-ucode|openssh|cloud-init|grub|syslinux) ' .build/arch/aarch64/*.packages.txt`. This proves the build, the names, the ALARM source and the absent packages.
- `just arch test` in the VM: the four shell checks, the wrong-token check, the screenshot and the serial log.
- `just arch iso` twice in a row: proves the fresh work dir.
- `just arch check` in the VM, and `just lefthook all` on a developer machine.
- Manual, by the operator, before the merge, on the PC: `just arch iso`, then `just arch test`. Both must pass.
- Manual, by the operator, after the merge, with both runners online (`just runner status` lists both):
  1. Watch the `images` run on `main`: `gh run watch`, and pick it.
  2. Every job must finish green.
  3. `gh release view v0.0.0-pr.<n>` must list the aarch64 ISO, its manifest and its `.iso.sha256`.
