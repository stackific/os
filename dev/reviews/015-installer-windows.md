# 015: Install next to Windows: BitLocker checked and confirmed, Windows in Limine's menu, refusals with guidance: review

Task: dev/tasks/015-installer-windows.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: the installer (BitLocker, Windows detection, RST and BIOS-mode refusals, Windows entries, the BitLocker screen), the packages, the VM tests and recipes, and the docs; the prompt-trail entry was not written yet.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The fake-Windows boot test polls the serial log once a second and sleeps a second more, so its keys can reach Limine's menu after its 3 s countdown, and the test fails at random | `tests/vm/installed.sh` (`windows_boot`) | fixed | Polls every 0.2 s; the first key stops the countdown |
| 2 | Nice to have | The user page says to turn BitLocker back on, which `-RebootCount 1` already does once Windows starts | `site/src/content/docs/user/windows.md` (Starting Windows) | fixed | Says it resumes by itself; the command only for a suspension without a count |
| 3 | Must fix | Found by the screenshot run after the review: a file system name wider than its column ("BitLocker") runs into the label ("BitLockerDESKTOP-NPM7RCA") on Space and Confirm | `installer/src/ui/space.rs` (`part_row`) | fixed | The row keeps a space after the file system name; a test in `installer/src/ui/tests.rs` |

Tests after the fixes: `just installer test` (66) and `just installer lint` pass; `just shellcheck lint` on the changed scripts passes; rebuilt both images: `just arch iso` (1545 MiB), `just arch test-install`, `just arch screenshots`, `just fedora iso` (1497 MiB), `just fedora test`, `just fedora test-install` all pass (`just arch test` passed on the first build).

## Round 2
Reviewed: the whole change after round 1's fixes and the `part_row` fix: the installer, packages, VM tests and recipes, docs, and the prompt-trail entry.
Verdict: Looks good. Ship.
