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

## Round 3
Reviewed: CI, `images` run 42546467105 at f682c01 (both aarch64 jobs).
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | Arch aarch64: mkarchiso stopped with `cannot resolve "libpisp", a dependency of "libcamera"`; libpisp is in ALARM's `[alarm]` repo, which the profile's `pacman.conf` lacks | `arch/iso/pacman.conf`, `arch/justfile` `iso` | fixed | `just arch iso` appends `[alarm]` to the copied profile's `pacman.conf` on aarch64 only (Arch's archive has no `alarm` repo) |
| 2 | Must fix | Fedora aarch64: kiwi stopped with `nothing provides libnvidia-rmapi-tegra.so.615.78.08()(64bit) needed by xorg-x11-drv-nvidia-libs-3:615.78.08-1.fc45.aarch64`; no package in RPM Fusion's release or updates repos ships it | `fedora/iso/config.kiwi` | fixed | The four NVIDIA lines stay `arch="x86_64"`, the `kmod-nvidia` build stays x86_64, the checks stay in `tests/vm/fedora-x86_64.sh`; docs say why, Later says how to undo |

Tests after the fixes: `just lefthook all` passes; shellcheck on the changed test scripts passes; the aarch64 builds rerun in the next `images` run.

## Round 4
Reviewed: the round 3 fixes.
Verdict: Looks good. Ship. (Checked: the `[alarm]` lines go only into the container's copy of the profile, only when `arch()` is aarch64; installed Arch Linux ARM systems keep ALARM's own `/etc/pacman.conf`, which lists `[alarm]`; `fedora-x86_64.sh` is the check file `boot.sh` picks on x86_64 again.)
