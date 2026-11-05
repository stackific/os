# Fedora ISO

The Fedora 45 x86_64 live ISO is built with [kiwi](https://osinside.github.io/kiwi/), the tool Fedora builds its own live ISOs with, inside a pinned builder container. It is then boot-tested in the same container under QEMU with OVMF. Both steps need Linux with rootless podman and read-write `/dev/kvm` and `/dev/vhost-vsock`. `just doctor` checks them on its `fedora:` line.

## Build

```sh
just fedora iso               # version 0.0.0-dev
just fedora iso 1.2.3         # stackific-1.2.3-fedora-x86_64.iso
```

The version must be semver, as for a release; anything else is refused. The recipe builds the builder image first (`localhost/stackific-fedora-builder:45`, from `fedora/Containerfile`), then runs `kiwi-ng system build` on the description in `fedora/iso/`:

- `config.kiwi`: repositories, packages and the ISO type;
- `config.sh`: runs in the image root after the packages are in;
- `root/`: files copied into the image.

kiwi runs as root of a rootless `--privileged` container, so it has no privileges on the host. It bind-mounts `/dev`, `/proc` and `/sys` into the image root, which needs `--privileged`. Its work directory is a tmpfs inside the container (`--tmpfs /work:rw,exec,dev,suid`), for two reasons:

- `--rm` discards it, so every build starts fresh.
- Labelling the image for SELinux needs a filesystem where rootless container root may set `security.selinux`. podman's overlay storage refuses that (`setfiles: Could not set context ... Operation not permitted`); a tmpfs inside the container accepts it.

podman's default tmpfs is `noexec`, which breaks kiwi's chroot, hence `exec`. The tmpfs holds about 2 GiB at the end of a build and can grow to half the host's RAM.

No loop device is needed: the live root is erofs, which `mkfs.erofs` writes straight from a directory, and kiwi makes the ISO's EFI image with `mkdosfs` and `mcopy`. kiwi's systemd-boot ISO path needs a loop device and a mount, so this ISO boots with GRUB.

Outputs go to `.build/fedora/x86_64/`:

| File                                             | What it is                                     |
| ------------------------------------------------ | ---------------------------------------------- |
| `stackific-<version>-fedora-x86_64.iso`          | The live ISO: UEFI only, GRUB, no shim         |
| `stackific-<version>-fedora-x86_64.packages.txt` | The package manifest, one package per line     |

Each manifest line is `name|epoch|version|release|arch|disturl|license`. Fedora builds from its live repos, so the manifest is the record of what a build got. The last line of the build prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail.

Packages are checked against Fedora's signing key: both repositories set `package_gpgcheck="true"` with the builder's `RPM-GPG-KEY-fedora-45-primary`.

## Our packages

```sh
just fedora packages          # stackific-release, stackific-desktop, stackific-artwork
```

Our Fedora packages are specs in `fedora/packages/<name>/<name>.spec`. They install the same base-neutral files from `desktop/` and `branding/`, at the same paths, as the Arch PKGBUILDs (`docs/arch-iso.md`), and render them from `branding/branding.env` with the same lines. Each spec's `SourceN:` files are found by name under `desktop/` and `branding/`, as `just arch packages` does, so every source name must be unique across both folders.

The recipe runs one fresh `--privileged` builder container. In it, mock runs as container root, so it has no privileges on the host:

- `--isolation=simple`, because the container has no systemd-nspawn, which mock's default isolation needs;
- `--no-bootstrap-image`, because mock's default bootstrap pulls a container image, which would need podman inside the container.

For each spec it runs `mock --buildsrpm`, then `mock --rebuild` of that source RPM, with the config `fedora-45-<host arch>`, and copies the `.noarch.rpm` to `.build/fedora/repo/`. `createrepo_c` then writes the repo metadata. The repo is unsigned until release signing lands. The first mock call takes about two minutes, because it sets up mock's bootstrap and build roots. Later calls in the same container reuse its root cache and take about 20 seconds.

Two spec gotchas:

- rpm expands `%` in spec sections and comments, so a shell `printf` format writes `%d` as `%%d`. Spec comments avoid `%`.
- Without a `%changelog`, rpmbuild warns `%source_date_epoch_from_changelog is set, but %changelog has no entries`. The build still succeeds.

A branding edit changes what `stackific-release` and `stackific-artwork` render, so bump their `Release`.

### stackific-release

`stackific-release` replaces only `generic-release`, which owns just `/usr/lib/os-release`. It keeps `generic-release-common`, which holds Fedora's rpm macros (`%fedora`, `%dist`), dnf5's defaults (`pkg_gpgcheck=True` among them) and the `/etc` symlinks. One of them is `/etc/os-release`, which points at `/usr/lib/os-release`, so on Fedora our package owns `/usr/lib/os-release`, not `/etc/os-release` as on Arch. The rendered file has Fedora's per-base lines `ID_LIKE=fedora`, `VERSION_ID=45` and `SUPPORT_END=2027-12-08`, and no `BUILD_ID`. `/etc/system-release` still reads `Generic release 45 (Generic)`, because `generic-release-common` owns it.

Its provides keep what depends on `generic-release` working:

- `system-release` and `system-release(45)`: `fedora-repos` requires `system-release(45)`.
- `system-release(releasever) = 45`: dnf5 takes `$releasever` from this provide. Without it, it would use the package's own version, `0.1.0`, and every Fedora repo URL would break.
- `generic-release-variant = 45-0.4`: `generic-release-common` requires exactly that version. Bump it when Fedora updates `generic-release`.

It also requires `distribution-systemd-presets`, as `generic-release` does: the presets turn on NetworkManager, PipeWire and the other services Fedora enables by default. On a system that has `generic-release`, `dnf install --allowerasing stackific-release` swaps it out. On one with `fedora-release*` it swaps those out too, because `generic-release-common` conflicts with `fedora-release-common`.

### stackific-desktop and stackific-artwork

They hold the same files as on Arch (`docs/arch-iso.md`, "Our packages"): the Noctalia unit, defaults and `/etc/skel` include, the wlroots and greeter profile.d scripts, the greeter's `greeter.toml` with its tmpfiles link, and the greeter's asset folder, whose links point `noctalia.svg` at `stackific-release`'s icon and `fonts` at Terra's `noctalia-greeter` fonts, which sit at the same path as on Arch; and the wallpapers with their credits and fastfetch's config. `stackific-artwork` requires `fastfetch` and `stackific-desktop` requires `noctalia-greeter` and `stackific-release`, as on Arch. `stackific-desktop` builds with `systemd-rpm-macros` for `%{_tmpfilesdir}`.

## What is in the image

- The kernel, `linux-firmware` and the default firmware of Fedora's `hardware-support` group, plus AMD and Intel microcode. The server and datacenter firmware in that group is optional there, so it stays out.
- NetworkManager, btrfs and FAT tools, and SELinux enforcing with the targeted policy.
- Root autologin on tty1. Fedora 45 gives tty1 to kmscon and its presets disable `getty@.service`, so the image enables `getty@tty1.service` itself (`root/etc/systemd/system/getty.target.wants/`) with an autologin drop-in. root has an empty password (`config.sh`).
- `generic-release`, not `fedora-release`. Fedora's trademark guidelines require a remix to leave out `fedora-release`, `fedora-logos` and `fedora-release-notes`; Fedora ships the `generic-*` packages for this. Until our own `stackific-release`, os-release says `Generic` and its URLs are placeholders.
- No weak dependencies: kiwi installs only what packages require, and `config.kiwi` names the few weak dependencies users commonly need (`tzdata`, `cracklib-dicts`, `intel-npu-firmware`). Task 003 lists what is left out and why.

The boot menu, volume ID (`STACKIFIC`) and ISO metadata say Stackific OS; the kernel, GRUB and every other binary are unmodified Fedora packages.

## Boot test

```sh
just fedora test              # boots stackific-0.0.0-dev-fedora-x86_64.iso
just fedora test 1.2.3
```

It runs the same `tests/vm/boot.sh` as `just arch test` (see `docs/arch-iso.md`), with Fedora's raw OVMF pair (`OVMF_CODE` and `OVMF_VARS` in the environment). After the checks every image passes, `boot.sh` sources `tests/vm/fedora-x86_64.sh`, which checks that:

- `who` shows root on `tty1`;
- `getenforce` prints `Enforcing`.

It saves `screen.png` (the root prompt on tty1) and `serial.log` to `.build/fedora/x86_64/`.

The test shell is a service running `systemd-socket-activate`, not a socket unit. With a socket unit, PID 1 labels the listening vsock socket for the shell it will start (`initrc_t`), and Fedora's policy denies PID 1 that socket. The service runs as `unconfined_service_t`, which may listen on vsock.

## Move the builder digest

The builder's base image is pinned by digest in `fedora/vars.just` (`builder_base`). To move it:

1. Look up the new digest of `registry.fedoraproject.org/fedora:45` (the `docker-content-digest` header of its manifest).
2. Set `builder_base` to it.
3. Run `just fedora iso`, then `just fedora test`. Both must pass before the change lands.

podman's layer cache keeps the builder's kiwi and QEMU until the digest moves.
