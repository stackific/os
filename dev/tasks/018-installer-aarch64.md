# 018: The installer on both aarch64 images, with Limine's BOOTAA64.EFI and each base's aarch64 kernel

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Bring the installer to both aarch64 images, Arch Linux ARM and Fedora 45, as the docs reserve task 018 for: stackific-installer, stackific-boot with Limine's BOOTAA64.EFI, GNOME Disks with ntfs-3g and ntfsprogs (on both Fedora images too, which lack it although AGENTS.md puts Disks on all four), Plasma's GTK integration on Arch, the "Install Stackific OS" launcher and desktop shortcut, and the boot menu's Install entry. An install onto free space or a whole drive, encrypted, next to Windows or not, must boot on its own through Limine from our ESP, as it does on x86_64. Prove it with the AAVMF boot, session and unattended-install tests, run on the Mac's runner VM by the images and release workflows.
```

## Goal
Both aarch64 ISOs carry what the x86_64 ISOs carry for installing: the installer, stackific-boot, Disks, the launcher, the desktop shortcut and the Install boot entry. `just arch test-install` and `just fedora test-install` pass on aarch64: the installed system boots from its own disk through Limine's `BOOTAA64.EFI`, unlocks its LUKS2 root, and starts Windows from Limine's menu.

## Context
What exists on `main` at `c7900af` (checked 2026-11-08):
- **The x86_64 install path** (tasks 012 to 017). Step 8 copies Limine's program, the constant `boot::PROGRAM` = `usr/share/limine/BOOTX64.EFI` in the target, to `boot/EFI/limine/BOOTX64.EFI` and `boot/EFI/BOOT/BOOTX64.EFI` (`boot::COPIES`). It creates the firmware entry with `--loader \EFI\limine\BOOTX64.EFI` (`boot::LOADER`). It deletes stale entries by matching `BOOTX64.EFI` (`installer/src/boot.rs`). The fixtures `installer/fixtures/efibootmgr*.txt` and the tests in `boot.rs` and `install.rs` use the same name.
- **Arch's kernel step** (`Arch::initramfs`, `installer/src/base.rs`): `printf '%s\n' usr/lib/modules/*/vmlinuz | /usr/share/libalpm/scripts/mkinitcpio install` in `arch-chroot`. `Arch::kernel_files` = (`vmlinuz-linux`, `initramfs-linux.img`) for `boot::limine_conf`. `Arch::live_medium` = `/run/archiso/bootmnt`, the mounted ISO.
- **Fedora's kernel step**: kernel-install runs in a chroot, and stackific-boot's plugin `fedora/packages/stackific-boot/90-stackific-limine.install` copies `vmlinuz-<version>`. It writes every `limine.conf` entry with `protocol: linux`, `path: boot():/vmlinuz-<v>` and `module_path: boot():/initramfs-<v>.img`. Before that, the installer removes the live image's GRUB with `dnf5 --installroot … remove stackific-installer grub2-efi-x64 grub2-efi-x64-cdboot grub2-efi-x64-modules`. Those are x86_64 names only, marked `shortcut:` in `base.rs`. The aarch64 names are `grub2-efi-aa64`, `grub2-efi-aa64-cdboot` and `grub2-efi-aa64-modules` (`fedora/iso/config.kiwi`).
- **stackific-boot keeps Limine current.** On Arch, `arch/packages/stackific-boot/stackific-limine.hook` re-copies `/usr/share/limine/BOOTX64.EFI`. On Fedora, the `limine` trigger in `fedora/packages/stackific-boot/stackific-boot.spec` does the same, with the same name.
- **Where aarch64 is gated off today:**
  - `arch/iso/packages.aarch64` lacks `stackific-installer`, `stackific-boot`, `gnome-disk-utility`, `ntfs-3g`, `ntfsprogs`, `breeze-gtk` and `kde-gtk-config`. Otherwise it mirrors `packages.x86_64` (`docs/arch-iso.md`, "aarch64").
  - `just arch iso` deletes `efiboot/loader/entries/02-stackific-install.conf` and `airootfs/home/live/Desktop/stackific-installer.desktop` when not on x86_64. The entry's title already uses `%ARCH%`.
  - `fedora/iso/config.kiwi` lists `stackific-installer` and `stackific-boot` with `arch="x86_64"`.
  - `fedora/iso/edit-boot.sh` exits on any other architecture, and it writes "(x86_64, UEFI)" into the titles itself.
  - Neither Fedora image ships GNOME Disks: `config.kiwi` has no `gnome-disk-utility`, `ntfs-3g` or `ntfsprogs`, and no Fedora check looks for them. `AGENTS.md` makes Disks the partition editor on all four images, so this task closes that gap on both Fedora images.
  - `fedora/iso/terra-repo.sh` already allows `limine`; `tests/vm/fedora.sh` notes "limine only on x86_64".
- **Tests.** The installer checks for each base sit in `tests/vm/arch-x86_64.sh` and `tests/vm/fedora-x86_64.sh`: the launcher, Disks, Plasma's GTK look and look-and-feel with its screenshot, the units, and the plugin. So do the logins into Plasma and GNOME with their screenshots. `tests/vm/install.sh`, `tests/vm/installed.sh` and `tests/vm/screens.sh` hard-code `BOOTX64.EFI`, including the fake Windows that uses Limine's program as `bootmgfw.efi`. `installed.sh` expects Arch's `vmlinuz-linux` and the `linux` protocol entries. `boot.sh` already runs aarch64 (`virt`, AAVMF, `virtio-gpu-pci`), and both `test-install` recipes already use `{{ arch() }}`.
- **CI.** The `arch-aarch64` and `fedora-aarch64` jobs in `.github/workflows/images.yml` and `release.yml` run `iso` and `test`, not `test-install`. In `release/justfile`, `just release changed` leaves `installer/`, `tests/vm/install.sh`, `installed.sh`, `unlock.sh` and `tests/vars.just` out of the aarch64 path lists.

Upstream facts (checked 2026-11-08):
- **Limine on Arch Linux ARM.** ALARM's `extra/limine` is 12.9.6-1, built 2026-10-28, and ships `/usr/share/limine/BOOTAA64.EFI` (file list of http://mirror.archlinuxarm.org/aarch64/extra/). These are in ALARM aarch64 too: `efibootmgr` 18-4 and `cryptsetup` 2.8.8-1 (core), and `gnome-disk-utility` 46.1-3, `ntfs-3g` and `ntfsprogs` 2026.9.28-1 (split packages), `breeze-gtk` and `kde-gtk-config` 6.8.2-1, `rsync` 3.5.2-1 and `arch-install-scripts` 31-2 (extra).
- **Limine on Fedora aarch64.** Terra's `limine` is 12.9.6-1.fc45 for aarch64 and x86_64, and the aarch64 rpm ships `/usr/share/limine/BOOTAA64.EFI` (https://repos.fyralabs.com/terra45/repodata/). Fedora 45 aarch64 has `efibootmgr` 18-13, `ntfsprogs` 2026.2.25-3, `gnome-disk-utility` 46.1-9 and `cryptsetup` 2.8.8-1 (https://dl.fedoraproject.org/pub/fedora/linux/releases/45/Everything/aarch64/os/Packages/). The builder checks `ntfs-3g` and the x86_64 versions when adding them.
- **ALARM's kernel** (`linux-aarch64` 7.3.3-1, http://mirror.archlinuxarm.org/aarch64/core/linux-aarch64-7.3.3-1-aarch64.pkg.tar.xz, and its PKGBUILD at https://github.com/archlinuxarm/PKGBUILDs/blob/master/core/linux-aarch64/PKGBUILD):
  - The package's own files go straight into `/boot`: `Image` (raw arm64 Image with the `ARMd` magic, 51.5 MB), `Image.gz` and `dtbs/**` (about 205 MB in all).
  - It has no `/usr/lib/modules/*/vmlinuz` and no `pkgbase`, so mkinitcpio's libalpm script skips it. The x86_64 step 8 command would do nothing.
  - It installs the dummy file `/usr/lib/initcpio/<kver>`. Piping that path into `/usr/share/libalpm/scripts/mkinitcpio install` runs `mkinitcpio -P` (mkinitcpio 42.2-1).
  - Its preset `linux-aarch64.preset` has `PRESETS=('default')` and `default_image="/boot/initramfs-linux.img"`.
  - The kernel config has `CONFIG_EFI_ZBOOT` unset and `CONFIG_ACPI=y`.
- **mkarchiso 92 empties the live root's `/boot`.** `_cleanup_pacstrap_dir` runs `find "${pacstrap_dir}/boot" -mindepth 1 -delete` after copying `/boot` out (https://gitlab.archlinux.org/archlinux/archiso/-/raw/v92/archiso/mkarchiso, lines 165–166 and 486–487). On aarch64 the kernel survives only on the ISO, as `arch/boot/aarch64/vmlinuz-linux`. That file is the live image's `stackific-vmlinuz.hook` copy of `/boot/Image`, and the live session sees it under `/run/archiso/bootmnt/`.
- **Fedora's aarch64 kernel is an EFI zboot image.** `kernel-core-7.2.15-300.fc45.aarch64` ships `vmlinuz` as a zstd-compressed PE file: `MZ` then `zimg`, with `CONFIG_EFI_ZBOOT=y` (https://dl.fedoraproject.org/pub/fedora/linux/updates/45/Everything/aarch64/Packages/k/).
- **Limine's `linux` protocol on aarch64** (Limine v12.9.6, `common/protos/linux_risc.c`, https://github.com/limine-bootloader/limine/tree/v12.9.6):
  - It takes a raw `Image` and checks the `ARMd` magic at offset 0x38. It decompresses gzip only, through a `$` path prefix.
  - It does not take zboot. Upstream issue #518 was closed 2025-08-02 with the `efi` protocol as the workaround (https://github.com/Limine-Bootloader/Limine/issues/518).
  - With no device tree it passes one holding only `/chosen`, and Linux arm64 then uses ACPI (`dt_is_stub()`, `arch/arm64/kernel/acpi.c`).
- **Limine's `efi` protocol** (same source, `chainload.c`):
  - It starts any PE image, including a zboot kernel, which decompresses itself, and passes `cmdline` as the image's load options.
  - The image's device is the partition the file came from. The kernel's EFI stub reads `initrd=\<path>` from that volume (`efi_load_initrd()`, `drivers/firmware/efi/libstub/efi-stub-helper.c`). Fedora's config has no option that turns this off.
  - `efi` and `efi_boot_entry`, the two Windows entries, are built for UEFI on every architecture.
- **Windows on ARM.** BCDBoot writes the same layout on every architecture, `\EFI\Microsoft\Boot\` with the firmware entry "Windows Boot Manager" (https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/bcdboot-command-line-options-techref-di, updated 2026-03-23). The UEFI removable-media path for AArch64 is `\EFI\BOOT\BOOTAA64.EFI` (UEFI 2.10, §3.5.1.1).

## Inputs
None. The task number and scope come from the tracked docs (`docs/installer.md`, "Later": "Task 018: the installer on both aarch64 images … with Limine's `BOOTAA64.EFI`"). The operator confirmed task 018 as the next task on 2026-11-08.

## Design
- **Limine's program by architecture.** The installer picks the file name at compile time: `BOOTX64.EFI` on x86_64, `BOOTAA64.EFI` on aarch64 (`cfg!(target_arch)`). `PROGRAM`, `COPIES`, `LOADER` and the stale-entry match all use it. The firmware's fallback copy goes to `\EFI\BOOT\BOOTAA64.EFI`. The tests that compare exact commands get their name from the same constant, so they pass on both architectures.
- **stackific-boot's re-copy.** Arch's hook and Fedora's trigger copy the program for the running machine: `BOOTX64.EFI` when `uname -m` is `x86_64`, `BOOTAA64.EFI` when it is `aarch64`. Both packages stay `any`/`noarch`.
- **Arch aarch64 kernel.** On aarch64, `Arch::initramfs` first copies `<live_medium>/arch/boot/aarch64/vmlinuz-linux` to `<target>/boot/Image`, the path `linux-aarch64` owns. It then runs mkinitcpio's libalpm script in `arch-chroot` on `usr/lib/initcpio/*` instead of `usr/lib/modules/*/vmlinuz`. The default preset writes `/boot/initramfs-linux.img`, with the installer's `stackific.conf` hooks drop-in as on x86_64. `Arch::kernel_files` on aarch64 is (`Image`, `initramfs-linux.img`), with the `linux` protocol as on x86_64. Later upgrades of `linux-aarch64` write `/boot/Image` on the ESP themselves.
- **Arch's live-only hook.** `stackific-vmlinuz.hook` (`/etc/pacman.d/hooks/` in the airootfs) is live-image glue. `Arch::live_state` deletes it from the target, so installed aarch64 systems keep no second 51 MB copy.
- **Fedora aarch64 kernel.** On aarch64, the plugin writes each kernel's entry with `protocol: efi`, `path: boot():/vmlinuz-<v>` and `cmdline: initrd=\initramfs-<v>.img <the /etc/kernel/cmdline text>`. The kernel's own EFI stub then decompresses it and loads the initramfs from the ESP. x86_64 entries do not change. The plugin picks the format with `uname -m`, which the chroot reports as the machine's.
  - I prefer this to unpacking the zboot payload into a raw `Image`. Unpacking needs glue code (offsets and zstd) and about 79 MB of ESP per kernel. This way is upstream Limine's documented workaround.
- **The console on aarch64.** The installed command line ends with `console=tty0` on aarch64 (`boot::CONSOLE`), on both bases (Fedora's plugin reads it from `/etc/kernel/cmdline`). Without it, Linux on arm64 makes the serial port in ACPI's SPCR table the console, and systemd asks for the passphrase there, off the screen (found by the first aarch64 `test-install` run, 2026-11-08: the prompt appeared only in `serial.log`). x86_64's command line does not change.
- **Fedora's GRUB removal.** `Fedora::live_state` and its package list name the GRUB packages of the installer's own architecture. That removes the `shortcut:` in `base.rs`.
- **Images.**
  - Arch: the seven packages join `packages.aarch64` in `packages.x86_64`'s order, and `just arch iso` stops deleting the Install entry and the desktop shortcut.
  - Fedora: `config.kiwi` drops `arch="x86_64"` from `stackific-installer` and `stackific-boot`, and adds `gnome-disk-utility`, `ntfs-3g` and `ntfsprogs` on both architectures. GNOME Shell's own polkit agent answers Disks' prompts.
  - Fedora: `edit-boot.sh` runs on both architectures and writes `(<uname -m>, UEFI)` into both titles. Its awk code works on kiwi's aarch64 `grub.cfg` (the builder confirms the entry shape, and the test checks it).
- **Tests.**
  - The installer checks move out of `arch-x86_64.sh` into `arch.sh`, and out of `fedora-x86_64.sh` into `fedora.sh`. So do the Plasma and GNOME logins with their screenshots. Both `-x86_64.sh` files are then deleted.
  - A check that fails on aarch64 for a reason this task cannot fix stays in a new `tests/vm/<base>-x86_64.sh`. The review record says why, and `docs/installer.md`'s improvements list gains a line for it.
  - `install.sh`, `installed.sh` and `screens.sh` use `BOOTAA64.EFI` on aarch64. `installed.sh` expects Arch's `Image` on aarch64, and Fedora's `protocol: efi` entries with `initrd=` on aarch64.
  - The Arch hook check downloads `limine` from the builder's mirror, which is ALARM's live repo on aarch64, as today. Fedora's `test-install` downloads the `limine` and `kernel-core` named in the image's manifest for `{{ arch() }}`.
- **CI.** The aarch64 jobs in `images.yml` and `release.yml` run `just <base> test-install "$VERSION"` after `test`, as the x86_64 jobs do. `just release changed` adds `installer/`, `tests/vm/install.sh`, `tests/vm/installed.sh`, `tests/vm/unlock.sh` and `tests/vars.just` to both aarch64 lists, and moves `tests/vm/arch.sh` and `tests/vm/fedora.sh` to cover both architectures as today.

## Changes
- `installer/src/boot.rs`: Limine's program name by architecture in `PROGRAM`, `COPIES`, `LOADER` and the stale match, tests.
- `installer/src/base.rs`: Arch aarch64's kernel copy, mkinitcpio input, `kernel_files` and `live_state` (the live hook); Fedora's GRUB names by architecture, tests.
- `installer/src/install.rs`: test expectations built from the program name.
- `installer/fixtures/efibootmgr.txt`, `installer/fixtures/efibootmgr-windows.txt`: only if the tests need an aarch64 twin.
- `arch/packages/stackific-boot/stackific-limine.hook`, `arch/packages/stackific-boot/PKGBUILD` (pkgrel, sha256, comment).
- `fedora/packages/stackific-boot/stackific-boot.spec` (trigger, Release), `fedora/packages/stackific-boot/90-stackific-limine.install` (aarch64 entries).
- `arch/iso/packages.aarch64`: the seven packages.
- `arch/justfile`: no aarch64 gating in `iso`; comments of `test` and `test-install`.
- `fedora/iso/config.kiwi` (the installer on aarch64, Disks on both), `fedora/iso/edit-boot.sh`.
- `tests/vm/arch.sh`, `tests/vm/fedora.sh` (checks moved in); `tests/vm/arch-x86_64.sh`, `tests/vm/fedora-x86_64.sh` (deleted unless a check must stay); `tests/vm/install.sh`, `tests/vm/installed.sh`, `tests/vm/screens.sh`.
- `fedora/justfile`: comments of `test` and `test-install`.
- `.github/workflows/images.yml`, `.github/workflows/release.yml`: `test-install` in the aarch64 jobs.
- `release/justfile`: `just release changed` path lists.
- `docs/installer.md`, `docs/arch-iso.md`, `docs/fedora-iso.md` (Disks too), `docs/checks.md`: aarch64 is no longer "until task 018"; each aarch64 ISO's size.
- `site/src/content/docs/user/installation.md`: the installer on all four images.
- `release/notes.md`: the early installer on every ISO.
- `AGENTS.md`: the installer line ("the aarch64 images get it with task 018"), the boot-chain line (Arch aarch64's `Image` from the medium, Fedora aarch64's `protocol: efi`), the tests line on per-arch check files.
- The prompt-trail entry and review record (`/s-build`).

## Acceptance criteria

Must:
- `cargo test` in `installer/` passes on x86_64. Run on aarch64 (the Mac's runner VM, or `just arch packages` there, which builds the crate), the boot tests expect `\EFI\limine\BOOTAA64.EFI`.
- `just arch iso` and `just fedora iso` on aarch64 print each ISO's size. The packages lists (`.packages.txt`) include `stackific-installer`, `stackific-boot`, `limine`, `gnome-disk-utility` and `ntfsprogs`, plus `breeze-gtk` and `kde-gtk-config` on Arch. The Fedora x86_64 list gains `gnome-disk-utility`, `ntfs-3g` and `ntfsprogs` too.
- `just arch test` and `just fedora test` pass on aarch64 and on x86_64, with the installer, launcher, Disks and desktop-session checks from the shared check files. On aarch64 they leave `plasma.png` or `gnome.png`.
- `just arch test-install` and `just fedora test-install` pass on aarch64 and on x86_64:
  - the boot menu's Install entry reads "Install Stackific OS (aarch64, UEFI)";
  - the installed system boots from its disk alone through `\EFI\limine\BOOTAA64.EFI`, its LUKS2 root unlocked by the credential and, in the last boot, by the typed passphrase;
  - Limine's two Windows entries start the fake Windows;
  - stackific-boot's re-copy restores a truncated `BOOTAA64.EFI` at both ESP paths.
- On installed Arch aarch64, `/boot/Image` and `/boot/initramfs-linux.img` sit on the ESP, `limine.conf` names `boot():/Image`, and `/etc/pacman.d/hooks/stackific-vmlinuz.hook` is gone.
- On installed Fedora aarch64, every `limine.conf` kernel entry has `protocol: efi` and `cmdline: initrd=\initramfs-<v>.img rd.luks.uuid=…`. A reinstalled `kernel-core` writes the same.
- On installed Fedora aarch64, no `grub2-efi-aa64*` package is installed.
- On both installed aarch64 systems, the command line ends with `console=tty0`, and the passphrase prompt shows on the screen.
- The `images` workflow started by hand passes all four jobs, with `test-install` in the two aarch64 ones.
- `just lefthook all` passes.

Must not:
- change what an x86_64 image boots or installs (its `limine.conf` entries and file names stay as they are; its package lists change only by Disks on Fedora);
- copy Arch's `/boot/dtbs` to the ESP: ACPI machines do not need them, and `linux-aarch64`'s next upgrade writes them. The improvements list in `docs/installer.md` notes that `pacman -Qk linux-aarch64` reports them missing until then;
- unpack Fedora's zboot kernel, or add Limine's `$` gzip path;
- add a package to an image beyond the seven named for Arch aarch64 and the three Disks packages for Fedora;
- touch files outside the Changes list.

## Tests
- `cargo test` (pre-commit through `just lefthook all` on x86_64), for the program name and the Arch and Fedora aarch64 commands under `cfg(target_arch)`: the exact-command tests build their expectations from the shared constant.
- `just arch test`, `just fedora test`, `just arch test-install` and `just fedora test-install` on x86_64 (the PC) for no regressions.
- The same four recipes on aarch64. They need the Mac's runner VM, which the builder cannot reach from the PC. The operator runs them:
  1. Push the branch and start the `images` workflow by hand on it (Actions → images → Run workflow), or run step 2 in the VM.
  2. In the VM (`limactl shell stackific-runner`), from the checkout: `just arch iso && just arch test && just arch test-install`, then `just fedora packages && just fedora iso && just fedora test && just fedora test-install`.
  3. Send the builder each failing recipe's last 40 lines and `.build/<base>/aarch64/limine.png`.
- `just arch screenshots` on aarch64 is not needed: the installer's screens are the same on both architectures, so the prompt-trail entry reuses the x86_64 screenshots only where a screen changed, and here none does.
