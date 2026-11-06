**Live preview.** Each ISO boots from a USB stick to MangoWM, with a second desktop to switch to. The live session writes nothing to your disks, and changes are gone after a reboot. Builds after 0.1.0 have an early installer on the `-arch-x86_64.iso` ("Install Stackific OS", in the app launcher and the boot menu); the drive it installs to cannot start yet, so it is for a spare drive.

**Unsigned.** `SHA256SUMS` shows that a download is intact, not who built it. Signed releases come later. Check an ISO in the folder that holds it and `SHA256SUMS`:

```sh
sha256sum -c SHA256SUMS --ignore-missing
```

| ISO | First desktop | Second desktop |
| --- | --- | --- |
| `-arch-x86_64.iso` | MangoWM | KDE Plasma |
| `-arch-aarch64.iso` | MangoWM | KDE Plasma |
| `-fedora-x86_64.iso` | MangoWM | GNOME |
| `-fedora-aarch64.iso` | MangoWM | GNOME |

The images boot in UEFI mode only, with Secure Boot off. NVIDIA's driver is not in them yet. How to download, check, write and boot an image: https://os.stackific.com/docs/user/installation/

Every change, one entry per task: https://os.stackific.com/docs/prompt-trail/

<sub>Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat.</sub>
