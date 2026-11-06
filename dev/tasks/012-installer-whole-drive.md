# 012: The installer, into free space or onto a whole drive, from the live desktop or the boot menu, on the Arch x86_64 image

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Write the installer skeleton and its disk step, and ship it on the Arch x86_64 image: our own 4 GiB FAT32 EFI System Partition and one partition for the OS, created either in a drive's free space (next to what is there) or on a whole drive, the btrfs subvolumes, a copy of the live image's root, and the user. One program serves both bases from the start: a shared disk step, with per-base steps behind one interface. The Fedora images reuse the same Rust program later; this PR puts it on the Arch x86_64 image only. Write it in Rust with a ratatui text interface. The user must be able to type in, and options must appear, for the timezone and the keyboard. Create an empty @swap now, because zram plus hibernation for laptops comes later. For the free space, refuse what is not safe yet (BitLocker, no GPT table, a table that needs repair); the BitLocker confirmation, OS detection and the Windows boot entry come later. Add a launcher, so the installer opens from the live desktop. The installer must also run from the live ISO itself, when the user picks Install instead of Try at boot. The user must be able to read the installer on any screen, so the wizard starts with a text size that lets them continue (automatic, and theirs to change); scaling the installed desktop comes later, as options after the install. Adapt the Apple design guide (a skill text from WWDC talks, written for the web) to these screens as far as possible; its rules are soft requirements, so skip what is close to impossible in a terminal. Add a partition editor to the live desktop's launcher, the one most compatible with our choices, so anyone who wants to set up their disk first can do it there. It has to work with KDE Plasma as well as MangoWM. Add an unattended install test. Add Rust tooling to just. Add ASCII screenshots of the screens to the task. In the prompt trail, show real screenshots of the screens, on the desktop and on the console.

Adopt the guide's principles, as soft requirements, as fully as a terminal reasonably allows: response, direct manipulation, interruptible spring motion, spatial consistency, rubber-banding, materials and depth, feedback, reduced motion and contrast, typography, and its eight design principles. It must look right on an ultrawide monitor, a 13-inch laptop and everything between, so the screens have a fixed maximum size.

Also, in the same PR: on the README, drop the "Built from" column from the image table, drop the "## Trademarks" heading and turn its three sentences into small print with <sub>. In the release notes, replace the "Built from" column with "First desktop" (MangoWM) and turn the same sentences into small print. Reduce the mentions of Noctalia in what users read, except in the prompt trail. Once the base column is gone, the README and the release notes name no Fedora mark (the ISO file names do not count), so they carry no Fedora notice and no Red Hat trademark line, only the non-affiliation sentence in small print; AGENTS.md says so.
```

## Goal
`stackific-installer`, a Rust program with a ratatui text interface, installs Stackific OS from the Arch x86_64 live session into a drive's free space or onto a whole drive, interactively (from an "Install Stackific OS" launcher in the live desktop, or from an "Install Stackific OS" boot-menu entry that runs it on the console) or from an unattended TOML file. Its first screen, Display, sets a text size the user can read, and motion and contrast. `just arch test-install` proves both modes on a 64 GiB disk image. The installed system does not boot yet: Limine comes in task 013. The same PR adds the Rust toolchain to `just setup` and trims the README, the release notes and the user docs as the Ask says.

## Context
What exists (main at `2128b35`, checked 2026-11-06):
- No `installer/` directory and no Rust in the repo. `just setup` (root `justfile` line 43) installs uv, rtk, Serena, the Claude plugins, graphify, gitleaks, shellcheck, actionlint, gh, the site's packages, the git hooks and the Mac's runner VM. `.serena/project.yml` lists only `bash`. The marketplace `claude-plugins-official` offers the plugin `rust-analyzer-lsp` (its `marketplace.json`, read 2026-11-06).
- The Arch live ISO comes from `arch/iso/` (archiso 92-1, `bootmodes=('uefi.systemd-boot')`, xz squashfs), with packages from `arch/iso/packages.x86_64`, among them `foot` and `konsole`. `just arch packages` builds every `arch/packages/*/PKGBUILD` with `makepkg --nodeps` in the builder (`arch/Containerfile`), staging each `source=()` file by name from `desktop/` and `branding/`. `stackific-desktop`'s PKGBUILD renders `*.in` files from `branding.env` with `envsubst`.
- `tests/vm/boot.sh <iso> <out-dir> <checks>` boots an ISO with `-cdrom` only, no disk. Its `vm` helper sends the token line, then the command, to a bash over vsock (`tests/vm/stackific-test-shell.service`), so multi-line commands work. `socat -t 30` closes the connection 30 s after the command's input ends, so a long command must run detached.
- `.github/workflows/images.yml` and `release.yml` run `just arch iso` and `just arch test` in the `arch-x86_64` job on `[self-hosted, Linux, X64]`.

Facts about the live image, from the x86_64 ISO built 2026-11-05 (1497.4 MiB, 3.38 GiB of files, squashfs 1345.6 MiB), its manifest, and the archiso sources (read 2026-11-06):
- archiso's hook (`mkinitcpio-archiso` 73-1, `hooks/archiso`) mounts the boot medium at `/run/archiso/bootmnt` and the squashfs root read-only at `/run/archiso/airootfs`, the lower layer of the live overlay. That is the pristine image to copy.
- mkarchiso (`_cleanup_pacstrap_dir`) empties `/boot` and deletes pacman's sync databases before packing, so the copied root has no kernel in `/boot`. The kernel stays in `/usr/lib/modules/<version>/vmlinuz`; task 013 puts it on the ESP.
- Live-only state in the root (`arch/iso/airootfs/`): `/etc/greetd/config.toml` with `[initial_session]` (`mango` as `live`) and `[default_session]` (`/usr/bin/noctalia-greeter-session` as `greeter`); `/etc/sysusers.d/stackific-live.conf` (`m live wheel`); user `live` (uid 1000, home `/home/live`) and `root`, both with empty passwords in `/etc/shadow`; `/etc/mkinitcpio.conf.d/archiso.conf`; `/etc/hostname`; `/etc/localtime` linking to `/usr/share/zoneinfo/UTC`; `/etc/locale.conf` with `LANG=C.UTF-8`. `/etc/machine-id` holds `uninitialized` (14 bytes), systemd's first-boot marker (machine-id(5)). `systemd-gpt-auto-generator` is masked (linked to `/dev/null`).
- Present in the root: `sfdisk`, `wipefs`, `blkid`, `mkfs.fat`, `mkfs.btrfs`, `chattr`, `lsattr`, `useradd`, `userdel`, `chpasswd`, `udevadm`, `systemd-firstboot`, `localectl`, `pkexec`, `run0` (a link to `systemd-run`), `foot`. Missing: `rsync`, `sudo`. (Corrected at build, 2026-11-06: `python` 3.14.9 is already in the image, pulled in by the existing desktop packages; this task adds none.)
- Keyboard data: XKB rules at `/usr/share/X11/xkb`, a link to `/usr/share/xkeyboard-config-2`; `rules/evdev.lst` lists 99 layouts (xkeyboard-config 2.48; counted at build), each a code and a name ("germ" matches exactly `de` German, `at` German (Austria) and `ch` German (Switzerland)). systemd's `/usr/share/systemd/kbd-model-map` maps console keymaps to XKB layouts (its first line for XKB `de` with no variant is the console keymap `de`; `us` maps to `us`). Timezones: 1866 entries under `/usr/share/zoneinfo/`.
- MangoWM 0.18.4: the stock `/etc/mango/config.conf` sets `xkb_rules_layout=us` (line 124), and `/etc/mango/config.toml` sets `xkb_rules_layout = "us"` (line 122). The `mango` binary looks for `~/.config/mango/config.conf` and `config.toml` and for `/etc/mango/config.conf` and `config.toml`, and knows the keywords `source` and `source_optional` (its strings). It reads no `XKB_DEFAULT_*` variable. `AGENTS.md` keeps MangoWM's config stock.
- Languages: Arch packages carry their translations (379 German `.mo` files in the root, among them `kwin.mo` and `dolphin.mo`; Noctalia ships `/usr/share/noctalia/assets/translations/*.json`). Only `C.utf8` is compiled under `/usr/lib/locale/`; glibc's 371 locale sources are present. The image has `noto-fonts` and `noto-fonts-emoji`, no CJK font. The installer stays English only, with no language screen (decision IN-14 under Inputs).
- `branding/os-release.in` gives `NAME`, `ID`, `DEFAULT_HOSTNAME` (the ID) and `ANSI_COLOR`, so the installer reads its name, labels, default computer name and accent from `/etc/os-release` and hard-codes none. `stackific-release` installs the icon `stackific` (hicolor, scalable).
- foot 1.28.0 takes the command to run as its arguments; `--app-id`, `--window-size-chars=<cols>x<rows>`, `--font=<name>:size=<pt>` and `--override=<key>=<value>` set its app ID, first size, font and any `foot.ini` key (foot(1)). With `dpi-aware=yes`, foot sizes fonts by the monitor's physical DPI, so a point size has the same physical size on every monitor; by default it scales by the compositor's scale instead (foot.ini(5); the option is in the binary). Its default bindings `font-increase` and `font-decrease` are Ctrl and + and Ctrl and −. Both MangoWM's launcher and KDE Plasma's menu list the `.desktop` files in `/usr/share/applications/`.
- Display scale: MangoWM's stock config sets no output scale, so every monitor runs at 100%. MangoWM supports the fractional-scale protocol, has per-monitor rules, and lacks wlr-output-management, so `wlr-randr` cannot change its scale at runtime (its binary's strings).
- The console: kbd 2.10.0 provides `setfont` and, among its 249 console fonts, `sun12x22`, `latarcyrheb-sun32` (16 × 32) and `solar24x32`, next to the kernel's built-in 8 × 16 (`/usr/share/kbd/consolefonts/`). The framebuffer's size is in `/sys/class/graphics/fb0/virtual_size` (`<width>,<height>`).
- Boot menu: `arch/iso/efiboot/loader/loader.conf` (`timeout 3`, `default 01-stackific-linux.conf`) and one entry, `entries/01-stackific-linux.conf`, titled `Stackific OS (%ARCH%, UEFI)` with `options archisobasedir=%INSTALL_DIR% archisosearchuuid=%ARCHISO_UUID% console=ttyS0,115200 console=ttyAMA0,115200 console=tty0`. A kernel option `systemd.unit=<target>` boots into that target instead of the default one (kernel-command-line(7)).

Packages and versions (Arch Linux Archive 2026-11-01 via the builder; Fedora 45 through `dnf repoquery` in `localhost/stackific-fedora-builder:45`; crates.io and static.rust-lang.org; all read 2026-11-06):
- Arch `rust` 1:1.99.1-1 (provides `cargo`; 309.14 MiB installed). Fedora 45 `rust` 1.99.1. Arch `rsync` 3.5.1-3 (858.91 KiB installed).
- rustup 1.29.3 (`release-stable.toml`). `rustup-init` SHA-256: x86_64-unknown-linux-gnu `b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55`, aarch64-unknown-linux-gnu `9f608c142e0e38a78957d0661ba1dfaf0b6fb37184cfaa4b9a0aa57af6184282`, aarch64-apple-darwin `63752a1919a3a4793173ec70f8f212621a7ce38ba6a246095984482f4de20f9f` (`https://static.rust-lang.org/rustup/archive/1.29.3/<target>/rustup-init.sha256`).
- Crates: `ratatui` 0.30.2 (crossterm backend; `crossterm` 0.29.0), `serde` 1.0.229, `serde_json` 1.0.154, `toml` 1.2.1.

Disk facts (util-linux and the UEFI specification, read 2026-11-06):
- GPT type GUIDs: EFI System `C12A7328-F81F-11D2-BA4B-00A0C93EC93B`, Linux filesystem `0FC63DAF-8483-4772-8E79-3D69D8477DE4`, Microsoft reserved `E3C9E316-0B5C-4DB8-817D-F92DF00215AE`, Microsoft basic data `EBD0A0A2-B9E5-4433-87C0-68B6B72699C7`, Windows recovery `DE94BBA4-06D1-4D40-A16A-BFD50179D6AC`.
- sfdisk aligns new partitions to 1 MiB (2048 sectors) by default; `--append` adds partitions without touching the existing ones; `--wipe-partitions always` clears old signatures only inside the new partitions; `--verify` reports table problems; `--json` gives each partition's start and size and the table's `firstlba` and `lastlba` (sfdisk(8)).
- `blkid -p` probes a partition directly and reports `TYPE="BitLocker"` for a BitLocker volume, which carries the signature `-FVE-FS-` at byte 3 of its first sector (libblkid's BitLocker prober).
- 4 GiB is 8388608 sectors of 512 bytes, and 40 GiB is 83886080. A FAT label holds at most 11 characters.

Partition editors (Arch Linux Archive 2026-11-01, Fedora 45, the live ISO's manifest; read 2026-11-06):
- The Arch image already has `udisks2` 2.11.2, `libblockdev` 3.5.0 with its `fs` and `part` plugins, `parted` 3.8, `polkit` 128, `polkit-kde-agent`, `qt6-base`, the KDE Frameworks Plasma needs (`kio`, `kauth`), `gtk3` and `xorg-xwayland`, but no `xorg-xhost` and no `ntfs-3g`. The Fedora image has `gnome-shell` and its session, and no disk tool.
- GParted 1.8.1 (7.5 MiB on Arch, 10.0 MB on Fedora): its `/usr/bin/gparted` runs the whole GUI as root through `pkexec --disable-internal-agent`, and under Wayland it first grants root access to the XWayland display with `xhost +SI:localuser:root` (Arch builds it with `--enable-xhost-root`).
- KDE Partition Manager 26.08.2 (`partitionmanager` 6.95 MiB plus `kpmcore` 4.6 MiB on Arch, `kde-partitionmanager` on Fedora): a Qt GUI that runs as the user, with a polkit helper (KAuth); it can move partitions. On the Fedora GNOME image it would pull in Qt 6 and KDE Frameworks.
- GNOME Disks 46.1 (`gnome-disk-utility`: 6.6 MiB on Arch, 7.0 MB on Fedora): a GTK 3 GUI that runs as the user and asks udisks2, which asks polkit, for each change; it ships `gnome-disks` and `org.gnome.DiskUtility.desktop` ("Disks"). It creates, deletes, formats and resizes partitions where libblockdev can resize the file system, but does not move partitions. NTFS resizing needs `ntfsresize`, which ships in `ntfsprogs` 2026.9.28 (850.51 KiB plus `libntfs-3g`; `ntfs-3g` mounts NTFS and lists `ntfsprogs` as optional; corrected at build, 2026-11-06). The operator's working plan already lists GNOME Disks among the first software picks for every image.
- A GUI that asks polkit needs an authentication agent in the session. Noctalia has one built in (its binary has `PolkitPanel` and `Application::syncPolkitAgent`); whether it is on by default, and the setting that turns it on, are not known yet. `polkit-kde-agent` autostarts only in KDE (`OnlyShowIn=KDE;` in `/etc/xdg/autostart/polkit-kde-authentication-agent-1.desktop`), so Plasma has its own. `mango-session.target` binds to `graphical-session.target`. `run0` in a terminal brings its own text agent when no other one answers.
- The Arch image has `xdg-desktop-portal-gtk` but not Plasma's GTK integration: `breeze-gtk` 6.8.2 (1.2 MiB, no dependencies) gives GTK apps Plasma's Breeze look, and `kde-gtk-config` 6.8.2 (0.3 MiB; its dependencies are on the image except possibly `xsettingsd`) applies Plasma's theme, fonts and colours to them and writes them into the user's GTK settings files, which a MangoWM session then reads too. Without them, GTK apps look like GNOME's in Plasma.

Not known yet; task 013 checks these on the booted system: whether KDE Plasma takes its first keyboard layout from systemd-localed's X11 setting (`/etc/X11/xorg.conf.d/00-keyboard.conf`), and whether MangoWM prefers `~/.config/mango/config.conf` to `/etc/mango/config.toml`.

## Inputs
Decisions from the operator's working plan and earlier decision records, restated:
- **Plan item 012.** The installer skeleton and its disk step on Arch x86_64: our own 4 GiB FAT32 ESP, btrfs subvolumes, the live-image copy and users, with an unattended install test. One program serves both bases from the start: a shared disk step, with per-base steps behind one interface. Limine on installed Arch is task 013; the installer on Fedora x86_64 is task 014, which reuses this crate unchanged and adds its `Base` implementation and packaging; both aarch64 images are task 018.
- **Free space now, Windows details later (operator 2026-11-06):** this task adds the free-space mode with safe refusals. Task 015 keeps the BitLocker typed confirmation with its protector checks (IN-02), OS detection (IN-04), the refusals of Intel RST RAID and BIOS-mode Windows (IN-07), the Windows-like disk fixture's remaining cases and the Windows entry in Limine (BT-05).
- **A launcher and a boot-menu entry now (operator 2026-11-06):** the live desktop gets "Install Stackific OS", and the boot menu gets "Install Stackific OS" next to "Try Stackific OS", although the installed system does not start until task 013. The user docs and release notes say so. This takes task 017's Arch part; task 017 keeps the Fedora part.
- **A partition editor in the launcher (operator 2026-11-06): GNOME Disks.** Of GParted, KDE Partition Manager and GNOME Disks, Disks fits our choices best: it is native Wayland in MangoWM, KDE Plasma and GNOME (in Plasma, Plasma's own `polkit-kde-agent` answers its prompts, and `breeze-gtk` with `kde-gtk-config` give it Plasma's look), runs as the user with polkit asking for each change (no root GUI, no `xhost`, no polkit rule that skips authentication), adds the least to both images, is the same app on all four, and is already in the plan's first software picks (task 023). Its limit is that it cannot move partitions; shrinking Windows itself is safest from Windows' Disk Management. It joins the Arch x86_64 image here; the other images get it with their installer tasks (014, 018) or task 023.
- **Readable from the first screen (operator 2026-11-06):** the installer opens at a readable size by itself (foot at physical size on the desktop; a console font picked for the screen's resolution) and starts with a Display screen (text size, motion, contrast), so the user can make it bigger before anything else. The display scale of the live and installed desktops is a separate task with its own decision table, offering options after the install.
- **Installer language (open decision 8 of the operator's spec; operator 2026-11-06):** Rust with a ratatui text interface, plus an unattended config file, so every install is testable. The test harness stays shell in `tests/vm/` (decision 14 is settled by what exists).
- **BT-01, our ESP:** our partition is a real EFI System Partition in both modes, FAT32, 4 GiB, mounted at `/boot` with an explicit fstab line, so `systemd-gpt-auto-generator` never guesses. Limine, the kernels and initramfs files go there (task 013). We never write to another OS's ESP. Two ESPs on one disk, which Microsoft calls unsupported, is an accepted and tested risk.
- **IN-08, adapted:** both modes create only our two partitions and touch nothing else; this works without archinstall, so one disk step serves both bases. Every partition-table entry that existed before stays byte-identical, and a test proves it. Free space is refused on a table libparted-style tools would rewrite or skip (a backup GPT header away from the disk's end, a hybrid MBR, a table with verify errors); erasing such a drive is still allowed. The whole-drive mode erases the drive after a typed confirmation (the operator's spec, its disk layout).
- **IN-01:** free space means the largest 1 MiB-aligned gap in the GPT, start rounded up and end rounded down to 2048 sectors; small leftover gaps are ignored.
- **IN-06, in part:** the gap, or the whole drive, must be at least 40 GiB. A later task sizes the swap file to RAM.
- **IN-02, in part:** a drive with any BitLocker partition gets no free-space install yet; it may still be erased.
- **IN-03, in part:** the last screen before any write shows what is on the drive now and what will be on it afterwards.
- **IN-10, adapted:** installs work offline by copying the live image's root.
- **IN-14:** the system language stays the live image's, and there is no language screen for now.
- **SN-01, the subvolume layout:** flat, top-level siblings of `@`, all mounted with `noatime,compress=zstd:1`:

  | Subvolume | Mount point | Rolls back? |
  | --- | --- | --- |
  | `@` | `/` (with `/usr`, `/etc`, `/opt`, `/var/lib`, pacman's database) | yes |
  | `@home` | `/home` | no |
  | `@root` | `/root` | no |
  | `@srv` | `/srv` | no |
  | `@cache` | `/var/cache` | no |
  | `@tmp` | `/var/tmp` | no |
  | `@log` | `/var/log` | no |
  | `@swap` | `/swap` | no |
  | `@flatpak` | `/var/lib/flatpak` | no |
  | `@libvirt` | `/var/lib/libvirt/images` | no |
  | `@containers` | `/var/lib/containers` | no |
  | `@docker` | `/var/lib/docker` | no |

  `/tmp` stays systemd's tmpfs. snapper's `/.snapshots` comes with the snapshots task (026). `@libvirt` and `@swap` are NOCOW (`chattr +C` on their empty roots), so btrfs never compresses them.
- **SN-05 and decision 10 (operator 2026-11-06):** `@swap` is created empty now, so the layout does not change when zram (SN-04) and a hibernation swap file in `@swap` arrive with task 026.

- **Trademark text only where a mark is used (operator 2026-11-06, after reading Fedora's current guidelines):** Fedora's notice is a condition of using the Fedora Remix marks, and Red Hat's trademark line belongs in the footer of a page that uses a Fedora mark. ISO file names such as `stackific-<version>-fedora-x86_64.iso` are not treated as such a use. So the README and the release notes, which name no mark once their base column is gone, carry only "Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat." in small print; the site, whose pages say "Fedora Remix", keeps both. `AGENTS.md` line 18 already says this in this branch, and the test build `v0.0.0-pr.11`'s notes were edited to match on GitHub.

No files copied into the tracked tree.

## Design
The operator's reference is a design guide distilled from Apple's WWDC talks (*Designing Fluid Interfaces*, *The Details of UI Typography*, *Principles of Great Design*), written for the web. Its rows are soft requirements: build each one where a terminal reasonably allows it, and drop one that turns out close to impossible, recording which and why in `docs/installer.md` and the review record. The hard requirements are under Acceptance criteria, Must; the soft ones are its Should list.

| The guide | In the installer |
| --- | --- |
| Response | Every key and mouse event is handled and redrawn in the same frame, with no debounce and no wait. A search filters on each keystroke, keys act on press, and an animation never delays input. |
| Direct manipulation | In foot, the mouse works: a press highlights a row or a bottom-border action at once, the release commits it, and moving off before the release cancels. The wheel moves a list's selection. The console has no mouse and stays keyboard-only. |
| Interruptibility | A key during an animation acts at once. A new target retargets the running spring from where it is, keeping its velocity, so Esc in the middle of a slide reverses it without a jump. Input is never locked out; during the install, Ctrl+C still works. |
| Springs | One spring model, set by a damping ratio and a response, drives all motion: screen slides (damping 1.0, response 0.3 s), the selection marker gliding between rows (1.0, 0.15 s) and the progress bar (1.0, 0.4 s, never moving backwards). There is no bounce anywhere, because keys carry no momentum. Frames run at up to 60 per second, and only while something moves. |
| Velocity handoff | A retarget keeps the spring's current velocity, so a reversal has no "brick wall". |
| Momentum | Holding ↑ or ↓ speeds the list up after half a second. A burst of wheel events (a touchpad flick) projects the resting row with the guide's projection (deceleration rate 0.99) and springs there. |
| Spatial consistency | Forward slides the next screen in from the right, and Back slides it out to the right. The stop dialog grows out of the bottom border's "Ctrl+C stop" and shrinks back into it. Keyboard and Timezone share one list, which looks and behaves the same. |
| Hint the direction | The selection marker glides toward the row it is going to, and a slide's direction says forward or back before the new screen settles. |
| Rubber-banding | Past a list's first or last row, a further key press nudges the list one row beyond the edge and springs it back (damping 1.0, response 0.2 s) instead of stopping dead. |
| Gesture details | No key waits for a second one: Esc acts at once, with no Alt-sequence delay, and there is no double press. With the mouse: highlight on press, commit on release, cancel by moving away. |
| Smoothness | ratatui redraws only the cells that changed. Motion steps at most one column or row per frame, and the progress bar moves in eighths of a cell. |
| Materials and depth | The frame is one surface. In a true-colour terminal (`COLORTERM=truecolor`, which foot sets), the frame's background is a slight tint of the terminal's, and the bottom border is a heavier band. The stop dialog sits on a dimming scrim with a one-cell shadow and materializes from its anchor in three frames. A scrolled list fades its first and last visible rows instead of drawing a divider. No tinted surface sits on another. |
| Sound and haptics | Utility only: one terminal bell, on the same frame as the Stopped screen, and nowhere else. |
| Reduced motion and accessibility | The Display screen sets Motion (Full, or Reduced: slides and glides become instant changes, the progress bar stays; Reduced is the console's default) and Contrast (Standard, or High: no dim text and no tints, with bold and reverse video). `NO_COLOR` (no-color.org) removes all colour. A selection is never shown by colour alone. Nothing blinks or loops. On the console (`TERM=linux`), whose fonts lack them, the markers `▶ ✓ ▸ • ▏ ◀` and the arrows become `> * > * _ <` and "Up/Down", "Left/Right"; the frame and the block characters stay. |
| Typography | The system's monospace font (the launcher asks foot for `monospace`), at the user's size. Hierarchy comes from weight and spacing as a set: titles bold, body regular, help dim; one blank line between groups and none inside tables; prose wraps at 76 columns; sizes are right-aligned with one decimal and a unit. |
| Purpose | No manual partitioning, no language screen, no option nobody asked for: Display, Welcome and six steps. |
| Agency | Free space or erase. Back keeps every answer. Confirmations exist only where data is at stake: the typed drive name before an erase, Enter on the free-space summary (nothing is erased), and the stop dialog. Nothing else asks "are you sure". |
| Responsibility | A before-and-after summary, refusals that explain themselves, and a password that is never echoed and never logged. |
| Familiarity | Arrows, Tab and Shift+Tab, Enter, Esc, Home, End, Page Up and Page Down, and the mouse. The same key means the same thing on every screen. |
| Flexibility | The desktop or the console; text size, motion and contrast. Any terminal from 80 × 24 up: the frame is exactly 80 × 24, centered horizontally and vertically, so it reads the same on an ultrawide monitor, a 13-inch laptop or a full-screen console at any resolution. A smaller terminal shows only "make the window bigger or the text smaller", and a resize redraws at once. |
| Simplicity | Plain words in sentence case ("Prepare the system partition", not "Create the btrfs subvolumes"). Defaults come first (`us`, `UTC`, `DEFAULT_HOSTNAME`, the user name from the first name, "Use the free space" when usable), so the common path is Enter, screen after screen. |
| Craft | Every spacing, colour and timing is a named constant in one module (`ui/theme.rs`), each with a comment saying why. Content starts at the same column on every screen. |
| Delight | The feeling to reach is calm: no exclamation marks, no spinners, steady motion. |
| The four kinds of feedback | Status: the progress bar and the step list. Completion: Done. Warning (the terminal's yellow): the Space screen's reasons. Error (its red): Stopped, with the failing command and the log. Validation runs on every keystroke, shows the first problem under the fields, and "Enter next" appears in the bottom border only once the screen is valid. |
| Wayfinding | The top border names the program and "n of 6 · Step". The bottom border lists only the keys that work here, named by what they do ("Enter install", never "Enter OK"). Esc goes back; on the first screen it quits, or on the console starts the live desktop. Ctrl+C quits at once before the install, since nothing is written yet, and asks during it. No screen traps the user. |
| Process | `just installer demo` runs every screen on any machine, without root and without touching a disk, against fixture drives and a simulated install; `--slow` runs all motion ten times slower, to review it frame by frame. |

## The screens
Drawn at the frame's size, 80 columns; the real frame is 24 rows, and blank rows at its bottom are trimmed here. `Stackific OS` and `stackific` come from `/etc/os-release`. A snapshot test renders each one with ratatui's `TestBackend` and compares it with a text fixture. The prompt-trail entry shows real screenshots of the screens instead, on the desktop and on the console, taken from the image in a VM (operator 2026-11-06).

Display, on the console (the boot-menu path, `TERM=linux`): ←→ on Text size changes the console font at once with `setfont`. The first pick goes by the screen's height (under 1000 pixels Small, under 1500 Medium, under 2200 Large, else Extra large), and a size is offered only when it leaves at least 80 × 24 characters. Motion starts at Reduced here.

```text
┌ Stackific OS installer ───────────────────────────────────────────── Display ┐
│                                                                              │
│  Make this easy to read. Changes show at once.                               │
│                                                                              │
│  > Text size     < Large  16 × 32 >                                          │
│    Motion        < Reduced >                                                 │
│    Contrast      < Standard >                                                │
│                                                                              │
│  Text size was picked for this 3840 × 2160 screen. Sizes that leave less     │
│  than 80 × 24 characters are not offered.                                    │
│                                                                              │
└ Up/Down choose · Left/Right change · Enter next · Esc try the live desktop ──┘
```

Display, on the desktop: foot opened with `dpi-aware=yes`, so the text has the same physical size on every monitor; the Text size row names foot's own keys and cannot be selected. Motion and Contrast change the screen as you press ←→.

```text
┌ Stackific OS installer ───────────────────────────────────────────── Display ┐
│                                                                              │
│  Make this easy to read. Changes show at once.                               │
│                                                                              │
│    Text size     Ctrl and + bigger, Ctrl and − smaller                       │
│  ▶ Motion        ◀ Full ▶                                                    │
│    Contrast      ◀ Standard ▶                                                │
│                                                                              │
│  The text opened at the same physical size on every screen, from a 13-inch   │
│  laptop to an ultrawide monitor.                                             │
│                                                                              │
└ ↑↓ choose · ←→ change · Enter next · Esc quit ───────────────────────────────┘
```

Welcome. On the console, its Esc is "Esc try the live desktop", as on Display.

```text
┌ Stackific OS installer ───────────────────────────────────────────── Welcome ┐
│                                                                              │
│  Install Stackific OS on this computer.                                      │
│                                                                              │
│  It goes into a drive's free space, next to what is already there, or onto   │
│  a whole drive, which it erases first.                                       │
│                                                                              │
│  You pick a keyboard layout, a timezone, your user, the drive and how to use │
│  it. Nothing is written until you confirm on the last screen.                │
│                                                                              │
└ Enter start · Esc quit ──────────────────────────────────────────────────────┘
```

Keyboard: typing filters the layouts from `evdev.lst` by code and name, ignoring case. An exact code comes first, then shorter names. `us` is preselected when the search is empty.

```text
┌ Stackific OS installer ─────────────────────────────────── 1 of 6 · Keyboard ┐
│                                                                              │
│  Keyboard layout for the installed system                                    │
│                                                                              │
│  Search: germ▏                                                               │
│                                                                              │
│  ▶ de      German                                                            │
│    at      German (Austria)                                                  │
│    ch      German (Switzerland)                                              │
│                                                                              │
│  3 matches                                                                   │
│                                                                              │
│                                                                              │
│  It takes effect on the installed system. Here, you keep typing with the     │
│  live session's layout (us).                                                 │
│                                                                              │
└ ↑↓ choose · type to search · Enter next · Esc back ──────────────────────────┘
```

The same screen on the Linux console (`TERM=linux`):

```text
┌ Stackific OS installer ─────────────────────────────────── 1 of 6 · Keyboard ┐
│                                                                              │
│  Keyboard layout for the installed system                                    │
│                                                                              │
│  Search: germ_                                                               │
│                                                                              │
│  > de      German                                                            │
│    at      German (Austria)                                                  │
│    ch      German (Switzerland)                                              │
│                                                                              │
│  3 matches                                                                   │
│                                                                              │
└ Up/Down choose · type to search · Enter next · Esc back ─────────────────────┘
```

Timezone: typing filters the zones from `timedatectl list-timezones` (it reads the zone list offline, no daemon), ignoring case. `UTC` comes first when the search is empty.

```text
┌ Stackific OS installer ─────────────────────────────────── 2 of 6 · Timezone ┐
│                                                                              │
│  Timezone for the installed system                                           │
│                                                                              │
│  Search: berl▏                                                               │
│                                                                              │
│  ▶ Europe/Berlin                                                             │
│                                                                              │
│  1 match                                                                     │
│                                                                              │
│                                                                              │
│  An empty search lists every timezone, UTC first.                            │
│                                                                              │
└ ↑↓ choose · type to search · Enter next · Esc back ──────────────────────────┘
```

You: the user name fills in from the first word of the name (lowercase ASCII) until edited; the computer name starts as `DEFAULT_HOSTNAME`. Passwords show as dots. The first problem shows under the fields as you type, and "Enter next" appears once every field is valid.

```text
┌ Stackific OS installer ──────────────────────────────────────── 3 of 6 · You ┐
│                                                                              │
│  Your user, an administrator of this computer                                │
│                                                                              │
│    Your name        Ada Lovelace                                             │
│    User name        ada                                                      │
│    Password         ••••••••                                                 │
│    Confirm          •••••••▏                                                 │
│    Computer name    stackific                                                │
│                                                                              │
│  ✗ The passwords do not match yet.                                           │
│                                                                              │
│                                                                              │
│  User name: lowercase letters, digits, - and _, starting with a letter.      │
│  Computer name: lowercase letters, digits and -, up to 63.                   │
│  Typed with the live session's layout (us).                                  │
│                                                                              │
└ Tab next field · Esc back ───────────────────────────────────────────────────┘
```

Drive: whole disks only (`lsblk` type `disk`, never zram), with their largest usable free space, and each left-out drive with its reason. The drives are read again each time this screen shows and when the terminal regains focus, so changes made in Disks appear on return. On the console, the Disks line is left out.

```text
┌ Stackific OS installer ────────────────────────────────────── 4 of 6 · Drive ┐
│                                                                              │
│  Drive to install on                                                         │
│                                                                              │
│                   Size       Largest free space                              │
│  ▶ /dev/nvme0n1   931.5 GiB  120.0 GiB    Samsung SSD 990 PRO 1TB            │
│    /dev/sda       465.8 GiB  no table     CT500MX500SSD1                     │
│                                                                              │
│  Not listed:                                                                 │
│    /dev/sdb       the USB stick this installer runs from                     │
│    /dev/sdc       29.8 GiB, under the 40 GiB minimum                         │
│                                                                              │
│                                                                              │
│  To change partitions first, open Disks from the app launcher. This list     │
│  updates when you come back.                                                 │
│                                                                              │
└ ↑↓ choose · Enter next · Esc back ───────────────────────────────────────────┘
```

Space: how to use the drive. "Use the free space" comes first when it is usable; otherwise its line says why not, and a drive with no partition table offers only the erase.

```text
┌ Stackific OS installer ────────────────────────────────────── 5 of 6 · Space ┐
│                                                                              │
│  How to use /dev/nvme0n1 (Samsung SSD 990 PRO 1TB, 931.5 GiB)                │
│                                                                              │
│  ▶ Use the free space     the 120.0 GiB after nvme0n1p3; the rest stays      │
│    Erase the whole drive  everything on it is erased                         │
│                                                                              │
│  On it now:                                                                  │
│    nvme0n1p1     260.0 MiB   vfat    SYSTEM                                  │
│    nvme0n1p2      16.0 MiB                                                   │
│    nvme0n1p3     811.2 GiB   ntfs    Windows                                 │
│    free          120.0 GiB                                                   │
│                                                                              │
│                                                                              │
│  When the free space cannot be used, its line says why: under 40 GiB, no GPT │
│  partition table, a table that needs repair, or BitLocker on the drive.      │
│                                                                              │
└ ↑↓ choose · Enter next · Esc back ───────────────────────────────────────────┘
```

Confirm, free space: what stays, what is new, and the answers. Enter installs.

```text
┌ Stackific OS installer ──────────────────────────────────── 6 of 6 · Confirm ┐
│                                                                              │
│  Install Stackific OS in the free space on /dev/nvme0n1?                     │
│                                                                              │
│  Stays as it is:                                                             │
│    nvme0n1p1     260.0 MiB   vfat    SYSTEM                                  │
│    nvme0n1p2      16.0 MiB                                                   │
│    nvme0n1p3     811.2 GiB   ntfs    Windows                                 │
│                                                                              │
│  New, in the 120.0 GiB of free space:                                        │
│    new             4.0 GiB   FAT32   Stackific OS boot                       │
│    new           116.0 GiB   btrfs   Stackific OS                            │
│                                                                              │
│  Keyboard de (German) · Timezone Europe/Berlin                               │
│  User Ada Lovelace (ada), administrator · Computer stackific                 │
│                                                                              │
│  Nothing else on the drive changes.                                          │
│                                                                              │
└ Enter install · Esc back ────────────────────────────────────────────────────┘
```

Confirm, whole drive: what is erased, the layout afterwards, the answers, and the typed confirmation. Enter does nothing until the typed text equals the drive's kernel name.

```text
┌ Stackific OS installer ──────────────────────────────────── 6 of 6 · Confirm ┐
│                                                                              │
│  Erase /dev/nvme0n1 (Samsung SSD 990 PRO 1TB, 931.5 GiB)?                    │
│                                                                              │
│  Now on it, all erased:                                                      │
│    nvme0n1p1     260.0 MiB   vfat    SYSTEM                                  │
│    nvme0n1p2      16.0 MiB                                                   │
│    nvme0n1p3     811.2 GiB   ntfs    Windows                                 │
│                                                                              │
│  After the install:                                                          │
│    1               4.0 GiB   FAT32   Stackific OS boot                       │
│    2             927.5 GiB   btrfs   Stackific OS                            │
│                                                                              │
│  Keyboard de (German) · Timezone Europe/Berlin                               │
│  User Ada Lovelace (ada), administrator · Computer stackific                 │
│                                                                              │
│  Type nvme0n1 to erase it:  nvme0n1▏                                         │
│                                                                              │
└ Enter install · Esc back ────────────────────────────────────────────────────┘
```

Installing: one line per step; the copy's percentage comes from rsync's `--info=progress2`.

```text
┌ Stackific OS installer ────────────────────────────────────────── Installing ┐
│                                                                              │
│  Installing Stackific OS on /dev/nvme0n1                                     │
│                                                                              │
│  ✓ Create the partitions                                                     │
│  ✓ Format them                                                               │
│  ✓ Prepare the system partition                                              │
│  ▸ Copy the system           ███████████████▏░░░░░░░░░░░░░   52%             │
│    Set the keyboard, timezone and computer name                              │
│    Create your user                                                          │
│    Remove the live session's settings                                        │
│                                                                              │
│  Do not turn the computer off.                                               │
│                                                                              │
└ Ctrl+C stop ─────────────────────────────────────────────────────────────────┘
```

Ctrl+C during the install asks first, over the dimmed screen. Enter stops: it ends rsync, unmounts and shows Stopped, saying the user stopped it.

```text
┌ Stackific OS installer ────────────────────────────────────────── Installing ┐
│                                                                              │
│  Installing Stackific OS on /dev/nvme0n1                                     │
│                                                                              │
│  ✓ Create ┌ Stop the install? ──────────────────────────────────┐            │
│  ✓ Format │                                                     │            │
│  ✓ Prepare│  The new partitions stay on /dev/nvme0n1, partly    │            │
│  ▸ Copy th│  written. Nothing else on the drive changes.        │            │
│    Set the│                                                     │            │
│    Create └ Enter stop · Esc keep installing ───────────────────┘            │
│    Remove the live session's settings                                        │
│                                                                              │
│  Do not turn the computer off.                                               │
│                                                                              │
└ Ctrl+C stop ─────────────────────────────────────────────────────────────────┘
```

Done, in this task (task 013 changes the text once Limine boots the drive):

```text
┌ Stackific OS installer ──────────────────────────────────────────────── Done ┐
│                                                                              │
│  Stackific OS is on /dev/nvme0n1.                                            │
│                                                                              │
│  The boot loader is not in this build yet, so the drive does not start on    │
│  its own. Keep using the live session; a later build adds the boot loader.   │
│                                                                              │
└ Enter quit ──────────────────────────────────────────────────────────────────┘
```

Done on the console, where Enter hands over to the live desktop:

```text
┌ Stackific OS installer ──────────────────────────────────────────────── Done ┐
│                                                                              │
│  Stackific OS is on /dev/nvme0n1.                                            │
│                                                                              │
│  The boot loader is not in this build yet, so the drive does not start on    │
│  its own. Keep using the live session; a later build adds the boot loader.   │
│                                                                              │
└ Enter open the live desktop ─────────────────────────────────────────────────┘
```

Stopped, when a step fails:

```text
┌ Stackific OS installer ───────────────────────────────────────────── Stopped ┐
│                                                                              │
│  The install stopped: mkfs.btrfs /dev/nvme0n1p5 failed (exit 1).             │
│                                                                              │
│  Only the new partitions on /dev/nvme0n1 are partly written. No other        │
│  partition or drive was touched.                                             │
│                                                                              │
│  Details: /run/stackific-installer.log                                       │
│                                                                              │
└ Enter quit ──────────────────────────────────────────────────────────────────┘
```

A terminal smaller than 80 × 24 (a small window, or text made too big), top left:

```text
 Stackific OS installer

 Make the window bigger or the text smaller
 (Ctrl and −): the installer needs 80 × 24.
 It is 60 × 18 now.

 Ctrl+C quits.
```

## How it installs
0. **Where it runs.** From the launcher, in foot at physical size, through `run0`. From the boot menu's Install entry, `systemd.unit=stackific-installer.target` boots to `multi-user.target` without the greeter, and `stackific-installer-console.service` runs `stackific-installer --console` as root on `/dev/tty1` (getty conflicts with it). When the installer ends there, for any reason, the unit isolates `graphical.target`, so the live desktop starts. A target and a service are the glue, because no packaged tool starts a program on tty1 from a boot entry. The Display screen comes first in both: on a console (`TERM=linux`) its Text size sets the font with `setfont`; in foot it names Ctrl and + and −.
1. **Checks** (before any screen; exit 2 with one line on stderr): running as root ("Run it as root: run0 stackific-installer"), booted in UEFI mode (`/sys/firmware/efi`), and the base's live root is mounted.
2. **Answers:** the screens above, or `--config <file>`, a TOML file with the same answers and the same checks:

   ```toml
   [disk]
   device = "/dev/vda"
   mode = "free-space"    # or "erase"
   erase = "vda"          # with mode = "erase" only: the typed confirmation, the device's kernel name

   [system]
   keyboard = "de"        # an XKB layout code from evdev.lst
   timezone = "Europe/Berlin"
   hostname = "stackific-test"

   [user]
   full_name = "Test User"
   name = "tester"
   password = "…"
   ```

   A password with a control character is refused, because chpasswd reads `name:password` lines (review round 1). Rules for both paths: the device is a listed drive (whole disk, writable, not zram, not the live medium found from `/run/archiso/bootmnt`, at least 40 GiB, nothing on it mounted or used as swap). Free space also needs a GPT table that `sfdisk --verify` passes, a protective MBR with nothing but its one `ee` entry, no partition that `blkid -p` calls `BitLocker`, and a 1 MiB-aligned gap of at least 40 GiB. The user name matches `^[a-z_][a-z0-9_-]{0,31}$` and is not a user in the live root's `/etc/passwd` (`live` excepted); the computer name matches `^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$`; the password is not empty (twice, and equal, on screen).
3. **Disk step (shared):**
   - Erase: `sfdisk --wipe always --wipe-partitions always` with a new GPT: partition 1 of 4 GiB, EFI System type, named `<NAME> boot`; partition 2 the rest, Linux filesystem type, named `<NAME>`. The script names `first-lba:` and each start and size in sectors (1 MiB, then to the last usable sector rounded down to 1 MiB, in the drive's logical sectors from lsblk's `LOG-SEC`), because sfdisk aligns its own defaults to the drive's reported I/O size (review round 1).
   - Free space: `sfdisk --append --wipe never --wipe-partitions always` with the same two partitions at explicit `start=` and `size=` inside the chosen gap, the ESP first, the system partition to the gap's aligned end, in one write.
   - Then: `udevadm settle`, and the new partitions' paths read back from `lsblk` by their start sector (never predicted). `mkfs.fat -F 32 -n <ID in capitals, 11 characters at most>` on the ESP; `mkfs.btrfs -f -L <ID>` on the system partition. Create the SN-01 subvolumes and `chattr +C` the roots of `@libvirt` and `@swap`. Mount `@`, then each subvolume, with `noatime,compress=zstd:1`, and the ESP at `/boot` (`umask=0077`), under `/run/stackific-installer/target`.
4. **Copy (shared):** `rsync -aHAXx --numeric-ids --info=progress2 --no-inc-recursive --exclude=/boot/ <live root>/ <target>/`. `--no-inc-recursive` makes progress2's percentage count the whole copy; `/boot` is empty in the live root and is the FAT ESP in the target, which takes no owners, ACLs or xattrs.
5. **System (shared):** write `/etc/fstab` (by UUID: one line per subvolume with `subvol=/<name>,noatime,compress=zstd:1`, pass 0; `/boot` as `vfat` with `umask=0077`, pass 2); `systemd-firstboot --root=<target> --force --timezone=<zone> --hostname=<name> --keymap=<console keymap>`, where the console keymap is the first `kbd-model-map` line whose XKB layout is the chosen one with no variant (`us` when there is none); `/etc/X11/xorg.conf.d/00-keyboard.conf` with `XkbLayout` set to the chosen layout, in the format `localectl set-x11-keymap` writes (the build checks that format once on the live image and matches it).
6. **User (shared):** `userdel --root <target> --remove live` first, so our user gets uid 1000; then `useradd --root <target> --create-home --groups wheel --comment <full name> --shell /usr/bin/bash <name>`; the password through `chpasswd --root <target>` on stdin (the base's default hash; yescrypt on Arch); `usermod --root <target> --lock root`; then `~/.config/mango/config.conf`, owned by the user, holding `source=/etc/mango/config.conf` and `xkb_rules_layout=<layout>`, so MangoWM keeps its stock config with our layout on top.
7. **Live state (shared, then per base):** remove `/etc/sysusers.d/stackific-live.conf` and the `[initial_session]` table of `/etc/greetd/config.toml` (keep `[terminal]` and `[default_session]`). `/etc/machine-id` stays `uninitialized`. Arch's own step removes `/etc/mkinitcpio.conf.d/archiso.conf` and uninstalls the package `stackific-installer` (and the dependencies only it needed) from the target with pacman, so the installed system has no installer or launcher. The masked `systemd-gpt-auto-generator` stays masked. `/etc/locale.conf` stays the live image's.
8. **End:** unmount the target (subvolumes, then `@`, then the ESP) and print or show Done. Exit 0 on success, 2 on a refusal or invalid answers (nothing written), 3 when the user leaves before the confirmation (nothing written), 1 when a step fails after the confirmation, 130 when the user stops it. Each step's command and output go to `/run/stackific-installer.log` (interactive) or stderr (unattended), never the password.

**The per-base interface:** a Rust trait (`Base`) with the live root path (Arch: `/run/archiso/airootfs`), the live medium's mount point (Arch: `/run/archiso/bootmnt`), and a step that removes the base's own live state (Arch: the line above). Task 013 adds the boot step (initramfs and Limine), and task 014 adds a Fedora implementation. The installer picks the base from `/etc/os-release` (`ID_LIKE`), so no flag is needed.

**The launcher:** `/usr/share/applications/stackific-installer.desktop`, rendered from `branding.env`: `Name=Install ${NAME}`, `Comment=Install ${NAME} on this computer`, `Icon=${ID}`, `Categories=System;`, `Keywords=install;installer;disk;drive;`, `Terminal=false` and `Exec=foot --app-id=stackific-installer --window-size-chars=100x30 --font=monospace:size=11 --override=dpi-aware=yes run0 stackific-installer`. One terminal in both desktops (foot is on the image), at 11 points of physical size; run0 asks for `live`'s password (empty, Enter) through polkit.

## Changes
Installer:
- `installer/Cargo.toml`, `installer/Cargo.lock` (new): crate `stackific-installer` 0.1.0, edition 2024, `rust-version = "1.99"`, dependencies `ratatui` (crossterm backend), `serde` (derive), `serde_json`, `toml`; nothing else.
- `installer/src/main.rs` (new): `--config <file>`, `--console`, `--demo`, `--slow` and `--help` (read by hand, no argument crate), the checks, then the screens or the unattended run.
- `installer/src/` modules (new, one concern each): `os_release.rs` (NAME, ID, DEFAULT_HOSTNAME, ANSI_COLOR), `answers.rs` (the answers, their validation and the TOML file), `drives.rs` (drives from `lsblk --json`, tables from `sfdisk --json` and `--verify`, the largest aligned gap, BitLocker from `blkid -p`, eligibility and reasons), `disk.rs` (both sfdisk scripts, formatting, subvolumes, mounts, fstab text), `copy.rs` (rsync, its progress and its stop), `system.rs` (keyboard, timezone, computer name, user, live state), `base.rs` (the `Base` trait and `Arch`), `run.rs` (every external command goes through one `Runner` trait: the real one, and a fake that tests and the demo use), `ui/` (one file per screen, the Display screen with its font pick and `setfont`, `theme.rs` with every spacing, colour and timing, `motion.rs` with the spring, the projection and the rubber band, mouse handling, the centered frame, the type-to-search list shared by keyboard and timezone, the stop dialog, the styles: accent from `ANSI_COLOR`, `NO_COLOR`, the `TERM=linux` symbols).
- `installer/fixtures/` (new): `lsblk`, `sfdisk` and `blkid` outputs for the tests and the demo (a laptop drive with Windows and free space, a blank drive, a BitLocker drive, the live USB stick, a small drive).
- `installer/stackific-installer.desktop.in` (new): the launcher above.
- `installer/systemd/stackific-installer.target` and `installer/systemd/stackific-installer-console.service` (new): the console path above (`AllowIsolate=yes` on the target; the service with `StandardInput=tty`, `TTYPath=/dev/tty1`, `TTYReset=yes`, `TTYVHangup=yes`, `Conflicts=getty@tty1.service`, `SuccessExitStatus=3 130` (leaving and stopping are the user's choices) and `ExecStopPost=/usr/bin/systemctl --no-block isolate graphical.target`).
- `installer/tests/` or `#[cfg(test)]` modules (new): the tests under Acceptance criteria, with the screen snapshots as text fixtures.
- `installer/justfile` (new, `mod installer` in the root `justfile`): `build`, `demo` (`cargo run -- --demo`, with any extra flags such as `--slow`), `test` (`cargo test --locked`), `lint` (`cargo fmt --check` and `cargo clippy --all-targets --locked -- -D warnings`), each depending on the Rust toolchain's install.
- `arch/packages/stackific-installer/PKGBUILD` (new): `arch=('x86_64' 'aarch64')`, `depends=('btrfs-progs' 'dosfstools' 'e2fsprogs' 'foot' 'kbd' 'rsync' 'shadow' 'systemd' 'util-linux' 'xkeyboard-config' 'stackific-release')`, `makedepends=('cargo' 'gettext')`; `prepare()` runs `cargo fetch --locked`, `build()` `cargo build --frozen --release` and renders the launcher with `envsubst '$NAME $ID'`, `check()` `cargo test --frozen`; installs `/usr/bin/stackific-installer`, the launcher and the two units in `/usr/lib/systemd/system/`. Its sources are `branding.env`, the launcher template, the two units and the crate staged by the recipe (next line), with a comment saying why the crate's files are not listed one by one.
- `arch/justfile` (`packages`): also mount `installer/`, find the launcher template and the units there by name like the other sources (never in `target/`), and pack `Cargo.toml`, `Cargo.lock`, `src/` and `fixtures/` (for `check()`) into `stackific-installer-crate.tar` in the installer's package folder, which the PKGBUILD lists with checksum `SKIP` (makepkg takes no directories as sources). `iso`: on aarch64, leave `02-stackific-install.conf` out of a copy of the profile, because the shared profile would otherwise give the aarch64 image an Install entry without the installer (operator 2026-11-06; task 018 removes that line).
- `arch/Containerfile`: add `rust` to the builder on both architectures (makepkg runs with `--nodeps`), and `qemu-img` for the install test's disk image.
- `arch/iso/packages.x86_64`: add `stackific-installer`, `gnome-disk-utility`, `ntfs-3g`, `ntfsprogs` (`ntfsresize`), `breeze-gtk` and `kde-gtk-config`. `packages.aarch64` waits for task 018.
- `desktop/noctalia/stackific-noctalia.toml`: turn on Noctalia's polkit agent, only if it is off by default (the build finds the setting); then new sha256s and a `pkgrel` bump in `arch/packages/stackific-desktop/PKGBUILD`, and a `Release` bump in `fedora/packages/stackific-desktop/stackific-desktop.spec`, as `AGENTS.md` requires for a shared file.
- `arch/iso/efiboot/loader/entries/01-stackific-linux.conf`: title `Try Stackific OS (%ARCH%, UEFI)`. `arch/iso/efiboot/loader/entries/02-stackific-install.conf` (new): title `Install Stackific OS (%ARCH%, UEFI)`, the same kernel, initrd and options plus `systemd.unit=stackific-installer.target`. `loader.conf` keeps Try as the default.

Tests:
- `tests/vm/boot.sh`: when `STACKIFIC_TEST_DISK` names a raw image file, attach it as a virtio disk (`/dev/vda` in the guest); nothing else changes.
- `tests/vm/screens.sh` (new): drives the real installer by key presses (QEMU `sendkey`) and saves a screenshot of each screen: on the desktop from MangoWM's launcher (a free-space install, then a stopped erase) and on the console from the Install target (an erase). `tests/vm/boot.sh`'s `sendkey` takes several keys.
- `arch/justfile`: `screenshots version="0.0.0-dev"` boots the ISO once with a fresh 64 GiB disk and `tests/vm/screens.sh`, leaving the PNGs in `.build/arch/<arch>/screens/`; it is not part of CI.
- `tests/vm/install.sh` (new): the check file for the unattended installs, base-neutral (task 014 reuses it), written to the criteria below. It writes the free-space install's ESP offset to `<out-dir>/esp-offset` for the host check.
- `tests/vm/arch.sh`: the launcher is in the live session: `/usr/share/applications/stackific-installer.desktop` with `Name=Install Stackific OS` and the `Exec=` line above.
- `tests/vm/boot.sh`: a `sendkey <key>` helper beside `screenshot`, through the same QEMU monitor socket.
- `tests/vm/arch.sh`: Disks is in the live session: `/usr/bin/gnome-disks`, `/usr/share/applications/org.gnome.DiskUtility.desktop` and `/usr/bin/ntfsresize`; Plasma's GTK integration is in: `/usr/share/themes/Breeze/gtk-3.0/` and `kde-gtk-config` in the package manifest.
- `arch/justfile`: `test-install version="0.0.0-dev"` reads the boot menu's entries from the ISO with `bsdtar`, runs boot.sh in the builder with a fresh 64 GiB sparse raw image (`qemu-img create -f raw`) in `.build/arch/<arch>/` and `tests/vm/install.sh`, then checks the free-space install's ESP with `minfo` (mtools, in the builder, outside the VM) at the recorded offset.
- `.github/workflows/images.yml` and `release.yml`: `just arch test-install "$VERSION"` after `just arch test` in the `arch-x86_64` jobs.

Rust tooling:
- `just/rust/vars.just` (new): rustup 1.29.3 with the three SHA-256s above, Rust 1.99.1 (the same version Arch and Fedora 45 build with), `RUSTUP_HOME` and `CARGO_HOME` under `.tools/`, and the `cargo` path.
- `just/rust/mod.just` (new): `install` fetches `rustup-init` through `just/fetch` (checksum-pinned), installs the toolchain with `--profile minimal --component clippy,rustfmt,rust-analyzer` (a no-op when present), and puts a `rust-analyzer` launcher in `~/.local/bin` the way the other language servers get theirs.
- Root `justfile`: `mod rust 'just/rust/mod.just'` and `mod installer`; `rust::install` in `setup`; a `rust:` line in `doctor` (version and wiring).
- `lefthook.yml`: pre-commit `installer-lint` and `installer-test` on `installer/**`; then `just lefthook hooks`.
- `.claude/settings.json`: enable `rust-analyzer-lsp@claude-plugins-official`.
- `.serena/project.yml`: add `rust` to `language_servers`.
- `.gitignore`: `installer/target/` (build output, as its header allows).

Messaging:
- `README.md`: the image table becomes `| ISO | First desktop | Second desktop |` with `MangoWM` in every row and `KDE Plasma` or `GNOME`; the base column goes. The sentence under it says the images log in "to the MangoWM desktop", without "with the Noctalia shell". The `## Trademarks` heading goes, and so do the Fedora notice and Red Hat's trademark line; the README ends with the non-affiliation sentence as one `<sub>…</sub>` paragraph, word for word.
- `release/notes.md`: line 1 says each ISO boots "to MangoWM", and that the early installer ("Install Stackific OS", in the app launcher and the boot menu) puts Stackific OS on a drive that cannot start yet, so it is for a spare drive; the live session itself still writes nothing to your disks. The table becomes `| ISO | First desktop | Second desktop |` with `MangoWM` in place of the base labels; the Fedora notice and Red Hat's trademark line go, and the non-affiliation sentence stays as one `<sub>…</sub>` paragraph, word for word (as on the test build `v0.0.0-pr.11`). Published releases keep their notes (a released version never changes).
- `site/src/content/docs/index.md` line 8: "Every image runs MangoWM, plus KDE Plasma on …" (the base labels stay).
- `site/src/content/docs/user/installation.md`: lines 8 and 57 say "to MangoWM" and "starts MangoWM"; "Limits of this preview" says that builds after 0.1.0 have an early installer ("Install Stackific OS" in the app launcher and in the boot menu) whose installed drive cannot start yet, so it is for a spare drive only; and that Disks (in the app launcher) can make free space or set up partitions first, while shrinking Windows itself is safest from Windows' own Disk Management.
- `site/src/content/docs/user/wallpapers.md` line 12: "Open the wallpaper panel from the desktop's bar or control center", and the choice "is kept in `~/.local/state/noctalia/settings.toml`". Line 14 keeps its paths.

Docs and rules:
- `docs/installer.md` (new): what the installer does, the design table, the screens, the config file, the `Base` interface, the launcher and the console path, how to build and test it, and what tasks 013, 014, 015, 017 and 018 add.
- `docs/checks.md`: the installer's pre-commit checks and the install VM test. `docs/arch-iso.md`: the installer, its launcher and the Install boot entry in the image, and the ISO size this task measures.
- `AGENTS.md` (line 18, the trademark rule, is already updated in this branch), one line each, concise: the installer (`installer/`, Rust with ratatui, package `stackific-installer`, the `Base` trait, unattended TOML, free space or whole drive, its launcher and boot-menu entry, the Display screen first, the fixed centered 80 × 24 frame, the design table in `docs/installer.md`); the subvolume layout and mount options under "Boot and disk"; GNOME Disks as the live images' partition editor, and why; Rust 1.99.1 and rustup 1.29.3 in the pins; `rust-analyzer-lsp` under LSP plugins; and "What users read (README, release notes, site user docs) names the desktop MangoWM; Noctalia appears there only in paths and settings; the prompt trail is exempt."
- `site/src/content/docs/prompt-trail/012-installer-whole-drive.md` (new, from `/s-build`): the entry, with real screenshots of the screens on the desktop and on the console (`site/src/assets/prompt-trail/012/`, from `just arch screenshots` through `just site screenshot`), not text (operator 2026-11-06).
- `dev/reviews/012-installer-whole-drive.md` (new, from `/s-build`): the review record.

## Acceptance criteria

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
- `just arch test-install` passes, in one boot of the ISO with one 64 GiB disk, and its log shows each of these:
  0. The console path: `systemctl isolate stackific-installer.target` leaves `stackific-installer-console.service` active and the greeter stopped, a screenshot `installer-console.png` of tty1 is saved, and after `sendkey esc` twice the service has ended and `graphical.target` is active again.
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
- `just arch screenshots` drives the real installer in a VM and leaves a PNG of each screen in `.build/arch/x86_64/screens/`; the prompt-trail entry shows them as WebP (`just site screenshot`), on the desktop (from the launcher: Display to Done, the stop dialog, Stopped, a terminal too small) and on the console (Display, Keyboard, Done), and no screen as text.
- `git grep -n -i noctalia -- README.md release/notes.md site/src/content/docs/index.md site/src/content/docs/user` prints only `wallpapers.md` lines that name a `noctalia` path. `README.md` and `release/notes.md` contain no `Built from`, no `## Trademarks`, no `Fedora Remix` and no `Red Hat, Inc.`, and each ends with the non-affiliation sentence inside `<sub>`; `git grep -n -i fedora -- README.md release/notes.md` prints only ISO file names and that sentence. `just site build` passes.

Should (the Design table's soft rows; drop one that is close to impossible and record which and why in `docs/installer.md` and the review record):
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

## Tests
- Unit, navigation and snapshot tests: `just installer test` (pre-commit, and in `check` through `just lefthook all`). Format and lint: `just installer lint`.
- Package and image: `just arch packages`, then `just arch iso` (size printed), then `just arch test` on the PC.
- Install: `just arch test-install` on the PC; in CI, the `arch-x86_64` jobs of `images` and `release`.
- Tooling: `just setup`, then `just doctor`, on the PC; the Mac's `just setup` covers macOS and, in the runner VM, Linux aarch64.
- Messaging: the `git grep` above, then `just site build`.
- Design review: `just installer demo`, then `just installer demo --slow`, on the PC or the Mac, in any terminal. No root, no disk.
- Screenshots for the prompt trail: `just arch screenshots` on the PC, then `just site screenshot` for each PNG in `.build/arch/x86_64/screens/`.
- By hand (operator), to try the screens and the launcher:
  1. Download the test build's Arch x86_64 ISO after this PR merges (release `v0.0.0-pr.<n>`).
  2. In a VM manager such as GNOME Boxes or virt-manager, make a UEFI VM with 4 GiB of memory, the ISO as its CD and an empty 64 GiB disk.
  3. Boot it. In MangoWM, press `Alt+space`, type "install" and open "Install Stackific OS"; press Enter at the password prompt (`live`'s password is empty).
  4. Search "germ" on the keyboard screen and "berl" on the timezone screen, fill in the user, pick the 64 GiB disk, choose to erase it and type its name to confirm.
  5. Check that the Done screen appears and the terminal closes on Enter. Then resize or maximise the window during a second run, and press Ctrl and + a few times, and check that the frame stays centered at the same size and readable.
  6. Restart the VM and pick "Install Stackific OS" in the boot menu. Check that Display comes up on the console, that ←→ on Text size changes the text size at once, and that Esc twice starts the live desktop.
  7. If you have a high-DPI laptop (such as a 13-inch 2560 × 1600 panel), repeat steps 3 and 6 on it from a USB stick, without installing, and check that both paths are readable from the first screen.
  8. In the live desktop, open Disks from the app launcher, create a partition on the empty disk and check that the password prompt appears (Enter, since `live`'s password is empty). Then go back to the installer's Drive screen and check that it shows the change.
  9. Log out, pick the Plasma session at the login screen, and repeat step 8 there: Disks opens from Plasma's menu with the Breeze look, and Plasma's own password prompt appears. Also open "Install Stackific OS" from Plasma's menu and check that it starts.
  10. Report anything that looks different from the screens above.
