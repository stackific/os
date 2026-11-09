---
title: A package manager GUI on every image
description: Shelly on the Arch Linux image and GNOME Software on both Fedora Remix images, each asking for the password before it changes the system.
sidebar:
  order: 24
  label: 024 A package manager GUI on every image
pr: https://github.com/stackific/os/pull/24
task: dev/tasks/024-package-manager-gui.md
---

## Goal

Every image gets a graphical package manager, live and installed, and none of them installs software without the administrator's password.

## Context

The images had no package manager GUI. Shelly, Seafoam Labs' GTK4 package manager for Arch's repositories, the AUR and Flatpak, exists only in the AUR; GNOME Software is Fedora Workstation's own store. Fedora's GNOME Software pulls in a polkit rule that lets administrators install signed packages without a password.

## Expectations

Must:

- `just arch packages` on x86_64 builds `shelly-3.2.0-1-x86_64.pkg.tar.zst` and `shelly-flatpak-backend-3.2.0-1-x86_64.pkg.tar.zst` into `.build/arch/repo/`; on aarch64 it builds the other packages and skips Shelly.
- The Arch x86_64 live image: `pacman -Q shelly shelly-flatpak-backend` lists both, `/usr/bin/shelly-ui` is executable, `com.shellyorg.shelly.desktop` is installed, and `pkcheck --action-id com.shellyorg.shelly.pkexec.cli --process <live's mango>` exits 2 (authentication required).
- Both Fedora live images: `rpm -q gnome-software` succeeds; `pkcheck --process <live's mango>` exits 2 (authentication required) for `org.rpm.dnf.v0.rpm.execute_trusted_transaction` and `org.rpm.dnf.v0.rpm.Repo.confirm_key`, `live` being an active local `wheel` user; `/etc/polkit-1/rules.d/10-stackific-dnf5daemon.rules` is owned by `stackific-desktop`.
- `just lefthook all` passes.

Must not:

- Ship Shelly on Arch aarch64.
- Change Shelly's sources or side files beyond `arch`, `options=('!debug')` (makepkg would otherwise need `debugedit` for a `-debug` package, which our packages never ship) and the header comment.

## Prompt

```text title="Prompt" wrap
Give every image a graphical package manager, live and installed. On Arch Linux, Shelly (Seafoam Labs' GTK4 package manager for the repos, the AUR and Flatpak) built from the AUR's PKGBUILD into our own Arch repo, with its Flatpak backend; on Fedora, GNOME Software, Fedora Workstation's own store. Shelly needs Zig 0.17.0: if it cannot build on Arch Linux ARM, leave it out of the Arch aarch64 image, say so in the docs and list it under Later. Neither may install software without the administrator's password: no polkit rule that skips authentication, so where a package we ship brings such a rule, override it.
```

## Outcome

- **Shelly on Arch x86_64.** `arch/packages/shelly/` holds the AUR's PKGBUILD for Shelly 3.2.0 with its four side files, changed only in a header comment, `arch=('x86_64')` and `options=('!debug')`. `just arch packages` builds `shelly` and `shelly-flatpak-backend` into our repo, and the x86_64 image installs both. The builder carries Shelly's makedepends, Zig 0.17.0 among them; Zig fetches Shelly's own dependencies by URL and checks each against its hash.
- **Not on Arch Linux ARM.** It has Zig 0.16.0 only, so `just arch packages` now skips any PKGBUILD whose `arch` names neither `any` nor the host's architecture. The user page says to use `pacman` there.
- **GNOME Software on both Fedora images.** It needs `dnf5daemon-server-polkit`, whose rule would let an active `wheel` user install signed packages and accept repository keys with no password. polkit runs every rules file in order of file name, and the first answer wins. A same-named file in `/etc` would only run first, not replace it. So `stackific-desktop` ships `10-stackific-dnf5daemon.rules`, which sorts first and answers with dnf5daemon's own defaults: the password, kept for a few minutes.
- **Checks.** The boot tests ask polkit with `pkcheck` about the live user's desktop session: Shelly's privileged action on Arch, and dnf5daemon's install and key actions on Fedora, must each need authentication.

### Screenshots

No UI.

## Review

- Round 1 found one thing, fixed: the user page said every change goes through for a few minutes after the password, but trusting a new repository's key asks every time.
- Round 2 found nothing.

## Manual changes

None.

## Result

- Shelly's build, `check()` included, in the changed x86_64 builder: `shelly-3.2.0-1-x86_64.pkg.tar.zst` (3.4 MiB) and `shelly-flatpak-backend-3.2.0-1-x86_64.pkg.tar.zst` (0.4 MiB), with only an action file under `usr/share/polkit-1/`. The architecture skip builds all seven packages for x86_64 and leaves out only Shelly for aarch64.
- `stackific-desktop` 0.2.0-5 builds and holds `/etc/polkit-1/rules.d/10-stackific-dnf5daemon.rules` as a config file. Run in name order before dnf5daemon's rule, it answers with the password for installs (kept a few minutes in an active session) and for repository keys (every time).
- `just lefthook all` passes. The boot tests of all four images, with the `pkcheck` checks, run in the `images` workflow.
- Docs: the user page `user/software.md`, `docs/arch-iso.md` (Shelly), `docs/fedora-iso.md` (GNOME Software and the polkit rule), `AGENTS.md`.

## Deviations

- The task first had an empty `/etc/polkit-1/rules.d/org.rpm.dnf.v0.rules` replace dnf5daemon's rule, and `pkaction` check it. polkit(8) says a same-named file in `/etc` only runs first, and `pkaction` shows no rules, so the task now names a rule that sorts first and `pkcheck` checks. The task file was updated.
- Shelly's PKGBUILD also sets `options=('!debug')`: without it, makepkg stops for want of `debugedit`, and our packages ship no `-debug` package. The task file was updated.
