# Arch ISO

The Arch live ISO is built with [archiso](https://gitlab.archlinux.org/archlinux/archiso) inside a pinned builder container, then boot-tested in the same container under QEMU with UEFI firmware. The host's architecture picks the image: x86_64 on a Linux PC, and aarch64 from Arch Linux ARM in the Mac's runner VM (docs/setup.md, "Self-hosted runners"). Nothing is cross-built. Both steps need Linux with rootless podman and read-write `/dev/kvm` and `/dev/vhost-vsock`. `just doctor` checks them on its `arch:` line.

## Build

```sh
just arch iso                 # version 0.0.0-dev
just arch iso 1.2.3           # stackific-1.2.3-arch-<arch>.iso
```

The version must be semver, as for a release; anything else is refused. The recipe builds the builder image first (`localhost/stackific-arch-builder:<date>`, from `arch/Containerfile`), then our packages (below), then runs `mkarchiso` on the profile in `arch/iso/`. Outputs go to `.build/arch/<arch>/`:

| File                                           | What it is                                          |
| ---------------------------------------------- | --------------------------------------------------- |
| `stackific-<version>-arch-<arch>.iso`          | The live ISO: UEFI only, systemd-boot               |
| `stackific-<version>-arch-<arch>.packages.txt` | The package manifest (`pacman -Q` of the live root) |

The last line prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail. The x86_64 ISO was 1543 MiB on 2026-11-06, with the installer, Disks and Plasma's GTK integration (1497 MiB before them).

`mkarchiso` runs as the unprivileged user `build` inside the container. Its work directory stays in the container, so each build starts fresh. pacman's package cache is not: `.build/arch/<arch>/pkgcache/` is mounted at `/var/cache/pacman/pkg`, so a build downloads only packages the last one did not, and the profile's `pacman.conf` sets `DisableDownloadTimeout`, because archive.archlinux.org sometimes stalls below pacman's 10-second low-speed limit and failed whole builds (`Operation too slow`, 2026-11-08). Delete the folder to start empty; on aarch64, whose live repos move, it grows until then. Rootless podman cannot mount devtmpfs, which archiso needs when it runs as root.

## Our packages

```sh
just arch packages            # also the first step of just arch iso
```

Our own Arch packages live in `arch/packages/<name>/PKGBUILD`. Their files are base-neutral and live in `desktop/` and `branding/`, so Fedora's packages can ship the same files later; an Arch-only file (`stackific-boot`'s pacman hook) sits next to its PKGBUILD. The recipe runs a fresh builder container. For each PKGBUILD, it copies the PKGBUILD and the files its `source=()` names (matched by file name, from the package's own folder, `desktop/` and `branding/`) into a new directory. It runs `makepkg` there as `build`, then `repo-add` puts every package into the local repo `.build/arch/repo/`.

The ISO build mounts that repo read-only at `/repo`, and `arch/iso/pacman.conf` reads it as `[stackific]`. It is unsigned (`SigLevel = Optional`) until release signing lands.

### The branding file

`branding/branding.env` is the one branding file: the product name, the ID, the four URLs and the accent colour, as shell `KEY="value"` lines. Every package file that names the product or uses its colour is rendered from it in the PKGBUILD's `build()`:

- the PKGBUILD sources `branding.env` (`set -a`, so the keys are exported);
- it runs `envsubst` on a base-neutral template (`*.in` in `branding/`) with an explicit list of keys, such as `envsubst '$ACCENT'`, so no other `$` in the template changes;
- values derived from a key are computed there too: `ANSI_COLOR` is the accent as a 24-bit terminal colour (`38;2;R;G;B`), and the icon is `logo.svg` with `currentColor` replaced by the accent.

A branding edit changes `branding.env`'s sha256, so update it, and bump `pkgrel`, in every PKGBUILD that sources it (`stackific-release`, `stackific-artwork`).

### The packages

`stackific-desktop` holds seven files and the greeter's asset folder:

- The Secret Service (task 017): `stackific-desktop` depends on `oo7` and links `/usr/lib/systemd/user/default.target.wants/oo7-daemon.service`, so every user's systemd manager starts oo7-daemon (Arch's package ships the unit unenabled), in MangoWM, Plasma and GNOME alike, as Fedora 45 chose. oo7 owns `org.freedesktop.secrets`; KDE apps keep the KWallet API through kwalletd6, but kwallet's `ksecretd` no longer gets the name. `/etc/pam.d/stackific-greetd` is greetd's own PAM lines plus `-auth optional pam_oo7.so` and `-session optional pam_oo7.so auto_start`, so the login password unlocks (or creates) the user's keyring in `~/.local/share/keyrings/` (oo7 0.7: `keyrings/v1/`); the image's `/etc/greetd/config.toml` selects it with `[general] service = "stackific-greetd"` (greetd 0.10's PAM service for user sessions), because greetd's own `/etc/pam.d/greetd` belongs to its package. The live user `live` logs in with no password, so its keyring is not unlocked at login. Changing the login password later (`passwd`) does not change the keyring's.

- `stackific-noctalia.service` (`desktop/systemd/user/`), a user unit that starts Noctalia, wanted by `mango-session.target` for every user. MangoWM starts that target itself once it has imported `WAYLAND_DISPLAY` into the user manager. That is how Noctalia starts while MangoWM's own config stays as packaged. Plasma never starts that target, so Noctalia stays out of Plasma.
- `/etc/profile.d/stackific-wlroots.sh` (`desktop/profile.d/`), which exports `WLR_RENDERER_ALLOW_SOFTWARE=1`. With no working GPU driver (a VM without 3D, the boot test's included), Mesa renders in software, and wlroots refuses that without the variable. MangoWM's renderer is GLES-only, so MangoWM would exit. greetd sources `/etc/profile`, so the variable reaches every session it starts.
- `/usr/share/stackific/noctalia.toml` (`desktop/noctalia/stackific-noctalia.toml`), our Noctalia defaults: no first-run setup wizard, the wallpaper picker browsing `/usr/share/backgrounds/stackific`, and Dahl's "An Eruption of Vesuvius" as the default wallpaper.
- `/etc/skel/.config/noctalia/00-stackific.toml` (`desktop/noctalia/`), which only includes the defaults above. Noctalia reads config only from the user's `~/.config/noctalia/` (no `/etc` or `/usr/share` layer), so each user gets this one-line file from `/etc/skel`, and package updates to the defaults reach them. It sorts before the user's own files, which win, and the settings Noctalia's UI writes (`~/.local/state/noctalia/settings.toml`) win over both. Users created before the package was installed have no copy; `cp /etc/skel/.config/noctalia/00-stackific.toml ~/.config/noctalia/` adds it.
- `/usr/share/stackific/greeter.toml`, rendered from `desktop/noctalia/stackific-greeter.toml.in` with the accent: noctalia-greeter's administrator config, with a dark palette in our emerald and the same painting as the desktop.
- `/usr/lib/tmpfiles.d/stackific-greeter.conf` (`desktop/tmpfiles.d/`), which links `/var/lib/noctalia-greeter/greeter.toml`, the file the greeter reads, to the one above. Its `L` line creates the link only when that file is missing, so an administrator's own `greeter.toml` is never replaced.
- `/etc/profile.d/stackific-greeter.sh` (`desktop/profile.d/`), which exports `NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets`, the greeter's documented override of its asset folder. The greeter has no key for another logo, but takes `noctalia.svg` and `fonts/` from that folder. Ours holds two links: `noctalia.svg` to `stackific-release`'s emerald icon, and `fonts` to `/usr/share/noctalia-greeter/assets/fonts`. greetd sources `/etc/profile` before the greeter, so our logo sits where Noctalia's was, with no package file changed.

For Plasma, Arch's second desktop, `stackific-desktop` also holds our wallpaper as a Plasma wallpaper package (`/usr/share/wallpapers/stackific/`, its image a link to `stackific-artwork`'s painting), our look-and-feel package `org.stackific.desktop` (`/usr/share/plasma/look-and-feel/`: Breeze's defaults with `[Wallpaper] Image=stackific`; everything else falls back to Breeze), and `/etc/xdg/kdeglobals`, which selects it for every user. Plasma's wallpaper plugin takes its default from the selected look and feel. The templates are `desktop/plasma/*.in`, rendered from the branding file.

`stackific-hardware` (sources in `hardware/`) holds the NVIDIA driver pick: a systemd generator, NVIDIA's legacy device list and MangoWM's low-effects include. The x86_64 image installs it with `linux-lts`, `nvidia-open`, `nvidia-open-lts` and `nvidia-utils` (task 019), the aarch64 image with `nvidia-open-dkms`, `nvidia-utils` and `linux-aarch64-headers` (task 022). How it works: [hardware.md](hardware.md).

Both images also carry the regular-hardware set of task 021 (on aarch64 task 022) ([hardware.md](hardware.md)): `linux-firmware-marvell` (without Marvell's Prestera switch firmware, kept out by `NoExtract` in `arch/iso/pacman.conf`), `sof-firmware`, `alsa-firmware`, `wireless-regdb`, `bolt`, `fprintd`, and libcamera with PipeWire's libcamera and V4L2 plugins.

`stackific-artwork` holds the wallpapers and fastfetch's logo, and depends on `fastfetch`, so installing our branding installs fastfetch:

- `/usr/share/backgrounds/stackific/*.jpg`: the ten public-domain paintings in `branding/wallpapers/`, credited in `branding/wallpapers.toml`, which the package installs as its licence file (`/usr/share/licenses/stackific-artwork/wallpapers.toml`);
- `/usr/share/stackific/logo.txt`: the block-character text logo, `$1` marking where fastfetch puts the colour;
- `/etc/xdg/fastfetch/config.jsonc`, rendered from `branding/fastfetch.jsonc.in`: the logo file in the accent, with fastfetch's default modules. fastfetch reads the first `fastfetch/config.jsonc` it finds in `~/.config/`, `$XDG_CONFIG_DIRS`, `/etc/xdg/` and `/etc/`, so a user's own config replaces ours.

`stackific-installer` is the installer ([installer.md](installer.md)), built from the Rust crate in `installer/` with Arch's `rust`, which the builder carries because makepkg runs with `--nodeps`. The recipe packs the crate (`Cargo.toml`, `Cargo.lock`, `src/`, `fixtures/`, never `target/`) into `stackific-installer-crate.tar` next to the PKGBUILD, which lists it with the checksum `SKIP`: makepkg takes no directories, and a checksum per file would change with every code change. `prepare()` fetches the locked crates, `build()` builds and renders the launcher, `check()` runs the tests. It installs `/usr/bin/stackific-installer`, the launcher `/usr/share/applications/stackific-installer.desktop`, and `stackific-installer.target` with `stackific-installer-console.service` for the boot menu's Install entry. It is on the x86_64 image only for now. It depends on `arch-install-scripts` (`arch-chroot`), `efibootmgr` and `limine` for the boot step.

`stackific-boot` keeps Limine on an installed system's ESP current: it depends on `limine` and `efibootmgr`, and its pacman hook (`stackific-limine.hook`) copies `/usr/share/limine/BOOTX64.EFI` (on aarch64 `BOOTAA64.EFI`) to `/boot/EFI/limine/` and `/boot/EFI/BOOT/` after every upgrade or reinstall of `limine`, and does nothing in the live session, which has no `/boot/EFI/limine/`. The image lists it explicitly, so the installer's `pacman -Rns stackific-installer` keeps it and its dependencies.

`stackific-release` makes the image identify as Stackific OS. It holds two files:

- `/etc/os-release`: `branding/os-release.in` rendered from the branding file (the base-neutral fields: `ID=stackific`, `NAME` and `PRETTY_NAME` "Stackific OS", the site and GitHub issues URLs, `DEFAULT_HOSTNAME=stackific`, `LOGO=stackific` and the emerald `ANSI_COLOR`), followed by Arch's per-base lines `ID_LIKE=arch` and `BUILD_ID=rolling`. The base is rolling, so there is no `VERSION` or `VERSION_ID`, as os-release(5) allows. The template has no comment lines, because some os-release readers split every line on `=`.
- `/usr/share/icons/hicolor/scalable/apps/stackific.svg`, the icon that `LOGO=stackific` names: `branding/logo.svg` with its `currentColor` fill replaced by the accent at build time. `currentColor` alone renders black, invisible on a dark panel.

Arch's `filesystem` package owns `/usr/lib/os-release` and `/etc/arch-release`, and both stay as shipped. os-release(5) readers use `/etc/os-release` when it exists and never merge it with `/usr/lib/os-release`, so owning the `/etc` file is enough, with no hook, patch or fork. No Arch package owns `/etc/os-release`: systemd's tmpfiles line `L /etc/os-release` only makes it a symlink when it is missing, and runs after the transaction, so pacman's conflict check never sees it. That holds because mkarchiso installs every package in one pacstrap call. Installing the package later onto a system that already has the symlink needs `pacman --overwrite /etc/os-release`. It is not in `backup=()`: it is vendor data, like Arch's own `/usr/lib/os-release`, so an upgrade replaces it.

mkarchiso appends `IMAGE_ID=stackific` and `IMAGE_VERSION=<version>-arch` to the file `/etc/os-release` resolves to, which is now ours. That is how the ISO's semver version reaches the live image.

To change a file in `desktop/` or `branding/`, update its `sha256sums` entry in the PKGBUILD and bump `pkgrel` (or `pkgver`).

## Live session

The ISO boots to a desktop. greetd (`arch/iso/airootfs/etc/greetd/config.toml`) logs the user `live` into MangoWM with no prompt, and MangoWM starts Noctalia, which shows Dahl's "An Eruption of Vesuvius" and no setup wizard (our defaults, above). `fastfetch` in a terminal prints our logo in emerald. In MangoWM's stock config, `Alt+Return` opens foot and `Alt+space` opens rofi; `live`'s own `~/.config/mango/config.conf` (`arch/iso/airootfs/home/live/`) sources the stock config and adds `Super+Space` for Noctalia's launcher, as the installer does for installed users.

The app launcher has "Install Stackific OS" (the installer in foot, through run0), which Plasma's desktop also shows as a shortcut (`arch/iso/airootfs/home/live/Desktop/`, a link to the launcher, which mkarchiso gives to `live` with `/etc/skel`; the install removes it with `/home/live`), and "Disks" (GNOME Disks, `gnome-disk-utility`), the partition editor, with `ntfs-3g` and `ntfsprogs` for NTFS. Disks runs as the user and asks polkit for each change: in MangoWM, Noctalia's own polkit agent answers (`polkit_agent = true` in our defaults), in Plasma `polkit-kde-agent`. `breeze-gtk` and `kde-gtk-config` give GTK apps such as Disks Plasma's look. The boot menu has "Try Stackific OS" (the default) and "Install Stackific OS", which boots to the installer on tty1 without the greeter.

Known, for later: in a VM without 3D (software rendering), MangoWM can leave stale regions on screen after small redraws, such as typing in a terminal or in rofi; what the apps draw is right (task 012's screenshot run). `WLR_SCENE_DEBUG_DAMAGE=rerender` (wlroots' scene setting, which scenefx keeps) re-renders every frame, but slowed the 2-CPU test VM until key presses arrived seconds late; `just arch screenshots` switches workspace away and back before each shot instead. A later task finds a fix for the live image.

After a logout, greetd shows noctalia-greeter. There, KDE Plasma is the other session. `live` has an empty password and is in `wheel`, so polkit treats it as an admin (with its empty password). root also has an empty password, on the other consoles. These accounts exist only in the live image: `arch/iso/airootfs/etc/{passwd,group,shadow,gshadow}` and `etc/sysusers.d/stackific-live.conf`. mkarchiso creates `/home/live` from `/etc/skel`.

## Boot test

```sh
just arch test                # boots stackific-0.0.0-dev-arch-<arch>.iso
just arch test 1.2.3
```

`tests/vm/boot.sh` boots the ISO under QEMU with KVM: on x86_64 a q35 machine with OVMF, on aarch64 a `virt` machine with AAVMF and a `virtio-gpu-pci` display. It reaches the VM only through one-boot SMBIOS credentials, which systemd reads at boot:

- A `systemd.extra-unit.*` credential adds the test shell, `tests/vm/stackific-test-shell.service`, which listens on vsock through `systemd-socket-activate`. A `systemd.unit-dropin.multi-user.target` credential starts it. It is a service, not a socket unit, because Fedora's SELinux policy denies PID 1 such a socket (`docs/fedora-iso.md`).
- A random token arrives as the `stackific.test.token` credential. The shell runs `bash` on a connection only when its first line is that token.

Nothing from `tests/` is in the image, and the image has no sshd. The test waits for `systemctl is-system-running --wait` over the shell, then runs the checks every image passes:

- the system is `running`;
- NetworkManager is running;
- a wrong token gets no shell.

`boot.sh` then sources the image's check file, which `just arch test` passes as its third argument: `tests/vm/arch-<arch>.sh` when the architecture has one (none has since task 018), else `tests/vm/arch.sh`, the Arch checks both architectures pass. `arch.sh` checks:

- `/etc/os-release` reads `ID=stackific`, `ID_LIKE=arch`, `BUILD_ID=rolling`, `PRETTY_NAME="Stackific OS"`, `LOGO=stackific` and mkarchiso's `IMAGE_ID=stackific`; `hostnamectl` shows `Operating System: Stackific OS`; pacman says `stackific-release` owns `/etc/os-release` and `filesystem` owns `/etc/arch-release` and `/usr/lib/os-release`; `/usr/lib/os-release` has no `IMAGE_` lines; the `stackific` icon is filled `#007a55`;
- greetd is active, `stackific-noctalia.service` is active for `live`, and `mango` runs as `live`;
- the MangoWM and Plasma session files are present;
- the ten wallpapers are installed, `live`'s `00-stackific.toml` matches `/etc/skel`'s, and `noctalia config export merged` (as `live`) shows the wizard off, the wallpaper folder and the default wallpaper;
- `fastfetch --pipe false` (as `live`) prints the logo's last line in the accent colour;
- the installer's launcher has its name and `Exec=` line; Disks, its launcher and `ntfsresize` are installed; Breeze's GTK theme and `kde-gtk-config` are installed; `live`'s `~/Desktop` holds the installer's shortcut; then greetd logs `live` into Plasma, and the test saves `plasma.png`, the desktop with the shortcut.

It saves `screen.png` (the live desktop), and then logs `live` out with `loginctl terminate-user live`. It checks that noctalia-greeter runs as `greeter`, and saves `greeter.png`. It also saves `serial.log` to `.build/arch/<arch>/`, then powers the VM off. On failure it prints the end of the serial log.

Logging in to Plasma from the greeter is not automated yet. To check it by hand:

1. Boot the ISO in a VM with a display, UEFI firmware and enough RAM (see `-m` in `tests/vm/boot.sh`).
2. Press `Alt+Return` and run `loginctl terminate-user live`.
3. Pick `live` and the Plasma session, type any character as the password, and log in. `live` has no password, but noctalia-greeter refuses an empty field (`site/src/content/docs/user/installation.md`).
4. In Konsole, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.

systemd's `vmm.notify_socket` is not used. Over vsock it opens one connection per message and drops some of them, READY=1 included.

## Install test

```sh
just arch test-install        # uses stackific-0.0.0-dev-arch-<arch>.iso
just arch test-install 1.2.3
```

It reads the boot menu from the ISO with bsdtar (Try is the default, Install adds `systemd.unit=stackific-installer.target`), creates a blank 64 GiB raw image with `qemu-img` in `.build/arch/<arch>/`, and boots the ISO once with it attached as `/dev/vda` (`STACKIFIC_TEST_DISK`). `tests/vm/install.sh` then checks, in order:

0. the console path: `systemctl isolate stackific-installer.target` runs the installer on tty1 with the greeter stopped (`installer-console.png`), two Esc presses (`sendkey`) end it, and `graphical.target` is active again;
1. an erase whose confirmation is not the drive's name exits 2 and leaves the disk blank;
2. an unattended erase (a transient unit, polled until it ends) leaves exactly our two partitions;
3. a Windows-like table (ESP `SYSTEM`, Microsoft reserved, a 2 GiB gap, basic data, a large gap, recovery; numbered 1, 2, 3 and 5), saved as `windows-like.dump`;
4. BitLocker's signature on partition 3 (the boot code's jump and `-FVE-FS-` at byte 3) refuses the free space with exit 2 and the table unchanged;
5. a free-space install keeps the label ID and every old entry byte-identical, and puts two new partitions on the 1 MiB grid inside the large gap.

After runs 2 and 5 it checks the installed system on its partitions: labels, the twelve subvolumes and NOCOW, fstab by UUID, `stackific-release` in and `stackific-installer` out, the user (uid 1000, `wheel`, yescrypt, root locked, its MangoWM file), the computer name, timezone and keymaps, no live state, and no password in the journal. Each check also mounts the ESP: `vmlinuz-linux` (aarch64: `Image`), `initramfs-linux.img`, `limine.conf` with that install's root UUID, and Limine's program at `EFI/limine/` and `EFI/BOOT/`, both equal to the package's. Before installed_checks of run 5, the Windows-like ESP is still empty. After the VM powers off, `minfo` reads the free-space install's ESP from the image as FAT32.

Then the installed system boots from the disk alone (`boot.sh -`, no `-cdrom`), with the firmware variables the first boot left in `vars.fd`, so the entries the installs created are there. Before it, the recipe downloads the pinned `limine` package with `pacman -Sw` (the builder's mirror is the archive date; `--disable-sandbox`, as in rootless podman) and copies it to the ESP's root with `mcopy`. `boot.sh` saves `limine.png` a second after serial.log shows OVMF's `BdsDxe: starting Boot#### "Stackific OS"`, while Limine's menu waits its 3 seconds. `tests/vm/installed.sh` then checks:

- the firmware started the entry labelled with os-release's `NAME`, which is first in `BootOrder` and the only one with that label after both installs (the erase install's entry, on a partition the Windows-like table replaced, was deleted); the listing is saved as `efibootmgr.txt`;
- `/proc/cmdline` has `root=UUID=<root> rootflags=subvol=/@ rw`; `/` is btrfs on `subvol=/@`, `/boot` vfat, and the btrfs default subvolume is the top level (ID 5);
- the ESP holds the kernel, the initramfs (with systemd, no archiso), `limine.conf` and both copies of Limine's program, and no fallback image or unified kernel image; `uname -r` matches the one folder under `/usr/lib/modules/`;
- `stackific-boot`, `limine` and `efibootmgr` are installed, `stackific-installer` and `arch-install-scripts` are not; hostname, timezone, `localectl`'s keymaps and `wheel` as the installer set them; the greeter is up (`installed.png`);
- with both ESP copies emptied, `pacman -U` of the same `limine` package restores them through `stackific-boot`'s hook and leaves the firmware entries unchanged;
- greetd logs `tester` in through an initial session the test adds, first to MangoWM (`mmsg get keyboardlayout` prints `{"layout":"German"}`), then to Plasma (`org.kde.KeyboardLayouts.getLayoutsList` lists `de` first).

The image and the package are removed after a pass.

## aarch64

The aarch64 image is built and tested on aarch64 Linux: the `arch-aarch64` jobs of the `images` workflow (on merges to `main` and by hand) and of the `release` workflow run it on the Mac's runner VM, and upload it like the x86_64 images (`docs/releases.md`). It is the same desktop live ISO as x86_64, with the same branding, defaults, greeter and wallpapers. It shares the profile in `arch/iso/` and the checks in `tests/vm/arch.sh` with x86_64. Its package list, `packages.aarch64`, mirrors `packages.x86_64`: the same lines in the same order, except `linux-aarch64` for `linux`, `linux-aarch64-headers` for `linux-lts`, `archlinuxarm-keyring` for `amd-ucode` and `intel-ucode`, and `nvidia-open-dkms` for `nvidia-open` and `nvidia-open-lts`. A change to one list makes the same change to the other. Arch Linux ARM's rofi is older than Arch's (1.7.5 against 2.0.0 on 2026-11-05); MangoWM's `Alt+space` binding, `rofi -show drun`, works with both.

Where its parts come from:

- [Arch Linux ARM](https://archlinuxarm.org) has no archive, so the image comes from its live repos, and the package manifest is the record of what went in.
- The builder starts from `docker.io/menci/archlinuxarm`, pinned by digest in `arch/vars.just`. Arch Linux ARM publishes no container image; this one is built daily from its repos with pacstrap.
- Arch Linux ARM does not package archiso or the AArch64 UEFI firmware. The builder installs Arch's `any` packages `archiso` and `edk2-aarch64` from the archive date in `ala_date`, checked against Arch's signing keys. archiso v92 builds aarch64 natively, with `systemd-bootaa64.efi` as `BOOTAA64.EFI`.

What differs from x86_64:

- Arch Linux ARM's kernel is `/boot/Image`, but mkarchiso and the boot entry take `vmlinuz-*`. The pacman hook `arch/iso/airootfs/etc/pacman.d/hooks/stackific-vmlinuz.hook` copies it to `/boot/vmlinuz-linux`. It is a copy, not a symlink, because mkarchiso sizes the ESP with `du`, which counts a symlink as the link itself. mkarchiso then empties the live root's `/boot`, so the installer takes the kernel from the ISO (`arch/boot/aarch64/vmlinuz-linux`) and installs it as `/boot/Image`, which Limine's `linux` protocol boots as is, and removes the hook from the installed system ([installer.md](installer.md), step 8).
- The squashfs takes only the `arm64` BCJ filter (`profiledef.sh`). Arch Linux ARM's kernel cannot decode x86 BCJ blocks, which releng's `x86,arm64` could pick.
- The boot entry names both serial consoles, `ttyS0` (x86_64) and `ttyAMA0` (aarch64). The kernel ignores the one the machine lacks.
- NVIDIA's open modules come from `nvidia-open-dkms` (task 022): Arch Linux ARM has no prebuilt module package. DKMS's pacman hook builds `nvidia.ko` for `linux-aarch64` while mkarchiso installs the packages, into `/usr/lib/modules/<kernel>/updates/dkms/`, so the live image boots with it and compiles nothing; installed systems rebuild it when `linux-aarch64` updates. This puts `gcc`, `make` and `linux-aarch64-headers` into this one image (operator 2026-11-08). `tests/vm/arch-aarch64.sh` checks `dkms status` and the module's vermagic.
- Arch Linux ARM's makepkg writes `.pkg.tar.xz` instead of `.pkg.tar.zst`, so `just arch packages` takes either.

The VM runs Fedora with SELinux enforcing, and rootless podman there has two gotchas:

- SELinux hides bind mounts and `/dev/vhost-vsock` from unprivileged containers. `just arch packages` and `just arch test` run with `--security-opt label=disable` (a no-op without SELinux), rather than relabelling the checkout with `:z`. The `--privileged` ISO build has labelling off already.
- pacman's Landlock download sandbox fails in an unprivileged container there, so the builder's pacman calls use `--disable-sandbox`. The `--privileged` ISO build keeps the sandbox.

## Move the package snapshot

Every package, in the builder and in the image, comes from one [Arch Linux Archive](https://archive.archlinux.org/repos/) date: `ala_date` in `arch/vars.just`. The builder's base image is pinned there by digest too. To move forward:

1. Set `ala_date` to the new date.
2. Set `builder_base` to an `archlinux/archlinux:base-<date>` tag from the same day, with its digest.
3. Run `just arch iso`, then `just arch test`. Both must pass before the change lands.

On aarch64, move the builder base the same way: set the aarch64 `builder_base` to a newer `menci/archlinuxarm:base-<date>` tag with its arm64 digest (from its Docker Hub tags), then run `just arch iso` and `just arch test` in the Mac's runner VM. The image's own packages need no pin: every build takes Arch Linux ARM's current repos.
