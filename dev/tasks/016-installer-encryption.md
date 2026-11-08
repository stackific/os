# 016: Every install encrypted: LUKS2 under btrfs, unlocked by the login password, with a plain screen after three wrong tries

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Encrypt every install of Stackific OS, on both x86_64 images (Arch and Fedora), with no opt-out: the installer puts LUKS2 on the Stackific OS partition and btrfs inside it, and the user's login password is also the disk passphrase, typed at every start. Refuse a password that is too easy to guess, using zxcvbn (score 3 of 4 or better, with the user's name, user name, computer name and the product name counted as guessable), on the You screen and in the unattended answers, and say on the screen that the password also unlocks the disk. Unlock with a passphrase only, no TPM. Use the systemd initramfs on both bases: on Arch add mkinitcpio's sd-encrypt hook, on Fedora dracut's own systemd-cryptsetup module, with the keyboard layout the user chose at the passphrase prompt. After three wrong passphrases show a plain screen instead of systemd's emergency shell: what went wrong, check Caps Lock and the keyboard layout, press Enter to restart. Tests unlock the installed system with systemd's cryptsetup.passphrase credential, and one boot types the passphrase with the German layout, after three wrong ones that must show the plain screen. Docs say what encryption protects (a lost or stolen computer) and what it does not (someone changing the computer while you are away), and how to change the disk passphrase.
```

## Goal
Every install, in both modes, writes a LUKS2 container on the Stackific OS partition with btrfs inside it, unlocked at boot by the login password through the systemd initramfs, and refuses weak passwords. `just arch test-install` and `just fedora test-install` prove the encrypted system starts through the credential, through the typed passphrase with the German layout, and shows the plain screen after three wrong passphrases.

## Context
What exists on `main` at `2b8dc29` (checked 2026-11-08):
- The installer (`installer/`, tasks 012 to 015) partitions with `disk::erase` or `disk::append` (ESP then the system partition), then `disk::format` runs `mkfs.fat` and `mkfs.btrfs -f -L <id> <partition>`, and `disk::prepare` makes the subvolumes and mounts them under `/run/stackific-installer/target` (`installer/src/disk.rs`). `install::steps` (`installer/src/install.rs`) reads the btrfs UUID with `blkid`, writes `/etc/fstab` by it, and passes `boot::cmdline(<btrfs uuid>)` = `root=UUID=<uuid> rootflags=subvol=/@ rw` to `Base::initramfs` (`installer/src/base.rs`): Arch puts it in `limine.conf`, Fedora in `/etc/kernel/cmdline` for stackific-boot's kernel-install plugin. `install::install` unmounts the target recursively whatever happened.
- Commands run through the `Runner` trait; `Cmd::stdin` carries secrets (`chpasswd` gets the password that way) and the log never shows stdin (`install.rs` header).
- Step 4 runs `systemd-firstboot` with the console keymap (`system::firstboot`, `system::console_keymap`), so `/etc/vconsole.conf` is in the target before the initramfs is built in step 7.
- The You screen (`installer/src/ui/you.rs`) checks a password only for being non-empty, free of control characters and typed twice (`answers::check_password`); unattended answers use the same check (`Answers::check`).
- The Installing screen lists `install::STEPS`; step 1 is "Format them".
- Arch's initramfs comes from mkinitcpio's pacman-hook script with Arch's default `/etc/mkinitcpio.conf`; archiso's own drop-in `/etc/mkinitcpio.conf.d/archiso.conf` is removed from the target first (`Base::live_state`).
- Fedora's initramfs comes from kernel-install's dracut plugin in a `chroot` with `/proc`, `/sys`, `/dev` and `/run` bound (task 014); dracut is host-only.
- Tests: `tests/vm/boot.sh` passes SMBIOS type 11 credentials (`cred <name> <file>`), presses keys (`sendkey`, QEMU key names) and takes screenshots; `boot.sh -` boots the installed disk alone and saves `limine.png` a second after the firmware's `BdsDxe: starting Boot` line. `tests/vm/install.sh` installs unattended with `password='stackific-test-password'` and keyboard `de`; `tests/vm/installed.sh` checks `/proc/cmdline` for `root=UUID=… rootflags=subvol=/@ rw` and restarts twice for Limine's Windows entries. `arch/justfile` and `fedora/justfile` `test-install` run the install boot, then `boot.sh -` with `installed.sh`.
- Installed Arch systems keep `cryptsetup` (Arch's `systemd` package depends on it: `depend = cryptsetup` in its `.PKGINFO`, 262.1-1 at the 2026-11-01 archive). The Fedora image has `cryptsetup` only because `stackific-installer` requires it (task 015), so `dnf5 remove stackific-installer` takes it off installed systems; Fedora's `systemd` 262.1 carries `systemd-cryptsetup` and its generator.

Upstream facts (checked 2026-11-08):
- **cryptsetup 2.8.8 defaults** (`cryptsetup --help` in the Arch builder at the 2026-11-01 archive): LUKS2 with argon2id, iteration time 2000 ms, up to 1 GiB of memory and 4 threads, `aes-xts-plain64` with a 512-bit key (XTS doubles 256). `luksFormat --batch-mode` skips the `YES` question; `--key-file -` reads the passphrase from stdin to its end, a trailing newline included, so stdin must carry the password without one.
- **The systemd initramfs unlock** (systemd 262, `systemd-cryptsetup-generator(8)`): `rd.luks.uuid=<uuid>` unlocks that LUKS device in the initramfs as `/dev/mapper/luks-<uuid>`. With `root=/dev/mapper/luks-<uuid>`, the root device unit requires the unlock service, so a failed unlock fails the root mount at once and the initramfs goes to `emergency.target`; with `root=UUID=<btrfs uuid>` it would wait for the device timeout (90 s). The default is 3 tries.
- **The passphrase credential**: `/usr/lib/systemd/systemd-cryptsetup` reads the credential `cryptsetup.passphrase` (strings of the binary, systemd 262.1); SMBIOS type 11 credentials reach PID 1 through `/sys/firmware/dmi/entries`, which needs `CONFIG_DMI_SYSFS`: built in on Arch's kernel (earlier decision TS-02) and on Fedora's (`CONFIG_DMI_SYSFS=y` in `kernel-core` 7.2.15-300.fc45's `config`).
- **mkinitcpio 42.2-1** (Arch, 2026-11-01 archive): default `HOOKS=(base systemd autodetect microcode modconf kms keyboard sd-vconsole block filesystems fsck)`; `sd-encrypt` (`/usr/lib/initcpio/install/sd-encrypt`) adds dm-crypt, systemd-cryptsetup, its generator and the console password agent. `/etc/mkinitcpio.conf.d/*.conf` files are read after `mkinitcpio.conf`, later ones winning. The `base` hook provides busybox's `/bin/sh`. An install hook is `/usr/lib/initcpio/install/<name>` with a `build()` that may call `add_file <source> <dest> [mode]`.
- **dracut 111-5.fc45**: `71systemd-cryptsetup`, `70crypt` and `11systemd-creds` are in `/usr/lib/dracut/modules.d/`; host-only builds include the crypt modules when the root's block device stack has `crypto_LUKS`. A module whose `check()` returns 0 is included in every build.
- **emergency.service** (systemd 262.1): `ExecStart=-/usr/lib/systemd/systemd-sulogin-shell emergency`; on an initramfs with no root password it prints that the root account is locked.
- **zxcvbn crate 3.1.1** (https://crates.io/crates/zxcvbn, MIT, published 2026-04-24, Rust 1.66 or newer): `zxcvbn(password, &user_inputs)` returns an `Entropy` whose `score()` is 0 to 4; it is Dropbox's estimator ported to Rust.

## Inputs
From the operator's working plan, item 016 ("Encryption: LUKS2 root with passphrase unlock, unattended unlock in tests", BT-04, BT-06, BT-09, TS-02), and the earlier decisions it names, checked against `AGENTS.md` and upstream on 2026-11-08. Nothing is copied into the tree.
- **BT-09, always on (operator 2026-11-08, SPEC decision 9):** every install encrypts the Stackific OS partition; no toggle. Plain LUKS2 on the partition (no LVM), cryptsetup's defaults (argon2id, 2000 ms).
- **BT-09, the passphrase (operator 2026-11-08):** the login password is the disk passphrase. It must reach zxcvbn score 3 of 4, with the full name, user name, computer name and product name as user inputs, on the You screen and in the unattended check. A "use a different disk password" toggle is left for later (the improvements list in `docs/installer.md`).
- **BT-06, passphrase only:** no TPM2 or FIDO2 enrolment; without Secure Boot no option protects against tampering, and the docs say so: encryption protects the data if the computer is lost or stolen, not against someone changing it while you are away.
- **BT-04, the systemd initramfs:** Arch adds `sd-encrypt` to its default hooks (busybox `encrypt` and `cryptdevice=` are not used); Fedora's dracut unlocks through systemd-cryptsetup. Command line: `rd.luks.uuid=<luks uuid> root=/dev/mapper/luks-<luks uuid> rootflags=subvol=/@ rw`. The keyboard layout at the prompt comes from `/etc/vconsole.conf`, written before the initramfs is built. No Plymouth.
- **BT-06, after three wrong passphrases (operator 2026-11-08: build it now):** systemd's 3 tries stay. Instead of the emergency shell, a plain screen: the passphrase was wrong 3 times (or the system could not start), check Caps Lock and the keyboard layout, press Enter to restart and try again. No shell is offered. The same screen covers any other initramfs failure.
- **TS-02, tests:** routine boots of the installed system unlock with the SMBIOS credential `cryptsetup.passphrase`; one boot types the passphrase (the real prompt and the layout), after three wrong ones that show the plain screen.

## Design
- **Disk step.** After partitioning: `cryptsetup luksFormat --type luks2 --batch-mode --key-file - <system partition>` (stdin: the password, no newline), then `cryptsetup open --key-file - <system partition> stackific-installer-root`; `mkfs.btrfs` and every later step use `/dev/mapper/stackific-installer-root`. The fixed name keeps clean-up simple; the installed system names it `luks-<uuid>` itself. `install::install` closes the mapping after the recursive unmount whenever it is open (`cryptsetup status` exits 0), also after a failure or a stop.
- **UUIDs.** `blkid` of the partition gives the LUKS UUID (for the command line), of the mapping the btrfs UUID (for fstab, unchanged).
- **Command line.** `boot::cmdline(luks_uuid)` = `rd.luks.uuid=<uuid> root=/dev/mapper/luks-<uuid> rootflags=subvol=/@ rw`, in Arch's `limine.conf` and Fedora's `/etc/kernel/cmdline` as today.
- **Arch hooks.** `Arch::initramfs` writes `/etc/mkinitcpio.conf.d/stackific.conf` in the target with Arch's default hooks plus `sd-encrypt` after `block` and `stackific-boot-failure` last. The installer writes it, not a package: a package's drop-in would sort after archiso's and change the live image's initramfs.
- **Fedora.** dracut's host-only detection adds systemd-cryptsetup; the test checks it. `stackific-boot` requires `cryptsetup` on both bases, so installed systems keep the command the docs use (Fedora would otherwise lose it with the installer).
- **The failure screen**, in `stackific-boot` on both bases, its shared sources in `installer/initramfs/`: `/usr/lib/stackific/boot-failure` (POSIX `sh`, busybox on Arch, bash on Fedora) prints the message on the console, waits for Enter and runs `systemctl reboot`; `/usr/lib/stackific/boot-failure.conf` is a drop-in that replaces `ExecStart` of `emergency.service` and `rescue.service`. Arch: the mkinitcpio install hook `stackific-boot-failure` adds both files to the initramfs (the drop-in under `/etc/systemd/system/<unit>.d/`). Fedora: dracut module `99stackific-boot-failure` does the same, its `check()` returning 0 in host-only builds only, so kiwi's live initramfs (not host-only) leaves it out. Nothing changes the installed system's own emergency mode.
- **Password strength.** `answers::check_strength(password, inputs)` refuses a score below 3 with "Too easy to guess: try a longer password, or a few unrelated words." The You screen shows it after the other checks, and its help says the password also unlocks the disk at every start. The Confirm screen says the Stackific OS partition is encrypted and the password unlocks it at every start; the new partition's row shows `LUKS2` as its file system.
- **Installing.** Step 1 becomes "Encrypt and format them".

## Changes
- `installer/Cargo.toml`, `installer/Cargo.lock`: the `zxcvbn` crate 3.1.1.
- `installer/src/disk.rs`: `encrypt` (luksFormat and open), `MAPPER`, `format` on the mapping, tests.
- `installer/src/install.rs`: the encrypt step, the LUKS UUID, closing the mapping, STEPS[1], tests.
- `installer/src/boot.rs`: `cmdline` with `rd.luks.uuid` and the mapper root, tests.
- `installer/src/base.rs`: Arch's hooks drop-in, tests.
- `installer/src/answers.rs`: `check_strength`, used by `Answers::check`, tests.
- `installer/src/ui/you.rs`, `installer/src/ui/confirm.rs`: the strength check and the encryption lines.
- `installer/src/ui/tests.rs`, `installer/fixtures/screens/*.txt`: snapshots and passwords that pass.
- `installer/src/demo.rs`: fake `cryptsetup` answers.
- `installer/initramfs/stackific-boot-failure` and `stackific-boot-failure.conf` (new, shared): the script and its drop-in.
- `arch/packages/stackific-boot/PKGBUILD` and its Arch-only `initcpio-install-stackific-boot-failure`; `fedora/packages/stackific-boot/stackific-boot.spec` and its `dracut-stackific-boot-failure.sh`; both require `cryptsetup`.
- `release/justfile`: the new paths in `just release changed`.
- `tests/vars.just`: the test password.
- `tests/vm/boot.sh`: the `cryptsetup.passphrase` credential from `STACKIFIC_TEST_PASSPHRASE`, and a pre-boot step (`PREBOOT`, sourced before it waits for the test shell) for an installed disk.
- `tests/vm/install.sh`: a strong password without `y`, `z` or symbols (they differ on the German layout), a refused weak password, and checks of the LUKS2 container.
- `tests/vm/installed.sh`: the command line and initramfs checks.
- `tests/vm/unlock.sh` (new): three wrong passphrases, the plain screen, Enter restarts, the right passphrase typed on the German layout.
- `tests/vm/screens.sh`: passwords that pass.
- `arch/justfile`, `fedora/justfile`: `test-install` passes the passphrase and runs the typed-unlock boot.
- `docs/installer.md`, `site/src/content/docs/user/installation.md`, `AGENTS.md`, `release/notes.md`: encryption.
- The prompt-trail entry and review record.

## Acceptance criteria

Must:
- `just installer test` and `just installer lint` pass, with tests for the luksFormat and open commands (password on stdin, no newline), the command line, Arch's hooks drop-in, the strength check (a weak password refused, a strong one accepted, the user name counted as guessable) and closing the mapping after a failure.
- The installed system's partition 2 (free-space mode) is `crypto_LUKS` version 2 with one keyslot that the test password opens, and btrfs is inside it (`install.sh`).
- An unattended install with a password scoring below 3 exits 2 and names the password (`install.sh`).
- The installed system boots with `/proc/cmdline` holding `rd.luks.uuid=<uuid> root=/dev/mapper/luks-<uuid> rootflags=subvol=/@ rw`; `/` is btrfs on `/dev/mapper/luks-<uuid>`; on Arch the initramfs has `systemd-cryptsetup` and the boot-failure drop-in, on Fedora `lsinitrd -m` lists `systemd-cryptsetup` and `stackific-boot-failure` (`installed.sh`, both bases).
- Typed unlock (`unlock.sh`, both bases): three wrong passphrases leave the VM on the plain screen with no restart; Enter restarts it; the right passphrase, typed with the German layout, starts the system.
- Every existing install and installed-system check still passes, including both Windows entries (which now unlock through the credential).
- The You screen refuses a weak password, and its help and the Confirm screen mention encryption (snapshots, screenshots).

Must not:
- offer a way to install without encryption, or enrol a TPM, FIDO2 key or key file;
- change the live images' initramfs or emergency mode, or the installed system's own emergency mode;
- log the password (the existing journal check in `install.sh` stays);
- touch files outside the Changes list, or add a dependency other than `zxcvbn`.

## Tests
- `just installer test`, `just installer lint`.
- `just arch iso`, `just arch test`, `just arch test-install`, `just arch screenshots`.
- `just fedora packages`, `just fedora iso`, `just fedora test`, `just fedora test-install`.
- `just lefthook all`.

Manual, on a real computer (operator):
1. Install from the USB stick with a laptop keyboard layout other than US, into the free space next to Windows.
2. Restart: the passphrase prompt appears after Limine's menu; type the password with that layout; the desktop's login screen follows.
3. Restart, type a wrong passphrase three times: the plain screen appears; press Enter; the computer restarts.
