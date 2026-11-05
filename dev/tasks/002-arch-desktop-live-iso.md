# 002: Arch x86_64 live ISO boots to MangoWM with Noctalia, with KDE Plasma

- [x] Done

Base: arch

## Ask

```text title="Prompt" wrap
Turn the Arch x86_64 console live ISO from task 001 into a desktop live ISO. It boots straight into MangoWM with Noctalia: greetd logs a live user in automatically, and noctalia-greeter is the login screen after a logout, where KDE Plasma is the second session. Take every desktop package unmodified from Arch extra at the pinned archive date. Start Noctalia from our own package, not from an edited MangoWM config. The same package lets wlroots compositors fall back to software rendering everywhere, so MangoWM also starts in VMs without 3D and on GPUs without a working driver. That package is the first PKGBUILD: build it with makepkg in a fresh pinned builder container, put it in a local repo, and install it from there into the ISO. Extend the QEMU boot test so it proves the MangoWM session and Noctalia are running and the greeter appears after a logout, with a screenshot of each. Report the ISO size against the 2 GiB target.
```

## Goal
`just arch iso` builds an Arch x86_64 live ISO that boots into MangoWM with Noctalia as the user `live`, with KDE Plasma available from noctalia-greeter. `just arch test` proves that session runs and that the greeter shows after a logout.

## Context
Task 001 built a console live ISO from `arch/iso/` in the builder `arch/Containerfile`, pinned to archive date 2026-11-01 in `arch/vars.just`. It autologs root on tty1 with `arch/iso/airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf`, and `tests/vm/boot.sh` checks the console over a token-gated vsock shell. That ISO is 898 MiB. This task adds a desktop, a live user, greetd, and the first package we build, `stackific-desktop`.

Facts, all checked 2026-11-04 in the builder `localhost/stackific-arch-builder:2026-11-01` unless another source is named:
- Arch extra at archive date 2026-11-01 has these versions (`pacman -Si`): mangowm 0.18.4-1, noctalia 5.4.1-1, noctalia-greeter 1.7.0-1, greetd 0.10.3-2, plasma-desktop 6.8.2-1, plasma-nm 6.8.2-1, plasma-pa 6.8.2-1, kscreen 6.8.2-1, konsole 26.08.2-1, dolphin 26.08.2-1, xdg-desktop-portal-kde 6.8.2-1, xdg-desktop-portal-gtk 1.15.3-2, xdg-desktop-portal-wlr 0.8.4-1, foot 1.28.0-2, rofi 2.0.0-1, pipewire 1:1.6.10-1, pipewire-pulse 1:1.6.10-1, wireplumber 0.5.18-1. Nothing on this list needs building.
- noctalia 5.4.1 depends on neither quickshell nor a font package. The desktop's dependencies already pull in `noto-fonts` (`pacman -Sp`), so no font needs naming.
- These packages, with their dependencies, download about 543 MiB on top of what the builder already has (`pacman -Sp`). The ISO should stay under 2048 MiB, but the build must measure it.
- The mangowm 0.18.4-1 package ships:
  - `/usr/bin/mango` and `/etc/mango/config.conf` (also `config.toml`);
  - `/usr/share/wayland-sessions/mango.desktop` (`Exec=mango`);
  - `/usr/lib/systemd/user/mango-session.target` (`BindsTo=graphical-session.target`);
  - `/usr/share/xdg-desktop-portal/mango-portals.conf` (`default=gtk`, with wlr for ScreenCast and Screenshot).

  Its stock `config.conf` binds `Alt+Return` to `foot` and `Alt+space` to `rofi -show drun`. It has no `exec_once` line, so it does not start Noctalia.
- MangoWM reads `~/.config/mango/config.conf`, then `config.toml`, and falls back to `/etc/mango/`. `exec_once` runs a command once at startup. Source: https://mangowm.github.io/docs/configuration/basics.
- At startup MangoWM 0.18.4 runs `systemctl --user import-environment DISPLAY WAYLAND_DISPLAY XDG_CURRENT_DESKTOP XDG_SESSION_TYPE XCURSOR_THEME XCURSOR_SIZE MANGO_INSTANCE_SIGNATURE; systemctl --user --no-block start mango-session.target`. It does this only when `DBUS_SESSION_BUS_ADDRESS` is set, and pam_systemd sets that in a logind session. On exit it stops `graphical-session.target`. Source: `set_activation_env()` in `src/main.c` (lines 292–329, called at line 446) at tag 0.18.4, https://github.com/mangowm/mango/blob/0.18.4/src/main.c.
- Noctalia 5.4.1 ships `/usr/bin/noctalia` and no systemd unit. Its docs start it from the compositor, and for Mango that means `exec_once=noctalia` in `~/.config/mango/config.conf`. Sources: https://docs.noctalia.dev/noctalia/v5.4.1/getting-started/running-the-shell/ and https://docs.noctalia.dev/noctalia/v5.4.1/compositor-settings/mango/.
- Why we need glue: Noctalia has no unit, and the stock MangoWM config does not start it. Starting it from MangoWM's config would mean changing `/etc/mango/config.conf`, a file the mangowm package owns. MangoWM already starts `mango-session.target`, so a user unit wanted by that target starts Noctalia with MangoWM unchanged. Plasma never starts that target, so Noctalia stays out of Plasma.
- With no GPU, or no working GPU driver, Mesa renders in software (llvmpipe). wlroots 0.20.2 refuses that unless `WLR_RENDERER_ALLOW_SOFTWARE` is set (`render/egl.c:334`: "Software rendering detected, please use the WLR_RENDERER_ALLOW_SOFTWARE environment variable to proceed"). MangoWM renders through scenefx 0.5's GLES-only `fx_renderer`, so it exits with `couldn't create renderer`. noctalia-greeter's compositor falls back to wlroots' pixman renderer and survives. Measured during /s-build on 2026-11-04 in the test VM (QEMU, `-vga std`). The operator chose to allow software rendering everywhere: with no working GPU driver the desktop starts, only slower.
- greetd sources `/etc/profile` before every session command (`source_profile`, default true; greetd(5)). Arch's `/etc/profile` (filesystem 2025.10.12-1) sources every `/etc/profile.d/*.sh`. So a profile.d script reaches the autologin session and every session started from the greeter.
- greetd 0.10.3-2 ships `/etc/greetd/config.toml` in its pacman `backup` array. Its `greetd.service` has `Alias=display-manager.service` and `Conflicts=getty@tty1.service`. Its `sysusers.d/greetd.conf` creates the user `greeter`, in group `video`. Source: package contents.
- greetd's `[initial_session]` is its auto-login. It runs only on greetd's first start since boot; after that session ends, greetd runs `[default_session]`. Source: greetd(5) in greetd 0.10.3-2.
- noctalia-greeter 1.7.0 expects this greetd config: `[terminal] vt = 1`, `[default_session] command = "<session bin>"`, `user = "greeter"`. The session bin is `/usr/bin/noctalia-greeter-session`, which starts its own wlroots compositor (`noctalia-greeter-compositor`) and the greeter `/usr/bin/noctalia-greeter`. Its tmpfiles file creates `/var/lib/noctalia-greeter`. Source: `print_greetd_config.sh` and `greetd_setup_lib.sh` in `/usr/share/noctalia-greeter/`.
- noctalia-greeter's optional setup script adds a pam_systemd session line to `/etc/pam.d/greetd`. Arch does not need it: Arch's `/etc/pam.d/greetd` includes `system-local-login`, and through it pambase 20260616-1's `system-login` (`-session optional pam_systemd.so`). So this task does not run that script.
- The comm name of `/usr/bin/noctalia-greeter` is cut to 15 characters (`noctalia-greete`), so `pgrep -x noctalia-greeter` never matches. Match on the full command line instead. Source: proc(5), `/proc/pid/comm`.
- filesystem 2025.10.12-1 ships `/etc/passwd`, `/etc/group`, `/etc/shadow` and `/etc/gshadow` with only root in them, all in its `backup` array. That is why task 001's airootfs `etc/shadow` replaced the package's file. Airootfs copies of these files must keep the root line: `root:x:0:0::/root:/usr/bin/bash`, `root:x:0:root`, `root:::root`.
- systemd's `basic.conf` creates `wheel` with a dynamic GID (`g wheel - - -`). A `m live wheel` line in a sysusers.d file adds the membership without hard-coding the GID. The systemd pacman hook runs `systemd-sysusers` while pacstrap installs, and mkarchiso copies airootfs before pacstrap (task 001, Context).
- mkarchiso 92 (`_make_customize_airootfs`, `/usr/bin/mkarchiso` lines 395–417) reads the profile's `airootfs/etc/passwd`. For each UID from 1000 to 59999, it creates the home directory with mode 0750, copies `/etc/skel` into it, and gives it to that user.
- makepkg as a regular user needs `fakeroot`, which the builder lacks (`command -v fakeroot` prints nothing). `/etc/makepkg.conf` enables `debug` in `OPTIONS`, and `debugedit` is missing too. A text-only `arch=('any')` package therefore sets `options=('!debug' '!strip')`.
- AGENTS.md rules this task applies:
  - MangoWM, Noctalia and Plasma are unmodified packages; our defaults ship as config files in a package, never as patches;
  - glue (a systemd unit) only when no packaged tool does the job, and the task says why;
  - our packages are built with makepkg in a fresh pinned container;
  - every image records its manifest and reports its size;
  - never `NOPASSWD`, and no polkit rule that skips authentication.

Left out, for later tasks:
- a Stackific MangoWM config with Noctalia's keybinds (`Super+space` launcher and others from the Mango page above);
- `sudo` for the live user (with a password prompt, never `NOPASSWD`);
- a hosted, signed `[stackific]` repo and package signing;
- Vulkan, VA-API and NVIDIA drivers (the hardware task);
- an automated Plasma login through greetd's IPC, and AGENTS.md's container test for headless MangoWM with Noctalia;
- branding (os-release `ID=stackific`, wallpapers), and user docs once a release ships;
- Noctalia defaults that skip its first-run welcome dialog, and `rtkit` for PipeWire's realtime priority (both seen during /s-build);
- a package cache shared by ISO builds: every build downloads all 660 packages from the archive, and one build during /s-build failed on an archive timeout.

## Inputs
None. The operator chose this task over a Fedora 45 console ISO, an Arch release package and an Arch Linux ARM console ISO. The design choices are made in this task: a greetd autologin into MangoWM with noctalia-greeter as the login screen; a user unit to start Noctalia; the user `live` with an empty password. During /s-build the operator chose to allow software rendering for wlroots everywhere (see Context), and allowed any VM RAM size the session needs.

## Changes
- `desktop/systemd/user/stackific-noctalia.service`: a user unit that runs `/usr/bin/noctalia` with `Restart=on-failure`. It has `PartOf=graphical-session.target` and `After=graphical-session.target`, and `[Install] WantedBy=mango-session.target`. A header comment says why the unit exists (see Context).
- `arch/packages/stackific-desktop/PKGBUILD`:
  - `pkgname=stackific-desktop`, `pkgver=0.1.0`, `pkgrel=1`, `arch=('any')`, `license=('Apache-2.0')`, `url="https://os.stackific.com"`;
  - `depends=('mangowm' 'noctalia' 'systemd')`, `options=('!debug' '!strip')`;
  - `source=('stackific-noctalia.service' 'stackific-wlroots.sh')` with their sha256s, and `backup=('etc/profile.d/stackific-wlroots.sh')`;
  - `package()` installs the unit to `/usr/lib/systemd/user/`, plus the symlink `/usr/lib/systemd/user/mango-session.target.wants/stackific-noctalia.service` → `../stackific-noctalia.service`, so every user has it on. It installs `stackific-wlroots.sh` to `/etc/profile.d/`.
- `desktop/profile.d/stackific-wlroots.sh`: `export WLR_RENDERER_ALLOW_SOFTWARE=1`, with a comment that says why (see Context).
- `arch/Containerfile`: add `fakeroot` to the `pacman -Syu` line.
- `arch/vars.just`: `repo := justfile_directory() / ".build" / "arch" / "repo"`.
- `arch/justfile`:
  - a new `packages` recipe, which depends on `builder`. It empties `{{ repo }}`, then runs a fresh `--rm` builder container as `build`. For each `arch/packages/*/PKGBUILD`, the container copies the PKGBUILD and the `desktop/` files named in its `source=()` into a new directory, runs `makepkg --nodeps` there (the directory is new, so nothing is stale), and copies the package out. It then runs `repo-add {{ repo }}/stackific.db.tar.zst` on the packages;
  - `iso` depends on `packages` and mounts `{{ repo }}` read-only at `/repo` in the container.
- `arch/iso/pacman.conf`: a `[stackific]` section above `[core]` with `SigLevel = Optional` and `Server = file:///repo`. A comment says the release-signing task switches it to `Required`.
- `arch/iso/packages.x86_64`, add: `stackific-desktop mangowm noctalia greetd noctalia-greeter foot rofi plasma-desktop plasma-nm plasma-pa kscreen konsole dolphin xdg-desktop-portal-kde xdg-desktop-portal-gtk xdg-desktop-portal-wlr pipewire pipewire-pulse wireplumber`.
- `arch/iso/profiledef.sh`: add `["/etc/gshadow"]="0:0:400"` to `file_permissions`.
- `arch/iso/airootfs/etc/passwd` (new): root's line above, plus `live:x:1000:1000:Live user:/home/live:/usr/bin/bash`.
- `arch/iso/airootfs/etc/group` (new): `root:x:0:root` and `live:x:1000:`.
- `arch/iso/airootfs/etc/gshadow` (new): `root:::root` and `live:!::`.
- `arch/iso/airootfs/etc/shadow`: keep root's line; add `live::14871::::::` (an empty password, like root's).
- `arch/iso/airootfs/etc/sysusers.d/stackific-live.conf` (new): `m live wheel`.
- `arch/iso/airootfs/etc/greetd/config.toml` (new), for the live image only:
  - `[terminal] vt = 1`;
  - `[initial_session] command = "mango"`, `user = "live"`;
  - `[default_session] command = "/usr/bin/noctalia-greeter-session"`, `user = "greeter"`.
- `arch/iso/airootfs/etc/systemd/system/display-manager.service` (new): a symlink to `/usr/lib/systemd/system/greetd.service`.
- `arch/iso/airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf`: delete it, because greetd owns tty1.
- `tests/vm/boot.sh`: replace the `who` root-on-tty1 check, and keep the other checks. After `systemctl is-system-running` prints `running`:
  1. `systemctl is-active greetd` prints `active`.
  2. Within 120 s, `systemctl --user -M live@ is-active stackific-noctalia.service` prints `active`.
  3. `pgrep -u live -x mango` prints a PID.
  4. `ls /usr/share/wayland-sessions` lists `mango.desktop` and `plasma.desktop`.
  5. Wait a few seconds for Noctalia to draw, then save `screen.png`.
  6. Run `loginctl terminate-user live`. Within 60 s, `pgrep -u greeter -f '^/usr/bin/noctalia-greeter( |$)'` prints a PID.
  7. Wait for the greeter to draw, then save `greeter.png`.

  Then power off as before. Raise `-m` if the session needs more than 4096 MiB (the operator allows any size), and say so in the review record.
- `README.md`: under Everyday use, `just arch packages` builds our Arch packages into `.build/arch/repo/` (`just arch iso` runs it first).
- `AGENTS.md`, one line each:
  - Builds: our Arch packages live in `arch/packages/<name>/PKGBUILD`, with base-neutral sources in `desktop/`. `just arch packages` builds them into the local repo `.build/arch/repo/`, which the ISO build reads as `[stackific]` (unsigned until release signing).
  - Product: MangoWM starts Noctalia through `stackific-noctalia.service` (package `stackific-desktop`), wanted by `mango-session.target`, which MangoWM starts itself. MangoWM's config stays stock.
  - Product: `stackific-desktop` exports `WLR_RENDERER_ALLOW_SOFTWARE=1` from `/etc/profile.d/`, so MangoWM also starts on software rendering.
  - Product: live images autologin the user `live` (empty password, group `wheel`) into MangoWM through greetd's `initial_session`, and noctalia-greeter is the login screen.
- `site/src/content/docs/developer/arch-iso.md`, updated for:
  - the `packages` step and the `[stackific]` local repo;
  - the live session: user `live`, MangoWM with Noctalia, Plasma from the greeter, `Alt+Return` foot and `Alt+space` rofi;
  - the new test checks and `greeter.png`.

  No user docs yet: nothing is released.

## Acceptance criteria

Must:
- `just arch packages` exits 0 and leaves `.build/arch/repo/stackific-desktop-0.1.0-1-any.pkg.tar.zst` and `.build/arch/repo/stackific.db.tar.zst`. `bsdtar -tf` on the package lists `usr/lib/systemd/user/stackific-noctalia.service`, `usr/lib/systemd/user/mango-session.target.wants/stackific-noctalia.service` and `etc/profile.d/stackific-wlroots.sh`.
- `just arch iso` exits 0, and its last line prints the ISO size in MiB. The manifest `.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.packages.txt` lists these packages at these versions: `stackific-desktop 0.1.0-1`, `mangowm 0.18.4-1`, `noctalia 5.4.1-1`, `noctalia-greeter 1.7.0-1`, `greetd 0.10.3-2` and `plasma-desktop 6.8.2-1`.
- `just arch test` exits 0, with these checks printed `ok`:
  - `running`;
  - greetd `active`;
  - `stackific-noctalia.service` `active` for `live`;
  - a `mango` PID for `live`;
  - both session files present;
  - a `noctalia-greeter` PID for `greeter` after the logout;
  - NetworkManager `running`;
  - wrong token rejected.
- `.build/arch/x86_64/screen.png` shows the MangoWM desktop with Noctalia's bar, and `.build/arch/x86_64/greeter.png` shows noctalia-greeter.
- `just lefthook all` passes.
- The task report states the ISO size in MiB against the 2048 MiB target.

Must not:
- change a file owned by mangowm, noctalia, noctalia-greeter or any Plasma package. Our only change to a package's `/etc` file is greetd's `config.toml`, and only in the live image;
- start Noctalia in the Plasma session;
- add `NOPASSWD`, a sudoers file or a polkit rule anywhere;
- keep the tty1 root autologin, or ship anything from `tests/`, `openssh`, `cloud-init`, syslinux or GRUB in the image;
- use sudo or root on the host;
- touch files outside the Changes list (the review record, prompt-trail entry and its screenshots from /s-build excepted);
- add a dependency this task does not name.

## Tests
- `just arch packages`, then `bsdtar -tf .build/arch/repo/stackific-desktop-0.1.0-1-any.pkg.tar.zst`: proves the package criterion.
- `just arch iso`, then `grep -E '^(stackific-desktop|mangowm|noctalia|noctalia-greeter|greetd|plasma-desktop|openssh|cloud-init|syslinux|grub) ' .build/arch/x86_64/*.packages.txt`: proves the manifest versions and the absent packages.
- `just arch test`: proves the session, unit, greeter and network checks, and leaves `screen.png` and `greeter.png`. Look at both images. Convert them with `just site screenshot` into `site/src/assets/prompt-trail/002/desktop.webp` and `greeter.webp` for the prompt-trail entry.
- `git grep -nE 'NOPASSWD|polkit' arch tests desktop` prints nothing.
- `just lefthook all` and `just doctor`.
- Manual, for the Plasma session (automation is a later task):
  1. Boot `.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.iso` in a VM with a display, UEFI firmware and 4 GiB of RAM (virt-manager or GNOME Boxes, for example). MangoWM with Noctalia's bar appears.
  2. Press `Alt+Return`. A foot terminal opens. Run `loginctl terminate-user live`.
  3. At noctalia-greeter, pick the user `live` and the Plasma session, leave the password empty and log in.
  4. Plasma's desktop appears. In Konsole, `systemctl --user is-active stackific-noctalia.service` prints `inactive`.
