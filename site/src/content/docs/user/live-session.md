---
title: Live session
description: What the live ISO starts, and how to switch to KDE Plasma or GNOME at the login screen.
sidebar:
  order: 1
---

The live ISO logs you in as the user `live` with no prompt, and starts MangoWM with Noctalia. In MangoWM, `Alt+Return` opens a terminal and `Alt+space` opens the app launcher.

## Switch to the other desktop

Each image has a second desktop: KDE Plasma on the Arch images, GNOME on the Fedora images.

1. Log out: run `loginctl terminate-user live` in a terminal.
2. At the login screen, pick the user `live` and the session (Plasma or GNOME) in the list below the password field.
3. Type any character as the password, and press Enter. `live` has no password, but the login screen does not accept an empty field.
