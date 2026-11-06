# 010: Fedora 45 aarch64 live ISO boots to MangoWM with Noctalia, with GNOME: review

Task: dev/tasks/010-fedora-aarch64-desktop-live-iso.md
Status: commit ready

## Build notes
Measured while building in the Mac's runner VM, before the first review round:
- The first `just fedora iso` stopped when kiwi copied `fedora/iso/root/` into the image: `rsync_xal_set: fsetxattr("/work/build/image-root/.","security.selinux") failed: Operation not supported (95)`. podman mounts its `--tmpfs` with one fixed `context=` on an SELinux host, even with `--privileged` and `label=disable`; a tmpfs that container root mounts itself takes labels (`chcon` to `etc_t` and `xdm_exec_t` works). The recipe now mounts it in the container; the task file is updated.
- The next build stopped in `setfiles`: `Could not set context for /usr/share/accountsservice: Invalid argument`. The VM ran Fedora 44 (`selinux-policy-targeted` 43.3), and a rootless build may set only contexts the host's policy knows. The operator chose Fedora 45 for the VM. It was upgraded in place (`dnf system-upgrade` to 45, `dnf offline reboot`) and now runs `selinux-policy-targeted` 45.23, the image's version. As chore work, `just runner vm` now pins Fedora 45's Cloud image by digest, `just setup` makes the VM on a Mac, and `just doctor` checks its release. The task file is updated.
- The next build stopped in kiwi's EFI step: `fsetxattr("/var/tmp/efi_folder…/.","security.selinux") failed: Operation not supported (95)`. kiwi's temp dir defaults to `/var/tmp`, on podman's overlay. kiwi now gets `--temp-dir /work/tmp` on the same tmpfs; the task file is updated.
- The next build ran out of room on the tmpfs, capped at 5959 MiB in the 12 GiB VM: xorriso said `Image size 733160s exceeds free space on media 323398s`. Per the task, the build stopped and the operator chose to give the VM 16 GiB (`limactl edit --memory 16`; `just runner vm` creates new VMs with 16 GiB). The task file is updated.
- In the 16 GiB VM, `just fedora iso` exits 0: `1431 MiB (target 2048 MiB)`, 673 packages, a tmpfs peak of 6763 MiB of 7971 MiB (`df` every 5 s in the build container).
- The task's criterion `grep 'arch=' fedora/iso/config.kiwi` also matched the two existing metalink lines (`&amp;arch=$basearch`). It now greps `<package .*arch=`; the task file is updated.

## Round 1
Reviewed: the task's uncommitted change: `fedora/iso/config.kiwi`, `fedora/Containerfile`, `fedora/vars.just`, `fedora/justfile`, `tests/vm/fedora.sh` (renamed from `tests/vm/fedora-x86_64.sh`), the `tests/vm/boot.sh` comment, both workflows, `docs/fedora-iso.md`, `docs/releases.md`, `AGENTS.md`, the task file and the prompt-trail entry. Also the chore on the same branch: `just/runner/vars.just`, `just/runner/mod.just`, `justfile` (`setup` runs `runner::vm`, `doctor` checks `runner`) and `README.md`. Read with the callers: `release/justfile` `upload`, `arch/justfile`, the workflows' other jobs, `boot.sh`.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | A new Mac's VM download has one location, `download.fedoraproject.org`, whose redirector picked a dead mirror during the review (`TLS handshake timeout` on `mirror.cpsc.ucalgary.ca`), so `just setup` on a new Mac can fail at the VM step | `just/runner/mod.just:93`, `just/runner/vars.just` | fixed | `runner_vm_image` is now the path under `/pub/`; `just runner vm` lists it on the redirector, then on `dl.fedoraproject.org`, both with the pinned digest. Lima tries image entries in order |

Tests after the fixes:
- A throwaway `limactl create` with the two entries: the redirector failed again, and Lima downloaded the image from `dl.fedoraproject.org` and checked the digest (`Downloaded the image from https://dl.fedoraproject.org/…`). The instance was deleted.
- `just --list runner` parses; `just runner vm` on the running VM exits 0 (`already running`, packages `already installed`); `just runner check` prints `runner vm: ok (stackific-runner, Fedora 45)`.
- The image and its test are unchanged by the fix: `just fedora iso` 1431 MiB, `just fedora test` `PASS` (below, in the entry's Result).
- `just dev check`, shellcheck on `tests/vm/fedora.sh` and `tests/vm/boot.sh`, actionlint on both workflows, `just --list --list-submodules`, `just site format-check`, `screenshots-check` and `build`, `git diff --check`, gitleaks `dir` (no leaks), and `git grep -nE 'NOPASSWD|polkit' fedora tests desktop` (nothing) pass.

## Round 2
Reviewed: the whole change again after the round 1 fix, docs, task file and entry included.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | README's in-place upgrade step says to run `just runner vm` after the VM stops, but the VM does not stop: after `dnf offline reboot` it applied the upgrade and came back on its own (`uptime` 0 min, `limactl list` Running) | `README.md`, "Mac (aarch64 images)" | fixed | It says the VM boots into Fedora 45 by itself, runner service included, and that `just runner check` then prints `runner vm: ok (stackific-runner, Fedora 45)` |
| 2 | Should fix | The entry's Prompt gives the tmpfs peak as near 6.8 GiB, the estimate from the failed run; the measured peak is 6763 MiB, as its Outcome says | prompt-trail entry, Prompt | fixed | The Prompt gives 6763 MiB; the task's estimate line now shows its sum (5327 MiB used plus 1432 MiB) |

Tests after the fixes: the fixes touch only `README.md`, the entry and the task file. `just site format-check` passes, and no changed file names 6.8 GiB any more.

## Round 3
Reviewed: the whole change after the round 2 fixes: code, workflows, docs (`docs/fedora-iso.md` in full), `AGENTS.md`, `README.md`, the task file and the entry, against the build and test logs, the manifest and both screenshots.
Verdict: Looks good. Ship.
Not checked: the x86_64 build and test on the PC with the self-mounted tmpfs and `--temp-dir` (the task's manual step 1), and the `images` run after the merge.
