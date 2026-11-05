# 006: Branding file, wallpapers and fastfetch logo, with an Arch artwork package

- [ ] Done

Base: arch

## Ask

```text title="Prompt" wrap
Give Stackific OS one branding file that every branded file renders from: the product name, the ID, the URLs and the accent colour. Make it a shell KEY="value" file, branding/branding.env, and have our package builds render base-neutral *.in templates from it with envsubst, so the os-release and the coloured logo icon come from it instead of from hard-coded copies. Commit the ten public-domain paintings as the wallpaper set, with their credits in branding/wallpapers.toml, and the block-character text logo for fastfetch. Ship them in a new Arch package, stackific-artwork: the wallpapers under /usr/share/backgrounds/stackific/ and a system-wide fastfetch config that shows our logo in the accent colour. Ship Noctalia defaults in stackific-desktop that point its wallpaper folder at the shipped set, make Johan Christian Dahl's "An Eruption of Vesuvius" the default wallpaper, and turn Noctalia's setup wizard off everywhere. Replace Noctalia's branding on the login screen with ours: stackific-desktop ships noctalia-greeter defaults with our palette (the accent from the branding file) and the same painting, linked into place by a tmpfiles line that leaves an administrator's own greeter.toml alone, and puts our green-on-transparent logo where Noctalia's was, through the greeter's documented NOCTALIA_GREETER_ASSETS_DIR override: an asset folder of links to stackific-release's icon and the greeter's own fonts, named in a profile.d script that greetd sources before the greeter. All four images are one product: the same branding, messaging, defaults and checks everywhere, differing only where a base forces it; write that into AGENTS.md. Make fastfetch a dependency of stackific-artwork, so installing our branding installs fastfetch with our logo, on the live ISO and on installed systems. The live ISO installs the new package. The Arch Linux ARM aarch64 console image installs our branding too (stackific-release and stackific-artwork, so fastfetch with our logo), with matching boot-test checks. Extend the boot test to prove the wallpapers, the fastfetch logo and the Noctalia defaults, take a screenshot of the desktop, and report the ISO size against the 2 GiB target. Fedora's RPMs of the same files are task 007.
```

## Goal
`branding/branding.env` is the one place that names the product, its ID, URLs and accent colour, and `stackific-release` renders os-release and the logo icon from it. The Arch live ISO boots to MangoWM showing Dahl's "An Eruption of Vesuvius" behind Noctalia's bar with no setup wizard, `fastfetch` prints our logo in emerald, and Noctalia's wallpaper picker browses the ten shipped paintings.

## Context
Task 004 made the image identify as Stackific OS: `arch/packages/stackific-release/PKGBUILD` installs `branding/os-release` (base-neutral fields) to `/etc/os-release`, appends `ID_LIKE=arch` and `BUILD_ID=rolling`, and installs `branding/logo-color.svg` as `/usr/share/icons/hicolor/scalable/apps/stackific.svg`. Task 004 left the branding file for later. Until it lands, the accent `#007a55` sits in two files: `ANSI_COLOR="38;2;0;122;85"` in `branding/os-release` and the fill in `branding/logo-color.svg`. Task 002 added `stackific-desktop` (the Noctalia user unit and `stackific-wlroots.sh`). `just arch packages` (`arch/justfile`) builds every `arch/packages/*/PKGBUILD` in a fresh builder container. It copies each file named in `source=()` from `desktop/` or `branding/`, matched by base name anywhere below them, so every source name must be unique across both folders. Then it runs `makepkg --nodeps` as `build` and `repo-add`s the result into `.build/arch/repo/`. The boot test's Arch checks live in `tests/vm/arch-x86_64.sh`; after a logout it takes `screen.png` and `greeter.png`.

Facts, checked 2026-11-05:

Repo files:
- `branding/logo.svg` (191 bytes, sha256 `43625b2beb9010a2ec1686c31e3b7173921c702b39f9050b2673ae6087a0514e`) has exactly one `fill="currentColor"`. `sed 's/currentColor/#007a55/' branding/logo.svg` gives sha256 `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`, byte for byte the committed `branding/logo-color.svg`, so the recolour can move to build time with an unchanged icon. Measured.
- `branding/os-release` has sha256 `d2c7db7827eeb4e17c99b29cd3dc5617034f3cee9d3f74f81a8bb813feeb8ceb` and these lines, with no comments (some readers split every line on `=`):
  ```text
  NAME="Stackific OS"
  PRETTY_NAME="Stackific OS"
  ID=stackific
  DEFAULT_HOSTNAME=stackific
  HOME_URL="https://os.stackific.com/"
  DOCUMENTATION_URL="https://os.stackific.com/docs/"
  SUPPORT_URL="https://github.com/stackific/os/issues"
  BUG_REPORT_URL="https://github.com/stackific/os/issues"
  LOGO=stackific
  ANSI_COLOR="38;2;0;122;85"
  ```
- `arch/iso/airootfs/etc/passwd` gives `live` UID 1000 and home `/home/live`. No `/home/live` exists in the profile.
- The pre-commit hygiene check (`just lefthook hygiene`, `just/lefthook/mod.just`) refuses files over 1 MiB except under `branding/wallpapers/`.

Builder (`localhost/stackific-arch-builder:2026-11-01`, archive date 2026-11-01; `pacman -Si`/`-Q` in it):
- `gettext` 1.0-2 (core) is installed and provides `envsubst`. With an explicit variable list (`envsubst '$NAME $ID'`), envsubst replaces only those variables and leaves every other `$` alone. Source: GNU gettext manual, "envsubst Invocation".
- archiso 92-1's `mkarchiso` copies `/etc/skel/.` into the home of every user in the profile's `airootfs/etc/passwd` with a UID from 1000 to 59999, and creates that home first (`/usr/bin/mkarchiso` lines 399-417). So a file the package puts in `/etc/skel` lands in `/home/live`.
- `fastfetch` 2.70.0-1 (extra) and `noctalia` 5.4.1-1 (extra) ship no file under `/etc` (`bsdtar -tf` on both packages from the archive). noctalia ships `usr/share/noctalia/assets/noctalia-wallpaper.png`.

Noctalia 5.4.1 (https://github.com/noctalia-dev/noctalia, tag v5.4.1, commit 815aebe1cfd79c9a18197441eea92f0b275035fc; docs at https://docs.noctalia.dev/noctalia/v5.4.1/):
- It reads config only from the user's config directory: `$NOCTALIA_CONFIG_HOME/noctalia`, else `$XDG_CONFIG_HOME/noctalia`, else `~/.config/noctalia` (`src/util/file_utils.h` lines 375-387). It has no system-wide or distro config directory; the docs' configuration page names none either.
- Load order: built-in defaults, then every `*.toml` in the config directory in alphabetical order, each one's `[include]` files first, then the file's own keys. Then `~/.local/state/noctalia/settings.toml`, which the settings UI writes, loads last and wins. The root files are deep-merged in sorted order, so a later file wins (`src/config/config_merge.cpp` lines 149-190; docs, "Configuration").
- `[include] files = ["<path>"]` takes absolute paths (`FileUtils::resolvePath`, `src/util/file_utils.h` lines 491-506, and `config_merge.cpp` lines 96-117). A missing include is logged as `config include not found` and skipped.
- `[wallpaper] directory` is the folder the wallpaper picker browses. Empty means the XDG Pictures folder. `[wallpaper.default] path` sets the initial wallpaper for every output. Choosing a wallpaper in the UI writes `settings.toml`, which overrides both. Source: docs, "Wallpaper & Backdrop".
- With no wallpaper configured, Noctalia shows its bundled `noctalia-wallpaper.png` until setup completes; a configured `wallpaper.default.path` takes precedence (`src/config/config_overrides.cpp` lines 2210-2230).
- `[shell] setup_wizard_enabled` (bool, default `true`, `src/config/config_types.h` line 1358, schema name in `src/config/schema/config_schema.cpp` line 1818) turns the first-run setup wizard off when `false` (`src/config/config_service.cpp` lines 683-690). Without it, the wizard opens at every first login, which on a live image is every boot.
- `noctalia config export merged` prints the merged user configuration, includes expanded (`src/config/cli.cpp` lines 404-430).

fastfetch 2.70.0 (https://github.com/fastfetch-cli/fastfetch, tag 2.70.0):
- It looks for `fastfetch/config.jsonc` in `~/.config/`, then `$XDG_CONFIG_DIRS`, then `/etc/xdg/`, then `/etc/`, and uses the first one it finds (`src/common/impl/FFPlatform_unix.c` lines 255-263, `src/fastfetch.c` lines 735-752). A user's own config therefore replaces ours.
- A config without `modules` or `structure` keeps the default module list (`src/fastfetch.c` lines 785-789).
- Logo `"type": "file"` reads a text file with colour placeholders (`doc/json_schema.json` line 608). `$1` takes `logo.color."1"`, which accepts `#RRGGBB` (`src/common/impl/option.c` line 184). In pipe mode (no terminal, or `--pipe`) the colour codes are dropped (`src/logo/logo.c` line 153); `--pipe false` keeps them.

Wallpapers (copied in, see Inputs): ten JPEGs, 41 MiB together, from 3744×3080 to 7816×6397 pixels. Nine are CC0-1.0, and Pissarro's is public domain with no named licence (the Dallas Museum of Art's open-access terms), so its SPDX id is `LicenseRef-PublicDomain`. The default, `an-eruption-of-vesuvius-johan-christian-dahl.jpg`, is 3872×2592 pixels (1.49:1).

## Inputs
Copied into this worktree's tracked tree while the task was written (all checked against the pins they came with):
- `branding/wallpapers/*.jpg`: the ten paintings, sha256s listed under Changes (`stackific-artwork`).
- `branding/wallpapers.toml`: one `[[wallpaper]]` entry per image: `file`, `title`, `artist`, `year`, `institution`, `object_id`, `object_url`, `licence`, and a comment naming the museum file and any crop. The header is new. The earlier copy's `sha256`, `size` and `version` keys are dropped, because git and the PKGBUILD now pin the bytes. sha256 `0af37796e25aea7b4ccf9bba41b8c898c2751801de625006cfdb38903e2c8c89`.
- `branding/logo.txt`: the operator's block-character text logo for fastfetch, nine lines, each starting with the colour placeholder `$1`. sha256 `d10a6fe45d48f754ec535750564c759133f916af16aae435cc5b28918b741d5f`.

Decisions restated from earlier material:
- Every branding item comes from one branding file rendered at build time. A rename is one edit and a rebuild. Runtime readers use os-release.
- The default wallpaper folder points at the shipped set. A user's own choice in Noctalia overrides it.
- Noctalia's defaults ship as a packaged file, so default fixes reach users who never changed them.

The operator chose, in this session (2026-11-05):
- the branding file is `branding/branding.env`, shell `KEY="value"` lines, rendered with `envsubst`. It holds only keys with a consumer today: name, ID, the four URLs and the accent;
- the default wallpaper is Dahl's "An Eruption of Vesuvius";
- Noctalia's setup wizard is off everywhere, on installed systems too.
- fastfetch is a hard dependency of `stackific-artwork`, so installing our branding installs fastfetch with our logo (after the first build, 2026-11-05).
- The aarch64 image gets the same branding: `stackific-release` and `stackific-artwork` with fastfetch (also after the first build).
- The login screen shows our branding instead of Noctalia's: our green-on-transparent logo (`stackific-release`'s icon) in place of Noctalia's, our palette and Vesuvius. The operator kept noctalia-greeter over SDDM or GDM.
- All four images are one product (branding, messaging, defaults, checks), differing only where a base forces it; `AGENTS.md` says so.

These design choices are made in this task:
- the package name `stackific-artwork` and the paths `/usr/share/backgrounds/stackific/` and `/usr/share/stackific/`;
- `/etc/xdg/fastfetch/config.jsonc` for fastfetch;
- a one-line `[include]` file in `/etc/skel` that points each user's Noctalia config at the packaged defaults;
- `ANSI_COLOR` derived from the accent at build time;
- the recolour of `logo.svg` at build time, which replaces the committed `logo-color.svg` that task 004 chose for the time before a branding file.

Left out, for later tasks:
- Fedora RPMs of these files (task 007), and `stackific-desktop` on the aarch64 image (its desktop task adds it).
- The site's fonts and colours from the branding file. AGENTS.md says they come from it, but the site uses shadcn's OKLCH tokens (`site/src/styles/theme.css`), so this is a site chore of its own.
- Boot-menu, plymouth and Limine branding; PNG icon sizes; UI strings in the branding file.
- Smaller wallpaper files: the Pissarro alone is 7.3 MB at 7816×6397.

## Changes
- `branding/branding.env` (new), with a header comment that says it is the one branding file: shell syntax, sourced by package builds, whose templates (`*.in`) name its keys as `${KEY}`. Its keys:
  ```sh
  NAME="Stackific OS"
  ID=stackific
  HOME_URL="https://os.stackific.com/"
  DOCUMENTATION_URL="https://os.stackific.com/docs/"
  SUPPORT_URL="https://github.com/stackific/os/issues"
  BUG_REPORT_URL="https://github.com/stackific/os/issues"
  ACCENT="#007a55"
  ```
- `branding/os-release` → `branding/os-release.in` (renamed, then edited). It has the same ten lines with `"${NAME}"` for `NAME` and `PRETTY_NAME`, `${ID}` for `ID`, `DEFAULT_HOSTNAME` and `LOGO`, `"${…_URL}"` for the four URLs and `"${ANSI_COLOR}"`. It has no comment lines.
- `branding/logo-color.svg`: delete; `stackific-release` renders it.
- `branding/fastfetch.jsonc.in` (new): a JSONC object with a header comment and only a `logo` key: `"type": "file"`, `"source": "/usr/share/stackific/logo.txt"`, `"color": { "1": "${ACCENT}" }`.
- `branding/logo.txt`, `branding/wallpapers.toml`, `branding/wallpapers/*.jpg`: already in place (see Inputs); do not edit them.
- `arch/packages/stackific-release/PKGBUILD`: `pkgrel=2`, `source=('branding.env' 'os-release.in' 'logo.svg')` with their sha256s, `makedepends=('gettext')`, and the header comment updated. A new `build()`:
  - sources `branding.env` with `set -a`;
  - derives `ANSI_COLOR` from `ACCENT` (`printf -v ANSI_COLOR '38;2;%d;%d;%d' "0x${ACCENT:1:2}" "0x${ACCENT:3:2}" "0x${ACCENT:5:2}"`);
  - renders `os-release` with `envsubst` and the explicit list `'$NAME $ID $HOME_URL $DOCUMENTATION_URL $SUPPORT_URL $BUG_REPORT_URL $ANSI_COLOR'`, then appends the Arch lines;
  - renders `logo-color.svg` with `sed "s/currentColor/$ACCENT/" logo.svg`.

  `package()` installs both files as before.
- `arch/packages/stackific-artwork/PKGBUILD` (new), shaped like the other two:
  - `pkgname=stackific-artwork`, `pkgver=0.1.0`, `pkgrel=1`, `arch=('any')`, `url='https://os.stackific.com'`, `license=('Apache-2.0' 'CC0-1.0' 'LicenseRef-PublicDomain')`, `options=('!debug' '!strip')`, `makedepends=('gettext')`, `depends=('fastfetch')`, `backup=('etc/xdg/fastfetch/config.jsonc')`;
  - `source=()`: `branding.env`, `fastfetch.jsonc.in`, `logo.txt`, `wallpapers.toml` and the ten JPEGs, with these sha256s for the JPEGs:
    - `an-eruption-of-vesuvius-johan-christian-dahl.jpg`: `da2450709d123ea009945144037c4d22bce474acece8ff0b082d26d86ba31b06`
    - `festoon-of-fruits-johannes-borman.jpg`: `e60ee595654d0cc53a19968c7a357d4974a5462e0b9cb367eaaad345c4793927`
    - `fog-effect-camille-pissarro.jpg`: `0249997196ea3d212f6f166f6d5bd83c589491efa393cb0865c431dedead2293`
    - `pines-along-the-shore-henri-edmond-cross.jpg`: `523cb31b3bafd9eba83247ec43efdaee456b9be2bf711dd254c2c61b98003405`
    - `still-life-with-a-gilt-cup-willem-claesz-heda.jpg`: `0e1849ce6d12d8347383070f8bb10375ef035274e8c671dc52039e756a13cd5d`
    - `sunset-after-a-storm-andreas-achenbach.jpg`: `522beca69964bc62da559e8f5f7d7e25b00a39b2696dd48efde439026ecc095e`
    - `the-north-cape-by-moonlight-peder-balke.jpg`: `c992c6b7b65f9be9a4b65745eb37b2ad9e993919ebf075201f1ed9945de22365`
    - `twilight-in-the-wilderness-frederic-edwin-church.jpg`: `8c54e04280e08cecfbdad90ea6a5afa10e554781184488028b21a7c5e9349d9b`
    - `view-from-mount-holyoke-thomas-cole.jpg`: `40c454b6cf712c6ef70cca067fbe551c069d4b9130d06716db05452d16d515c4`
    - `wheat-field-with-cypresses-vincent-van-gogh.jpg`: `6212f93c9549bbc2858a137f8db4d76073ef703fda7ac329157135f0a03fdfca`
  - `build()` sources `branding.env` and renders `fastfetch.jsonc` with `envsubst '$ACCENT'`;
  - `package()` installs the JPEGs (mode 644) to `/usr/share/backgrounds/stackific/`, `wallpapers.toml` to `/usr/share/licenses/stackific-artwork/`, `logo.txt` to `/usr/share/stackific/` and `fastfetch.jsonc` as `/etc/xdg/fastfetch/config.jsonc`.
- `desktop/noctalia/stackific-noctalia.toml` (new): our Noctalia defaults, with a header comment that says what each key is for:
  ```toml
  [shell]
  setup_wizard_enabled = false

  [wallpaper]
  directory = "/usr/share/backgrounds/stackific"

  [wallpaper.default]
  path = "/usr/share/backgrounds/stackific/an-eruption-of-vesuvius-johan-christian-dahl.jpg"
  ```
- `desktop/noctalia/00-stackific.toml` (new), the per-user pointer. A comment says the package keeps the defaults it includes up to date, and that the user's own `*.toml` files in that folder sort after it and win. Then:
  ```toml
  [include]
  files = ["/usr/share/stackific/noctalia.toml"]
  ```
- `arch/packages/stackific-desktop/PKGBUILD`: `pkgver=0.2.0`, `depends+=('stackific-artwork')`, the two new sources with their sha256s, and `backup+=('etc/skel/.config/noctalia/00-stackific.toml')`. `package()` installs `stackific-noctalia.toml` as `/usr/share/stackific/noctalia.toml` and `00-stackific.toml` to `/etc/skel/.config/noctalia/`. It also sources `branding.env` and renders `stackific-greeter.toml.in` with `envsubst '$ACCENT'` in `build()` (`makedepends=('gettext')`), and installs the result as `/usr/share/stackific/greeter.toml` and `stackific-greeter.conf` to `/usr/lib/tmpfiles.d/`. The header comment and `pkgdesc` name the Noctalia and greeter defaults.
- `desktop/noctalia/stackific-greeter.toml.in` (new): noctalia-greeter's administrator config: `[appearance] scheme = "Synced"`, `theme_mode = "dark"`; a complete `[appearance.palette]` (16 keys) with `primary`, `tertiary` and `hover` as `${ACCENT}` on the site's dark taupe; `[appearance.wallpaper] path` the Vesuvius painting, `fill_mode = "crop"`. noctalia-greeter 1.7.0 reads `/var/lib/noctalia-greeter/greeter.toml`, which wins over Sync's `sync.toml`, and has no key for another logo; it takes `noctalia.svg` and `fonts/` from its asset folder, which `NOCTALIA_GREETER_ASSETS_DIR` overrides (`PACKAGING.md` line 70; `src/core/resource_paths.cpp`; Arch's and Terra's packages both install `/usr/share/noctalia-greeter/assets/{fonts,noctalia.svg}`) (its `docs/user/configuration.md`, `examples/greeter.toml`, tag v1.7.0, checked 2026-11-05).
- `desktop/profile.d/stackific-greeter.sh` (new): `export NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets`, with a comment saying why. greetd sources `/etc/profile` before the greeter. `stackific-desktop` installs it to `/etc/profile.d/` (in `backup=()`), creates `/usr/share/stackific/greeter-assets/` with two links, `noctalia.svg` → `/usr/share/icons/hicolor/scalable/apps/stackific.svg` (stackific-release's emerald icon) and `fonts` → `/usr/share/noctalia-greeter/assets/fonts`, and adds `noctalia-greeter` and `stackific-release` to `depends`.
- `desktop/tmpfiles.d/stackific-greeter.conf` (new): `L /var/lib/noctalia-greeter/greeter.toml - - - - /usr/share/stackific/greeter.toml`. `L` without `+` creates the link only when the file is missing. Nothing in the image writes `greeter.toml`, and Arch's noctalia-greeter 1.7.0-1 creates the directory itself (`d /var/lib/noctalia-greeter 0750 greeter greeter -`).
- `arch/iso/packages.x86_64`: add `stackific-artwork` (it pulls in `fastfetch`).
- `tests/vm/arch-x86_64.sh`: the checks under Tests, before the screenshot.
- `arch/iso/packages.aarch64`: add `stackific-release` and `stackific-artwork`. Arch Linux ARM has `fastfetch` 2.70.0-1 (extra) and `gettext` 1.0-2 (core, pulled in by `base`, so the aarch64 builder has `envsubst`). Its `filesystem` 2025.10.12-1 owns `/usr/lib/os-release` and `/etc/arch-release`, not `/etc/os-release`, as on Arch (ALARM repo databases `extra.db`, `core.db`, `core.files`, checked 2026-11-05).
- `tests/vm/arch-aarch64.sh`: the identity checks from `arch-x86_64.sh` (os-release fields, file owners), the wallpaper count and the fastfetch logo check (as root), before the getty checks.
- `docs/arch-iso.md`, "Our packages": the branding file and how packages render from it (and that a branding edit means new sha256s and a `pkgrel` bump in the PKGBUILDs that source it), `stackific-artwork`, the two new `stackific-desktop` files, why the defaults reach users through an `/etc/skel` include (Noctalia has no system config directory), and `stackific-release`'s rendered files in place of the committed coloured SVG.
- `site/src/content/docs/user/wallpapers.md` (new): where the wallpapers live, how to pick another in Noctalia's wallpaper panel, and a credits table (title, artist, year, museum, licence) from `branding/wallpapers.toml`.
- `AGENTS.md`:
  - Product: the `stackific-release` bullet says the icon is `branding/logo.svg` filled with the accent at build time.
  - Product: one bullet for the branding file (`branding/branding.env`, rendered with `envsubst` and an explicit variable list).
  - Product: one bullet for `stackific-artwork` and the Noctalia defaults (`/usr/share/stackific/noctalia.toml`, included from each user's `~/.config/noctalia/00-stackific.toml`).
  - A gotcha: Noctalia reads no system-wide config.
  - Product: one bullet that the four images are one product, differing only where a base or architecture forces it; one bullet for the greeter defaults.

## Acceptance criteria

Must:
- `just arch packages` exits 0 and leaves `stackific-release-0.1.0-2-any.pkg.tar.zst`, `stackific-desktop-0.2.0-1-any.pkg.tar.zst` and `stackific-artwork-0.1.0-1-any.pkg.tar.zst` in `.build/arch/repo/`.
- `bsdtar -xOf` of `etc/os-release` from the `stackific-release` package prints `branding/os-release`'s ten lines from Context, unchanged, followed by `ID_LIKE=arch` and `BUILD_ID=rolling`. The package's `usr/share/icons/hicolor/scalable/apps/stackific.svg` has sha256 `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`.
- `bsdtar -tf` on `stackific-artwork` lists the ten JPEGs under `usr/share/backgrounds/stackific/`, plus `usr/share/licenses/stackific-artwork/wallpapers.toml`, `usr/share/stackific/logo.txt` and `etc/xdg/fastfetch/config.jsonc`. That config contains `"1": "#007a55"` and no `${`.
- `bsdtar -xOf` of `.PKGINFO` from `stackific-artwork` prints `depend = fastfetch`, so installing the package installs fastfetch.
- `bsdtar -tf` on `stackific-desktop` lists `usr/share/stackific/noctalia.toml`, `etc/skel/.config/noctalia/00-stackific.toml`, `usr/share/stackific/greeter.toml` and `usr/lib/tmpfiles.d/stackific-greeter.conf`, plus the files from task 002.
- Changing `ACCENT` in `branding/branding.env` alone changes the accent in the rendered os-release (`ANSI_COLOR`), the icon fill and the fastfetch config. Proved once during /s-build with a throwaway value, then reverted.
- `just arch iso` exits 0 and its last line prints the ISO size in MiB. The manifest lists `stackific-artwork 0.1.0-1`, `stackific-desktop 0.2.0-1`, `stackific-release 0.1.0-2` and `fastfetch 2.70.0-1`.
- `just arch test` passes: every check from tasks 001, 002 and 004, plus the new ones under Tests.
- On an aarch64 host (the Mac runner VM), `just arch iso` and `just arch test` pass, and the aarch64 manifest lists `stackific-release`, `stackific-artwork` and `fastfetch`.
- `.build/arch/x86_64/screen.png` shows Dahl's "An Eruption of Vesuvius" behind Noctalia's bar, and no setup wizard. `.build/arch/x86_64/greeter.png` shows noctalia-greeter over the same painting with our emerald accent, and our green-on-transparent logo where Noctalia's was.
- `just lefthook all` passes (hygiene, shellcheck on the PKGBUILDs, `just dev check`, site checks).

Must not:
- touch files outside the Changes list, or any Fedora file (`fedora/`, `tests/vm/fedora-x86_64.sh`);
- change a file owned by `noctalia`, `fastfetch` or `mangowm`, or set `NOCTALIA_CONFIG_HOME` or `XDG_CONFIG_DIRS`;
- leave `#007a55` or `Stackific OS` hard-coded in a file a package renders. It may appear only in `branding/branding.env`, and in docs and tests that check the result;
- add a dependency this task does not name (the new ones are `gettext` as a makedepend, and `fastfetch` as a dependency of `stackific-artwork`).

## Tests
- `just arch packages`, then `bsdtar -tf` / `bsdtar -xOf` on the three packages and `sha256sum` on the icon: the package criteria.
- The `ACCENT` check: in a scratch copy of `branding/`, set `ACCENT="#123456"`, then build `stackific-release` and `stackific-artwork` from it in the builder as `just arch packages` does, but with `makepkg --skipchecksums`, because the PKGBUILDs pin `branding.env`'s sha256. The three rendered files then show `38;2;18;52;86`, `fill="#123456"` and `"1": "#123456"`. The working tree stays unchanged. Note the result in the review record.
- `just arch iso`, then `grep -E '^(stackific-artwork|stackific-desktop|stackific-release|fastfetch) ' .build/arch/x86_64/*.packages.txt`: the manifest versions.
- `just arch test`, with these new checks in `tests/vm/arch-x86_64.sh`, before the screenshot:
  - `ls /usr/share/backgrounds/stackific/*.jpg | wc -l` prints `10`;
  - `cmp /etc/skel/.config/noctalia/00-stackific.toml /home/live/.config/noctalia/00-stackific.toml` exits 0;
  - as `live`, `noctalia config export merged` prints `setup_wizard_enabled = false`, `directory = "/usr/share/backgrounds/stackific"` and the default wallpaper's path (match the exact line format to what 5.4.1 prints);
  - as `live`, `fastfetch --pipe false` prints the logo's last line (`▄▄▄▄▄█████████████████████████████████`) and the colour sequence `38;2;0;122;85`.
  - `readlink /var/lib/noctalia-greeter/greeter.toml` prints `/usr/share/stackific/greeter.toml`, and that file has `primary = "#007a55"`; `/usr/share/stackific/greeter-assets/noctalia.svg` resolves to `/usr/share/icons/hicolor/scalable/apps/stackific.svg`; the greeter's `/proc/<pid>/environ` has `NOCTALIA_GREETER_ASSETS_DIR=/usr/share/stackific/greeter-assets`.
- The screenshots `screen.png` and `greeter.png` from `just arch test`: the default wallpaper with no wizard, and the greeter with our look. Both go into the prompt-trail entry.
- Manual, on the Mac runner VM (this PC cannot build aarch64): `just arch iso`, then `just arch test`, then `grep -E '^(stackific-artwork|stackific-release|fastfetch) ' .build/arch/aarch64/*.packages.txt`. The `images` workflow runs the same on every merge to `main`.
- Manual, for the wallpaper picker (no automated check reaches Noctalia's panels):
  1. Boot `.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.iso` in a VM with a display, UEFI firmware and 4 GiB of RAM.
  2. MangoWM shows Dahl's Vesuvius behind Noctalia's bar, with no setup wizard.
  3. Open Noctalia's wallpaper panel (its bar widget or control center). It lists the ten paintings.
  4. Pick another one. It replaces Vesuvius, and `~/.local/state/noctalia/settings.toml` now names it.
  5. Open foot (`Alt+Return`) and run `fastfetch`. The emerald Stackific logo is on the left.
