# 021: Regular hardware on both x86_64 images: matched firmware, Thunderbolt, fingerprint readers and MIPI webcams

- [x] Done

Base: both

## Ask

```text title="Prompt" wrap
Bring both x86_64 images to the same regular-hardware support, live and installed. Match the firmware sets across Arch and Fedora: Arch adds linux-firmware-marvell (without Marvell's Prestera switch firmware, a datacenter part we leave out), sof-firmware, alsa-firmware and wireless-regdb; Fedora adds alsa-firmware and dvb-firmware, which cover what Arch's linux-firmware-other and alsa-firmware carry. Both get bolt for Thunderbolt and USB4, fprintd for fingerprint readers (PAM untouched: nothing in login, sudo or polkit changes), and the open MIPI webcam stack: libcamera with its IPA modules and PipeWire's libcamera and V4L2 plugins. Install everything on every image and every install, so nothing depends on detection and no detection tool is needed: no chwd. Broadcom STA (wl) needs a compiler on both bases, so it is not shipped; the user docs give the online steps for the few machines that need it. The Arch x86_64 ISO goes over GitHub's 2 GiB per-file cap with these packages, so wire the Cloudflare R2 path AGENTS.md already describes: an ISO over 2 GiB goes to R2 and the release links it. R2 takes three repository secrets, the access key, the secret key and the bucket's endpoint URL, and the public URLs are https://cdn.stackific.com/os/releases/…. Keep bolt's packaged polkit rule as upstream ships it. Fedora aarch64 takes the Fedora list as is (one config.kiwi); Arch aarch64's list stays for task 022.
```

## Goal
Both x86_64 images carry the same firmware families plus bolt, fprintd and the libcamera stack, checked by sentinel files and packages in the boot tests; an ISO over 2 GiB is published through Cloudflare R2 and linked from its release.

## Context
What exists (branch `task/019-nvidia-arch`, checked 2026-11-08):
- Arch x86_64 (`arch/iso/packages.x86_64`): `linux-firmware` (the meta package: `-amd`, `-amdgpu`, `-atheros`, `-broadcom`, `-cirrus`, `-intel`, `-mediatek`, `-nvidia`, `-other`, `-radeon`, `-realtek`, `-ti`, `-whence`), `amd-ucode`, `intel-ucode`. Its manifest (`.build/arch/x86_64/*.packages.txt`) has no `linux-firmware-marvell`, `sof-firmware`, `alsa-firmware`, `wireless-regdb`, `bolt`, `fprintd`, `libcamera` or PipeWire's libcamera and V4L2 plugins; it has `iio-sensor-proxy` and `alsa-ucm-conf`. The ISO is 2041 MiB (task 019), 7 MiB under GitHub's 2 GiB cap.
- Fedora (`fedora/iso/config.kiwi`, both architectures): `linux-firmware` plus `alsa-sof-firmware`, `amd-gpu-firmware`, `atheros-firmware`, `brcmfmac-firmware`, `cirrus-audio-firmware`, `intel-audio-firmware`, `intel-gpu-firmware`, `intel-vsc-firmware`, `iwlegacy-firmware`, `iwlwifi-dvm-firmware`, `iwlwifi-mvm-firmware`, `libertas-firmware`, `mt7xxx-firmware`, `nvidia-gpu-firmware`, `nxpwireless-firmware`, `qcom-wwan-firmware`, `realtek-firmware`, `tiwilink-firmware`, `intel-npu-firmware`, x86 microcode; the manifest also has `iwlwifi-mld-firmware`, `iwlbluetooth-firmware` and `wireless-regdb`, and no `alsa-firmware`, `dvb-firmware`, `bolt`, `fprintd` or libcamera. The x86_64 ISO is 1502 MiB (before task 020).
- Releases: `release/justfile` `upload` sends each ISO, its manifest and its `.iso.sha256` with `gh release upload`; `publish` builds `SHA256SUMS` from the `.iso.sha256` files. `images.yml` and `release.yml` call them. A file over 2 GiB fails the upload, so the merge of this task would break the `images` workflow's test release. No R2 secret or variable exists in the repository (`gh secret list`, `gh variable list`: empty, 2026-11-08).
- Task 019's design: every image and install carries all GPU packages; `stackific-gpu-generator` picks the module at boot; no chwd.

Upstream facts (Arch Linux Archive 2026-11-01 in `localhost/stackific-arch-builder:2026-11-01`; Fedora 45 repos in `localhost/stackific-fedora-builder:45`; both read 2026-11-08):
- Arch sizes (download / installed): `linux-firmware-marvell` 20260916-1 77.9 / 79.5 MiB, of which 71.3 MiB is `usr/lib/firmware/mrvl/prestera/` (Marvell Prestera Ethernet switch ASICs) and the rest Wi-Fi and Bluetooth (`mrvl/*8897*`, `*8997*`, `mwlwifi/`); `sof-firmware` 2026.09.2-1 2.4 / 55.1 MiB; `alsa-firmware` 1.2.4-4 3.3 / 14.1 MiB; `wireless-regdb` 2026.09.03-1 12 KiB (depends on `iw`, 108 KiB); `bolt` 0.9.11-3 156 KiB; `fprintd` 1.94.5-2 151 KiB with `libfprint` 427 KiB and `libgusb`; `libcamera` 0.7.2-4 704 KiB, `libcamera-ipa` 2.1 MiB, `libyuv`; `pipewire-libcamera` and `pipewire-v4l2` 1:1.6.10-1, 62 and 43 KiB. Arch's firmware files are zstd-compressed, so squashfs does not shrink them further.
- Fedora sizes (download): `alsa-firmware` 1.2.4-18.fc45 3.5 MiB; `dvb-firmware` 0.8 MiB; `bolt` 0.9.11-2.fc45 196 KiB; `fprintd` 1.94.5-6.fc45 186 KiB, `libfprint` 1.94.100-1.fc45 399 KiB; `libcamera` 0.7.2-5.fc45 764 KiB, `libcamera-ipa` 188 KiB, `pipewire-plugin-libcamera` and `pipewire-v4l2` 1.6.11-1.fc45, 77 and 55 KiB. Fedora's Marvell Wi-Fi firmware is in `linux-firmware` itself; Prestera is its own `mrvlprestera-firmware` (not installed).
- Firmware parity, compared by top-level entry under `/usr/lib/firmware` (package file lists, compression suffix ignored): with Arch's set above plus `-marvell`, `sof-firmware`, `alsa-firmware` and `wireless-regdb`, and Fedora's current set, Arch has and Fedora lacks only DVB and analog-TV tuner firmware (`dvb-*`, `sms1xxx-*`, `v4l-cx*`, `as102_*`, `cmmb_*`, `isdbt_*`, `tdmb_*`) and old sound-card firmware (`ea`, `emu`, `mixart`, `pcxhr`, `vx`, `yamaha`, `sb16`, `asihpi`, `cs46xx`, `ctefx*`, `ctspeq.bin`, RME `digiface`/`multiface`, `aica_firmware.bin`); Fedora's `alsa-firmware` and `dvb-firmware` carry these. Fedora has and Arch lacks server parts (QLogic `ql*`, Brocade `cbfw`/`ct*fw`, `aic`), which Fedora's `linux-firmware` itself ships, and newer `iwlwifi` files (Fedora's firmware is 2026-10-19, Arch's archive 2026-09-16).
- Broadcom STA: Arch has only `broadcom-wl-dkms` 6.30.223.271-50 (`broadcom-wl` no longer exists), which needs `dkms`, gcc and `linux-headers` (68.8 MiB) plus `linux-lts-headers` (58.9 MiB). RPM Fusion nonfree for Fedora 45 has `akmod-wl` 6.30.223.271-63.fc45 (built locally, needs gcc and `kernel-devel`) and the meta package `kmod-wl` with no prebuilt `kmod-wl-<kernel>` in the repository (2026-11-08). Both blacklist the in-tree `b43`, `bcma`, `ssb` and `brcmsmac`. The chips that need it (CachyOS chwd's list: `14e4:4311 4312 4315 4727 4328 4329 432a 432b 432c 432d 0576 4353 4357 4358 4359 4365 4331 43b1 43a0 4360`) are in laptops from about 2008 to 2015, MacBooks with BCM4360 among them.
- `bolt` ships `usr/share/polkit-1/rules.d/org.freedesktop.bolt.rules`, which returns `YES` without a password for `org.freedesktop.bolt.enroll`, `.authorize` and `.manage` when the subject is in an active local session and in group `wheel` (upstream's rule; Fedora's package ships the same).
- `fprintd` is D-Bus activated (`net.reactivated.Fprint.service`, `fprintd.service`); without `pam_fprintd` (Arch: in `fprintd`; Fedora: `fprintd-pam`) wired into a PAM stack, no login or prompt uses the reader.
- What fills the 2 GiB: the largest Arch packages by download are `nvidia-utils` 315 MiB, `linux` 161, `linux-lts` 156, `linux-firmware-intel` 134, `linux-firmware-nvidia` 99, `linux-firmware-atheros` 53, `llvm-libs` 41, `breeze` 38 (pulled by `kwin`), `noto-fonts` 28 (by `plasma-integration`), `perl` 21 (by `git`, which `noctalia` depends on). None of these is optional for the image.

## Inputs
Decisions from the operator's earlier design work, restated, with where this task deviates:
- Every install and the live image carry `linux-firmware`, `linux-firmware-marvell`, `sof-firmware`, `alsa-firmware`, the microcode packages and `wireless-regdb`, so a disk move or a new USB adapter never misses firmware. Leave out server and datacenter parts (Mellanox, Netronome, QLogic, LiquidIO) and `linux-firmware-qcom` on x86_64. **Deviation:** Marvell's Prestera switch firmware is left out too (71 MiB of a datacenter part), by `NoExtract` in the image's build-time `pacman.conf`.
- Fedora maps that set to its split firmware packages (SPEC question 9: which Fedora package set covers the Arch list). Answered above: add `alsa-firmware` and `dvb-firmware`.
- `bolt` on every image for Thunderbolt authorisation.
- Intel MIPI webcams use the open stack: `libcamera`, `libcamera-ipa`, PipeWire's libcamera plugin and `pipewire-v4l2`; IPU6 works per sensor, IPU7 is unsupported for now; no proprietary HAL or DKMS.
- Fingerprint readers get `fprintd`; PAM is wired only after the user enrols, the password always works, and the login screen always takes the password (a fingerprint gives the keyring no password). **Deviation:** `fprintd` is installed on every image, not on detection, and this task wires no PAM at all: there is no enrolment page yet, and the lock screen's PAM stack is Noctalia's. `fprintd-enroll` works; PAM wiring after enrolment is logged for later.
- Out-of-tree packages by hardware detection from the ISO's offline repository, with chwd as the engine on Arch and a small detection step on Fedora. **Deviation:** no detection and no chwd. Everything this task adds is installed everywhere; the one out-of-tree driver left, Broadcom STA, needs a compiler and kernel headers on both bases (about 190 MiB on Arch for two kernels), which the ISO cannot carry, so it stays a documented online step, as the earlier design already did for `b43-firmware`, `broadcom-bt-firmware` and Validity readers.
- Cloudflare R2 for an ISO over 2 GiB (`AGENTS.md`, CI and releases): secrets `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, variable `R2_BUCKET`; the release links the ISO; wired only once an ISO needs it. This task's additions make the Arch x86_64 ISO need it. **Deviation (operator 2026-11-08):** the secrets `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY` and the variable `R2_ENDPOINT` (which names the bucket) instead, and the public prefix `https://cdn.stackific.com/os/releases/`. Broadcom STA ships as docs only (operator 2026-11-08).

Nothing is copied into the tree: the Broadcom ID list is not needed without detection.

## Design
- **Arch x86_64** (`arch/iso/packages.x86_64`): add `linux-firmware-marvell`, `sof-firmware`, `alsa-firmware`, `wireless-regdb`, `bolt`, `fprintd`, `libcamera`, `libcamera-ipa`, `pipewire-libcamera`, `pipewire-v4l2`. `arch/iso/pacman.conf` gets `NoExtract = usr/lib/firmware/mrvl/prestera/*` with a comment; installed systems keep Arch's own `/etc/pacman.conf`, so a later `linux-firmware-marvell` update there writes the Prestera files (71 MiB of disk, harmless) — say so in `docs/hardware.md`.
- **Fedora** (`fedora/iso/config.kiwi`, both architectures): add `alsa-firmware`, `dvb-firmware`, `bolt`, `fprintd`, `libcamera`, `libcamera-ipa`, `pipewire-plugin-libcamera`, `pipewire-v4l2`; not `fprintd-pam`.
- **bolt's polkit rule** stays as packaged (operator 2026-11-08: upstream's rule, unmodified, as Fedora ships it).
- **Broadcom STA**: a new section in `site/src/content/docs/user/system-requirements.md` ("Broadcom Wi-Fi that needs the STA driver"): how to tell (`lspci -nn` shows one of the IDs above), and the online steps over Ethernet or USB tethering: Arch `sudo pacman -S broadcom-wl-dkms linux-headers linux-lts-headers`; Fedora: enable RPM Fusion nonfree (its `rpmfusion-nonfree-release` package), then `sudo dnf install akmod-wl` and reboot.
- **R2** (only for an ISO over 2 GiB = 2147483648 bytes, GitHub's per-file cap):
  - `just/rclone/` (pinned rclone release and checksum, `just setup` and `just doctor` lines, `docs/setup.md`), as `AGENTS.md` foresees ("Other tools (rclone) join `just setup` with the first task that needs them").
  - `release/justfile` `upload`: when the ISO is larger than 2 GiB, `rclone copyto` it to the object `os/releases/v<version>/<name>.iso` in the bucket named by `R2_ENDPOINT` (rclone's on-the-fly S3 remote with `provider=Cloudflare`, keys from the environment, never printed) and upload `<name>.iso.url`, one line with the public URL `https://cdn.stackific.com/os/releases/v<version>/<name>.iso`, to the release instead of the ISO; the manifest and `.iso.sha256` still go to GitHub, so `publish` and `SHA256SUMS` are unchanged. A test build's object is overwritten by a rerun, like its release.
  - As the operator keeps them (operator 2026-11-08): repository secrets `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY`, and repository variable `R2_ENDPOINT` (`vars.R2_ENDPOINT` in the workflows), the bucket's S3 API URL as Cloudflare's bucket settings show it (`https://<account>.r2.cloudflarestorage.com/<bucket>`); the recipe splits it into rclone's endpoint (scheme and host) and the bucket (the path). The public prefix `https://cdn.stackific.com/os/releases/` is fixed (operator 2026-11-08), so no URL variable.
  - `images.yml` and `release.yml` pass the three secrets to the `upload` steps as environment variables; nothing else changes there. `AGENTS.md`'s R2 line changes to these names.
- **No installer change**, no chwd, no new glue script.
- **aarch64**: Fedora aarch64 gets the Fedora list (one `config.kiwi`); the `images` workflow builds and tests it on the Mac on merge. Arch aarch64's list and Qualcomm firmware stay for task 022.

## Changes
- `arch/iso/packages.x86_64`, `arch/iso/pacman.conf`.
- `fedora/iso/config.kiwi`.
- `just/rclone/mod.just`, `just/rclone/vars.just` (new), `justfile` (`mod rclone`, `setup`, `doctor`), `docs/setup.md`.
- `release/justfile` (`upload`), `.github/workflows/images.yml`, `.github/workflows/release.yml` (environment of the upload steps), `docs/releases.md`.
- `tests/vm/arch-x86_64.sh`, `tests/vm/fedora.sh` (the checks below).
- `docs/hardware.md` (the firmware sets per base, the parity rule, the Prestera note, the improvement log), `docs/arch-iso.md`, `docs/fedora-iso.md`, `site/src/content/docs/user/system-requirements.md` (fingerprint readers, webcams, Thunderbolt, Broadcom STA), `release/notes.md` (an ISO may be a link to R2), `AGENTS.md` (the repo-layout line for `hardware/` drops chwd and the Fedora detection step; the hardware line gets this task's set; the R2 line says it is wired).
- The prompt-trail entry `site/src/content/docs/prompt-trail/021-hardware-parity.md` and the review record `dev/reviews/021-hardware-parity.md`.

## Acceptance criteria

Must:
- `just arch iso` and `just fedora iso` build and print their sizes; the task's entry reports both, and the Fedora aarch64 size from the `images` run after merge.
- On both x86_64 live ISOs (`just arch test`, `just fedora test`): the added packages are installed (`pacman -Q` / `rpm -q`, one line each); these sentinel files exist under `/usr/lib/firmware` (any compression suffix; Fedora's `alsa-firmware` installs under `/lib/firmware`, the same directory): `regulatory.db`, `mrvl/pcie8897_uapsta.bin`, `intel/sof` or `intel/sof-ipc4` (a directory), `ctefx.bin`, `dvb-usb-dib0700-1.20.fw`, `yamaha/yss225_registers.bin`, `ea/3g_asic.fw`; `mrvl/prestera` does not exist; `boltctl list` exits 0; `busctl --system introspect net.reactivated.Fprint /net/reactivated/Fprint/Manager` exits 0; the PipeWire libcamera plugin exists (`/usr/lib*/spa-0.2/libcamera/libspa-libcamera.so`); every existing check still passes.
- No PAM file changes: `grep -rl pam_fprintd /etc/pam.d /usr/lib/pam.d` prints nothing on both images but the desktops' own fingerprint services, `kde-fingerprint` (kscreenlocker, Plasma's lock screen) and `gdm-fingerprint` (GDM, which nothing here starts).
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

## Tests
- `just arch packages`, `just arch iso`, `just arch test`, `just arch test-install`.
- `just fedora packages`, `just fedora iso`, `just fedora test`, `just fedora test-install`.
- `just lefthook all`; `just doctor`.
- The R2 path runs only in CI with the secrets: the operator checks the first `images` run after merge (step 4 below).
- For the operator:
  1. In Cloudflare, check that the bucket behind `https://cdn.stackific.com/` serves `os/releases/`, and make an API token with Object Read & Write on it.
  2. In the repository's settings, add the secrets `R2_ACCESS_KEY_ID` and `R2_SECRET_ACCESS_KEY` and the variable `R2_ENDPOINT` (the bucket's S3 API URL, `https://<account>.r2.cloudflarestorage.com/<bucket>`), before this task's PR merges.
  3. After merge, open the test release of the merge commit: the Arch x86_64 ISO is a `.iso.url` file; the other three are ISOs.
  4. Download the URL in that file and the release's `SHA256SUMS`; `sha256sum -c SHA256SUMS --ignore-missing` prints `OK` for it.
  5. On a laptop with a fingerprint reader, in the live session: `fprintd-enroll` enrols a finger and `fprintd-verify` matches it; logging in and `sudo` still ask only for the password.
  6. On a laptop with an Intel IPU6 webcam: Firefox shows the camera on a test page (PipeWire camera).
  7. With a Thunderbolt or USB4 dock: `boltctl list` shows it authorised.
