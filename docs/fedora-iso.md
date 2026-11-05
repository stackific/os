# Fedora ISO

The Fedora 45 x86_64 desktop live ISO is built with [kiwi](https://osinside.github.io/kiwi/), the tool Fedora builds its own live ISOs with, inside a pinned builder container. It is then boot-tested in the same container under QEMU with OVMF. Both steps need Linux with rootless podman and read-write `/dev/kvm` and `/dev/vhost-vsock`. `just doctor` checks them on its `fedora:` line.

## Build

```sh
just fedora iso               # version 0.0.0-dev
just fedora iso 1.2.3         # stackific-1.2.3-fedora-x86_64.iso
```

The version must be semver, as for a release; anything else is refused. The recipe builds the builder image first (`localhost/stackific-fedora-builder:45`, from `fedora/Containerfile`), then our RPMs (`just fedora packages`, below), then runs `kiwi-ng system build` on the description in `fedora/iso/`, with the local repo `.build/fedora/repo/` mounted read-only at `/repo`:

- `config.kiwi`: repositories, packages and the ISO type;
- `config.sh`: runs in the image root after the packages are in;
- `root/`: files copied into the image;
- `terra-repo.sh` and `terra-45.asc`: the Terra repository's limits and key (below).

kiwi runs as root of a rootless `--privileged` container, so it has no privileges on the host. It bind-mounts `/dev`, `/proc` and `/sys` into the image root, which needs `--privileged`. Its work directory is a tmpfs inside the container (`--tmpfs /work:rw,exec,dev,suid`), for two reasons:

- `--rm` discards it, so every build starts fresh.
- Labelling the image for SELinux needs a filesystem where rootless container root may set `security.selinux`. podman's overlay storage refuses that (`setfiles: Could not set context ... Operation not permitted`); a tmpfs inside the container accepts it.

podman's default tmpfs is `noexec`, which breaks kiwi's chroot, hence `exec`. The tmpfs peaks at about 5.3 GiB (5398 MiB, sampled with `df` during a build of the desktop ISO). By default the tmpfs is capped at half the host's RAM, so on a host with less than about 12 GiB the build stops with `No space left on device` under `/work`; there it needs an explicit `size=` on the tmpfs, with the RAM or swap behind it.

No loop device is needed: the live root is erofs, which `mkfs.erofs` writes straight from a directory, and kiwi makes the ISO's EFI image with `mkdosfs` and `mcopy`. kiwi's systemd-boot ISO path needs a loop device and a mount, so this ISO boots with GRUB.

Outputs go to `.build/fedora/x86_64/`:

| File                                             | What it is                                     |
| ------------------------------------------------ | ---------------------------------------------- |
| `stackific-<version>-fedora-x86_64.iso`          | The live ISO: UEFI only, GRUB, no shim         |
| `stackific-<version>-fedora-x86_64.packages.txt` | The package manifest, one package per line     |

Each manifest line is `name|epoch|version|release|arch|disturl|license`. Fedora builds from its live repos, so the manifest is the record of what a build got. The last line of the build prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail.

## Repositories

| Alias       | Source                                  | Supplies                                    | Checked against                                   |
| ----------- | --------------------------------------- | ------------------------------------------- | ------------------------------------------------- |
| `fedora`    | Fedora's metalink                       | everything not listed below                 | packages: the builder's `RPM-GPG-KEY-fedora-45-primary` |
| `updates`   | Fedora's metalink                       | the same, updated                           | packages: the same key                            |
| `terra`     | `https://repos.fyralabs.com/terra45`    | `mangowm` and `noctalia-greeter` only       | packages and metadata: `fedora/iso/terra-45.asc`  |
| `stackific` | `/repo` (`.build/fedora/repo/`)         | our three RPMs                              | nothing yet: unsigned until release signing       |

[Terra](https://terra.fyralabs.com) packages MangoWM and noctalia-greeter, which Fedora does not. Everything else comes from Fedora, Noctalia and scenefx included. kiwi writes one repo file per repository, then runs a repository's `customize` script with that file's path. `terra-repo.sh` appends two lines kiwi has no attribute for:

- `includepkgs=mangowm,noctalia-greeter`. Unlimited, dnf takes Terra's `cardwire` (a GPU manager) for `gnome-shell`'s `switcheroo-control`, and Terra's builds of packages Fedora ships too. Add a package here only when Fedora lacks it (Limine joins with the installer).
- `gpgkey=file:///desc/terra-45.asc`. `repository_gpgcheck="true"` makes dnf check Terra's signed metadata, which it does only against a key named in the repo file; without it the metadata does not load. `package_gpgcheck="true"` checks each package against the same key, which kiwi imports into the image's rpmdb (`<signing>`).

`terra-45.asc` is Terra 45's public key, committed verbatim from `https://repos.fyralabs.com/terra45/key.asc`: sha256 `411e0c66b22a3f3d79651c05b13c0dbcd9edd72f6839e5e33d1fd5e1c6248f7b`, fingerprint `C2AC02124AF114F086592E3C8DDE7D14C1C23D8C` (`gpg --show-keys`). A key change is a reviewed change in git. With another key in its place (Terra 44's, for example), the build stops on Terra's signature check: `repomd.xml GPG signature verification error: Signing key not found`. Terra has one key per Fedora release, so for Fedora 46:

1. Download `https://repos.fyralabs.com/terra46/key.asc`, and compare `gpg --show-keys` with the fingerprint Terra publishes (its `terra-gpg-keys` package, or dnf's import prompt on a Fedora 46 system with Terra).
2. Commit it as `fedora/iso/terra-46.asc` in place of `terra-45.asc`, and update the file name in `config.kiwi`, `terra-repo.sh` and `AGENTS.md`.
3. Run `just fedora iso`, then `just fedora test`.

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
- NetworkManager with Wi-Fi, btrfs and FAT tools, and SELinux enforcing with the targeted policy.
- The desktop: MangoWM and noctalia-greeter from Terra, Noctalia, greetd, foot and rofi from Fedora, and GNOME (`gnome-shell`, its Wayland session, Ptyxis and Nautilus), with the GNOME, GTK and wlr portals and PipeWire. Our three RPMs carry the defaults (the live session, below).
- `stackific-release`, not `fedora-release`. Fedora's trademark guidelines require a remix to leave out `fedora-release`, `fedora-logos` and `fedora-release-notes`; `stackific-release` takes the place of Fedora's `generic-release` (above). `gnome-shell` requires `gdm`, which requires `system-logos`; `config.kiwi` names `generic-logos` for it, or dnf would pick `fedora-logos`.
- No weak dependencies: kiwi installs only what packages require, and `config.kiwi` names the few weak dependencies users commonly need: `tzdata`, `cracklib-dicts` and `intel-npu-firmware` (task 003), and for the desktop Wi-Fi (`NetworkManager-wifi`, `wpa_supplicant`, `wireless-regdb`, which NetworkManager alone lacks on Fedora) and GNOME's fonts (`adwaita-sans-fonts`, `adwaita-mono-fonts`). Left out until the hardware tasks, as on Arch: Bluetooth, power profiles, Thunderbolt, Vulkan, VA-API and libcamera. Left out as rare on a live desktop: input methods (`ibus`), JACK and ALSA bridges, GNOME's tour, remote desktop, colour manager, parental controls and search indexer, filesystem tools beyond btrfs and FAT, and the codec libraries. Tasks 003 and 008 list them all.

## Live session

The ISO boots to a desktop. greetd (`root/etc/greetd/config.toml`, the live image's only change to a desktop package's file) logs the user `live` into MangoWM through `[initial_session]`, with no prompt. MangoWM starts `mango-session.target`, which pulls in `stackific-noctalia.service` from `stackific-desktop`, so Noctalia comes up on Dahl's "An Eruption of Vesuvius" with no setup wizard. `/etc/profile.d/stackific-wlroots.sh` lets MangoWM start on the VM's software renderer. In MangoWM's stock config, `Alt+Return` opens foot and `Alt+space` opens rofi.

After a logout, greetd runs noctalia-greeter as Fedora's greeter user `greetd` (Arch's is `greeter`), in our palette and with our logo (`docs/arch-iso.md`, "Our packages"). There, GNOME is the other session, and the session picker shows it first, where Arch's shows Mango first. GNOME never starts `mango-session.target`, so Noctalia stays out of it. Without gdm, GNOME turns its screen lock off and says so in a notification (`Screen Locking requires the GNOME display manager`). Terra's noctalia-greeter ships no tmpfiles file, so `root/etc/tmpfiles.d/noctalia-greeter.conf` creates its state directory `/var/lib/noctalia-greeter`, owned by `greetd`, before `stackific-greeter.conf` links our `greeter.toml` into it.

`gnome-shell` requires `gdm`, and Fedora's presets enable it as `display-manager.service`. `config.sh` disables it and enables greetd instead. greetd runs in `xdm_t`, the domain gdm uses (`greetd-selinux`), and `pam_selinux` moves user sessions to the user's context.

`config.sh` creates `live` with `useradd` (home from `/etc/skel`, our Noctalia include among it), in `wheel`, with an empty password. root also has an empty password, on the other consoles. tty1 belongs to greetd. noctalia-greeter refuses an empty password field, so to log `live` in from the greeter, type any character: Fedora's authselect `local` profile has `nullok`, so `pam_unix` authenticates a user with an empty password without asking for one, and what was typed is never checked.

The boot menu, volume ID (`STACKIFIC`) and ISO metadata say Stackific OS; the kernel, GRUB and every other binary are unmodified Fedora packages.

## Boot test

```sh
just fedora test              # boots stackific-0.0.0-dev-fedora-x86_64.iso
just fedora test 1.2.3
```

It runs the same `tests/vm/boot.sh` as `just arch test` (see `docs/arch-iso.md`), with Fedora's raw OVMF pair (`OVMF_CODE` and `OVMF_VARS` in the environment). After the checks every image passes, `boot.sh` sources `tests/vm/fedora-x86_64.sh`. It makes the checks of `tests/vm/arch-x86_64.sh`, with Fedora's names, and adds Terra and SELinux:

- `/etc/os-release` reads `ID=stackific`, `ID_LIKE=fedora`, `VERSION_ID=45`, `PRETTY_NAME="Stackific OS"` and `LOGO=stackific`; `hostnamectl` shows `Operating System: Stackific OS`; rpm says `generic-release-common` owns `/etc/os-release` (the link) and `stackific-release` owns `/usr/lib/os-release`; no `fedora-release*`, `fedora-logos*` or `fedora-release-notes` is installed; the `stackific` icon is filled `#007a55`;
- the only packages with vendor `Terra` are `mangowm` and `noctalia-greeter`;
- `getenforce` prints `Enforcing`;
- greetd is active and `display-manager.service` links to it; `stackific-noctalia.service` is active for `live`, and `mango` runs as `live`. The unit check runs `systemctl --user` as `live` through `runuser`, not `systemctl --user -M live@` as on Arch: that runs `systemd-run -P`, which hands the test shell's vsock socket to PID 1 and dbus-broker, and Fedora's policy denies them that socket, silently through dontaudit rules (`Failed to start transient service unit: Connection reset by peer`);
- the MangoWM and GNOME session files are present;
- the ten wallpapers are installed, `live`'s `00-stackific.toml` matches `/etc/skel`'s, and `noctalia config export merged` (as `live`) shows the wizard off, the wallpaper folder and the default wallpaper;
- `/var/lib/noctalia-greeter` belongs to `greetd`, its `greeter.toml` links to ours, which has our accent as `primary`, and the greeter's asset folder links `noctalia.svg` to our icon;
- `fastfetch` (as `live`) prints the logo's last line in the accent colour, and `OS: Stackific OS`.

It saves `screen.png` (the live desktop), then logs `live` out with `loginctl terminate-user live`. It checks that noctalia-greeter runs as `greetd` with `NOCTALIA_GREETER_ASSETS_DIR` set to our asset folder, and saves `greeter.png`. Last, the journal of the boot has no `avc:  denied` line. journald reads the kernel's audit records itself (`systemd-journald-audit.socket`), so the kernel's rate limit on printing them does not hide any. All three files go to `.build/fedora/x86_64/`, with `serial.log`.

Logging in to GNOME from the greeter is not automated yet. To check it by hand:

1. Boot `.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.iso` in a VM with a display, UEFI firmware and 4 GiB of RAM (virt-manager or GNOME Boxes, for example). MangoWM with Noctalia's bar appears.
2. Press `Alt+Return`. A foot terminal opens. Run `loginctl terminate-user live`.
3. At noctalia-greeter, pick the user `live` and the GNOME session, type any character as the password (above) and press Enter.
4. GNOME's desktop appears. In Ptyxis, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.

The test shell is a service running `systemd-socket-activate`, not a socket unit. With a socket unit, PID 1 labels the listening vsock socket for the shell it will start (`initrc_t`), and Fedora's policy denies PID 1 that socket. The service runs as `unconfined_service_t`, which may listen on vsock.

## Move the builder digest

The builder's base image is pinned by digest in `fedora/vars.just` (`builder_base`). To move it:

1. Look up the new digest of `registry.fedoraproject.org/fedora:45` (the `docker-content-digest` header of its manifest).
2. Set `builder_base` to it.
3. Run `just fedora iso`, then `just fedora test`. Both must pass before the change lands.

podman's layer cache keeps the builder's kiwi and QEMU until the digest moves.
