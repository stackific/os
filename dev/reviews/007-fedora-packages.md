# 007: Fedora RPMs of our packages, built with mock into a local repo: review

Task: dev/tasks/007-fedora-packages.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `fedora/Containerfile`, `fedora/vars.just`, the `packages` recipe in `fedora/justfile`, the three new specs in `fedora/packages/`, `docs/fedora-iso.md` and `AGENTS.md`.
Verdict: Looks good. Ship.

Checked:
- the recipe's source staging against `just arch packages`: a missing source makes mock fail, and the repo receives only `.noarch.rpm` files;
- the specs' render lines against task 006's PKGBUILDs;
- the release package's provides against what `fedora-repos`, `generic-release-common` and dnf5 need;
- that rpm expands no `%` in spec comments;
- the docs' claim about presets, against `redhat-systemd-presets-common` 102-3.fc45: it enables `NetworkManager.service`, `pipewire.socket` and `wireplumber.service`.

Tests:
- `just fedora packages` exits 0:
  - 295 s on the first run, which rebuilt the builder with mock;
  - 169 s on a rerun with the builder cached, about 2 minutes of it the first mock call's bootstrap.
- `.build/fedora/repo/` holds `stackific-release-0.1.0-1.fc45.noarch.rpm`, `stackific-desktop-0.2.0-1.fc45.noarch.rpm`, `stackific-artwork-0.1.0-1.fc45.noarch.rpm` and `repodata/repomd.xml`, and no `.src.rpm` or logs.
- Compared with `just arch packages` in the same tree, with directory entries filtered out:
  - `stackific-desktop` and `stackific-artwork` list the same file paths as the Arch packages;
  - `stackific-release` differs only in `usr/lib/os-release` against Arch's `etc/os-release`.
- Rendered files against the Arch packages:
  - os-release's first ten lines are identical, followed by `ID_LIKE=fedora`, `VERSION_ID=45` and `SUPPORT_END=2027-12-08`;
  - the icon has sha256 `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`, the same as the Arch icon;
  - `/etc/xdg/fastfetch/config.jsonc` is byte-identical, with `"1": "#007a55"`.
- `rpm -qp` on `stackific-release`:
  - provides `generic-release-variant = 45-0.4`, `system-release`, `system-release(45)` and `system-release(releasever) = 45`;
  - requires `distribution-systemd-presets`, `generic-release-common` and `hicolor-icon-theme`;
  - conflicts with `generic-release`.
- `rpm -qp` on the other two:
  - `stackific-desktop`'s config files are `/etc/profile.d/stackific-wlroots.sh` and `/etc/skel/.config/noctalia/00-stackific.toml`;
  - `stackific-desktop` requires `mangowm`, `noctalia`, `stackific-artwork` and `systemd`;
  - `stackific-artwork`'s config file is `/etc/xdg/fastfetch/config.jsonc`, it requires `fastfetch` (2.68.1-1.fc45 in Fedora 45), and its license tag is `Apache-2.0 AND CC0-1.0 AND LicenseRef-PublicDomain`;
  - `wallpapers.toml` is `stackific-artwork`'s license file.
- Install probe in the pinned base (`fedora:45@sha256:10989b31…`), with `dnf -y install --allowerasing --repofrompath=stackific,/repo --setopt=stackific.gpgcheck=0 stackific-release`:
  - dnf installs 3 packages (`hicolor-icon-theme`, `stackific-release`, `generic-release-common`) and removes 3 (`fedora-release-common`, `fedora-release-identity-container`, `fedora-release-container`);
  - `. /etc/os-release` prints `stackific/fedora/45/Stackific OS`, and `/etc/os-release` links to `../usr/lib/os-release`;
  - `generic-release` and `fedora-release-common` are not installed; `generic-release-common-45-0.4` and `redhat-systemd-presets-102-3.fc45` are;
  - `dnf5 --dump-variables` prints `releasever = 45`, `dnf repoquery kernel` lists `kernel-0:7.2.15-300.fc45.x86_64`, and `rpm -E %fedora` prints `45`;
  - `/etc/system-release` reads `Generic release 45 (Generic)`, as the task expects.
- `just lefthook all` passes on all 12 checks, after `just site format` added two blank lines to the prompt-trail entry. Run directly on the untracked files, the hygiene, justfile parse and `just dev check` recipes pass too. `just doctor` prints `fedora: ok`.

## Round 2
Source: the operator (2026-11-05), after task 006 grew during its PR and merged as #6: mirror 006's greeter files and logo in the Fedora `stackific-desktop`, and show fastfetch in a screenshot. The screenshot found a 006 bug, fixed here in its own commit.
Reviewed: `stackific-desktop.spec` (the greeter config, its tmpfiles line, `stackific-greeter.sh`, the asset folder's two links, `Requires: noctalia-greeter stackific-release`, `BuildRequires: systemd-rpm-macros`); `branding/fastfetch.jsonc.in` with fastfetch's default modules, the Arch `stackific-artwork` at `pkgrel=2`, and the `OS: Stackific OS` checks in `tests/vm/arch-x86_64.sh` and `tests/vm/arch-aarch64.sh`; the task file, `docs/fedora-iso.md`, `AGENTS.md` and the entry.
Verdict: Looks good. Ship.

Checked: a JSON config makes fastfetch print only its own `modules` (`src/fastfetch.c` lines 780-790), and `DEFAULT_STRUCTURE` is the same list in 2.68.1 and 2.70.0; the Arch package's content changed after #6 merged, so its `pkgrel` moves to 2, while the Fedora RPMs are unreleased and stay at release 1; Terra's `noctalia-greeter` installs `/usr/share/noctalia-greeter/assets/fonts`, the links' target; no spec comment carries a `%`.

Tests:
- `just fedora packages` and `just arch packages` exit 0. With directory entries filtered out, `stackific-desktop` and `stackific-artwork` list the same paths and link targets on both bases; `stackific-release` differs only in `usr/lib/os-release` against `etc/os-release`.
- `greeter.toml`, `config.jsonc` and the icon are byte-identical between the RPMs and the Arch packages.
- `rpm -qp --configfiles` on `stackific-desktop` lists `/etc/profile.d/stackific-greeter.sh`, `/etc/profile.d/stackific-wlroots.sh` and `/etc/skel/.config/noctalia/00-stackific.toml`; `--requires` lists `mangowm`, `noctalia`, `noctalia-greeter`, `stackific-artwork`, `stackific-release` and `systemd`.
- In the pinned Fedora 45 base, `dnf install --allowerasing stackific-release stackific-artwork` succeeds; os-release reads `stackific/fedora/45/Stackific OS`, `dnf repoquery kernel` finds `kernel 7.2.15`, and `fastfetch` prints our logo in `38;2;0;122;85` beside `OS: Stackific OS x86_64`. Before the fix it printed the logo alone.
- `just arch packages`, `just arch iso` and `just arch test` with the fix exit 0: 1538 MiB, `PASS`, with `ok` for `OS: Stackific OS`.
