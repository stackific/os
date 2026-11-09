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
