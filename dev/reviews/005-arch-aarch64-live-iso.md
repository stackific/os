# 005: Arch Linux ARM aarch64 console live ISO: review

Task: dev/tasks/005-arch-aarch64-live-iso.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `arch/vars.just`, `arch/Containerfile`, `arch/justfile`, `arch/iso/profiledef.sh`, `arch/iso/packages.aarch64`, `arch/iso/airootfs/etc/pacman.d/hooks/stackific-vmlinuz.hook`, the boot entry, `tests/vm/boot.sh`, `.github/workflows/images.yml`, `docs/arch-iso.md`, the README and AGENTS.md lines, and the prompt-trail entry. The agent-tool and runner-VM chores in the same tree were out of scope (their own commits).
Verdict: Ship. Fix 1 if cheap.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Nice to have | Called without the new third argument, `boot.sh` stops at `checks=$3` with `$3: unbound variable` (`set -u`) before its usage line can print | `tests/vm/boot.sh:14` | fixed | `checks=${3-}`; a missing argument now prints `usage: boot.sh <iso> <out-dir> <console\|desktop>` and exits 2 |

Not checked by the review: the x86_64 path (Containerfile branch, `-vga std` moved into the machine array, `label=disable`, the `*.pkg.tar.*` glob, the `desktop` argument); the runner VM is aarch64, so the task's manual `workflow_dispatch` run on the PC proves it.

Tests after the fixes, in the runner VM:
- `just shellcheck lint tests/vm/boot.sh`: passes.
- `boot.sh /x.iso /tmp` in the builder: prints the usage line, exit 2.
- `just arch test`: PASS (running, NetworkManager running, wrong token rejected, `uname -m`, `getty@tty1` active, `stackific login:` in `/dev/vcs1`, `screen.png`).

## Round 2
Reviewed: the same scope after the round-1 fix.
Verdict: Looks good. Ship.

## Round 3
Reviewed: the rebase onto `main` after tasks 003 and 004. It covered `tests/vm/boot.sh` (main's check-file design plus the aarch64 machine and firmware), the new `tests/vm/arch-aarch64.sh`, the `arch-aarch64` jobs in `images.yml` and `release.yml` with `publish` waiting for them, `arch/justfile` merged with 004's `branding/` mount, `docs/arch-iso.md` and `docs/releases.md`.
Verdict: Looks good. Ship.

Tests after the rebase, in the runner VM: `just arch iso` exits 0 (730 MiB; `stackific-desktop` and `stackific-release` build on aarch64). `just arch test` prints PASS, with main's `systemd-socket-activate` test shell on Arch Linux ARM. Shellcheck, actionlint, justfiles and `just dev check` pass on the Mac.
