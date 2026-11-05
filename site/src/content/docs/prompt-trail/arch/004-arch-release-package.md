---
title: Arch release package, the image identifies as Stackific OS
description: stackific-release owns /etc/os-release and the logo icon, so the Arch live image reports Stackific OS while Arch's own files stay as shipped.
sidebar:
  order: 4
  label: 004 Arch release package
pr: https://github.com/stackific/os/pull/4
task: dev/tasks/004-arch-release-package.md
---

## Goal

`just arch packages` builds `stackific-release`, which owns `/etc/os-release` (`ID=stackific`, `PRETTY_NAME="Stackific OS"`) and the icon `stackific`. `just arch iso` installs it, and `just arch test` proves the live image identifies as Stackific OS, while Arch's `filesystem` package and `/etc/arch-release` stay as shipped.

## Context

Tasks 001 and 002 built the Arch x86_64 live ISO and our first package, `stackific-desktop`, but the image still identified as Arch Linux. This step adds a second package, `stackific-release`, from base-neutral files in `branding/`, and boot-test checks for the image's identity.

## Expectations

Must:

- `sha256sum branding/logo.svg branding/logo-color.svg` prints `43625b2beb9010a2ec1686c31e3b7173921c702b39f9050b2673ae6087a0514e` and `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`.
- `diff <(sed 's/currentColor/#007a55/' branding/logo.svg) branding/logo-color.svg` prints nothing: the emerald copy is the same mark.
- `just arch packages` exits 0 and leaves `.build/arch/repo/stackific-release-0.1.0-1-any.pkg.tar.zst`.
  - Besides pacman's metadata files, `bsdtar -tf` on it lists only `etc/os-release` and `usr/share/icons/hicolor/scalable/apps/stackific.svg` (and their directories).
  - `bsdtar -xOf <pkg> etc/os-release` prints the ten lines of `branding/os-release`, then `ID_LIKE=arch` and `BUILD_ID=rolling`, and nothing else.
  - `cmp branding/logo-color.svg <(bsdtar -xOf <pkg> usr/share/icons/hicolor/scalable/apps/stackific.svg)` prints nothing.
- `stackific-desktop` still builds unchanged into the same repo.
- `just arch iso` exits 0, and its last line prints the ISO size in MiB. The manifest lists `stackific-release 0.1.0-1` and `filesystem 2025.10.12-1`.
- `just arch test` exits 0. It prints `ok` for the five identity checks in Changes and for every check it ran before.
- `just lefthook all` passes (shellcheck on the new PKGBUILD and on `tests/vm/boot.sh`).
- The task report states the ISO size in MiB against the 2048 MiB target.

Must not:

- change a file owned by `filesystem` or any other Arch package: no pacman hook, no `sed` on `/usr/lib/os-release`, no fork of `filesystem` and no install scriptlet;
- delete, replace or shadow `/etc/arch-release`;
- set `VERSION` or `VERSION_ID` on Arch, or put a per-base field into `branding/os-release`;
- recolour the logo at build time;
- run the Arch-only checks on an ISO whose name is not `*-arch-*.iso`;
- edit `branding/logo.svg` or `branding/logo-color.svg`;
- touch files outside the Changes list (the review record and prompt-trail entry from /s-build excepted);
- add a dependency this task does not name.

## Prompt

```text title="Prompt" wrap
Add an Arch release package, stackific-release, so the Arch live image identifies as Stackific OS: os-release with ID=stackific, the name "Stackific OS", the project URLs (site https://os.stackific.com, repo https://github.com/stackific/os) and our logo. Keep it small and quick. Ship it without patching any Arch package and without pacman file conflicts, so a later upgrade of Arch's filesystem package neither undoes nor breaks it, and keep /etc/arch-release as Arch ships it. Check every os-release field against os-release(5), and settle how a rolling base with semver releases states its version. The logo is the operator's mark, committed as branding/logo.svg; install it so LOGO= resolves as an icon, in the brand colour emerald #007a55, from a second committed SVG coloured emerald rather than recoloured at build time. On Arch set BUILD_ID=rolling, as Arch does; point SUPPORT_URL at GitHub issues; set DEFAULT_HOSTNAME=stackific. Keep the os-release and logo sources base-neutral, so Fedora's stackific-release RPM can reuse them later with only its per-base fields changed. Install the package into the Arch ISO, and prove in the boot test that the image reports ID=stackific and that hostnamectl shows Stackific OS, without running Arch-only checks on the Fedora ISOs that share the test.
```

## Outcome

`branding/os-release` holds the ten base-neutral fields. `arch/packages/stackific-release/PKGBUILD` writes them to `/etc/os-release`, appends Arch's `ID_LIKE=arch` and `BUILD_ID=rolling`, and installs `branding/logo-color.svg` as the hicolor icon `stackific`. Owning `/etc/os-release` is enough: os-release(5) readers use it instead of `/usr/lib/os-release`, and systemd's tmpfiles `L` line leaves an existing file alone, so `filesystem` stays untouched. No Arch package owns `/etc/os-release`, so pacman sees no conflict. mkarchiso now appends `IMAGE_ID` and `IMAGE_VERSION` to our file, which carries the ISO's semver version into the image. `just arch packages` stages sources from `branding/` as well as `desktop/`, and the ISO lists the package after `stackific-desktop`. The boot test adds five identity checks to the Arch checks, `tests/vm/arch-x86_64.sh` (after rebasing onto task 003, which split the checks per image), so they never run on Fedora ISOs.

### Screenshots

No UI.

## Review

- Round 1 found one small thing: `package()` created the directory, wrote os-release and set its mode in three lines, where `install -Dm644` followed by an append does it in two. Fixed; the package's files came out byte-identical.
- Round 2 found nothing.

## Manual changes

None.

## Result

`stackific-release 0.1.0-1` is in the local repo and in the Arch live ISO, next to `stackific-desktop` and Arch's untouched `filesystem 2025.10.12-1`.

- `sha256sum` on the two logos prints the pinned sums, and the `currentColor` to `#007a55` diff prints nothing.
- `just arch packages` exits 0. `bsdtar` shows only `etc/os-release` and `usr/share/icons/hicolor/scalable/apps/stackific.svg`; os-release is `branding/os-release` plus `ID_LIKE=arch` and `BUILD_ID=rolling`, and the icon matches `branding/logo-color.svg`.
- `just arch iso` exits 0: 1497 MiB against the 2048 MiB target. The manifest lists `stackific-release 0.1.0-1`, `stackific-desktop 0.1.0-1` and `filesystem 2025.10.12-1`.
- `just arch test` prints `PASS`, with `ok` for the five identity checks (`stackific/arch/rolling/Stackific OS/stackific/stackific`, `Operating System: Stackific OS`, `stackific-release filesystem filesystem`, no `IMAGE_` lines in `/usr/lib/os-release`, the emerald icon) and every earlier check.
- `just lefthook all` passes.

Docs changed: `docs/arch-iso.md` (Our packages, the `stackific-release` part, Boot test) and `AGENTS.md`.

## Deviations

None.
