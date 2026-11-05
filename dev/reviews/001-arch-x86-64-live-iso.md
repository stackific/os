# 001: Arch x86_64 console live ISO with a QEMU boot test: review

Task: dev/tasks/001-arch-x86-64-live-iso.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change, including `arch/`, `tests/vm/`, the root `justfile`, `.gitignore`, `README.md`, `AGENTS.md`, the task file, the docs page and the prompt-trail entry.
Verdict: fix 1, 2 and 3 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The seccomp comment names a READY listener that no longer exists | `arch/justfile:31` | fixed | The comment now names only the test shell client |
| 2 | Should fix | If the token credential is missing, an empty first line opens the root test shell | `tests/vm/stackific-test-shell@.service:12` | fixed | The service also requires a non-empty line (`[ -n "$$t" ] &&`), and the unit comment says why |
| 3 | Should fix | `version` is pasted into host and container shell code unchecked, so a quote in it runs commands, and a non-semver name is accepted | `arch/justfile:15,23,35` | fixed | `iso` and `test` take `version` as `$1` (`[positional-arguments]`). `iso` refuses a version that is not semver and passes the file name into the container as `NAME` in the environment. The docs page states the semver rule |

Tests after the fixes:
- `just arch iso abc` and `just arch iso '1.0.0"x'` both print `version must be semver, got '…'` and exit 1.
- `just arch iso` exits 0, 898 MiB.
- `just arch test` passes 4 of 4 runs, with every check `ok`.
- `just lefthook justfiles` and `just shellcheck lint tests/vm/boot.sh arch/iso/profiledef.sh` pass.

## Round 2
Reviewed: the whole uncommitted change again, after the round 1 fixes.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The README calls podman needed only for images, but `just doctor` (run by `just setup`) fails on any Linux host without it. It also claims every systemd host makes `/dev/kvm` 0666, and Ubuntu makes it 0660 for the `kvm` group | `README.md:13` | fixed | The line now says Linux needs rootless podman, that Arch and Fedora make both devices 0666, and that other hosts add the user to the `kvm` group; doctor fails without them |

Tests after the fixes: README only. `just lefthook hygiene README.md` passes, and `just doctor` still prints `arch: ok (podman 6.1.4, /dev/kvm and /dev/vhost-vsock read-write)`.

## Round 3
Reviewed: the whole uncommitted change again, after the round 2 fix.
Verdict: Looks good. Ship.
