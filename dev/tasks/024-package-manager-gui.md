# 024: A package manager GUI on every image: Shelly on Arch Linux, GNOME Software on Fedora

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Give every image a graphical package manager, live and installed. On Arch Linux, Shelly (Seafoam Labs' GTK4 package manager for the repos, the AUR and Flatpak) built from the AUR's PKGBUILD into our own Arch repo, with its Flatpak backend; on Fedora, GNOME Software, Fedora Workstation's own store. Shelly needs Zig 0.17.0: if it cannot build on Arch Linux ARM, leave it out of the Arch aarch64 image, say so in the docs and list it under Later. Neither may install software without the administrator's password: no polkit rule that skips authentication, so where a package we ship brings such a rule, override it.
```

## Goal
Arch x86_64 carries Shelly and its Flatpak backend from our repo, both Fedora images carry GNOME Software, and every privileged action either one starts asks for the administrator's password.

## Context
What exists (`main` at f9e419e, tasks 021 and 022 merged, 2026-11-08):
- No image has a package manager GUI. `arch/packages/` holds our six packages, built by `just arch packages` with `makepkg --nodeps` in the pinned builder (`arch/Containerfile`), so the builder carries every makedepend; sources found by name next to the PKGBUILD or in `desktop/`, `branding/`, `hardware/`, `installer/`; URL sources are downloaded by makepkg and checked against `sha256sums`. The recipe builds every `arch/packages/*/PKGBUILD` on both architectures.
- `fedora/packages/stackific-desktop/stackific-desktop.spec` (Release 4) ships our Fedora desktop defaults.
- `AGENTS.md`: "Never configure passwordless sudo (`NOPASSWD`) or polkit rules that skip authentication"; bolt's upstream rule is the one operator-approved exception (2026-11-08).

Upstream facts:
- Shelly (AUR `shelly`, https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h=shelly, read 2026-11-08): 3.2.0-1, `arch=('x86_64' 'aarch64')`, GPL-3.0-only, split into `shelly` and `shelly-flatpak-backend`. Source `https://github.com/Seafoam-Labs/Shelly-ALPM/archive/v3.2.0.tar.gz` (sha256 `2217744ade8b05e3b79a854a812c2613de1ae5cc026f8c8aa006117f4667d2bc`, 3.5 MiB) plus four side files from the AUR: `shellybuild.conf` (`4126da20…`, an inert, all-comments config), `com.shellyorg.shelly.desktop` (`4b7cb0e4…`), `com.shellyorg.shelly-notifications.desktop` (`cf0dc0a6…`), `shelly-flatpak-integrate` (`07bca403…`). Makedepends: `libarchive curl sqlite gnupg git pkgconf gtk4 zig=0.17.0 clang gettext flatpak ripgrep go-md2man binutils pacman`. Depends: `pacman gtk4 glib2 sudo tar bash git hicolor-icon-theme dbus glibc libarchive curl sqlite dconf gnupg zstd json-glib`. Its privileged CLI runs through pkexec with its own action `com.shellyorg.shelly.pkexec.cli` (`allow_active` `auth_admin_keep`, the others `auth_admin`): a password, kept a few minutes.
- Shelly's Zig builds fetch seven dependencies by URL (zeit, conch, goose, zig-time, zig-xml, zig-toml, zig-gobject), each pinned by its `.hash` in `build.zig.zon`, which Zig verifies (source tarball, read 2026-11-08); the build needs network, as the installer's `cargo` build does.
- Arch Linux Archive 2026-11-01 (`arch/vars.just` `ala_date`; queried in `localhost/stackific-arch-builder:2026-11-01`, 2026-11-08): `zig` 0.17.0-1, `clang` 23.1.4-1, `go-md2man` 2.0.7-2, `ripgrep` 15.2.0-1, `flatpak` 1:1.18.4-2, `gnome-software` 51.1-1.
- Arch Linux ARM (`http://mirror.archlinuxarm.org/aarch64/extra/extra.db`, 2026-11-08): `zig` 0.16.0-1 only, so Shelly 3.2.0 (`minimum_zig_version = "0.17.0"`) cannot build there.
- Fedora 45 (queried in `localhost/stackific-fedora-builder:45`, 2026-11-08): `gnome-software` 51.1-1.fc45 (2.8 MiB), requiring among others `dnf5daemon-server`, `dnf5daemon-server-polkit`, `fwupd`, `flatpak`, `appstream-data` (16 MiB), `epiphany-runtime`, `libdnf5-plugin-appstream`, `gnome-app-list`. `dnf5daemon-server-polkit` ships `/usr/share/polkit-1/rules.d/org.rpm.dnf.v0.rules`, which returns `polkit.Result.YES` for `org.rpm.dnf.v0.rpm.execute_trusted_transaction` and `org.rpm.dnf.v0.rpm.Repo.confirm_key` when the subject is active, local and in `wheel`: installing signed packages and accepting repository keys without a password. Without it, `dnf5daemon-server`'s `org.rpm.dnf.v0.policy` applies: `execute_trusted_transaction` `auth_admin_keep`, `Repo.confirm_key` `auth_admin` for active sessions.
- polkit reads `/etc/polkit-1/rules.d/` and `/usr/share/polkit-1/rules.d/` "in lexical order based on the basename of each file"; on a tie the `/etc` file only runs first, so a same-named file does not replace the other (polkit(8) of Fedora 45's `polkit`, read 2026-11-08). The first rule that returns a result decides; `pkaction` shows only the `.policy` defaults, never the rules, while `pkcheck --process <pid>` asks the authority with the rules applied and exits 2 when authentication is required.

## Inputs
From the operator's plan, row 024 ("Shelly in our Arch repo (PKGBUILD from the AUR, Zig), GNOME Software on Fedora"), and the operator's design notes: decision 15 makes GNOME Software Shelly's counterpart on Fedora (Fedora Workstation's own store for RPMs, Flatpaks and firmware updates, with no extra toolkit), and question 4 asks whether Shelly builds on Arch Linux ARM. Operator decision (2026-11-08): if it cannot, Arch aarch64 goes without Shelly, the docs saying so. No files copied.

Open decisions (operator to confirm; the recommended answer is built):
1. dnf5daemon's passwordless rule is overridden by `stackific-desktop`'s `/etc/polkit-1/rules.d/10-stackific-dnf5daemon.rules`, which sorts before `org.rpm.dnf.v0.rules` and returns the package's own defaults for its two actions (`execute_trusted_transaction`: `AUTH_ADMIN_KEEP` in an active session, else `AUTH_ADMIN`; `Repo.confirm_key`: `AUTH_ADMIN`), so GNOME Software asks for the password, following `AGENTS.md`'s polkit rule; the alternative is keeping it, as was done for bolt.
2. Shelly's `check()` runs in our build as in the AUR's.

## Changes
- `arch/packages/shelly/PKGBUILD` (new): the AUR's 3.2.0-1, `arch=('x86_64')`, our header comment; with its four side files next to it, byte for byte the AUR's.
- `arch/Containerfile`: Shelly's makedepends in the x86_64 builder.
- `arch/justfile` `packages`: skip a PKGBUILD whose `arch` names neither `any` nor the host's architecture.
- `arch/iso/packages.x86_64`: `shelly`, `shelly-flatpak-backend`.
- `fedora/iso/config.kiwi`: `gnome-software`.
- `fedora/packages/stackific-desktop/stackific-desktop.spec` and `stackific-dnf5daemon.rules` (new, beside it): the rule at `/etc/polkit-1/rules.d/10-stackific-dnf5daemon.rules`; Release 5.
- `tests/vm/arch-x86_64.sh`, `tests/vm/fedora.sh`: the checks below.
- `docs/arch-iso.md`, `docs/fedora-iso.md`, `site/src/content/docs/user/software.md` (new), `AGENTS.md`.
- `site/src/content/docs/prompt-trail/024-package-manager-gui.md`, `dev/reviews/024-package-manager-gui.md`.

## Acceptance criteria

Must:
- `just arch packages` on x86_64 builds `shelly-3.2.0-1-x86_64.pkg.tar.zst` and `shelly-flatpak-backend-3.2.0-1-x86_64.pkg.tar.zst` into `.build/arch/repo/`; on aarch64 it builds the other packages and skips Shelly.
- The Arch x86_64 live image: `pacman -Q shelly shelly-flatpak-backend` lists both, `/usr/bin/shelly-ui` is executable, `com.shellyorg.shelly.desktop` is installed, and `pkcheck --action-id com.shellyorg.shelly.pkexec.cli --process <live's mango>` exits 2 (authentication required).
- Both Fedora live images: `rpm -q gnome-software` succeeds; `pkcheck --process <live's mango>` exits 2 (authentication required) for `org.rpm.dnf.v0.rpm.execute_trusted_transaction` and `org.rpm.dnf.v0.rpm.Repo.confirm_key`, `live` being an active local `wheel` user; `/etc/polkit-1/rules.d/10-stackific-dnf5daemon.rules` is owned by `stackific-desktop`.
- `just lefthook all` passes.

Must not:
- Ship Shelly on Arch aarch64.
- Change Shelly's sources or side files beyond `arch`, `options=('!debug')` (makepkg would otherwise need `debugedit` for a `-debug` package, which our packages never ship) and the header comment.

## Tests
- `just arch packages` on the PC (Shelly's build), `just arch test` on both Arch images and `just fedora test` on both Fedora images through the `images` workflow.
- Manual, on an installed Fedora system (the VM tests cannot click through GNOME Software):
  1. Open GNOME Software, install a small app such as "Calculator".
  2. Expect a password prompt; enter the password; the install finishes.
