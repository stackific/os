# 003: Fedora 45 x86_64 console live ISO with a QEMU boot test

- [x] Done

Base: fedora

## Ask

```text title="Prompt" wrap
Build the second base: a Fedora 45 x86_64 console live ISO that mirrors task 001 (kernel, firmware, microcode, btrfs and FAT tools, NetworkManager, root autologin on tty1; no desktop, branding or hardware detection yet; no sshd, cloud-init or VM guest agents; UEFI only, with a serial console on the kernel command line). Build it with Fedora's own live-image tool inside a pinned builder container, unattended and rootless like task 001: no sudo and no loop devices on the host. Fedora builds from its live repos, so the package manifest is the record. Respect Fedora's trademark rules for a remix. Name it stackific-<version>-fedora-x86_64.iso next to stackific-<version>-fedora-x86_64.packages.txt, refuse a version that is not semver, and print the ISO size in MiB with a warning above the 2 GiB target. Boot-test it under QEMU with OVMF through tests/vm/boot.sh, the same token-gated vsock shell from one-boot SMBIOS credentials, waiting with systemctl is-system-running --wait. Keep boot.sh base-neutral: a check only one base passes must not run on the other. Add fedora/justfile (builder, iso, test, check), mod fedora and the doctor loop in the root justfile, an images.yml job fedora-x86_64 on [self-hosted, Linux, X64] calling the same recipes, developer notes in docs/fedora-iso.md and one-line AGENTS.md facts. Keep edits to shared files minimal, because task 004 edits some of them in parallel. Every release needs all four ISOs, so the second base unblocks the most.
```

## Goal
`just fedora iso` builds a bootable console live ISO for Fedora 45 x86_64 with kiwi in a pinned rootless builder container, and `just fedora test` boots it under QEMU with OVMF and proves it reached a running system with root logged in on tty1 and SELinux enforcing.

## Context
Tasks 001 and 002 built the Arch x86_64 live ISO: `arch/` (archiso profile, `arch/Containerfile`, `arch/vars.just`, `arch/justfile`), the boot test `tests/vm/boot.sh` with the test shell units `tests/vm/stackific-test-shell.socket` and `tests/vm/stackific-test-shell@.service`, the developer notes `docs/arch-iso.md`, and the `arch-x86_64` job in `.github/workflows/images.yml`. `boot.sh` hard-codes Arch's OVMF paths and runs the Arch desktop checks (greetd, MangoWM, Noctalia, noctalia-greeter) on every ISO it boots. Nothing exists for Fedora yet. This task adds `fedora/`, makes `boot.sh` serve both bases, and replaces the test shell's socket unit with a service that also works under Fedora's SELinux.

Task 004 (Arch `stackific-release`), built in parallel, adds Arch-only identity checks to `boot.sh`. If 004 lands first, they sit behind a `[[ $iso == *-arch-*.iso ]]` guard: this task moves them into `tests/vm/arch-x86_64.sh` and drops the guard. If this task lands first, 004 adds them there.

Facts, all checked 2026-11-05. "Probe" means measured on 2026-11-05 on the operator's Linux PC (rootless podman 6.1.4, kernel 7.3, no SELinux on the host), with a kiwi description and builder matching Changes:
- Fedora 45 is released. The pinned image's `/etc/os-release` says `VERSION_ID=45`, `RELEASE_TYPE=stable` and `SUPPORT_END=2027-12-08`. Fedora 46 is rawhide (registry tags `46`, `rawhide`). Source: `registry.fedoraproject.org/v2/fedora/tags/list`.
- Builder base: `registry.fedoraproject.org/fedora:45`, multi-arch index digest `sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd` (amd64 manifest `sha256:056cf36ff1b736f21a6fa0af3ce04a35e84fc3e0d20b45d13be5e7c82907f182`, created 2026-11-05). The tag is rebuilt often, so the digest is the pin. Source: the registry's manifest API (`docker-content-digest` header).
- Fedora's live ISOs have been built with kiwi since Fedora 42 (pungi-fedora PR 1419, https://pagure.io/pungi-fedora/pull-request/1419, summarised at https://maxwell-k.github.io/blog/how-does-fedora-build-disk-images.html). The f45 branch of https://forge.fedoraproject.org/releng/kiwi-descriptions (`components/liveinstall.xml`) builds them as `image="iso"` with `flags="dmsquash"`, `filesystem="erofs"`, `erofscompression="lzma,level=6"`, `firmware="uefi"`, `efiparttable="gpt"`, `efifatimagesize="30"` and `<bootloader name="grub2">`. Its repositories (`repositories/core.xml`) are the `fedora` and `updates-released` metalinks, written with `$releasever` and `$basearch`.
- The other candidates need root or cannot make an ISO:
  - livemedia-creator `--no-virt` runs under sudo and "may totally trash your host" (https://weldr.io/lorax/livemedia-creator.html, Lorax 41.3).
  - image-builder runs as `sudo image-builder build` or `sudo podman run --privileged` (https://github.com/osbuild/image-builder-cli README).
  - mkosi's `Format=` has no ISO output (https://github.com/systemd/mkosi, `mkosi.1.md`).
- kiwi in Fedora 45 is `kiwi-cli` 11.1.1-2.fc45. `kiwi-systemdeps-iso-media` pulls the tools its ISO builder calls (xorriso, mtools, dosfstools, grub2 tools). Source: `dnf repoquery` in the pinned image.
- kiwi 11.1.1 needs loop devices or mounts only on paths this task avoids. Source: kiwi's code in the pinned image (`/usr/lib/python3.15/site-packages/kiwi/`):
  - `builder/live.py`: a loop device is used only for a live root filesystem other than `squashfs` or `erofs`. With `erofs`, `mkfs.erofs` writes the image straight from the root directory.
  - `bootloader/config/grub2.py`, `_create_embedded_fat_efi_image`: the ISO's EFI image is made with `mkdosfs` and `mcopy`, with no loop device.
  - `bootloader/config/systemd_boot.py`: the systemd-boot ISO path uses `LoopDevice` and a mount, so it cannot run rootless. That is why this ISO boots with GRUB, while the Arch ISO uses systemd-boot.
  - kiwi bind-mounts `/dev`, `/proc` and `/sys` into the image root, so the container needs `--privileged` (rootless, as `just arch iso` already runs).
- Probe: `kiwi-ng system build` runs as root of a rootless `--privileged` container, with no host privileges, and builds the ISO in about 2 minutes. Each of these probe failures needs the fix shown:
  - `firmware="efi"` alone stops with `grub path i386-pc not found`. `eficsm="false"` drops BIOS support (`_supports_platform_modules` in `grub2.py`).
  - Without `grub2-efi-x64-modules` in the image, it stops with `grub path x86_64-efi not found`.
  - Without `binutils` in the builder, it stops with `KiwiCommandNotFound: strings command not found on buildhost`.
  - Without `glibc-langpack-en` in the image, `<locale>en_US</locale>` stops with `Locale en_US.UTF-8 is not installed`.
  - The image root on the container's overlay storage gives `setfiles: Could not set context for ...: Operation not permitted`. A rootless user may not set `security.selinux` on a host-backed filesystem. On a tmpfs mounted inside the container (`--tmpfs /work:rw,exec,dev,suid`), labelling works. podman's default tmpfs options include `noexec`, which breaks the chroot. That tmpfs held 2.0 GiB at the end of the console build, and its default size is half the host's RAM.
  - A kiwi work dir on a host bind mount leaves root-owned sub-UID files that the host user cannot delete. Keeping it inside the container (`--rm`) makes every build fresh, as for archiso.
- kiwi checks package signatures only when a repository says `package_gpgcheck="true"`. The default is off (`repository/dnf5.py`). Probe: with that attribute and the builder's `file:///usr/share/pki/rpm-gpg/RPM-GPG-KEY-fedora-$releasever-primary` (from `fedora-gpg-keys`), the build passes. With Fedora 44's key in its place, it fails: `Transaction failed: Signature verification failed`.
- kiwi writes `<name>.<arch>-<version>.iso` and `<name>.<arch>-<version>.packages` to its target dir. The manifest has one line per package: `name|epoch|version|release|arch|disturl|license`. Source: `builder/live.py` (`isoname`) and `system/setup.py` (`_export_rpm_package_list`). kiwi's own `<version>` does not matter here, because the recipe renames both files.
- The ISO's GRUB entry is titled with the image's `displayname`. Its kernel line is `console=ttyS0,115200 console=tty0 root=live:CDLABEL=STACKIFIC rd.live.image`, so dracut's `dmsquash-live` module (package `dracut-live`) finds the live root by volume ID. Source: `/boot/grub2/grub.cfg` in the probe ISO.
- kiwi's `<packages type="image">` defaults to `patternType="onlyRequired"` (no weak dependencies). Probe with `plusRecommended`: 35 more packages and 70 MB more, among them systemd-networkd, systemd-resolved, tpm2-tools and i686 glibc. This task stays with `onlyRequired` and names the weak dependencies it keeps, so the choice is reviewable. One package has to be named: `systemd-pam`. Without it (it is only a Recommends of systemd 262.1-1.fc45), login opens no logind session and `who` prints nothing.
- Weak dependencies of this image's package list, measured on 2026-11-05 with `dnf install --assumeno --installroot=… --setopt=install_weak_deps=True` against `False` in the pinned builder: 212 packages without, 250 with. The operator's rule: keep what a regular desktop or laptop user commonly needs on a console live system, leave out the rare and exotic.
  - Kept, named in the image list:
    - `tzdata`: time zones for `timedatectl set-timezone` and local time; without it only UTC works.
    - `cracklib-dicts`: the dictionary `passwd` checks a new password against (libpwquality); setting a password is common.
    - `intel-npu-firmware`: firmware for the NPU in Intel Core Ultra laptops; AGENTS.md ships the firmware regular hardware needs.
  - Left out:
    - `systemd-networkd`, `systemd-resolved`: NetworkManager is the network stack, and task 001 left both out too.
    - `tpm2-tools`, `tpm2-tss-fapi`: TPM administration, rare on a live console.
    - `rpm-plugin-audit`: audit records for package transactions; the live image has no package manager.
    - `python3`, `python3-libs`, `python-pip-wheel`, `python-unversioned-command`, `mpdecimal`: nothing in the image runs Python.
    - `grubby`, `kpartx`, `memstrack`: boot-entry editing, device-mapper partition maps and dracut memory tracing, for installed or debugged systems.
    - `crypto-policies-scripts`, `pkcs11-provider`: changing the system crypto policy and PKCS#11 tokens, rare.
    - `glib-networking`, `libproxy`, `duktape`, `gsettings-desktop-schemas`: GLib's proxy and TLS stack, used by graphical apps the console image does not have.
    - `adwaita-sans-fonts`, `adwaita-mono-fonts`, `fonts-filesystem`: desktop fonts; the console uses kbd's fonts. The desktop task brings its own.
    - `glibc-gconv-extra`, `gawk-all-langpacks`, `ntfs-3g-system-compression`, `mkpasswd`, `whois-nls`, `wcurl`: legacy charset conversion, gawk translations, an NTFS plugin without ntfs-3g, and small tools a console live system rarely uses.
    - i686 `glibc` (seen in the kiwi probe): 32-bit libraries, exotic on this image.
  - In the image anyway, as hard dependencies: `selinux-policy` requires `rpm-plugin-selinux`, which pulls `rpm`, `rpm-libs`, `rpm-sequoia`, `lua-libs`, `libarchive` and `lz4-libs` (build manifest, 2026-11-05).
- Fedora splits `linux-firmware`. Its Recommends are amd-gpu, amd-ucode, atheros, brcmfmac, cirrus-audio, intel-audio, intel-gpu, mt7xxx, nvidia-gpu, nxpwireless, qcom-wwan, realtek and tiwilink. Fedora's comps group `hardware-support` installs these default firmware packages: `alsa-sof-firmware amd-gpu-firmware atheros-firmware brcmfmac-firmware cirrus-audio-firmware intel-audio-firmware intel-gpu-firmware intel-vsc-firmware iwlegacy-firmware iwlwifi-dvm-firmware iwlwifi-mvm-firmware libertas-firmware mt7xxx-firmware nvidia-gpu-firmware nxpwireless-firmware qcom-wwan-firmware realtek-firmware tiwilink-firmware`. `iwlwifi-mvm-firmware` requires `iwlwifi-mld-firmware` and `iwlbluetooth-firmware`. The server and datacenter firmware (`liquidio`, `mlxsw_spectrum`, `mrvlprestera`, `netronome`, `qed`, `qcom-accel`) is only optional in that group, so naming the defaults leaves out what AGENTS.md excludes. Source: `dnf repoquery` and `dnf group info hardware-support` in the pinned image. Microcode: `amd-ucode-firmware`, and `microcode_ctl` for Intel.
- Versions in the live repos on 2026-11-05 (`dnf repoquery`): kernel 7.2.15-300.fc45, systemd 262.1-1.fc45, linux-firmware 20261019-1.fc45, NetworkManager 1:1.58.1-3.fc45, btrfs-progs 7.1-1.fc45, dosfstools 4.2-19.fc45, dracut 111-5.fc45, grub2 1:2.12-83.fc45, selinux-policy-targeted 45.23-1.fc45, generic-release 45-0.4, edk2-ovmf 20260812-10.fc45, qemu-system-x86-core 2:11.1.3-1.fc45, socat 1.8.1.3-1.fc45. They move with Fedora's updates; the manifest records what each build got.
- Fedora 45 gives tty1 to kmscon. `redhat-systemd-presets-common` 102-3.fc45, `90-default.preset`, has `enable kmsconvt@.service` and `disable getty@.service` (rhbz#2429626). Probe: without kmscon, nothing runs on tty1. This image enables `getty@tty1.service` itself with a symlink and gives it Arch's autologin drop-in (`/usr/bin/agetty`, from `util-linux-core`). The same presets enable `NetworkManager.service`.
- Trademarks. Fedora's trademark guidelines require that "the fedora-logos, fedora-release, and fedora-release-notes RPM packages are removed", and allow replacing them "with packages of your own devising not containing the Fedora Trademarks" (https://gitlab.com/fedora/legal/fedora-legal-docs, `modules/ROOT/pages/trademarks.adoc`, published at https://docs.fedoraproject.org/en-US/legal/trademarks/). Fedora provides `generic-logos`, `generic-release` and `generic-release-notes` for this (https://fedoraproject.org/wiki/Remix).
  - `generic-release` 45-0.4 provides `system-release(45)`, requires `fedora-repos(45)` and `distribution-systemd-presets`, and ships `/usr/lib/os-release` with `NAME=Generic`, `ID=generic` and `ID_LIKE=fedora`. That file has placeholder URLs, among them `HOME_URL="http://www.zombo.com/"`.
  - Naming `generic-release` among the bootstrap packages keeps dnf from picking `fedora-release` for `system-release`. Probe: the manifest has no `fedora-release*` and no `fedora-logos`. Nothing in this console set requires `system-logos`.
  - The boot menu, volume ID and ISO metadata say Stackific OS. The kernel, GRUB and other binaries are unmodified Fedora packages.
- Probe boot (QEMU 11.1.3, Fedora's OVMF): the system reaches `running` with no failed units, SELinux `Enforcing`, NetworkManager running, and the root shell prompt on tty1 under the banner `Generic 45 (Generic)`. The ISO is 967157760 bytes, 922 MiB, against the 2048 MiB target.
- Fedora's edk2-ovmf ships raw `/usr/share/edk2/ovmf/OVMF_CODE.fd` (1966080 bytes) and `/usr/share/edk2/ovmf/OVMF_VARS.fd` (131072 bytes). Its 4 MiB builds come only as qcow2. Arch's are `/usr/share/edk2/x64/OVMF_CODE.4m.fd` and `OVMF_VARS.4m.fd`, as `boot.sh` hard-codes. Source: `ls` in each builder. Probe: `boot.sh` with Fedora's raw pair, `format=raw`, boots the ISO.
- systemd 262.1-1.fc45 has `systemd-debug-generator`, so `systemd.extra-unit.*` and `systemd.unit-dropin.*` credentials (systemd 256+) work. Units from them "are added to both the initrd and the final system" (https://www.freedesktop.org/software/systemd/man/latest/systemd-debug-generator.html). The Fedora initrd is systemd-based, so it receives them too (`Received regular credentials: ...` appears twice in the serial log). A drop-in on `multi-user.target` only acts in the final system.
- SELinux blocks the current test shell. Probe, with the socket unit from task 001 on the enforcing Fedora image:
  - PID 1 is denied the socket: `avc: denied { create } for pid=1 comm="systemd" scontext=system_u:system_r:init_t:s0 tcontext=system_u:system_r:initrc_t:s0 tclass=vsock_socket permissive=0`. systemd labels a socket unit's sockets like the process its service will run, and bash (`shell_exec_t`) started by PID 1 becomes `initrc_t` (`type_transition init_t shell_exec_t:process initrc_t`).
  - In selinux-policy-targeted 45.23 (`sesearch`), PID 1 may create vsock sockets only of type `init_t` (without `listen` or `accept`) or `sshd_t`.
  - `SELinuxContextFromNet=yes` on the socket unit gave the same denial.
  - A service running `systemd-socket-activate` works. PID 1 starts it as `unconfined_service_t` (`type_transition init_t bin_t:process unconfined_service_t`), and that domain may create, listen on and accept vsock sockets (`allow unconfined_domain_type domain:vsock_socket { ... create listen accept ... }`).
  - `systemd-socket-activate` (systemd-socket-activate(1); shipped in `systemd` on both bases) clears the child's environment. `--setenv=CREDENTIALS_DIRECTORY` passes the credential directory through, and `--inetd` connects the socket to the child's stdin and stdout.
  - Probe: with that service, all Fedora checks pass with SELinux enforcing, and the Arch x86_64 desktop ISO from task 002 passes every one of its checks.
- AGENTS.md rules this task applies:
  - Fedora builds from live repos, and every image records its full package manifest;
  - every image task reports its ISO size;
  - VM test access comes only from one-boot SMBIOS credentials with a token-gated vsock shell, and no image ships sshd or test units;
  - tests read and edit disk images only through `qemu-img`, `qemu-io` and `mtools`, never with root, loop devices or mounts;
  - no `NOPASSWD`;
  - Secure Boot is off, so the ISO needs no shim.

Left out, for later tasks:
- Fedora's `stackific-release` (os-release `ID=stackific`), which replaces `generic-release` and its placeholder URLs (task 007 in the operator's plan);
- the desktop, greetd and the live user (task 008);
- kmscon, Fedora 45's own VT console (agetty does the autologin here);
- BIOS boot, shim and Secure Boot, and the ISO's media check (`mediacheck`, `isomd5sum`);
- systemd-resolved, Fedora's default DNS resolver (NetworkManager writes `/etc/resolv.conf` itself);
- pinning the builder's kiwi version: Fedora's updates repo keeps only the newest build, so podman's layer cache keeps the builder's tools until the base digest moves;
- RAM for the labelling tmpfs on larger images and on the aarch64 runner VM (12 GiB). Labelling inside a rootless container on an SELinux-enforcing host (the Fedora runner VM) is also untested;
- a package cache shared by ISO builds (every build downloads every package).

## Inputs
None. The operator named no earlier material. These design choices are made in this task: kiwi with GRUB, `firmware="efi"` without shim, an erofs live root, SELinux enforcing with labels set on a container tmpfs, `generic-release` for os-release, named packages instead of weak dependencies (operator, 2026-11-05: keep commonly needed weak dependencies, exclude rare and exotic ones), the test shell as a `systemd-socket-activate` service, and per-image check files for `boot.sh`.

## Changes
- `justfile`: add `mod fedora` with a one-line comment, like `mod arch`'s, and add `fedora` to the `doctor` loop.
- `fedora/vars.just` (new):
  - `builder_base := "registry.fedoraproject.org/fedora:45@sha256:10989b31ee8859a592f8f8d9992406d7d4483e99091e6ff686a352b6c040dccd"`;
  - `builder := "localhost/stackific-fedora-builder:45"`;
  - `out := justfile_directory() / ".build" / "fedora" / "x86_64"`;
  - a header comment: Fedora builds from its live repos, so the manifest is the record, and the digest moves only through the test gate (`just fedora iso`, then `just fedora test`).
- `fedora/Containerfile` (new): from the pinned base (build arg), `dnf -y install kiwi-cli kiwi-systemdeps-iso-media binutils edk2-ovmf qemu-system-x86-core socat && dnf clean all`. A header comment says why `binutils` is there (kiwi calls `strings`).
- `fedora/justfile` (new), shaped like `arch/justfile`:
  - `builder`: `podman build --quiet` of `fedora/Containerfile` with `--build-arg BASE={{ builder_base }}`, tagged `{{ builder }}`.
  - `iso version="0.0.0-dev"` (`[positional-arguments]`, depends on `builder`):
    - It refuses a version that is not semver, with `arch/justfile`'s check.
    - It removes old outputs, then runs `kiwi-ng --type iso system build --description /desc --target-dir /work` in `podman run --rm --privileged --tmpfs /work:rw,exec,dev,suid`, with `fedora/iso` mounted read-only at `/desc` and `{{ out }}` at `/out`.
    - It installs `/work/*.iso` and `/work/*.packages` (mode 0644) as `/out/stackific-<version>-fedora-x86_64.iso` and `/out/stackific-<version>-fedora-x86_64.packages.txt`, passing the name as data in the environment like `arch/justfile` does.
    - It prints the ISO size in MiB, with `arch/justfile`'s warning above 2048 MiB.
    - A comment says why the tmpfs (fresh work dir, SELinux labels) and why `exec`.
  - `test version="0.0.0-dev"` (`[positional-arguments]`, depends on `builder`): runs `/tests/boot.sh "/out/stackific-$1-fedora-x86_64.iso" /out /tests/fedora-x86_64.sh` in the builder.
    - It uses `arch/justfile`'s devices and `--security-opt seccomp=unconfined`.
    - It sets `--env OVMF_CODE=/usr/share/edk2/ovmf/OVMF_CODE.fd --env OVMF_VARS=/usr/share/edk2/ovmf/OVMF_VARS.fd`.
    - It mounts `tests/vm` read-only at `/tests` and `{{ out }}` at `/out`.
  - `check`: `arch/justfile`'s check with the prefix `fedora:`.
- `fedora/iso/config.kiwi` (new), the kiwi description (schema 8.5):
  - `<image schemaversion="8.5" name="stackific" displayname="Stackific OS">`, with a `<description type="system">` naming Stackific OS and https://os.stackific.com.
  - Preferences:
    - `<version>0.0.0</version>`, with a comment that the recipe names the outputs;
    - `<packagemanager>dnf5</packagemanager>`, `<release-version>45</release-version>`;
    - `<locale>en_US</locale>`, `<keytable>us</keytable>`, `<timezone>UTC</timezone>`.
  - One `<type>`, with `<bootloader name="grub2" timeout="3"/>`:
    - `image="iso" primary="true" flags="dmsquash" filesystem="erofs" erofscompression="lzma,level=6"`;
    - `firmware="efi" eficsm="false" efiparttable="gpt" efifatimagesize="30" mediacheck="false"`;
    - `volid="STACKIFIC" publisher="Stackific OS" application_id="Stackific OS Live (Fedora base)"`;
    - `kernelcmdline="console=ttyS0,115200 console=tty0"`.
  - Repositories `fedora` and `updates`:
    - Fedora's metalinks `https://mirrors.fedoraproject.org/metalink?repo=fedora-$releasever&amp;arch=$basearch` and `...?repo=updates-released-f$releasever&amp;arch=$basearch`;
    - each with `package_gpgcheck="true"` and `<signing key="file:///usr/share/pki/rpm-gpg/RPM-GPG-KEY-fedora-$releasever-primary"/>`.
  - Bootstrap packages: `filesystem generic-release`.
  - Image packages:
    - `kernel kernel-modules-extra systemd systemd-pam systemd-udev dracut-live`;
    - `grub2-efi-x64 grub2-efi-x64-cdboot grub2-efi-x64-modules`;
    - `linux-firmware`, the 18 `hardware-support` firmware packages listed in Context, `amd-ucode-firmware microcode_ctl`;
    - `NetworkManager btrfs-progs dosfstools selinux-policy-targeted passwd util-linux procps-ng glibc-langpack-en kbd`;
    - the kept weak dependencies from Context: `tzdata cracklib-dicts intel-npu-firmware`.
  - A comment says why `generic-release` (trademarks) and why each package named only because a probe needed it (`systemd-pam`, `grub2-efi-x64-modules`, `glibc-langpack-en`).
- `fedora/iso/config.sh` (new): `set -euo pipefail` and `passwd -d root`, so root has an empty password like on the Arch ISO. A header comment says kiwi runs it in the image root.
- `fedora/iso/root/etc/hostname` (new): `stackific`.
- `fedora/iso/root/etc/systemd/system/getty@tty1.service.d/autologin.conf` (new): the same content as `arch/iso/airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf` in task 001 (`[Service]`, `ExecStart=`, `ExecStart=-/usr/bin/agetty --noreset --noclear --autologin root - ${TERM}`).
- `fedora/iso/root/etc/systemd/system/getty.target.wants/getty@tty1.service` (new): a symlink to `/usr/lib/systemd/system/getty@.service`. Fedora 45's presets disable getty@ (see Context).
- `tests/vm/stackific-test-shell.socket` and `tests/vm/stackific-test-shell@.service`: delete them.
- `tests/vm/stackific-test-shell.service` (new):
  - `ImportCredential=stackific.test.token`;
  - `ExecStart=/usr/bin/systemd-socket-activate --listen=vsock::5001 --accept --inetd --setenv=CREDENTIALS_DIRECTORY /usr/bin/bash -c 'exec 2>&1; IFS= read -r t; [ -n "$$t" ] && [ "$$t" = "$$(< "$$CREDENTIALS_DIRECTORY/stackific.test.token")" ] || exit 0; exec /usr/bin/bash'`;
  - a header comment keeping the old units' reasons (never in an image; the empty-line guard; a rejected probe still succeeds), and saying why it is a service and not a socket unit (Fedora's SELinux, see Context).
- `tests/vm/boot.sh`:
  - Usage becomes `boot.sh <iso> <out-dir> <checks>`. The third argument is required. After the base checks (end of boot, `running`, NetworkManager `running`, wrong token rejected), it `source`s `<checks>`, then powers off as before.
  - OVMF comes from `OVMF_CODE` and `OVMF_VARS` in the environment, with the Arch builder's current paths as defaults.
  - It reads the shell port from `--listen=vsock::<port>` in `stackific-test-shell.service`.
  - It passes the credentials `systemd.extra-unit.stackific-test-shell.service` and `systemd.unit-dropin.multi-user.target` (`[Unit]`, `Wants=stackific-test-shell.service`), in place of the socket, its service and the `sockets.target` drop-in.
  - It removes `screen.png`, `greeter.png` and `serial.log` before booting, as now.
  - The header comment describes the check files.
- `tests/vm/arch-x86_64.sh` (new): the Arch checks moved unchanged out of `boot.sh`, from "The live session" through `screenshot greeter`, with 004's identity checks if 004 landed first (see Context). Header: `# shellcheck shell=bash` and a comment saying `boot.sh` sources it after the base checks.
- `tests/vm/fedora-x86_64.sh` (new), with the same header:
  - `vm_until 'who' '^root +tty1 ' 30`;
  - `vm_until 'getenforce' '^Enforcing$' 0`;
  - `screenshot screen`.
- `arch/justfile`: the `test` recipe passes `/tests/arch-x86_64.sh` as `boot.sh`'s third argument. Nothing else changes.
- `.github/workflows/images.yml`: a job `fedora-x86_64` like `arch-x86_64` (the same runner labels, timeout, pinned actions and `persist-credentials: false`), with the steps `just fedora iso` and `just fedora test`.
- `docs/fedora-iso.md` (new), developer notes shaped like `docs/arch-iso.md`:
  - build: kiwi in the builder, the tmpfs and why;
  - the outputs, and the manifest format;
  - what is in the image and why (firmware set, getty@tty1, `generic-release` and the trademark rule, SELinux enforcing);
  - the boot test and its Fedora checks;
  - how to move the builder digest (edit `fedora/vars.just`; `just fedora iso` and `just fedora test` must pass).

  No user docs yet: nothing is released.
- `docs/arch-iso.md`, "Boot test": the shell is now the `systemd-socket-activate` service started by a `multi-user.target` drop-in, and the Arch checks live in `tests/vm/arch-x86_64.sh`.
- `AGENTS.md`, one line each:
  - Builds: the Fedora builder is `fedora/Containerfile` (kiwi, QEMU, OVMF, socat), its base pinned by digest in `fedora/vars.just`. kiwi runs as container root in rootless `--privileged` podman, with its work dir on a container tmpfs, because SELinux labels cannot be set on podman's overlay.
  - Builds: Fedora images never ship `fedora-release*`, `fedora-logos` or `fedora-release-notes` (Fedora's trademark guidelines). `generic-release` stands in until our Fedora `stackific-release`.
  - Tests: `tests/vm/boot.sh <iso> <out-dir> <checks>` runs the checks every image passes, then sources the image's `tests/vm/<base>-<arch>.sh`. A check that only one image passes goes there.
  - Tests: the test shell is a `systemd-socket-activate` service (`tests/vm/stackific-test-shell.service`), not a socket unit, because Fedora's SELinux policy denies PID 1 a listening vsock socket labelled for a shell.

## Acceptance criteria

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

## Tests
- `just fedora iso`, then run these on `.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.packages.txt`. Together they prove the build, naming, manifest and absent-package criteria.
  - `ls -l .build/fedora/x86_64/`;
  - `grep -E '^(kernel|systemd|systemd-pam|selinux-policy-targeted|generic-release|dracut-live|linux-firmware|iwlwifi-mvm-firmware|amd-ucode-firmware|microcode_ctl|NetworkManager|btrfs-progs|dosfstools|tzdata|cracklib-dicts|intel-npu-firmware)\|'` lists all sixteen;
  - `grep -E '^(fedora-(release|logos)[a-z-]*|openssh[a-z-]*|cloud-init|qemu-guest-agent|open-vm-tools|hyperv-daemons|spice-vdagent|shim[a-z0-9-]*|grub2-pc|syslinux|liquidio-firmware|mlxsw_spectrum-firmware|mrvlprestera-firmware|netronome-firmware|qed-firmware|qcom-accel-firmware|systemd-networkd|systemd-resolved|tpm2-tools|python3)\|'` prints nothing.
- `just fedora iso 1.0` and `just fedora iso '1.0.0"x'`: prove the semver check.
- Run `just fedora iso` a second time: proves the fresh work dir.
- `just fedora test`: proves the boot, the shell checks, SELinux enforcing, the screenshot and the serial log. Look at `screen.png`, and convert it with `just site screenshot` into `site/src/assets/prompt-trail/003/console.webp` for the prompt-trail entry.
- `just arch iso`, then `just arch test`: proves the Arch ISO still passes with the moved checks and the new shell service.
- `just doctor`, `just lefthook all`, and `git grep -nE 'NOPASSWD|polkit' fedora tests` (prints nothing).
- Manual, only if the operator wants a real-hardware check:
  1. Write the ISO to a USB stick: `sudo dd if=.build/fedora/x86_64/stackific-0.0.0-dev-fedora-x86_64.iso of=/dev/sdX bs=4M conv=fsync`, where `/dev/sdX` is the stick, not a disk you need.
  2. Boot it on a UEFI machine with Secure Boot off and pick "Stackific OS" in the boot menu.
  3. Confirm the root prompt on tty1, that `getenforce` prints `Enforcing`, and that `nmcli device` lists the network devices.
