# 004: Arch release package: the image identifies as Stackific OS

- [x] Done

Base: arch

## Ask

```text title="Prompt" wrap
Add an Arch release package, stackific-release, so the Arch live image identifies as Stackific OS: os-release with ID=stackific, the name "Stackific OS", the project URLs (site https://os.stackific.com, repo https://github.com/stackific/os) and our logo. Keep it small and quick. Ship it without patching any Arch package and without pacman file conflicts, so a later upgrade of Arch's filesystem package neither undoes nor breaks it, and keep /etc/arch-release as Arch ships it. Check every os-release field against os-release(5), and settle how a rolling base with semver releases states its version. The logo is the operator's mark, committed as branding/logo.svg; install it so LOGO= resolves as an icon, in the brand colour emerald #007a55, from a second committed SVG coloured emerald rather than recoloured at build time. On Arch set BUILD_ID=rolling, as Arch does; point SUPPORT_URL at GitHub issues; set DEFAULT_HOSTNAME=stackific. Keep the os-release and logo sources base-neutral, so Fedora's stackific-release RPM can reuse them later with only its per-base fields changed. Install the package into the Arch ISO, and prove in the boot test that the image reports ID=stackific and that hostnamectl shows Stackific OS, without running Arch-only checks on the Fedora ISOs that share the test.
```

## Goal
`just arch packages` builds `stackific-release`, which owns `/etc/os-release` (`ID=stackific`, `PRETTY_NAME="Stackific OS"`) and the icon `stackific`. `just arch iso` installs it, and `just arch test` proves the live image identifies as Stackific OS, while Arch's `filesystem` package and `/etc/arch-release` stay as shipped.

## Context
Tasks 001 and 002 built the Arch x86_64 live ISO from `arch/iso/` and our first package, `stackific-desktop` (`arch/packages/stackific-desktop/PKGBUILD`). `just arch packages` stages each PKGBUILD's `source=()` files from `desktop/` by file name, builds them with `makepkg`, and puts them into the local repo `.build/arch/repo/`, which the ISO reads as `[stackific]`. The image still identifies as Arch Linux. The last ISO (task 002, built 2026-11-04) is 1497 MiB (1570109440 bytes).

Facts, all checked 2026-11-05 in the builder `localhost/stackific-arch-builder:2026-11-01` unless another source is named:
- File ownership at archive date 2026-11-01:
  - filesystem 2025.10.12-1 owns `/usr/lib/os-release`, `/etc/arch-release` (one byte), `/usr/share/factory/etc/arch-release` and the Arch logos in `/usr/share/pixmaps/` (`pacman -Ql filesystem`).
  - No package in core or extra owns `etc/os-release`. `pacman -F etc/os-release usr/lib/os-release etc/arch-release` (after `pacman -Fy`) names only `core/filesystem`, for the other two paths.
  - filesystem's `/usr/lib/os-release`: `NAME="Arch Linux"`, `PRETTY_NAME="Arch Linux"`, `ID=arch`, `BUILD_ID=rolling`, `ANSI_COLOR="38;2;23;147;209"`, `HOME_URL`, `DOCUMENTATION_URL`, `SUPPORT_URL`, `BUG_REPORT_URL`, `PRIVACY_POLICY_URL` and `LOGO=archlinux-logo`. It has no `VERSION` or `VERSION_ID`.
- `/etc/os-release` comes from systemd 262.1-1's `/usr/lib/tmpfiles.d/etc.conf` (line 10): `L /etc/os-release - - - - ../usr/lib/os-release`. `L` only creates the symlink if the path does not exist yet. Only `L+` replaces an existing file. Source: tmpfiles.d(5), https://www.freedesktop.org/software/systemd/man/latest/tmpfiles.d.html.
- systemd runs tmpfiles from the alpm hook `/usr/share/libalpm/hooks/21-systemd-tmpfiles.hook`, `When = PostTransaction`. mkarchiso 92 installs every package in one pacstrap call (`_make_packages`, `/usr/bin/mkarchiso` line 351). So pacman's file-conflict check runs before the symlink exists.
- pacman refuses to install a file that already exists on disk when no package owns it ("exists in filesystem"), unless `--overwrite` is given. Source: pacman(8), https://man.archlinux.org/man/pacman.8.
- In task 002's live image, `/etc/os-release` is that symlink. `/usr/lib/os-release` ends with `IMAGE_ID=stackific` and `IMAGE_VERSION=0.0.0-dev-arch`, so the image build already changes a file that filesystem owns. Source: `unsquashfs -lls` and `-cat` on `arch/x86_64/airootfs.sfs` of `.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.iso`.
- The cause is mkarchiso 92's `_make_version` (`/usr/bin/mkarchiso` lines 1782–1792). It resolves `/etc/os-release` with `realpath`, or uses `/usr/lib/os-release` when `/etc/os-release` is missing. It deletes any `IMAGE_ID=` and `IMAGE_VERSION=` lines there, then appends `IMAGE_ID=${iso_name}` and `IMAGE_VERSION=${iso_version}`. `arch/iso/profiledef.sh` sets `iso_name="stackific"` and `iso_version="<version>-arch"`. With a regular `/etc/os-release`, mkarchiso appends to that file and leaves `/usr/lib/os-release` alone.
- os-release(5), at systemd 262 (https://www.freedesktop.org/software/systemd/man/latest/os-release.html):
  - `/etc/os-release` takes precedence over `/usr/lib/os-release`. If `/etc/os-release` exists, readers use only it and never merge the two. `/usr/lib/os-release` is the recommended place for vendor trees, with `/etc/os-release` as a relative symlink to it.
  - `NAME` is the OS name without a version. `PRETTY_NAME` is the name shown to users. `ID` is a lower-case identifier. `ID_LIKE` is a space-separated list of IDs, closest first.
  - Vendors "may choose not to provide version information, for example to accommodate for rolling releases", and then `VERSION` and `VERSION_ID` may be unset.
  - `BUILD_ID` identifies "the system image originally used as the installation base", and does not change during updates. `IMAGE_ID` and `IMAGE_VERSION` (since 249) identify an OS image, for systems built and updated as whole images.
  - `HOME_URL`, `DOCUMENTATION_URL`, `SUPPORT_URL` and `BUG_REPORT_URL` are optional links.
  - `LOGO` (since 240) is an icon name as defined by the freedesktop.org Icon Theme Specification.
  - `ANSI_COLOR` is an ECMA-48 SGR string (the part between `ESC [` and `m`) for the OS name on the console.
  - Values with spaces, semicolons or other characters outside `A–Z a–z 0–9` must be quoted.
- Readers of `/etc/os-release` in the image:
  - systemd-hostnamed: `hostnamectl` prints `Operating System:` and `PRETTY_NAME`.
  - agetty, through filesystem's `/etc/issue`: `\S{PRETTY_NAME} \r (\l)`.
  - mkinitcpio 42.2: it copies `/etc/os-release` into the initramfs (`/usr/lib/initcpio/functions` line 1226), for systemd's boot banner.
  - noctalia 5.4.1: it reads `/etc/os-release`, then `/usr/lib/os-release`, for its system information. Its icon lookup searches hicolor `scalable/apps` and `/usr/share/pixmaps` (`strings /usr/bin/noctalia`).
- hicolor-icon-theme 0.18-1's `index.theme` defines `[scalable/apps]` with `Type=Scalable`, `MinSize=1` and `MaxSize=256`. The live image already ships hicolor-icon-theme (task 002's manifest) and `/usr/share/icons/hicolor/scalable/apps/`.
- The logo `branding/logo.svg` is one path with `fill="currentColor"` in a 600×600 viewBox. When nothing sets the SVG `color` property, icon renderers draw `currentColor` as black. Measured with rsvg-convert 2.63.3: every rendered pixel is `#000000` (the image ships librsvg 2.63.4). Black is invisible on a dark panel.
- Emerald `#007a55` has a contrast ratio of 5.4:1 against white and 3.9:1 against black. Both are above the 3:1 that WCAG 2.1 SC 1.4.11 asks of graphics, so an emerald mark reads on light and dark backgrounds.
- `https://os.stackific.com/docs/` and `https://github.com/stackific/os/issues` both return HTTP 200 (curl). The repo has issues on and discussions off (`gh repo view stackific/os`). The docs root is `site/src/content/docs/index.md`, slug `docs` in `site/astro.config.mjs`.
- `tests/vm/boot.sh` takes the ISO path, and AGENTS.md fixes ISO names as `stackific-<version>-<base>-<arch>.iso`. So the test can tell an Arch ISO from its name. Task 003 reuses `boot.sh` for the Fedora console ISO, which does not ship this package.
- AGENTS.md rules this task applies:
  - configure and package, never patch;
  - our packages are built with makepkg in a fresh pinned container, with base-neutral sources;
  - `branding/` holds the logo;
  - every image records its manifest and reports its size;
  - shell files pass shellcheck (`PKGBUILD` included).

How `stackific-release` provides os-release, weighed:
- **Own `/etc/os-release` as a regular file (chosen).** systemd's `L` line then leaves the file alone. A `filesystem` upgrade rewrites only `/usr/lib/os-release`, which readers no longer use. Nothing of Arch's is changed, and mkarchiso's `IMAGE_*` lines move into our file. pacman owns and verifies the file.
  - Limit: the package must arrive in the same transaction as systemd (true for the ISO's single pacstrap). Installing it later onto a system where tmpfiles already made the symlink needs `pacman --overwrite /etc/os-release`. Our installed systems are copied from the live image, so they never take that path.
  - Leftover: `/usr/lib/os-release` still says Arch Linux.
- A pacman hook that rewrites `/usr/lib/os-release` after every `filesystem` upgrade: it edits a file another package owns, so `pacman -Qkk filesystem` flags it, and a `sed` breaks silently when Arch changes the file.
- Replacing `filesystem` with our own fork (`provides`, `conflicts`, `replaces`): every Arch change to `filesystem` must be tracked forever.
- An install scriptlet that copies the file into place: pacman does not own the result, and every upgrade overwrites it unseen.
- Owning `/usr/lib/os-release` as well: two packages cannot own one path, so pacman reports a conflict.

`/etc/os-release` is not in `backup=()`. It is vendor data, which os-release(5) places in the vendor tree, not user configuration. Arch treats it the same way: filesystem 2025.10.12-1's `backup` lists only files admins edit (`/etc/fstab`, `/etc/passwd`, `/etc/hosts` and the like), not `/usr/lib/os-release` (`pacman -Qii filesystem`, checked 2026-11-05). So a `stackific-release` upgrade replaces it, and it never leaves a `.pacnew`.

Versioning: the Arch base is rolling, so the package sets `BUILD_ID=rolling`, as Arch's own os-release does, and no `VERSION` or `VERSION_ID` (os-release(5) allows that for rolling releases). The ISO's semver version reaches the live image without our code: mkarchiso appends `IMAGE_ID=stackific` and `IMAGE_VERSION=<version>-arch` to `/etc/os-release`. Per-base fields such as `ID_LIKE` and `BUILD_ID` (or `VERSION_ID` on Fedora later) come from the base's package, not from `branding/os-release`.

Left out, for later tasks:
- Fedora's `stackific-release` RPM from the same `branding/` files. The `ID=stackific` check then moves out of the Arch-only part of `boot.sh`.
- The branding file that every branding item renders from. Until then, `#007a55` sits in two places: `ANSI_COLOR` in `branding/os-release` and the fill in `branding/logo-color.svg`.
- The installer's handling of mkarchiso's `IMAGE_ID` and `IMAGE_VERSION` lines on installed systems. The next `stackific-release` upgrade drops them anyway.
- `VENDOR_NAME`, `VENDOR_URL`, `PRIVACY_POLICY_URL`, `ANSI_COLOR_REVERSE` and `RELEASE_TYPE`.
- Boot menu, wallpaper and greeter branding.
- `lsb_release`, which would still print Arch Linux if a user installs `lsb-release`. The image does not ship it.

## Inputs
The operator's logo, copied verbatim to `branding/logo.svg` (191 bytes, sha256 `43625b2beb9010a2ec1686c31e3b7173921c702b39f9050b2673ae6087a0514e`). Its emerald copy is committed as `branding/logo-color.svg`: `logo.svg` with `currentColor` replaced by `#007a55` (186 bytes, sha256 `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`). No other earlier material.

The operator chose, in this session: `BUILD_ID=rolling` on Arch; a second committed coloured SVG instead of recolouring at build time; `SUPPORT_URL` at GitHub issues; `DEFAULT_HOSTNAME=stackific`; no `backup` entry for `/etc/os-release`; the ISO name as the Arch-only guard in the boot test. The other design choices are made in this task: own `/etc/os-release` and a hicolor icon named `stackific`.

## Changes
- `branding/logo.svg` and `branding/logo-color.svg`: already in place (see Inputs); do not edit them.
- `branding/os-release` (new), the base-neutral fields, with no comment lines (some readers split every line on `=`):
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
- `arch/packages/stackific-release/PKGBUILD` (new), shaped like `stackific-desktop`'s:
  - A header comment says why `/etc/os-release` and why the icon is the emerald copy (see Context), and carries `# shellcheck disable=SC2034,SC2154`.
  - `pkgname=stackific-release`, `pkgver=0.1.0`, `pkgrel=1`, `arch=('any')`, `url='https://os.stackific.com'`, `license=('Apache-2.0')`, `options=('!debug' '!strip')`. No `depends`, no `backup`.
  - `source=('os-release' 'logo-color.svg')`, with their sha256s.
  - `package()` writes `$pkgdir/etc/os-release`: `os-release` followed by the per-base lines `ID_LIKE=arch` and `BUILD_ID=rolling`, for example `{ cat os-release; echo ID_LIKE=arch; echo BUILD_ID=rolling; } >"$pkgdir/etc/os-release"`.
  - It installs `logo-color.svg` unchanged as `$pkgdir/usr/share/icons/hicolor/scalable/apps/stackific.svg`.
  - Both files are mode 0644.
- `arch/justfile`, in the `packages` recipe:
  - mount `branding/` read-only at `/branding`;
  - stage sources from `find /desktop /branding` instead of `find /desktop`;
  - the recipe comment says local sources come from `desktop/` and `branding/`.
- `arch/iso/packages.x86_64`: add `stackific-release` after `stackific-desktop`.
- `tests/vm/boot.sh`: Arch-only identity checks, after the wrong-token check and before the live-session checks. If `boot.sh` already splits checks per base when this task is built (task 003), add them to the Arch part. Otherwise guard them with `[[ $iso == *-arch-*.iso ]]` and a one-line comment saying why. Each check is one `vm_until ... 0`:
  1. `. /etc/os-release && echo "$ID/$ID_LIKE/$BUILD_ID/$PRETTY_NAME/$LOGO/$IMAGE_ID"` prints `stackific/arch/rolling/Stackific OS/stackific/stackific`. The `$` belongs to the guest shell, so silence SC2016 on that line.
  2. `hostnamectl` prints a line matching `Operating System: Stackific OS`.
  3. `pacman -Qqo /etc/os-release /etc/arch-release /usr/lib/os-release | paste -sd ' '` prints `stackific-release filesystem filesystem`.
  4. `grep -c IMAGE_ /usr/lib/os-release` prints `0`, so Arch's file stays as shipped.
  5. `grep -c 'fill="#007a55"' /usr/share/icons/hicolor/scalable/apps/stackific.svg` prints `1`.
- `docs/arch-iso.md`:
  - "Our packages": sources come from `desktop/` and `branding/`.
  - A `stackific-release` part: what it owns, why `/etc/os-release` and not `/usr/lib/os-release`, that `/etc/arch-release` stays, the `IMAGE_*` lines from mkarchiso, and why the icon is the committed emerald copy.
  - The sha256 note covers `branding/` too.
  - "Boot test": the identity checks.

  No user docs: nothing is released yet.
- `AGENTS.md`, one line each:
  - Builds, in the Arch packages line: base-neutral sources in `desktop/` and `branding/`.
  - Product: `stackific-release` owns `/etc/os-release` (`ID=stackific`, `ID_LIKE` and `BUILD_ID` set per base, `LOGO=stackific`, the icon being `branding/logo-color.svg`, the mark filled emerald `#007a55`). It takes precedence over the base's `/usr/lib/os-release`, so base packages and `/etc/arch-release` stay as shipped.

## Acceptance criteria

Must:
- `sha256sum branding/logo.svg branding/logo-color.svg` prints `43625b2beb9010a2ec1686c31e3b7173921c702b39f9050b2673ae6087a0514e` and `1644ae7c18a55a86489b80b79ae301b8645f333b25da7684c0b93d21f9ff4dde`.
- `diff <(sed 's/currentColor/#007a55/' branding/logo.svg) branding/logo-color.svg` prints nothing: the emerald copy is the same mark.
- `just arch packages` exits 0 and leaves `.build/arch/repo/stackific-release-0.1.0-1-any.pkg.tar.zst`.
  - Besides pacman's metadata files, `bsdtar -tf` on it lists only `etc/os-release` and `usr/share/icons/hicolor/scalable/apps/stackific.svg` (and their directories).
  - `bsdtar -xOf <pkg> etc/os-release` prints the ten lines of `branding/os-release`, then `ID_LIKE=arch` and `BUILD_ID=rolling`, and nothing else.
  - `cmp branding/logo-color.svg <(bsdtar -xOf <pkg> usr/share/icons/hicolor/scalable/apps/stackific.svg)` prints nothing.
- `stackific-desktop` still builds unchanged into the same repo.
- `just arch iso` exits 0, and its last line prints the ISO size in MiB. The manifest lists `stackific-release 0.1.0-1` and `filesystem 2025.10.12-1`.
- `just arch test` exits 0. It prints `ok` for the five identity checks in Changes and for every check it ran before.
- `just lefthook all` passes (shellcheck on the new PKGBUILD and on `tests/vm/boot.sh`).
- The task report states the ISO size in MiB against the 2048 MiB target.

Must not:
- change a file owned by `filesystem` or any other Arch package: no pacman hook, no `sed` on `/usr/lib/os-release`, no fork of `filesystem` and no install scriptlet;
- delete, replace or shadow `/etc/arch-release`;
- set `VERSION` or `VERSION_ID` on Arch, or put a per-base field into `branding/os-release`;
- recolour the logo at build time;
- run the Arch-only checks on an ISO whose name is not `*-arch-*.iso`;
- edit `branding/logo.svg` or `branding/logo-color.svg`;
- touch files outside the Changes list (the review record and prompt-trail entry from /s-build excepted);
- add a dependency this task does not name.

## Tests
- `sha256sum` and the `diff` on the two logos: prove the verbatim logo and that the emerald copy is the same mark.
- `just arch packages`, then the three `bsdtar` commands in Acceptance criteria: prove the package contents, the appended `ID_LIKE=arch` and `BUILD_ID=rolling`, and the emerald icon.
- `just arch iso`, then `grep -E '^(stackific-release|stackific-desktop|filesystem) ' .build/arch/x86_64/*.packages.txt`: proves the manifest.
- `just arch test`: proves the five identity checks, which cover `ID=stackific`, the `hostnamectl` name, ownership of the three files, Arch's untouched `/usr/lib/os-release` and the icon. It also reruns every earlier check.
- `just lefthook all` and `just doctor`.
- Manual, optional: boot the ISO in a VM with a display and switch to tty2 (`Ctrl+Alt+F2`). The login banner reads `Stackific OS <kernel> (tty2)`.
