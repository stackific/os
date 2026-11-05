# 002: Arch x86_64 live ISO boots to MangoWM with Noctalia, with KDE Plasma: review

Task: dev/tasks/002-arch-desktop-live-iso.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `arch/` (Containerfile, vars, justfile, the profile and its airootfs files, the new PKGBUILD), `desktop/`, `tests/vm/boot.sh`, `README.md`, `AGENTS.md`, the task file, the docs page, the prompt-trail entry and its screenshots. One extra VM run confirmed `live` is in `wheel`, `/home/live` is `live 750`, and no `.pacnew` file is left in `/etc`.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Nice to have | `makepkg --cleanbuild` is dead: every package builds in a new directory in a new `--rm` container | `arch/justfile:27` | fixed | Dropped the flag; the task's Changes line now says the directory is new |
| 2 | Nice to have | `ls` of the repo repeats makepkg's own "Finished making" line in every build | `arch/justfile:31` | fixed | Deleted the line |

Tests after the fixes: `just arch packages` exits 0, and `bsdtar -tf` lists the unit, its `mango-session.target.wants` link and `etc/profile.d/stackific-wlroots.sh`. `just lefthook justfiles` passes. Neither fix changes the ISO's content, so the three `just arch test` passes on the 1497 MiB ISO stand.

## Round 2
Reviewed: the whole uncommitted change again after the round 1 fixes, docs and entry included.
Verdict: Looks good. Ship.
