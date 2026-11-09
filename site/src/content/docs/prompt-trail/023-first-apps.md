---
title: First apps on all four images
description: kitty, btop, Thunar with File Roller, and Bazaar with Flathub on every image, from each base's own packages, with Bazaar's Flatpak installed while the Fedora images are built.
sidebar:
  order: 23
  label: 023 First apps on all four images
pr: https://github.com/stackific/os/pull/22
task: dev/tasks/023-first-apps.md
---

## Goal

kitty, btop, Thunar (with `thunar-archive-plugin` and gvfs), File Roller, Flatpak with the Flathub remote, and Bazaar are on all four images and their installs, checked by the boot and install tests.

## Context

The images had a terminal (foot, plus Ptyxis on Fedora), the second desktops' own apps, GNOME Disks and the installer, but no file manager or app store for the MangoWM desktop. Noctalia's polkit agent was already on, so the plan's polkit item needed nothing. Arch and Arch Linux ARM package every app, Bazaar included; Fedora and Terra have no Bazaar, so the Fedora images take its Flatpak from Flathub.

## Expectations

Must:

- `just arch test` (both architectures) passes with `pacman -Q kitty btop thunar thunar-archive-plugin gvfs file-roller flatpak bazaar` listing 8 packages and `flatpak remotes --system --columns=name` printing `flathub`.
- `just fedora test` (both architectures) passes with the six RPMs installed, `flatpak remotes --system --columns=name` printing `flathub`, and `flatpak info --system io.github.kolunmi.Bazaar` succeeding.
- `just arch test-install` and `just fedora test-install` pass with the apps and Bazaar on the installed system.
- Each Fedora manifest lists `io.github.kolunmi.Bazaar` and its runtime; each image task reports its ISO size.
- `just lefthook all` passes.

Must not:

- Change MangoWM's stock config or bind a key.
- Add a repo file for a third-party RPM repository.
- Add GNOME Disks, a polkit agent or foot again.

## Prompt

```text title="Prompt" wrap
Give all four images their first everyday apps, live and installed: kitty as a second terminal next to foot (and Ptyxis on Fedora, GNOME's own), btop as the system monitor, Thunar as the file manager for MangoWM with its archive plugin and gvfs for Trash and removable drives, File Roller for archives, and Bazaar as the app store, with Flathub set up as the system's Flatpak remote. GNOME Disks and a polkit agent are already there (GNOME Disks since the installer tasks, Noctalia's polkit agent on in our defaults), so they stay as they are. On Arch and Arch Linux ARM every app is a native package from the base's repos, Bazaar included, and flatpak's own remotes.d file adds Flathub. Fedora's flatpak ships no remote, so stackific-desktop links fedora-flathub-remote's Flathub file to the same path, so no key is committed; Fedora has no Bazaar package, so Fedora's images install Bazaar's Flatpak from Flathub while the image is built, so it works offline in the live session and on installs, and the image's package manifest lists each Flatpak with its commit. MangoWM's config stays stock: foot stays on Alt+Return, and kitty, Thunar, btop, File Roller and Bazaar start from the launcher. Every image's boot test checks the apps, the Flathub remote and Bazaar, and the install tests check them on the installed system.
```

## Outcome

- **Arch, both architectures.** `packages.x86_64` and `packages.aarch64` gain `kitty`, `btop`, `thunar`, `thunar-archive-plugin`, `gvfs`, `file-roller`, `flatpak` and `bazaar`, the same lines in the same place. Arch's `flatpak` ships `/usr/share/flatpak/remotes.d/flathub.flatpakrepo`, which flatpak adds to the system installation on first use, so nothing of ours is needed for Flathub.
- **Fedora, both architectures.** `config.kiwi` adds `kitty`, `btop`, `Thunar`, `thunar-archive-plugin`, `file-roller` and `flatpak` (Thunar pulls `gvfs` and `tumbler`). `stackific-desktop` requires `fedora-flathub-remote` and links its Flathub file (`/usr/lib/fedora-third-party/conf.d/`, read otherwise only by `fedora-third-party`) to the path Arch uses, so no key is committed. `config.sh` installs `io.github.kolunmi.Bazaar` from Flathub into the system installation while kiwi builds the image; flatpak checks it and its runtime, `org.gnome.Platform` 51, against Flathub's key from that file. `just fedora iso` appends one `flatpak|<kind>/<id>/<arch>/<branch>|<commit>` line per installed Flatpak to the manifest.
- **Installs.** The installer copies `/var/lib/flatpak` with the live root (`rsync -aH`), so Bazaar and its runtime arrive on the `@flatpak` subvolume unchanged.
- **Tests.** `tests/vm/arch.sh` and `tests/vm/fedora.sh` check the packages, the Flathub remote and, on Fedora, Bazaar's Flatpak and its exported launcher entry; `tests/vm/installed.sh` checks the apps (and Bazaar's Flatpak on Fedora) on the installed system.

### Screenshots

No UI.

## Review

- **Round 1.** Two findings, both fixed. Bazaar's runtime adds about 1.9 GiB to kiwi's work dir, which would push the Fedora aarch64 build past the tmpfs's default cap of half the Mac VM's RAM (8 GiB): task 022, merged first, already mounts it with `size=75%` (12 GiB on the Mac), and the Fedora build notes now count Bazaar in the peak. And nothing checked that the manifest lists Bazaar, so a moved image root would have written junk lines silently: `just fedora iso` now fails when the manifest has no `flatpak|app/io.github.kolunmi.Bazaar/` line.
- **Round 2.** Nothing found.
- **Round 3 (pre-commit gitleaks).** gitleaks' `generic-api-key` rule flagged the `GPGKey=` line of the committed copy of Flathub's file, a public key. Fixed by linking `fedora-flathub-remote`'s file instead of committing a copy.
- **Round 4.** Nothing found.

## Manual changes

None.

## Result

- `just lefthook all` passes (15 checks); shellcheck passes on the changed test scripts and `config.sh`; Prettier passes on the new pages; the manifest loop, run on a fake image root, prints one `flatpak|<kind>/<id>/<arch>/<branch>|<commit>` line per Flatpak.
- The image builds, boot tests and install tests of all four images run in the `images` workflow (x86_64 on the PC, aarch64 on the Mac); the ISO sizes come from that run. Both Fedora ISOs are expected over 2 GiB with Bazaar's runtime, so they go to R2 (task 021).
- Docs: `site/src/content/docs/user/apps.md` (new: the apps and Flathub), `docs/arch-iso.md` and `docs/fedora-iso.md` (the apps, Bazaar's source, Bazaar's share of the build's tmpfs), `AGENTS.md` (one line).

## Deviations

- The task's open decisions (Bazaar's Flatpak installed at image build on Fedora, Thunar's extras, no key binding for kitty) were taken with their recommended answers and wait for the operator's confirmation.
