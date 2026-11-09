# 022: NVIDIA and regular hardware on both aarch64 images: review

Task: dev/tasks/022-aarch64-hardware.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (packages, `config.kiwi`, `just fedora packages`, the installer's hooks, the boot and install checks, the runner VM's memory, docs and entry).
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | kiwi's tmpfs, capped at half the RAM (8 GiB in the Mac's VM), would fill: the aarch64 build peaked at 6.6 GiB before NVIDIA, which adds about 1.2 GiB | `fedora/justfile` `iso` | fixed | `mount -t tmpfs -o size=75%` (12 GiB in the VM); `docs/fedora-iso.md`'s RAM paragraph says so |
| 2 | Should fix | The hardware notes still said the installer drops `kms` only on x86_64 | `docs/hardware.md` "Initramfs" | fixed | "on both architectures (aarch64 since task 022)" |
| 3 | Nice to have | Moved comments over 120 columns; "QEMU's std VGA" wrong on aarch64 | `tests/vm/arch.sh`, `tests/vm/fedora.sh` | fixed | Rewrapped; "QEMU's display" |

Tests after the fixes: `just lefthook all` passes (15 checks, after `just site format` on the new entry); shellcheck on `tests/vm/arch-aarch64.sh` passes; `just installer test` 66 passed.

## Round 2
Reviewed: the round 1 fixes and the whole change again.
Verdict: Looks good. Ship. (Checked: the tmpfs takes RAM only for what it holds, so 75% changes nothing on the PC; the moved checks run in the same order as before; `lsinitcpio` gets one file per call.)
