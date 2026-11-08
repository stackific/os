# 017: A Secret Service in every session: oo7, unlocked at login

- [x] Done

Base: both

## Ask

```text title="Prompt" wrap
Give every desktop session on all images a Secret Service (org.freedesktop.secrets), so browsers, git credential helpers and other libsecret apps can store passwords: oo7 on both bases, in MangoWM, Plasma and GNOME alike, as Fedora 45 itself chose. Start oo7-daemon with each user's systemd manager, from stackific-desktop, and unlock the user's keyring at login through oo7's PAM module in greetd's stack. On Fedora greetd's own PAM file already calls pam_oo7; on Arch give greetd a PAM service of our own with the same lines. Prove it in the boot tests (the live session has a Secret Service) and the installed-system tests (after a password login through the greeter, a secret stored with secret-tool reads back, and the keyring file exists), and say in the docs that changing the login password later does not change the keyring's or the disk's password.
```

## Goal
oo7 owns `org.freedesktop.secrets` in every session on both bases, started by the user's systemd manager and unlocked by the login password; `just arch test`, `just fedora test` and both `test-install` recipes prove it.

## Context
What exists on `main` (checked 2026-11-08; the operator's SPEC decision 18 checked the live ISOs the same day):
- No MangoWM session on either base has a Secret Service: `org.freedesktop.secrets` is "not activatable". Arch has `kwallet` 6.31.0 (Plasma's), whose `ksecretd` takes the name only once started; Fedora has `oo7-daemon` and `oo7-portal` 0.7.0~beta^1.e91b98a (from GNOME), not started in MangoWM. No test checks it.
- `stackific-desktop` (`arch/packages/stackific-desktop/PKGBUILD`, `fedora/packages/stackific-desktop/stackific-desktop.spec`) already ships user units (`stackific-noctalia.service`, wanted by `mango-session.target`).
- greetd's config is the image's own file: `arch/iso/airootfs/etc/greetd/config.toml`, `fedora/iso/root/etc/greetd/config.toml` (`[terminal]`, `[initial_session]` for `live`, `[default_session]` with noctalia-greeter). The installer copies it and drops `[initial_session]`.
- Tests: `tests/vm/arch.sh` and `tests/vm/fedora.sh` check the live session; `tests/vm/installed.sh` logs `tester` into MangoWM through an initial session and, on Fedora, into GNOME through the greeter with a typed password; on Arch into Plasma through an initial session.

Upstream facts (checked 2026-11-08):
- **Fedora 45 made oo7 the default Secret Service on all desktops**, replacing GNOME Keyring and KWallet and migrating their data (https://fedoraproject.org/wiki/Changes/oo7_Secrets_Service_Provider; devel-announce https://lists.fedoraproject.org/archives/list/devel-announce@lists.fedoraproject.org/message/MFGKLUQVOSQYZE6TT65JBSKWESFYYCR5/). Users report that `oo7-daemon.service` needed enabling by hand (https://discussion.fedoraproject.org/t/issues-with-credentials-on-fedora-45-beta/202421).
- **Arch `extra/oo7` 0.6.0-5** (2026-11-01 archive): `Provides org.freedesktop.secrets`, `Conflicts gnome-keyring`; ships `/usr/lib/oo7-daemon`, `/usr/lib/security/pam_oo7.so`, `/usr/lib/systemd/user/oo7-daemon.service` (`Type=simple`, `[Install] WantedBy=default.target`, not enabled by the package), the alias `dbus-org.freedesktop.secrets.service`, and `/usr/share/dbus-1/services/org.freedesktop.secrets.service` (`SystemdService=dbus-org.freedesktop.secrets.service`). kwallet 6.31.0 ships no D-Bus activation file for `org.freedesktop.secrets` (only `org.kde.kwalletd6` and `org.kde.secretservicecompat`), so the two packages do not conflict.
- **Fedora 45 `oo7-daemon` 0.7.0~beta^1.e91b98a-1.fc45**: `/usr/libexec/oo7-daemon`, `/usr/lib/systemd/user/oo7-daemon.service` (`Type=dbus`, `BusName=org.freedesktop.secrets`, `WantedBy=default.target`) and its alias, **no D-Bus activation file**; `pam_oo7` (separate package, requires `oo7-daemon`) ships `/usr/lib64/security/pam_oo7.so`.
- **pam_oo7** (`/usr/share/doc/pam_oo7/README.md` in Fedora's package): in `auth` it stashes the password, in `session` it sends it over `$XDG_RUNTIME_DIR` to the running daemon, which unlocks (or creates) the login keyring; `auto_start` starts the daemon when its socket is missing; in `password` it re-encrypts the keyring on a password change. Use `optional`.
- **greetd's PAM files**: Fedora's `greetd` 0.10.3 `/usr/lib/pam.d/greetd` already has `-auth optional pam_oo7.so` and `-session optional pam_oo7.so auto_start` (after `session include system-auth`). Arch's `greetd` 0.10.3-2 `/etc/pam.d/greetd` (a `backup` file) has only `pam_securetty`, `pam_nologin` and `include system-local-login` lines.
- **greetd 0.10.3 config** (`greetd/src/config/mod.rs`, https://git.sr.ht/~kennylevinsen/greetd/blob/0.10.3/greetd/src/config/mod.rs): `[general] service` names the PAM service for user sessions (default `greetd`), which `[initial_session]` also uses unless it sets its own; the greeter uses `[default_session] service` (default `greetd-greeter`).

## Inputs
SPEC decision 18 (operator 2026-11-08): oo7 on both bases, started by a `stackific-desktop` user unit, unlocked at login by its PAM module; in every session, Plasma and GNOME included (operator 2026-11-08, as Fedora 45 does); its own task after 016 (operator 2026-11-08). Nothing is copied into the tree.

## Design
- `stackific-desktop` on both bases depends on oo7 (Arch `oo7`; Fedora `oo7-daemon` and `pam_oo7`) and ships the symlink `/usr/lib/systemd/user/default.target.wants/oo7-daemon.service`, so every user manager starts it before any session; pam_oo7's `session` step finds it running.
- Arch: `stackific-desktop` ships `/etc/pam.d/stackific-greetd`, Arch's greetd lines plus `-auth optional pam_oo7.so` and `-session optional pam_oo7.so auto_start`; the Arch image's greetd config sets `[general] service = "stackific-greetd"`. Fedora keeps greetd's own PAM file.
- Plasma on Arch: oo7 owns the Secret Service; KDE apps keep the KWallet API through kwalletd6.

## Changes
- `arch/packages/stackific-desktop/PKGBUILD` and the Arch-only `stackific-greetd` PAM file beside it; `fedora/packages/stackific-desktop/stackific-desktop.spec`.
- `arch/iso/airootfs/etc/greetd/config.toml`.
- `desktop/systemd/user/stackific-noctalia.service`: a short stop timeout, if Noctalia waits on a Secret Service prompt at logout.
- `tests/vm/arch.sh`, `tests/vm/fedora.sh`: the live session's Secret Service.
- `tests/vm/installed.sh`: a greeter login with the password, then `secret-tool store` and `lookup`.
- `docs/arch-iso.md`, `docs/fedora-iso.md`, `site/src/content/docs/user/installation.md`, `AGENTS.md`, `release/notes.md`.
- The prompt-trail entry and review record.

## Acceptance criteria

Must:
- In the live MangoWM session on both x86_64 images, `org.freedesktop.secrets` is owned by `oo7-daemon` (`busctl --user status`).
- On both installed systems, after `tester` logs in through the greeter with the password, `secret-tool store` then `secret-tool lookup` returns the stored value without a prompt, and the keyring file is under `~/.local/share/keyrings/`.
- On Arch, Plasma still starts, and the Secret Service in it is oo7.
- Both packages build (`just arch packages`, `just fedora packages`); every existing boot and install test passes.

Must not:
- patch greetd's or PAM's own files, or add gnome-keyring;
- change how the live user logs in;
- touch files outside the Changes list.

## Tests
- `just arch packages`, `just arch iso`, `just arch test`, `just arch test-install`.
- `just fedora packages`, `just fedora iso`, `just fedora test`, `just fedora test-install`.
- `just lefthook all`.
