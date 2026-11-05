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

The version must be semver, as for a release; anything else is refused. The recipe builds the builder image first (`localhost/stackific-arch-builder:<date>`, from `arch/Containerfile`), then runs `mkarchiso` on the profile in `arch/iso/`. Outputs go to `.build/arch/x86_64/`:

| File                                           | What it is                                          |
| ---------------------------------------------- | --------------------------------------------------- |
| `stackific-<version>-arch-x86_64.iso`          | The live ISO: UEFI only, systemd-boot               |
| `stackific-<version>-arch-x86_64.packages.txt` | The package manifest (`pacman -Q` of the live root) |

The last line prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail.

`mkarchiso` runs as the unprivileged user `build` inside the container. Its work directory stays in the container, so each build starts fresh. Rootless podman cannot mount devtmpfs, which archiso needs when it runs as root.

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
- root is logged in on `tty1`;
- NetworkManager is running;
- a wrong token gets no shell.

It saves `screen.png` (the console) and `serial.log` to `.build/arch/x86_64/`, then powers the VM off. On failure it prints the end of the serial log.

systemd's `vmm.notify_socket` is not used. Over vsock it opens one connection per message and drops some of them, READY=1 included.

## Move the package snapshot

Every package, in the builder and in the image, comes from one [Arch Linux Archive](https://archive.archlinux.org/repos/) date: `ala_date` in `arch/vars.just`. The builder's base image is pinned there by digest too. To move forward:

1. Set `ala_date` to the new date.
2. Set `builder_base` to an `archlinux/archlinux:base-<date>` tag from the same day, with its digest.
3. Run `just arch iso`, then `just arch test`. Both must pass before the change lands.
