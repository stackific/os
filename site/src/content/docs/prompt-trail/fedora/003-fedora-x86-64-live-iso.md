---
title: Fedora 45 x86_64 console live ISO with a QEMU boot test
description: The second base, a console-only Fedora 45 x86_64 live ISO built with kiwi in a pinned rootless container and boot-tested under QEMU with OVMF.
sidebar:
  order: 3
  label: 003 Fedora x86_64 live ISO
pr: https://github.com/stackific/os/pull/3
task: dev/tasks/003-fedora-x86-64-live-iso.md
---

## Goal

`just fedora iso` builds a bootable console live ISO for Fedora 45 x86_64 with kiwi in a pinned rootless builder container, and `just fedora test` boots it under QEMU with OVMF and proves it reached a running system with root logged in on tty1 and SELinux enforcing.

## Context

Tasks 001 and 002 built the Arch x86_64 live ISO and a boot test that hard-coded Arch's firmware paths and desktop checks. This step adds the Fedora base with kiwi, the tool Fedora builds its own live ISOs with, and makes the boot test serve both bases. It also replaces the test shell's socket unit with a service, because Fedora's SELinux policy denies the socket unit.

## Expectations

Must:

- `just fedora iso` exits 0 on a Linux host with rootless podman. It writes `.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.iso` and `.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.packages.txt`, and its last line prints the ISO size in MiB.
- `just fedora iso 1.0` and `just fedora iso '1.0.0"x'` print `version must be semver, got '…'` and exit 1.
- The manifest has a line for each of `kernel`, `systemd`, `systemd-pam`, `selinux-policy-targeted`, `generic-release`, `dracut-live`, `linux-firmware`, `iwlwifi-mvm-firmware`, `amd-ucode-firmware`, `microcode_ctl`, `NetworkManager`, `btrfs-progs`, `dosfstools`, `tzdata`, `cracklib-dicts` and `intel-npu-firmware`. Each line has the form `name|epoch|version|release|arch|…`. The `kernel` and `systemd` releases end in `.fc45`.
- `just fedora test` exits 0 and prints `ok` for:
  - the end of boot and `running`;
  - NetworkManager `running`;
  - wrong token rejected;
  - `who` showing root on `tty1`;
  - `getenforce` printing `Enforcing`.
- `just fedora test` leaves `.build/fedora/x86_64/screen.png`, showing the root shell prompt on tty1, and `.build/fedora/x86_64/serial.log`.
- `just arch test` still exits 0, with every check it printed before, now run through `tests/vm/arch-x86_64.sh` and the new test shell service.
- `just doctor` prints one `fedora:` line naming the podman version and the `/dev/kvm` and `/dev/vhost-vsock` state, and fails with a hint when one is missing.
- `just lefthook all` passes: shellcheck on `tests/vm/*.sh` and `fedora/iso/config.sh`, the justfiles parse, actionlint on `images.yml`, `just dev check`.
- Running `just fedora iso` twice in a row rebuilds cleanly both times.
- The task report states the ISO size in MiB against the 2048 MiB target (the probe measured 922 MiB).

Must not:

- list any of these in the manifest:
  - `fedora-release`, `fedora-release-common`, `fedora-release-identity-*`, `fedora-logos`, `fedora-release-notes`;
  - `openssh`, `openssh-server`, `cloud-init`, `qemu-guest-agent`, `open-vm-tools`, `hyperv-daemons`, `spice-vdagent`;
  - `shim-x64`, `grub2-pc`, `syslinux`;
  - `systemd-networkd`, `systemd-resolved`, `tpm2-tools`, `python3` (weak dependencies left out, see Context);
  - `liquidio-firmware`, `mlxsw_spectrum-firmware`, `mrvlprestera-firmware`, `netronome-firmware`, `qed-firmware`, `qcom-accel-firmware`;
- ship any file from `tests/` in the image;
- use sudo, root or loop devices on the host for the build or the test;
- set SELinux permissive or disabled in the image or in the test;
- run an Arch check on the Fedora ISO, or a Fedora check on the Arch ISO;
- add `NOPASSWD` or a polkit rule anywhere;
- touch files outside the Changes list (the review record, prompt-trail entry and its screenshot from /s-build excepted). `README.md` stays as it is, because the host prerequisites do not change;
- add a dependency this task does not name.

## Prompt

```text title="Prompt" wrap
Build the second base: a Fedora 45 x86_64 console live ISO that mirrors task 001 (kernel, firmware, microcode, btrfs and FAT tools, NetworkManager, root autologin on tty1; no desktop, branding or hardware detection yet; no sshd, cloud-init or VM guest agents; UEFI only, with a serial console on the kernel command line). Build it with Fedora's own live-image tool, kiwi, inside a pinned builder container, unattended and rootless like task 001: no sudo and no loop devices on the host. Boot it with GRUB and no shim (Secure Boot is off), keep SELinux enforcing, and use generic-release in place of fedora-release to respect Fedora's trademark rules for a remix. Install no weak dependencies by default; name the commonly needed ones (tzdata, cracklib-dicts, intel-npu-firmware) and leave out the rare and exotic ones (systemd-networkd, systemd-resolved, TPM tools, Python, 32-bit libraries and the like). Fedora builds from its live repos, so the package manifest is the record. Name it stackific-<version>-fedora-x86_64.iso next to stackific-<version>-fedora-x86_64.packages.txt, refuse a version that is not semver, and print the ISO size in MiB with a warning above the 2 GiB target. Boot-test it under QEMU with OVMF through tests/vm/boot.sh, the same token-gated vsock shell from one-boot SMBIOS credentials, waiting with systemctl is-system-running --wait. Make that shell a systemd-socket-activate service on both bases, since Fedora's SELinux denies the socket unit. Keep boot.sh base-neutral: a check only one base passes goes into a per-image check file that boot.sh sources. Add fedora/justfile (builder, iso, test, check), mod fedora and the doctor loop in the root justfile, an images.yml job fedora-x86_64 on [self-hosted, Linux, X64] calling the same recipes, developer notes in docs/fedora-iso.md and one-line AGENTS.md facts. Keep edits to shared files minimal, because task 004 edits some of them in parallel. Every release needs all four ISOs, so the second base unblocks the most.
```

## Outcome

`just fedora iso` builds the builder image from `fedora/Containerfile` (kiwi, QEMU, OVMF and socat on a digest-pinned `fedora:45`), then runs `kiwi-ng system build` on `fedora/iso/`. kiwi runs as root of a rootless `--privileged` container. Its work directory is a tmpfs inside the container, so every build starts fresh, and SELinux labels can be set there, which podman's overlay storage refuses. The live root is erofs and the EFI image is made with `mkdosfs` and `mcopy`, so no loop device is needed. The image boots with GRUB, keeps SELinux enforcing, and carries `generic-release`, so no Fedora trademark package is in it.

The ISO is 929 MiB, against the 2048 MiB target, with 231 packages. Without the three kept weak dependencies it was 922 MiB and 228 packages.

`tests/vm/boot.sh` now takes a third argument, the image's check file. It runs the checks every image passes, then sources `tests/vm/arch-x86_64.sh` or `tests/vm/fedora-x86_64.sh`. The test shell is `tests/vm/stackific-test-shell.service`, which runs `systemd-socket-activate` on vsock port 5001.

### Screenshots

![Console on tty1 showing "Generic 45 (Generic)", "stackific login: root (automatic login)" and a root shell prompt](../../../../assets/prompt-trail/003/console.webp)

_The Fedora live ISO after boot: root is logged in on tty1 with no password prompt. The banner says Generic until our Fedora release package lands._

## Review

- **Round 1** found one thing, fixed: the task listed `rpm`, `rpm-libs`, `rpm-plugin-selinux`, `rpm-sequoia`, `lua-libs`, `libarchive` and `lz4-libs` as weak dependencies left out, but the image has them, because `selinux-policy` requires `rpm-plugin-selinux`. The task now lists them as hard dependencies; the image did not change.
- **Round 2** found nothing.

## Manual changes

None.

## Result

The repo now builds and boot-tests a second base:

- `fedora/` holds the builder container, its digest pin, the kiwi description and the image's overlay files.
- `tests/vm/boot.sh` serves both bases, with `tests/vm/arch-x86_64.sh` and `tests/vm/fedora-x86_64.sh` for each image's own checks, and the test shell is one `systemd-socket-activate` service.
- `just fedora iso`, `just fedora test`, a `fedora:` doctor line and the `fedora-x86_64` job in `images.yml` wire them in.

Verified on a Linux host with rootless podman 6.1.4:

- `just fedora iso` exits 0 twice in a row, about 3 minutes each, and prints `929 MiB (target 2048 MiB)`. The manifest has 231 packages, with `kernel 7.2.15-300.fc45`, `systemd 262.1-1.fc45` and every package the expectations name, and none of the ones they exclude.
- `just fedora iso 1.0` and `just fedora iso '1.0.0"x'` print `version must be semver` and exit 1.
- `just fedora test` prints `ok` for the end of boot, `running`, NetworkManager, the wrong token, root on `tty1` and `Enforcing`, then `PASS`.
- `just arch iso` (1497 MiB) and `just arch test` pass every check they printed before.
- `just doctor` prints `fedora: ok (podman 6.1.4, /dev/kvm and /dev/vhost-vsock read-write)`. shellcheck, actionlint, the justfile parse and `just dev check` pass, and `git grep -nE 'NOPASSWD|polkit' fedora tests` prints nothing.

Docs: the new developer notes `docs/fedora-iso.md` cover the build, the image, the boot test and moving the builder digest. `docs/arch-iso.md` describes the new test shell and the Arch check file, and `AGENTS.md` gains four lines.

## Deviations

- The operator changed the weak-dependency rule during the build: keep the commonly needed ones and leave out the rare and exotic. The task file was updated first (Context, Inputs, Changes, the manifest criteria), and the image names `tzdata`, `cracklib-dicts` and `intel-npu-firmware`.
- The two kiwi repositories also carry `sourcetype="metalink"`, which kiwi needs to read a metalink URL. The task's Changes list does not name the attribute; the task file was not updated for it.
