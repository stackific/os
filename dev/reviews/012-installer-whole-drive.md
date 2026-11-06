# 012: The installer, into free space or onto a whole drive, from the live desktop or the boot menu, on the Arch x86_64 image: review

Task: dev/tasks/012-installer-whole-drive.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `installer/` (the crate, fixtures, launcher, units, justfile), `just/rust/`, the root `justfile`, `lefthook.yml`, `.claude/settings.json`, `.serena/project.yml`, `.gitignore`, `arch/` (PKGBUILDs, justfile, Containerfile, package list, boot entries), `desktop/noctalia/stackific-noctalia.toml`, the Fedora `stackific-desktop` spec, `tests/vm/`, both workflows, README, release notes, the site's user docs, `docs/`, `AGENTS.md`, the task file and the prompt-trail entry, against the task's acceptance criteria and `AGENTS.md`. Load assumed: one person installing on one machine.
Verdict: fix 1, 2 and 3 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | An erase left the partitions' starts to sfdisk, which aligns its own defaults to the drive's reported I/O size; on a drive with an odd optimal I/O size the lookup by start fails after the drive is wiped | `installer/src/disk.rs` `erase`, `installer/src/install.rs` | fixed | The erase script names `first-lba:` and every start and size in sectors from `disk::erase_layout`, in the drive's logical sectors (lsblk `LOG-SEC`, `Drive::sector`); `Plan` carries one `Layout` for both modes, from `disk::layout`, which Confirm shows too |
| 2 | Should fix | A password with a line break would split chpasswd's `name:password` input (the unattended file is a trust boundary) | `installer/src/answers.rs` `check_password` | fixed | Control characters are refused, with a test |
| 3 | Should fix | `drives::probe`'s refusals (BitLocker from `blkid -p`, no table, MBR) were tested only in the VM; the task's BitLocker drive fixture was missing | `installer/src/drives.rs` `probe` | fixed | `probe_reads_the_table_verify_and_bitlocker` over the fake runner, with `fixtures/blkid-bitlocker.txt` |
| 4 | Nice to have | The five list cursors were listed twice, in `tick` and `animating` | `installer/src/ui/mod.rs` | fixed | One `lists()` used by both |

Tests after the fixes: `just installer lint` passes; `just installer test`: 52 passed (with a `scan` test over the edge fixture, added while checking the criteria). `just arch iso` rebuilt the ISO (1543 MiB, with `ntfsprogs`); `just arch test` passed (28 checks); `just arch test-install` passed (86 checks), its erase now writing `first-lba: 2048`, `start=2048, size=8388608` and `start=8390656, size=125825024`, the same table as before. The site's format, lint, types, screenshots and build checks, shellcheck, actionlint, the justfile parse, `just dev check`, hygiene and gitleaks (file by file: lefthook only sees tracked files) pass on every changed and new file.

Before the review, during the build:

- `just rust install` installed Rust 1.99.1 with clippy, rustfmt and rust-analyzer into `.tools/rust/` from the checksum-checked rustup-init 1.29.3; `just rust check` prints `rust: ok (1.99.1 with clippy, rustfmt and rust-analyzer; rustup 1.29.3)`.
- The demo ran in a real pty (`script`), took scripted keys and exited 3 on Esc.
- `just arch packages` built `stackific-installer-0.1.0-1-x86_64.pkg.tar.zst` (634 KiB) with `check()` passing (50 tests then).
- The first `just arch iso` failed on a download timeout from archive.archlinux.org; the retry built a 1543 MiB ISO (1497 MiB before).
- `just arch test` failed on the Disks check: `ntfsresize` is in `ntfsprogs`, not `ntfs-3g` (the task's fact was outdated). `ntfsprogs` joined the package list and the task file.
- `just arch test-install` passed on that ISO, all 86 checks: the console path (`installer-console.png`), the refused erase, the erase, the Windows-like table, the BitLocker refusal, the free-space install with every old entry byte-identical, both installed systems' checks, and `minfo` reading the new ESP as FAT32.
- The package diff against main's ISO manifest adds only the five named packages and `ntfsprogs` with their dependencies (`rsync`, `xsettingsd`, `libntfs-3g`, `fuse2`, `libhandy`, `libpwquality`, `cracklib`, `libnotify`); `python` was already in the image.
- The operator chose to leave the Install boot entry out of the aarch64 image until task 018, because the Arch profile is shared.

## Round 2
Reviewed: the whole uncommitted change again, with round 1's fixes and this record: the explicit erase script and the shared `disk::layout` (Confirm, the screens' install and the unattended path), the password check, the new `probe` and `scan` tests, and `lists()`.
Verdict: Looks good. Ship.

## Round 3
Reviewed: the operator's follow-up (2026-11-06): the prompt-trail entry showed the screens as text drawings; it must show real screenshots, on the desktop and on the console. The change: `tests/vm/screens.sh`, `just arch screenshots`, `sendkey` taking several keys in `tests/vm/boot.sh`, the entry with 21 WebP screenshots, the task file, `docs/installer.md` and `docs/arch-iso.md`.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | The entry showed the screens as text, not as screenshots | `site/src/content/docs/prompt-trail/012-installer-whole-drive.md` | fixed | `just arch screenshots` drives the real installer in a VM (launcher, a free-space install, a terminal too small, a stopped erase, then the console's Install target and an erase); 21 screenshots replace the text; the fixtures stay for the snapshot tests |
| 2 | Should fix | On the VM's software renderer MangoWM left stale regions on screen after small redraws, so the first desktop shots were wrong though the installer drew them right (checked through run0's pty forwarder on the host, and on the console) | `tests/vm/screens.sh` | fixed | Before each desktop shot the script switches to workspace 2 and back (Ctrl+2, Ctrl+1), then waits until two dumps 2 s apart match; wlroots' `WLR_SCENE_DEBUG_DAMAGE=rerender` was tried and slowed the VM until keys arrived seconds late. The live-image issue is logged in `docs/arch-iso.md` for a later task |

Tests after the fixes: `just arch screenshots` passed and every shot shows the intended screen; `just site screenshots-check`, `format-check` and `build` (19 pages) and `just dev check` pass; shellcheck passes on `tests/vm/`.

## Round 4
Reviewed: round 3's change again: the shot wait and repaint, the recipe, the entry's images, alt texts and captions.
Verdict: Looks good. Ship.
