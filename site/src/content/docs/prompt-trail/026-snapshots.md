---
title: Snapshots with snapper and btrfs-assistant, and zram
description: A flat @snapshots subvolume, snapper's root config and a "Fresh install" snapshot from the installer, pre and post snapshots around every package transaction, restore from the live USB stick, and swap in zram, on all four images.
sidebar:
  order: 26
  label: 026 Snapshots and zram
task: dev/tasks/026-snapshots.md
---

## Goal

Installed systems have a flat `@snapshots` at `/.snapshots`, snapper's root config, a "Fresh install" snapshot, pre and post snapshots on every package transaction and zram swap; the live images carry snapper, btrfs-assistant and zram, so a broken install can be restored from the USB stick.

## Context

The installer already made twelve flat btrfs subvolumes inside the LUKS2 container, and no image carried snapper, btrfs-assistant or any swap. This step adds the snapshot subvolume, snapper's configuration and first snapshot, the packages on every image, and zram.

## Expectations

Must:

- `just installer test` passes, with tests that `prepare` creates and mounts `@snapshots` at `/.snapshots` with mode 750, fstab has its line, and an install on both bases writes the snapper files before the last relabel and runs `snapper --no-dbus --root <target> -c root create -t single -d "Fresh install" -c number -u important=yes` after it and before `efibootmgr --create`.
- The live boot tests (`tests/vm/arch.sh`, `tests/vm/fedora.sh`) see `snapper`, `btrfs-assistant` and `stackific-storage` installed, `/dev/zram0` as swap at priority 100, zswap off and `snapper-cleanup.timer` enabled.
- The install tests (`tests/vm/installed.sh`) see `/.snapshots` mounted from `@snapshots`, `snapper list-configs` listing `root`, snapshot 1 "Fresh install" marked important, `btrfs-assistant --list` (Fedora: `btrfs-assistant-bin --list`) listing it for `@`, zram swap, the timer enabled, and snap-pac (Arch) or the actions file (Fedora).
- The user page tells how to undo an update from the running system and how to restore from the live USB stick on an encrypted disk.

Must not:

- Create a snapper config for `/home`, enable timeline snapshots or btrfs quotas.
- Add a swap file or hibernation.
- Touch files outside the Changes list.

## Prompt

```text title="Prompt" wrap
Give every image and every install snapshots and restore: snapper with btrfs-assistant on both bases, in the live images and on installed systems, nothing more. The installer adds a flat @snapshots subvolume mounted at /.snapshots, a sibling of @ like the others (noatime,compress=zstd:1, fstab by UUID), so restoring @ keeps the snapshots. The installer writes snapper's root config itself (snapper's create-config refuses an existing /.snapshots) with ranged number limits and FREE_LIMIT, no quotas and no timeline, enables snapper's cleanup, and ends every install with a "Fresh install" snapshot taken after the boot setup. Package updates take pre and post snapshots: snap-pac on Arch, the dnf5 actions plugin on Fedora. Swap is zram, sized to RAM at priority 100, with zswap off and the kernel's own compression (Arch Linux ARM's kernel has only LZO for zram); hibernation stays out. A system that no longer starts is restored from the live USB stick with btrfs-assistant: the user docs say how, including unlocking the encrypted disk first and the one mapping line btrfs-assistant needs to find a flat @snapshots from a live system. Test what can run unattended in the install test; give the operator numbered steps for the restore from the live ISO.
```

## Outcome

- **Installer.** `@snapshots` joins `SUBVOLUMES` at `/.snapshots`, mode 750 (snapper's own for `.snapshots`), so fstab and Fedora's relabel cover it. A new `snapper` module holds the root config (`2-10` snapshots, `4-10` important ones, `FREE_LIMIT="0.2"`, no quotas, no timeline) and the "Fresh install" command. Step 8 writes the config and `SNAPPER_CONFIGS="root"` (where each base's snapper reads it, `Base::snapper_configs`) before Fedora's relabel, then takes the snapshot with the live system's snapper (`--no-dbus --root <target>`), so it keeps the labels, before the firmware entry.
- **`stackific-storage`**, a new package on both bases with sources in `storage/`: it depends on snapper, btrfs-assistant and zram-generator, plus snap-pac on Arch and dnf5's actions plugin with our `snapper.actions` on Fedora (the actions dnf5's manual gives to emulate DNF 4's snapper plugin, with number cleanup and `enabled=host-only`, so the installer's `--installroot` removal takes none). It enables `snapper-cleanup.timer` and ships a zram drop-in (as large as the RAM, priority 100), the swap sysctls and zswap off.
- **Restore from the live ISO.** btrfs-assistant finds a flat `@snapshots` on the running system by itself, but from a live system only through a `[Subvol-Mapping]` line naming the file system's UUID; the user page gives it as one command after unlocking the disk.
- **Tests.** The live checks see the packages, the timer, zram and zswap off; the install checks see the mount, the config, snapshot 1, the pre and post snapshots around the limine (Arch) and kernel-core (Fedora) reinstalls the test already runs, and btrfs-assistant listing snapshot 1 for `@`.

### Screenshots

No UI.

## Review

- Round 1 found three things, all fixed: Arch's btrfs-assistant 3.0 has no `btrfs-assistant-bin` (its own `btrfs-assistant --list` is the command line), so the install check now uses whichever exists; `SPACE_LIMIT`, inert without quotas, left the config; the install test's header now names the new checks.
- Round 2 found that the task's Changes list missed `main.rs` and `demo.rs`; the task now names them.
- Round 3 found nothing.

## Manual changes

None.

## Result

- `just installer test` passes 67 tests, `just lefthook all` passes, and shellcheck passes on the changed test scripts.
- The images' boot and install tests run in the `images` workflow on both architectures; this step built no image.
- Docs: the new user page `user/snapshots.md` (what is kept, undoing an update, restoring from the USB stick on an encrypted disk, swap), `docs/installer.md` (the subvolume, snapper's files and the "Fresh install" snapshot in step 8) and `AGENTS.md`.

## Deviations

- zram uses each kernel's default compression instead of zstd: `linux-aarch64` builds zram with LZO only. The task says so.
