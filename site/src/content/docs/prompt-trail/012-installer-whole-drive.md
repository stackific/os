---
title: The installer, into free space or onto a whole drive, from the live desktop or the boot menu, on the Arch x86_64 image
description: A Rust installer with a ratatui text interface puts Stackific OS into a drive's free space or onto a whole drive, from a launcher, the boot menu or a TOML file, with an install test in a VM.
sidebar:
  order: 12
  label: 012 The installer
pr: https://github.com/stackific/os/pull/12
task: dev/tasks/012-installer-whole-drive.md
---

## Goal

`stackific-installer`, a Rust program with a ratatui text interface, installs Stackific OS from the Arch x86_64 live session into a drive's free space or onto a whole drive: interactively, from an "Install Stackific OS" launcher or boot-menu entry, or unattended from a TOML file. Its first screen, Display, makes the text readable. `just arch test-install` proves both modes on a 64 GiB disk image. The installed system does not boot yet: Limine comes in task 013.

## Context

Before this step the four images were live previews with no installer and no Rust in the repo. This step adds the installer's crate with its shared disk, copy and system steps and an Arch `Base`, ships it with GNOME Disks on the Arch x86_64 image, adds the Rust toolchain to `just setup`, and trims the README, the release notes and the user docs so they name the desktop MangoWM and carry only the non-affiliation sentence.

## Expectations

Must:

- `just installer lint` and `just installer test` pass. The tests cover:
  - the sfdisk scripts for both modes, the fstab text and the subvolume list;
  - the gap search on `sfdisk --json` fixtures: alignment, the largest of several gaps, a gap under 40 GiB, a full disk, a numbering hole;
  - the free-space refusals: no table, an MBR table, `--verify` output with a problem, a hybrid MBR, a BitLocker partition;
  - answer validation (good and bad user names, computer names, passwords, keyboard codes, timezones, modes, the typed confirmation);
  - drive eligibility from `lsblk --json` fixtures (live medium, read-only, under 40 GiB, mounted, zram, `rom` and `loop` devices);
  - the search order ("germ" gives `de`, `at`, `ch` on a fixture list; an exact code comes first; an empty search starts with `us` or `UTC`) and that each typed character re-filters;
  - the `kbd-model-map` lookup (`de` gives `de`, an unknown layout gives `us`), the greetd edit (no `[initial_session]`, `[default_session]` unchanged), the MangoWM file, and a log that never holds the password;
  - navigation: Esc back from Drive to You keeps every typed value; "Enter next" is absent while a field is invalid; Ctrl+C before the install exits without writing; Ctrl+C during the install shows the stop dialog, Esc closes it and the install goes on;
  - the text size pick (3840 × 2160 gives Large, 2560 × 1600 Large, 1920 × 1080 Medium, 1366 × 768 Small, 5120 × 2880 Extra large) and the sizes offered (on 1024 × 768, neither 16 × 32 nor 24 × 32, which leave 64 × 24 and 42 × 24 characters);
  - console mode: Esc on Display or Welcome exits 3 without writing; ←→ on Text size calls `setfont` through the fake `Runner` with each offered font;
  - Display: Contrast changes the rendered buffer at once; with High contrast, no cell is dim;
  - the drive list is read again when the Drive screen shows and on a focus-gained event;
  - the demo: `--demo` walks every screen and a simulated install with the fake `Runner`, which fails the test on any command it was not told to expect;
  - a `TestBackend` snapshot of each of the seventeen screens above at 80 × 24 (the console ones with `TERM=linux`, the small one at 60 × 18), matching its fixture; the same Keyboard screen at 200 × 60, with the frame at the centered position; with `NO_COLOR` set, no cell has a colour; without it, the selected row has the accent and reverse video.
- `just arch packages` builds `stackific-installer-0.1.0-1-x86_64.pkg.tar.zst` with `check()` passing; `just arch iso` builds an ISO that contains it, and its size is reported in the task's PR (under 2 GiB).
- `just arch test` passes, including the new launcher check.
- The ISO's `loader/entries/` (read from the ISO with `bsdtar`, never a mount) has `01-stackific-linux.conf` titled `Try Stackific OS (x86_64, UEFI)` and `02-stackific-install.conf` titled `Install Stackific OS (x86_64, UEFI)` with `systemd.unit=stackific-installer.target`, and `loader.conf` still defaults to the first.
- `just arch test-install` passes, in one boot of the ISO with one 64 GiB disk, and its log shows each of these: 0. The console path: `systemctl isolate stackific-installer.target` leaves `stackific-installer-console.service` active and the greeter stopped, a screenshot `installer-console.png` of tty1 is saved, and after `sendkey esc` twice the service has ended and `graphical.target` is active again.
  1. Wrong confirmation: an erase whose `erase` is not the device's name exits 2 and leaves `/dev/vda` blank (`sfdisk --json /dev/vda` finds no partition table).
  2. Erase: the run, started with `systemd-run --unit=<name>` and polled until it ends, ends with `Result=success` (on failure, the check prints the unit's journal). `sfdisk --json /dev/vda` shows a `gpt` table with exactly two partitions: 1 starts at sector 2048, has 8388608 sectors, the EFI System type and the name `Stackific OS boot`; 2 fills the rest, with the Linux filesystem type and the name `Stackific OS`. Then the installed-system checks below.
  3. A Windows-like table, made in the guest on `/dev/vda` with sfdisk: an EFI System partition of 100 MiB (FAT, label `SYSTEM`), a Microsoft reserved one of 16 MiB, a 2 GiB gap, a Microsoft basic data one of 10 GiB, a gap of at least 40 GiB, and a Windows recovery one of 1 GiB at the end, numbered 1, 2, 3 and 5 (a hole at 4). Its `sfdisk --dump` is saved.
  4. BitLocker refusal: with `-FVE-FS-` written at byte 3 of partition 3, after the boot code's jump `EB 52 90` (libblkid needs both when the volume has no FVE metadata; checked at build), `blkid -p` reports `BitLocker`, and a free-space run exits 2 with the dump unchanged. The signature is then cleared.
  5. Free space: the run ends with `Result=success`. In the new `sfdisk --dump`, the label ID and every line of the four old partitions are byte-identical to the saved dump; exactly two new partitions lie inside the large gap, both starting on a multiple of 2048, the first with 8388608 sectors, the EFI System type and the name `Stackific OS boot`, the second reaching the gap's aligned end with the Linux filesystem type and the name `Stackific OS`. The old ESP keeps its UUID and its label `SYSTEM`. Then the installed-system checks below.
  6. After the VM powers off, `minfo -i <image>@@<recorded offset>` reports a FAT32 file system.

  The installed-system checks, after each of runs 2 and 5:
  - `blkid`: the new ESP is `vfat` with label `STACKIFIC`; the new system partition is `btrfs` with label `stackific`.
  - On the system partition mounted with `subvolid=5`: `btrfs subvolume list` names exactly the twelve SN-01 subvolumes, all top level, and `lsattr -d` shows `C` on `@libvirt` and `@swap`.
  - In `@`: `etc/fstab` has the twelve subvolume lines and the `/boot` line by UUID with the options above; `etc/os-release` has `ID=stackific`; `var/lib/pacman/local/` has `stackific-release-*` but no `stackific-installer-*`, and `usr/share/applications/stackific-installer.desktop` is absent; `etc/machine-id` reads `uninitialized`.
  - The user: `etc/passwd` has `tester:x:1000:1000:Test User:/home/tester:/usr/bin/bash`, `etc/group` puts `tester` in `wheel`, `etc/shadow` has a `$y$` hash for `tester` and a locked root (`!`); `@home` has `tester/.config/noctalia/00-stackific.toml` from `/etc/skel` and `tester/.config/mango/config.conf` with the two lines above, owned by uid 1000.
  - The system: `etc/hostname` is `stackific-test`; `etc/localtime` links to `../usr/share/zoneinfo/Europe/Berlin`; `etc/vconsole.conf` has `KEYMAP=de`; `etc/X11/xorg.conf.d/00-keyboard.conf` has `XkbLayout` `de`; `etc/locale.conf` has `LANG=C.UTF-8`.
  - The live state is gone: no `live` in `etc/passwd`, `etc/shadow` or `etc/group`; no `live` in `@home`; no `etc/sysusers.d/stackific-live.conf` and no `etc/mkinitcpio.conf.d/archiso.conf`; `etc/greetd/config.toml` has no `initial_session` and keeps `default_session` with `/usr/bin/noctalia-greeter-session`.

- `just setup` installs Rust 1.99.1 with clippy, rustfmt and rust-analyzer into `.tools/` from the checksum-checked rustup 1.29.3, on Linux x86_64, Linux aarch64 and macOS on Apple silicon, and `just doctor` prints a `rust:` line with that version. `just lefthook all` passes, so the `check` workflow passes too.
- `git grep -n -i noctalia -- README.md release/notes.md site/src/content/docs/index.md site/src/content/docs/user` prints only `wallpapers.md` lines that name a `noctalia` path. `README.md` and `release/notes.md` contain no `Built from`, no `## Trademarks`, no `Fedora Remix` and no `Red Hat, Inc.`, and each ends with the non-affiliation sentence inside `<sub>`; `git grep -n -i fedora -- README.md release/notes.md` prints only ISO file names and that sentence. `just site build` passes.

Should (the design table's soft rows; drop one that is close to impossible and record which and why in `docs/installer.md` and the review record):

- motion: a spring with damping 1.0 reaches its target without overshoot; a retarget keeps position and velocity continuous; Reduced motion reaches the target in one frame; screens slide forward from the right and back to the right; the stop dialog grows from and returns to its bottom-border hint;
- the rubber band past a list's edge, and a held key speeding up after half a second;
- the mouse in foot: a press on a row highlights it without selecting; the release selects it; moving off before the release cancels; a click on an action in the bottom border acts like its key; a wheel flick lands on the row the guide's projection gives;
- materials in a true-colour terminal: the frame's tint, the dialog's scrim and shadow, and the faded edges of a scrolled list; none of them with High contrast or `NO_COLOR`;
- Motion: Full and Reduced change the rendered motion at once;
- the single bell on the Stopped screen;
- `just installer demo --slow`.

Must not:

- write to any drive before the confirmation, to any drive other than the chosen one, or, in free-space mode, outside the chosen gap;
- change, move or renumber an existing partition, or write to another OS's ESP;
- edit MangoWM's stock files under `/etc/mango/`, or ship a MangoWM config in a package;
- add `sudo`, a `NOPASSWD` rule or a polkit rule that skips authentication (the user administers through `wheel`, with `run0` or polkit, as `live` does);
- bring Python into the image, or add a crate beyond the four named;
- add a language screen, change the system locale, or add fonts or locale packages;
- change the display scale of the live or installed desktops (its own task, with a decision table);
- add GParted, KDE Partition Manager or `xorg-xhost`;
- change the Fedora images, `arch/iso/packages.aarch64` or published releases (the Fedora `stackific-desktop` changes only if the shared Noctalia defaults do);
- touch files outside the Changes list.

## Prompt

```text title="Prompt" wrap
Write the installer skeleton and its disk step, and ship it on the Arch x86_64 image: our own 4 GiB FAT32 EFI System Partition and one partition for the OS, created either in a drive's free space (next to what is there) or on a whole drive, the btrfs subvolumes, a copy of the live image's root, and the user. One program serves both bases from the start: a shared disk step, with per-base steps behind one interface. The Fedora images reuse the same Rust program later; this PR puts it on the Arch x86_64 image only. Write it in Rust with a ratatui text interface. The user must be able to type in, and options must appear, for the timezone and the keyboard. Create an empty @swap now, because zram plus hibernation for laptops comes later. For the free space, refuse what is not safe yet (BitLocker, no GPT table, a table that needs repair); the BitLocker confirmation, OS detection and the Windows boot entry come later. Add a launcher, so the installer opens from the live desktop. The installer must also run from the live ISO itself, when the user picks Install instead of Try at boot. The user must be able to read the installer on any screen, so the wizard starts with a text size that lets them continue (automatic, and theirs to change); scaling the installed desktop comes later, as options after the install. Adapt the Apple design guide (a skill text from WWDC talks, written for the web) to these screens as far as possible; its rules are soft requirements, so skip what is close to impossible in a terminal. Add a partition editor to the live desktop's launcher, the one most compatible with our choices, so anyone who wants to set up their disk first can do it there. It has to work with KDE Plasma as well as MangoWM. Add an unattended install test. Add Rust tooling to just. Add ASCII screenshots of the screens to the task. In the prompt trail, show real screenshots of the screens, on the desktop and on the console. On Plasma's desktop, add a shortcut to the installer, and make Plasma show our wallpaper. In MangoWM, Super+Space opens Noctalia's launcher, as on the operator's own desktop. Adopt the guide's principles, as soft requirements, as fully as a terminal reasonably allows: response, direct manipulation, interruptible spring motion, spatial consistency, rubber-banding, materials and depth, feedback, reduced motion and contrast, typography, and its eight design principles. It must look right on an ultrawide monitor, a 13-inch laptop and everything between, so the screens have a fixed maximum size. Also, in the same PR: on the README, drop the "Built from" column from the image table, drop the "## Trademarks" heading and turn its three sentences into small print with <sub>. In the release notes, replace the "Built from" column with "First desktop" (MangoWM) and turn the same sentences into small print. Reduce the mentions of Noctalia in what users read, except in the prompt trail. Once the base column is gone, the README and the release notes name no Fedora mark (the ISO file names do not count), so they carry no Fedora notice and no Red Hat trademark line, only the non-affiliation sentence in small print; AGENTS.md says so. On the Arch aarch64 image, which shares the boot menu files, leave the Install entry out until the installer joins that image. Disks needs ntfsprogs for ntfsresize.
```

## Outcome

`installer/` holds one Rust crate, `stackific-installer` (edition 2024, Rust 1.99), with four dependencies: ratatui (its crossterm backend), serde, serde_json and toml. One module per concern: os-release names and the accent, the answers and their checks, the drives (`lsblk`, `sfdisk --json`/`--verify`/`--label-nested dos`, `blkid -p`, the largest 1 MiB-aligned gap, every refusal with its reason), the disk step (both sfdisk scripts at explicit sectors, mkfs, the twelve subvolumes, the mounts, fstab), the copy (rsync and its percentage), the system step (keyboard, timezone, computer name, the user, the live state), the `Base` trait with `Arch`, and the install that runs them in order. Every command and file goes through a `Runner`: the real one, or a fake that the tests and `just installer demo` use, which records any command it was not told to expect.

The screens are ratatui on crossterm in a fixed 80 × 24 frame, centered: Display, Welcome, the six steps, Installing, and Done or Stopped. One spring model moves the slides, the selection marker, the rubber band and the progress bar; Reduced motion makes them instant. The mouse works in foot (press, release, cancel by moving off, the wheel with a projected flick); true-colour terminals get a tinted surface, a scrim and shadow behind the stop dialog, and faded list edges; High contrast and `NO_COLOR` drop all of it. Every spacing, colour and timing is a named constant in `ui/theme.rs`.

On the Arch x86_64 image, `stackific-installer` installs the program, the "Install Stackific OS" launcher (foot at physical size, `run0`), and the target and service behind the boot menu's new "Install Stackific OS" entry, which runs the installer on tty1 and hands over to the live desktop when it ends. GNOME Disks, `ntfs-3g`, `ntfsprogs`, `breeze-gtk` and `kde-gtk-config` join the image, and Noctalia's own polkit agent is on, so Disks asks for the password in MangoWM too. The installer removes its own package from the system it installs.

`just setup` installs Rust 1.99.1 with clippy, rustfmt and rust-analyzer through checksum-checked rustup 1.29.3; `just installer test` and `lint` run in pre-commit; `just arch test-install` installs twice in a VM, unattended, and checks each result on its partitions.

The README and the release notes name MangoWM as the first desktop and end with the non-affiliation sentence in small print; the user docs say "MangoWM", and the installation page names the early installer and Disks.

### Screenshots

Taken from the image in a VM by `just arch screenshots`, which drives the real installer with key presses on a 64 GiB disk that starts with a Windows-like table.

On the desktop, from Noctalia's launcher (Super+Space):

![Noctalia's launcher listing Install Stackific OS and Disks](../../../assets/prompt-trail/012/launcher.webp)

_Super+Space opens Noctalia's launcher, which lists the installer and Disks, the partition editor._

![The Display screen in foot, centered in the full-screen window](../../../assets/prompt-trail/012/display.webp)

_Display comes first. Motion and Contrast change the screen at once; Text size names foot's own keys. The red around the frame is run0's tint for a root session._

![The Welcome screen](../../../assets/prompt-trail/012/welcome.webp)

_Welcome says what happens and that nothing is written before the last screen._

![The Keyboard screen after typing "germ", with German first](../../../assets/prompt-trail/012/keyboard.webp)

_Typing filters the layouts by code and name; the exact match comes first._

![The Timezone screen after typing "berl", with Europe/Berlin](../../../assets/prompt-trail/012/timezone.webp)

_The same list, for the timezones._

![The You screen with the confirmation one character short](../../../assets/prompt-trail/012/you.webp)

_The user name follows the first word of the name; the first problem shows under the fields as you type._

![The Drive screen listing the VM's 64 GiB disk with 50.9 GiB of free space](../../../assets/prompt-trail/012/drive.webp)

_Whole disks with their largest free space._

![The Space screen offering the 50.9 GiB of free space after vda3](../../../assets/prompt-trail/012/space.webp)

_Free space first, with what is on the drive now._

![The Confirm screen for a free-space install](../../../assets/prompt-trail/012/confirm-free-space.webp)

_What stays, what is new, and the answers. Enter installs._

![The Confirm screen for an erase, with vda typed](../../../assets/prompt-trail/012/confirm-erase.webp)

_An erase needs the drive's name typed first._

![The Installing screen during the copy, with the progress bar](../../../assets/prompt-trail/012/installing.webp)

_One line per step, and the copy's progress._

![The stop dialog over the dimmed Installing screen](../../../assets/prompt-trail/012/installing-stop.webp)

_Ctrl+C asks first; Esc keeps installing._

![The Done screen after the free-space install](../../../assets/prompt-trail/012/done.webp)

_Done, and why the drive does not start yet._

![The installer's message in a terminal smaller than 80 × 24](../../../assets/prompt-trail/012/small.webp)

_Text made too big for the window: the installer says what it needs._

![The Space screen with the free space refused and only the erase offered](../../../assets/prompt-trail/012/space-refused.webp)

_Free space under 40 GiB is refused with its reason; the erase stays._

![The Stopped screen after the user stopped an erase](../../../assets/prompt-trail/012/stopped.webp)

_Stopped says why, what is left on the drive, and where the log is._

Plasma, from `just arch test`: our wallpaper and the installer's shortcut:

![Plasma's desktop on Dahl's An Eruption of Vesuvius, with the Install Stackific OS shortcut at the top left](../../../assets/prompt-trail/012/plasma-shortcut.webp)

_Plasma starts on our wallpaper through our look-and-feel package, and shows the installer on its desktop; the italic label marks a link to the launcher._

On the console, from the boot menu's Install entry:

![The Display screen on the console, picked for the VM's 1280 × 800 screen](../../../assets/prompt-trail/012/display-console.webp)

_The boot menu's Install entry: the installer on tty1, with no greeter. Text size sets the console font._

![The Keyboard screen on the console after typing "germ"](../../../assets/prompt-trail/012/keyboard-console.webp)

_On the console, the markers are ASCII and the arrows are words._

![The Confirm screen for an erase on the console](../../../assets/prompt-trail/012/confirm-console.webp)

_The same summary and typed confirmation on the console._

![The Installing screen on the console, the bar in half cells](../../../assets/prompt-trail/012/installing-console.webp)

_The console's fonts have half blocks only, so the bar moves in halves._

![The Done screen on the console, with Enter opening the live desktop](../../../assets/prompt-trail/012/done-console.webp)

_When the installer ends on the console, the live desktop starts._

## Review

Round 1 found three things to fix and one to tidy, all fixed:

- An erase left the partitions' starts to sfdisk, which aligns its own defaults to a drive's reported I/O size; on a drive that reports an odd one, the lookup of the new partitions would fail after the wipe. The erase script now names `first-lba:` and every start and size, in the drive's logical sectors, from the same layout the Confirm screen shows.
- A password with a line break would have split chpasswd's `name:password` input; control characters are now refused.
- The drive probe's refusals (BitLocker from `blkid -p`, no table, MBR) gained a test over the fake runner, with the task's BitLocker fixture.
- The five list cursors, listed twice, now come from one place.

Round 2 found nothing: looks good, ship.

Round 3 answered the operator's follow-up: real screenshots in place of the text drawings. Taking them showed that MangoWM on the VM's software renderer can leave stale regions on screen after small redraws, so the screenshot script switches workspace away and back before each desktop shot; the issue is logged for a later task. Round 4 found nothing. Round 5 added the operator's Plasma desktop shortcut, and moved the x86_64-only checks (launcher, Disks, GTK integration, the shortcut) from `tests/vm/arch.sh`, which aarch64 runs too, to `tests/vm/arch-x86_64.sh`. Round 6 found nothing. Round 7 answered three more: Plasma now starts on our wallpaper (our look-and-feel package), Super+Space opens Noctalia's launcher in MangoWM, and the screenshots open the installer from that launcher instead of rofi. Round 8 found nothing.

## Manual changes

None. The operator picked how to treat the aarch64 image's boot menu (below); the steps for trying the screens by hand are in the task's Tests.

## Result

- `just rust install` puts Rust 1.99.1 with clippy, rustfmt and rust-analyzer in `.tools/rust/` from checksum-checked rustup 1.29.3; `just doctor` prints `rust: ok (1.99.1 with clippy, rustfmt and rust-analyzer; rustup 1.29.3)`.
- `just installer lint` passes; `just installer test` passes 52 tests, among them the seventeen screen snapshots, the 200 × 60 frame, `NO_COLOR`, High contrast, navigation, the console path, the mouse, motion and the demo walked through both kinds of install.
- `just arch packages` builds `stackific-installer-0.1.0-1-x86_64.pkg.tar.zst` (634 KiB) with `check()` passing; `just arch iso` builds a 1543 MiB ISO (1497 MiB before), under the 2048 MiB target.
- `just arch test` passes, with the launcher, Disks, `ntfsresize` and Plasma's GTK integration checks.
- `just arch test-install` passes, 86 checks: the boot menu read with bsdtar (Try the default, Install with `systemd.unit=stackific-installer.target`), the console path, a refused erase, an erase, the Windows-like table, the BitLocker refusal, a free-space install with every old entry byte-identical, both installed systems, and `minfo` reading the new ESP as FAT32.
- `git grep -n -i noctalia` over the README, the release notes and the user docs prints only the two `wallpapers.md` lines with `noctalia` paths; the README and the release notes have no `Built from`, `## Trademarks`, `Fedora Remix` or `Red Hat, Inc.`, and end with the non-affiliation sentence in `<sub>`. `just site build` builds 19 pages.
- `just arch screenshots` passes and gives the 21 screenshots above; `just arch test` gives Plasma's (all 22 together about 0.5 MB as WebP).
- Docs: `docs/installer.md` (new), `docs/arch-iso.md`, `docs/checks.md`, `docs/setup.md`; user docs: `installation.md`, `wallpapers.md` and the docs home.

## Deviations

Each of these is in the task file too.

- The Arch profile in `arch/iso/` serves both architectures, so the Install boot entry would have reached the aarch64 image, which has no installer yet. The operator chose to leave it out there: `just arch iso` copies the profile and drops `02-stackific-install.conf` on aarch64 until task 018.
- `ntfsresize` now ships in `ntfsprogs`, not `ntfs-3g`, so `ntfsprogs` joined the package list (the boot test caught it).
- The builder gained `qemu-img` for the install test's disk image.
- The crate reaches makepkg as one tar (`stackific-installer-crate.tar`, checksum `SKIP`), with `fixtures/` for `check()`, because makepkg takes no directories as sources.
- rsync also gets `--no-inc-recursive`, so its percentage counts the whole copy, and `--exclude=/boot/`, because `/boot` is empty in the live root and is the FAT ESP in the target.
- `userdel live` runs at the start of "Create your user", before `useradd`, so the new user gets uid 1000.
- After review round 1, the erase script names `first-lba:` and every start and size, and a password with a control character is refused.
- The console service counts exits 3 (left) and 130 (stopped) as success, so a choice does not mark the system degraded.
- libblkid reports BitLocker on a volume without metadata only with the boot code's jump (`EB 52 90`) before `-FVE-FS-`, so the test writes both.
- Two drawings changed with their fixtures: at 52 % the bar's partial cell is `▏` (15.08 of 29 cells), and the Space hint breaks after "no GPT". Prose wraps at 75 columns, the width the drawings use.
- Corrected facts: `python` was already in the image (this change adds none), and evdev.lst lists 99 layouts.
- The release notes say the early installer is on the `-arch-x86_64.iso` only, and `docs/setup.md` names the installer recipes, as `AGENTS.md` asks of a setup change.
- The boot menu check from the ISO (bsdtar) runs in `just arch test-install`.
- The entry shows real screenshots from `just arch screenshots` instead of the screens as text, at the operator's request; the text fixtures stay for the snapshot tests.
- Plasma's desktop shows a shortcut to the installer (a link in `live`'s `~/Desktop`), at the operator's request; aarch64 leaves it out with the Install entry. The x86_64-only checks moved to `tests/vm/arch-x86_64.sh`.
- Plasma starts on our wallpaper (our look-and-feel package in `stackific-desktop`, Arch only, since Plasma is Arch's second desktop), and every user's MangoWM config adds Super+Space for Noctalia's launcher, at the operator's request; GNOME's wallpaper waits for the Fedora installer task.
