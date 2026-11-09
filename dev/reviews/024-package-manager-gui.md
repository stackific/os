# 024: A package manager GUI on every image: Shelly on Arch Linux, GNOME Software on Fedora: review

Task: dev/tasks/024-package-manager-gui.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (Shelly's PKGBUILD and side files, the builder's makedepends, the architecture skip in `just arch packages`, the package lists, `stackific-desktop`'s polkit rule, the boot checks, docs and entry).
Verdict: Ship.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Nice to have | The user page said every change goes through for a few minutes after the password, but trusting a new repository's key (`Repo.confirm_key`, `auth_admin`) asks every time | `site/src/content/docs/user/software.md` | fixed | "further installs and updates for a few minutes … Trusting a new software repository asks for the password every time" |

Tests after the fix: `just lefthook all` passes (15 checks, `just site format` first); shellcheck on `tests/vm/arch-x86_64.sh`, `tests/vm/fedora.sh` and Shelly's PKGBUILD passes; Shelly's makepkg run (with `check()`) in the changed x86_64 builder built `shelly-3.2.0-1-x86_64.pkg.tar.zst` (3.4 MiB) and `shelly-flatpak-backend-3.2.0-1-x86_64.pkg.tar.zst` (0.4 MiB), with only an action file under `usr/share/polkit-1/`; `stackific-desktop` 0.2.0-5 built with rpmbuild and holds `/etc/polkit-1/rules.d/10-stackific-dnf5daemon.rules` as a config file; the rule, run in basename order before dnf5daemon's in a node harness, answers `auth_admin_keep` (active) or `auth_admin` for installs and `auth_admin` for repository keys; the architecture skip builds all seven packages for x86_64 and skips only `shelly` for aarch64.

## Round 2
Reviewed: the round 1 fix and the whole change again.
Verdict: Looks good. Ship. (Checked: the skip sources each PKGBUILD in a subshell, as the loop's source list already does; `pkcheck` exits 2 for both "ask once" and "ask always", and 0 only if some rule allowed without a password; the rule's name sorts before `org.rpm.dnf.v0.rules`.)
