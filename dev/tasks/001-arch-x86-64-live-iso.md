# 001: Arch x86_64 console live ISO with a QEMU boot test

- [x] Done

Base: arch

## Ask

```text title="Prompt" wrap
Build the first Stackific OS image: an Arch Linux x86_64 live ISO, console only (kernel, firmware, microcode, btrfs and FAT tools, NetworkManager, root autologin on tty1; no desktop, branding or hardware detection yet). Build it with archiso inside a pinned container from one pinned Arch Linux Archive date. Boot it with archiso's systemd-boot mode, UEFI only. Name it stackific-<version>-arch-x86_64.iso, record its package manifest and report its size against the 2 GiB target. Add a QEMU/OVMF boot test that runs checks through a token-gated vsock shell injected by one-boot SMBIOS credentials, so the image itself ships no test units and no sshd. It waits for the end of boot through that shell with `systemctl is-system-running --wait`; systemd's vmm.notify_socket READY over vsock proved lossy here, so the test does not use it. Expose both as just recipes that run unattended on a local Linux PC; a CI job will call the same recipes later.
```

## Goal
`just arch iso` builds a bootable console live ISO for Arch x86_64 in a pinned container, and `just arch test` boots it under QEMU with OVMF and proves it reached a running, logged-in console.

## Context
Nothing image-related exists yet: no `arch/`, no `tests/`, no `.build/` entry in `.gitignore`. The root `justfile` wires toolchain modules, `dev` and `site`; `just doctor` loops over each module's `check` recipe, which prints one `<name>: ok (...)` line or fails with a hint (see `just/rtk/mod.just`).

Facts, all checked 2026-11-04:
- archiso latest release is v92 (2026-10-31). Source: https://gitlab.archlinux.org/archlinux/archiso/-/tags.
- archiso v92 boot modes are `bios.syslinux`, `uefi.grub` and `uefi.systemd-boot`. There is no Limine mode. On x86_64, `uefi.systemd-boot` also builds the ia32 (mixed-mode) loader. Source: `docs/README.profile.rst` at tag v92.
- mkarchiso names the ISO `${iso_name}-${iso_version}-${arch}.iso` (`archiso/mkarchiso` line 1945 at v92). It writes `pacman -Q --sysroot` output to `<install_dir>/pkglist.x86_64.txt` in the ISO tree (line 1826).
- mkarchiso copies `airootfs/` into the root before pacstrap installs the packages (`_make_custom_airootfs`, then `_make_packages`). It applies `file_permissions` from `profiledef.sh`.
- Since v91, mkarchiso fully supports running as a regular user through `unshare`, and warns when run as root via sudo. Source: `CHANGELOG.rst` at v92.
- The live root is found by `archisosearchuuid=%ARCHISO_UUID%`, not by label. Source: `configs/baseline/efiboot/loader/entries/01-archiso-linux.conf` at v92.
- archiso v92 `configs/baseline` is the minimal profile: packages `base cloud-init hyperv linux mkinitcpio mkinitcpio-archiso open-vm-tools openssh pv qemu-guest-agent syslinux virtualbox-guest-utils-nox`, `HOOKS=(base udev modconf archiso block filesystems)` in `airootfs/etc/mkinitcpio.conf.d/archiso.conf`, root with an empty password in `airootfs/etc/shadow` (`root::14871::::::`), and `airootfs/etc/systemd/system-generators/systemd-gpt-auto-generator` as a symlink to `/dev/null`. It enables sshd and systemd-networkd, which this task must not.
- archiso v92 `configs/releng` autologs root on tty1 with `airootfs/etc/systemd/system/getty@tty1.service.d/autologin.conf`: `[Service]`, `ExecStart=`, `ExecStart=-/usr/bin/agetty --noreset --noclear --autologin root - ${TERM}`. Its squashfs options are `airootfs_image_type="squashfs"` and `('-comp' 'xz' '-Xbcj' 'x86,arm64' '-b' '1M' '-Xdict-size' '1M')`.
- The Arch Linux Archive serves dated repos at `https://archive.archlinux.org/repos/YYYY/MM/DD/$repo/os/$arch`; `2026/11/01/core/os/x86_64/core.db` returns HTTP 200.
- Versions at archive date 2026-11-01 (from its `core.db` and `extra.db`): archiso 92-1, systemd 262.1-1, linux 7.2.11.arch1-1, linux-firmware 20260916-1, amd-ucode 20260916-1, intel-ucode 20260925-1, mkinitcpio-archiso 73-1, networkmanager 1.58.1-2, btrfs-progs 7.3-1, edk2-ovmf 202608-1, qemu-system-x86 11.1.3-1, socat 1.8.1.3-1.
- The `linux-firmware` meta package depends on the amd, amdgpu, atheros, broadcom, cirrus, intel, mediatek, nvidia, other, radeon, realtek and ti splits. The server NIC firmware (mellanox, nfp, liquidio, qlogic) is only optional, so the meta package already leaves out what AGENTS.md excludes.
- Container base image: `docker.io/archlinux/archlinux:base-20261101.0.620341`, digest `sha256:837194727013a6e81fb55be11e61d6dcfe7fdd095b3313f8f3cff9bb3f9b182d`. Source: Docker Hub tags API for `archlinux/archlinux`.
- edk2-ovmf installs `/usr/share/edk2/x64/OVMF_CODE.4m.fd` and `/usr/share/edk2/x64/OVMF_VARS.4m.fd`. Source: https://archlinux.org/packages/extra/any/edk2-ovmf/files/.
- systemd as PID 1 reads system credentials from SMBIOS type 11 strings `io.systemd.credential:name=value` and `io.systemd.credential.binary:name=<base64>`. Source: systemd(1), https://www.freedesktop.org/software/systemd/man/latest/systemd.html.
- `vmm.notify_socket` (systemd 253+) makes PID 1 send `READY=1` when boot completes. The payload for vsock is `vsock:CID:PORT`, or `vsock-stream:`, `vsock-dgram:` or `vsock-seqpacket:` to force a socket type. Source: systemd(1) and systemd.system-credentials(7). Measured during /s-build on 2026-11-04 (QEMU 11.1.3, systemd 262.1): sd_notify opens one vsock connection per message and drops a message when connect fails. A listener got 17 to 19 unit messages out of 149 unit events, and missed READY=1 in 2 of 10 boots. The test therefore does not use it.
- `systemd.extra-unit.<unit>` credentials add units, and `systemd.unit-dropin.<unit>` credentials add drop-ins, to the final system (systemd 256+, via systemd-debug-generator). Source: systemd.system-credentials(7), systemd-debug-generator(8).
- QEMU accepts `-smbios type=11,value=<str>` or `-smbios type=11,path=<file>`; `path=` reads the string from a file. Source: https://www.qemu.org/docs/master/system/invocation.html.
- AGENTS.md rules this task applies: Arch x86_64 builds pin one archive date, moved forward only through a test gate; Arch builds and tests run in a pinned container; outputs go to `.build/`; every image records its full package manifest; every image task reports its ISO size; VM test access comes only from one-boot SMBIOS credentials with a token-gated vsock shell; no image ships sshd or test units; no NOPASSWD.

## Inputs
None. The operator chose, in this session: console base only; systemd-boot, UEFI only (no syslinux, no Limine on the ISO); a token-gated vsock shell boot test. During /s-build the operator dropped the READY notification, because it proved lossy (see Context), and chose to gate the boot on the shell with `systemctl is-system-running --wait`.

## Changes
- `.gitignore`: add `/.build/` under a "Build output" heading.
- `justfile`: add `mod arch` with a one-line comment, and add `arch` to the `doctor` loop.
- `arch/vars.just`: `ala_date := "2026-11-01"` and the builder base image reference with the digest above.
- `arch/Containerfile`: from the pinned base image, point `/etc/pacman.d/mirrorlist` at the archive date (passed as a build arg), then `pacman -Syu --noconfirm archiso edk2-ovmf qemu-system-x86 socat`.
- `arch/justfile`, recipes:
  - `builder`: builds the image `localhost/stackific-arch-builder:<ala_date>` with podman;
  - `iso version="0.0.0-dev"`: runs mkarchiso in the builder with a fresh work dir. It writes `.build/arch/x86_64/stackific-<version>-arch-x86_64.iso` and `stackific-<version>-arch-x86_64.packages.txt` (archiso's pkglist), then prints the ISO size in MiB. It warns, but does not fail, above 2048 MiB;
  - `test version="0.0.0-dev"`: runs `tests/vm/boot.sh` on that ISO inside the builder;
  - `check`: the doctor line for podman, `/dev/kvm` and `/dev/vhost-vsock`. On macOS it prints that image builds need Linux and passes.
- `arch/iso/profiledef.sh`:
  - `iso_name="stackific"`, with the ISO version taken from the recipe so the file name ends in `-arch-x86_64.iso`;
  - `iso_publisher="Stackific OS <https://os.stackific.com>"`;
  - `bootmodes=('uefi.systemd-boot')`;
  - releng's squashfs options;
  - `file_permissions` for `/etc/shadow` (`0:0:400`).
- `arch/iso/packages.x86_64`: `base linux linux-firmware amd-ucode intel-ucode mkinitcpio mkinitcpio-archiso networkmanager btrfs-progs dosfstools`.
- `arch/iso/pacman.conf`: baseline's (core and extra through `/etc/pacman.d/mirrorlist`, which the builder points at the archive date).
- `arch/iso/efiboot/loader/loader.conf` and `arch/iso/efiboot/loader/entries/01-stackific-linux.conf`:
  - title "Stackific OS (x86_64, UEFI)";
  - baseline's kernel and initrd paths;
  - options `archisobasedir=%INSTALL_DIR% archisosearchuuid=%ARCHISO_UUID% console=ttyS0,115200 console=tty0`;
  - `timeout 3`.
- `arch/iso/airootfs/`, from baseline minus sshd, cloud-init, networkd/resolved and guest-agent files:
  - `etc/hostname` (`stackific`), `etc/locale.conf`, `etc/localtime`, `etc/shadow`;
  - `etc/mkinitcpio.conf.d/archiso.conf`;
  - `etc/systemd/system-generators/systemd-gpt-auto-generator` (symlink to `/dev/null`);
  - releng's `etc/systemd/system/getty@tty1.service.d/autologin.conf`;
  - `etc/systemd/system/multi-user.target.wants/NetworkManager.service` (symlink to `/usr/lib/systemd/system/NetworkManager.service`).
- `tests/vm/boot.sh`: the base-neutral boot test, given an ISO path and an output dir. Steps:
  1. Make a random token in a 0600 file, a random guest CID and a copy of OVMF_VARS.
  2. Start `qemu-system-x86_64` with KVM, q35, OVMF pflash, the ISO as CD-ROM, `vhost-vsock-pci`, a serial log, `-display none` and an HMP monitor socket. Pass these SMBIOS type 11 credentials, the token through `path=` so it never appears in `ps`:
     - `stackific.test.token`;
     - `systemd.extra-unit.stackific-test-shell.socket` and `systemd.extra-unit.stackific-test-shell@.service`;
     - `systemd.unit-dropin.sockets.target`, which adds `Wants=stackific-test-shell.socket`.
  3. Retry the shell for up to 300 s until `systemctl is-system-running --wait` prints `running`.
  4. Run each check over a new vsock connection: send the token line, then the command.
  5. Save a PNG screenshot of the console with HMP `screendump -f png`.
  6. Send `systemctl poweroff` and wait up to 60 s for QEMU to exit.
  7. On any failure, print the tail of the serial log and exit non-zero.
- `tests/vm/stackific-test-shell.socket`: `ListenStream=vsock::<port>`, `Accept=yes`.
- `tests/vm/stackific-test-shell@.service`: `ImportCredential=stackific.test.token`, socket stdin and stdout. It reads one line and execs `bash` only if the line equals the token; otherwise it exits 1.
- `README.md`: Linux host prerequisite `podman` for image builds, plus read-write `/dev/kvm` and `/dev/vhost-vsock` (and how to check them); `just arch iso` and `just arch test` under Everyday use.
- `AGENTS.md`, one line each:
  - the doctor line becomes "Image tasks add podman, KVM and vhost-vsock; OVMF and AAVMF come from the pinned builder containers";
  - under Builds: the Arch builder is `arch/Containerfile`, pinned with the archive date in `arch/vars.just`;
  - under Tests: boot tests reach the VM only through the vsock shell from `systemd.extra-unit.*`/`systemd.unit-dropin.*` credentials and wait with `systemctl is-system-running --wait` over it; `vmm.notify_socket` over vsock drops messages, so it is not used (`tests/vm/`).
- `site/src/content/docs/developer/arch-iso.md`: build and test the Arch ISO, its outputs, how the test reaches the VM, and how to move the archive date (edit `arch/vars.just`, then `just arch iso` and `just arch test` must pass). No user docs yet: nothing is released, so users cannot see this image.

## Acceptance criteria

Must:
- `just arch iso` exits 0 on a Linux host with rootless podman. It writes `.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.iso` and `.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.packages.txt`, and its last line prints the ISO size in MiB.
- The manifest lists `linux 7.2.11.arch1-1` and `systemd 262.1-1`, which proves the 2026-11-01 archive pin.
- `just arch test` exits 0. While it runs, checks over the vsock shell print:
  - `running` for `systemctl is-system-running`;
  - `root` on `tty1` for `who`;
  - `running` for `nmcli -t -f RUNNING general`.
- `just arch test` leaves the screenshot `.build/arch/x86_64/screen.png` showing the root shell prompt on tty1, and `.build/arch/x86_64/serial.log`.
- A connection to the test shell's vsock port with a wrong token gets no shell: `echo bad; echo id` prints nothing and the connection closes.
- `just doctor` prints one `arch:` line naming the podman version and the `/dev/kvm` and `/dev/vhost-vsock` state, and fails with a hint when one is missing.
- `just lefthook all` passes (shellcheck on `tests/vm/boot.sh` and `arch/iso/profiledef.sh`, justfiles parse, docs build).
- The task report states the ISO size in MiB against the 2048 MiB target.

Must not:
- ship `openssh`, `cloud-init`, VM guest agents, syslinux or GRUB in the image (absent from the manifest), or any file from `tests/` in the image;
- use sudo or root on the host for the build or the test;
- leave a stale archiso work dir that makes a rebuild skip steps;
- add `NOPASSWD` or a polkit rule anywhere;
- touch files outside the Changes list (the review record and prompt-trail entry from /s-build excepted);
- add a dependency this task does not name, or add the CI workflow or a self-hosted runner, which is a separate chore.

## Tests
- `just arch iso`, then `ls -l .build/arch/x86_64/` and `grep -E '^(linux|systemd|openssh|cloud-init|syslinux|grub) ' .build/arch/x86_64/*.packages.txt`. This proves the build and naming criteria, the manifest pin and the absent packages.
- `just arch test`: proves the boot wait, the three shell checks, the screenshot and the serial log. Add the wrong-token check to `tests/vm/boot.sh` itself, so it runs on every test.
- `just doctor` and `just lefthook all`.
- Run `just arch iso` twice in a row; the second run must rebuild cleanly. This proves the fresh work dir.
- Manual, only if the operator wants a real-hardware check:
  1. Write the ISO to a USB stick: `sudo dd if=.build/arch/x86_64/stackific-0.0.0-dev-arch-x86_64.iso of=/dev/sdX bs=4M conv=fsync`, where `/dev/sdX` is the stick, not a disk you need.
  2. Boot it on a UEFI machine with Secure Boot off.
  3. Confirm the root prompt on tty1 and that `nmcli device` lists the network devices.
