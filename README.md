```text
                         ▄
                          █▄
                           ██
                            ██▄
                             ███
                              ▀██▄
                               ▀███
                   ▄▄▄▄▄▄▄▄▄▄▄▄▄████▄
▄▄▄▄▄█████████████████████████████████
```

# Stackific OS

**An Open Prompt: the most vibe-codable OS foundation for making your own Linux.**

[Download](https://github.com/stackific/os/releases/latest) · [Docs](https://os.stackific.com/docs/) · [Prompt Trail](https://os.stackific.com/docs/prompt-trail/) · [Make your own](docs/setup.md)

Stackific OS is a Linux distribution built in the open from prompts. Every image, package, default and test started as a written prompt with the result it had to meet, and every one of those prompts is published in the [Prompt Trail](https://os.stackific.com/docs/prompt-trail/). Read how it was made, fork it, change a prompt, and let a coding agent build your own Linux on the same foundation.

## Four images, one version

Every release ships four live ISOs with the same branding, defaults and live session:

| ISO | Built from | Desktops |
| --- | --- | --- |
| `stackific-<version>-arch-x86_64.iso` | Based on Arch Linux | MangoWM with Noctalia, KDE Plasma |
| `stackific-<version>-arch-aarch64.iso` | Based on Arch Linux ARM | MangoWM with Noctalia, KDE Plasma |
| `stackific-<version>-fedora-x86_64.iso` | Fedora Remix | MangoWM with Noctalia, GNOME |
| `stackific-<version>-fedora-aarch64.iso` | Fedora Remix | MangoWM with Noctalia, GNOME |

Each one boots from a USB stick in UEFI mode, logs in to the MangoWM desktop with the Noctalia shell, and lets you switch to the second desktop.

## Status

Version 0.1.0 is a live preview: the four images boot to the desktop, and nothing is written to your disk. They are unsigned, and there is no installer yet. Coming next: an installer that puts Stackific OS alongside another OS or on a whole drive, with Limine and btrfs snapshots; NVIDIA's open driver; signed releases.

Try it: check the [system requirements](https://os.stackific.com/docs/user/system-requirements/), then follow [Installation](https://os.stackific.com/docs/user/installation/).

## Make your own

The repo is the foundation. [AGENTS.md](AGENTS.md) holds the rules every change follows, and [docs/setup.md](docs/setup.md) sets up a machine to build and test the images. Distro work runs in two steps: `/s-task` turns an ask into a self-contained task, and `/s-build` builds it, reviews it and writes its Prompt Trail entry.

## Trademarks

The Fedora Remix images contain modified Fedora® packages and software that the Fedora Project does not provide. They are not provided or supported by the Fedora Project. Official, unmodified Fedora releases are available at https://fedoraproject.org/.

Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat.

Fedora and Fedora Remix are trademarks or registered trademarks of Red Hat, Inc., or its subsidiaries in the United States and other countries.
