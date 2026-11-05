# Arch ISO

The Arch live ISO is built with [archiso](https://gitlab.archlinux.org/archlinux/archiso) inside a pinned builder container, then boot-tested in the same container under QEMU with UEFI firmware. The host's architecture picks the image: x86_64 on a Linux PC, and aarch64 from Arch Linux ARM in the Mac's runner VM (README.md, "Self-hosted runners"). Nothing is cross-built. Both steps need Linux with rootless podman and read-write `/dev/kvm` and `/dev/vhost-vsock`. `just doctor` checks them on its `arch:` line.

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

The last line prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail.

`mkarchiso` runs as the unprivileged user `build` inside the container. Its work directory stays in the container, so each build starts fresh. Rootless podman cannot mount devtmpfs, which archiso needs when it runs as root.

## Our packages

```sh
just arch packages            # also the first step of just arch iso
```

Our own Arch packages live in `arch/packages/<name>/PKGBUILD`. Their files are base-neutral and live in `desktop/` and `branding/`, so Fedora's packages can ship the same files later. The recipe runs a fresh builder container. For each PKGBUILD, it copies the PKGBUILD and the `desktop/` and `branding/` files its `source=()` names (matched by file name) into a new directory. It runs `makepkg` there as `build`, then `repo-add` puts every package into the local repo `.build/arch/repo/`.

The ISO build mounts that repo read-only at `/repo`, and `arch/iso/pacman.conf` reads it as `[stackific]`. It is unsigned (`SigLevel = Optional`) until release signing lands.

### The branding file

`branding/branding.env` is the one branding file: the product name, the ID, the four URLs and the accent colour, as shell `KEY="value"` lines. Every package file that names the product or uses its colour is rendered from it in the PKGBUILD's `build()`:

- the PKGBUILD sources `branding.env` (`set -a`, so the keys are exported);
- it runs `envsubst` on a base-neutral template (`*.in` in `branding/`) with an explicit list of keys, such as `envsubst '$ACCENT'`, so no other `$` in the template changes;
- values derived from a key are computed there too: `ANSI_COLOR` is the accent as a 24-bit terminal colour (`38;2;R;G;B`), and the icon is `logo.svg` with `currentColor` replaced by the accent.

A branding edit changes `branding.env`'s sha256, so update it, and bump `pkgrel`, in every PKGBUILD that sources it (`stackific-release`, `stackific-artwork`).

### The packages

`stackific-desktop` holds seven files and the greeter's asset folder:

- `stackific-noctalia.service` (`desktop/systemd/user/`), a user unit that starts Noctalia, wanted by `mango-session.target` for every user. MangoWM starts that target itself once it has imported `WAYLAND_DISPLAY` into the user manager. That is how Noctalia starts while MangoWM's own config stays as packaged. Plasma never starts that target, so Noctalia stays out of Plasma.
- `/etc/profile.d/stackific-wlroots.sh` (`desktop/profile.d/`), which exports `WLR_RENDERER_ALLOW_SOFTWARE=1`. With no working GPU driver (a VM without 3D, the boot test's included), Mesa renders in software, and wlroots refuses that without the variable. MangoWM's renderer is GLES-only, so MangoWM would exit. greetd sources `/etc/profile`, so the variable reaches every session it starts.
- `/usr/share/stackific/noctalia.toml` (`desktop/noctalia/stackific-noctalia.toml`), our Noctalia defaults: no first-run setup wizard, the wallpaper picker browsing `/usr/share/backgrounds/stackific`, and Dahl's "An Eruption of Vesuvius" as the default wallpaper.
- `/etc/skel/.config/noctalia/00-stackific.toml` (`desktop/noctalia/`), which only includes the defaults above. Noctalia reads config only from the user's `~/.config/noctalia/` (no `/etc` or `/usr/share` layer), so each user gets this one-line file from `/etc/skel`, and package updates to the defaults reach them. It sorts before the user's own files, which win, and the settings Noctalia's UI writes (`~/.local/state/noctalia/settings.toml`) win over both. Users created before the package was installed have no copy; `cp /etc/skel/.config/noctalia/00-stackific.toml ~/.config/noctalia/` adds it.
- `/usr/share/stackific/greeter.toml`, rendered from `desktop/noctalia/stackific-greeter.toml.in` with the accent: noctalia-greeter's administrator config, with a dark palette in our emerald and the same painting as the desktop.
- `/usr/lib/tmpfiles.d/stackific-greeter.conf` (`desktop/tmpfiles.d/`), which links `/var/lib/noctalia-greeter/greeter.toml`, the file the greeter reads, to the one above. Its `L` line creates the link only when that file is missing, so an administrator's own `greeter.toml` is never replaced.
- `/etc/profile.d/stackific-greeter.sh` (`desktop/profile.d/`), which exports `NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets`, the greeter's documented override of its asset folder. The greeter has no key for another logo, but takes `noctalia.svg` and `fonts/` from that folder. Ours holds two links: `noctalia.svg` to `stackific-release`'s emerald icon, and `fonts` to `/usr/share/noctalia-greeter/assets/fonts`. greetd sources `/etc/profile` before the greeter, so our logo sits where Noctalia's was, with no package file changed.

`stackific-artwork` holds the wallpapers and fastfetch's logo, and depends on `fastfetch`, so installing our branding installs fastfetch:

- `/usr/share/backgrounds/stackific/*.jpg`: the ten public-domain paintings in `branding/wallpapers/`, credited in `branding/wallpapers.toml`, which the package installs as its licence file (`/usr/share/licenses/stackific-artwork/wallpapers.toml`);
- `/usr/share/stackific/logo.txt`: the block-character text logo, `$1` marking where fastfetch puts the colour;
- `/etc/xdg/fastfetch/config.jsonc`, rendered from `branding/fastfetch.jsonc.in`: the logo file in the accent, with fastfetch's default modules. fastfetch reads the first `fastfetch/config.jsonc` it finds in `~/.config/`, `$XDG_CONFIG_DIRS`, `/etc/xdg/` and `/etc/`, so a user's own config replaces ours.

`stackific-release` makes the image identify as Stackific OS. It holds two files:

- `/etc/os-release`: `branding/os-release.in` rendered from the branding file (the base-neutral fields: `ID=stackific`, `NAME` and `PRETTY_NAME` "Stackific OS", the site and GitHub issues URLs, `DEFAULT_HOSTNAME=stackific`, `LOGO=stackific` and the emerald `ANSI_COLOR`), followed by Arch's per-base lines `ID_LIKE=arch` and `BUILD_ID=rolling`. The base is rolling, so there is no `VERSION` or `VERSION_ID`, as os-release(5) allows. The template has no comment lines, because some os-release readers split every line on `=`.
- `/usr/share/icons/hicolor/scalable/apps/stackific.svg`, the icon that `LOGO=stackific` names: `branding/logo.svg` with its `currentColor` fill replaced by the accent at build time. `currentColor` alone renders black, invisible on a dark panel.

Arch's `filesystem` package owns `/usr/lib/os-release` and `/etc/arch-release`, and both stay as shipped. os-release(5) readers use `/etc/os-release` when it exists and never merge it with `/usr/lib/os-release`, so owning the `/etc` file is enough, with no hook, patch or fork. No Arch package owns `/etc/os-release`: systemd's tmpfiles line `L /etc/os-release` only makes it a symlink when it is missing, and runs after the transaction, so pacman's conflict check never sees it. That holds because mkarchiso installs every package in one pacstrap call. Installing the package later onto a system that already has the symlink needs `pacman --overwrite /etc/os-release`. It is not in `backup=()`: it is vendor data, like Arch's own `/usr/lib/os-release`, so an upgrade replaces it.

mkarchiso appends `IMAGE_ID=stackific` and `IMAGE_VERSION=<version>-arch` to the file `/etc/os-release` resolves to, which is now ours. That is how the ISO's semver version reaches the live image.

To change a file in `desktop/` or `branding/`, update its `sha256sums` entry in the PKGBUILD and bump `pkgrel` (or `pkgver`).

## Live session

The ISO boots to a desktop. greetd (`arch/iso/airootfs/etc/greetd/config.toml`) logs the user `live` into MangoWM with no prompt, and MangoWM starts Noctalia, which shows Dahl's "An Eruption of Vesuvius" and no setup wizard (our defaults, above). `fastfetch` in a terminal prints our logo in emerald. In MangoWM's stock config, `Alt+Return` opens foot and `Alt+space` opens rofi.

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

`boot.sh` then sources the image's own checks, `tests/vm/arch-<arch>.sh`, which `just arch test` passes as its third argument. The x86_64 file, `tests/vm/arch-x86_64.sh`, checks:

- `/etc/os-release` reads `ID=stackific`, `ID_LIKE=arch`, `BUILD_ID=rolling`, `PRETTY_NAME="Stackific OS"`, `LOGO=stackific` and mkarchiso's `IMAGE_ID=stackific`; `hostnamectl` shows `Operating System: Stackific OS`; pacman says `stackific-release` owns `/etc/os-release` and `filesystem` owns `/etc/arch-release` and `/usr/lib/os-release`; `/usr/lib/os-release` has no `IMAGE_` lines; the `stackific` icon is filled `#007a55`;
- greetd is active, `stackific-noctalia.service` is active for `live`, and `mango` runs as `live`;
- the MangoWM and Plasma session files are present;
- the ten wallpapers are installed, `live`'s `00-stackific.toml` matches `/etc/skel`'s, and `noctalia config export merged` (as `live`) shows the wizard off, the wallpaper folder and the default wallpaper;
- `fastfetch --pipe false` (as `live`) prints the logo's last line in the accent colour.

It saves `screen.png` (the live desktop), and then logs `live` out with `loginctl terminate-user live`. It checks that noctalia-greeter runs as `greeter`, and saves `greeter.png`.

The aarch64 file, `tests/vm/arch-aarch64.sh`, checks a console: `uname -m` prints `aarch64`, `getty@tty1.service` is active, and tty1's text (`/dev/vcs1`) shows the `stackific login:` prompt. It saves `screen.png` of that prompt.

Both save `serial.log` to `.build/arch/<arch>/`, then power the VM off. On failure the test prints the end of the serial log.

Logging in to Plasma from the greeter is not automated yet. To check it by hand:

1. Boot the ISO in a VM with a display, UEFI firmware and enough RAM (see `-m` in `tests/vm/boot.sh`).
2. Press `Alt+Return` and run `loginctl terminate-user live`.
3. Pick `live` and the Plasma session, and log in with an empty password.
4. In Konsole, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.

systemd's `vmm.notify_socket` is not used. Over vsock it opens one connection per message and drops some of them, READY=1 included.

## aarch64

The aarch64 image is built and tested on aarch64 Linux: the `arch-aarch64` jobs of the `images` workflow (after each merge to `main`) and of the `release` workflow run it on the Mac's runner VM, and upload it like the x86_64 images (`docs/releases.md`). Like the x86_64 one before task 002, it boots to a console: the desktop follows in a later task. It shares the profile in `arch/iso/` with x86_64 and has its own package list, `packages.aarch64`. It installs our branding as x86_64 does (`stackific-release`, and `stackific-artwork` with fastfetch), and its boot test (`tests/vm/arch-aarch64.sh`) checks the identity, the wallpapers and fastfetch's logo; the desktop packages wait for the desktop task. Until the desktop arrives, the x86_64 desktop's airootfs files (the greetd config, the `live` and `greeter` accounts, the `display-manager.service` alias) sit unused in it.

Where its parts come from:

- [Arch Linux ARM](https://archlinuxarm.org) has no archive, so the image comes from its live repos, and the package manifest is the record of what went in.
- The builder starts from `docker.io/menci/archlinuxarm`, pinned by digest in `arch/vars.just`. Arch Linux ARM publishes no container image; this one is built daily from its repos with pacstrap.
- Arch Linux ARM does not package archiso or the AArch64 UEFI firmware. The builder installs Arch's `any` packages `archiso` and `edk2-aarch64` from the archive date in `ala_date`, checked against Arch's signing keys. archiso v92 builds aarch64 natively, with `systemd-bootaa64.efi` as `BOOTAA64.EFI`.

What differs from x86_64:

- Arch Linux ARM's kernel is `/boot/Image`, but mkarchiso and the boot entry take `vmlinuz-*`. The pacman hook `arch/iso/airootfs/etc/pacman.d/hooks/stackific-vmlinuz.hook` copies it to `/boot/vmlinuz-linux`. It is a copy, not a symlink, because mkarchiso sizes the ESP with `du`, which counts a symlink as the link itself.
- The squashfs takes only the `arm64` BCJ filter (`profiledef.sh`). Arch Linux ARM's kernel cannot decode x86 BCJ blocks, which releng's `x86,arm64` could pick.
- The boot entry names both serial consoles, `ttyS0` (x86_64) and `ttyAMA0` (aarch64). The kernel ignores the one the machine lacks.
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
