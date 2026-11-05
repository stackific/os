# 007: Fedora RPMs of our packages, built with mock into a local repo

- [ ] Done

Base: fedora

## Ask

```text title="Prompt" wrap
Build Fedora 45 RPMs of our three packages, stackific-release, stackific-desktop and stackific-artwork, from the same base-neutral files in desktop/ and branding/ that the Arch PKGBUILDs use. They must install the same files at the same paths, rendered from branding/branding.env with the same envsubst lines. Build them with mock in the pinned Fedora builder container, rootless like every other build. Add `just fedora packages`, which puts them into the local dnf repo .build/fedora/repo/ with createrepo_c metadata, unsigned until release signing. stackific-release replaces generic-release without patches and without fedora-release*: it provides system-release for Fedora 45, owns the os-release, and keeps what Fedora's release packages do for dnf, rpm and systemd presets. That includes task 006's greeter files: the greeter defaults rendered with the accent and their tmpfiles link, the profile.d script that sets NOCTALIA_GREETER_ASSETS_DIR, and the asset folder that links our icon and the greeter's fonts, so the Fedora login screen shows our logo too. The live ISO and its boot test do not change here; task 008 installs these packages into the Fedora desktop image.
```

## Goal
`just fedora packages` builds `stackific-release`, `stackific-desktop` and `stackific-artwork` as noarch RPMs with mock and indexes them in `.build/fedora/repo/`. The RPMs ship the same files at the same paths as the Arch packages from task 006, and `stackific-release` swaps cleanly for `generic-release` on Fedora 45.

## Context
Task 003 built the Fedora 45 x86_64 console live ISO with kiwi in the pinned builder (`fedora/Containerfile`, base pinned by digest in `fedora/vars.just`, recipes in `fedora/justfile`). Its `fedora/iso/config.kiwi` names `generic-release` among the bootstrap packages, so the image says `Generic`. Task 006 must merge first. It adds the branding file and the Arch artwork package, and changes the other two Arch packages (`dev/tasks/006-branding.md`). The files this task packages, all added or kept by task 006:
- `branding/branding.env`: shell `KEY="value"` lines `NAME`, `ID`, `HOME_URL`, `DOCUMENTATION_URL`, `SUPPORT_URL`, `BUG_REPORT_URL` and `ACCENT` (`#007a55`).
- `branding/os-release.in`: the ten base-neutral os-release lines with `${NAME}`, `${ID}`, the four `${…_URL}` and `${ANSI_COLOR}`, and no comment lines.
- `branding/logo.svg`: the mark, one `fill="currentColor"`.
- `branding/fastfetch.jsonc.in`: fastfetch's `logo` key, colour `"1": "${ACCENT}"`.
- `branding/logo.txt`, `branding/wallpapers.toml` and the ten `branding/wallpapers/*.jpg`.
- `desktop/systemd/user/stackific-noctalia.service`, `desktop/profile.d/stackific-wlroots.sh` (from task 002), `desktop/noctalia/stackific-noctalia.toml` and `desktop/noctalia/00-stackific.toml`.

The render rules, the same as task 006's PKGBUILDs:
- source `branding.env` with `set -a`;
- derive `ANSI_COLOR` with `printf -v ANSI_COLOR '38;2;%d;%d;%d' "0x${ACCENT:1:2}" "0x${ACCENT:3:2}" "0x${ACCENT:5:2}"`;
- render `os-release` with `envsubst '$NAME $ID $HOME_URL $DOCUMENTATION_URL $SUPPORT_URL $BUG_REPORT_URL $ANSI_COLOR'`;
- render `logo-color.svg` with `sed "s/currentColor/$ACCENT/" logo.svg`;
- render `fastfetch.jsonc` with `envsubst '$ACCENT'`.

The install paths, the same as the Arch packages after task 006:

| Package | File | Installed as |
| --- | --- | --- |
| `stackific-release` | rendered `os-release` | `/usr/lib/os-release` (Fedora; Arch uses `/etc/os-release`, see below) |
| `stackific-release` | rendered `logo-color.svg` | `/usr/share/icons/hicolor/scalable/apps/stackific.svg` |
| `stackific-desktop` | `stackific-noctalia.service` | `/usr/lib/systemd/user/`, plus the symlink `mango-session.target.wants/stackific-noctalia.service` → `../stackific-noctalia.service` beside it |
| `stackific-desktop` | `stackific-wlroots.sh` | `/etc/profile.d/stackific-wlroots.sh` (config) |
| `stackific-desktop` | `stackific-noctalia.toml` | `/usr/share/stackific/noctalia.toml` |
| `stackific-desktop` | `00-stackific.toml` | `/etc/skel/.config/noctalia/00-stackific.toml` (config) |
| `stackific-desktop` | rendered `stackific-greeter.toml` (from `desktop/noctalia/stackific-greeter.toml.in`, `envsubst '$ACCENT'`) | `/usr/share/stackific/greeter.toml` |
| `stackific-desktop` | `stackific-greeter.conf` (`desktop/tmpfiles.d/`) | `/usr/lib/tmpfiles.d/stackific-greeter.conf` |
| `stackific-desktop` | `stackific-greeter.sh` (`desktop/profile.d/`) | `/etc/profile.d/stackific-greeter.sh` (config) |
| `stackific-desktop` | two links | `/usr/share/stackific/greeter-assets/noctalia.svg` → `/usr/share/icons/hicolor/scalable/apps/stackific.svg`, `/usr/share/stackific/greeter-assets/fonts` → `/usr/share/noctalia-greeter/assets/fonts` |
| `stackific-artwork` | ten JPEGs | `/usr/share/backgrounds/stackific/` |
| `stackific-artwork` | `wallpapers.toml` | `/usr/share/licenses/stackific-artwork/wallpapers.toml` |
| `stackific-artwork` | `logo.txt` | `/usr/share/stackific/logo.txt` |
| `stackific-artwork` | rendered `fastfetch.jsonc` | `/etc/xdg/fastfetch/config.jsonc` (config) |

This task does not change the ISO (`fedora/iso/`), the boot test (`tests/vm/`) or the images workflow. Task 008 makes `just fedora iso` depend on `packages`, mounts `.build/fedora/repo/` read-only, and reads it as repo `stackific`. It installs the three packages and names `stackific-release` in place of `generic-release`. It also adds the repos that provide `mangowm` (Terra) and keeps `generic-logos` as `system-logos`.

Facts, checked 2026-11-05. "Measured" means run in the pinned builder image `localhost/stackific-fedora-builder:45` (Fedora 45, base digest in `fedora/vars.just`) or in that base image, under rootless podman on the operator's x86_64 PC.

mock in the builder:
- Fedora 45 has `mock` 6.9-1.fc45 and `mock-core-configs` 45.2-1.fc45 (configs `fedora-45-x86_64` and `fedora-45-aarch64`). mock requires `createrepo_c` (1.2.6-1.fc45, updates) and `systemd-container`. `rpmlint` 2.9.0-3.fc45 also exists. None of them is in the builder today. Measured (`dnf repoquery`).
- mock 6.9 builds as container root in `podman run --rm --privileged` with `--isolation=simple --no-bootstrap-image`. `mock -r fedora-45-x86_64 --buildsrpm --spec <spec> --sources <dir> --resultdir <dir>` then `mock -r fedora-45-x86_64 --rebuild <srpm> --resultdir <dir>` produced `<name>-<version>-1.fc45.noarch.rpm` next to the `.src.rpm` and its logs (`build.log`, `root.log`, `state.log`, `hw_info.log`, `installed_pkgs.log`). Measured with a probe spec.
  - `--no-bootstrap-image`: mock's default bootstrap pulls a container image, which needs podman inside the container.
  - `--isolation=simple`: mock's default, nspawn, needs systemd-nspawn in the container.
- The first mock call takes about 2 minutes: it installs dnf5 tooling into a bootstrap chroot, then the minimal buildroot. Later calls in the same container unpack mock's root cache and take about 20 seconds. Measured (`state.log`: 19:12:45 to 19:14:50 for both steps).
- `rpmbuild` warns `%source_date_epoch_from_changelog is set, but %changelog has no entries to take a date from` for a spec without `%changelog`. The build still succeeds. Measured.
- `gettext-envsubst` 1.0-4.fc45 is the package that provides `/usr/bin/envsubst`. In the probe, `BuildRequires: gettext-envsubst` installed it, and `envsubst '$X'` replaced `${X}` and left `$Y` alone. Measured.
- rpm expands `%` in spec scriptlets, so a shell `%d` is written `%%d` (a probe `printf '%%d'` reached the shell as `%d`). Measured.
- `bsdtar` on the host reads RPM payloads (`bsdtar -tf`, `bsdtar -xOf <rpm> ./<path>`). The builder has `rpm` but not `rpm2cpio`. Measured.
- just's built-in `arch()` returns `x86_64` or `aarch64`, the names in mock's configs. Source: https://just.systems/man/en/functions.html.

Fedora 45's release packages (measured: `dnf repoquery` and the payloads of `generic-release-45-0.4.noarch.rpm` and `generic-release-common-45-0.4.noarch.rpm`):
- `generic-release` 45-0.4 owns only `/usr/lib/os-release`: `NAME=Generic`, `ID=generic`, `ID_LIKE=fedora`, `VERSION_ID=45`, `LOGO=generic-logo-icon`, placeholder URLs, and no `PLATFORM_ID`.
  - Provides: `system-release`, `system-release(45)`, `generic-release-variant = 45-0.4`, `generic-release-identity = 45-0.4`.
  - Requires: `generic-release-common = 45-0.4` and `distribution-systemd-presets` (provided by `redhat-systemd-presets` 102-3.fc45).
- `generic-release-common` 45-0.4 requires `fedora-repos(45)` and `generic-release-variant = 45-0.4`, and conflicts with `fedora-release-common`. It owns:
  - the symlinks `/etc/os-release` → `../usr/lib/os-release`, `/etc/system-release` and `/etc/redhat-release` → `fedora-release`, `/etc/fedora-release` → `../usr/lib/fedora-release` (`Generic release 45 (Generic)`), and `/etc/issue`, `/etc/issue.net` and `/etc/system-release-cpe`;
  - `/usr/lib/rpm/macros.d/macros.dist` (`%fedora 45`, `%dist .fc45`);
  - `/usr/share/dnf5/libdnf.conf.d/20-defaults.conf` (`pkg_gpgcheck=True`, `allow_vendor_change=False`, `best=False`, `skip_if_unavailable=True`), and dnf's vendor and COPR files.
- `fedora-repos` 45 (the Fedora repo files and keys) requires `system-release(45)`.
- Fedora's own `/usr/lib/os-release` (`fedora-release-identity-container` 45-14) has `VERSION_ID=45` and `SUPPORT_END=2027-12-08`, and no `PLATFORM_ID`.
- dnf5 5.4.6 takes `$releasever` from the first package that provides `system-release(releasever)`, then `system-release`, and so on. A provide `system-release(releasever) = X` sets it to X; otherwise it is the providing package's own version. Source: `Vars::detect_releasevers` in https://github.com/rpm-software-management/dnf5/blob/5.4.6.0/libdnf5/conf/vars.cpp. Our package version is 0.1.0, so without that provide `$releasever` would be `0.1.0`, and every Fedora repo URL would break.
- Probe (measured in the pinned base image): a test `stackific-release` with this task's `Provides`, `Requires` and `Conflicts` installs with `dnf install --allowerasing`. dnf swaps out `fedora-release-common` and the container's two `fedora-release*` packages, and keeps `redhat-systemd-presets`. Then:
  - `/etc/os-release` is `generic-release-common`'s symlink to our `/usr/lib/os-release`;
  - `rpm -E %fedora` prints `45`, and `dnf5 --dump-variables` prints `releasever = 45`;
  - `dnf repoquery kernel` resolves `kernel 7.2.15-300.fc45`;
  - `/etc/system-release` still reads `Generic release 45 (Generic)`.
- So on Fedora `stackific-release` owns `/usr/lib/os-release`, not `/etc/os-release` as on Arch: `generic-release-common` owns the `/etc` path as a symlink. Replacing `generic-release-common` too would mean copying Fedora's rpm macros, dnf5 defaults (among them `pkg_gpgcheck=True`) and `/etc` symlinks, so it stays.

Package names the specs' `Requires` name (measured, `dnf repoquery`):
- Fedora 45: `noctalia` 5.4.1-1.fc45 (updates), `systemd` 262.1-1.fc45, `hicolor-icon-theme` 0.18-6.fc45 (owns `/usr/share/icons/hicolor/scalable/apps`), `fastfetch` 2.68.1-1.fc45.
- Terra (`https://repos.fyralabs.com/terra45`): `mangowm` 0.18.5-1.fc45, which ships `/usr/lib/systemd/user/mango-session.target`.
- `filesystem` 3.18-59.fc45 owns `/usr/share/backgrounds`, `/etc/xdg`, `/etc/skel`, `/usr/lib/systemd/user` and `/usr/share/licenses`.
- fastfetch 2.68.1 reads `/etc/xdg/fastfetch/config.jsonc` like 2.70.0 (`src/common/impl/FFPlatform_unix.c` lines 212-218 at tag 2.68.1).
- `gdm`, which `gnome-shell` requires, requires `system-logos`. `generic-logos` 18.0.0-29.fc45 provides it. That is task 008's to install; `stackific-artwork` does not provide `system-logos`.

## Inputs
None from earlier material. The design is task 006's, restated in Context. These design choices are made in this task:
- mock as container root in the existing builder, with `--isolation=simple --no-bootstrap-image`;
- one spec per package in `fedora/packages/<name>/<name>.spec`, matched to its `Source` files by name, as `just arch packages` does;
- `stackific-release` replaces only `generic-release`. It keeps `generic-release-common`, provides `system-release(releasever) = 45`, and owns `/usr/lib/os-release`.

The operator chose, in this session (2026-11-05):
- Fedora's per-base os-release lines are `ID_LIKE=fedora`, `VERSION_ID=45` and `SUPPORT_END=2027-12-08`, with no `BUILD_ID`. The base is versioned, not rolling, and these are the fields Fedora tools read.
- `/etc/system-release` may keep reading "Generic" for now, rather than owning `generic-release-common`'s rpm macros and dnf5 defaults.
- No rpmlint, and `generic-logos` keeps providing `system-logos`, so `stackific-artwork` does not.

Left out, for later tasks:
- The ISO and boot-test changes that install these packages (task 008).
- `/etc/system-release` still reads `Generic release 45 (Generic)`, from `generic-release-common`. Fixing it means owning that package's files too.
- Release signing of the RPMs and the repo metadata.
- Reproducible build times (`%changelog`, `SOURCE_DATE_EPOCH`).
- rpmlint.
- An aarch64 build on the Mac runner: the recipe picks mock's config with `arch()`, but nothing here runs it there. The RPMs are noarch.

## Changes
- `fedora/Containerfile`: add `mock` to the `dnf -y install` line (it pulls `createrepo_c`). Extend the header comment: mock builds our RPMs.
- `fedora/vars.just`: `repo := justfile_directory() / ".build" / "fedora" / "repo"`, with a comment.
- `fedora/justfile`: a new `packages` recipe, which depends on `builder`. It empties `{{ repo }}`, then runs one fresh `podman run --rm --privileged` container with:
  - `fedora/packages` at `/packages`, `desktop` at `/desktop` and `branding` at `/branding` (all read-only), and `{{ repo }}` at `/repo`.

  For each `/packages/*/*.spec`, the container makes a new directory and copies in the spec and every `Source<N>:` file. It finds them by name under `/desktop` and `/branding`, like `just arch packages`. It runs `mock -r fedora-45-{{ arch() }} --isolation=simple --no-bootstrap-image` twice: `--buildsrpm --spec … --sources …`, then `--rebuild` of the SRPM, each with its own `--resultdir`. It copies the `*.noarch.rpm` to `/repo`. Last, `createrepo_c /repo`. Update the module's header comment.
- `fedora/packages/stackific-release/stackific-release.spec` (new). A header comment says why it replaces only `generic-release` and why each provide is there. Its fields:
  - `Name: stackific-release`, `Version: 0.1.0`, `Release: 1%{?dist}`, `License: Apache-2.0`, `URL: https://os.stackific.com`, `BuildArch: noarch`;
  - `Source0: branding.env`, `Source1: os-release.in`, `Source2: logo.svg`, `BuildRequires: gettext-envsubst`;
  - `Requires: generic-release-common`, `Requires: distribution-systemd-presets`, `Requires: hicolor-icon-theme`;
  - `Provides: system-release`, `Provides: system-release(45)`, `Provides: system-release(releasever) = 45`, `Provides: generic-release-variant = 45-0.4`, and `Conflicts: generic-release`.
  - `generic-release-variant = 45-0.4` gets the comment `# shortcut: matches generic-release-common 45-0.4's exact Requires; bump it when Fedora updates generic-release`.
  - `%build` renders `os-release` and `logo-color.svg` with the render rules in Context, `%` written `%%`. It then appends `ID_LIKE=fedora`, `VERSION_ID=45` and `SUPPORT_END=2027-12-08`.
  - `%install` puts them at `/usr/lib/os-release` and `/usr/share/icons/hicolor/scalable/apps/stackific.svg`, and `%files` lists both.
- `fedora/packages/stackific-desktop/stackific-desktop.spec` (new):
  - `Version: 0.2.0` (as the Arch package), `Release: 1%{?dist}`, `BuildArch: noarch`;
  - `Source0`–`Source7`: `stackific-noctalia.service`, `stackific-wlroots.sh`, `stackific-noctalia.toml`, `00-stackific.toml`, `branding.env`, `stackific-greeter.toml.in`, `stackific-greeter.conf`, `stackific-greeter.sh`;
  - `BuildRequires: gettext-envsubst` and `systemd-rpm-macros` (for `%{_tmpfilesdir}`);
  - `Requires: mangowm noctalia noctalia-greeter systemd stackific-artwork stackific-release`, as the Arch package's `depends`. Terra's `noctalia-greeter` 1.7.0-2.fc45 installs `/usr/share/noctalia-greeter/assets/fonts` and `noctalia.svg`, the same paths as Arch's (`dnf repoquery -l`, checked 2026-11-05).

  `%build` renders `stackific-greeter.toml` with the same lines as the PKGBUILD. `%install` and `%files` follow the table in Context. `%dir` covers `mango-session.target.wants`, `/usr/share/stackific`, `/etc/skel/.config` and `/etc/skel/.config/noctalia`. `%config(noreplace)` covers the three config files the Arch package lists in `backup=()`; `%dir` also covers `/usr/share/stackific/greeter-assets`.
- `fedora/packages/stackific-artwork/stackific-artwork.spec` (new):
  - `Version: 0.1.0`, `Release: 1%{?dist}`, `BuildArch: noarch`, `License: Apache-2.0 AND CC0-1.0 AND LicenseRef-PublicDomain`;
  - sources `branding.env`, `fastfetch.jsonc.in`, `logo.txt`, `wallpapers.toml` and the ten JPEGs, `BuildRequires: gettext-envsubst`, `Requires: fastfetch` (Arch's `depends` since task 006 made fastfetch a dependency of the artwork, so installing our branding installs it);
  - `%build` renders `fastfetch.jsonc`. `%install` and `%files` follow the table in Context, with `%dir /usr/share/backgrounds/stackific` and `%dir /usr/share/stackific`, and `%config(noreplace)` on `/etc/xdg/fastfetch/config.jsonc`.
- `docs/fedora-iso.md`: a new section "Our packages", like `docs/arch-iso.md`'s. It covers `just fedora packages`, mock's flags and why, staging by `Source` name, the repo, and how `stackific-release` replaces `generic-release` (the provides, `releasever`, the kept `generic-release-common`). It also covers the `%%` gotcha, and that a branding edit means a `Release` bump. The "What is in the image" bullet on `generic-release` stays until task 008.
- `AGENTS.md`:
  - Product, the `stackific-release` bullet: it owns `/etc/os-release` on Arch and `/usr/lib/os-release` on Fedora (replacing `generic-release`; `generic-release-common`'s `/etc/os-release` symlink points at it). `ID_LIKE`, and `BUILD_ID` or `VERSION_ID`, are set per base.
  - Builds, the Fedora builder line: it also has mock.
  - Builds, the `generic-release` line: our `stackific-release` replaces `generic-release`, and `generic-release-common` stays (Fedora's rpm macros, dnf5 defaults, `/etc` symlinks).
  - Builds, "Our packages": add a Fedora sub-bullet: `fedora/packages/<name>/<name>.spec`, mock as container root with `--isolation=simple --no-bootstrap-image`, and `just fedora packages` into `.build/fedora/repo/` (createrepo_c, unsigned until release signing).

## Acceptance criteria

Must:
- `just fedora packages` exits 0 and leaves `stackific-release-0.1.0-1.fc45.noarch.rpm`, `stackific-desktop-0.2.0-1.fc45.noarch.rpm`, `stackific-artwork-0.1.0-1.fc45.noarch.rpm` and `repodata/repomd.xml` in `.build/fedora/repo/`, and no `.src.rpm` or logs.
- `bsdtar -tf` on each RPM lists exactly the paths in Context's table (plus the directories), and the Arch package from task 006 lists the same paths, except for the os-release path. Compare `bsdtar -tf` output of the RPM and the `.pkg.tar.zst` with the metadata and directory entries filtered out.
- `bsdtar -xOf` of `./usr/lib/os-release` from `stackific-release` prints the ten lines of task 006's rendered os-release, byte for byte the Arch package's first ten lines, followed by `ID_LIKE=fedora`, `VERSION_ID=45` and `SUPPORT_END=2027-12-08`. The icon has sha256 `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`. `/etc/xdg/fastfetch/config.jsonc` from `stackific-artwork` matches the Arch package's file byte for byte.
- `rpm -qp --provides` on `stackific-release` lists `system-release`, `system-release(45)`, `system-release(releasever) = 45` and `generic-release-variant = 45-0.4`. `rpm -qp --requires` lists `generic-release-common`, `distribution-systemd-presets` and `hicolor-icon-theme`. `rpm -qp --conflicts` lists `generic-release`.
- `rpm -qp --configfiles` on `stackific-desktop` lists `/etc/profile.d/stackific-wlroots.sh`, `/etc/profile.d/stackific-greeter.sh` and `/etc/skel/.config/noctalia/00-stackific.toml`. Its rendered `/usr/share/stackific/greeter.toml` is byte-identical to the Arch package's, and its two `greeter-assets` links have the same targets. On `stackific-artwork` it lists `/etc/xdg/fastfetch/config.jsonc`, and `rpm -qp --requires` lists `fastfetch`.
- In a fresh container of the pinned Fedora base, installing `stackific-release` from `.build/fedora/repo/` with `dnf -y install --allowerasing` succeeds. After it:
  - `. /etc/os-release; echo "$ID/$ID_LIKE/$VERSION_ID/$PRETTY_NAME"` prints `stackific/fedora/45/Stackific OS`;
  - `rpm -q generic-release fedora-release-common` reports both not installed, and `rpm -q generic-release-common redhat-systemd-presets` reports both installed;
  - `dnf5 --dump-variables` prints `releasever = 45`, and `dnf repoquery kernel` prints a `.fc45` kernel.
- `just lefthook all` passes.

Must not:
- touch files outside the Changes list (the review record and prompt-trail entry from /s-build excepted), and in particular `fedora/iso/`, `tests/vm/` and `.github/workflows/`;
- copy, patch or edit any file of `desktop/` or `branding/` for Fedora; the specs install the same files and render them with the same lines;
- ship or require `fedora-release*`, `fedora-logos` or `fedora-release-notes`, or provide `system-logos`;
- add a dependency this task does not name (the new ones: `mock` in the builder, `gettext-envsubst` and `systemd-rpm-macros` as BuildRequires, `fastfetch` as a Requires of `stackific-artwork`, and `noctalia-greeter` and `stackific-release` as Requires of `stackific-desktop`, as on Arch).

## Tests
- `just fedora builder`, then `just fedora packages`: the repo criterion. Note the wall time in the review record (expect about 2 minutes for the first mock call, then about 20 seconds per call).
- `bsdtar -tf`, `bsdtar -xOf` and `sha256sum` on the RPMs and on task 006's Arch packages from `just arch packages`: the path list, os-release, icon and fastfetch criteria.
- `podman run --rm -v "$PWD/.build/fedora/repo:/repo:ro" localhost/stackific-fedora-builder:45 rpm -qp --provides --requires --conflicts --configfiles /repo/<rpm>` (one flag per call): the metadata criteria.
- The install probe, in the pinned base (`registry.fedoraproject.org/fedora:45@sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd`), with `.build/fedora/repo` mounted at `/repo`:
  - `dnf -y install --allowerasing --repofrompath=stackific,/repo --setopt=stackific.gpgcheck=0 stackific-release`;
  - then the os-release, `rpm -q`, `dnf5 --dump-variables` and `dnf repoquery kernel` commands in Acceptance criteria.

  Record the output in the review record. `stackific-desktop` is not installed here: it needs Terra's `mangowm`, which task 008's image brings, and task 008's boot test proves it.
- `just lefthook all` and `just doctor`.
