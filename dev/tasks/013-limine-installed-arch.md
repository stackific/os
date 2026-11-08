# 013: The installed Arch system starts: Limine from our own ESP, first in the firmware's boot order

- [x] Done

Base: arch

## Ask

```text title="Prompt" wrap
Make the drive the installer writes start on its own, on the Arch x86_64 image. Limine boots it from our own 4 GiB FAT32 EFI System Partition, never another OS's. The kernel and its initramfs reach the ESP through Arch's own mkinitcpio hooks, with no fallback initramfs and no unified kernel image, and Limine's entry finds root through rootflags=subvol=/@. The firmware's boot entry is labelled with the product name, "Stackific OS", and comes first in the boot order; the installer keeps every other entry in its order and writes NVRAM only at install. The install test boots the installed system and checks it. Change the installer's Done screen, the user docs and the release notes, which say today that the drive cannot start yet. On the booted system, check the two things task 012 left open (whether KDE Plasma takes its first keyboard layout from systemd-localed's X11 setting, and whether MangoWM reads the user's ~/.config/mango/config.conf rather than /etc/mango/config.toml), and fix what the check finds.
```

## Goal
After an install from the Arch x86_64 live ISO, in either mode, the computer starts Stackific OS from the drive: the firmware's "Stackific OS" entry starts Limine, Limine's menu starts the kernel, and the system reaches its login screen. `just arch test-install` proves it by booting the installed disk image and checking the result.

## Context
What exists (main at `cb4e2f1`, checked 2026-11-07):
- `installer/` (task 012) installs into a drive's largest free space or onto a whole drive, in seven steps (`installer/src/install.rs`, `STEPS`): create the partitions, format them, prepare the system partition, copy the system, set the keyboard, timezone and computer name, create the user, remove the live session's settings. The ESP is partition "Stackific OS boot" (GPT type `C12A7328-F81F-11D2-BA4B-00A0C93EC93B`, FAT32, 4 GiB), mounted at the target's `/boot` during the install; `/etc/fstab` mounts it at `/boot` (`vfat umask=0077 0 2`) and `/` with `subvol=/@,noatime,compress=zstd:1` by UUID (`installer/src/disk.rs`, `fstab`).
- The installer reads `NAME`, `ID`, `DEFAULT_HOSTNAME` and the accent from `/etc/os-release` (`installer/src/os_release.rs`) and hard-codes none; partition labels already use `NAME`.
- `installer/src/base.rs`: the `Base` trait (live root `/run/archiso/airootfs`, live medium `/run/archiso/bootmnt`, `live_state`). Arch's `live_state` removes `/etc/mkinitcpio.conf.d/archiso.conf` (archiso's `HOOKS=(base udev modconf archiso block filesystems)`) and runs `pacman --sysroot=<target> --remove --nosave --recursive --noconfirm stackific-installer`. `--recursive` removes only dependencies that nothing else needs and that were not installed explicitly (pacman(8)).
- Every command goes through the `Runner` trait (`installer/src/run.rs`: `run`, `stream`, `read`, `write`, `remove`; `Cmd::stdin`), faked in tests and in `just installer demo`. Screen snapshots are `installer/fixtures/screens/*.txt` (`installing.txt`, `done.txt`, `done-console.txt` among them).
- The Done screen (`installer/src/ui/done.rs`) says: "The boot loader is not in this build yet, so the drive does not start on its own. Keep using the live session; a later build adds the boot loader." The same limit is in `release/notes.md` (first paragraph), `site/src/content/docs/user/installation.md` ("Limits of this preview") and `docs/installer.md` (line 5 and "Later").
- The live ISO boots through systemd-boot (`arch/iso/efiboot/loader/loader.conf`, `timeout 3`). `arch/iso/packages.x86_64` has `linux`, `amd-ucode`, `intel-ucode`, `mkinitcpio`, `mkinitcpio-archiso`, `btrfs-progs`, `dosfstools`, but not `limine`, `efibootmgr` or `arch-install-scripts` (ISO manifest of 2026-11-06: `linux` 7.2.11.arch1-1, `mkinitcpio` 42.2-1, `systemd` 262.1-1).
- mkarchiso empties `/boot` before packing, so the copied root has no kernel in `/boot`; the kernel is `/usr/lib/modules/<version>/vmlinuz` (task 012). The live root keeps `/etc/mkinitcpio.d/linux.preset`, written at build time by mkinitcpio's pacman hook from its `hook.preset`.
- `just arch packages` copies only each `PKGBUILD` and the files its `source=()` names from `desktop/`, `branding/` and `installer/` into the build directory (`arch/justfile`, `packages`).
- `just arch test-install` (`arch/justfile`): checks the ISO's boot menu with bsdtar, then one boot of the ISO with a blank 64 GiB raw disk (`tests/vm/boot.sh` sourcing `tests/vm/install.sh`): the console path, a refused erase, an erase, a Windows-like table with a BitLocker refusal, then a free-space install next to the Windows-like partitions. `install.sh` checks each installed system from the live VM (`installed_checks`) and leaves `<out>/esp-offset`, the free-space install's ESP in bytes; the recipe then reads it with `minfo` (mtools). `boot.sh` copies `OVMF_VARS` (default `/usr/share/edk2/x64/OVMF_VARS.4m.fd`) to a temporary `vars.fd`, always passes `-cdrom <iso>`, attaches `STACKIFIC_TEST_DISK` as virtio `/dev/vda`, logs serial to `<out>/serial.log`, and has `vm`, `vm_until`, `sendkey`, `screenshot` and `fail` for the checks file. After boot it requires `systemctl is-system-running` to be `running` and NetworkManager running.
- OVMF prints the entry it starts on the serial console: `BdsDxe: starting Boot0002 "UEFI QEMU DVD-ROM QM00005 " from PciRoot(0x0)/...` (`.build/arch/x86_64/serial.log` of a test run, read 2026-11-07).
- `release/justfile` `changed`: the `arch_x86_64` image rebuilds when `arch/packages/`, `arch/iso/packages.x86_64`, `installer/`, `tests/vm/install.sh` or `tests/vm/arch-x86_64.sh` change, among others.
- `images.yml` and `release.yml` run `just arch test-install` in the `arch-x86_64` job.

Upstream facts (checked 2026-11-07):
- `limine` 12.9.6-1 in Arch `extra` at the archive date 2026-11-01 (https://archive.archlinux.org/repos/2026/11/01/extra/os/x86_64/limine-12.9.6-1-x86_64.pkg.tar.zst; 5386.22 KiB installed; depends on `glibc`): `/usr/bin/limine`, `/usr/share/limine/BOOTX64.EFI` (and `BOOTAA64.EFI` for aarch64), docs under `/usr/share/doc/limine/`. It ships no pacman hook, no `limine-install` and no default config (package file list; https://gitlab.archlinux.org/archlinux/packaging/packages/limine/-/blob/12.9.6-1/PKGBUILD; https://wiki.archlinux.org/title/Limine, revision 887253).
- Limine reads FAT12/16/32 and ISO9660, not btrfs (https://github.com/Limine-Bootloader/Limine/blob/v12.9.6/README.md), so the kernel and initramfs must be on the ESP.
- Limine's config search on UEFI (https://github.com/Limine-Bootloader/Limine/blob/v12.9.6/CONFIG.md): the volume holding the running EFI program first, then the drive, then every partition. On each volume, in order: `<EFI program's folder>/limine.conf`, `/boot/limine/limine.conf`, `/boot/limine.conf`, `/limine/limine.conf`, `/limine.conf`; the first found wins. So one `limine.conf` at the ESP's root serves both copies of the EFI program below, and our ESP is searched before any other partition.
- `limine.conf` syntax (same CONFIG.md): global options before the first entry; `timeout: <seconds>`; `interface_branding: <text>` replaces the menu's "Limine" header; a line `/<name>` opens an entry with `protocol: linux`, `path:` (the kernel), `module_path:` (the initramfs; may repeat) and `cmdline:`. `boot():/` is the partition that holds `limine.conf`. Secure Boot and `limine enroll-config` are opt-in; with no enrolled config hash, Limine treats Secure Boot as inactive (USAGE.md at v12.9.6).
- `mkinitcpio` 42.2-1 (core, 2026-11-01): its `hook.preset` is `ALL_kver="/boot/vmlinuz-%PKGBASE%"`, `PRESETS=('default')`, `default_image="/boot/initramfs-%PKGBASE%.img"`; the fallback image is commented out ("The default kernel preset files no longer includes the fallback image", CHANGELOG v40). Arch builds it with systemd hooks: `HOOKS=(base systemd autodetect microcode modconf kms keyboard sd-vconsole block filesystems fsck)`. The `microcode` hook prepends the CPU's microcode to the initramfs, so no separate microcode file is loaded (https://gitlab.archlinux.org/archlinux/mkinitcpio/mkinitcpio/-/tree/v42.2: `mkinitcpio.d/hook.preset`, `install/microcode`; https://gitlab.archlinux.org/archlinux/packaging/packages/mkinitcpio/-/blob/42.2-1/PKGBUILD).
- `/usr/share/libalpm/scripts/mkinitcpio install` (mkinitcpio's pacman-hook script, v42.2) reads targets such as `usr/lib/modules/<version>/vmlinuz` on stdin; for each it reads `pkgbase` from that folder, writes `/etc/mkinitcpio.d/<pkgbase>.preset` from `hook.preset` only when missing, installs the kernel to `/boot/vmlinuz-<pkgbase>`, and runs `mkinitcpio -p <pkgbase>`. `90-mkinitcpio-install.hook` runs it after every kernel install or upgrade (and after changes to initcpio files, firmware, systemd and similar); `60-mkinitcpio-remove.hook` removes a kernel's files before the kernel package goes. So with the ESP at `/boot`, `vmlinuz-linux` and `initramfs-linux.img` stay current on the ESP with no further setup.
- `efibootmgr` 18-4 (core, 2026-11-01; 83.85 KiB): `-c`/`--create` creates an entry and "puts it at the top of the boot order list" (efibootmgr(8); the binary's `-I`/`--index` defaults to 0); `-d`/`--disk`, `-p`/`--part`, `-L`/`--label`, `-l`/`--loader`; `-b`/`--bootnum` with `-B`/`--delete-bootnum` deletes one entry; `-o`/`--bootorder` sets the order (https://github.com/rhboot/efibootmgr/blob/18/src/efibootmgr.8.in).
- `arch-install-scripts` 32-1 (extra, 2026-11-01; 39.35 KiB): `arch-chroot <dir> <command>` mounts `/proc`, `/sys`, `/dev`, `/run` (and efivarfs) into the target and runs the command there (the builder's `pacman -Si`, read 2026-11-07).
- The Arch wiki's Limine page suggests the same layout (`esp/EFI/<folder>/BOOTX64.EFI`, an efibootmgr entry, `path: boot():/vmlinuz-linux`, `module_path: boot():/initramfs-linux.img`, `cmdline: root=UUID=… rw`, with `rootflags=subvol=/@` in its btrfs example) and a pacman hook that copies `BOOTX64.EFI` after `limine` upgrades (https://wiki.archlinux.org/title/Limine, revision 887253 of 2026-10-08).
- The UEFI fallback path for removable media on x86_64 is `\EFI\BOOT\BOOTX64.EFI`; most firmware boots it from any drive (https://wiki.archlinux.org/title/Unified_Extensible_Firmware_Interface, "Default boot path for removable drives").
- `limine-mkinitcpio-hook` and `limine-entry-tool` 1.40.0-1 (Zesko) are in the AUR only, not in Arch's repos (https://aur.archlinux.org/packages/limine-mkinitcpio-hook, checked 2026-11-07).

Keyboard facts task 012 left open, settled from source (versions in the 2026-11-01 archive: `mangowm` 0.18.4-1, `kwin`, `plasma-workspace` and `plasma-desktop` 6.8.2-1, `systemd` 262.1-1; checked 2026-11-07). Both answers are yes, so the installer needs no change, and the booted-system test checks both at runtime:
- MangoWM reads exactly one file, the first that exists: `-c <path>`, `~/.config/mango/config.conf`, `~/.config/mango/config.toml`, `/etc/mango/config.conf`, `/etc/mango/config.toml`; the `/etc` files are a fallback, never merged, and `$XDG_CONFIG_HOME` is not read (https://github.com/mangowm/mango/blob/0.18.4/src/config/load.c#L392-L417). `source=<file>` parses that file in place, and a later `xkb_rules_layout` line overwrites an earlier one (`src/config/option_defs.c#L2352-L2355`, `#L292-L295`), so the installer's `source=/etc/mango/config.conf` then `xkb_rules_layout=de` gives `de`.
- `mmsg get keyboardlayout` (installed with `mangowm`) prints one JSON line `{"layout":"<name>"}`, the layout's long xkb name (expected `German` for `de`; not run yet), and needs `MANGO_INSTANCE_SIGNATURE`, which the compositor sets to `$XDG_RUNTIME_DIR/mango-<pid>.sock` (`mmsg/mmsg.c#L20-L21`, `#L97-L103`; `src/ipc/ipc.c#L685-L687`, `#L1357-L1358`, at tag 0.18.4).
- Plasma follows localed while the user has no `LayoutList` in `kxkbrc`: `startplasma-wayland` copies `org.freedesktop.locale1`'s `X11Layout` into `XKB_DEFAULT_LAYOUT` (https://invent.kde.org/plasma/plasma-workspace/-/blob/v6.8.2/startkde/startplasma.cpp#L815-L845), and KWin uses `XKB_DEFAULT_LAYOUT` when `kxkbrc` has no layout (https://invent.kde.org/plasma/kwin/-/blob/v6.8.2/src/xkb.cpp#L560-L578). KWin never reads `00-keyboard.conf` itself; systemd-localed does, and `XKBLAYOUT=` in `/etc/vconsole.conf` would take precedence over it (https://github.com/systemd/systemd/blob/v262.1/src/locale/localed-util.c#L117-L127), so `localectl`'s X11 layout is the value to check.
- The active Plasma layouts: `busctl --user call org.kde.keyboard /Layouts org.kde.KeyboardLayouts getLayoutsList` returns `a(sss)`, each layout's short name (`de`) first (kwin v6.8.2 `src/keyboard_layout.h#L80-L98`, `src/keyboard_layout.cpp#L185-L244`).
- The live test reaches Plasma by pointing greetd's `[initial_session]` at `startplasma-wayland` from the test shell and restarting greetd (`tests/vm/arch-x86_64.sh`); the image itself ships no such change.

## Inputs
From the operator's working plan, item 013 ("Limine on installed Arch: entries through mkinitcpio hooks, NVRAM label and boot order, install test boots the result"), and the decisions it names, each checked against `AGENTS.md` and upstream on 2026-11-07. Nothing is copied into the tree.
- **BT-02, the firmware entry's label:** the installer creates the NVRAM entry itself with `efibootmgr --create --label "<NAME>"`, `NAME` from `/etc/os-release` ("Stackific OS"), so the firmware menu shows the product name instead of "Limine" or "Arch Linux Limine Bootloader". Before creating it, the installer deletes stale entries: those that start our loader on a partition the computer no longer has (a reinstall over an earlier Stackific OS). Deviation: the loader is `\EFI\limine\BOOTX64.EFI` (the Arch wiki's file name in our own folder), not `\EFI\limine\limine_x64.efi`, which only mattered for limine-entry-tool, which this task does not use.
- **BT-03, the boot order:** the new entry goes first in `BootOrder` at install (efibootmgr's default), and every other entry keeps its place after it. A copy of the same EFI program at `\EFI\BOOT\BOOTX64.EFI` on our ESP (never on another OS's) lets the firmware start the drive when its entry is lost. Nothing writes NVRAM at boot or on updates. Left for later: re-creating a lost entry on updates (it came from limine-entry-tool), the guard that Windows Boot Manager's entry survives (task 015, with Windows), the user docs on putting Stackific OS first again after a big Windows update (task 015). Dropped: the `sos` repair command, because `AGENTS.md` rules out our own command-line tool.
- **BT-07, root by a fixed subvolume path:** Limine's entry carries `rootflags=subvol=/@`, fstab mounts `/` with `subvol=/@` (task 012), and nothing sets the btrfs default subvolume, which stays the top level (ID 5). The snapshot-entry parts (limine-snapper-sync) are dropped: `AGENTS.md` keeps backup to snapper with btrfs-assistant.
- **BT-08, the ESP's contents:** separate `vmlinuz` and initramfs files, no fallback initramfs and no unified kernel image. mkinitcpio 42.2's default preset already builds no fallback image, so nothing changes it. The space-based pruning is dropped: no snapshot entries live on the ESP, and one kernel set fits a 4 GiB ESP many times over.
- **K-11, the kernel command line:** taken: `rootflags=subvol=/@` and `rw`, after `root=UUID=<system partition>`. Left for their tasks: the LUKS options (task 016) and `zswap.enabled=0` (with zram, task 026). Dropped: the boot-failure initramfs hook and limine-entry-tool's config; the initramfs keeps Arch's default hooks.
- **"Entries through mkinitcpio hooks", as built here:** Arch's own mkinitcpio pacman hooks keep `/boot/vmlinuz-linux` and `/boot/initramfs-linux.img` current, and `limine.conf`'s one entry names those fixed paths, so a kernel update never rewrites `limine.conf`. Deviation from limine-mkinitcpio-hook: it is AUR-only (we would build and carry it), and with one kernel and no snapshot entries its entry management has nothing to do. A second kernel (`linux-lts`, task 019) adds its own entry then.

## Design
- **New step 8, "Install the boot loader",** after "Remove the live session's settings" (archiso's mkinitcpio drop-in must be gone first). In order:
  1. The initramfs (per base, behind `Base`): Arch runs mkinitcpio's own pacman-hook script in the target, `arch-chroot <target> /usr/share/libalpm/scripts/mkinitcpio install`, with every `usr/lib/modules/*/vmlinuz` of the target on stdin, as pacman does when it installs a kernel. That writes `/boot/vmlinuz-linux` and `/boot/initramfs-linux.img` on the ESP with Arch's default hooks.
  2. The EFI program (shared): the target's `/usr/share/limine/BOOTX64.EFI`, installed (`install -D -m 0644`) to `<target>/boot/EFI/limine/BOOTX64.EFI` and `<target>/boot/EFI/BOOT/BOOTX64.EFI`.
  3. `limine.conf` (shared; the kernel and initramfs file names from `Base`), written to `<target>/boot/limine.conf`, the ESP's root:

     ```text
     timeout: 3
     interface_branding: Stackific OS

     /Stackific OS
         protocol: linux
         path: boot():/vmlinuz-linux
         module_path: boot():/initramfs-linux.img
         cmdline: root=UUID=<system partition's UUID> rootflags=subvol=/@ rw
     ```

     `Stackific OS` is os-release's `NAME`; `timeout: 3` matches the live ISO's boot menu.
  4. The firmware entry (shared): list the entries (`efibootmgr` with its device paths), delete each one whose file is `\EFI\limine\BOOTX64.EFI` on a partition GUID no present partition has (`efibootmgr --bootnum <n> --delete-bootnum`), then `efibootmgr --create --disk <drive> --part <ESP's partition number> --label "<NAME>" --loader '\EFI\limine\BOOTX64.EFI'`. It runs last, so a failure earlier leaves no entry behind.
- **`stackific-boot` (new Arch package, `arch=('any')`):** `depends=('limine' 'efibootmgr')`, so both stay on the installed system, and one pacman hook, `/usr/share/libalpm/hooks/stackific-limine.hook`: on `Operation = Upgrade` of the package `limine` ("Installations are considered an upgrade if the package or file is already present on the system", alpm-hooks(5), so a reinstall triggers it too), `When = PostTransaction`, it copies `/usr/share/limine/BOOTX64.EFI` over both ESP copies, and does nothing where `/boot/EFI/limine/` is missing (the live session). Why glue: the `limine` package ships no hook (upstream fact above), and this is the Arch wiki's own hook, extended to our two paths. The hook file sits beside the PKGBUILD, so `just arch packages` also stages files from the package's own folder (an Arch-only source has no place in `desktop/` or `branding/`).
- **`stackific-installer`'s `depends`** add `arch-install-scripts`, `efibootmgr` and `limine`. The live image lists `stackific-boot` explicitly, so removing the installer with `--recursive` keeps `limine` and `efibootmgr` on the installed system and drops `arch-install-scripts`.
- **The Done screen** (both the desktop and the console text):

  ```text
  ┌ Stackific OS installer ──────────────────────────────────────────────── Done ┐
  │                                                                              │
  │  Stackific OS is on /dev/nvme0n1.                                            │
  │                                                                              │
  │  Remove the USB stick, then restart the computer to start Stackific OS. If   │
  │  it starts something else, pick "Stackific OS" in its boot menu.             │
  ```

  (wrapped at 75 columns by the existing prose rule; the rest of the screen and its keys stay as they are).
- **The second boot of the install test:** `boot.sh` takes `-` for the ISO (no `-cdrom`), keeps the firmware's variables in `<out>/vars.fd` after the VM stops, and on a boot without an ISO waits for the serial line `BdsDxe: starting Boot.... "<NAME>"`, then saves `limine.png` (Limine's menu) a second later. `just arch test-install` boots the disk image again after the install boot, with `OVMF_VARS=<out>/vars.fd`, and sources the new `tests/vm/installed.sh`.

## Changes
- `installer/src/boot.rs` (new): the shared boot step: the EFI copies, `limine.conf`'s text, parsing efibootmgr's listing for stale entries, the delete and create commands; unit tests with a real listing as a fixture.
- `installer/fixtures/efibootmgr.txt` (new): one real `efibootmgr` listing from the test VM, with a stale and a current entry.
- `installer/src/base.rs`: Arch's initramfs command and kernel and initramfs file names behind `Base`; its test.
- `installer/src/install.rs`: step 8, "Install the boot loader"; the ESP's partition number from lsblk; its test.
- `installer/src/main.rs`: `mod boot;`.
- `installer/src/demo.rs`: fake outputs for the new commands, so `just installer demo` runs to Done.
- `installer/src/ui/done.rs`: the new text, and its module comment.
- `installer/fixtures/screens/installing.txt`, `installing-stop.txt`, `done.txt`, `done-console.txt`: the eighth step and the new Done text.
- `arch/packages/stackific-boot/PKGBUILD` (new) and `arch/packages/stackific-boot/stackific-limine.hook` (new).
- `arch/packages/stackific-installer/PKGBUILD`: `depends` add `arch-install-scripts`, `efibootmgr`, `limine`; `pkgrel` bump.
- `arch/iso/packages.x86_64`: add `stackific-boot`.
- `arch/justfile`: `packages` stages the files beside each PKGBUILD; `test-install` downloads the pinned `limine` package (`pacman -Sw` in the builder, whose mirror is the archive date), copies it onto the installed ESP with `mcopy`, and boots the disk image again with `tests/vm/installed.sh`; its comments.
- `tests/vm/boot.sh`: `-` for no ISO, `<out>/vars.fd` kept, `limine.png` on a disk boot; its header.
- `tests/vm/install.sh`: `installed_checks` also checks the ESP's files and `limine.conf`'s root UUID for each install.
- `tests/vm/installed.sh` (new): the checks on the booted installed system (Tests below).
- `release/justfile`: `changed` adds `tests/vm/installed.sh` to the `arch_x86_64` paths.
- `installer/src/system.rs`: only if the runtime keyboard check shows `us` in MangoWM or Plasma.
- `docs/installer.md`: the boot step, the firmware entry, `stackific-boot`, the second boot of the test; drop "does not start" and task 013 from "Later"; add to "Later": re-creating a lost firmware entry on updates, a Limine wallpaper in our look, an ESP free-space guard once several kernels share it.
- `docs/arch-iso.md`: `just arch packages` also stages files beside the PKGBUILD; `stackific-boot`.
- `site/src/content/docs/user/installation.md`: the installed drive starts; after the install, remove the USB stick and restart; the firmware's boot menu shows "Stackific OS"; drop the "cannot start yet" limit.
- `release/notes.md`: the installer's drive starts; drop "it is for a spare drive".
- `AGENTS.md`: one line under "Boot and disk" for the boot chain (Limine's two EFI copies, `limine.conf` at the ESP's root written by the installer, the firmware entry labelled `NAME` and first, `stackific-boot`'s hook, the kernel and initramfs through mkinitcpio's own hooks); the `just arch packages` staging rule under "Builds".
- `site/src/content/docs/prompt-trail/arch/013-limine-installed-arch.md` (new) and `site/src/assets/prompt-trail/013/*.webp` (new): Limine's menu, the Installing screen with eight steps, Done, and the installed system's login screen.
- `dev/reviews/013-limine-installed-arch.md` (new).

## Acceptance criteria

Must:
- `just installer test` and `just installer lint` pass; unit tests cover `limine.conf`'s text, the stale-entry rule on the fixture listing (only the entry on a missing partition is deleted), the create command, Arch's initramfs command, and step 8's place after the live-state step.
- `just installer demo` runs to Done and shows eight steps.
- `just arch packages` builds `stackific-boot`; `pacman -Qlp` of it lists exactly `/usr/share/libalpm/hooks/stackific-limine.hook` (plus its folders).
- `just arch iso` builds and prints the ISO size (task 012: 1543 MiB; target 2048 MiB); its manifest lists `stackific-boot`, `limine 12.9.6-1`, `efibootmgr 18-4` and `arch-install-scripts 32-1`. `just arch test` passes unchanged.
- `just arch test-install` passes, and in its second boot, from the disk image alone:
  - serial.log shows OVMF starting `Boot#### "Stackific OS"`, and `limine.png` shows Limine's menu with "Stackific OS" as its header and entry;
  - the system reaches `running`, and greetd's login screen shows on the installed system;
  - `efibootmgr` shows `BootCurrent` labelled "Stackific OS", first in `BootOrder`, and exactly one entry labelled "Stackific OS" after the erase and the free-space install on the same disk;
  - `/proc/cmdline` has `root=UUID=<UUID of />`, `rootflags=subvol=/@` and `rw`; `findmnt` shows `/` as btrfs with `subvol=/@` and `/boot` as vfat on the ESP; `btrfs subvolume get-default /` prints `ID 5 (FS_TREE)`;
  - `/boot` holds `vmlinuz-linux`, `initramfs-linux.img`, `limine.conf`, `EFI/limine/BOOTX64.EFI` and `EFI/BOOT/BOOTX64.EFI`, and no `*-fallback.img` and no `EFI/Linux/`; both EFI copies equal `/usr/share/limine/BOOTX64.EFI`; the initramfs has no `archiso` hook; `uname -r` names the one folder under `/usr/lib/modules/`;
  - after both EFI copies are emptied, `pacman -U` of the same `limine` package restores them, and the firmware entries are unchanged;
  - `stackific-boot`, `limine` and `efibootmgr` are installed; `stackific-installer` and `arch-install-scripts` are not; the hostname is `stackific-test`, the timezone `Europe/Berlin`, `localectl` shows the X11 layout and console keymap `de`, and `tester` is in `wheel`;
  - after the login screen's screenshot, the test logs `tester` in through greetd's `[initial_session]` (edited from the test shell, as the live test does): in MangoWM, `mmsg get keyboardlayout` as `tester` prints the German layout; in Plasma, `getLayoutsList` as `tester` names `de` first. If either shows `us`, the build fixes the installer's keyboard files and says so in the prompt trail.
- `just arch test-install`'s first boot still passes every task 012 check, and `installed_checks` finds the boot files and `limine.conf` with that install's root UUID for both the erase and the free-space install.
- The Done screen, `release/notes.md`, `site/src/content/docs/user/installation.md` and `docs/installer.md` no longer say the drive cannot start: `git grep -n -i -e "cannot start" -e "does not start" -e "spare drive" -- release site/src/content/docs/user docs installer/src` prints nothing.
- `just lefthook all` passes (shellcheck on the new and changed scripts and the PKGBUILD, `just dev check`, the site's checks).

Must not:
- use limine-mkinitcpio-hook, limine-entry-tool, limine-snapper-sync or any other AUR package;
- build a fallback initramfs or a unified kernel image, or enroll a config hash;
- write to any ESP but ours, or set the btrfs default subvolume;
- write NVRAM anywhere but in the installer's step 8, or change the order of the entries it did not create;
- add a boot entry or Limine to the aarch64 image (task 018) or touch Fedora (task 014);
- touch files outside the Changes list, or add a dependency this task does not name.

## Tests
- Unit, snapshot and demo: `just installer test`, `just installer lint`, `just installer demo`.
- Packages and image: `just arch packages`, `just arch iso` (size printed), `just arch test` on the PC.
- Install and first boot: `just arch test-install` on the PC; in CI, the `arch-x86_64` jobs of `images` and `release`.
- Messaging: the `git grep` above, then `just site build`.
- Screenshots for the prompt trail: `limine.png` and the installed system's login screen from `just arch test-install`; the Installing and Done screens from `just arch screenshots`; each through `just site screenshot`.
- By hand (operator), on a real computer, because firmware differs from OVMF:
  1. Download the test build's Arch x86_64 ISO after this PR merges (release `v0.0.0-pr.<n>`) and write it to a USB stick (`site/src/content/docs/user/installation.md`).
  2. On a computer with a spare drive and Secure Boot off, boot the stick, pick Install, and install onto the spare drive with "Erase".
  3. At Done, remove the stick and restart. Expect Limine's menu with "Stackific OS" for 3 seconds, then the login screen.
  4. Log in as the user you made, in MangoWM, then in Plasma; type a few letters that differ between your layout and US English (for German, `z` and `y`) in a terminal.
  5. Restart into the firmware's boot menu (the key differs per vendor, often F12, F8 or Esc). Expect one "Stackific OS" entry, first.
  6. Report the computer's make and model and any step that differed.
