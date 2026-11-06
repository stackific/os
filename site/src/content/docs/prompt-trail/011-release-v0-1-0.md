---
title: Release v0.1.0, the live preview, labelled the way Arch's and Fedora's trademark rules allow
description: Release notes, base labels and Fedora's notice for the first release, and a user guide with system requirements and installation.
sidebar:
  order: 11
  label: 011 Release v0.1.0
pr: https://github.com/stackific/os/pull/11
task: dev/tasks/011-release-v0-1-0.md
---

## Goal

Make the first real release, v0.1.0, ready to cut: a live preview of all four images, with notes that say what it is, what each ISO is built from in words the bases' trademark rules allow, and a user guide that tells people what they need and how to try it.

## Context

The release workflow already refused versions that are not semver, built and boot-tested all four images, uploaded them to a draft, attached `SHA256SUMS` and published. Its notes were only GitHub's generated list, which would have started at the newest test build and listed one pull request. The user guide had an overview page that said there was no release yet.

## Expectations

The wording below is fixed. `release/notes.md` and `installation.md` carry these sentences verbatim.

- Labels: `Based on Arch Linux` for `stackific-<version>-arch-x86_64.iso`, `Based on Arch Linux ARM` for `stackific-<version>-arch-aarch64.iso`, and `Fedora Remix` for `stackific-<version>-fedora-x86_64.iso` and `stackific-<version>-fedora-aarch64.iso`.
- Fedora notice: "The Fedora Remix images contain modified Fedora® packages and software that the Fedora Project does not provide. They are not provided or supported by the Fedora Project. Official, unmodified Fedora releases are available at https://fedoraproject.org/."
- Non-affiliation: "Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat."
- Trademark line: "Fedora and Fedora Remix are trademarks or registered trademarks of Red Hat, Inc., or its subsidiaries in the United States and other countries."

Must:

- `release/notes.md` says the release is a live preview with no installer: nothing is written to the disk, and changes are gone after a reboot. It says the images are unsigned: `SHA256SUMS` shows a download is intact, not who built it.
- `release/notes.md` has a table of the four ISOs by file-name suffix (`-arch-x86_64.iso` and so on). Each row gives its label and its second desktop: KDE Plasma on the Arch ISOs, GNOME on the Fedora ISOs.
- `release/notes.md` carries the Fedora notice, the non-affiliation sentence and the trademark line, and the check command `sha256sum -c SHA256SUMS --ignore-missing`.
- `release/notes.md` links the installation page https://os.stackific.com/docs/user/installation/ and the prompt trail https://os.stackific.com/docs/prompt-trail/ as the record of every change.
- `create` for a test version builds notes that start with "Test build of the images after pull request #<n> merged" and then hold all of `release/notes.md`.
- For a non-test version with no full release yet, `create` passes `release/notes.md` as the notes and no `--generate-notes`. With a full release, it adds `--generate-notes --notes-start-tag v<newest full release>`. The semver check, the exists check, the draft, the target and the title stay as they are.
- `system-requirements.md` states:
  - the architectures: x86_64 for 64-bit Intel and AMD PCs; aarch64 for 64-bit ARM machines with UEFI firmware, tested in QEMU;
  - UEFI firmware with Secure Boot off (no legacy BIOS);
  - 4 GiB of RAM, what the images are tested with;
  - a USB stick of 4 GB or more;
  - graphics: AMD and Intel GPUs, NVIDIA without NVIDIA's driver yet, and software rendering when no GPU driver works;
  - the hardware left out, as in Context.
- `installation.md` says there is no installer yet. It covers the following, with the four labels, the Fedora notice and the non-affiliation sentence:
  1. Pick the ISO: the four labels and second desktops, and a link to System requirements for the architecture.
  2. Download the ISO and `SHA256SUMS` from https://github.com/stackific/os/releases/latest.
  3. Check the download: `sha256sum -c SHA256SUMS --ignore-missing` on Linux, `shasum -a 256 -c SHA256SUMS --ignore-missing` on macOS. Either prints `<iso>: OK`.
  4. Write it to a USB stick with `dd`, after a warning that it erases the stick, and `lsblk` to find the device.
  5. Boot in UEFI mode with Secure Boot off.
  6. The live session, with everything `live-session.md` said: the autologin, the two MangoWM keys and the three steps to the second desktop.

  It also states three limits: the images are unsigned, there is no installer yet, and there is no NVIDIA driver yet.

- In `installation.md` and `release/notes.md`, the first standalone "Fedora" carries ®. "Fedora" and "Remix" are never separated, and every Fedora mark modifies a noun ("Fedora Remix images", "Fedora 45 packages").
- `just site build` exits 0. In `site/dist/`:
  - `docs/user/installation/index.html` contains `Based on Arch Linux ARM`, `Fedora Remix`, `https://fedoraproject.org/`, `not provided or supported by the Fedora Project` and `terminate-user live`;
  - every `docs/**/index.html` contains `trademarks or registered trademarks of Red Hat, Inc.`;
  - `docs/user/index.html`, `docs/user/download/` and `docs/user/live-session/` do not exist.
- The User Guide sidebar reads System requirements, Installation, Wallpapers.
- No built page links `/docs/user/live-session/`, `/docs/user/download/` or `/docs/user/` alone.
- `git grep -niP 'based on (fedora|arch(?! linux))' -- release site docs AGENTS.md README.md` prints nothing.
- `AGENTS.md` gains two lines:
  - The base labels: `Based on Arch Linux` (Arch x86_64), `Based on Arch Linux ARM` (Arch aarch64) and `Fedora Remix` (both Fedora images), Fedora's name never after "based on", and no base logos. The Fedora notice and Red Hat's trademark line go wherever the Fedora Remix mark appears.
  - `release/notes.md`: it opens every release's and test build's notes, and generated notes start from the newest full release.
- `just lefthook all` passes.
- After the merge, the operator's steps under Tests show:
  - the next test build's notes hold the notice;
  - `v0.1.0` is published, not a prerelease and marked Latest, with 13 assets;
  - `sha256sum -c` prints `OK` for all four ISOs.

  The prompt-trail entry and `dev/reviews/011-release-v0-1-0.md` record the result and the four ISO sizes against 2048 MiB.

Must not:

- Put Fedora's name, or Arch without Linux, after "based on"; or use any Arch, Arch Linux ARM, Fedora or Fedora Remix logo.
- Use a base's mark in a product name, or say the release is signed.
- Change the workflows, the ISO file names, os-release, any package or image, or wire R2.
- Touch files outside the Changes list, or add a dependency.

## Prompt

```text title="Prompt" wrap
Prepare the first real release, v0.1.0: a live preview of all four images, with no installer. The release workflow already refuses a version that is not semver, builds and boot-tests the four images, uploads them to a draft, attaches SHA256SUMS and publishes; add what a first release still lacks. Every release and test build opens its notes with one committed notes file. It says this is a live preview with no installer, that the images are unsigned, and how to check a download against SHA256SUMS, and it names what each ISO is built from. Say "based on" for each base where the bases' trademark rules allow it, and use only wording they allow. The Arch Linux x86_64 image is "Based on Arch Linux" and the aarch64 image "Based on Arch Linux ARM". Both Fedora images are "Fedora Remix" images: Fedora's guidelines allow Fedora's name after "based on" only with the Fedora Council's permission. Carry the notice Fedora requires with that mark, and never name our images by a base's mark on public pages. GitHub's generated notes for v0.1.0 would start from the last test build, so start them from the newest full release instead, and for the first release link the prompt trail as the changelog. Give the user guide two pages in place of its overview. System requirements covers the architectures, firmware, memory, USB stick, graphics and the hardware left out. Installation covers picking an ISO, checking it, writing it to a USB stick and booting it in UEFI mode with Secure Boot off; until the installer exists it also covers the live session, merged from the live-session page. Remove the overview page, and keep the wallpapers page. Put Red Hat's trademark line in the site footer. Every ISO is under 2 GiB, so do not wire R2. Once the change merges, the operator cuts v0.1.0 with the release workflow.
```

## Outcome

`release/notes.md` now opens every release's notes. It says v0.1.0 is a live preview with no installer and unsigned images, shows how to check a download, and lists the four ISOs with their labels and second desktops. It closes with Fedora's notice, a sentence saying Stackific OS is not affiliated with any base, and Red Hat's trademark line. `just release create` prepends its test-build line for `0.0.0-pr.<n>`. A full release gets the file as its notes, plus GitHub's generated notes started at the newest full release (`--notes-start-tag`). The first release has none to start from, so it gets the file alone, which links the prompt trail. Checked against GitHub on 2026-11-05: left alone, GitHub started v0.1.0's generated notes at `v0.0.0-pr.9` and listed only pull request #10.

The labels follow the two policies. Arch's trademark policy allows "based on Arch Linux", and the aarch64 image is built from Arch Linux ARM. Fedora's guidelines allow only the Fedora Remix mark for a distribution made from modified Fedora packages, with a notice that it is not provided or supported by the Fedora Project and where official Fedora is.

The user guide is now System requirements, Installation and Wallpapers. Installation takes in the old live-session page. The overview page is gone. A Starlight `Footer` override puts Red Hat's trademark line under every docs page. The docs home names the images by their labels, and no longer claims Limine or an installer.

### Screenshots

No UI.

## Review

Round 1 found one gap. The installation page showed macOS readers how to check a download, but its write step used `lsblk` and `/dev/sdX`, which exist only on Linux. A macOS paragraph now finds the stick with `diskutil list`, unmounts it and writes to the raw `/dev/rdiskN` device.

Round 2 found nothing. It also confirmed that `create` reads `release/notes.md` from the repo root, as the module's `upload` recipe already does.

During the build, the operator asked to move the trademark line to the bottom of the left nav. Fedora's guidelines word it as the page footer, and on phones the nav sits behind the menu button, so the operator kept the footer. The line is 12 px, in the faintest theme colour that still reads at 4.5:1 contrast in both themes.

## Manual changes

Cutting v0.1.0 is the operator's: the `release` workflow runs only by hand, after this change merges (`docs/releases.md`, Cutting v0.1.0).

## Result

`just release create` was run against a stand-in `gh` that prints its arguments, so no release was opened:

- `0.0.0-pr.99`: a prerelease whose notes are the test-build line, then `release/notes.md`.
- `0.1.0` with no full release yet: the notes file alone, with no generated notes.
- `0.2.0-rc.1` after `v0.1.0`: generated notes from `--notes-start-tag v0.1.0`, as a prerelease.
- `1.2`: refused as not semver.

`just site build` exits 0, and all 16 docs pages carry the trademark line in their footer. The installation page carries the labels, Fedora's notice and the live-session steps. The old overview, download and live-session pages are gone, and no page links them. The wording check finds no base name after "based on" other than Arch Linux. `just lefthook all` passes.

The ISOs stay under the 2048 MiB target: 1539 MiB (Arch x86_64), 1332 MiB (Arch aarch64) and 1489 MiB (Fedora x86_64) in the latest test build, and 1431 MiB (Fedora aarch64) from task 010. The v0.1.0 sizes follow once the operator cuts it.

Docs pages added or changed:

- User guide: System requirements and Installation, both new; Wallpapers moved to third.
- The docs home.
- Developer notes: `docs/releases.md` (release notes, base labels, cutting v0.1.0) and `docs/arch-iso.md`.
- `AGENTS.md`.

## Deviations

- The operator changed the user guide's layout while the task was being built: System requirements and Installation, with the live session folded in, in place of Overview and a Download page. The task file was updated first.
- The docs home (`site/src/content/docs/index.md`), `AGENTS.md`'s live-image line and `docs/arch-iso.md` pointed at the old pages or named our images by a base's mark; they changed too, and the task's Changes list now names them.
- The rule lines in `AGENTS.md`, `docs/releases.md` and the task describe the forbidden wording without spelling it, so the task's wording check stays strict.
