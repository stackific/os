**Live preview.** Each ISO boots from a USB stick to MangoWM with Noctalia, with a second desktop to switch to. There is no installer yet: nothing is written to your disk, and changes are gone after a reboot.

**Unsigned.** `SHA256SUMS` shows that a download is intact, not who built it. Signed releases come later. Check an ISO in the folder that holds it and `SHA256SUMS`:

```sh
sha256sum -c SHA256SUMS --ignore-missing
```

| ISO | Built from | Second desktop |
| --- | --- | --- |
| `-arch-x86_64.iso` | Based on Arch Linux | KDE Plasma |
| `-arch-aarch64.iso` | Based on Arch Linux ARM | KDE Plasma |
| `-fedora-x86_64.iso` | Fedora Remix | GNOME |
| `-fedora-aarch64.iso` | Fedora Remix | GNOME |

The images boot in UEFI mode only, with Secure Boot off. NVIDIA's driver is not in them yet. How to download, check, write and boot an image: https://os.stackific.com/docs/user/installation/

Every change, one entry per task: https://os.stackific.com/docs/prompt-trail/

The Fedora Remix images contain modified Fedora® packages and software that the Fedora Project does not provide. They are not provided or supported by the Fedora Project. Official, unmodified Fedora releases are available at https://fedoraproject.org/.

Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat.

Fedora and Fedora Remix are trademarks or registered trademarks of Red Hat, Inc., or its subsidiaries in the United States and other countries.
