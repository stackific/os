---
title: Apps
description: The apps every Stackific OS image ships, and how to get more from Flathub with Bazaar.
sidebar:
  order: 5
---

Every Stackific OS image ships the same everyday apps, in the live session and on the installed system. Press Super+Space in the MangoWM desktop to open the launcher, type a few letters of the app's name, and press Enter.

| App         | What it does                                                                                     |
| ----------- | ------------------------------------------------------------------------------------------------ |
| foot        | A light terminal. Alt+Return opens it in MangoWM.                                                |
| kitty       | A terminal with tabs, splits and images in the terminal.                                         |
| btop        | A system monitor: CPU, memory, disks, network and processes. It runs in a terminal.              |
| Thunar      | A file manager, with Trash and USB drives. Right-click an archive to unpack it with File Roller. |
| File Roller | Opens, makes and unpacks archives (zip, tar, 7z and others).                                     |
| Disks       | Partitions, formats and checks drives.                                                           |
| Bazaar      | An app store for Flatpak apps from Flathub.                                                      |

The second desktops bring their own apps too: Konsole and Dolphin in Plasma on the Arch Linux images, Ptyxis and Files in GNOME on the Fedora Remix images.

## More apps from Flathub

Flathub is set up as the system's app source on every image. Open Bazaar, search for an app, and click Install; the app then shows in the launcher. Bazaar needs a network connection to browse and install. Apps installed in the live session are gone after a restart, like everything else there.

To install from a terminal instead:

```sh
flatpak install flathub <app-id>
```

Flatpak apps update with `flatpak update`, or from Bazaar.
