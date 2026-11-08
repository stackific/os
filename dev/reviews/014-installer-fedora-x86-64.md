# 014: The installer on the Fedora x86_64 image, with Limine kept current through kernel-install and dracut: review

Task: dev/tasks/014-installer-fedora-x86-64.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `installer/src/base.rs`, `boot.rs`, `install.rs`, `copy.rs`, `system.rs`, `demo.rs`, `ui/tests.rs`; `fedora/packages/stackific-boot/`, `stackific-installer/`, `stackific-desktop/`; `fedora/justfile`; `fedora/iso/config.kiwi`, `terra-repo.sh`, `edit-boot.sh`; `tests/vm/install.sh`, `installed.sh`, `fedora.sh`, `fedora-x86_64.sh`; `release/justfile`; `.github/workflows/images.yml`, `release.yml`; the docs, `AGENTS.md` and the prompt-trail entry.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Nice to have | The GRUB packages the Fedora base removes are x86_64's names only, unmarked; on aarch64 (task 018) dnf5 would name packages that are not installed and GRUB would stay | `installer/src/base.rs:84` | fixed | A `shortcut:` comment names the limit and task 018 |

Tests after the fixes: `just installer test` (57 pass) and `just installer lint` pass; the fix is a comment, so the images were not rebuilt. The image runs before the review: `just fedora iso` (1497 MiB), `just fedora test`, `just fedora test-install` (`PASS` for both boots, 143 checks); `just arch iso` (1545 MiB), `just arch test`, `just arch test-install` (`PASS` for both boots, 128 checks); `just lefthook all` passes, with shellcheck run by hand on the new scripts and the site's format, screenshots and build checks on the new entry.

## Round 2
Reviewed: the round-1 fix in `installer/src/base.rs`, and the whole change again.
Verdict: Looks good. Ship.
