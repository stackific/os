# Apps offered outside the image

`apps/` holds what installs apps that the images do not carry, packaged as `stackific-apps` (`arch/packages/stackific-apps/PKGBUILD`, `fedora/packages/stackific-apps/stackific-apps.spec`, the same files on both bases). Today that is the creative apps (task 025): the 13 apps of github.com/storytold (PhotoCraft, VectorCraft, FilmCraft, LightCraft, PdfCraft, EffectCraft, DesignCraft, FigCraft, WordCraft, SoundCraft, DeckCraft, CADCraft, GridCraft). The user page is `site/src/content/docs/user/creative-apps.md`.

## How it works

- The launcher `stackific-creative-apps.desktop`, "Install creative apps", runs `foot --hold … run0 /usr/lib/stackific/install-creative-apps`, as the installer's launcher does. run0 asks for the password through polkit; nothing skips it.
- The script reads the pins in `/usr/share/stackific/creative-apps.sha256`, keeps the lines for this base's format (`.pkg.tar.zst` where `pacman` exists, else `.rpm`) and `uname -m`, and downloads each package and its signature from `https://github.com/storytold/<app>/releases/download/v<version>/` into a fresh folder in `/var/tmp`.
- Every package must pass both checks before anything is installed: its signature (`.sig` for `.pkg.tar.zst`, `.asc` for `.rpm`) with `gpgv` against upstream's key in `/usr/share/stackific/creative-apps-key.pgp`, dearmored at run time (`gpgv` 2.4 reads only binary keyrings), and its SHA-256 with `sha256sum --check --strict` against the pin. Then `pacman -U --needed --noconfirm` or `dnf install -y` installs all 13; their dependencies come from the base's repositories.
- The signatures stay in a subfolder, because `pacman -U` checks a `<file>.sig` next to a package against pacman's own keyring (Arch's `LocalFileSigLevel = Optional`), which does not hold upstream's key.
- It needs 5 GiB free in `/var/tmp` (about 1.3 GiB of downloads and 3 GiB installed), so the live session, whose overlay is small, stops with a message.
- Why glue: no packaged tool installs pinned, signed release assets from GitHub, and the apps must never be redistributed by us (operator 2026-11-08): they are in no image, repository or release of ours.

## Brand terms

The apps' code is MIT or Apache-2.0, but the ArtCraft name, wordmark and logos are trademarks allowed only in the apps' own official builds (`docs/brand/LICENSE-brand.txt` in each app's repository). So nothing of ours carries the ArtCraft name or a mark in its name, icon or listing: the launcher is "Install creative apps" with the generic `system-software-install` icon, and the README and marketing pages never name ArtCraft. The user page names the apps in plain text. The pins file and the key keep upstream's words, unchanged.

## Test

```sh
just apps test   # the script against a fake signed release of two apps, with a fake pacman
```

The recipe makes a throwaway OpenPGP key, two fake apps with signed packages and their pins, and serves them as `file://` URLs (`STACKIFIC_APPS_URL`, `STACKIFIC_APPS_SHARE`). The good set reaches the fake pacman once, with both packages and no signature beside them; a tampered package (`BAD signature`) and a re-signed one with another hash (`FAILED`) stop before pacman. The boot tests check that the launcher and the script are on every image and install, and that no app is.

## Updating the pins

All 13 releases are signed by one key, "ArtCraft Linux Release Signing Key", primary `2B76741B414AAD6CD33E631213A005C0B9D7C3ED`, subkey `568DC1F68F8FAECC797F3C7B006DAF6BB0EF334E`; each release also attaches it as `release-signing-key.asc` (byte-identical in all 13, 2026-11-08). To move to newer releases:

1. For each app, `gh release download -R storytold/<app> <tag> -p SHA256SUMS.txt -p SHA256SUMS.txt.asc -p release-signing-key.asc`.
2. Check the key is unchanged (`cmp release-signing-key.asc apps/creative-apps-key.pgp`), and every `SHA256SUMS.txt` with `gpg --dearmor <apps/creative-apps-key.pgp >key.gpg` and `gpgv --keyring ./key.gpg SHA256SUMS.txt.asc SHA256SUMS.txt`.
3. Replace the app's four lines in `apps/creative-apps.sha256` with its `linux-{x86_64,aarch64}.{pkg.tar.zst,rpm}` lines, unchanged.
4. Update the sha256 in the PKGBUILD and bump `pkgrel` (and `Release` in the spec).

A changed key needs a reason from upstream before it replaces `creative-apps-key.pgp`.

## Later

- Hide the "Install creative apps" launcher in the live session, where it cannot work: the root lives in RAM, and `run0` may refuse the live user's empty password (task 025).
