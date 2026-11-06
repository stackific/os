# 010: Fedora 45 aarch64 live ISO boots to MangoWM with Noctalia, with GNOME

- [ ] Done

Base: fedora

## Ask

```text title="Prompt" wrap
Build the Fedora 45 desktop live ISO from task 008 for aarch64 too, so it is the same product as the x86_64 image, as AGENTS.md asks. Skip a separate console step: tasks 003 and 008 settled the layout, so this task only adds the architecture. The image boots straight into MangoWM with Noctalia as the user live. noctalia-greeter shows our logo, palette and wallpaper, and GNOME is the second session. Terra supplies only mangowm and noctalia-greeter, and its key is checked. SELinux is enforcing, and no Fedora trademark packages are in the image. Build it natively in the Mac's runner VM from the same kiwi description, builder and recipes, with the host's architecture picking the image, as on Arch. The package list differs only where aarch64 forces it. Both Fedora images then pass the same checks, so the boot test runs one Fedora check file on both, with a screenshot of the desktop and of the greeter. Add the image to the images and release workflows on the Mac's runner. Report the ISO size against the 2 GiB target.
```

## Goal
In the Mac's runner VM, `just fedora iso` builds a Fedora 45 aarch64 live ISO that looks and behaves like the x86_64 one. It boots into MangoWM with Noctalia as `live`, with our noctalia-greeter and GNOME available from it. `just fedora test` runs the same Fedora checks there as on x86_64, and the `images` and `release` workflows build, test and publish it.

## Context
This task starts from `main` after task 009 (`0f7d4cb`).

What `main` has:
- Task 003 built a Fedora 45 x86_64 console live ISO with kiwi, from the description in `fedora/iso/` (`config.kiwi`, `config.sh`, `root/`). The builder is `fedora/Containerfile`, pinned in `fedora/vars.just`.
- Task 007 added `just fedora packages`. It builds our three noarch RPMs with mock (`-r fedora-45-{{ arch() }}`, so it already follows the host) into `.build/fedora/repo/`.
- Task 008 made the image a desktop:
  - Terra, limited by `fedora/iso/terra-repo.sh` to `mangowm` and `noctalia-greeter`, with packages and metadata checked against `fedora/iso/terra-45.asc`;
  - greetd with noctalia-greeter (`fedora/iso/root/`), gdm disabled in `config.sh`;
  - GNOME as the second session;
  - the checks in `tests/vm/fedora-x86_64.sh`.

  The ISO is 1488 MiB with 677 packages (`dev/reviews/008-fedora-desktop-live-iso.md`).
- Every Fedora build input is x86_64-only:
  - `fedora/vars.just`: `out` is `.build/fedora/x86_64`;
  - `fedora/justfile`: `iso` names the ISO `stackific-$1-fedora-x86_64`, and `test` passes OVMF's paths and `/tests/fedora-x86_64.sh`;
  - `fedora/Containerfile`: installs `edk2-ovmf` and `qemu-system-x86-core`;
  - `fedora/iso/config.kiwi`: names `grub2-efi-x64`, `grub2-efi-x64-cdboot`, `grub2-efi-x64-modules`, `amd-ucode-firmware` and `microcode_ctl`, and its `kernelcmdline` has only `console=ttyS0,115200 console=tty0`.
- Tasks 005 and 009 did the same step on Arch:
  - the host's architecture picks the image (`arch/vars.just` `out := … / arch()`);
  - `arch/Containerfile` installs QEMU and UEFI firmware per architecture with a `case "$(uname -m)"`;
  - the Arch boot entry names both serial consoles, `ttyS0` (x86_64) and `ttyAMA0` (aarch64), and the kernel ignores the one the machine lacks (`docs/arch-iso.md`);
  - both Arch images source one check file, `tests/vm/arch.sh`.
- `tests/vm/boot.sh` already picks the machine per architecture. On aarch64 it uses a `virt` machine with KVM and `virtio-gpu-pci`, and it takes firmware paths from `OVMF_CODE` and `OVMF_VARS`. Its defaults are the Arch builder's paths. Its header comment still names the check file `tests/vm/<base>-<arch>.sh`, which 009 made out of date.
- `.github/workflows/images.yml` and `release.yml` have jobs `arch-x86_64` and `fedora-x86_64` on `[self-hosted, Linux, X64]` and `arch-aarch64` on `[self-hosted, Linux, ARM64]`, each with `just <base> iso`, `just <base> test` and `just release upload <version> <base> <arch>`. `publish` needs all three. `release/justfile` `upload` already takes the base and the architecture.
- AGENTS.md rules this task applies:
  - the four images are one product: same branding, defaults, files, live session and boot-test checks, differing only where a base or architecture forces it;
  - the boot test sources `tests/vm/<base>.sh` when every image of the base passes the same checks;
  - Terra is limited with `includepkgs`, and its key is checked;
  - no `fedora-release*`, `fedora-logos` or `fedora-release-notes`;
  - every image records its manifest and reports its size;
  - never `NOPASSWD`, and no polkit rule that skips authentication.
- AGENTS.md gotcha: in the Mac's Fedora runner VM, SELinux keeps unprivileged containers off bind mounts and `/dev/vhost-vsock`, so `just arch test` runs with `--security-opt label=disable`. `just fedora test` does not have that option yet. `just fedora iso` and `just fedora packages` run `--privileged`, which turns labelling off already.

Facts, checked 2026-11-05 in the Mac's runner VM `stackific-runner` (aarch64, 6 CPUs, 12 GiB memory, from `just runner vm`), unless another source is named. They come from a throwaway container of the pinned builder base `registry.fedoraproject.org/fedora:45@sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd` (`dnf repoquery` against Fedora 45's live repos and Terra 45). "Dry run" means `dnf install --assumeno --installroot=… --releasever=45 --setopt=install_weak_deps=False`, as in task 008.

The builder base:
- The pinned digest is a manifest list. On the VM, `podman pull` takes its arm64 image (`podman image inspect`: `arm64 linux`), so `builder_base` needs no aarch64 pin of its own.

Terra 45 on aarch64:
- `mangowm` 0.18.5-1.fc45 and `noctalia-greeter` 1.7.0-2.fc45, the same EVRs as on x86_64. Both have vendor `Terra` and are signed with key ID `8dde7d14c1c23d8c` (`rpm -qp --qf '%{vendor} %{rsaheader:pgpsig}'` on the downloaded aarch64 RPMs).
- dnf imports the same Terra 45 key for the aarch64 repo, fingerprint `C2AC02124AF114F086592E3C8DDE7D14C1C23D8C`, so `fedora/iso/terra-45.asc` serves both architectures.
- The aarch64 `mangowm` ships `/etc/mango/config.conf`, `/etc/mango/config.toml`, `/usr/lib/systemd/user/mango-session.target` and `/usr/share/wayland-sessions/mango.desktop`, the files 008 relies on.
- Terra also has `limine` 12.9.6-1.fc45 and `scenefx` 0.5-1.fc45 for aarch64. `includepkgs` keeps both out, as on x86_64.

Fedora 45 on aarch64:
- The desktop packages have the same EVRs as task 008's x86_64 list:
  - `noctalia` 5.4.1-1, `greetd` and `greetd-selinux` 0.10.3-12, `scenefx` 0.5-1, `wlroots` 0.20.2-2;
  - `gnome-shell` 51.1-1, `gnome-session-wayland-session` 51.0-1, `gdm` 1:51.1.1-1, `nautilus` 51.1-2, `ptyxis` 50.1-6;
  - `foot` 1.28.0-1, `rofi` 2.0.0-3;
  - `xdg-desktop-portal-gnome` 51.0-2, `xdg-desktop-portal-gtk` 1.15.3-6, `xdg-desktop-portal-wlr` 0.8.4-1;
  - `pipewire` and `pipewire-pulseaudio` 1.6.11-1, `wireplumber` 0.5.19-1;
  - `mesa-dri-drivers` 26.2.6-1, `generic-logos` 18.0.0-29, `NetworkManager` and `NetworkManager-wifi` 1:1.58.1-3, `fastfetch` 2.68.1-1.

  All are `.fc45`. Also `kernel` 7.2.15-300.fc45, `systemd` 262.1-1.fc45, `dracut-live` 111-5.fc45 and `linux-firmware` 20261019-1.fc45.
- Every firmware package of `config.kiwi` exists on aarch64. They are noarch, at 20261019-1.fc45, except `alsa-sof-firmware` at 2026.09.2-1.fc45.
- Not on aarch64: `grub2-efi-x64`, `grub2-efi-x64-cdboot` and `microcode_ctl`.
- On aarch64 but useful only on x86_64: `grub2-efi-x64-modules`, which holds GRUB's x86_64-efi modules, and `amd-ucode-firmware`, which holds AMD's x86 CPU microcode. Both are noarch. Arch's aarch64 list leaves out the CPU microcode too (task 009).
- The aarch64 counterparts are `grub2-efi-aa64`, `grub2-efi-aa64-cdboot` (it ships `gcdaa64.efi`) and `grub2-efi-aa64-modules` (it ships `/usr/lib/grub/arm64-efi/`), all at 1:2.12-83.fc45.
- `qcom-firmware` (Qualcomm SoC firmware) exists on aarch64. The x86_64 list does not have it, so it waits for the aarch64 hardware task.
- No package is named `passwd`. `shadow-utils` 2:4.20.0-2.fc45 provides it (`dnf repoquery --whatprovides passwd`), so `config.kiwi`'s `passwd` line resolves.
- Dry run of `config.kiwi`'s image set for aarch64: every package of the x86_64 set except the five x86_64-only packages above, plus the three `grub2-efi-aa64*` packages. `generic-release` stood in for `stackific-release` and `fastfetch` for `stackific-artwork`, because our RPMs are in no online repo.
  - It resolves: 661 packages, 981 MiB download, 2 GiB installed. Task 008's x86_64 dry run had 655 packages, about 1 GiB download and about 2 GiB installed.
  - Terra supplied only `mangowm` and `noctalia-greeter`; `scenefx` and `switcheroo-control` came from Fedora.
  - It has no `fedora-release*`, `fedora-logos`, `fedora-release-notes`, `cardwire` or `openssh-server`. `openssh` and `openssh-clients` are in it, as on x86_64 (`noctalia` requires `git-core`).
  - `kernel-core` is 292.9 MiB installed.
  - The dry run turned off Terra's metadata check, because the throwaway installroot had no Terra key. It installed nothing. kiwi imports the key (task 008), so the build keeps both checks.
- `mesa-dri-drivers` ships `kms_swrast_dri.so`, `swrast_dri.so` and `virtio_gpu_dri.so` (`dnf repoquery -l`). So on the test VM's `virtio-gpu-pci`, Mesa renders in software, which `WLR_RENDERER_ALLOW_SOFTWARE=1` from `stackific-desktop` lets MangoWM use. On Arch Linux ARM, MangoWM, Noctalia and noctalia-greeter draw this way in the same VM (task 009).
- `mock-core-configs` ships `/etc/mock/fedora-45-aarch64.cfg`.

kiwi 11.1.1-2.fc45 (installed in the throwaway container):
- `<package>` takes an optional `arch` attribute (`schema/kiwi.rnc` lines 976–985). `xml_state.py` `_section_matches_host_architecture` keeps an element only when the host architecture is in its comma-separated `arch` list. One description can therefore name architecture-specific packages without a profile.
- aarch64 support in `defaults.py`:
  - line 1523 allows the firmware `efi` or `uefi` on aarch64;
  - line 1622 names the GRUB module directory `arm64-efi`;
  - line 1663 names the EFI boot file `bootaa64.efi`;
  - line 1240 lists `gcdaa64.efi` for the ISO's EFI image.
- `kiwi-systemdeps-iso-media` on aarch64 requires `grub2-efi-aa64-cdboot`, `xorriso` and `isomd5sum`.

The builder on aarch64:
- `qemu-system-aarch64-core` 2:11.1.3-1.fc45 requires `edk2-aarch64` 20260812-10.fc45.
- `qemu-system-aarch64-core` does not pull in `qemu-device-display-virtio-gpu-pci` 2:11.1.3-1.fc45: with weak dependencies off, it was not installed. So the builder names that package, as Arch's names its virtio-gpu modules.
- `edk2-aarch64` files:
  - `/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw` and `vars-template-pflash.raw` are 67108864 bytes (64 MiB);
  - `QEMU_EFI.fd` is 3145728 bytes, and `QEMU_VARS.fd` is 786432 bytes.

  QEMU's `virt` machine takes only 64 MiB flash images, so the boot test uses the `-pflash.raw` pair. `boot.sh`'s aarch64 defaults are Arch's paths, so `just fedora test` passes these two in `OVMF_CODE` and `OVMF_VARS`.

kiwi's work tmpfs on an SELinux host (measured during this task's build in the runner VM, where `getenforce` prints `Enforcing`):
- The first `just fedora iso` there stopped when kiwi copied `fedora/iso/root/` into the image: `rsync_xal_set: fsetxattr("/work/build/image-root/.","security.selinux") failed: Operation not supported (95)`.
- podman mounts `--tmpfs /work` with `context="system_u:object_r:container_file_t:s0:c1022,c1023"` (`/proc/mounts` in the container), even with `--privileged` and `--security-opt label=disable`. A `context=` mount gives the whole filesystem one label, so `chcon` on any file there fails with `Operation not supported`.
- A tmpfs that container root mounts itself (`mount -t tmpfs tmpfs /work` in the `--privileged` container) has `seclabel` and no `context=`. There `chcon` sets `system_u:object_r:etc_t:s0` and `system_u:object_r:xdm_exec_t:s0`. The kernel lets SELinux labels be set on a tmpfs mounted in a user namespace.
- The operator's PC builds with podman's `--tmpfs` (task 008), so SELinux is not enforcing there. A tmpfs mounted by container root works there too, so the recipe uses it on both hosts.
- kiwi's temporary files default to `/var/tmp` (`kiwi-ng --help`: `--temp-dir`, default `/var/tmp`). In the container that is podman's overlay, which also has one fixed label. With the work dir on the self-mounted tmpfs, the build stopped in kiwi's EFI step: `rsync_xal_set: fsetxattr("/var/tmp/efi_folder…/.","security.selinux") failed: Operation not supported (95)`. So kiwi gets `--temp-dir` on the same tmpfs.
- A rootless container may set only SELinux contexts that the host's loaded policy knows. With the VM on Fedora 44 (`selinux-policy-targeted` 43.3), kiwi's `setfiles` stopped with `Could not set context for /usr/share/accountsservice: Invalid argument`, a context of Fedora 45's policy. The operator chose to move the VM to Fedora 45 (2026-11-05). After an in-place `dnf system-upgrade` it runs `selinux-policy-targeted` 45.23, the image's version, and `setfiles` passes. That move is chore work: `just runner vm` pins Fedora 45's Cloud image, and `just setup` on a Mac makes the VM. It is not part of this task's change.

Build resources:
- kiwi's work tmpfs peaks at 5398 MiB for the x86_64 desktop ISO (`docs/fedora-iso.md`). A tmpfs without `size=` is capped at half the host's RAM: 5959 MiB in the 12 GiB VM.
- In the 12 GiB VM the aarch64 build ran out of room on it when xorriso wrote the ISO: `Image size 733160s exceeds free space on media 323398s` (2048-byte sectors: 1432 MiB needed, 632 MiB free). So the build needs about 6.6 GiB there (5327 MiB used plus 1432 MiB); the 16 GiB VM then measured 6763 MiB of a 7971 MiB cap (`df` every 5 s in the build container). The Mac has 48 GiB of RAM. The operator chose to give the VM 16 GiB (2026-11-05), which makes the cap about 8 GiB. `limactl edit --memory 16` resized this VM, and `just runner vm` creates new ones with 16 GiB (chore work, like the Fedora 45 move).
- The Mac has one runner, so the `arch-aarch64` and `fedora-aarch64` jobs of one workflow run one after the other.

Left out, for later tasks:
- `qcom-firmware`, NVIDIA and other aarch64 hardware (task 022 in the operator's plan);
- the installer on aarch64;
- automated GNOME login (manual, as in 008).

## Inputs
Decisions from the operator's plan of work (2026-11-05), restated:
- Number 010 is reserved for this task. It runs on the Mac lane, and it starts after task 008 (Fedora x86_64 desktop) has merged, which it has (`531db03`).
- It skips a separate Fedora aarch64 console step: 003 and 008 settled the layout, so this task only adds the architecture.
- The `release` task for v0.1.0 waits for this one, because it builds all four images.
- Mac-lane tasks are built inside the Mac's Lima VM (`limactl shell stackific-runner`, then `cd ~/os`). The VM mounts nothing from the Mac, so builds, tests and the repo clone all live in it.
- Both lanes edit `tests/vm/boot.sh`, the workflows, `AGENTS.md` and `docs/`. Keep each change there small, because whoever merges second rebases.

Design choices made while writing this task, which the operator may change in review:
- **One kiwi description.** `config.kiwi` marks its architecture-specific packages with kiwi's `arch` attribute, and every other package line serves both. So the two package sets cannot drift apart, and no `diff` check is needed, unlike Arch's two lists (`dev/reviews/009-arch-aarch64-desktop-live-iso.md`, finding 1).
- **Left out on aarch64:** only what aarch64 lacks or cannot use: the x86_64 GRUB packages and the x86 CPU microcode (`amd-ucode-firmware`, `microcode_ctl`). The Intel GPU, audio, NPU and VSC firmware stay on both, as on Arch Linux ARM, whose `linux-firmware` carries them too. They are noarch, and leaving them out would be a new difference the base does not force.
- **One Fedora check file.** `tests/vm/fedora-x86_64.sh` becomes `tests/vm/fedora.sh`, with its checks unchanged, as 009 did for Arch. Nothing in it depends on the architecture.
- **Both serial consoles** on the kernel command line, as on Arch: `console=ttyS0,115200 console=ttyAMA0,115200 console=tty0`. So the x86_64 image's command line gains `ttyAMA0`, which its kernel ignores.

No files copied.

## Changes
- `fedora/iso/config.kiwi`:
  - the top comment and `<specification>` say x86_64 and aarch64;
  - `kernelcmdline="console=ttyS0,115200 console=ttyAMA0,115200 console=tty0"`, with the type's comment saying the kernel ignores the serial console the machine lacks;
  - `arch="x86_64"` on `grub2-efi-x64`, `grub2-efi-x64-cdboot`, `grub2-efi-x64-modules`, `amd-ucode-firmware` and `microcode_ctl`;
  - new `grub2-efi-aa64`, `grub2-efi-aa64-cdboot` and `grub2-efi-aa64-modules` with `arch="aarch64"`, after the x86_64 GRUB lines;
  - the GRUB comment names both module directories (`x86_64-efi`, `arm64-efi`), and a comment above the microcode lines says why they are x86_64 only;
  - no other package line changes.
- `fedora/Containerfile`: install QEMU and UEFI firmware for the build host. Use `case "$(uname -m)"` as `arch/Containerfile` does: `edk2-ovmf qemu-system-x86-core` on x86_64, `qemu-system-aarch64-core edk2-aarch64 qemu-device-display-virtio-gpu-pci` on aarch64, and fail on anything else. The rest of the install stays shared. Its comment says why the display package is named.
- `fedora/vars.just`:
  - `out := justfile_directory() / ".build" / "fedora" / arch()`, with its comment saying the host's architecture picks it;
  - new `uefi_code` and `uefi_vars`, the raw firmware pair for the boot test: OVMF's `/usr/share/edk2/ovmf/OVMF_CODE.fd` and `OVMF_VARS.fd` on x86_64, and AAVMF's `/usr/share/edk2/aarch64/QEMU_EFI-pflash.raw` and `vars-template-pflash.raw` on aarch64. A comment gives the 64 MiB reason.
- `fedora/justfile`:
  - the header says the host's architecture picks the image (x86_64 on the PC, aarch64 in the Mac's runner VM) and names UEFI firmware instead of OVMF;
  - `iso` names the ISO `stackific-$1-fedora-{{ arch() }}`, and its comment says `<arch>`;
  - `iso` drops podman's `--tmpfs /work:rw,exec,dev,suid`, and the container mounts the work tmpfs itself (`mkdir -p /work && mount -t tmpfs tmpfs /work`) before `kiwi-ng`, which gets `--temp-dir /work/tmp` on it. The comment says why: the SELinux facts above. mount(8)'s defaults include `exec`, `dev` and `suid`;
  - `test` passes `OVMF_CODE={{ uefi_code }}` and `OVMF_VARS={{ uefi_vars }}`, adds `--security-opt label=disable` with the comment `arch/justfile` has, boots `stackific-$1-fedora-{{ arch() }}.iso`, and passes `/tests/fedora.sh`. Its comment names the file.
- `tests/vm/fedora.sh`: `tests/vm/fedora-x86_64.sh` renamed, with its checks unchanged. Only the header comment changes: both Fedora architectures source it.
- `tests/vm/boot.sh`: the header comment names the check file `tests/vm/<base>.sh`, or `tests/vm/<base>-<arch>.sh` for checks that only one architecture passes. No code change.
- `.github/workflows/images.yml`: a `fedora-aarch64` job like `arch-aarch64` (`runs-on: [self-hosted, Linux, ARM64]`, `timeout-minutes: 90`) that runs `just fedora iso`, `just fedora test` and `just release upload "$VERSION" fedora aarch64`. `publish` needs it too.
- `.github/workflows/release.yml`: the same job, `publish` needing it, and the header comment saying all four images build, the aarch64 ones on the Mac's runner VM.
- `docs/fedora-iso.md`:
  - the intro and the outputs say the host's architecture picks the image and outputs go to `.build/fedora/<arch>/`;
  - the build section says the container mounts its own work tmpfs, and why (podman's `--tmpfs` takes no labels on an SELinux host);
  - a new aarch64 section covers:
    - the `arch` attribute and the eight packages that carry it;
    - the second serial console;
    - the builder's AAVMF pflash pair and the virtio-gpu display package;
    - `label=disable` in the runner VM;
    - the work tmpfs peak on aarch64, and why the VM has 16 GiB (in the build section's tmpfs paragraph);
    - the jobs on the Mac's runner;
  - the boot test section names `tests/vm/fedora.sh`;
  - moving the builder digest takes the manifest list's digest, and runs the test gate on both hosts.
- `docs/releases.md`: the "Today" sentence says the release holds all four images (the x86_64 ones from the PC, the aarch64 ones from the Mac's runner VM), unsigned, and that signing joins later.
- `AGENTS.md`:
  - Builds: one line saying the host's architecture picks the Fedora image too, and that aarch64 builds natively in the Mac's runner VM from the same description, with `arch="…"` on the architecture-specific packages in `config.kiwi`. The Fedora builder line names OVMF or AAVMF, and says the work tmpfs is one container root mounts itself;
  - Tests, the `boot.sh` line: drop "(Arch)", since both bases now have one check file;
  - CI and releases, the "today" line: all four images, the aarch64 ones on the Mac, unsigned; signing joins when it exists.

## Acceptance criteria

Must:
- In the runner VM, `just fedora iso` exits 0 and writes `.build/fedora/aarch64/stackific-0.0.0-dev-fedora-aarch64.iso` and its `.packages.txt`. Its last line prints the ISO size in MiB against 2048 MiB, and the ISO is 2048 MiB or less.
- The aarch64 manifest has these names, each as the first field of a line: `stackific-release`, `stackific-desktop`, `stackific-artwork`, `mangowm`, `noctalia`, `noctalia-greeter`, `greetd`, `greetd-selinux`, `gnome-shell`, `generic-logos`, `NetworkManager-wifi`, `grub2-efi-aa64`, `grub2-efi-aa64-cdboot`, `grub2-efi-aa64-modules` and `kernel`.
- The aarch64 manifest has none of these: `fedora-release`, `fedora-release-common`, `fedora-release-identity-*`, `fedora-logos`, `fedora-release-notes`, `generic-release`, `cardwire`, `grub2-efi-x64*`, `amd-ucode-firmware`, `microcode_ctl` or `openssh-server`.
- `grep '<package .*arch=' fedora/iso/config.kiwi` prints eight lines: five with `arch="x86_64"` (`grub2-efi-x64`, `grub2-efi-x64-cdboot`, `grub2-efi-x64-modules`, `amd-ucode-firmware`, `microcode_ctl`) and three with `arch="aarch64"` (the `grub2-efi-aa64*` packages). It prints nothing else.
- In the runner VM, `just fedora test` exits 0, and every check of `tests/vm/fedora.sh` prints `ok` on aarch64:
  - the os-release fields and `hostnamectl`; `generic-release-common` owns `/etc/os-release` and `stackific-release` owns `/usr/lib/os-release`; no Fedora release or logo packages; the icon's accent fill;
  - only `mangowm` and `noctalia-greeter` come from Terra;
  - `Enforcing`;
  - greetd is `active` as `display-manager.service`; `stackific-noctalia.service` is `active` for `live`, and `live` runs `mango`; both session files exist;
  - the wallpaper, Noctalia, greeter and fastfetch checks;
  - `greetd` owns `/var/lib/noctalia-greeter`, and a `noctalia-greeter` PID runs as `greetd` after the logout, with our asset folder in its environment;
  - `0` AVC denials.
- `just fedora test` leaves `.build/fedora/aarch64/screen.png` and `greeter.png`:
  - `screen.png` shows MangoWM with Noctalia's bar on "An Eruption of Vesuvius", with no setup wizard;
  - `greeter.png` shows noctalia-greeter in our palette on the same painting, with the Stackific logo.
- `git diff -M main` shows `tests/vm/fedora-x86_64.sh` renamed to `tests/vm/fedora.sh`, with only its header comment changed.
- On the operator's PC, `just fedora iso` and `just fedora test` pass on this branch. The x86_64 manifest still lists `grub2-efi-x64`, `grub2-efi-x64-cdboot`, `amd-ucode-firmware` and `microcode_ctl`, and lists no `grub2-efi-aa64*`.
- After the merge, the `images` run on `main` passes every job, `fedora-aarch64` included. The test release `v0.0.0-pr.<n>` holds `stackific-0.0.0-pr.<n>-fedora-aarch64.iso`, its `.packages.txt` and its `.iso.sha256`.
- `just lefthook all` passes: actionlint covers the workflows, and shellcheck covers `tests/vm/fedora.sh` and `tests/vm/boot.sh`.
- The task report states:
  - the aarch64 ISO size in MiB against the 2048 MiB target;
  - the versions the aarch64 manifest took for `mangowm`, `noctalia`, `noctalia-greeter`, `greetd`, `gnome-shell` and `kernel`;
  - the work tmpfs peak in the runner VM.
- If the build runs out of room on the work tmpfs (`No space left on device` under `/work`, or xorriso's `exceeds free space on media`), stop and tell the operator, with the measured peak. The fix is an explicit tmpfs `size=` in `fedora/justfile`, or more memory for the VM in `just runner vm`, and the operator picks.

Must not:
- change any check in the moved check file, any package line of the x86_64 set other than its `arch="x86_64"` attribute, or `config.sh`, `root/`, `terra-repo.sh`, `terra-45.asc`, `fedora/packages/`, `desktop/` or `branding/`;
- add a second kiwi description, a kiwi profile, or an architecture-specific Fedora check file;
- take any package from Terra other than `mangowm` and `noctalia-greeter`, or turn Terra's signature checks off;
- cross-build, emulate (qemu-user or TCG), or pin Fedora packages;
- add `NOPASSWD`, a sudoers file or a polkit rule;
- use sudo or root on the host;
- touch files outside the Changes list (the review record, prompt-trail entry and its screenshots from /s-build excepted), or add a dependency this task does not name.

## Tests
- In the runner VM, run `just fedora iso`. While it runs, sample `df -m /work` in the build container every 5 s (`podman ps` gives its ID), as 008 did. Then run:

  ```sh
  grep -E '^(stackific-|mangowm|noctalia|greetd|gnome-shell|generic-|NetworkManager-wifi|grub2-efi-|kernel\||fedora-|cardwire|amd-ucode|microcode_ctl|openssh-server)' .build/fedora/aarch64/*.packages.txt
  ```

  This proves the build, the size line, both manifest criteria and the tmpfs peak.
- `grep '<package .*arch=' fedora/iso/config.kiwi`: proves the architecture split.
- `git diff -M --stat main` and `git diff -M main -- tests/vm/`: prove the rename and that only the header comment changed.
- In the runner VM, `just fedora test`: proves every check, and leaves both screenshots and `serial.log`. Look at both PNGs. Convert them with `just site screenshot` into `site/src/assets/prompt-trail/010/desktop.webp` and `greeter.webp` for the prompt-trail entry `site/src/content/docs/prompt-trail/fedora/010-fedora-aarch64-desktop-live-iso.md`.
- `git grep -nE 'NOPASSWD|polkit' fedora tests desktop` prints nothing.
- `just lefthook all`.
- Manual, by the operator:
  1. Before the merge, on the PC, run `just fedora iso`, then `just fedora test`. Both must pass. Then grep the x86_64 manifest for `grub2-efi-` and `microcode_ctl`.
  2. After the merge, with both runners online (`just runner status`), watch the `images` run on `main` with `gh run watch`. Every job must finish green, `fedora-aarch64` included.
  3. `gh release view v0.0.0-pr.<n>` lists the Fedora aarch64 ISO, its manifest and its `.iso.sha256`.
- Manual, optional: the GNOME login steps under "Boot test" in `docs/fedora-iso.md`, in any aarch64 UEFI VM with a display (the runner VM has none).
