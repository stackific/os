---
title: A Secret Service in every session, unlocked at login
description: oo7 owns org.freedesktop.secrets in MangoWM, Plasma and GNOME on both bases, starts with each user's systemd manager, and the login password unlocks its keyring through pam_oo7.
sidebar:
  order: 17
  label: 017 A Secret Service in every session
pr: https://github.com/stackific/os/pull/17
task: dev/tasks/017-secret-service-oo7.md
---

## Goal

oo7 owns `org.freedesktop.secrets` in every session on both bases, started by the user's systemd manager and unlocked by the login password; the boot and install tests on both x86_64 images prove it.

## Context

MangoWM sessions had no Secret Service on either base, so browsers, git credential helpers and other libsecret apps could not store passwords. Arch had KWallet for Plasma and Fedora had oo7 for GNOME, neither started in MangoWM. Fedora 45 made oo7 its default Secret Service on every desktop, and this step does the same on both bases.

## Expectations

Must:

- In the live MangoWM session on both x86_64 images, `org.freedesktop.secrets` is owned by `oo7-daemon` (`busctl --user status`).
- On both installed systems, after `tester` logs in through the greeter with the password, `secret-tool store` then `secret-tool lookup` returns the stored value without a prompt, and the keyring file is under `~/.local/share/keyrings/`.
- On Arch, Plasma still starts, and the Secret Service in it is oo7.
- Both packages build (`just arch packages`, `just fedora packages`); every existing boot and install test passes.

Must not:

- patch greetd's or PAM's own files, or add gnome-keyring;
- change how the live user logs in;
- touch files outside the Changes list.

## Prompt

```text title="Prompt" wrap
Give every desktop session on all images a Secret Service (org.freedesktop.secrets), so browsers, git credential helpers and other libsecret apps can store passwords: oo7 on both bases, in MangoWM, Plasma and GNOME alike, as Fedora 45 itself chose. Start oo7-daemon with each user's systemd manager, from stackific-desktop, and unlock the user's keyring at login through oo7's PAM module in greetd's stack. On Fedora greetd's own PAM file already calls pam_oo7; on Arch give greetd a PAM service of our own with the same lines, selected in greetd's config. A session with no password (the live user) keeps its keyring locked; make sure that cannot stall logout or shutdown (Noctalia waits on the prompt it asked for under Arch's oo7 0.6, so give its unit a short stop timeout). Prove it in the boot tests (the live session has a Secret Service) and the installed-system tests (after a password login through the greeter, a secret stored with secret-tool reads back, and a keyring file exists, wherever the oo7 version keeps it), and say in the docs that changing the login password later does not change the keyring's.
```

## Outcome

`stackific-desktop` depends on oo7 on both bases (Arch `oo7`, Fedora `oo7-daemon` and `pam_oo7`) and links `oo7-daemon.service` into `default.target.wants`, so each user's systemd manager starts it before any session. On Fedora, greetd's own PAM file already calls `pam_oo7`. On Arch, `stackific-desktop` ships `/etc/pam.d/stackific-greetd`, which is greetd's lines plus `pam_oo7`, and the image's greetd config selects it with `[general] service`. The login password unlocks the keyring, or creates it on the first login. In Plasma on Arch, oo7 holds the name; KDE apps keep the KWallet API.

The live user logs in with no password, so its keyring stays locked. Under Arch's oo7 0.6, Noctalia then waits on a prompt nothing can show and hangs on stop, so its unit now has a 5-second stop timeout.

### Screenshots

No UI.

## Review

- **Round 1** found two problems, both from the test runs, and both are fixed:
  - **Must fix:** on Arch, Noctalia's hang on stop in a session with no password delayed logout and shutdown by 90 seconds. Fixed with the stop timeout.
  - **Should fix:** the keyring check expected one folder, but Fedora's oo7 0.7 uses `keyrings/v1/`. The check now searches the whole keyrings folder.
- **Round 2** found nothing.

## Manual changes

None.

## Result

- `just arch packages`, `just arch iso` (1552 MiB), `just arch test` and `just arch test-install` pass:
  - the live session's Secret Service is `oo7-daemon`;
  - oo7 owns it in Plasma;
  - after a greeter login with the password, `secret-tool` stores and reads back a secret, and a keyring file exists.
- `just fedora packages`, `just fedora iso` (1497 MiB), `just fedora test` and `just fedora test-install` pass the same checks, the last after the GNOME login through the greeter.
- `just lefthook all` passes.
- Docs: `docs/arch-iso.md`, `docs/fedora-iso.md`, `site/src/content/docs/user/installation.md`, `release/notes.md`, `AGENTS.md`.

## Deviations

- `desktop/systemd/user/stackific-noctalia.service` gets `TimeoutStopSec=5s`. The task did not foresee Noctalia hanging on a pending prompt; the task's Changes list was updated.
- The keyring check finds any `*.keyring` under `~/.local/share/keyrings/`, because Fedora's oo7 0.7 keeps it in `v1/`.
- oo7-daemon also starts for the greeter's own user, because it is wanted by every user's manager. It holds nothing there.
