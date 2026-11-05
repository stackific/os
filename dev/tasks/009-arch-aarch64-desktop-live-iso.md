# 009: Arch Linux ARM aarch64 live ISO boots to MangoWM with Noctalia, with KDE Plasma

- [ ] Done

Base: arch

## Ask

```text title="Prompt" wrap
Turn the Arch Linux ARM aarch64 console live ISO from task 005 into the same desktop live ISO that x86_64 has after tasks 002, 004 and 006: one product, as AGENTS.md asks. It boots straight into MangoWM with Noctalia, with no first-run welcome. greetd logs the user live in automatically, and noctalia-greeter, with our logo, palette and wallpaper, is the login screen after a logout, with KDE Plasma as the second session. Take every desktop package unmodified from Arch Linux ARM's live repos, and reuse our stackific-desktop, stackific-release and stackific-artwork packages, which are arch=any. The aarch64 package list mirrors the x86_64 one; only the kernel, the microcode and the ALARM keyring differ. Both Arch images then pass the same checks, so the boot test runs one Arch check file on both, with a screenshot of the desktop and of the greeter. Report the ISO size against the 2 GiB target. Build and test it in the Mac's runner VM.
```

## Goal
In the Mac's runner VM, `just arch iso` builds an aarch64 live ISO identical in look and defaults to the x86_64 one: it boots into MangoWM with Noctalia as `live`, on our wallpaper and with no first-run welcome, with our noctalia-greeter and Plasma available from it. `just arch test` runs the same Arch checks there as on x86_64.

## Context
This task starts from `main` after task 008 (`531db03`). Task 008 gave Fedora x86_64 the same desktop and checks; it changed no Arch build or test file.

What `main` has:
- Task 002 made the x86_64 image a desktop: the desktop part of `arch/iso/packages.x86_64`, the `stackific-desktop` package, and the greetd config and live accounts in `arch/iso/airootfs/`.
- Task 004 added `stackific-release`, which owns `/etc/os-release` and installs the Stackific icon.
- Task 006 added the branding file `branding/branding.env` and the package `stackific-artwork` (ten wallpapers in `/usr/share/backgrounds/stackific/`, fastfetch's logo in the accent). It also gave `stackific-desktop` our defaults:
  - Noctalia's defaults `/usr/share/stackific/noctalia.toml`: setup wizard off, the wallpaper folder, and Dahl's "An Eruption of Vesuvius" as the default wallpaper. Each user's `~/.config/noctalia/00-stackific.toml` from `/etc/skel` includes it.
  - noctalia-greeter's defaults `/usr/share/stackific/greeter.toml` (our palette, Vesuvius), linked from `/var/lib/noctalia-greeter/greeter.toml` by a tmpfiles line.
  - our logo on the greeter, through `NOCTALIA_GREETER_ASSETS_DIR` set in `/etc/profile.d/stackific-greeter.sh`, pointing at `/usr/share/stackific/greeter-assets/`.
- Task 007 built our three packages as Fedora RPMs too, from the same sources and to the same paths. It also fixed fastfetch's config: `branding/fastfetch.jsonc.in` now lists fastfetch's default modules, because a config file without a `modules` list prints only the logo. `stackific-artwork` is at `pkgrel=2` for it.
- `tests/vm/arch-x86_64.sh` checks all of that: identity, the desktop session, wallpapers, Noctalia's merged config, the greeter's config, assets and environment, fastfetch's logo, and (since 007) fastfetch's `OS: Stackific OS` line.
- Task 005 made the profile, builder and recipes build for the host's architecture. aarch64 still has a console package list (with 006's `stackific-release` and `stackific-artwork`), and `tests/vm/arch-aarch64.sh` holds its console checks plus 006's identity, wallpaper and fastfetch checks and 007's fastfetch `OS:` line check. `just arch test` passes `/tests/arch-<arch()>.sh`.
- AGENTS.md: "The four images are one product: the same branding, messaging, defaults, files at the same paths, live session and boot-test checks. They differ only where a base or architecture forces it." This task brings the aarch64 Arch image in line. The Fedora images get the desktop, and with it the same defaults and checks, in their own tasks: 007 already built our packages as RPMs, and Fedora's desktop follows in tasks 008 and 010.

Facts, checked 2026-11-05 in the aarch64 builder `localhost/stackific-arch-builder:2026-11-01` on the Mac's runner VM (Fedora 44, Apple M4 Pro, nested KVM), unless another source is named:
- ALARM aarch64 carries every desktop package of the x86_64 list (`pacman -Si`):
  - in extra: mangowm 0.18.4-1, noctalia 5.4.1-1, noctalia-greeter 1.7.0-1, greetd 0.10.3-2;
  - plasma-desktop, plasma-nm, plasma-pa and kscreen at 6.8.2-1; konsole and dolphin at 26.08.2-1;
  - xdg-desktop-portal-kde 6.8.2-1, xdg-desktop-portal-gtk 1.15.3-2, xdg-desktop-portal-wlr 0.8.4-1;
  - foot 1.28.0-2, pipewire 1:1.6.10-1, pipewire-pulse 1:1.6.10-1, wireplumber 0.5.18-1;
  - mangowm's dependencies scenefx 0.5-1, wlroots0.20 0.20.2-1 and mesa 1:26.2.6-1.

  Except rofi, these are the same versions as Arch's at archive date 2026-11-01 (task 002, Context). ALARM's repos move daily; the manifest records what each build took.
- ALARM's rofi is 1.7.5-3, while Arch's is 2.0.0-1. MangoWM's stock config binds `Alt+space` to `rofi -show drun` (task 002, Context); rofi has had the `drun` mode since 1.0.
- An earlier build of this task (before 006, so without `stackific-artwork`) gave a 1290 MiB ISO with 655 packages. 006's wallpapers add about 40 MiB of JPEGs. GitHub release assets are capped at 2 GiB per file, and `just release upload` puts the ISO on a GitHub release, so the ISO must stay at 2048 MiB or less.
- ALARM's `filesystem` 2025.10.12-1 owns `/etc/arch-release` and `/usr/lib/os-release`, and no package owns `/etc/os-release` (`pacman -Fl filesystem`), as on Arch. So `stackific-release` installs without a conflict, and 004's ownership check holds on aarch64; 005's aarch64 checks already pass it.
- The ALARM packages have the file layout tasks 002 and 006 rely on (`pacman -Fl`):
  - noctalia-greeter: `/usr/bin/noctalia-greeter`, `noctalia-greeter-session` and `/usr/share/noctalia-greeter/assets/fonts/`;
  - mangowm: `/etc/mango/config.conf`, `/usr/lib/systemd/user/mango-session.target` and `/usr/share/wayland-sessions/mango.desktop`;
  - greetd: `greetd.service` and `/usr/lib/sysusers.d/greetd.conf`.
- ALARM's mesa ships `usr/lib/dri/kms_swrast_dri.so` (`pacman -Fl`), so on the test VM's `virtio-gpu-pci` Mesa renders in software, which `stackific-desktop`'s `WLR_RENDERER_ALLOW_SOFTWARE=1` lets MangoWM use. Measured in the earlier build of this task: MangoWM, Noctalia and noctalia-greeter draw there under nested KVM, and the `virtio-gpu-pci` screenshots show them.
- AGENTS.md rules this task applies:
  - the four images are one product;
  - MangoWM, Noctalia and Plasma are unmodified packages, and our defaults ship as config files in a package;
  - Arch Linux ARM builds from its live repos, and every image records its manifest and reports its size;
  - never `NOPASSWD`, and no polkit rule that skips authentication.

## Inputs
Decisions the operator made for this task (2026-11-05), restated:
- The number 009 is reserved for this task by the operator's order of work; it follows 005 to 008, all merged.
- It mirrors x86_64 on aarch64: same desktop, live session, greeter, defaults and checks (AGENTS.md's one-product rule).
- While reviewing an earlier build, the operator asked for no Noctalia first-run welcome for any user and for our branding on the greeter. Task 006 then delivered both for every Arch image, and the operator asked to follow it, so this task adds nothing of its own for them: the mirror brings them to aarch64.
- Like 005, it is built and tested in the Mac's runner VM.

A design choice made while writing this task, which the operator may change in review:
- **One Arch check file.** Both Arch images now pass exactly the same checks, so `tests/vm/arch-x86_64.sh` becomes `tests/vm/arch.sh`, unchanged, and `tests/vm/arch-aarch64.sh` goes away. Its identity, wallpaper and fastfetch checks (006's and 007's) are already in the x86_64 file, and its console checks no longer apply. `just arch test` passes `/tests/arch.sh` on both architectures. That widens 003's naming rule: a base whose images all pass the same checks has one `tests/vm/<base>.sh`, and `<base>-<arch>.sh` stays for checks only one architecture passes. The alternative is two identical files that must change together.

No files copied.

## Changes
- `arch/iso/packages.aarch64`: every line of `arch/iso/packages.x86_64`, in the same order, except:
  - `linux` becomes `linux-aarch64`;
  - `amd-ucode` and `intel-ucode` are left out;
  - `archlinuxarm-keyring` follows `linux-firmware`.
- `tests/vm/arch.sh`: `tests/vm/arch-x86_64.sh` renamed, with its checks unchanged; the header comment says both Arch architectures source it.
- `tests/vm/arch-aarch64.sh`: removed.
- `arch/justfile`: `test` passes `/tests/arch.sh` instead of `/tests/arch-{{ arch() }}.sh`, and its comment names the file.
- `docs/arch-iso.md`:
  - the aarch64 section says the image is the same desktop as x86_64, explains the mirrored package list, and notes ALARM's older rofi; it drops the sentences about the console and the unused desktop files;
  - the boot test section says both architectures source `tests/vm/arch.sh`, and drops the aarch64 console paragraph;
  - the manual Plasma login step types any character as the password, as task 008 found for Fedora: noctalia-greeter refuses an empty field, and `live` has no password (AGENTS.md, `site/src/content/docs/user/live-session.md`).
- `AGENTS.md`: the Tests line about `boot.sh` names the check file as `tests/vm/<base>.sh`, or `tests/vm/<base>-<arch>.sh` when a check holds for one architecture only.

## Acceptance criteria

Must:
- In the runner VM, `just arch iso` exits 0, writes `.build/arch/aarch64/stackific-0.0.0-dev-arch-aarch64.iso` and its `.packages.txt`, and its last line prints the ISO size in MiB against 2048 MiB. The ISO is 2048 MiB or less.
- The manifest lists `stackific-desktop`, `stackific-release`, `stackific-artwork`, `mangowm`, `noctalia`, `noctalia-greeter`, `greetd`, `plasma-desktop` and `linux-aarch64`, and lists neither `amd-ucode` nor `intel-ucode`.
- The mirror rule holds: `diff <(grep -vx archlinuxarm-keyring arch/iso/packages.aarch64 | sed 's/^linux-aarch64$/linux/') <(grep -vxE 'amd-ucode|intel-ucode' arch/iso/packages.x86_64)` prints nothing.
- In the runner VM, `just arch test` exits 0, and every check of `tests/vm/arch.sh` passes on aarch64: identity, the desktop session, wallpapers, Noctalia's merged defaults, the greeter's config, logo and environment, fastfetch's logo, and its `OS: Stackific OS` line.
- `just arch test` leaves `.build/arch/aarch64/screen.png` and `greeter.png`:
  - `screen.png` shows MangoWM with Noctalia's bar on "An Eruption of Vesuvius", with no welcome dialog;
  - `greeter.png` shows noctalia-greeter in our palette on the same painting, with the Stackific logo.
- On the operator's PC, `just arch iso` and `just arch test` pass on this branch with `tests/vm/arch.sh`.
- After the merge, the `images` run on `main` passes every job, and the test release `v0.0.0-pr.<n>` holds the aarch64 desktop ISO.
- `just lefthook all` passes.
- The task report states the aarch64 ISO size in MiB against the 2048 MiB target, and the versions the manifest took for the packages named above.

Must not:
- change `arch/iso/packages.x86_64`, any check in the moved check file, or any of 006's defaults;
- patch or replace MangoWM's, Noctalia's or noctalia-greeter's packaged files;
- pin ALARM packages, cross-build, or use qemu-user;
- add `NOPASSWD` or a polkit rule;
- touch files outside the Changes list (the review record and prompt-trail entry from /s-build excepted), or add a dependency this task does not name.

## Tests
- In the runner VM: `just arch iso`, then `grep -E '^(stackific-desktop|stackific-release|stackific-artwork|mangowm|noctalia|noctalia-greeter|greetd|plasma-desktop|linux-aarch64|amd-ucode|intel-ucode) ' .build/arch/aarch64/*.packages.txt`. This proves the build, the size line and the manifest criteria.
- The `diff` command in the criteria: the mirror rule.
- `git diff -M` shows `tests/vm/arch-x86_64.sh` renamed to `tests/vm/arch.sh`, with only its header comment changed.
- `just arch test` in the VM: every check, both screenshots and the serial log. Look at both PNGs before writing the prompt-trail entry.
- `just lefthook all` on a developer machine.
- Manual, by the operator:
  1. Before the merge, on the PC: `just arch iso`, then `just arch test`. Both must pass.
  2. After the merge, with both runners online (`just runner status` lists both): watch the `images` run on `main` with `gh run watch`. Every job must finish green.
  3. `gh release view v0.0.0-pr.<n>` lists the aarch64 ISO, its manifest and its `.iso.sha256`.
- Manual, optional: log in to Plasma from the greeter with the four steps under "Boot test" in `docs/arch-iso.md`. The runner VM has no display, so boot the ISO in any aarch64 UEFI VM that has one.
