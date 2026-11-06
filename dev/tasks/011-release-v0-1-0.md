# 011: Release v0.1.0, the live preview, labelled the way Arch's and Fedora's trademark rules allow

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Prepare the first real release, v0.1.0: a live preview of all four images, with no installer. The release workflow already refuses a version that is not semver, builds and boot-tests the four images, uploads them to a draft, attaches SHA256SUMS and publishes; add what a first release still lacks. Every release and test build opens its notes with one committed notes file. It says this is a live preview with no installer, that the images are unsigned, and how to check a download against SHA256SUMS, and it names what each ISO is built from. Use only wording the bases' trademark rules allow. The Arch Linux x86_64 image is "Based on Arch Linux" and the aarch64 image "Based on Arch Linux ARM". Both Fedora images are "Fedora Remix" images: Fedora's guidelines allow Fedora's name after "based on" only with the Fedora Council's permission. Carry the notice Fedora requires with that mark. GitHub's generated notes for v0.1.0 would start from the last test build, so start them from the newest full release instead, and for the first release link the prompt trail as the changelog. Give the user guide two pages in place of its overview. System requirements covers the architectures, firmware, memory, USB stick, graphics and the hardware left out. Installation covers picking an ISO, checking it, writing it to a USB stick and booting it in UEFI mode with Secure Boot off; until the installer exists it also covers the live session, merged from the live-session page. Remove the overview page, and keep the wallpapers page. Put Red Hat's trademark line in the site footer. Every ISO is under 2 GiB, so do not wire R2. Once the change merges, the operator cuts v0.1.0 with the release workflow.
```

## Goal
After this task merges, the operator runs the `release` workflow with `0.1.0`. It publishes GitHub release `v0.1.0` with the four ISOs, their manifests and `.sha256` files and `SHA256SUMS`, and its notes open with `release/notes.md`. The site's user guide is System requirements, Installation (with the live session) and Wallpapers, and every docs page's footer carries Red Hat's trademark line.

## Context
What exists (main at `57696cc`, checked 2026-11-05):
- `.github/workflows/release.yml` runs only by hand (`workflow_dispatch`, input `version`). Its jobs:
  - `create` (GitHub-hosted) runs `just release create "$VERSION" "$GITHUB_SHA"`.
  - Four image jobs run `just <base> iso`, `just <base> test` and `just release upload "$VERSION" <base> <arch>`: x86_64 on `[self-hosted, Linux, X64]`, aarch64 on `[self-hosted, Linux, ARM64]`.
  - `publish` runs `just release publish "$VERSION"`.
  - The workflow needs no change for this task.
- `release/justfile`:
  - `create` checks the version against `^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$`.
  - For `0.0.0-pr.<n>` it deletes an earlier test release with its tag. It then opens a prerelease titled "Test build for #<n>", notes "Test build of the images after pull request #<n> merged: built and boot-tested, unsigned. Not a release."
  - Any other version must have no release or tag `v<version>` yet. It gets the title "Stackific OS <version>" and `--generate-notes`, and a version with a suffix becomes a prerelease. Every release is created with `--draft --target <sha>`.
  - `upload` writes `<iso>.sha256` and uploads the ISO, `<name>.packages.txt` and the `.sha256` file with `--clobber`.
  - `publish` joins the `.sha256` files into `SHA256SUMS`, uploads it and sets `--draft=false`.
- `.github/workflows/images.yml` calls the same `just release create` with `0.0.0-pr.<n>` after each merge to `main`.
- `docs/releases.md` describes both flows.
- Releases so far (`gh release list -R stackific/os`, 2026-11-05): test builds `v0.0.0-pr.4` to `v0.0.0-pr.10` only. `pr.8` and `pr.10` were drafts at the time (pr.10's `images` run 42142998371 was in progress). There is no full release: `gh release list --exclude-drafts --exclude-pre-releases` prints nothing.
- The site's user guide is `site/src/content/docs/user/` with three pages. `index.md` (order 0, label Overview) says "Stackific OS has no release yet". `live-session.md` (order 1) covers the autologin as `live`, `Alt+Return` and `Alt+space` in MangoWM, and switching to the second desktop: `loginctl terminate-user live`, pick the session, type any character as the password. `wallpapers.md` is order 2.
- Links to `live-session.md` by path: `AGENTS.md` (the live-image line) and `docs/arch-iso.md` line 110. Older tasks and prompt-trail entries name it too; they record the past and stay.
- The docs home `site/src/content/docs/index.md` says "two bases, Arch Linux and Fedora 45", "GNOME on Fedora" and that the image "boots with Limine from its own EFI partition and installs alongside another OS". The first two use a base's name for our images, and the live ISOs boot with systemd-boot (Arch) and GRUB (Fedora), with no installer yet.
- Memory: the boot tests run every image with `-m 4096` (`tests/vm/boot.sh` line 91). The Arch session uses about 710 MiB of it (`site/src/content/docs/prompt-trail/arch/002-arch-desktop-live-iso.md`). The Fedora VM has 3011 MiB available with GNOME running (`dev/reviews/008-fedora-desktop-live-iso.md`).
- Graphics: `stackific-desktop` exports `WLR_RENDERER_ALLOW_SOFTWARE=1`, so MangoWM also starts without a working GPU driver (AGENTS.md, Product).
- Hardware left out: server and datacenter NICs, and hardware that needs a vendor kernel: Microsoft Surface, Apple T2 Macs, Apple Silicon, Raspberry Pi (AGENTS.md, Product).
- Starlight overrides live in `site/src/components/starlight/` and are registered under `components:` in `site/astro.config.mjs`. Each override imports Starlight's default component from `@astrojs/starlight/components/<Name>.astro` (example: `PageTitle.astro`). Starlight 0.42.5 (`site/package.json`) ships `components/Footer.astro`.
- Prompt-trail entries for tasks on both bases sit at `site/src/content/docs/prompt-trail/` root (`site/astro.config.mjs`, comment above the Prompt Trail group). This is the first one.

ISO sizes, all under the 2048 MiB target, so R2 stays unwired (AGENTS.md: "Wired only once an ISO needs it"):

| ISO | Size | Source |
| --- | --- | --- |
| `arch-x86_64` | 1613606912 B (1539 MiB) | `v0.0.0-pr.9` assets |
| `arch-aarch64` | 1396801536 B (1332 MiB) | `v0.0.0-pr.9` assets |
| `fedora-x86_64` | 1561219072 B (1489 MiB) | `v0.0.0-pr.9` assets |
| `fedora-aarch64` | 1431 MiB | `dev/reviews/010-fedora-aarch64-desktop-live-iso.md` |

Boot modes: every image boots in UEFI mode only.
- Arch: `arch/iso/profiledef.sh` sets `bootmodes=('uefi.systemd-boot')`.
- Fedora: `fedora/iso/config.kiwi` sets `firmware="efi" eficsm="false"`.

Boot tests run the images in QEMU with 4 GiB of RAM.

NVIDIA: no image carries NVIDIA's driver. The Arch package lists name no `nvidia` package, and `fedora/iso/config.kiwi` has only `nvidia-gpu-firmware` (line 87). NVIDIA's open modules come in a later task.

Generated release notes (checked 2026-11-05):
- `gh api -X POST repos/stackific/os/releases/generate-notes -f tag_name=v0.1.0 -f target_commitish=main` returned notes for PR #10 only, with "Full Changelog: …/compare/v0.0.0-pr.9...v0.1.0". GitHub takes the newest test-build tag as the previous release, so plain `--generate-notes` would leave out PRs #1 to #9.
- `gh release create --help` (gh 2.102.1 on the host; the repo pins 2.103.3) says: "Additional release notes can be prepended to automatically generated notes by using the `--notes` flag." It also lists `--notes-start-tag string`: "Tag to use as the starting point for generating release notes".
- `gh release list --help` lists `--exclude-drafts`, `--exclude-pre-releases` and `--json`, newest first by default.

Arch Linux trademark policy, version 2021-04-18 (https://terms.archlinux.org/docs/trademark-policy/, read 2026-11-05):
- A product with substantial changes may say it is "derived from Arch Linux", "based on Arch Linux" or "a derivative of Arch Linux". It may not use the marks as its own name.
- Any mark beginning with the letters ARCH is close enough to the trademarks to need permission.
- Logos keep the ™ declaration, and a logo is never combined with other marks. This task uses no logo.

Arch Linux ARM is a separate project from Arch Linux. The aarch64 Arch image builds from its repos (AGENTS.md, Builds).

Fedora trademark guidelines (source `modules/ROOT/pages/trademarks.adoc` in https://gitlab.com/fedora/legal/fedora-legal-docs, last changed 2024-12-05 in commit `a7507b7e`, read 2026-11-05). docs.fedoraproject.org serves the same page but blocks scripted fetches.
- **Permission needed.** Without the Fedora Council's permission, nobody may:
  - use the marks in a way that implies sponsorship, endorsement or affiliation;
  - use them as part of a product name or domain name.
- **Distributions mixing Fedora with other software.** Such a distribution, or one with modified Fedora Materials, may use the Fedora Remix marks if:
  - `fedora-logos`, `fedora-release` and `fedora-release-notes` are removed. AGENTS.md already rules them out, and our `stackific-release` replaces `generic-release`.
  - A notice is prominently displayed. It says the software is not provided or supported by the Fedora Project, and that official Fedora Materials are available from https://fedoraproject.org/, with a link.
- **Fedora Remix section.** Downstream distributors may use the Fedora Remix word mark to show their work includes or is derived from Fedora Materials, if:
  - a prominent statement tells users they receive modified Fedora Materials and where to get unmodified ones;
  - our own name and brand are more prominent than the Fedora Remix mark;
  - nothing implies endorsement or affiliation.
- **Other uses are not permitted.** "Other uses of any Fedora Trademarks to refer to work that includes or is derived from Fedora Materials are not permitted." "Based on Fedora" is such a use.
- **Proper use.** A mark is used as an adjective that modifies a noun, in its exact form, and is never combined with other words. It is set apart by capitals or other styling. The first or most prominent mention of a registered mark (Fedora®) carries ®.
- **Footer notice.** Pages that use a Fedora mark carry this in the footer: "[marks] is/are [a] trademark[s] or registered trademark[s] of Red Hat, Inc., or its subsidiaries in the United States and other countries."
- **Naming rule.** The Fedora Project wiki's Remix page (https://fedoraproject.org/wiki/Remix, read 2026-11-05) adds that the words "Fedora" and "Remix" are never separated.

Left for later, not in this task: the release version in the image. On Arch, mkarchiso already appends `IMAGE_ID=stackific` and `IMAGE_VERSION=<version>-arch` to `/etc/os-release` (task 004's facts). The Fedora images carry no image version.

## Inputs
The operator's working plan for item 011: "`release` workflow and v0.1.0 (live preview, no installer)". Restated:
- v0.1.0 is the first release. It ships the four live ISOs, which boot to the desktop, without an installer.
- Signing waits for its own later task, so v0.x releases say they are unsigned.
- R2 is wired only if an ISO passes 2 GiB.
- The trademark wording that release notes and the site use for both bases is settled before the release.

Most of the workflow already landed with tasks 001 to 010. This task adds what is still missing: the notes, the base labels, the notices and the user docs.

The operator chose the labels on 2026-11-05. They asked for "Based on Fedora" and "Based on Arch" if allowed. After the rules above, they picked:
- "Based on Arch Linux" for the Arch x86_64 ISO;
- "Based on Arch Linux ARM" for the Arch aarch64 ISO;
- "Fedora Remix" for both Fedora ISOs, with Fedora's notice.

No files copied.

## Changes
- `release/notes.md` (new): the text every release's and test build's notes open with. Its content is under Acceptance criteria.
- `release/justfile`: changes to `create`:
  - Test builds: notes are the existing test-build line, a blank line, then `release/notes.md`.
  - Other versions: `--notes` from `release/notes.md`. When a full release exists (newest tag from `gh release list --exclude-drafts --exclude-pre-releases`), add `--generate-notes --notes-start-tag <that tag>`. With none (the first release), add no generated notes.
  - The title stays `--title "Stackific OS <version>"`.
- `site/src/content/docs/user/system-requirements.md` (new, sidebar order 1): what a machine needs to run the images.
- `site/src/content/docs/user/installation.md` (new, sidebar order 2): pick, check, write and boot an ISO, then the live session.
- `site/src/content/docs/user/index.md` (removed): the Overview page.
- `site/src/content/docs/user/live-session.md` (removed): merged into `installation.md`.
- `site/src/content/docs/user/wallpapers.md`: sidebar order 3.
- `site/src/content/docs/index.md`: the base labels in place of the bases' names for our images; v0.1.0 is a live preview; drop the Limine and install claims.
- `AGENTS.md` (live-image line) and `docs/arch-iso.md` line 110: point at `installation.md`.
- `site/src/components/starlight/Footer.astro` (new): Starlight's default footer, then the trademark line.
- `site/astro.config.mjs`: register the `Footer` override, with a one-line comment like the others.
- `docs/releases.md`: the notes file, the labels and where the Fedora notice goes, how generated notes pick their start, and how the operator cuts v0.1.0.
- `AGENTS.md`: the label rule and `release/notes.md`, one line each (text under Acceptance criteria).
- `site/src/content/docs/prompt-trail/011-release-v0-1-0.md` (new, from `/s-build`): the prompt-trail entry.
- `dev/reviews/011-release-v0-1-0.md` (new, from `/s-build`): the review record.

## Acceptance criteria

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

## Tests
- `just lefthook all`: justfile parse, `just dev check`, site format, lint, types, audit and build.
- Notes: `create`'s two branches have no automated test, because every run opens a real release.
  - The builder reads both branches against the criteria. They run `bash -n` on the recipe body: `just --show release::create` piped to a file with the shebang line removed.
  - The builder may preview generated notes with `gh api -X POST repos/stackific/os/releases/generate-notes -f tag_name=v0.1.0 -f target_commitish=main`. It creates nothing.
- Site: `just site build`, then `grep -l` for the strings above in `site/dist/docs/user/installation/index.html`, and `grep -rlE 'href="/docs/user/(live-session/|download/)?"' site/dist` must print nothing. Then `grep -L 'trademarks or registered trademarks of Red Hat, Inc.'` over every `site/dist/docs/**/index.html` must print nothing.
- Wording: the `git grep -niP` line above.

Operator steps after the merge:
1. Wait for the `images` run of the merge commit to pass. Run `gh release view v0.0.0-pr.<n> -R stackific/os --json body --jq .body`. Check that the body starts with "Test build of the images after pull request #<n> merged" and holds the Fedora notice.
2. Open Actions, release, Run workflow on `main` with version `0.1.0`. Wait until all six jobs pass: create, four images, publish.
3. Run `gh release view v0.1.0 -R stackific/os --json isDraft,isPrerelease,tagName,body,assets --jq '{isDraft, isPrerelease, tagName, n: (.assets|length), assets: [.assets[]|"\(.name) \(.size)"]}'`. Check `isDraft` false, `isPrerelease` false and 13 assets. The assets are four ISOs, four `.packages.txt` files, four `.iso.sha256` files and `SHA256SUMS`, and every ISO is 2147483648 bytes or less.
4. Run `gh release list -R stackific/os` and check that `v0.1.0` shows as Latest.
5. In an empty folder, run `gh release download v0.1.0 -R stackific/os -p '*.iso' -p SHA256SUMS`, then `sha256sum -c SHA256SUMS`. It prints four lines ending in `: OK`.
6. Send the four ISO sizes in MiB to the builder for the prompt-trail entry and the review record.
