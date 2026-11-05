# 008: Fedora 45 x86_64 live ISO boots to MangoWM with Noctalia, with GNOME

- [ ] Done

Base: fedora

## Ask

```text title="Prompt" wrap
Turn the Fedora 45 x86_64 console live ISO from task 003 into a desktop live ISO that mirrors the Arch one from task 002. It boots straight into MangoWM with Noctalia: greetd logs the user `live` (empty password, group `wheel`) in through `initial_session`, and noctalia-greeter is the login screen after a logout, where GNOME is the second session. Take MangoWM from Terra, with Terra's repo limited by `includepkgs` to the packages we need and Terra's signing key checked at build time; take everything else unmodified from Fedora's repos. Install our three Fedora packages from task 007 (stackific-release, stackific-desktop, stackific-artwork) from the local repo that `just fedora packages` builds, so `stackific-noctalia.service` starts Noctalia and `WLR_RENDERER_ALLOW_SOFTWARE=1` lets MangoWM start on software rendering. Keep Fedora's trademark rules: no fedora-release*, fedora-logos or fedora-release-notes in the image. The login screen carries our branding, not Noctalia's, as on Arch: stackific-desktop's greeter defaults (from task 006, packaged by 007) use our palette and Dahl's Vesuvius through a tmpfiles link to /var/lib/noctalia-greeter/greeter.toml, and put our green-on-transparent logo where Noctalia's was through the greeter's NOCTALIA_GREETER_ASSETS_DIR, set in /etc/profile.d/stackific-greeter.sh, which greetd sources before the greeter. Extend the QEMU boot test (tests/vm/fedora-x86_64.sh) with the checks the Arch desktop test makes, plus the image's identity and SELinux, with a screenshot of the desktop and of the greeter. Report the ISO size against the 2 GiB target. Settle how Terra's key is fetched and pinned, and how kiwi limits a repository to named packages.
```

## Goal
`just fedora iso` builds a Fedora 45 x86_64 live ISO that boots into MangoWM with Noctalia as the user `live`, with GNOME available from noctalia-greeter, and identifies as Stackific OS. `just fedora test` proves that session runs with SELinux enforcing and that the greeter shows after a logout.

## Context
Task 003 built a console live ISO from `fedora/iso/` (`config.kiwi`, `config.sh`, `root/`) with kiwi in the builder `fedora/Containerfile`, pinned in `fedora/vars.just`. It autologs root on tty1 through `fedora/iso/root/etc/systemd/system/getty.target.wants/getty@tty1.service` and the drop-in `getty@tty1.service.d/autologin.conf`, uses `generic-release` for os-release, and `tests/vm/fedora-x86_64.sh` checks root on tty1 and `getenforce`. That ISO is 922 MiB. Task 002 did the same desktop step on Arch: `arch/iso/airootfs/etc/greetd/config.toml`, the live user, `display-manager.service` → greetd, and the checks in `tests/vm/arch-x86_64.sh`.

This task depends on two tasks that must be merged first:
- 006 adds the base-neutral branding file `branding/branding.env` and the artwork package `stackific-artwork`: the wallpapers under `/usr/share/backgrounds/stackific/` and fastfetch's logo config `/etc/xdg/fastfetch/config.jsonc`. It also adds Noctalia's defaults to `stackific-desktop`: `/usr/share/stackific/noctalia.toml`, with the setup wizard off, the wallpaper folder and Dahl's Vesuvius as the default wallpaper, included from each user's `~/.config/noctalia/00-stackific.toml`, which comes from `/etc/skel`. This task only installs the packages.
- 007 adds `just fedora packages`, which builds the RPMs `stackific-release`, `stackific-desktop` and `stackific-artwork` with mock into the local repo `.build/fedora/repo/` (createrepo_c metadata, unsigned until release signing), with a `fedora/vars.just` variable for that path. `stackific-release` replaces `generic-release` and keeps `generic-release-common`; this task follows 007's mechanism. `stackific-desktop` holds `stackific-noctalia.service` (wanted by `mango-session.target`), `/etc/profile.d/stackific-wlroots.sh` and 006's Noctalia defaults, as on Arch.

Facts, all checked 2026-11-05. "Dry run" means `dnf install --assumeno --installroot=… --releasever=45 --setopt=install_weak_deps=False` in the pinned builder base `registry.fedoraproject.org/fedora:45@sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd` on the operator's PC, against Fedora's live repos and Terra 45. "Package contents" means the RPM fetched with `dnf download` and unpacked.

Package sources and versions (`dnf repoquery`; they move with the repos, and the manifest records what each build got):
- Terra only: `mangowm` 0.18.5-1.fc45, `noctalia-greeter` 1.7.0-2.fc45. Both have `Vendor: Terra` and are signed by key ID `8dde7d14c1c23d8c` (`rpm -qp --qf '%{vendor} %{rsaheader:pgpsig}'`). Terra also has `limine` 12.9.6-1.fc45, which the installer tasks need, not this one.
- Fedora 45, not Terra: `noctalia` 5.4.1-1.fc45 (updates, Fedora Project; Terra has only `noctalia-legacy`, `noctalia-nightly` and `noctalia-qs`), `greetd` 0.10.3-12.fc45 and `greetd-selinux` 0.10.3-12.fc45 (fedora), `scenefx` 0.5-1.fc45 (fedora; Terra has the same EVR, built earlier), `wlroots` 0.20.2-2.fc45.
- GNOME and the rest, Fedora 45: `gnome-shell` 51.1-1.fc45, `mutter` 51.1-1.fc45, `gnome-session-wayland-session` 51.0-1.fc45, `gdm` 1:51.1.1-1.fc45, `nautilus` 51.1-2.fc45, `ptyxis` 50.1-6.fc45, `foot` 1.28.0-1.fc45, `rofi` 2.0.0-3.fc45, `xdg-desktop-portal` 1.22.1-2.fc45, `xdg-desktop-portal-gnome` 51.0-2.fc45, `xdg-desktop-portal-gtk` 1.15.3-6.fc45, `xdg-desktop-portal-wlr` 0.8.4-1.fc45, `pipewire` and `pipewire-pulseaudio` 1.6.11-1.fc45, `wireplumber` 0.5.19-1.fc45, `mesa-dri-drivers` 26.2.6-1.fc45 (llvmpipe), `generic-logos` 18.0.0-29.fc45.

Why Terra must be limited (dry run, Terra unlimited): `gnome-shell` requires `switcheroo-control`, and Terra's `cardwire` 0.12.3-3.fc45 (a GPU manager using eBPF LSM hooks) provides it, so dnf took `cardwire` from Terra over Fedora's `switcheroo-control` 3.0-9.fc45. With `includepkgs=mangowm,noctalia-greeter` on Terra, the same dry run takes `switcheroo-control` and `scenefx` from Fedora, and `dnf repoquery --repo terra` lists only `mangowm` and `noctalia-greeter`.

How kiwi limits a repository and checks its key (kiwi-cli 11.1.1-2.fc45 in `localhost/stackific-fedora-builder:45`):
- `<repository customize="script">`: "The script receives the repo file according to the used package manager as a parameter such that custom modifications can be placed when needed" (`schema/kiwi.rnc` line 1220). `repository/dnf5.py` `add_repo` writes the repo file (one section, with `repo_gpgcheck` and `gpgcheck` from the attributes below and no `gpgkey`), then runs `bash --norc <script> <repo file>` (`repository/base.py` line 162). A relative script path resolves against the description directory (`system/prepare.py` line 688). So the script appends `includepkgs=` to Terra's repo file; kiwi has no `includepkgs` attribute of its own.
- `package_gpgcheck` and `repository_gpgcheck` default to false (`schema/kiwi.rnc` lines 1209–1219). `<signing key="file://…"/>` keys are imported into the image's rpmdb before packages install (`xml_state.py` `get_repositories_signing_keys`, `system/prepare.py` line 160), which is how task 003 checks Fedora's packages.
- dnf5 in the pinned base, with a repo file like kiwi's (probe, installing `terra-gpg-keys` from Terra into an empty root):
  - `gpgcheck=1`, Terra's key not in the root's rpmdb: `Transaction failed: Signature verification failed`.
  - `gpgcheck=1`, key imported into the root's rpmdb: installs.
  - `repo_gpgcheck=1` without `gpgkey=`: Terra's metadata does not load (`No match for argument`). With `gpgkey=file:///…/terra45.asc` added: loads and installs. So the customize script also appends `gpgkey=` for `repository_gpgcheck="true"`.
- Terra's repo definition: `baseurl=https://repos.fyralabs.com/terra$releasever`, `gpgcheck=1`, `repo_gpgcheck=1`, `gpgkey=https://repos.fyralabs.com/terra$releasever/key.asc` (https://github.com/terrapkg/subatomic-repos/blob/main/terra.repo, commit 0ae117e).
- Terra 45's key, https://repos.fyralabs.com/terra45/key.asc: 831 bytes, sha256 `411e0c66b22a3f3d79651c05b13c0dbcd9edd72f6839e5e33d1fd5e1c6248f7b`, ed25519, created 2026-07-23, user ID `Terra 45 <security@fyralabs.com>`, fingerprint `C2AC02124AF114F086592E3C8DDE7D14C1C23D8C` (`gpg --show-keys`). dnf showed the same fingerprint when it imported the key. Committing this file pins it: a key change in git is a reviewed change.

greetd on Fedora (package contents of greetd 0.10.3-12.fc45, and https://src.fedoraproject.org/rpms/greetd/blob/f45/f/greetd.spec):
- The greeter user is `greetd` (`/usr/lib/sysusers.d/greetd.conf`: `u greetd - "greetd daemon" /var/lib/greetd`), not Arch's `greeter`, and it is in no `video` group. `/etc/greetd/config.toml` ships `vt = 1` and `user = "greetd"`.
- PAM: `/usr/lib/pam.d/greetd` (for user sessions) has `pam_selinux.so close`/`open` and includes `system-auth` and `postlogin`. `/usr/lib/pam.d/greetd-greeter` (for the greeter) ends with `session optional pam_systemd.so`, so the greeter gets a logind session and its seat devices; it needs no `video` group. noctalia-greeter's optional `setup_greetd_pam.sh` would add pam_systemd to `/etc/pam.d/greetd`; Fedora's `system-auth` (authselect 1.8.0 `local` profile) already has `-session optional pam_systemd.so`, so this task does not run it.
- authselect's `local` profile has `pam_unix.so … nullok` unless `without-nullok` is selected (`/usr/share/authselect/default/local/system-auth` line 8), so an empty password logs in, as on Arch.
- `greetd.service`: `Conflicts=getty@tty1.service kmsconvt@tty1.service`, `[Install] Alias=display-manager.service`.
- SELinux: `greetd` requires `(greetd-selinux = 0.10.3-12.fc45 if selinux-policy-targeted)`, so the policy module comes with it. Its file contexts label `/usr/bin/greetd` `xdm_exec_t`, `/etc/greetd(/.*)?` `xdm_etc_t` and `/var/lib/greetd(/.*)?` `xdm_var_lib_t` (`greetd.fc` in dist-git f45), so greetd runs as `xdm_t`, the domain GDM uses, and `pam_selinux` moves user sessions to the user's context.
- greetd sources `/etc/profile` before every command (`source_profile`, default true; greetd(5) in the package), and Fedora's `/etc/profile` sources `/etc/profile.d/*.sh`, so `stackific-wlroots.sh` reaches every session.
- `[initial_session]` runs only on greetd's first start since boot (greetd(5)).

gdm and presets:
- `gnome-shell` 51.1 requires `gdm` (`dnf repoquery --whatrequires gdm`: gdm-devel, gnome-flashback, gnome-initial-setup, gnome-panel, gnome-shell), so gdm is in the image whether wanted or not.
- `gdm` requires `system-logos`. Only `fedora-logos` and `generic-logos` provide it. The dry run without `generic-logos` named pulls `fedora-logos` 42.0.1-6.fc45, which Fedora's trademark guidelines forbid in a remix; with `generic-logos` named, no `fedora-*` logos or release package is in the transaction.
- `gdm.service` has only `[Install] Alias=display-manager.service`. `redhat-systemd-presets-common` 102-3.fc45 `85-display-manager.preset` has `enable gdm.service`, and nothing enables greetd (`99-default-disable.preset`). gdm's `%post` applies the preset, so gdm holds `display-manager.service` after the packages install. `systemctl disable gdm.service`, then `systemctl enable greetd.service` in `config.sh` hands it to greetd.
- The same presets enable `NetworkManager.service`, `accounts-daemon`, `upower` and `switcheroo-control`. They come today through `generic-release`'s `Requires: distribution-systemd-presets`; 007's `stackific-release` must keep that requirement, which the NetworkManager check in `boot.sh` proves.

MangoWM, Noctalia and GNOME:
- Terra's `mangowm` 0.18.5 ships `/usr/bin/mango`, `/etc/mango/config.conf` and `config.toml`, `/usr/share/wayland-sessions/mango.desktop` (`Exec=mango`), `/usr/lib/systemd/user/mango-session.target` (`BindsTo=graphical-session.target`) and `/usr/share/xdg-desktop-portal/mango-portals.conf` (`default=gtk`; wlr for ScreenCast and Screenshot). Its stock config binds `Alt+Return` to `foot` and `Alt+space` to `rofi -show drun` (lines 181–182) and has no `exec_once`. Package contents.
- MangoWM 0.18.5 still imports the environment and starts `mango-session.target` when `DBUS_SESSION_BUS_ADDRESS` is set (https://github.com/mangowm/mango/blob/0.18.5/src/main.c lines 293–358), so `stackific-noctalia.service` starts Noctalia unchanged from Arch.
- Fedora's `noctalia` 5.4.1 ships `/usr/bin/noctalia` and no unit; its hard requirements outside libraries are `git-core`, `hicolor-icon-theme`, `pipewire` and `sound-theme-freedesktop`.
- Terra's `noctalia-greeter` 1.7.0-2 ships `/usr/bin/noctalia-greeter-session`, `noctalia-greeter-compositor` and `noctalia-greeter`, and requires `greetd` and `dbus`. Unlike Arch's 1.7.0-1 it ships no tmpfiles file, so `/var/lib/noctalia-greeter` (its appearance sync dir) does not exist; the binary logs `failed to open appearance manifest '{}'` when its files are missing (`strings`), which suggests it starts without them. Not yet measured in a boot.
- `/usr/bin/noctalia-greeter`'s comm name is cut to 15 characters, so tests match the command line (proc(5)), as on Arch.
- `gnome-session-wayland-session` 51.0 ships `/usr/share/wayland-sessions/gnome.desktop` with `Exec=/usr/bin/gnome-session`. GNOME starts its session through the systemd user instance (GNOME 49 release notes, https://linuxiac.com/gnome-49-will-require-deeper-systemd-integration/), which greetd's logind session provides. A GNOME login from greetd without GDM was not measured, so it is a manual step in Tests. GNOME never starts `mango-session.target`, so Noctalia stays out of GNOME.
- An earlier open question asked what SDDM and GDM need to start the MangoWM session; with greetd as the display manager, neither starts it. Noctalia's IPC is not needed: the tests use unit state and processes, as 002's do.

Size and weak dependencies (dry run):
- Task 003's console set installs 214 packages (972 MiB installed, 663 MiB download). Adding this task's packages: 655 packages, about 2 GiB installed, about 1 GiB download. The ISO should stay under 2048 MiB (003's 972 MiB installed made a 922 MiB ISO); the build measures it.
- It resolves without Fedora's Cisco openh264 repo, which kiwi's description does not have.
- With weak dependencies on, the same set grows to 800 packages and about 3 GiB installed. The operator's rule from task 003 (2026-11-05): keep weak dependencies a regular desktop or laptop user commonly needs on a live system, leave out the rare and exotic.
  - Kept, named in the image list:
    - `NetworkManager-wifi`, `wpa_supplicant`, `wireless-regdb`: Wi-Fi. NetworkManager alone has no Wi-Fi on Fedora; Arch's `networkmanager` 1.58.1-2 depends on `wpa_supplicant` (`pacman -Si` in the Arch builder), so this is parity with 002.
    - `adwaita-sans-fonts`, `adwaita-mono-fonts`: GNOME's UI and monospace fonts. Without them, GNOME and foot fall back to the Noto and Cantarell fonts that `gnome-shell`'s requirements pull.
  - Left out, as 002 left them out on Arch (the hardware tasks bring them): `bluez`, `power-profiles-daemon`, `bolt`, `mesa-vulkan-drivers`, the VA-API and Intel media stacks (`intel-mediasdk`, `intel-vpl-gpu-rt`, `libvpl`), `libcamera`, `pipewire-plugin-libcamera`, `ddcutil`, `i2c-tools`.
  - Left out as rare on a live desktop: `ibus` and its parts, `pipewire-alsa`, `pipewire-jack-audio-connection-kit`, `gnome-tour`, `gnome-remote-desktop`, `gnome-color-manager`, `malcontent-*`, `rygel`, `tinysparql`, `evolution-ews-*`, `flatpak-libs`, `ostree-libs`, `gvfs-fuse`, `pinentry-gnome3`, `hunspell-en*`, the filesystem tools (`e2fsprogs`, `exfatprogs`, `ntfsprogs`, `f2fs-tools`, `xfsprogs`, `nilfs-utils`, `udftools`), `7zip`, `unzip`, `xdg-utils`, `usbutils`, `iw`, `slurp`, `xrandr`, the multimedia codec libraries (`libavcodec-free` and its kin, `gstreamer1-plugins-bad-free`), and everything 003 already left out (`systemd-networkd`, `systemd-resolved`, `tpm2-*`, `grubby`, `kpartx`, `memstrack`, `rpm-plugin-audit`, `crypto-policies-scripts`, `pkcs11-provider`, `gawk-all-langpacks`, `mkpasswd`, `whois-nls`, `wcurl`, `ntfs-3g-system-compression`, `python-unversioned-command`, `lsb_release`, `logrotate`).
- The kiwi work tmpfs held 2.0 GiB at the end of 003's build and defaults to half the host's RAM. This image roughly doubles it; if the build stops with `No space left on device` under `/work`, set an explicit `size=` on the tmpfs and say so in the review record.

AGENTS.md rules this task applies:
- MangoWM and Noctalia are unmodified packages; MangoWM's config stays stock; our defaults ship in packages;
- Terra's repo is limited with `includepkgs` and its signing key is checked at build time;
- Fedora images never ship `fedora-release*`, `fedora-logos` or `fedora-release-notes`;
- live images autologin `live` (empty password, group `wheel`) through greetd's `initial_session`, with noctalia-greeter as the login screen;
- every image records its manifest and reports its size; never `NOPASSWD` or a polkit rule that skips authentication.

Left out, for later tasks:
- Bluetooth, power profiles, Thunderbolt, Vulkan, VA-API, libcamera and NVIDIA (the hardware tasks);
- an automated GNOME login through greetd's IPC (manual here, like Plasma in 002);
- a polkit agent in MangoWM, `sudo` for `live`, `rtkit`, a secrets service (`gnome-keyring`), and Noctalia defaults that skip its first-run dialog;
- input methods (`ibus`) and the weak dependencies listed above;
- Terra's `limine` (the installer tasks add it to `includepkgs`);
- the Fedora aarch64 desktop ISO (task 010);
- a package cache shared by ISO builds.

## Inputs
None from earlier material. The ask restates the operator's plan for this task (a Fedora 45 x86_64 desktop live ISO mirroring 002, MangoWM from Terra with `includepkgs` and a checked key, Noctalia, GNOME, greetd with noctalia-greeter). The design choices are made in this task: a committed Terra key and a kiwi `customize` script for `includepkgs` and `gpgkey`; gdm installed but disabled; the live user made in `config.sh`.

The operator chose, in this session (2026-11-05):
- `scenefx` comes from Fedora rather than Terra: Fedora ships the same 0.5-1.fc45, and Terra then carries only two packages;
- Terra gets both package and repository signature checks;
- `generic-logos` provides `system-logos`;
- the only optional packages are Wi-Fi and the Adwaita fonts. The rest waits for the hardware tasks, as on Arch;
- the login screen shows our branding instead of Noctalia's, the same as on Arch: our logo in place of Noctalia's, our palette and Dahl's Vesuvius.
- after the build (2026-11-05): `live` keeps its empty password. noctalia-greeter refuses an empty field, so users type any character there; a user page and AGENTS.md say so, for all four images.

## Changes
- `fedora/iso/terra-45.asc` (new): Terra 45's public key, downloaded from https://repos.fyralabs.com/terra45/key.asc and committed verbatim. Before committing, `sha256sum` prints `411e0c66b22a3f3d79651c05b13c0dbcd9edd72f6839e5e33d1fd5e1c6248f7b` and `gpg --show-keys` prints fingerprint `C2AC02124AF114F086592E3C8DDE7D14C1C23D8C`.
- `fedora/iso/terra-repo.sh` (new): kiwi's customize script for Terra, with `set -euo pipefail`. It appends `includepkgs=mangowm,noctalia-greeter` and `gpgkey=file:///desc/terra-45.asc` to the repo file `$1`. A header comment says why (the `cardwire` fact above) and that kiwi passes the repo file.
- `fedora/iso/config.kiwi`:
  - a `terra` repository: `type="rpm-md"`, `package_gpgcheck="true"`, `repository_gpgcheck="true"`, `customize="terra-repo.sh"`, source `https://repos.fyralabs.com/terra$releasever` with `<signing key="file:///desc/terra-45.asc"/>`;
  - a `stackific` repository: source `dir:///repo`, with a comment that it is unsigned until release signing;
  - bootstrap packages: `stackific-release` in place of `generic-release`, as 007 specifies; update the comment above them;
  - image packages, added: `generic-logos`, `stackific-desktop`, `stackific-artwork`, `mangowm`, `noctalia`, `greetd`, `noctalia-greeter`, `foot`, `rofi`, `gnome-shell`, `gnome-session-wayland-session`, `ptyxis`, `nautilus`, `xdg-desktop-portal-gnome`, `xdg-desktop-portal-gtk`, `xdg-desktop-portal-wlr`, `pipewire`, `pipewire-pulseaudio`, `wireplumber`, `NetworkManager-wifi`, `wpa_supplicant`, `wireless-regdb`, `adwaita-sans-fonts`, `adwaita-mono-fonts`, each group with a one-line comment (why `generic-logos`: gdm requires `system-logos`);
  - the description's `<specification>` and the top comment say desktop live ISO.
- `fedora/iso/config.sh`: after `passwd -d root`, create the live user and hand the display manager to greetd:
  - `useradd --create-home --groups wheel --comment 'Live user' live`, then `passwd -d live`;
  - `systemctl disable gdm.service`, then `systemctl enable greetd.service`, with a comment (gdm is only gnome-shell's dependency, and Fedora's preset enables it).
- `fedora/iso/root/etc/greetd/config.toml` (new), for the live image only: `[terminal] vt = 1`; `[initial_session] command = "mango"`, `user = "live"`; `[default_session] command = "/usr/bin/noctalia-greeter-session"`, `user = "greetd"`. A header comment like Arch's, naming GNOME as the other session.
- `fedora/iso/root/etc/systemd/system/getty.target.wants/getty@tty1.service` and `fedora/iso/root/etc/systemd/system/getty@tty1.service.d/autologin.conf`: delete them, because greetd owns tty1.
- `fedora/iso/root/etc/tmpfiles.d/noctalia-greeter.conf` (new): `d /var/lib/noctalia-greeter 0750 greetd greetd -`, with a comment saying why: Terra's package ships no tmpfiles file, and the greeter's state directory must belong to Fedora's greeter user `greetd` so it can write `sync.toml`. Without it, `stackific-greeter.conf`'s `L` line would create the directory owned by root. tmpfiles handles a directory before the paths inside it, so the `d` line runs before the link.
- `fedora/justfile`: `iso` depends on `packages` (from 007) and mounts 007's repo variable read-only at `/repo`; the `test` comment lists `greeter.png`.
- `tests/vm/fedora-x86_64.sh`: replace the `who` root-on-tty1 check; keep `getenforce`. In order:
  1. `. /etc/os-release && echo "$ID/$ID_LIKE/$VERSION_ID/$PRETTY_NAME/$LOGO"` prints `stackific/fedora/45/Stackific OS/stackific` (with `ID_LIKE` and `VERSION_ID`, Fedora's per-base fields, as 007 sets them; Arch checks `BUILD_ID` there), and `hostnamectl` shows `Operating System: Stackific OS`.
  2. `rpm -qf --qf '%{name}\n' /etc/os-release /usr/lib/os-release | paste -sd ' '` prints `generic-release-common stackific-release`: rpm names the owner of the `/etc/os-release` link itself, not of its target (measured in the pinned base, where it prints `fedora-release-common`), as Arch's check names `filesystem` for `/usr/lib/os-release`. `rpm -qa 'fedora-release*' 'fedora-logos*' fedora-release-notes | wc -l` prints `0`, and `grep -c 'fill="#007a55"' /usr/share/icons/hicolor/scalable/apps/stackific.svg` prints `1`.
  3. `rpm -qa --qf '%{vendor} %{name}\n' | awk '$1 == "Terra" {print $2}' | sort | paste -sd ' '` prints `mangowm noctalia-greeter`.
  4. `getenforce` prints `Enforcing`.
  5. `systemctl is-active greetd` prints `active`; `readlink /etc/systemd/system/display-manager.service` prints `/usr/lib/systemd/system/greetd.service`.
  6. Within 120 s, `runuser -u live -- env XDG_RUNTIME_DIR=/run/user/1000 systemctl --user is-active stackific-noctalia.service` prints `active`; `pgrep -u live -x mango` prints a PID. Not `systemctl --user -M live@`, as on Arch: it runs `systemd-run -P`, which hands the test shell's vsock socket to PID 1 and dbus-broker, and Fedora's policy denies `init_t` and `system_dbusd_t` that socket (`unconfined_service_t`), silently through dontaudit rules. Measured in the VM: `Failed to start transient service unit: Connection reset by peer`; it prints `active` after `setenforce 0`, and `semodule -DB` shows the denials.
  7. `/usr/share/wayland-sessions/mango.desktop` and `gnome.desktop` both exist.
  8. The artwork and defaults checks of `tests/vm/arch-x86_64.sh`, unchanged, because AGENTS.md makes the four images one product with the same boot-test checks and every image's boot test checks fastfetch's logo: ten wallpapers in `/usr/share/backgrounds/stackific/`, `live`'s `00-stackific.toml` matching `/etc/skel`'s, `noctalia config export merged` as `live` showing the wizard off, the wallpaper folder and Vesuvius, `primary = "#007a55"` in `/usr/share/stackific/greeter.toml`, `greeter-assets/noctalia.svg` resolving to our icon, `fastfetch --pipe false` as `live` printing the logo's last line in `38;2;0;122;85`, and `fastfetch --pipe true` printing `OS: Stackific OS`. Wait for Noctalia to draw, then save `screen.png`.
  9. `readlink /var/lib/noctalia-greeter/greeter.toml` prints `/usr/share/stackific/greeter.toml`, and `stat -c %U /var/lib/noctalia-greeter` prints `greetd`. After the logout in the next step, the greeter's `/proc/<pid>/environ` has `NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets`, as in `tests/vm/arch-x86_64.sh`.
  10. `loginctl terminate-user live`; within 60 s, `pgrep -u greetd -f '^/usr/bin/noctalia-greeter( |$)'` prints a PID; wait, then save `greeter.png`.
  11. `journalctl -b | grep -c 'avc:  denied'` prints `0` (two spaces, as the kernel writes it).

  `tests/vm/boot.sh` stays as it is (it already removes `greeter.png` before a run). Raise its `-m` only if the session needs more than 4096 MiB, and say so in the review record.
- `docs/fedora-iso.md`: the Terra repository (what `includepkgs` allows and why, the committed key, how to replace it for Fedora 46), the `stackific` local repo and the `packages` step, the live session (user `live`, MangoWM with Noctalia, GNOME from the greeter, `Alt+Return` foot and `Alt+space` rofi, gdm disabled), the kept and left-out weak dependencies, and the new checks and `greeter.png`. No user docs: nothing is released.
- `AGENTS.md`, Builds:
  - replace "Fedora takes MangoWM, scenefx and Limine from Terra. Terra's repo file is limited with `includepkgs`, and its signing key is checked at build time." with: Fedora takes MangoWM and noctalia-greeter from Terra (Limine joins with the installer); scenefx and Noctalia come from Fedora. `fedora/iso/terra-repo.sh` limits Terra with `includepkgs`, and packages and metadata are checked against Terra's key committed as `fedora/iso/terra-45.asc`;
  - in the Fedora trademark line, drop the `generic-release` clause if 007 left it, and add that `generic-logos` provides `system-logos`.

## Acceptance criteria

Must:
- `just fedora iso` exits 0 and its last line prints the ISO size in MiB. The manifest `.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.packages.txt` lists `stackific-release`, `stackific-desktop`, `stackific-artwork`, `mangowm`, `noctalia`, `noctalia-greeter`, `greetd`, `greetd-selinux`, `gnome-shell`, `generic-logos` and `NetworkManager-wifi`, and no `fedora-release`, `fedora-release-common`, `fedora-release-identity-*`, `fedora-logos`, `fedora-release-notes`, `generic-release` or `cardwire`.
- With a different key in `fedora/iso/terra-45.asc` (Terra 44's, from https://repos.fyralabs.com/terra44/key.asc), `just fedora iso` fails on Terra's signature check. Run once during /s-build, then restore the key; the review record quotes the error.
- `just fedora test` exits 0, with these checks printed `ok`: `running`, NetworkManager `running`, wrong token rejected, the os-release fields and `hostnamectl`, `generic-release-common` owning the `/etc/os-release` link and `stackific-release` owning `/usr/lib/os-release`, no Fedora release or logo packages, the icon's accent fill, only `mangowm` and `noctalia-greeter` from Terra, `Enforcing`, greetd `active` as `display-manager.service`, `stackific-noctalia.service` `active` for `live`, a `mango` PID for `live`, both session files, the artwork, Noctalia, greeter and fastfetch checks of item 8, `greetd` owning `/var/lib/noctalia-greeter`, a `noctalia-greeter` PID for `greetd` after the logout, and `0` AVC denials.
- `.build/fedora/x86_64/screen.png` shows the MangoWM desktop with Noctalia's bar, and `.build/fedora/x86_64/greeter.png` shows noctalia-greeter over Dahl's Vesuvius in our emerald palette, with our logo where Noctalia's was.
- The manual GNOME steps in Tests pass, or the review record says which step failed and the operator decides.
- `just lefthook all` passes (shellcheck covers `terra-repo.sh`).
- The task report states the ISO size in MiB against the 2048 MiB target.

Must not:
- change a file owned by mangowm, noctalia, noctalia-greeter, greetd or any GNOME package, except greetd's `config.toml` in the live image;
- take any package from Terra other than `mangowm` and `noctalia-greeter`, or turn Terra's signature checks off;
- start Noctalia in the GNOME session;
- add `NOPASSWD`, a sudoers file or a polkit rule anywhere;
- keep the tty1 root autologin, or ship anything from `tests/`, an SSH server (`openssh-server`), `cloud-init` or a VM guest agent in the image. `openssh` (`ssh-keygen`) and `openssh-clients` are in it: `noctalia` requires `git-core`, which requires `openssh-clients`, which requires `openssh` (`dnf repoquery --requires`), and neither holds sshd;
- use sudo or root on the host;
- touch files outside the Changes list (the review record, prompt-trail entry and its screenshots from /s-build excepted);
- add a dependency this task does not name.

## Tests
- `sha256sum fedora/iso/terra-45.asc` and `gpg --show-keys fedora/iso/terra-45.asc`: prove the committed key.
- `just fedora iso`, then `grep -E '^(stackific-|mangowm|noctalia|greetd|gnome-shell|generic-|NetworkManager-wifi|fedora-|cardwire)' .build/fedora/x86_64/*.packages.txt`: proves the manifest criterion.
- The wrong-key build above: proves the key is checked.
- `just fedora test`: proves the identity, Terra, SELinux, session, greeter and network checks, and leaves `screen.png` and `greeter.png`. Look at both images. Convert them with `just site screenshot` into `site/src/assets/prompt-trail/008/desktop.webp` and `greeter.webp` for the prompt-trail entry `site/src/content/docs/prompt-trail/fedora/008-fedora-desktop-live-iso.md`.
- `git grep -nE 'NOPASSWD|polkit' fedora tests desktop` prints nothing.
- `just lefthook all` and `just doctor`.
- Manual, for the GNOME session (automation is a later task):
  1. Boot `.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.iso` in a VM with a display, UEFI firmware and 4 GiB of RAM (virt-manager or GNOME Boxes, for example). MangoWM with Noctalia's bar appears.
  2. Press `Alt+Return`. A foot terminal opens. Run `loginctl terminate-user live`.
  3. At noctalia-greeter, pick the user `live` and the GNOME session, type any character as the password and press Enter. noctalia-greeter refuses an empty field (`Enter a password`, measured in the VM), and PAM does not ask for `live`'s empty password, so what is typed is never checked (greetd's journal: `pam_unix(greetd:auth): user [live] has blank password; authenticated without it`).
  4. GNOME's desktop appears. In Ptyxis, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.
