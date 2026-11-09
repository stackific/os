---
title: Regular hardware on both x86_64 images, and R2 for an ISO over 2 GiB
description: Matched firmware, Thunderbolt, fingerprint readers and MIPI webcams on the Arch and Fedora x86_64 images, installed everywhere with no detection, and Cloudflare R2 for the Arch ISO now over GitHub's 2 GiB cap.
sidebar:
  order: 21
  label: 021 Regular hardware on both x86_64 images
pr: https://github.com/stackific/os/pull/21
task: dev/tasks/021-hardware-parity.md
---

## Goal

Both x86_64 images carry the same firmware families plus bolt, fprintd and the libcamera stack, checked by sentinel files and packages in the boot tests. An ISO over 2 GiB is published through Cloudflare R2 and linked from its release.

## Context

Tasks 019 and 020 brought NVIDIA to both x86_64 images. The firmware sets still differed between the bases, and Thunderbolt authorisation, fingerprint readers and MIPI webcams had no packages. The earlier design installed some of this on detection with chwd; with everything installed everywhere, no detection is needed. The Arch ISO was 7 MiB under GitHub's 2 GiB file limit, so this step also wires the R2 path planned in `AGENTS.md`.

## Expectations

Must:

- `just arch iso` and `just fedora iso` build and print their sizes; the task's entry reports both, and the Fedora aarch64 size from the `images` run after merge.
- On both x86_64 live ISOs (`just arch test`, `just fedora test`): the added packages are installed (`pacman -Q` / `rpm -q`, one line each); these sentinel files exist under `/usr/lib/firmware` (any compression suffix; Fedora's `alsa-firmware` installs under `/lib/firmware`, the same directory): `regulatory.db`, `mrvl/pcie8897_uapsta.bin`, `intel/sof` or `intel/sof-ipc4` (a directory), `ctefx.bin`, `dvb-usb-dib0700-1.20.fw`, `yamaha/yss225_registers.bin`, `ea/3g_asic.fw`; `mrvl/prestera` does not exist; `boltctl list` exits 0; `busctl --system introspect net.reactivated.Fprint /net/reactivated/Fprint/Manager` exits 0; the PipeWire libcamera plugin exists (`/usr/lib*/spa-0.2/libcamera/libspa-libcamera.so`); every existing check still passes.
- No PAM file changes: `grep -rl pam_fprintd /etc/pam.d /usr/lib/pam.d` prints nothing on both images.
- `just release upload` puts an ISO of at most 2 GiB on the GitHub release as before, and a larger one on R2 with `<name>.iso.url` on the release; `just release publish`'s `SHA256SUMS` lists both kinds. Proven by the `images` run after merge (the Arch x86_64 test build), whose release shows `stackific-0.0.0-pr.<n>-arch-x86_64.iso.url`, and by downloading that URL and checking it with `sha256sum -c`.
- `just doctor` prints an `rclone:` line with the pinned version; `just lefthook all` passes.
- Every existing boot and install test passes on both x86_64 images.

Must not:

- install `fprintd-pam` on Fedora or wire `pam_fprintd` anywhere;
- add `broadcom-wl-dkms`, `akmod-wl`, a compiler or kernel headers, or chwd;
- add `linux-firmware-qcom`, `mrvlprestera-firmware` or server NIC firmware;
- print or commit any R2 secret;
- change Arch aarch64's package list;
- touch files outside the Changes list.

## Prompt

```text title="Prompt" wrap
Bring both x86_64 images to the same regular-hardware support, live and installed. Match the firmware sets across Arch and Fedora: Arch adds linux-firmware-marvell (without Marvell's Prestera switch firmware, a datacenter part we leave out), sof-firmware, alsa-firmware and wireless-regdb; Fedora adds alsa-firmware and dvb-firmware, which cover what Arch's linux-firmware-other and alsa-firmware carry. Both get bolt for Thunderbolt and USB4, fprintd for fingerprint readers (PAM untouched: nothing in login, sudo or polkit changes), and the open MIPI webcam stack: libcamera with its IPA modules and PipeWire's libcamera and V4L2 plugins. Install everything on every image and every install, so nothing depends on detection and no detection tool is needed: no chwd. Broadcom STA (wl) needs a compiler on both bases, so it is not shipped; the user docs give the online steps for the few machines that need it. The Arch x86_64 ISO goes over GitHub's 2 GiB per-file cap with these packages, so wire the Cloudflare R2 path AGENTS.md already describes: an ISO over 2 GiB goes to R2 and the release links it. R2 takes three repository secrets, the access key, the secret key and the bucket's endpoint URL, and the public URLs are https://cdn.stackific.com/os/releases/…. Keep bolt's packaged polkit rule as upstream ships it. Fedora aarch64 takes the Fedora list as is (one config.kiwi); Arch aarch64's list stays for task 022.
```

## Outcome

- **Firmware parity.** Arch x86_64 adds `linux-firmware-marvell`, `sof-firmware`, `alsa-firmware` and `wireless-regdb`; Fedora adds `alsa-firmware` and `dvb-firmware`. Comparing both bases' firmware file lists showed these close the only gaps, apart from server parts both leave out. Marvell's Prestera switch firmware is 71 of the Marvell package's 78 MiB, so `NoExtract` in `arch/iso/pacman.conf` keeps it out of the Arch image.
- **Devices.** Both x86_64 images now have `bolt` (Thunderbolt and USB4; its upstream polkit rule kept), `fprintd` (fingerprint readers, with no PAM wiring of ours) and libcamera with PipeWire's libcamera and V4L2 plugins (MIPI webcams). Everything is on every image and install, so no detection step and no chwd. The Fedora list is shared, so Fedora aarch64 gets it too.
- **Broadcom STA** needs a compiler on both bases, so it is not on the images. The user page lists the chips that need it and the online steps.
- **R2.** The Arch x86_64 ISO is now 2069 MiB, over GitHub's 2 GiB file limit. `just release upload` sends such an ISO with the pinned rclone (1.75.2, new in `just setup`) to `os/releases/v<version>/<name>.iso` and attaches `<name>.iso.url` with `https://cdn.stackific.com/os/releases/…` instead; `SHA256SUMS` lists it as before. Two repository secrets, `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY`, and the repository variable `R2_ENDPOINT` reach the upload steps of `images.yml` and `release.yml`. The recipe takes the endpoint and the bucket from `R2_ENDPOINT`, and the keys reach rclone only through its environment.

### Screenshots

No UI.

## Review

- **Round 1** found nothing in the change. The test runs before it had found two wrong checks, fixed then. The libcamera plugin check expected one file, but `/usr/lib64` is a link to `/usr/lib` on Arch. The PAM check flagged kscreenlocker's own `kde-fingerprint`; it now leaves out the desktops' fingerprint services. The same build also found a quoting bug in task 020's `fedora/justfile`, fixed on that task's branch.

## Manual changes

None. The operator adds the three R2 secrets before the merge (the task's manual steps 1 and 2).

## Result

- `just arch iso`: 2069 MiB, over the 2048 MiB cap, so the R2 path carries it. `just fedora iso`: 1930 MiB. The Fedora aarch64 size comes from the `images` run after the merge.
- `just arch test` and `just fedora test` pass, with the new checks: the added packages, one sentinel file per firmware family (`regulatory.db`, Marvell's `pcie8897_uapsta.bin`, Intel SOF, `ctefx.bin`, `dvb-usb-dib0700-1.20.fw`, `yamaha/yss225_registers.bin`, `ea/3g_asic.fw`), no Prestera firmware, `boltctl list`, fprintd answering on D-Bus, PipeWire's libcamera plugin, and no PAM stack of ours using `pam_fprintd` (on Fedora also no `fprintd-pam`).
- `just arch test-install` and `just fedora test-install` pass.
- `just rclone check` prints `rclone: ok (1.75.2)`. A local run of `just release upload` with a 2.1 GB sparse ISO reached rclone's S3 call at the endpoint and bucket taken from `R2_ENDPOINT`, and stopped there without uploading to GitHub when the copy failed. The real upload is proven by the first `images` run after the merge, with the operator's secrets and variable.
- `just lefthook all` passes.
- Docs: `docs/hardware.md`, `docs/arch-iso.md`, `docs/fedora-iso.md`, `docs/releases.md`, `docs/setup.md`, `site/src/content/docs/user/system-requirements.md`, `release/notes.md`, `AGENTS.md`.

## Deviations

- The PAM criterion excepts the desktops' own fingerprint services: kscreenlocker's `kde-fingerprint`, so Plasma's lock screen accepts an enrolled finger, and GDM's `gdm-fingerprint`, which never runs. The task file was updated.
- The R2 settings are the operator's three (secrets `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY`, variable `R2_ENDPOINT`) with the fixed public prefix `https://cdn.stackific.com/os/releases/`, not the four names `AGENTS.md` had listed; `AGENTS.md` changed with it.
- rclone is pinned to 1.75.2, not 1.76.0, which was two days old.
