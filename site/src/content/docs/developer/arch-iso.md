---
title: Arch ISO
description: Build the Arch x86_64 live ISO, boot-test it under QEMU, and move its package snapshot forward.
---

The Arch x86_64 live ISO is built with [archiso](https://gitlab.archlinux.org/archlinux/archiso) inside a pinned builder container, then boot-tested in the same container under QEMU with OVMF. Both steps need Linux with rootless podman and read-write `/dev/kvm` and `/dev/vhost-vsock`. `just doctor` checks them on its `arch:` line.

## Build

```sh
just arch iso                 # version 0.0.0-dev
just arch iso 1.2.3           # stackific-1.2.3-arch-x86_64.iso
```

The version must be semver, as for a release; anything else is refused. The recipe builds the builder image first (`localhost/stackific-arch-builder:<date>`, from `arch/Containerfile`), then our packages (below), then runs `mkarchiso` on the profile in `arch/iso/`. Outputs go to `.build/arch/x86_64/`:

| File                                           | What it is                                          |
| ---------------------------------------------- | --------------------------------------------------- |
| `stackific-<version>-arch-x86_64.iso`          | The live ISO: UEFI only, systemd-boot               |
| `stackific-<version>-arch-x86_64.packages.txt` | The package manifest (`pacman -Q` of the live root) |

The last line prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail.

`mkarchiso` runs as the unprivileged user `build` inside the container. Its work directory stays in the container, so each build starts fresh. Rootless podman cannot mount devtmpfs, which archiso needs when it runs as root.

## Our packages

```sh
just arch packages            # also the first step of just arch iso
```

Our own Arch packages live in `arch/packages/<name>/PKGBUILD`. Their files are base-neutral and live in `desktop/`, so Fedora's packages can ship the same files later. The recipe runs a fresh builder container. For each PKGBUILD, it copies the PKGBUILD and the `desktop/` files its `source=()` names (matched by file name) into a new directory. It runs `makepkg` there as `build`, then `repo-add` puts every package into the local repo `.build/arch/repo/`.

The ISO build mounts that repo read-only at `/repo`, and `arch/iso/pacman.conf` reads it as `[stackific]`. It is unsigned (`SigLevel = Optional`) until release signing lands.

`stackific-desktop` holds two files:

- `stackific-noctalia.service` (`desktop/systemd/user/`), a user unit that starts Noctalia, wanted by `mango-session.target` for every user. MangoWM starts that target itself once it has imported `WAYLAND_DISPLAY` into the user manager. That is how Noctalia starts while MangoWM's own config stays as packaged. Plasma never starts that target, so Noctalia stays out of Plasma.
- `/etc/profile.d/stackific-wlroots.sh` (`desktop/profile.d/`), which exports `WLR_RENDERER_ALLOW_SOFTWARE=1`. With no working GPU driver (a VM without 3D, the boot test's included), Mesa renders in software, and wlroots refuses that without the variable. MangoWM's renderer is GLES-only, so MangoWM would exit. greetd sources `/etc/profile`, so the variable reaches every session it starts.

To change a file in `desktop/`, update its `sha256sums` entry in the PKGBUILD and bump `pkgrel` (or `pkgver`).

## Live session

The ISO boots to a desktop. greetd (`arch/iso/airootfs/etc/greetd/config.toml`) logs the user `live` into MangoWM with no prompt, and MangoWM starts Noctalia, which opens its first-run welcome dialog. In MangoWM's stock config, `Alt+Return` opens foot and `Alt+space` opens rofi.

After a logout, greetd shows noctalia-greeter. There, KDE Plasma is the other session. `live` has an empty password and is in `wheel`, so polkit treats it as an admin (with its empty password). root also has an empty password, on the other consoles. These accounts exist only in the live image: `arch/iso/airootfs/etc/{passwd,group,shadow,gshadow}` and `etc/sysusers.d/stackific-live.conf`. mkarchiso creates `/home/live` from `/etc/skel`.

## Boot test

```sh
just arch test                # boots stackific-0.0.0-dev-arch-x86_64.iso
just arch test 1.2.3
```

`tests/vm/boot.sh` boots the ISO under QEMU with KVM and OVMF. It reaches the VM only through one-boot SMBIOS credentials, which systemd reads at boot:

- `systemd.extra-unit.*` credentials add a vsock socket and service (`tests/vm/stackific-test-shell.*`), and a `systemd.unit-dropin.sockets.target` credential starts the socket.
- A random token arrives as the `stackific.test.token` credential. The shell runs `bash` on a connection only when its first line is that token.

Nothing from `tests/` is in the image, and the image has no sshd. The test waits for `systemctl is-system-running --wait` over the shell, then checks:

- the system is `running`;
- NetworkManager is running;
- a wrong token gets no shell;
- greetd is active, `stackific-noctalia.service` is active for `live`, and `mango` runs as `live`;
- the MangoWM and Plasma session files are present.

It saves `screen.png` (the live desktop), and then logs `live` out with `loginctl terminate-user live`. It checks that noctalia-greeter runs as `greeter`, and saves `greeter.png`. It also saves `serial.log` to `.build/arch/x86_64/`, then powers the VM off. On failure it prints the end of the serial log.

Logging in to Plasma from the greeter is not automated yet. To check it by hand:

1. Boot the ISO in a VM with a display, UEFI firmware and enough RAM (see `-m` in `tests/vm/boot.sh`).
2. Press `Alt+Return` and run `loginctl terminate-user live`.
3. Pick `live` and the Plasma session, and log in with an empty password.
4. In Konsole, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.

systemd's `vmm.notify_socket` is not used. Over vsock it opens one connection per message and drops some of them, READY=1 included.

## Move the package snapshot

Every package, in the builder and in the image, comes from one [Arch Linux Archive](https://archive.archlinux.org/repos/) date: `ala_date` in `arch/vars.just`. The builder's base image is pinned there by digest too. To move forward:

1. Set `ala_date` to the new date.
2. Set `builder_base` to an `archlinux/archlinux:base-<date>` tag from the same day, with its digest.
3. Run `just arch iso`, then `just arch test`. Both must pass before the change lands.
