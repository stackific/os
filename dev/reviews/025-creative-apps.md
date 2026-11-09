# 025: The creative apps as a one-click install from upstream's signed releases: review

Task: dev/tasks/025-creative-apps.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (`apps/`, both package recipes, the staging in `just arch packages` and `just fedora packages`, the image lists, `just release changed`, lefthook, the boot and install checks, docs and entry).
Verdict: Ship after 1 (nice to have).

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Nice to have | The test's throwaway GNUPGHOME starts a gpg-agent that keeps running after the recipe deletes its folder, one stray process per run, in CI too | `apps/justfile` `test` | fixed | The EXIT trap runs `gpgconf --kill gpg-agent` before removing the folder |

Tests after the fixes: `just apps test` prints three `ok:` lines and leaves no gpg-agent; shellcheck passes on `apps/install-creative-apps`, `tests/vm/boot.sh`, `tests/vm/installed.sh` and the PKGBUILD; `just lefthook all` passes (16 checks).

## Round 2
Reviewed: the round 1 fix and the whole change again.
Verdict: Looks good. Ship. (Checked: no package is installed unless every signature and every pinned hash passed; run0 passes no environment, so the test's URL and share overrides cannot redirect a real run; signatures never lie next to a package pacman reads; the pins file keeps upstream's lines byte for byte, 13 per base and architecture.)

## Round 3
Reviewed: CI, `images` run 42549967351 (arch-aarch64 `just arch packages`).
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | makepkg takes every `.asc` in `source=()` for a detached signature, so upstream's key `creative-apps-key.asc` made it look for `creative-apps-key` to verify: `SOURCE FILE NOT FOUND`, `One or more PGP signatures could not be verified!` | `arch/packages/stackific-apps/PKGBUILD` | fixed | The key is `apps/creative-apps-key.pgp` everywhere (PKGBUILD, spec, script, test, docs); new sha256sums for the two changed files |

Tests after the fix: `just apps test` passes its 3 cases; `makepkg --verifysource` and `makepkg --nodeps` in the Arch builder pass; the four images rerun in the next `images` run.
