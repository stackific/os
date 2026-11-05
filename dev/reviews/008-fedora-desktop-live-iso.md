# 008: Fedora 45 x86_64 live ISO boots to MangoWM with Noctalia, with GNOME: review

Task: dev/tasks/008-fedora-desktop-live-iso.md
Status: commit ready

## Build notes
Measured while building, before the first review round:
- The first `just fedora test` failed at `systemctl --user -M live@ is-active stackific-noctalia.service`: `Failed to start transient service unit: Connection reset by peer`. The session itself was up: `mango` and `noctalia` ran as `live`, and the unit was `active` through `runuser`. In the VM, `systemd-run -PGq --wait -pUser=live -- id` fails the same way, `systemctl --user -M live@` prints `active` after `setenforce 0`, and `semodule -DB` shows the denials: `init_t` `ioctl` and `system_dbusd_t` `read write` on the test shell's `unconfined_service_t` `vsock_socket`. The check now uses `runuser`; the task file is updated.
- `rpm -qf /etc/os-release` names the owner of the link, `generic-release-common` (in the pinned base it prints `fedora-release-common`). The check names both files; the task file is updated.
- The manifest has `openssh` and `openssh-clients`: `noctalia` requires `git-core`, which requires `openssh-clients`, which requires `openssh`. There is no `openssh-server`. The must-not now names the SSH server; the task file is updated.
- The wrong-key build, with Terra 44's key (sha256 `77e2f7e3…`, fingerprint `AE09157A4DE88B497EA1D5D300CDAB43DE226D6F`) in `fedora/iso/terra-45.asc`, stops in kiwi's bootstrap phase:

  ```text
  >>> repomd.xml GPG signature verification error: Signing key not found
  Failed to download metadata (baseurl: "https://repos.fyralabs.com/terra45") for repository "terra": repomd.xml GPG signature verification error: Signing key not found
  error: recipe `iso` failed with exit code 1
  ```

  Terra 45's key was restored after it: sha256 `411e0c66b22a3f3d79651c05b13c0dbcd9edd72f6839e5e33d1fd5e1c6248f7b`, fingerprint `C2AC02124AF114F086592E3C8DDE7D14C1C23D8C`.
- Manual GNOME steps, run in the boot-test VM with keystrokes from QEMU's monitor: step 3 fails as first written. With the password field empty, noctalia-greeter says `Enter a password` and sends nothing. With one character typed, greetd logs `live` in (`pam_unix(greetd:auth): user [live] has blank password; authenticated without it`) and GNOME starts, with `stackific-noctalia.service` `inactive`. Step 3 now says to type any character; the task file is updated. The operator decides whether that is acceptable, or whether `live` gets a password, which would change the AGENTS.md live-session line on all images. Arch's noctalia-greeter is the same 1.7.0, so Arch's manual Plasma step likely has the same gap (not checked here).
- `/var/lib/noctalia-greeter` is `0750 greetd greetd`, the same line as Arch's own `/usr/lib/tmpfiles.d/noctalia-greeter.conf` (`d /var/lib/noctalia-greeter 0750 greeter greeter -`), whose comment says to override it under `/etc/tmpfiles.d/` when the greeter user differs.
- `-m 4096` stays: the VM has 3011 MiB available with the GNOME session running.

## Round 1
Reviewed: the task's uncommitted change: `fedora/iso/` (`config.kiwi`, `config.sh`, `terra-repo.sh`, `terra-45.asc`, `root/etc/greetd/config.toml`, `root/etc/tmpfiles.d/noctalia-greeter.conf`, the deleted getty autologin files), `fedora/justfile`, `tests/vm/fedora-x86_64.sh`, `docs/fedora-iso.md`, `AGENTS.md`, the task file and the prompt-trail entry.
Verdict: fix 1, 2, 3 and 4 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The Fedora test-shell gotcha (`systemctl --user -M` fails under SELinux) is not in AGENTS.md, so task 010 would copy the Arch check and hit it again | `AGENTS.md`, Tests | fixed | One line under Tests: the cause, and that checks use `runuser` |
| 2 | Should fix | The docs say the work tmpfs holds about 2 GiB, measured for the console ISO; the desktop image is about twice as large | `docs/fedora-iso.md:24` | fixed | Sampled `df -m /work` every 5 s during a rebuild: peak 5398 MiB. The docs give that, the default cap (half the host's RAM, so about 12 GiB of RAM) and the `size=` remedy. The build host has 31 GiB, so no `size=` is set |
| 3 | Should fix | The docs give an unchecked reason for the greeter listing GNOME first | `docs/fedora-iso.md`, Live session | fixed | The reason is dropped; the docs state the order seen on each base |
| 4 | Should fix | The entry says the manual steps ran in the VM, but step 2 (`Alt+Return`, foot) and step 4 (Ptyxis) were not run as written | prompt-trail entry, Manual changes | fixed | All four steps ran by keystrokes through QEMU's monitor, with a screenshot after each. The entry names what ran, and its GNOME screenshot is now Ptyxis printing `inactive` |

Tests after the fixes:
- `just fedora iso` exits 0 in 5 min 8 s: `1488 MiB (target 2048 MiB)`, 677 packages. The manifest lists `stackific-release`, `stackific-desktop`, `stackific-artwork` (0.1.0, 0.2.0, 0.1.0, `-1.fc45`), `mangowm 0.18.5-1.fc45`, `noctalia 5.4.1-1.fc45`, `noctalia-greeter 1.7.0-2.fc45`, `greetd` and `greetd-selinux 0.10.3-12.fc45`, `gnome-shell 51.1-1.fc45`, `generic-logos 18.0.0-29.fc45` and `NetworkManager-wifi 1.58.1-3.fc45`. It has no `fedora-release*`, `fedora-logos`, `fedora-release-notes`, `generic-release`, `cardwire`, `openssh-server`, `cloud-init` or VM guest agent. `scenefx` and `switcheroo-control` come from Fedora. `terra-45.asc` has sha256 `411e0c66…`.
- `just fedora test` exits 0 in 45 s: `PASS`, every check `ok`, including `0` AVC denials. journald reads the audit records itself (`systemd-journald-audit.socket` active), so the kernel's printk rate limit hides none.
- Manual steps by keystrokes in the VM: foot opens on `Alt+Return`; the greeter shows after `loginctl terminate-user live` typed into foot; one typed character logs `live` into GNOME; Ptyxis prints `inactive` for `stackific-noctalia.service`, and no `noctalia` process runs. `ps -eZ` shows greetd and noctalia-greeter in `xdm_t`, and `mango` in `unconfined_t`. GNOME shows `Screen Lock disabled: Screen Locking requires the GNOME display manager`.
- `screen.png` shows Noctalia's bar on MangoWM over Vesuvius, with no wizard. `greeter.png` shows noctalia-greeter over Vesuvius in our palette, with our emerald logo at the bottom centre, as on Arch.
- `shellcheck` on `fedora/iso/terra-repo.sh`, `fedora/iso/config.sh` and `tests/vm/fedora-x86_64.sh`, and `just lefthook justfiles`, pass.

## Round 2
Reviewed: the whole change again after the round 1 fixes, docs and entry included.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The docs call greetd's config the live image's only change to a package's file, but `config.sh` also changes `setup`'s `/etc/passwd`, `/etc/group` and `/etc/shadow` | `docs/fedora-iso.md`, Live session | fixed | "only change to a desktop package's file", matching the task's must-not |
| 2 | Should fix | The Fedora 46 key steps would leave AGENTS.md naming the deleted `terra-45.asc` | `docs/fedora-iso.md`, Repositories, step 2 | fixed | Step 2 names `AGENTS.md` with `config.kiwi` and `terra-repo.sh` |

Tests after the fixes: the fixes touch only `docs/fedora-iso.md`, so the image and its test stay as tested in round 1 (`just fedora iso` 1488 MiB, `just fedora test` `PASS`). The static checks rerun under Round 3.

## Round 3
Reviewed: the whole change after the round 2 fixes: code, tests, docs, AGENTS.md, the task file and the entry, against the build and test logs and the screenshots; the callers of `just fedora iso` (the `images` and `release` workflows), which now build the packages first through the recipe's dependency.
Verdict: Looks good. Ship.

Tests:
- `just lefthook all` passes all 12 checks (dev, hygiene, justfiles, secrets, shell, site-audit, site-build, site-format, site-lint, site-screenshots, site-typecheck, workflows).
- On the untracked files directly: `just lefthook hygiene` on every new and changed file, `just shellcheck lint` on `terra-repo.sh`, `config.sh`, `fedora-x86_64.sh` and `boot.sh`, `just lefthook justfiles`, `just dev check`, gitleaks `dir` on a copy of every new and changed text file (no leaks), and `just site format-check`, `lint`, `typecheck`, `screenshots-check` and `build` with the new entry and its three WebPs.
- `git grep -nE 'NOPASSWD|polkit' fedora tests desktop` and `grep -rnE` on the same folders (untracked files included) print nothing.
- `just doctor` prints `fedora: ok (podman 6.1.4, /dev/kvm and /dev/vhost-vsock read-write)`. It exits 1 in this worktree only because the worktree's own `.tools/` has no uv, Go or Serena and its Claude plugins are not installed, which this task does not touch.
