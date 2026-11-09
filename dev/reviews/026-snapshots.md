# 026: Snapshots with snapper and btrfs-assistant, a "Fresh install" snapshot, and zram: review

Task: dev/tasks/026-snapshots.md
Status: open

## Round 1
Reviewed: the task's uncommitted change (installer, `stackific-storage` on both bases, package lists and recipes, boot and install checks, docs and entry).
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | btrfs-assistant 3.0.0 (Arch) installs no `btrfs-assistant-bin`; its CLI is `btrfs-assistant --list`, so the install check would fail on Arch | `tests/vm/installed.sh` | fixed | The check runs `btrfs-assistant-bin` where it exists (Fedora's 2.2), else `btrfs-assistant`; the task and entry name both |
| 2 | Should fix | `SPACE_LIMIT="0.5"` has no effect without quotas, which the config turns off | `installer/src/snapper.rs` | fixed | Line removed |
| 3 | Nice to have | `installed.sh`'s header did not list the new checks | `tests/vm/installed.sh:7` | fixed | "its snapshots and zram (task 026)" |

Tests after the fixes: `just installer test` 67 passed; `just lefthook all` passes (15 checks); shellcheck on the changed test scripts passes.

## Round 2
Reviewed: the round 1 fixes and the whole change again.
Verdict: fix 1.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The task's Changes list left out `installer/src/main.rs` (the module) and `installer/src/demo.rs` (the demo's fake answers), which the change touches | `dev/tasks/026-snapshots.md` | fixed | Both added to Changes |

Tests after the fixes: unchanged code; `just lefthook all` passes.

## Round 3
Reviewed: the whole change.
Verdict: Looks good. Ship. (Checked: snapper's files are written before Fedora's last relabel and the snapshot after it, so the snapshot and the files snapper creates in the relabelled `/.snapshots` keep their labels; snap-pac and the dnf5 actions do nothing during the install, because `SNAPPER_CONFIGS` is still empty while the installer removes packages and the actions are `enabled=host-only`; the mapping line from the user page lands under `[Subvol-Mapping]`, the last section of both versions' `/etc/btrfs-assistant.conf`; zram-generator 1.2.1 reads `/usr/lib/systemd/zram-generator.conf.d/`.)

## Round 4
Reviewed: CI, `images` run 42550000626 (fedora-aarch64 `just fedora test-install`).
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | The install test still expected twelve subvolumes: `btrfs subvolume list` printed `@snapshots` too, so the check failed after a correct install | `tests/vm/install.sh` | fixed | `@snapshots` in the expected list; 14 subvolumes in all (the thirteen flat ones and the "Fresh install" snapshot), exactly one inside `@snapshots`, 13 fstab lines; `docs/arch-iso.md` says thirteen |

Tests after the fix: shellcheck on `tests/vm/install.sh` passes; the four images rerun in the next `images` run.

## Round 5
Reviewed: CI, `images` run 42553225857 (arch-x86_64 `just arch test-install`).
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | The erase install stopped: removing `stackific-installer` with `pacman --sysroot` ran snap-pac's pre-transaction hook, which needs /proc (`FileNotFoundError: [Errno 2] No such file or directory: '/proc/1/root/.'`, `failed to commit transaction (failed to run transaction hooks)`) | `installer/src/base.rs` `live_state` | fixed | The removal runs as `env SNAP_PAC_SKIP=y pacman --sysroot=…`: snap-pac 3.0.1 checks `SNAP_PAC_SKIP` before anything else, and an install should take no pre and post snapshots before "Fresh install" anyway. Fedora needs nothing: its dnf5 actions are `enabled=host-only`, so `--installroot` takes none |

Tests after the fix: `just installer test` passes 67 tests, `just installer lint` is clean; the four images rerun in the next `images` run.

## Round 6
Reviewed: CI, `images` run 42559637348 (fedora-aarch64 `just fedora test-install`; arch-aarch64 passed).
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | On installed Fedora, `dnf5 reinstall kernel-core` made no pre and post snapshots (`snapper … list --columns type` counted 0 `pre`/`post`) | `fedora/packages/stackific-storage/snapper.actions` | open | Not the actions file: in the Fedora builder, dnf5 5.4.7's actions plugin calls a stand-in `snapper` with `create -t pre -p -c number -d …` and `create -t post --pre-number 42 …`. Not SELinux: Fedora 45's policy makes `rpm_t` `dbusd_unconfined`, and `snapperd_t` may answer it. The install test now prints dnf5's log lines for the actions, snapperd's journal and the actions folder before the check, so the next run shows the real error |
Second CI round on it (run 42567485506, fedora-x86_64 too): dnf5 loaded the actions plugin, `snapper.actions` is in place, and `snapperd` started at the transaction's second, so the action called `snapper create` and the daemon made no snapshot. Fedora's policy labels `/.snapshots` and below `snapperd_data_t`, which `snapperd_t` may write. The test now prints the labels under `/.snapshots`, the boot's AVC denials, and a direct `snapper create` with its exit code.
