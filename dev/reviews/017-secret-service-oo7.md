# 017: A Secret Service in every session: oo7, unlocked at login: review

Task: dev/tasks/017-secret-service-oo7.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (oo7 in `stackific-desktop` on both bases, Arch's greetd PAM service and greetd config, the live and installed-system checks, the docs), with what the first test runs showed.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | On Arch (oo7 0.6.0 at the archive date), a session with no password (the live user) leaves the keyring locked; Noctalia asks for a prompt nothing can show, then hangs on SIGTERM, so logout, `runuser` for that user and shutdown wait systemd's 90 s (`just arch test` failed at the Plasma check) | `desktop/systemd/user/stackific-noctalia.service` | fixed | `TimeoutStopSec=5s` on our Noctalia unit, with the reason in a comment; Fedora's oo7 0.7 does not hang, and logins with a password unlock the keyring, so no prompt is pending there |
| 2 | Should fix | The keyring check expected `~/.local/share/keyrings/*.keyring`; Fedora's oo7 0.7 keeps it under `keyrings/v1/` | `tests/vm/installed.sh` (`secrets`) | fixed | `find … -name "*.keyring"`; `docs/arch-iso.md` names both paths |

Tests after the fixes: `just arch packages`, `just arch iso` (1552 MiB), `just arch test` and `just arch test-install` pass; `just fedora packages`, `just fedora iso` (1497 MiB), `just fedora test` and `just fedora test-install` pass.

## Round 2
Reviewed: the whole change after round 1's fixes.
Verdict: Looks good. Ship.
