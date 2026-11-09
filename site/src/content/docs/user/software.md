---
title: Installing software
description: The package manager on each image, Shelly or GNOME Software, and the password it asks for.
sidebar:
  order: 5
---

Every image has a graphical package manager for installing, updating and removing software, live and installed.

| Image                             | Package manager | What it manages                                      |
| --------------------------------- | --------------- | ---------------------------------------------------- |
| Arch Linux (x86_64)               | Shelly          | Arch's repositories, the AUR and Flatpak apps        |
| Arch Linux ARM (aarch64)          | none yet        | use `pacman` in a terminal                           |
| Fedora Remix (x86_64 and aarch64) | GNOME Software  | Fedora's packages, Flatpak apps and firmware updates |

Open it from the app launcher: "Shelly" on Arch Linux, "Software" on the Fedora Remix images.

Installing or removing software changes the whole system, so the package manager asks for your password first. After you enter it, further installs and updates for a few minutes go through without asking again. Trusting a new software repository asks for the password every time.

On the Arch Linux ARM image, Shelly is not available yet: it needs a newer Zig compiler than Arch Linux ARM ships. Install and update software with `sudo pacman -S <package>` and `sudo pacman -Syu` in a terminal.

The AUR holds packages built from recipes that anyone can upload, and Arch Linux does not review them. Read a package's build recipe in Shelly before you install it from the AUR.

The Fedora Remix images are not provided or supported by the Fedora Project; official Fedora software is available at https://fedoraproject.org/.
