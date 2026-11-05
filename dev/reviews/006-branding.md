# 006: Branding file, wallpapers and fastfetch logo, with an Arch artwork package: review

Task: dev/tasks/006-branding.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `branding/branding.env`, `branding/os-release.in`, `branding/fastfetch.jsonc.in`, the copied `branding/logo.txt`, `branding/wallpapers.toml` and `branding/wallpapers/*.jpg`, the removal of `branding/os-release` and `branding/logo-color.svg`, the three PKGBUILDs, `desktop/noctalia/*.toml`, `arch/iso/packages.x86_64`, `tests/vm/arch-x86_64.sh`, `docs/arch-iso.md`, `AGENTS.md`, `site/src/content/docs/user/wallpapers.md`, the prompt-trail entry and its screenshot.
Verdict: Looks good. Ship.

Checked: the rendered os-release and icon are byte-identical to task 004's committed files; the three package listings; a throwaway accent reaching all three rendered files; every caller of the changed sources (`just arch packages` stages them by name); Noctalia 5.4.1's include and merge order and its default bar `wallpaper` widget (`src/config/config_types.h` line 185) against the source; the boot test with the new checks.

Tests: `just arch packages` exits 0 with `stackific-release-0.1.0-2`, `stackific-desktop-0.2.0-1` and `stackific-artwork-0.1.0-1`; `bsdtar` listings and `sha256sum` match the criteria. The `ACCENT="#123456"` scratch build gives `38;2;18;52;86`, `fill="#123456"` and `"1": "#123456"`. `just arch iso` exits 0 at 1538 MiB (target 2048 MiB), with `fastfetch 2.70.0-1` and the three packages in the manifest. `just arch test` prints `PASS` with `ok` for every check, the four new ones included.

## Round 2
Source: the operator, after the first build (2026-11-05): install fastfetch with our branding, and put the branding on the aarch64 image too.
Reviewed: `stackific-artwork` now `depends=('fastfetch')` instead of `optdepends`; `arch/iso/packages.x86_64` drops its own `fastfetch` line; `arch/iso/packages.aarch64` adds `stackific-release` and `stackific-artwork`; `tests/vm/arch-aarch64.sh` gains the identity, wallpaper and fastfetch checks; `docs/arch-iso.md`, `AGENTS.md`, the task file and the prompt-trail entry follow.
Verdict: Looks good. Ship.

Checked: Arch Linux ARM's `extra.db` has `fastfetch` 2.70.0-1, and its `base` 3-3 depends on `gettext`, so the aarch64 builder has `envsubst`; its `filesystem` 2025.10.12-1 owns `/usr/lib/os-release` and `/etc/arch-release` but not `/etc/os-release` (`core.files`), so `stackific-release` and the copied owner check hold there; the shared profile gives aarch64 the same `IMAGE_ID`; root's fastfetch reads `/etc/xdg/fastfetch/config.jsonc`. The four checks repeated from `arch-x86_64.sh` stay, because `boot.sh` takes one check file per image.

Tests: on the branch rebased onto `origin/main` (with task 005), `just arch packages`, `just arch iso` and `just arch test` exit 0: 1538 MiB, `PASS`, manifest `fastfetch 2.70.0-1`, `stackific-artwork 0.1.0-1`, `stackific-desktop 0.2.0-1`, `stackific-release 0.1.0-2`; the package's `.PKGINFO` says `depend = fastfetch`. `just shellcheck lint tests/vm/arch-aarch64.sh arch/packages/stackific-artwork/PKGBUILD` passes. Not run: the aarch64 image, which only an aarch64 host builds (the Mac runner VM, or the `images` workflow after the merge).

## Round 3
Source: the operator (2026-11-05): replace Noctalia's branding on the greeter with ours, keep noctalia-greeter rather than SDDM or GDM, and make the four images one product in AGENTS.md.
Reviewed: `desktop/noctalia/stackific-greeter.toml.in`, `desktop/tmpfiles.d/stackific-greeter.conf`, `stackific-desktop`'s PKGBUILD (`build()` renders the greeter config, `package()` installs it and the tmpfiles line), the two new checks in `tests/vm/arch-x86_64.sh`, and `AGENTS.md`, `docs/arch-iso.md`, the task file and the entry.
Verdict: Looks good. Ship.

Checked: noctalia-greeter 1.7.0 (tag v1.7.0) reads `/var/lib/noctalia-greeter/greeter.toml`, which wins over `sync.toml`; `hide_logo` is its only logo key, and the Synced scheme uses a complete `[appearance.palette]` from that file; nothing in the image writes `greeter.toml`, because the greeter's `--setup-system` never runs, and Sync writes only `sync.toml`; Arch's package creates the directory (`d /var/lib/noctalia-greeter 0750 greeter greeter -`), and tmpfiles handles it before the link inside it; `L` without `+` leaves an existing file alone; the greeter user can read the link's target and the wallpaper (both 644 under `/usr/share`).

Tests: `just shellcheck lint` on the PKGBUILD and `tests/vm/arch-x86_64.sh` passes. `just arch packages`, `just arch iso` and `just arch test` exit 0: 1538 MiB, `PASS`, with `ok` for the `greeter.toml` link and its `hide_logo = true` and `primary = "#007a55"` lines, and for the greeter's PID after the logout. `greeter.png` shows the login panel in our palette over Vesuvius, with no Noctalia logo.

## Round 4
Source: the operator (2026-11-05): show the Stackific OS logo, green on transparent, on the greeter instead of hiding Noctalia's.
Reviewed: `desktop/profile.d/stackific-greeter.sh` (new), `stackific-desktop`'s PKGBUILD (the asset folder with its two links, `depends` on `noctalia-greeter` and `stackific-release`, the new `backup` entry), `hide_logo` dropped from `desktop/noctalia/stackific-greeter.toml.in`, the greeter checks in `tests/vm/arch-x86_64.sh`, and `AGENTS.md`, `docs/arch-iso.md`, the task file and the entry.
Verdict: Looks good. Ship.

Checked: noctalia-greeter 1.7.0 documents `NOCTALIA_GREETER_ASSETS_DIR` (`PACKAGING.md` line 70) and accepts a folder with `fonts/tabler.ttf` and `fonts/tabler.json` (`src/core/resource_paths.cpp`); it loads only `fonts/tabler.json`, `fonts/tabler.ttf` and `noctalia.svg` from it, so links to the package's `fonts` and to `stackific-release`'s icon cover every asset; Arch's and Terra's packages both install `/usr/share/noctalia-greeter/assets/{fonts,noctalia.svg}`; greetd sources `/etc/profile` before the greeter, so the variable reaches it; nothing in a package is changed.

Tests: `just shellcheck lint` on the PKGBUILD, `stackific-greeter.sh` and `tests/vm/arch-x86_64.sh` passes. `just arch packages`, `just arch iso` and `just arch test` exit 0: 1538 MiB, `PASS`, with `ok` for the asset folder's link to `/usr/share/icons/hicolor/scalable/apps/stackific.svg` and for `NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets` in the greeter's environment. `greeter.png` shows the emerald mark at the bottom centre, where Noctalia's logo was.
