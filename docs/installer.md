# Installer

`stackific-installer` installs Stackific OS from a live session into a drive's free space, next to what is there, or onto a whole drive, which it erases. It is one Rust program for both bases, with a [ratatui](https://ratatui.rs) text interface and an unattended mode that reads the same answers from a TOML file. Its crate is `installer/`; the Arch package is `arch/packages/stackific-installer/`. Today it ships on the Arch x86_64 image only (task 012).

The installed drive does not start on its own yet: Limine comes with task 013. Use it on a spare drive.

## Build, test, try

```sh
just installer test           # unit, navigation and snapshot tests (pre-commit, and the check workflow)
just installer lint           # cargo fmt --check, then clippy with warnings as errors
just installer demo           # every screen and a simulated install: no root, no disk
just installer demo --slow    # all motion ten times slower, to review it frame by frame
just installer demo --console # the console's screens (ASCII markers, Reduced motion)
just installer build          # a debug build in installer/target/
```

The recipes use the pinned toolchain from `just rust install` (Rust 1.99.1, rustup 1.29.3, in `.tools/rust/`), which `just setup` installs; `just doctor` prints its `rust:` line. The images build with their base's own `rust` package, the same release.

The image tests are in [arch-iso.md](arch-iso.md): `just arch test` checks the launcher, Disks and Plasma's GTK integration in the live session, and `just arch test-install` runs real installs in a VM.

`just arch screenshots` (not in CI) boots the ISO with a blank 64 GiB disk and drives the real installer by key presses (`tests/vm/screens.sh`): from MangoWM's launcher through a free-space install, a terminal too small and a stopped erase, then on the console from the Install target through an erase. It saves a PNG of each screen in `.build/arch/<arch>/screens/`; `just site screenshot` turns them into the prompt trail's WebP files. Before each desktop shot it switches to workspace 2 and back (Ctrl+2, Ctrl+1), so MangoWM repaints the whole screen: on the VM's software renderer it can leave stale regions after small redraws. Then it waits until two dumps 2 s apart match.

## What it does

1. **Checks**, before any screen (exit 2 with one line on stderr): running as root, booted in UEFI mode, a known base (os-release's `ID_LIKE`), and that base's live root mounted.
2. **Answers**, from the screens or from `--config <file>`; both paths run the same checks (below).
3. **Disk step**, shared by both bases:
   - Erase: `sfdisk --wipe always --wipe-partitions always` with a new GPT: partition 1, 4 GiB, EFI System, named `Stackific OS boot`, at 1 MiB; partition 2, Linux filesystem, named `Stackific OS`, to the last usable sector rounded down to 1 MiB. The script gives `first-lba:` and every start and size in sectors (the drive's logical sector size, from lsblk's `LOG-SEC`), because sfdisk aligns only values it picks itself, and some drives report an I/O size that would move its defaults.
   - Free space: `sfdisk --append --wipe never --wipe-partitions always` with the same two partitions at explicit sectors inside the largest 1 MiB-aligned gap, in one write. Every existing entry stays byte-identical.
   - The Confirm screen shows these exact places (`disk::layout`), and the install writes them.
   - `udevadm settle`, then the new partitions' device paths from `lsblk` by their start (never predicted from numbers).
   - `mkfs.fat -F 32 -n STACKIFIC` on the ESP and `mkfs.btrfs -f -L stackific` on the system partition (labels from os-release's `ID`).
   - The subvolumes, flat siblings of `@`, all mounted with `noatime,compress=zstd:1`; `@libvirt` and `@swap` get `chattr +C` (NOCOW):

     | Subvolume     | Mount point               |
     | ------------- | ------------------------- |
     | `@`           | `/`                       |
     | `@home`       | `/home`                   |
     | `@root`       | `/root`                   |
     | `@srv`        | `/srv`                    |
     | `@cache`      | `/var/cache`              |
     | `@tmp`        | `/var/tmp`                |
     | `@log`        | `/var/log`                |
     | `@swap`       | `/swap` (empty for now)   |
     | `@flatpak`    | `/var/lib/flatpak`        |
     | `@libvirt`    | `/var/lib/libvirt/images` |
     | `@containers` | `/var/lib/containers`     |
     | `@docker`     | `/var/lib/docker`         |

   - `@` at `/run/stackific-installer/target`, the other subvolumes under it, and the ESP at its `/boot` (`umask=0077`).
4. **Copy**: `rsync -aHAXx --numeric-ids --info=progress2 --no-inc-recursive --exclude=/boot/` from the live image's pristine root (Arch: `/run/archiso/airootfs`, the squashfs under the live overlay). `--no-inc-recursive` makes the overall percentage count the whole copy, so it only grows. `/boot` is empty in the live root and is the FAT ESP in the target, which takes no owners, ACLs or xattrs.
5. **System**: `/etc/fstab` by UUID (twelve subvolume lines and `/boot`); `systemd-firstboot --root --force` for the timezone, the computer name and the console keymap (the first `kbd-model-map` line for the XKB layout with no variant, `us` when none); `/etc/X11/xorg.conf.d/00-keyboard.conf` in the format `localectl set-x11-keymap` writes.
6. **User**: `userdel --root … --remove live` first, so ours gets uid 1000; `useradd --root … --create-home --groups wheel`; the password through `chpasswd --root …` on stdin (yescrypt on Arch); `usermod --root … --lock root`; and `~/.config/mango/config.conf` with `source=/etc/mango/config.conf` and `xkb_rules_layout=<layout>`, so MangoWM's stock config stays as packaged with our layout on top.
7. **Live state**: `/etc/sysusers.d/stackific-live.conf` removed, and greetd's `[initial_session]` (the autologin) removed with every other line kept; then the base's own step. Arch removes `/etc/mkinitcpio.conf.d/archiso.conf` and runs `pacman --sysroot <target> -Rns stackific-installer`, so the installed system has no installer and no launcher. `/etc/machine-id` stays `uninitialized` (first boot), and the locale stays the live image's.
8. **End**: `umount --recursive` of the target, whatever happened.

Each command and its output go to `/run/stackific-installer.log` on the screens and to stderr when unattended. stdin, which carries the password, is never logged.

Exit status: 0 installed; 1 a step failed after the confirmation; 2 refused or invalid answers (nothing written); 3 left before the confirmation (nothing written); 130 stopped by the user.

### What it refuses

- A drive that is not a whole disk (`lsblk` type `disk`, not zram), is read-only, holds the live medium (found from `/run/archiso/bootmnt`), is under 40 GiB, or has anything mounted or used as swap.
- Free space unless the GPT is clean (`sfdisk --verify` says `No errors detected.` and warns about nothing), the protective MBR has only its `ee` entry (no hybrid MBR), no partition is BitLocker (`blkid -p`), and the largest aligned gap is at least 40 GiB. Such a drive can still be erased.
- A user name that is not `^[a-z_][a-z0-9_-]{0,31}$` or that the live root already has (`live` excepted), a computer name that is not `^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$`, an empty password, an unknown XKB layout or timezone, and an erase whose confirmation is not the drive's kernel name.

## The unattended file

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

Run it with `stackific-installer --config <file>` as root. Unknown keys are refused.

## Where it runs

- **Desktop shortcut**: on Plasma's desktop, a link in `live`'s `~/Desktop` (`arch/iso/airootfs/home/live/Desktop/`) to the launcher below. Plasma trusts it because its real path is in the system's applications. The install removes it with `/home/live`; MangoWM has no desktop icons, and GNOME (task 014) gets the installer in its dash instead.
- **Launcher**: `/usr/share/applications/stackific-installer.desktop` ("Install Stackific OS"), rendered from `installer/stackific-installer.desktop.in` and `branding/branding.env`. It opens foot at 11 points with `dpi-aware=yes`, so the text has the same physical size on every monitor, and runs `run0 stackific-installer`. run0 asks polkit; `live`'s password is empty, so Enter. One terminal for both desktops: foot is on the image, and MangoWM's launcher and Plasma's menu both list the file.
- **Boot menu**: "Install Stackific OS" (`arch/iso/efiboot/loader/entries/02-stackific-install.conf`) next to "Try Stackific OS", which stays the default. It adds `systemd.unit=stackific-installer.target`, which boots `multi-user.target` without the greeter and starts `stackific-installer-console.service`: `stackific-installer --console` as root on tty1 (`TTYPath=/dev/tty1`, getty conflicts with it). However it ends, `ExecStopPost` isolates `graphical.target`, so the live desktop starts. A target and a service are the glue because no packaged tool starts a program on tty1 from a boot entry. Leaving (3) and stopping (130) count as success for the unit. On aarch64, `just arch iso` leaves this entry out until the installer joins that image (task 018).

## The Base interface

`installer/src/base.rs` holds what differs per base, behind the `Base` trait: the live root (Arch: `/run/archiso/airootfs`), the live medium's mount point (Arch: `/run/archiso/bootmnt`), and the base's own live state (files to remove, commands to run). The installer picks the base from os-release's `ID_LIKE`. Task 013 adds the boot step (initramfs and Limine), task 014 a Fedora implementation and its packaging.

Every external command and file goes through the `Runner` trait (`installer/src/run.rs`): `Real` on a live system, `Fake` in the tests and the demo, which answers from rules and an in-memory file table and records any command it was not told to expect.

## The screens

Display, Welcome, then six steps: Keyboard, Timezone, You, Drive, Space, Confirm; then Installing, and Done or Stopped. The frame is exactly 80 × 24, centered, so it reads the same on an ultrawide monitor, a 13-inch laptop or a full-screen console; a smaller terminal shows only "make the window bigger or the text smaller". Every screen's text is a fixture in `installer/fixtures/screens/`, which the snapshot tests compare with what ratatui renders; the prompt-trail entry shows them.

- **Display** comes first. On the console, Text size is the console font (`setfont`): the first pick goes by the framebuffer's height (`/sys/class/graphics/fb0/virtual_size`: under 1000 pixels Small, 8 × 16; under 1500 Medium, 12 × 22; under 2200 Large, 16 × 32; else Extra large, 24 × 32), lowered to a size that leaves at least 80 × 24 characters, and ←→ sets another at once. In foot, Text size names foot's own Ctrl and + and −. Motion is Full or Reduced (the console's default), Contrast Standard or High.
- **Keyboard** and **Timezone** share one type-to-search list: the layouts from evdev.lst by code and name, the zones from `timedatectl list-timezones`, ignoring case; an exact code first, then shorter names; an empty search starts with `us` or `UTC`.
- **You**: the user name follows the name's first word until edited; passwords show as dots; the first problem shows under the fields as you type, and "Enter next" appears once every field is valid.
- **Drive** lists whole disks with their largest free space, and each left-out drive with its reason. It reads the drives again each time it shows and when the terminal regains focus, so changes made in Disks show up.
- **Space**: "Use the free space" first when usable, otherwise its line says why; a drive with no partition table offers only the erase.
- **Confirm** shows what is on the drive now and afterwards, and the answers. Free space installs on Enter; an erase needs the drive's kernel name typed first.
- **Installing** lists the steps and the copy's progress bar; Ctrl+C asks first (the stop dialog), and Enter there stops the install.

## Design

The screens follow a design guide distilled from Apple's WWDC talks (*Designing Fluid Interfaces*, *The Details of UI Typography*, *Principles of Great Design*), adapted to a terminal. Its rows are soft requirements; every spacing, colour and timing is a named constant in `installer/src/ui/theme.rs`, with a comment saying why.

| The guide | In the installer |
| --- | --- |
| Response | Every key and mouse event is handled and redrawn in the same frame, with no debounce. A search filters on each keystroke. The install runs on its own thread. |
| Direct manipulation | In foot, a press highlights a row or a bottom-border action, the release commits it, and moving off first cancels. The wheel moves a list. The console has no mouse. |
| Interruptibility, velocity handoff | One spring model (`ui/motion.rs`); a retarget keeps position and velocity, so Esc mid-slide reverses without a jump. |
| Springs | Critically damped (no bounce): slides 0.3 s, the selection marker 0.15 s, the progress bar 0.4 s (never backwards), the rubber band 0.2 s. Up to 60 frames per second, and none while nothing moves. |
| Momentum | A held ↑ or ↓ moves three rows per repeat after half a second. A wheel flick carries on by the guide's projection (deceleration rate 0.99). |
| Spatial consistency, hint the direction | Forward slides the next screen in from the right, Back from the left. The marker glides toward the row it goes to. The stop dialog grows out of the bottom border's "Ctrl+C stop" and shrinks back into it. |
| Rubber-banding | Past a list's edge, a press nudges the list one row and springs it back. |
| Smoothness | ratatui redraws only changed cells; motion moves at most one cell per frame; the bar moves in eighths of a cell (halves on the console). Slides cover six columns, so the direction shows without the text racing past. |
| Materials and depth | In a true-colour terminal (`COLORTERM=truecolor`, or `TERM=foot`, which run0 passes on while it drops `COLORTERM`): the frame's surface is the accent at 6 % over foot's default background, the bottom border a heavier band, the dialog sits on a dim scrim with a one-cell shadow, and a scrolled list fades its edge rows. None of it with High contrast or `NO_COLOR`. |
| Sound | One bell, on the frame that shows Stopped. |
| Reduced motion, contrast, colour | Reduced: slides, glides, the band and the dialog's growth are instant; the bar still moves. High contrast: no dim text, no tints, bold and reverse video. `NO_COLOR` removes all colour. A selection is reverse video, never colour alone. On the console (`TERM=linux`) the markers become ASCII and the arrows words. |
| Typography | The system's monospace font; titles bold, body regular, help dim; prose wraps at 75 columns with no one-letter word at a line end; sizes right-aligned with one decimal and a unit. |
| Purpose, agency, responsibility | No manual partitioning, no language screen. Confirmations only where data is at stake: the typed name before an erase, Enter on the free-space summary, the stop dialog. A before-and-after summary, refusals that say why, and a password never echoed or logged. |
| Familiarity, flexibility | Arrows, Tab and Shift+Tab, Enter, Esc, Home, End, Page Up and Down, and the mouse; the same key means the same everywhere. The desktop or the console; text size, motion and contrast. |
| Wayfinding | The top border names the program and "n of 6 · Step"; the bottom border lists only the keys that work, by what they do. Esc goes back, and on the first two screens leaves (on the console, to the live desktop). Ctrl+C leaves at once before the install and asks during it. |
| Process | `just installer demo`, and `--slow`. |

Dropped or narrowed from the guide, and why:

- The slides cover six columns, not a whole screen: a full 80-column slide in 0.3 s moves several columns per frame, which reads as a jump in a terminal. The short slide keeps "at most one cell per frame".
- The terminal's own background colour is assumed (foot's default) rather than read: reading it needs an escape-sequence round trip that would delay the first frame.
- Haptics: none in a terminal.

## Later

- Task 013: Limine on the ESP, the initramfs, and the Done screen's text once the drive starts.
- Task 014: the Fedora `Base`, its RPM and the Fedora images' launcher and boot entry; the crate stays unchanged.
- Task 015: BitLocker with a typed confirmation and protector checks, OS detection, refusals of Intel RST RAID and BIOS-mode Windows, and the Windows entry in Limine.
- Task 017: the Fedora images' Install boot entry.
- Task 018: the installer on both aarch64 images (and the aarch64 boot entry `just arch iso` leaves out today).
