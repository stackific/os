# 026: Snapshots with snapper and btrfs-assistant, a "Fresh install" snapshot, and zram: review

Task: dev/tasks/026-snapshots.md
Status: commit ready

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
