# Fedora ISO

The Fedora 45 desktop live ISO is built with [kiwi](https://osinside.github.io/kiwi/), the tool Fedora builds its own live ISOs with, inside a pinned builder container. It is then boot-tested in the same container under QEMU with UEFI firmware. The host's architecture picks the image: x86_64 on a Linux PC, and aarch64 in the Mac's runner VM (docs/setup.md, "Self-hosted runners"). Nothing is cross-built. Both steps need Linux with rootless podman and read-write `/dev/kvm` and `/dev/vhost-vsock`. `just doctor` checks them on its `fedora:` line.

## Build

```sh
just fedora iso               # version 0.0.0-dev
just fedora iso 1.2.3         # stackific-1.2.3-fedora-<arch>.iso
```

The version must be semver, as for a release; anything else is refused. The recipe builds the builder image first (`localhost/stackific-fedora-builder:45`, from `fedora/Containerfile`), then our RPMs (`just fedora packages`, below), then runs `kiwi-ng system build` on the description in `fedora/iso/`, with the local repo `.build/fedora/repo/` mounted read-only at `/repo`:

- `config.kiwi`: repositories, packages and the ISO type;
- `config.sh`: runs in the image root after the packages are in;
- `root/`: files copied into the image;
- `terra-repo.sh` and `terra-45.asc`: the Terra repository's limits and key (below).

kiwi runs as root of a rootless `--privileged` container, so it has no privileges on the host. It bind-mounts `/dev`, `/proc` and `/sys` into the image root, which needs `--privileged`. Its work directory, and its temp directory (`--temp-dir /work/tmp`, in place of the default `/var/tmp` on podman's overlay), are on a tmpfs that container root mounts itself (`mount -t tmpfs tmpfs /work`, before `kiwi-ng`), for two reasons:

- `--rm` discards it, so every build starts fresh.
- Labelling the image for SELinux needs a filesystem where rootless container root may set `security.selinux`. podman's overlay storage refuses that (`setfiles: Could not set context ... Operation not permitted`); a tmpfs container root mounts accepts it.

It is not podman's `--tmpfs`. On a host with SELinux enabled, such as the Mac's runner VM, podman mounts that with one fixed `context=` label, even with `--privileged`. There, kiwi's first copy into the image root stops with `fsetxattr(... "security.selinux") failed: Operation not supported (95)`, and so does its EFI step in `/var/tmp`. mount(8)'s defaults include `exec`, which kiwi's chroot needs.

On an SELinux host, rootless container root may also set only the contexts the host's own policy knows. So the build host's Fedora release must be the image's: on a Fedora 44 host, `setfiles` stops on Fedora 45's contexts (`Could not set context for /usr/share/accountsservice: Invalid argument`). The Mac's runner VM therefore runs Fedora 45 (docs/setup.md, "Mac (aarch64 images)").

The tmpfs is capped at half the host's RAM. Sampled with `df` during a build of the desktop ISO, it peaks at about 5.3 GiB on x86_64 (5398 MiB) and about 6.6 GiB on aarch64 (6763 MiB), when xorriso writes the ISO last. So the host needs about 14 GiB of RAM; the Mac's runner VM has 16 GiB. In the VM's earlier 12 GiB, xorriso stopped with `Image size 733160s exceeds free space on media 323398s`. On a smaller host, the build stops like that or with `No space left on device` under `/work`, and it needs an explicit `size=` on the tmpfs, with the RAM or swap behind it.

No loop device is needed: the live root is erofs, which `mkfs.erofs` writes straight from a directory, and kiwi makes the ISO's EFI image with `mkdosfs` and `mcopy`. kiwi's systemd-boot ISO path needs a loop device and a mount, so this ISO boots with GRUB.

Outputs go to `.build/fedora/<arch>/`:

| File                                             | What it is                                     |
| ------------------------------------------------ | ---------------------------------------------- |
| `stackific-<version>-fedora-<arch>.iso`          | The live ISO: UEFI only, GRUB, no shim         |
| `stackific-<version>-fedora-<arch>.packages.txt` | The package manifest, one package per line     |

Each manifest line is `name|epoch|version|release|arch|disturl|license`. Fedora builds from its live repos, so the manifest is the record of what a build got. The last line of the build prints the ISO size against the 2048 MiB target. Above the target, a warning comes first, but the build does not fail.

## Repositories

| Alias       | Source                                  | Supplies                                    | Checked against                                   |
| ----------- | --------------------------------------- | ------------------------------------------- | ------------------------------------------------- |
| `fedora`    | Fedora's metalink                       | everything not listed below                 | packages: the builder's `RPM-GPG-KEY-fedora-45-primary` |
| `updates`   | Fedora's metalink                       | the same, updated                           | packages: the same key                            |
| `terra`     | `https://repos.fyralabs.com/terra45`    | `mangowm`, `noctalia-greeter`, `limine`     | packages and metadata: `fedora/iso/terra-45.asc`  |
| `rpmfusion-nonfree`, `rpmfusion-nonfree-updates` | RPM Fusion's metalinks | x86_64: `xorg-x11-drv-nvidia`, `-libs`, `-cuda-libs`, `nvidia-modprobe`, `nvidia-settings` | packages: `fedora/iso/rpmfusion-nonfree-45.asc` (metadata is unsigned) |
| `stackific` | `/repo` (`.build/fedora/repo/`)         | our RPMs, and on x86_64 `kmod-nvidia-<kernel>` | nothing yet: unsigned until release signing       |

[Terra](https://terra.fyralabs.com) packages MangoWM, noctalia-greeter and Limine, which Fedora does not. Everything else comes from Fedora, Noctalia and scenefx included. kiwi writes one repo file per repository, then runs a repository's `customize` script with that file's path. `terra-repo.sh` appends two lines kiwi has no attribute for:

- `includepkgs=mangowm,noctalia-greeter,limine`. Unlimited, dnf takes Terra's `cardwire` (a GPU manager) for `gnome-shell`'s `switcheroo-control`, and Terra's builds of packages Fedora ships too. Add a package here only when Fedora lacks it. The image keeps no repo file for Terra, so installed systems update these three only once our repos are online (task 028).
- `gpgkey=file:///desc/terra-45.asc`. `repository_gpgcheck="true"` makes dnf check Terra's signed metadata, which it does only against a key named in the repo file; without it the metadata does not load. `package_gpgcheck="true"` checks each package against the same key, which kiwi imports into the image's rpmdb (`<signing>`).

`terra-45.asc` is Terra 45's public key, committed verbatim from `https://repos.fyralabs.com/terra45/key.asc`: sha256 `411e0c66b22a3f3d79651c05b13c0dbcd9edd72f6839e5e33d1fd5e1c6248f7b`, fingerprint `C2AC02124AF114F086592E3C8DDE7D14C1C23D8C` (`gpg --show-keys`). A key change is a reviewed change in git. With another key in its place (Terra 44's, for example), the build stops on Terra's signature check: `repomd.xml GPG signature verification error: Signing key not found`. Terra has one key per Fedora release, so for Fedora 46:

1. Download `https://repos.fyralabs.com/terra46/key.asc`, and compare `gpg --show-keys` with the fingerprint Terra publishes (its `terra-gpg-keys` package, or dnf's import prompt on a Fedora 46 system with Terra).
2. Commit it as `fedora/iso/terra-46.asc` in place of `terra-45.asc`, and update the file name in `config.kiwi`, `terra-repo.sh` and `AGENTS.md`.
3. Run `just fedora iso`, then `just fedora test`.

## Our packages

```sh
just fedora packages          # stackific-release, -desktop, -artwork, -boot and -installer
```

Our Fedora packages are specs in `fedora/packages/<name>/<name>.spec`. They install the same base-neutral files from `desktop/` and `branding/`, at the same paths, as the Arch PKGBUILDs (`docs/arch-iso.md`), and render them from `branding/branding.env` with the same lines. Each spec's `SourceN:` files are found by name next to the spec (Fedora-only files, such as `stackific-boot`'s plugin and `stackific-desktop`'s GNOME override) and under `desktop/`, `branding/` and `installer/`, as `just arch packages` does, so every source name must be unique across them.

The recipe runs one fresh `--privileged` builder container. In it, mock runs as container root, so it has no privileges on the host:

- `--isolation=simple`, because the container has no systemd-nspawn, which mock's default isolation needs;
- `--no-bootstrap-image`, because mock's default bootstrap pulls a container image, which would need podman inside the container.

For each spec it runs `mock --buildsrpm`, then `mock --rebuild` of that source RPM, with the config `fedora-45-<host arch>`, and copies the binary RPMs (`noarch`, and the host's architecture for `stackific-installer`) to `.build/fedora/repo/`. `createrepo_c` then writes the repo metadata. The repo is unsigned until release signing lands. The first mock call takes about two minutes, because it sets up mock's bootstrap and build roots. Later calls in the same container reuse its root cache and take about 20 seconds.

`stackific-installer` is Rust. The recipe packs the crate (`Cargo.toml`, `Cargo.lock`, `src/`, `fixtures/`) into `stackific-installer-crate.tar`, as for makepkg, and builds that spec alone with `mock --enable-network`, because `%prep` runs `cargo fetch --locked`, as the PKGBUILD's `prepare()` does; `%build` and `%check` (`cargo test`) run `--frozen`, offline. Every other spec builds offline. It has no debug package (`debug_package %{nil}`, the PKGBUILD's `!debug`). The program is the same on both bases; it picks the base from os-release (`docs/installer.md`).

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

**RPM Fusion** (task 020) supplies NVIDIA's driver for the x86_64 image. `rpmfusion-repo.sh` appends `includepkgs` with the five packages above and the `gpgkey` line, as `terra-repo.sh` does; `akmod-nvidia` is left out, because it pulls a compiler and kernel headers, so `nvidia-kmod` (which `xorg-x11-drv-nvidia` requires) resolves to our `kmod-nvidia-<kernel>`. `rpmfusion-nonfree-45.asc` is the key `rpmfusion-nonfree-release` 45-2 installs as `RPM-GPG-KEY-rpmfusion-nonfree-fedora-45` (a link to its 2020 key, which also serves Fedora 46 and 47): sha256 `7ac65a8dfeced70c8c335862b5f0696d57abac378deda34c4af78cd2220e3de6`, fingerprint `79BD B88F 9BBF 7391 0FD4 095B 6A2A F961 9484 3C65`. The image keeps no RPM Fusion repo file.

**NVIDIA's kernel modules.** RPM Fusion publishes no prebuilt `kmod-nvidia-<kernel>`: its `akmod-nvidia` builds the modules on each machine. On x86_64, `just fedora packages` builds them once, for the newest Fedora kernel (the one kiwi installs minutes later): it downloads `akmod-nvidia` and `xorg-x11-drv-nvidia-kmodsrc` from RPM Fusion, checks both with `rpmkeys -K` against the committed key, takes the source RPM `nvidia-kmod-<version>.src.rpm` out of `akmod-nvidia`, and rebuilds it in mock with the two as a local repo, `--additional-package kernel-devel-<kernel>` (the source RPM names no kernel headers when `kernels` is set), `--define "kernels <kernel>"` and `--with kmod_nvidia_open` (without it, the spec checks the build machine's GPUs with lspci to choose open or closed modules). About two minutes. The result, `kmod-nvidia-<kernel>-<version>`, goes into our repo; it provides `nvidia-kmod` and puts the modules in `/lib/modules/<kernel>/extra/nvidia/`. No compiler or kernel headers reach the image. If Fedora publishes a newer kernel between the two steps, the image's kernel has no module and the boot test's vermagic check fails: run `just fedora iso` again.

### stackific-hardware

The NVIDIA driver pick at every boot, the same generator as on Arch (`docs/hardware.md`), x86_64 only for now. Two Fedora-only files sit next to the spec: `/usr/lib/modprobe.d/nvidia-utils.conf` blacklists nouveau and nova and loads `nvidia_drm` and `nvidia_uvm` with `nvidia` (RPM Fusion blacklists nouveau only through `grubby` on GRUB's command line, which Limine systems never read), with the name of Arch's file, so the generator's `/run` file shadows it on both bases; and `/usr/lib/dracut/dracut.conf.d/99-stackific-gpu.conf` omits nouveau and nova from every initramfs (RPM Fusion's `99-nvidia-dracut.conf` omits NVIDIA's own modules). AMD and Intel keep early KMS on Fedora.

### stackific-boot

Limine on the ESP and its menu, on both images and kept on installed systems: a kernel-install drop-in (`layout=other`, `BOOT_ROOT=/boot`), the kernel-install plugin `90-stackific-limine.install` that copies each kernel to the ESP and writes `limine.conf`, and a trigger on `limine` that copies its program (`BOOTX64.EFI` or `BOOTAA64.EFI`) to the ESP. On aarch64 the plugin's entries start Fedora's EFI zboot kernel with Limine's `efi` protocol and `initrd=`, because the `linux` protocol refuses zboot. `docs/installer.md` has the details.

### stackific-desktop and stackific-artwork

They hold the same files as on Arch (`docs/arch-iso.md`, "Our packages"): the Noctalia unit, defaults and `/etc/skel` include, the wlroots and greeter profile.d scripts, the greeter's `greeter.toml` with its tmpfiles link, and the greeter's asset folder, whose links point `noctalia.svg` at `stackific-release`'s icon and `fonts` at Terra's `noctalia-greeter` fonts, which sit at the same path as on Arch; and the wallpapers with their credits and fastfetch's config. `stackific-artwork` requires `fastfetch` and `stackific-desktop` requires `noctalia-greeter` and `stackific-release`, as on Arch. `stackific-desktop` builds with `systemd-rpm-macros` for `%{_tmpfilesdir}`. On Fedora it also ships GNOME's defaults, `/usr/share/glib-2.0/schemas/20_stackific.gschema.override` (glib2's file trigger compiles it): Dahl's "An Eruption of Vesuvius" for the desktop (light and dark) and the lock screen, and `favorite-apps` with the installer first, then Files and Ptyxis. GNOME skips a favourite whose app is missing, as on aarch64 and on installed systems. The Secret Service (task 017) as on Arch: `stackific-desktop` requires `oo7-daemon` and `pam_oo7` and links `default.target.wants/oo7-daemon.service` (Fedora 45's oo7-daemon has no D-Bus activation file, and the preset that would enable it is in `fedora-release`, which the images leave out); greetd's own `/usr/lib/pam.d/greetd` already calls `pam_oo7`, so Fedora needs no PAM file of ours.

## What is in the image

- The kernel, `linux-firmware` and the default firmware of Fedora's `hardware-support` group, plus AMD and Intel microcode. The server and datacenter firmware in that group is optional there, so it stays out.
- NetworkManager with Wi-Fi, btrfs and FAT tools, SELinux enforcing with the targeted policy, and dnf5, so installed systems can update.
- The installer (`stackific-installer`) and `stackific-boot` with Limine from Terra, and Disks (`gnome-disk-utility`) with `ntfs-3g` and `ntfsprogs` (`ntfsresize`), the partition editor.
- The desktop: MangoWM and noctalia-greeter from Terra, Noctalia, greetd, foot and rofi from Fedora, and GNOME (`gnome-shell`, its Wayland session, Ptyxis and Nautilus), with the GNOME, GTK and wlr portals and PipeWire. Our three RPMs carry the defaults (the live session, below).
- `stackific-release`, not `fedora-release`. Fedora's trademark guidelines require a remix to leave out `fedora-release`, `fedora-logos` and `fedora-release-notes`; `stackific-release` takes the place of Fedora's `generic-release` (above). `gnome-shell` requires `gdm`, which requires `system-logos`; `config.kiwi` names `generic-logos` for it, or dnf would pick `fedora-logos`.
- No weak dependencies: kiwi installs only what packages require, and `config.kiwi` names the few weak dependencies users commonly need: `tzdata`, `cracklib-dicts` and `intel-npu-firmware` (task 003), and for the desktop Wi-Fi (`NetworkManager-wifi`, `wpa_supplicant`, `wireless-regdb`, which NetworkManager alone lacks on Fedora) and GNOME's fonts (`adwaita-sans-fonts`, `adwaita-mono-fonts`). Left out until the hardware tasks, as on Arch: Bluetooth, power profiles, Thunderbolt, Vulkan, VA-API and libcamera. Left out as rare on a live desktop: input methods (`ibus`), JACK and ALSA bridges, GNOME's tour, remote desktop, colour manager, parental controls and search indexer, filesystem tools beyond btrfs and FAT, and the codec libraries. Tasks 003 and 008 list them all.

## Live session

The ISO boots to a desktop. greetd (`root/etc/greetd/config.toml`, the live image's only change to a desktop package's file) logs the user `live` into MangoWM through `[initial_session]`, with no prompt. MangoWM starts `mango-session.target`, which pulls in `stackific-noctalia.service` from `stackific-desktop`, so Noctalia comes up on Dahl's "An Eruption of Vesuvius" with no setup wizard. `/etc/profile.d/stackific-wlroots.sh` lets MangoWM start on the VM's software renderer. In MangoWM's stock config, `Alt+Return` opens foot and `Alt+space` opens rofi.

After a logout, greetd runs noctalia-greeter as Fedora's greeter user `greetd` (Arch's is `greeter`), in our palette and with our logo (`docs/arch-iso.md`, "Our packages"). There, GNOME is the other session, and the session picker shows it first, where Arch's shows Mango first. GNOME never starts `mango-session.target`, so Noctalia stays out of it. Without gdm, GNOME turns its screen lock off and says so in a notification (`Screen Locking requires the GNOME display manager`). Terra's noctalia-greeter ships no tmpfiles file, so `root/etc/tmpfiles.d/noctalia-greeter.conf` creates its state directory `/var/lib/noctalia-greeter`, owned by `greetd`, before `stackific-greeter.conf` links our `greeter.toml` into it.

`gnome-shell` requires `gdm`, and Fedora's presets enable it as `display-manager.service`. `config.sh` disables it and enables greetd instead. greetd runs in `xdm_t`, the domain gdm uses (`greetd-selinux`), and `pam_selinux` moves user sessions to the user's context.

`config.sh` creates `live` with `useradd` (home from `/etc/skel`, our Noctalia include among it), in `wheel`, with an empty password. root also has an empty password, on the other consoles. tty1 belongs to greetd. noctalia-greeter refuses an empty password field, so to log `live` in from the greeter, type any character: Fedora's authselect `local` profile has `nullok`, so `pam_unix` authenticates a user with an empty password without asking for one, and what was typed is never checked.

The boot menu ("Try Stackific OS (<arch>, UEFI)", the default, and "Install Stackific OS (<arch>, UEFI)", which `edit-boot.sh` adds; `docs/installer.md`), volume ID (`STACKIFIC_LIVE`, not the installed ESP's FAT label `STACKIFIC`, which the live initramfs's `root=live:CDLABEL=` would otherwise find on an installed drive) and ISO metadata say Stackific OS; the kernel, GRUB and every other binary are unmodified Fedora packages.

## Boot test

```sh
just fedora test              # boots stackific-0.0.0-dev-fedora-<arch>.iso
just fedora test 1.2.3
```

It runs the same `tests/vm/boot.sh` as `just arch test` (see `docs/arch-iso.md`), with the builder's raw firmware pair (`uefi_code` and `uefi_vars` in `fedora/vars.just`, passed as `OVMF_CODE` and `OVMF_VARS`). After the checks every image passes, `boot.sh` sources `tests/vm/fedora.sh`, which both Fedora architectures pass. It makes the checks of `tests/vm/arch.sh`, with Fedora's names, and adds Terra and SELinux:

- `/etc/os-release` reads `ID=stackific`, `ID_LIKE=fedora`, `VERSION_ID=45`, `PRETTY_NAME="Stackific OS"` and `LOGO=stackific`; `hostnamectl` shows `Operating System: Stackific OS`; rpm says `generic-release-common` owns `/etc/os-release` (the link) and `stackific-release` owns `/usr/lib/os-release`; no `fedora-release*`, `fedora-logos*` or `fedora-release-notes` is installed; the `stackific` icon is filled `#007a55`;
- the only packages with vendor `Terra` are `mangowm`, `noctalia-greeter` and `limine`; `dnf5` is installed;
- GNOME's settings for `live` show our wallpaper (`picture-uri`, `picture-uri-dark`, the screensaver's) and the installer first in `favorite-apps`;
- `getenforce` prints `Enforcing`;
- greetd is active and `display-manager.service` links to it; `stackific-noctalia.service` is active for `live`, and `mango` runs as `live`. The unit check runs `systemctl --user` as `live` through `runuser`, not `systemctl --user -M live@` as on Arch: that runs `systemd-run -P`, which hands the test shell's vsock socket to PID 1 and dbus-broker, and Fedora's policy denies them that socket, silently through dontaudit rules (`Failed to start transient service unit: Connection reset by peer`);
- the MangoWM and GNOME session files are present;
- the ten wallpapers are installed, `live`'s `00-stackific.toml` matches `/etc/skel`'s, and `noctalia config export merged` (as `live`) shows the wizard off, the wallpaper folder and the default wallpaper;
- `/var/lib/noctalia-greeter` belongs to `greetd`, its `greeter.toml` links to ours, which has our accent as `primary`, and the greeter's asset folder links `noctalia.svg` to our icon;
- `fastfetch` (as `live`) prints the logo's last line in the accent colour, and `OS: Stackific OS`.

It saves `screen.png` (the live desktop), then logs `live` out with `loginctl terminate-user live`. It checks that noctalia-greeter runs as `greetd` with `NOCTALIA_GREETER_ASSETS_DIR` set to our asset folder, and saves `greeter.png`. Last, after GNOME (below), the journal of the boot has no `avc:  denied` line. journald reads the kernel's audit records itself (`systemd-journald-audit.socket`), so the kernel's rate limit on printing them does not hide any. All three files go to `.build/fedora/<arch>/`, with `serial.log`.

After the greeter, `fedora.sh` checks the installer's launcher and units, `stackific-boot`'s plugin, Limine's program for the machine from Terra, Disks with `ntfsresize`, and logs `live` into GNOME through the greeter (`a`, Enter: GNOME is first in its session list), saving `gnome.png`: GNOME's overview with our wallpaper and the installer first in the dash. Not through greetd's `[initial_session]`, as the Arch test does for Plasma: that opens a logind session of type `tty`, and GNOME Shell starts only in a `wayland` one (`Failed to find any matching session`), which noctalia-greeter asks greetd for. Then, again, no `avc:  denied` line.

To check GNOME by hand:

1. Boot `.build/fedora/<arch>/stackific-0.0.0-dev-fedora-<arch>.iso` in a VM with a display, UEFI firmware and 4 GiB of RAM (virt-manager or GNOME Boxes, for example). MangoWM with Noctalia's bar appears.
2. Press `Alt+Return`. A foot terminal opens. Run `loginctl terminate-user live`.
3. At noctalia-greeter, pick the user `live` and the GNOME session, type any character as the password (above) and press Enter.
4. GNOME's desktop appears. In Ptyxis, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.

The test shell is a service running `systemd-socket-activate`, not a socket unit. With a socket unit, PID 1 labels the listening vsock socket for the shell it will start (`initrc_t`), and Fedora's policy denies PID 1 that socket. The service runs as `unconfined_service_t`, which may listen on vsock.

## Install test

```sh
just fedora test-install      # needs stackific-0.0.0-dev-fedora-<arch>.iso and its manifest
```

As `just arch test-install` (`docs/arch-iso.md`): it checks the ISO's `boot/grub2/grub.cfg` for Try (the default) and Install, boots the ISO with a blank 64 GiB disk through `tests/vm/install.sh`, reads the free-space install's ESP with `minfo`, and boots the disk alone with the first boot's firmware variables through `tests/vm/installed.sh`. Before the second boot it downloads the image's `kernel-core` (Fedora) and `limine` (Terra) at the manifest's versions, checks both with `rpmkeys -K` against Fedora 45's and Terra 45's keys, and copies them onto the ESP with `mcopy`, for the reinstall checks. Fedora's `updates` keeps only the newest kernel, so the download fails once the image's kernel is superseded: build the image again. `docs/installer.md` lists what the checks cover.

## aarch64

The aarch64 image is built and tested on aarch64 Linux: the `fedora-aarch64` jobs of the `images` workflow (on merges to `main` and by hand) and of the `release` workflow run it on the Mac's runner VM, and upload it like the other images (`docs/releases.md`). The Mac has one runner, so `arch-aarch64` and `fedora-aarch64` run one after the other. It is the same desktop live ISO as x86_64, from the same description, builder, recipes and checks. What differs:

- `config.kiwi` keeps twelve packages to one architecture with kiwi's `arch` attribute, which kiwi compares with the build host's architecture. On x86_64 they are `grub2-efi-x64`, `grub2-efi-x64-cdboot` and `grub2-efi-x64-modules`, the x86 CPU microcode, `amd-ucode-firmware` and `microcode_ctl`, and NVIDIA (`stackific-hardware`, `xorg-x11-drv-nvidia`, `xorg-x11-drv-nvidia-libs`, `nvidia-kmod`; task 022 brings them to aarch64). Fedora has no `microcode_ctl` on aarch64, and AMD's microcode serves only x86 CPUs. On aarch64 they are `grub2-efi-aa64`, `grub2-efi-aa64-cdboot` and `grub2-efi-aa64-modules`. Every other line serves both images, so the two package sets cannot drift apart. The firmware packages are noarch and stay on both, as on Arch Linux ARM.
- The kernel command line names both serial consoles, `ttyS0` (x86_64) and `ttyAMA0` (aarch64), as on Arch; the kernel ignores the one the machine lacks.
- The builder installs `qemu-system-aarch64-core` with `edk2-aarch64` (AAVMF) and `qemu-device-display-virtio-gpu-pci`, the display `boot.sh` uses with KVM on `virt`, which QEMU does not require. The base image's pin is a manifest list, so the same digest gives the arm64 image.
- The boot test gives QEMU `edk2-aarch64`'s `QEMU_EFI-pflash.raw` and `vars-template-pflash.raw`. QEMU's `virt` machine takes only 64 MiB flash images, and `QEMU_EFI.fd` is 3 MiB.
- The runner VM enforces SELinux, which keeps unprivileged containers off `/dev/vhost-vsock` and bind mounts, so `just fedora test` runs with `--security-opt label=disable`, as `just arch test` does. It is also why the ISO build mounts its own work tmpfs (above).
- Terra's aarch64 `mangowm` and `noctalia-greeter` have the same versions as on x86_64 and are signed with the same Terra 45 key. On `virtio-gpu-pci`, Mesa renders in software, which `WLR_RENDERER_ALLOW_SOFTWARE=1` lets MangoWM use.

## Move the builder digest

The builder's base image is pinned by digest in `fedora/vars.just` (`builder_base`). To move it:

1. Look up the new digest of `registry.fedoraproject.org/fedora:45` (the `docker-content-digest` header of its manifest list, which serves both architectures).
2. Set `builder_base` to it.
3. Run `just fedora iso`, then `just fedora test`, on the PC and in the Mac's runner VM. Both must pass on both before the change lands.

podman's layer cache keeps the builder's kiwi and QEMU until the digest moves.
