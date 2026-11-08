# 019: NVIDIA on Arch x86_64: open modules for linux and linux-lts, the driver picked at every boot, nouveau for older cards: review

Task: dev/tasks/019-nvidia-arch-x86-64.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (generator, its test and package, the image's package list and live files, the installer's two Limine entries, HOOKS without `kms` on x86_64 and the user config line, the VM checks, docs), after the ISO came within 2 GiB.
Verdict: Looks good. Ship. (Checked: every caller of `kernel_files` and `limine_conf`, the live-state removal of a missing file (`Real::remove` ignores NotFound), pacman's hook order (`90-mkinitcpio-install.hook` before `stackific-live-lts.hook`, confirmed in the build log), the generator on empty and audio-only PCI sets. A machine with a Pascal and a Turing card leaves the Pascal card without a driver while the Turing card draws: accepted, not a wrong result for the desktop.)

## Round 2
Reviewed: the change after the first `just arch test-install` run, which failed picking "Windows" in Limine's menu by a fixed number of key presses (the LTS entry moved it down): `tests/vm/installed.sh` now finds the Windows entries' place in `limine.conf` and first boots the LTS entry; the prompt-trail entry and docs.
Verdict: Looks good. Ship.

Tests after the fixes: `just hardware test`, `just installer lint`, `just installer test` (66), `just arch packages`, `just arch iso` (2041 MiB), `just arch test`, `just arch test-install` and `just lefthook all` pass.
