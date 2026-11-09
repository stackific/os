# 025: The creative apps as a one-click install from upstream's signed releases

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Offer the 13 creative apps from github.com/storytold (PhotoCraft, VectorCraft, FilmCraft, LightCraft, PdfCraft, EffectCraft, DesignCraft, FigCraft, WordCraft, SoundCraft, DeckCraft, CADCraft and GridCraft) as a one-click install on all four images, never inside the ISOs and never in our repositories: we do not redistribute them. A desktop launcher named for what it does, "Install creative apps", with a generic icon, opens a terminal that asks for the password through run0 (polkit; never NOPASSWD, never a polkit rule that skips authentication), then downloads each app's pinned version from upstream's GitHub release on the user's machine, the package for the base (.pkg.tar.zst on Arch, .rpm on Fedora) and the architecture (uname -m). It checks every package's signature against upstream's release key, committed to the repo, and its SHA-256 against the pins taken from upstream's signed SHA256SUMS.txt, and installs them all with pacman -U or dnf only when every check passes. One data file holds the pins. Brand terms: upstream's brand licence allows the ArtCraft name and marks only in its unmodified official builds and never in the name, icon, listing or marketing of anything else, so our launcher, package, README and marketing pages never say ArtCraft; the user docs may name the apps in plain text. Keep the glue small: one short bash script in a package of ours, the same on all four images, with a test that runs without a person.
```

## Goal
An installed Stackific OS gets the 13 creative apps from upstream with one click, every package checked against upstream's signature and our pinned SHA-256 before anything is installed; our images and repos carry none of them.

## Context
What exists (branch `task/021-hardware-parity-main` at 14a98e2, 2026-11-08):
- No creative apps on any image. Our packages are built from `arch/packages/*/PKGBUILD` and `fedora/packages/*/*.spec`, sources staged by file name from `desktop/`, `branding/`, `hardware/` and `installer/` (`arch/justfile` and `fedora/justfile` `packages`).
- The installer's launcher (`installer/stackific-installer.desktop.in`) runs `foot … run0 stackific-installer`: foot is on all four images, and run0 asks for the password through polkit (Noctalia's polkit agent in MangoWM, the desktops' own in Plasma and GNOME).
- `just release changed` picks images by path; `lefthook.yml` runs `just hardware test` for `hardware/**`.

Upstream facts (GitHub releases of github.com/storytold/<app>, read 2026-11-08 with `gh release view` and `gh release download`):
- Latest releases, all published 2026-11-06: photocraft v0.21.0, vectorcraft v0.22.0, filmcraft v0.18.0, lightcraft v0.19.0, pdfcraft v0.19.0, effectcraft v0.22.0, designcraft v0.18.0, figcraft v0.16.0, wordcraft v0.16.0, soundcraft v0.18.0, deckcraft v0.16.0, cadcraft v0.18.0, gridcraft v0.17.0.
- Each release has `<app>-<version>-linux-{x86_64,aarch64}.pkg.tar.zst` with a binary detached OpenPGP signature `.pkg.tar.zst.sig`, `.rpm` with an armored `.rpm.asc`, `SHA256SUMS.txt`, `SHA256SUMS.txt.asc` and `release-signing-key.asc`. All 13 key files are byte-identical (sha256 `8e6925a173d6a7134118da099948de4d9eeec7c1c0770998aec33c9d3f484740`): "ArtCraft Linux Release Signing Key", primary `2B76741B414AAD6CD33E631213A005C0B9D7C3ED`, subkey `568DC1F68F8FAECC797F3C7B006DAF6BB0EF334E`. Every `SHA256SUMS.txt` verifies against it with `gpgv`, and so does `photocraft-0.21.0-linux-x86_64.pkg.tar.zst.sig`.
- PhotoCraft's x86_64 `.pkg.tar.zst` is 97 MiB, installs 237 MiB (`.PKGINFO` `size = 248152398`) as `pkgname = ai.storyteller.photocraft` (the `.rpm` is named `photocraft`, `packaging/linux/nfpm.yaml`), `license = MIT OR Apache-2.0`, depending only on Arch repo packages (`glibc gcc-libs hicolor-icon-theme libxkbcommon libxkbcommon-x11 vulkan-icd-loader`); the `.rpm` requires sonames (`packaging/linux/nfpm.yaml`). The 13 apps download about 1.3 GiB and install about 3 GiB.
- Brand terms (`docs/brand/LICENSE-brand.txt` and `NOTICE` in photocraft, 2026-11-08): code under MIT or Apache-2.0; the ArtCraft name, wordmark and logos are trademarks, allowed only in copies of the repository and within the app's official builds; never in the name, logo, icon, listing, advertising or marketing of anything else.
- `gpgv` 2.4 reads only a binary keyring (`keydb_search failed: Invalid packet` with the armored key), so the key is dearmored with `gpg --dearmor` at run time. pacman's `LocalFileSigLevel = Optional` (Arch's `/etc/pacman.conf`) checks a `<file>.sig` lying next to a package given to `pacman -U` against pacman's own keyring, which does not hold upstream's key, so the signatures are kept in another folder.
- Asset names use `uname -m`'s words (`x86_64`, `aarch64`), so the script picks `linux-$(uname -m)`.

## Inputs
From the operator's plan, row 025 and SPEC.md ("The ArtCraft suite", question 12), restated: the 13 apps above, from upstream's own release packages, checked against upstream's checksums and pinned by version. Operator decisions (2026-11-08) that replace the plan's "in both repos, installed on every image": not in the ISOs and not in our repos, but a one-click install from upstream on the user's machine, so we never redistribute them and question 12 (may we redistribute) no longer applies; the launcher never names or shows ArtCraft. Files copied: `apps/creative-apps-key.asc` (upstream's `release-signing-key.asc`, unchanged) and `apps/creative-apps.sha256` (the 52 Linux package lines of the 13 signed `SHA256SUMS.txt`, unchanged, under a comment header).

## Changes
- `apps/install-creative-apps`: the script (bash, `set -euo pipefail`); `apps/creative-apps.sha256`: the pins; `apps/creative-apps-key.asc`: upstream's key; `apps/stackific-creative-apps.desktop`: the launcher.
- `apps/justfile`: `test`, the script against a fake release served from files (`file://`), with a throwaway key: good packages install, a tampered package and a re-signed package with another hash both stop before installing.
- `arch/packages/stackific-apps/PKGBUILD`, `fedora/packages/stackific-apps/stackific-apps.spec`: the new package `stackific-apps`.
- `arch/justfile`, `fedora/justfile` `packages`: stage sources from `apps/` too.
- `arch/iso/packages.x86_64`, `arch/iso/packages.aarch64`, `fedora/iso/config.kiwi`: `stackific-apps`.
- `justfile` (`mod apps`), `lefthook.yml` (`just apps test` for `apps/**`), `release/justfile` `changed` (`apps/` in the common paths).
- `tests/vm/boot.sh`, `tests/vm/installed.sh`: the launcher and script are installed, live and installed.
- `docs/apps.md`, `site/src/content/docs/user/creative-apps.md`, `AGENTS.md`, `site/src/content/docs/prompt-trail/025-creative-apps.md`, `dev/reviews/025-creative-apps.md`.

## Acceptance criteria

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

## Tests
- `just apps test` (pre-commit and `check`).
- `just arch test`, `just fedora test`, `just arch test-install`, `just fedora test-install` through the `images` workflow.
- Manual, on an installed system with network:
  1. Open "Install creative apps" from the app launcher and type the password.
  2. Wait for "Done"; PhotoCraft and the other 12 are in the app launcher.
