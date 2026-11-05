# 004: Arch release package: the image identifies as Stackific OS: review

Task: dev/tasks/004-arch-release-package.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `branding/os-release`, `arch/packages/stackific-release/PKGBUILD`, `arch/justfile`, `arch/iso/packages.x86_64`, `tests/vm/boot.sh`, `docs/arch-iso.md`, `AGENTS.md` and the prompt-trail entry.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Nice to have | `package()` used `install -d`, a redirect and `chmod 644` where `install -Dm644` plus an append does the same | `arch/packages/stackific-release/PKGBUILD:22-24` | fixed | `install -Dm644 os-release` then `printf` appends `ID_LIKE=arch` and `BUILD_ID=rolling` |

Tests after the fixes: `just arch packages` exits 0. `bsdtar -tvf` lists only `etc/os-release` (356 bytes, 0644) and the icon (186 bytes, 0644) besides metadata. The `etc/os-release` diff against `branding/os-release` plus the two lines and the icon `cmp` print nothing. `just shellcheck lint` passes on the PKGBUILD and `tests/vm/boot.sh`. The package's files are byte-identical with the same modes, so the `just arch iso` (1497 MiB) and `just arch test` passes from before the fix stand.

## Round 2
Reviewed: the whole uncommitted change again after the round 1 fix, docs and entry included.
Verdict: Looks good. Ship.
