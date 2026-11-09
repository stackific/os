---
title: First apps on all four images
description: kitty, btop, Thunar with thumbnails and File Roller, and Flathub on every image from each base's own packages, with Bazaar as the store on the Arch images.
sidebar:
  order: 23
  label: 023 First apps on all four images
pr: https://github.com/stackific/os/pull/27
task: dev/tasks/023-first-apps.md
---

## Goal

kitty, btop, Thunar (with `thunar-archive-plugin`, `tumbler` and gvfs), File Roller and Flatpak with the Flathub remote are on all four images and their installs, with Bazaar on the two Arch images, checked by the boot and install tests.

## Context

The images had a terminal (foot, plus Ptyxis on Fedora), the second desktops' own apps, GNOME Disks and the installer, but no file manager or app store for the MangoWM desktop. Noctalia's polkit agent was already on, so the plan's polkit item needed nothing. Arch and Arch Linux ARM package every app, Bazaar included; on Fedora, GNOME Software (task 024) is the store.

## Expectations

Must:

- `just arch test` (both architectures) passes with `pacman -Q kitty btop thunar thunar-archive-plugin tumbler gvfs file-roller flatpak bazaar` listing 9 packages and `flatpak remotes --system --columns=name` printing `flathub`.
- `just fedora test` (both architectures) passes with the seven RPMs (`kitty btop Thunar thunar-archive-plugin tumbler file-roller flatpak`) installed and `flatpak remotes --system --columns=name` printing `flathub`.
- `just arch test-install` and `just fedora test-install` pass with the apps on the installed system (Bazaar too on Arch).
- Each image task reports its ISO size; `just lefthook all` passes.

Must not:

- Change MangoWM's stock config or bind a key.
- Add a repo file for a third-party RPM repository.
- Add GNOME Disks, a polkit agent or foot again.
- Install Bazaar or any Flatpak into a Fedora image, or add `thunar-volman`.

## Prompt

```text title="Prompt" wrap
Give all four images their first everyday apps, live and installed: kitty as a second terminal next to foot (and Ptyxis on Fedora, GNOME's own), btop as the system monitor, Thunar as the file manager for MangoWM with its archive plugin, tumbler for thumbnails and gvfs for Trash and removable drives (no thunar-volman), File Roller for archives, and Flathub set up as the system's Flatpak remote. GNOME Disks and a polkit agent are already there (GNOME Disks since the installer tasks, Noctalia's polkit agent on in our defaults), so they stay as they are. On Arch and Arch Linux ARM every app is a native package from the base's repos, with Bazaar, Arch's own package, as the app store; flatpak's own remotes.d file adds Flathub. Fedora's flatpak ships no remote, and fedora-flathub-remote keeps Flathub's file where only fedora-third-party reads it, so stackific-desktop links that file to the path Arch uses; commit no copy of Flathub's key, which gitleaks flags. Fedora gets no Bazaar: GNOME Software (task 024) is its store and already browses and installs Flathub's apps, Bazaar among them, and a second store installed from Flathub would cost about 1 GiB of ISO and of the live session's RAM. This deviates from the SPEC table, which had Bazaar preinstalled on Fedora. MangoWM's config stays stock: foot stays on Alt+Return, and the other apps start from the launcher. Every image's boot test checks the apps and the Flathub remote, and the install tests check the apps on the installed system.
```

## Outcome

- **Arch, both architectures.** `packages.x86_64` and `packages.aarch64` gain `kitty`, `btop`, `thunar`, `thunar-archive-plugin`, `tumbler`, `gvfs`, `file-roller`, `flatpak` and `bazaar`, the same lines in the same place. Arch's `flatpak` ships `/usr/share/flatpak/remotes.d/flathub.flatpakrepo`, which flatpak adds to the system installation on first use, so nothing of ours is needed for Flathub.
- **Fedora, both architectures.** `config.kiwi` adds `kitty`, `btop`, `Thunar`, `thunar-archive-plugin`, `tumbler`, `file-roller` and `flatpak`. `stackific-desktop` requires `fedora-flathub-remote` and links its Flathub file (`/usr/lib/fedora-third-party/conf.d/`, read otherwise only by `fedora-third-party`) to the path Arch uses, so no key is committed. No Bazaar and no Flatpak in the image: GNOME Software (task 024) is Fedora's store.
- **Installs.** The installer copies the live root, so the apps and the Flathub remote arrive unchanged.
- **Tests.** `tests/vm/arch.sh` and `tests/vm/fedora.sh` check the packages and the Flathub remote; `tests/vm/installed.sh` checks the apps on the installed system.

### Screenshots

No UI.

## Review

- **Round 1.** Two findings on the first build, which installed Bazaar's Flatpak into the Fedora images: its runtime would push the Fedora aarch64 build's tmpfs past half the Mac VM's RAM (task 022, merged first, already mounts it with `size=75%`), and nothing checked that the manifest listed Bazaar (a guard was added). Both went away with the operator's decision below, which drops the Flatpak.
- **Round 2.** Nothing found.
- **Round 3 (pre-commit gitleaks).** gitleaks' `generic-api-key` rule flagged the `GPGKey=` line of a committed copy of Flathub's file, a public key. Fixed by linking `fedora-flathub-remote`'s file instead.
- **Round 4.** Nothing found.
- **Round 5 (operator decisions, 2026-11-09).** No Bazaar on Fedora, `tumbler` on both bases, no key for kitty. Fedora's Flatpak install, the manifest's Flatpak lines with their guard and Bazaar's checks were removed; `tumbler` joined both package lists and the checks.
- **Round 6.** Nothing found.

## Manual changes

None.

## Result

- `just lefthook all` passes (15 checks); shellcheck passes on the changed test scripts and `config.sh`; Prettier passes on the new pages.
- The image builds, boot tests and install tests of all four images run in the `images` workflow (x86_64 on the PC, aarch64 on the Mac); the ISO sizes come from that run.
- Docs: `site/src/content/docs/user/apps.md` (new: the apps, and Flathub through Bazaar or GNOME Software), `docs/arch-iso.md` and `docs/fedora-iso.md` (the apps, Flathub's remote, why Fedora has no Bazaar), `AGENTS.md` (the apps line, Fedora's sources line).

## Deviations

- Fedora has no Bazaar, against the SPEC table, which had it preinstalled (operator 2026-11-09): GNOME Software (task 024) is Fedora's store and installs Flathub's apps, Bazaar among them. The task file records the decision.
- The user page names GNOME Software as Fedora's store, which task 024 (PR #24) adds.
