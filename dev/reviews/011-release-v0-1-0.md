# 011: Release v0.1.0, the live preview, labelled the way Arch's and Fedora's trademark rules allow: review

Task: dev/tasks/011-release-v0-1-0.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `release/notes.md`, `release/justfile` (`create`), `site/src/components/starlight/Footer.astro`, `site/astro.config.mjs`, the user guide (`system-requirements.md`, `installation.md`, `wallpapers.md`; `index.md`, `download.md` and `live-session.md` removed), the docs home, `docs/releases.md`, `docs/arch-iso.md`, `AGENTS.md`, the prompt-trail entry and the task file, against the task's acceptance criteria.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | macOS readers get a check step but no write step: `lsblk` and `/dev/sdX` exist only on Linux | `site/src/content/docs/user/installation.md`, Write it to a USB stick | fixed | One macOS paragraph: `diskutil list`, `diskutil unmountDisk /dev/disk4`, then `sudo dd … of=/dev/rdisk4 bs=4m` |

Tests after the fixes: `just site format-check` passes; `just site build` exits 0 (18 pages) and `docs/user/installation/index.html` contains `diskutil unmountDisk`.

Before the review, during the build:

- `create`'s three branches ran against a stub `gh` that prints its arguments (no release opened):
  - `0.0.0-pr.99` gives `--prerelease --title "Test build for #99" --notes` with the test line, a blank line and `release/notes.md`.
  - `0.1.0` with no full release gives `--title "Stackific OS 0.1.0" --notes <notes.md>` and no `--generate-notes`.
  - `0.2.0-rc.1` with `v0.1.0` gives `--generate-notes --notes-start-tag v0.1.0 --prerelease`.
  - `1.2` exits 1 with `version must be semver, got '1.2'`.
- `just site build`: every one of the 16 `docs/**/index.html` pages carries the trademark line. The installation page carries `Based on Arch Linux ARM`, `Fedora Remix`, `https://fedoraproject.org/`, `not provided or supported by the Fedora Project` and `terminate-user live`. `docs/user/index.html`, `download/` and `live-session/` are gone, no page links them, and the sidebar reads System requirements, Installation, Wallpapers.
- `git grep -niP 'based on (fedora|arch(?! linux))' -- release site docs AGENTS.md README.md` prints nothing.
- `just lefthook all` passes, and gitleaks finds no leaks in the new untracked files.
- The operator asked to move the trademark line to the bottom of the left nav. Fedora's guidelines word it as the page footer, and on phones the nav sits behind the menu button, so the operator kept it in the page footer, at 12 px (`--sl-text-2xs`) in `--sl-color-gray-3`. That is the faintest theme colour with 4.5:1 contrast in both themes; gray-4 is about 2.5:1 on the light theme.

## Round 2
Reviewed: the whole uncommitted change again, with round 1's fix and this record. It also settled round 1's open point: `justfile_directory()` resolves to the repo root inside the `release` module, which `upload` and the Serena module's `mcp` already rely on.
Verdict: Looks good. Ship.
