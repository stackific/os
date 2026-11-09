# 023: First apps on all four images: kitty, btop, Thunar, File Roller and Bazaar with Flathub

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Give all four images their first everyday apps, live and installed: kitty as a second terminal next to foot (and Ptyxis on Fedora, GNOME's own), btop as the system monitor, Thunar as the file manager for MangoWM with its archive plugin, File Roller for archives, and Bazaar as the app store, with Flathub set up as the system's Flatpak remote. GNOME Disks and a polkit agent are already there (GNOME Disks since the installer tasks, Noctalia's polkit agent on in our defaults), so they stay as they are. On Arch and Arch Linux ARM every app is a native package from the base's repos, Bazaar included. Fedora has no Bazaar package, so Fedora's images install Bazaar from Flathub while the image is built, so it works offline in the live session and on installs. MangoWM's config stays stock: foot stays on Alt+Return, and kitty, Thunar, btop, File Roller and Bazaar start from the launcher. Every image's boot test checks the apps, the Flathub remote and Bazaar.
```

## Goal
kitty, btop, Thunar (with `thunar-archive-plugin` and gvfs), File Roller, Flatpak with the Flathub remote, and Bazaar are on all four images and their installs, checked by the boot and install tests.

## Context
What exists (branch `task/021-hardware-parity-main`, 2026-11-08):
- `arch/iso/packages.x86_64` and `arch/iso/packages.aarch64`: `foot`, `rofi`, Plasma with `konsole` and `dolphin`, `gnome-disk-utility`; no kitty, btop, Thunar, File Roller, Flatpak or Bazaar. The Arch x86_64 manifest has no `gtk4`, `libadwaita`, `webkitgtk-6.0` or `gvfs` (`.build/arch/x86_64/*.packages.txt`, local build of task 021).
- `fedora/iso/config.kiwi`: `gnome-disk-utility`, `foot`, `ptyxis`, `nautilus`; the manifest already has `gvfs` 1.62.0, `gtk4`, `libadwaita`, `webkitgtk6.0`, and no `flatpak`, kitty, btop, Thunar or File Roller.
- `fedora/iso/config.sh` runs in the image root after the packages and `fedora/iso/root/` (kiwi); kiwi gives that chroot the host's `/etc/resolv.conf`, so it has network.
- MangoWM 0.18.4-1's stock `/etc/mango/config.conf` binds `Alt+space` to `rofi -show drun` and `Alt+Return` to `foot` (Arch archive 2026-11-01, read 2026-11-08); `AGENTS.md` keeps it stock.
- The installer copies the live root with `rsync -aHAXx` (`installer/src/copy.rs`), so `/var/lib/flatpak` with its hardlinks reaches installs unchanged, on the `@flatpak` subvolume.
- ISO sizes before this task: Arch x86_64 2069 MiB, Fedora x86_64 1931 MiB (local builds of task 021, 2026-11-08), Arch aarch64 1330 MiB (images run 42536619116). An ISO over 2 GiB goes to R2 (task 021).

Upstream facts (2026-11-08):
- Arch Linux Archive 2026-11-01 (`extra.db`, x86_64) and Arch Linux ARM (`mirror.archlinuxarm.org/aarch64/extra/extra.db`) have the same versions: `kitty` 0.49.2-1 (19.1 MiB download, 76.7 MiB installed), `btop` 1.4.7-1, `thunar` 4.20.10-1, `thunar-archive-plugin` 0.6.0-1, `gvfs` 1.62.0-3, `file-roller` 44.7-1, `flatpak` 1:1.18.4-2, `bazaar` 0.9.9-1 (depends among others on `flatpak`, `gtk4`, `libadwaita`, `webkitgtk-6.0` 2.54.2-1, 38.5 MiB download, `gtksourceview5`, `glycin`, `libmalcontent`).
- Arch's `flatpak` ships `/usr/share/flatpak/remotes.d/flathub.flatpakrepo`; on first use the system installation adds it: `flatpak remotes --system` in the Arch builder after `pacman -S flatpak` prints `flathub https://dl.flathub.org/repo/` (tested 2026-11-08).
- Fedora 45 (`fedora` repo, both architectures): `kitty` 0.48.2-1, `btop` 1.4.7-2, `Thunar` 4.20.10-2 (requires `gvfs` and `tumbler`), `thunar-archive-plugin` 0.6.0-4, `file-roller` 44.7-3, `flatpak` 1.19.0^really1.18.4-1 (owns `/usr/share/flatpak/remotes.d/`, empty). No `bazaar` in Fedora 45 nor in Terra 45. `fedora-flathub-remote` keeps its file under `/usr/lib/fedora-third-party/conf.d/` (active only through `fedora-third-party`, not installed), so Fedora needs its own `remotes.d` file.
- Flathub (`flathub.org/api/v2/summary/io.github.kolunmi.Bazaar`): Bazaar 0.9.10, GPL-3.0-or-later, x86_64 and aarch64, 11.4 MiB download, on `org.gnome.Platform//51` (1.05 GiB installed).

## Inputs
From the operator's plan, row 023: "First software picks: kitty, foot (Arch), Ptyxis (Fedora), btop, Thunar, File Roller, GNOME Disks, a polkit agent for MangoWM if Noctalia has none, Flathub with Bazaar; four images in one task; the Mac lane builds and tests the aarch64 pair." Restated against `AGENTS.md`: foot is already on all four images (the installer's launcher runs in it) and Ptyxis on Fedora; GNOME Disks is already the partition editor; Noctalia's polkit agent is on in our defaults; so this task adds only kitty, btop, Thunar, File Roller and Bazaar with Flathub. `AGENTS.md` says "Bazaar from Flathub" for Fedora; Arch takes its native package. No files copied.

### Open decisions (operator to confirm)
These were taken with the recommended answer, because the operator could not be asked while the task was written:
1. Bazaar on Fedora: installed from Flathub while the image is built (recommended: works offline live and installed, like Arch's native package; costs `org.gnome.Platform//51`, about 1 GiB installed, so the Fedora ISOs go over 2 GiB and to R2). Other answers: only the Flathub remote, Bazaar installed by the user online; or a native RPM from a third-party repo (`AGENTS.md` names Flathub).
2. Thunar's extras: `thunar-archive-plugin` (File Roller from Thunar's menu) and `gvfs` (Trash, removable drives) on Arch; no `tumbler` or `thunar-volman` (Fedora's Thunar pulls `tumbler` itself).
3. No key binding for kitty: MangoWM's config stays stock, so `Alt+Return` stays foot.

## Changes
- `arch/iso/packages.x86_64`, `arch/iso/packages.aarch64`: add `kitty`, `btop`, `thunar`, `thunar-archive-plugin`, `gvfs`, `file-roller`, `flatpak`, `bazaar`.
- `fedora/iso/config.kiwi`: add `kitty`, `btop`, `Thunar`, `thunar-archive-plugin`, `file-roller`, `flatpak`.
- `fedora/packages/stackific-desktop/stackific-desktop.spec`: require `fedora-flathub-remote` and link its Flathub file (`/usr/lib/fedora-third-party/conf.d/fedora-flathub.flatpakrepo`, the same Flathub key as Arch's file, sha256 of the key `f4d6c33e44d9627ad3d2e32a5c8de25769b52f6454157e2db233e7d35d358a9a` in both, 2026-11-08) to `/usr/share/flatpak/remotes.d/flathub.flatpakrepo`, the path Arch's `flatpak` uses; Release bumped. (Built first as a committed copy of Flathub's file; gitleaks' `generic-api-key` rule flags its `GPGKey=` line, so the package's own file is used instead.)
- `fedora/iso/config.sh`: `flatpak install --system --noninteractive flathub io.github.kolunmi.Bazaar`.
- `fedora/justfile` `iso`: append the image's Flatpaks (ref and commit) to its package manifest.
- `tests/vm/arch.sh`, `tests/vm/fedora.sh`: the apps, the Flathub remote, Bazaar.
- `tests/vm/installed.sh`: the apps and Bazaar on installs.
- `docs/arch-iso.md`, `docs/fedora-iso.md`: the apps and Bazaar's source.
- `site/src/content/docs/user/apps.md` (new): the apps on every image and Bazaar with Flathub; sidebar entry if needed.
- `AGENTS.md`: one line for the apps.
- `site/src/content/docs/prompt-trail/023-first-apps.md`, `dev/reviews/023-first-apps.md`.

## Acceptance criteria

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

## Tests
- Boot and install tests above, x86_64 on the PC and aarch64 on the Mac (`images` workflow, `workflow_dispatch` builds all four).
- Operator, once on a live image with network: open Bazaar from the launcher, search for an app, install it, and start it from the launcher.
