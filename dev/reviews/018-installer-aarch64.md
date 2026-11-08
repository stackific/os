# 018: The installer on both aarch64 images, with Limine's BOOTAA64.EFI and each base's aarch64 kernel: review

Task: dev/tasks/018-installer-aarch64.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (installer, stackific-boot on both bases, both images, tests, CI, docs), without the separate chore that restores `images` on pushes to `main`.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | QEMU's `virt` machine has no keyboard, so on aarch64 every `sendkey` goes nowhere: the GNOME login now in the shared `fedora.sh`, the installer's console keys, Limine's menu keys and the typed passphrase in `unlock.sh` | `tests/vm/boot.sh:37` | fixed | The aarch64 machine gets `-device qemu-xhci -device usb-kbd`, a USB keyboard that AAVMF, the initramfs and the desktop all read |
| 2 | Must fix | Arch Linux ARM's packages are `.pkg.tar.xz`, so the hook check's `limine-*.pkg.tar.zst` glob finds nothing on aarch64: `mcopy` fails in the recipe, and `pacman -U` in the installed system | `arch/justfile:121`, `tests/vm/installed.sh:114` | fixed | Both take `limine-*.pkg.tar.*`; the recipe's comment names where the package comes from on each architecture |

Tests after the fixes: `just lefthook all` passed (installer tests 66 passed, clippy, shellcheck, actionlint, site build and checks). `cargo clippy --all-targets --target aarch64-unknown-linux-gnu -- -D warnings` passed.

## Round 2
Reviewed: the fixes above and the whole change again (callers of `boot::loader`, `boot::EFI` and the new `base.rs` helpers, the demo's fake runner, the live-state file removal of a missing file, the check files now shared by both architectures, Fedora's `/proc/cmdline` check with `initrd=` in front).
Verdict: Looks good. Ship.

## Round 3
Reviewed: the x86_64 regression run (`just arch test-install`) after round 1's glob fix.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | `pacman -Sw` also saves `limine-*.pkg.tar.zst.sig`, which the wider `limine-*.pkg.tar.*` glob copies to the ESP and hands to `pacman -U`, which fails on it ("cannot open package file") | `arch/justfile:120` | fixed | The recipe deletes the downloaded signatures before `mcopy`; the installed system's `pacman -U` checks the package against its own keyring |

Tests after the fix: `just arch test-install` passed on x86_64, then `just fedora iso`, `just fedora test` and `just fedora test-install`.

## Round 4
Reviewed: round 3's one-line fix and the globs it serves (`arch/justfile`, `tests/vm/installed.sh`).
Verdict: Looks good. Ship.
