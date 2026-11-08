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

## Round 5
Reviewed: the aarch64 run on the Mac (source: `just installer test` on the Mac, an aarch64 host, before `just arch iso` and the rest in the runner VM).
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | `windows_entries_follow_the_system_on_both_bases` expects `vmlinuz-linux`, but Arch on aarch64 boots `Image`, so `cargo test` fails on aarch64: in `just installer test` and in the `check()`/`%check` of both installer package builds, which stops `just arch iso` and `just fedora packages` there | `installer/src/install.rs:708` | fixed | The expectation takes the kernel files from `Arch.kernel_files()`, as the other exact tests take the program name from the shared constant |

Tests after the fix: `just installer test` (66 passed) and `just installer lint` on the Mac (aarch64); `just lefthook all` passed.

## Round 6
Reviewed: round 5's fix and the other uses of `vmlinuz-linux` in the installer's tests (`boot.rs`'s `limine_conf` test takes it as plain input; `base.rs` tests `arch_kernel_files(false)`).
Verdict: Looks good. Ship.

## Round 7
Reviewed: the first aarch64 run in the Mac's runner VM (source: `just arch iso`, `test` and `test-install`, then `just fedora packages`, `iso`, `test` and `test-install`). Both ISOs built (Arch 1345 MiB, Fedora 1445 MiB) and both `test` recipes passed; both `test-install` runs stopped at the same check.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | `virt` has no IDE, so `-cdrom` attaches the ISO as a virtio disk ahead of the test disk: the ISO is `/dev/vda`, the blank disk `/dev/vdb`, and install.sh's "refused config leaves /dev/vda blank" check finds the ISO's table (`sfdisk --json /dev/vda` printed `0`, want `1`) | `tests/vm/boot.sh:131` | fixed | On aarch64 the ISO is a SCSI CD (`virtio-scsi-pci` with `scsi-cd`), `/dev/sr0` as on q35, so `/dev/vda` is the test disk again |

## Round 8
Reviewed: the second aarch64 run with round 7's fix (source: `just arch test` and `test-install`, `just fedora test` and `test-install` in the runner VM). Both `test` recipes passed, the live ISO now booting from the SCSI CD; Arch's `test-install` passed every check up to the typed passphrase, Fedora's stopped earlier.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | With no `console=`, Linux on arm64 makes the serial port in ACPI's SPCR table the console, so systemd asked for the passphrase on `ttyAMA0` (only `serial.log` showed it) while the screen stayed black; typed keys never reached the prompt, so "Enter on the boot-failure screen did not restart". Any SPCR machine (QEMU, UTM, many boards) could not unlock its installed system | `installer/src/boot.rs:54` | fixed | The command line gets `console=tty0` on aarch64 (`boot::CONSOLE`), on both bases; x86_64's does not change. The task gains the design line and a criterion; `AGENTS.md` and `docs/installer.md` say why |
| 2 | Must fix | install.sh's `limine.conf` check expects x86_64's `cmdline: rd.luks…`, but Fedora on aarch64 writes `cmdline: initrd=\initramfs-<v>.img rd.luks…` | `tests/vm/install.sh:132` | fixed | install.sh and installed.sh build the expected command line once, with `console=tty0` on aarch64 and the `initrd=` prefix for Fedora's aarch64 entry |

Round 7's finding is fixed by the same run: both live ISOs boot from the SCSI CD, and the installs see the test disk as `/dev/vda`.

Tests after the fixes: `just installer test` (66 passed) and `just installer lint` on the Mac (aarch64); shellcheck on the changed test scripts. In the runner VM (10 CPUs, 32 GiB), both bases in parallel: `just arch iso` (1345 MiB), `just arch test` and `just arch test-install` passed; `just fedora packages`, `just fedora iso` (1445 MiB), `just fedora test` and `just fedora test-install` passed, each install booted from its disk through `\EFI\limine\BOOTAA64.EFI` and unlocked by the typed passphrase.

## Round 9
Reviewed: rounds 7 and 8's fixes (`tests/vm/boot.sh`, `boot::CONSOLE` and its callers, Arch's `limine_conf` and Fedora's `/etc/kernel/cmdline`, the expected command lines in `install.sh` and `installed.sh`, `/proc/cmdline` with Fedora's `initrd=` prefix).
Verdict: Looks good. Ship.
