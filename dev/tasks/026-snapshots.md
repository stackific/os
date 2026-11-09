# 026: Snapshots with snapper and btrfs-assistant, a "Fresh install" snapshot, and zram

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Give every image and every install snapshots and restore: snapper with btrfs-assistant on both bases, in the live images and on installed systems, nothing more. The installer adds a flat @snapshots subvolume mounted at /.snapshots, a sibling of @ like the others (noatime,compress=zstd:1, fstab by UUID), so restoring @ keeps the snapshots. The installer writes snapper's root config itself (snapper's create-config refuses an existing /.snapshots) with ranged number limits and FREE_LIMIT, no quotas and no timeline, enables snapper's cleanup, and ends every install with a "Fresh install" snapshot taken after the boot setup. Package updates take pre and post snapshots: snap-pac on Arch, the dnf5 actions plugin on Fedora. Swap is zram, sized to RAM at priority 100, with zswap off; hibernation stays out. A system that no longer starts is restored from the live USB stick with btrfs-assistant: the user docs say how, including unlocking the encrypted disk first. Test what can run unattended in the install test; give the operator numbered steps for the restore from the live ISO.
```

## Goal
Installed systems have a flat `@snapshots` at `/.snapshots`, snapper's root config, a "Fresh install" snapshot, pre/post snapshots on every package transaction and zram swap; the live images carry snapper, btrfs-assistant and zram, so a broken install can be restored from the USB stick.

## Context
What exists (branch `task/021-hardware-parity-main` at 14a98e2, 2026-11-08):
- `installer/src/disk.rs` `SUBVOLUMES`: twelve flat subvolumes (`@`, `@home`, `@root`, `@srv`, `@cache`, `@tmp`, `@log`, `@swap`, `@flatpak`, `@libvirt`, `@containers`, `@docker`), created on the top level, mounted with `noatime,compress=zstd:1`, written to fstab by UUID; `@swap` and `@libvirt` NOCOW. No `@snapshots`.
- `installer/src/install.rs` `steps`: the eight steps; step 7 ("Install the boot loader") ends with Fedora's relabel (`Base::relabel`, all of the target) and then the firmware entry.
- No image carries snapper, btrfs-assistant, snap-pac or zram-generator (manifests in `.build/`, 2026-11-08). Fedora's image has no `zram-generator-defaults`.
- Package sources: `just arch packages` and `just fedora packages` stage `Source` files by name from the package's folder, `desktop/`, `branding/`, `hardware/` and `installer/`; `just release changed` lists those folders for every image.

Upstream facts:
- Arch Linux Archive 2026-11-01 (`arch/vars.just` `ala_date`; `extra.db`, read 2026-11-08): `snapper` 0.13.2-1 (depends `btrfs-progs libxml2 dbus boost-libs acl json-c`), `btrfs-assistant` 3.0.0-1 (Qt 6; optional `snapper`), `snap-pac` 3.0.1-3 (depends `snapper pacman btrfs-progs python`), `zram-generator` 1.2.1-1. Arch Linux ARM's live `extra` has the same four versions (`mirror.archlinuxarm.org/aarch64/extra/extra.db`, 2026-11-08).
- Fedora 45 (`mdapi.fedoraproject.org/f45`, 2026-11-08; no newer build in `f45-updates`): `snapper` 0.13.0-5 (with `snapper-libs`, which owns `/etc/sysconfig/snapper` and `/usr/share/snapper/config-templates/default`), `btrfs-assistant` 2.2-7 (owns `/etc/btrfs-assistant.conf`), `zram-generator` 1.2.1-6, `zram-generator-defaults` (only `/usr/lib/systemd/zram-generator.conf`, `zram-size = min(ram, 8192)`), `libdnf5-plugin-actions` 5.4.6.0 (reads `/etc/dnf/libdnf5-plugins/actions.d/*.actions`). `python3-dnf-plugin-snapper` is DNF 4's plugin; Fedora 45's dnf5 does not load it. No Fedora package ships a snapper actions file.
- dnf5's actions plugin (`doc/libdnf5_plugins/actions.8.rst`, tag 5.4.6.0, 2026-11-08) gives the actions that "emulate the DNF4 snapper plugin": a `pre_transaction` action storing `snapper create -t pre -p -d <command>` in `tmp.snapper_pre_number` and a `post_transaction` action running `snapper create -t post --pre-number …`. The option `enabled=host-only` skips an action for `--installroot` transactions, such as the installer's removal of GRUB.
- snapper lists its configs from `SNAPPER_CONFIGS` in `/etc/conf.d/snapper` (Arch) or `/etc/sysconfig/snapper` (Fedora), and reads each config from `/etc/snapper/configs/<name>`; `--no-dbus --root <dir>` (snapper(8)) works on another root, as openSUSE's installer does. Missing keys take snapper's defaults. Neither base enables `snapper-cleanup.timer` (Arch packages enable nothing; Fedora's presets live in `fedora-release`, which the images leave out).
- snapper's space-aware cleanup (`FREE_LIMIT`, `SPACE_LIMIT`) runs only with ranged number limits (`2-10`); `SPACE_LIMIT` needs btrfs quotas, `FREE_LIMIT` does not (snapper-configs(5)).
- btrfs-assistant (gitlab.com/btrfs-assistant/btrfs-assistant, tags 3.0.0 and 2.2, `src/core/Snapper.cpp` and `src/util/Snapper.cpp`, 2026-11-08): on a running system it maps a non-nested subvolume mounted at `/.snapshots` to the subvolume mounted at `/`, so a flat `@snapshots` works with no setting. From a live ISO nothing is mounted there, and it finds the target of a flat snapshot subvolume only through a line in `/etc/btrfs-assistant.conf`'s `[Subvol-Mapping]`, `root = "@snapshots,@,<btrfs UUID>"` (the file's own example); nested `.snapshots` are found without one. Both versions have `--list` and `--restore <index>` (3.0.0: `btrfs-assistant`, which runs them through its helper; 2.2: `btrfs-assistant-bin`, behind a pkexec launcher, as root); a restore snapshots the chosen snapshot to `@` and keeps the old `@` under a backup name.
- zram-generator (1.2.1) reads drop-ins from `/usr/lib/systemd/zram-generator.conf.d/*.conf` with or without a main file; keys `zram-size`, `compression-algorithm`, `swap-priority`.

## Inputs
From the operator's plan (row 026) and earlier decisions, restated:
- Snapshots: snapper with btrfs-assistant on both bases and in the live images, nothing more; a system that no longer boots is restored from the USB stick.
- The install ends with one snapper snapshot taken by our code after the boot setup: `snapper --no-dbus -c root create -t single -d "Fresh install" -c number -u important=yes`.
- Cleanup: `NUMBER_LIMIT="2-10"`, `NUMBER_LIMIT_IMPORTANT="4-10"`, `NUMBER_MIN_AGE="1800"`, `TIMELINE_CREATE="no"`, `FREE_LIMIT="0.2"`, `QGROUP=""`, `snapper-cleanup.timer` (hourly, snapper's default); no snapper config for `/home`. Why: on a small free-space install snapshots could fill btrfs and break updates; with ranges snapper deletes down towards the minimum once free space drops under 20 %, with no quota cost.
- zram: `zram-size = ram`, `swap-priority = 100`, the kernel's default compression (the earlier decision named zstd, but `linux-aarch64` 7.3.3-1 builds zram with LZO only, `CONFIG_ZRAM_BACKEND_FORCE_LZO=y` in `linux-aarch64-headers`' `.config`, read 2026-11-08); zswap off by a boot-time write; `vm.swappiness=150`, `vm.page-cluster=0`. Hibernation (a swap file in `@swap`) stays out.
- Operator decision 2026-11-08: a flat `@snapshots` at `/.snapshots`.

Decisions (operator 2026-11-09, the recommendations as built; the helper in 1 is logged under Later in `docs/installer.md`):
1. The restore from the live ISO needs one mapping line in the live session's `/etc/btrfs-assistant.conf` (the flat layout's cost); the user docs give it as one command. Alternative: a small live-session helper that writes it, which is glue we would own.
2. The snapshot is taken inside step 7, "Install the boot loader", so the Installing screen keeps its eight steps.
3. Our files ship as a new package `stackific-storage` (sources in `storage/`), not inside `stackific-desktop`, because snapshots and swap are not desktop defaults.

No files copied.

## Changes
- `installer/src/disk.rs`: `@snapshots` at `/.snapshots` in `SUBVOLUMES`, its root `chmod 750`; unit test.
- `installer/src/snapper.rs` (new): the root config text, the `SNAPPER_CONFIGS` line and the "Fresh install" command; unit tests.
- `installer/src/base.rs`: `Base::snapper_configs`, the base's `SNAPPER_CONFIGS` file.
- `installer/src/main.rs`, `installer/src/demo.rs`: the new module, and the demo's fake answers for `chmod` and `snapper`.
- `installer/src/install.rs`: step 7 writes the two snapper files before the last relabel and takes the snapshot after it, before the firmware entry; unit tests.
- `storage/` (new): `zram-generator.conf` drop-in, `sysctl` file, zswap tmpfiles line.
- `arch/packages/stackific-storage/PKGBUILD`, `fedora/packages/stackific-storage/stackific-storage.spec` and `fedora/packages/stackific-storage/snapper.actions` (new): the files above, the `snapper-cleanup.timer` link, the dependencies (`snapper`, `btrfs-assistant`, `zram-generator`; Arch `snap-pac`; Fedora `libdnf5-plugin-actions`).
- `arch/justfile`, `fedora/justfile`: stage `storage/`; `release/justfile` `changed`: `storage/` for every image.
- `arch/iso/packages.x86_64`, `arch/iso/packages.aarch64`, `fedora/iso/config.kiwi`: `stackific-storage`.
- `tests/vm/arch.sh`, `tests/vm/fedora.sh`: the packages, zram swap and the timer in the live session.
- `tests/vm/installed.sh`: `@snapshots` mounted, snapper's root config listed, the "Fresh install" snapshot, btrfs-assistant listing it, zram, the timer, the pre/post hook.
- `docs/installer.md`, `AGENTS.md`, `site/src/content/docs/user/snapshots.md` (new).
- `site/src/content/docs/prompt-trail/026-snapshots.md`, `dev/reviews/026-snapshots.md`.

## Acceptance criteria

Must:
- `just installer test` passes, with tests that `prepare` creates and mounts `@snapshots` at `/.snapshots` with mode 750, fstab has its line, and an install on both bases writes the snapper files before the last relabel and runs `snapper --no-dbus --root <target> -c root create -t single -d "Fresh install" -c number -u important=yes` after it and before `efibootmgr --create`.
- The live boot tests (`tests/vm/arch.sh`, `tests/vm/fedora.sh`) see `snapper`, `btrfs-assistant` and `stackific-storage` installed, `/dev/zram0` as swap at priority 100, zswap off and `snapper-cleanup.timer` enabled.
- The install tests (`tests/vm/installed.sh`) see `/.snapshots` mounted from `@snapshots`, `snapper list-configs` listing `root`, snapshot 1 "Fresh install" marked important, `btrfs-assistant --list` (Fedora: `btrfs-assistant-bin --list`) listing it for `@`, zram swap, the timer enabled, and snap-pac (Arch) or the actions file (Fedora).
- The user page tells how to undo an update from the running system and how to restore from the live USB stick on an encrypted disk.

Must not:
- Create a snapper config for `/home`, enable timeline snapshots or btrfs quotas.
- Add a swap file or hibernation.
- Touch files outside the Changes list.

## Tests
- `just installer test`, `just lefthook all`.
- `just arch test`, `just arch test-install`, `just fedora test`, `just fedora test-install` on both architectures in the `images` workflow.
- The restore from the live ISO, by the operator (no unattended flow boots the ISO again after an install):
  1. In a VM, install Stackific OS, start it, run `sudo touch /etc/restore-marker` and check `snapper list` shows "Fresh install".
  2. Boot the live ISO with the same disk. In a terminal: `sudo cryptsetup open /dev/<system partition> restore` with the login password.
  3. Run `echo "root = \"@snapshots,@,$(sudo blkid -s UUID -o value /dev/mapper/restore)\"" | sudo tee -a /etc/btrfs-assistant.conf`.
  4. Open Btrfs Assistant, Snapper tab, Restore mode: pick "Fresh install" and Restore (or `sudo btrfs-assistant --list`, then `sudo btrfs-assistant --restore <index>`; on Fedora `btrfs-assistant-bin`).
  5. Restart from the disk: `/etc/restore-marker` is gone and `sudo btrfs subvolume list /` shows the old `@` under its backup name.
