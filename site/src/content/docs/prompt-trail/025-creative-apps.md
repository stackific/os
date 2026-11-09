---
title: The creative apps as a one-click install from upstream's signed releases
description: A launcher on all four images that downloads 13 creative apps from their developers' GitHub releases, checks each signature and pinned SHA-256, and installs them; none of them ships with Stackific OS.
sidebar:
  order: 25
  label: 025 Creative apps, one-click from upstream
pr: https://github.com/stackific/os/pull/22
task: dev/tasks/025-creative-apps.md
---

## Goal

An installed Stackific OS gets 13 creative apps with one click, every package checked against its developers' signature and a pinned SHA-256 before anything is installed, while our images and repositories carry none of them.

## Context

The plan put the 13 apps from github.com/storytold in every image and in our repositories. They add about 1.3 GiB per ISO, our repositories for installed systems do not exist yet (task 028), and the apps' brand licence allows their name and marks only in their own builds. So the apps stay with their developers: Stackific OS ships only a launcher and a short script that fetch them on the user's machine.

## Expectations

Must:

- `just apps test` prints `ok:` for the good, tampered and re-signed cases; the fake pacman is called once, only in the good case, with both packages and no `.sig` next to them.
- `just arch test` and `just fedora test` (all four images) find `/usr/lib/stackific/install-creative-apps` executable and `/usr/share/applications/stackific-creative-apps.desktop` with `Name=Install creative apps`; `installed.sh` finds both on installed systems.
- No creative app is on any image: the boot test finds no `ai.storyteller.*` launcher in `/usr/share/applications` (upstream's app ID, the same in both formats), and no file of ours outside `apps/creative-apps.sha256`, `apps/creative-apps-key.asc`, docs and the task names ArtCraft in a user-visible name or icon.
- Each image task reports its ISO size (the package adds a few KiB).
- `just lefthook all` passes.

Must not:

- Download, mirror or ship any ArtCraft package in an image, a repo of ours or a release.
- Add NOPASSWD, a polkit rule or any other way around the password.
- Name the launcher, package or any marketing page after ArtCraft.

## Prompt

```text title="Prompt" wrap
Offer the 13 creative apps from github.com/storytold (PhotoCraft, VectorCraft, FilmCraft, LightCraft, PdfCraft, EffectCraft, DesignCraft, FigCraft, WordCraft, SoundCraft, DeckCraft, CADCraft and GridCraft) as a one-click install on all four images, never inside the ISOs and never in our repositories: we do not redistribute them. A desktop launcher named for what it does, "Install creative apps", with a generic icon, opens a terminal that asks for the password through run0 (polkit; never NOPASSWD, never a polkit rule that skips authentication), then downloads each app's pinned version from upstream's GitHub release on the user's machine, the package for the base (.pkg.tar.zst on Arch, .rpm on Fedora) and the architecture (uname -m). It checks every package's signature against upstream's release key, committed to the repo, and its SHA-256 against the pins taken from upstream's signed SHA256SUMS.txt, and installs them all with pacman -U or dnf only when every check passes. One data file holds the pins. Brand terms: upstream's brand licence allows the ArtCraft name and marks only in its unmodified official builds and never in the name, icon, listing or marketing of anything else, so our launcher, package, README and marketing pages never say ArtCraft; the user docs may name the apps in plain text. Keep the glue small: one short bash script in a package of ours, the same on all four images, with a test that runs without a person.
```

## Outcome

- **The package.** `stackific-apps`, built from `apps/` on both bases (`arch/packages/stackific-apps/PKGBUILD`, `fedora/packages/stackific-apps/stackific-apps.spec`), is on all four images. It holds the launcher, the script in `/usr/lib/stackific/` (not on `PATH`), the pins and upstream's key. `just arch packages` and `just fedora packages` now stage sources from `apps/` too.
- **The pins.** `apps/creative-apps.sha256` holds the 52 Linux package lines of the 13 apps' `SHA256SUMS.txt` unchanged (13 apps, x86_64 and aarch64, `.pkg.tar.zst` and `.rpm`), each file verified against upstream's key on 2026-11-08. One upstream signing key signs all 13 releases; it is committed as `apps/creative-apps-key.asc`, and `docs/apps.md` gives its fingerprint.
- **The script.** It runs as root through run0 from the launcher, keeps the pins for its base and `uname -m`, downloads each package and its signature into a fresh folder in `/var/tmp`, checks every signature with `gpgv` and every hash with `sha256sum --check --strict`, then installs all 13 at once. It stops with a message when less than 5 GiB is free, as in the live session. The signatures live in a subfolder, because `pacman -U` would check a `.sig` lying next to a package against pacman's own keyring.
- **Brand terms.** The launcher is "Install creative apps" with the generic `system-software-install` icon; the user page names the apps in plain text.
- **Tests.** `just apps test` runs the script against a fake release of two apps served as `file://` URLs, signed with a throwaway key, with a fake pacman, and joins pre-commit for `apps/**`. The boot and install tests check the launcher and script on every image and install, and that no app is on the image.

### Screenshots

No UI. The launcher opens a terminal; its screen appears only when a person runs it with network.

## Review

- Round 1 found one thing: the test's throwaway key started a gpg-agent that outlived its deleted folder. The test now stops it on exit.
- Round 2 found nothing.

## Manual changes

None.

## Result

- `just apps test` prints `ok: good packages install`, `ok: a tampered package stops` and `ok: a re-signed package with another hash stops`; `just lefthook all` passes all 16 checks, `apps` among them.
- The image builds and their boot and install checks run in the `images` workflow on the PC (x86_64) and the Mac (aarch64).
- Docs: `docs/apps.md` (how it works, the test, updating the pins) and the user page `user/creative-apps.md`; `AGENTS.md` describes the new design and `apps/`.

## Deviations

- The plan's "in both repos, installed on every image" became a one-click install from upstream on the user's machine (operator 2026-11-08). The task says so.
- The pins are the 2026-11-06 releases, each one minor version newer than the plan's 2026-11-04 list (PhotoCraft 0.21.0, VectorCraft and EffectCraft 0.22.0, GridCraft 0.17.0, the rest 0.16.0 to 0.19.0).
