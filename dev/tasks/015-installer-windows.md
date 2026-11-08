# 015: Install next to Windows: BitLocker checked and confirmed, Windows in Limine's menu, refusals with guidance

- [ ] Done

Base: both

## Ask

```text title="Prompt" wrap
Make the installer safe and complete next to Windows, on both x86_64 images (Arch and Fedora). Find BitLocker volumes on every drive, read each one's protectors with cryptsetup, and before anything is written show one screen that says what Windows will do at its next start and which recovery key it will ask for, or that no recovery key exists. Then require a typed confirmation, "I have my key", which also lets the free space next to BitLocker be used. Find Windows by its loader on any EFI System Partition, read-only, and add Windows to Limine's menu: a default entry that restarts into the firmware's "Windows Boot Manager" (Limine's efi_boot_entry, so BitLocker does not ask for its key after Limine updates) and a fallback entry that starts Windows' loader directly. Keep these entries when Fedora's kernel-install rewrites limine.conf, and keep Windows' firmware entry in the boot order. Refuse, with guidance, what we cannot install next to: Intel RST RAID members (switch the firmware to AHCI, prepare Windows first) and Windows on an MBR drive, which starts in legacy BIOS mode (convert with MBR2GPT.EXE first). Erasing such a drive stays allowed. Say when Intel RST hides NVMe drives. No os-prober, no entries for other Linux installs, and no change to when the free space is offered. Prove it in both bases' install tests with real BitLocker volumes made by Windows (cryptsetup's test images), an MBR Windows-like drive, and a fake Windows Boot Manager that the installed system starts through both Limine entries. Add a user page, "Next to Windows": save the BitLocker key, Fast Startup off, Secure Boot off, starting Windows from Limine, putting Stackific OS first again after a big Windows update, and Windows' clock (RealTimeIsUniversal).
```

## Goal
The installer warns about every BitLocker volume and asks for a typed confirmation before it writes, installs next to BitLocker, refuses RST RAID members and BIOS-mode Windows with guidance, and writes Windows entries into Limine's menu that survive kernel updates on Fedora. `just arch test-install` and `just fedora test-install` prove it: the installed system starts the fake Windows through both entries.

## Context
What exists on `main` at `cea9bb0` (checked 2026-11-07):
- Tasks 012, 013 and 014 built the installer (`installer/`, one crate, `Base` for Arch and Fedora) on both x86_64 images. Free space is used when the GPT is clean, the protective MBR has only its `ee` entry, no partition is BitLocker (`blkid -p`), and the largest 1 MiB-aligned gap is at least 40 GiB (`installer/src/drives.rs`, `free_space`). A BitLocker partition anywhere on the drive refuses the free space ("BitLocker is on this drive"). Erase is always allowed after the drive's kernel name is typed.
- `drives::scan` reads `lsblk --json --bytes --output NAME,PATH,TYPE,SIZE,RO,TRAN,MODEL,FSTYPE,LABEL,START,LOG-SEC,MOUNTPOINTS`, leaves out the live medium (`findmnt` of `Base::live_medium`), zram, read-only drives, drives under 40 GiB and drives in use, each with a reason the Drive screen shows. The Space and Confirm screens list partitions with lsblk's FSTYPE and LABEL (`installer/src/ui/space.rs`, `draw_parts`).
- The Confirm screen (`installer/src/ui/confirm.rs`) already has a typed confirmation for an erase (`State::typed`); the unattended file has `[disk] erase = "<kernel name>"` (`installer/src/answers.rs`, `check_erase`). Exit 2 means refused, nothing written.
- `installer/src/boot.rs`: `limine_conf(name, kernel, initramfs, root_uuid)` writes Arch's static `limine.conf` (one entry); `stale()` deletes only firmware entries that start our loader `\EFI\limine\BOOTX64.EFI` from a vanished partition; `create()` runs `efibootmgr --create`, which puts our entry first and keeps every other entry after it.
- Fedora: `fedora/packages/stackific-boot/90-stackific-limine.install` rewrites `/boot/limine.conf` whole on every `kernel-install add` or `remove` (header, then one entry per kernel on the ESP). Anything else in the file is lost.
- Tests: `tests/vm/install.sh` (sourced by `tests/vm/boot.sh` in `just arch test-install` and `just fedora test-install`) runs the console path, a refused erase, an erase, then a Windows-like GPT on `/dev/vda` (ESP 100 MiB FAT `SYSTEM`, Microsoft reserved 16 MiB, a 2 GiB gap, basic data 10 GiB as partition 3, a large gap, Windows recovery 1 GiB as partition 5), fakes BitLocker by writing `EB 52 90 -FVE-FS-` at partition 3's start, checks the free-space refusal, clears it, then installs into the free space and checks the Windows-like ESP is still empty. `tests/vm/installed.sh` boots the installed disk alone (`boot.sh -`, with `OVMF_VARS=<out>/vars.fd` from the install boot), checks `efibootmgr`, `limine.conf`, and on Fedora the plugin with a second kernel. `boot.sh` gives `vm` (one command line to the guest's test shell), `vm_until`, `sendkey` (QEMU monitor) and `screenshot`. The guest's test shell is `bash` reading its commands from the vsock socket (`tests/vm/stackific-test-shell.service`). `tests/vm/screens.sh` (`just arch screenshots`) drives the screens by key presses.
- The images (manifests in `.build/`, built 2026-11-07): Arch x86_64 has `cryptsetup 2.8.8-1`, `limine 12.9.6-1`, `mdadm 4.6-2`, `efibootmgr 18-4`, `util-linux 2.42.4-1`, `ntfsprogs`. Fedora x86_64 has `cryptsetup-libs 2.8.8-1.fc45` but **not the `cryptsetup` command**, `limine 12.9.6-1.fc45` (Terra), `mdadm 4.6-2.fc45`, `os-prober 1.81-12.fc45` (a GRUB dependency; we do not use it), and no `ntfsprogs` (`mkfs.ntfs`).
- `site/src/content/docs/user/installation.md` says the installer puts Stackific OS first in the firmware's boot order and a large Windows update can put Windows first again; it has no page on Windows.

Upstream facts (checked 2026-11-07):
- **Limine 12.9.6 config** (https://github.com/Limine-Bootloader/Limine/blob/v12.9.6/CONFIG.md): `protocol: efi_boot_entry` takes `entry:` "The name of the EFI boot entry to reboot into" and ignores `cmdline`. `protocol: efi` (aliases `uefi`, `efi_chainload`) takes `path:`. `guid(<GUID>):/path` names a filesystem GUID or a GPT partition GUID. An optional `comment:` is shown when the entry is selected. Comments start with `#` on their own line. **There is no include directive**, so Fedora's plugin must write any extra entries itself.
- **`efi_boot_entry` behaviour** (`common/protos/efi_boot_entry.c` at v12.9.6): it walks `BootOrder` in order, matches each `Boot####` description against `entry` case-insensitively and in full, sets `BootNext` to the first match and does a warm reset (`EfiResetWarm`). With no match it panics: "efi_boot_entry: Failed to find boot entry '<name>'". So the entry only works while "Windows Boot Manager" is in `BootOrder`.
- **BitLocker and Secure Boot** (the operator's earlier decision, restated in Inputs): Windows 11 24H2 turns on device encryption on any edition with a TPM and Secure Boot, bound to PCR 7. Turning Secure Boot off, which our unsigned ISO needs, makes Windows ask for the recovery key at its next start unless BitLocker was suspended first. Chainloading `bootmgfw.efi` from Limine measures Limine into PCR 4, so with Secure Boot off Windows asks again after each Limine update; `efi_boot_entry` lets the firmware start Windows itself, as systemd-boot's `reboot-for-bitlocker` does.
- **cryptsetup 2.8.8 `bitlkDump`** (https://gitlab.com/cryptsetup/cryptsetup/-/blob/v2.8.8/lib/bitlk/bitlk.c, L218-233, L876-928): prints `Keyslots:`, then per VMK `\tGUID:       \t<lowercase GUID>` and `\tProtection: \t<text>`, the text being one of `VMK protected with clear key`, `… with TPM`, `… with startup key`, `… with TPM and PIN`, `… with passphrase`, `… with recovery passphrase`, `… with smart card`, `VMK with unknown protection`. A clear-key VMK is what Windows adds when BitLocker is suspended (or not yet protected).
- **cryptsetup's BitLocker test images** (https://gitlab.com/cryptsetup/cryptsetup/-/raw/v2.8.8/tests/bitlk-images.tar.xz, 376,840 bytes, SHA-256 `68bf5669f777668112d497234ebe2166b5feaf0e33b206b095b917f82937bd30`): volumes made by Windows, each 100 MiB or 128 MiB raw, with `images.conf` giving each one's VMK GUIDs. `blkid -p` prints `TYPE=BitLocker` for them. Checked locally with cryptsetup 2.8.8:
  - `bitlk-images/bitlk-aes-xts-128.img`: passphrase, and recovery passphrase with VMK GUID `64311dea-4587-4029-924a-ba299647998e`.
  - `bitlk-images/bitlk-aes-xts-128-clearkey-only.img`: only a clear key (suspended, no recovery key).
  - `bitlk-images/bitlk-aes-xts-128-two-recovery.img`: two recovery passphrases. `bitlk-partially-encrypted-aes-cbc-128.img`: passphrase, recovery passphrase and clear key.
  - None has a TPM protector; TPM cases are unit fixtures edited from these dumps.
- **The key ID Windows shows is the VMK GUID.** Windows names a startup-key file by its protector ID: the archive's `AA80A52B-9B66-47AE-B097-33F536FFBB07.BEK` belongs to `bitlk-aes-xts-128-startup-key-win11.img`, whose startup-key VMK GUID in `bitlkDump` is `aa80a52b-9b66-47ae-b097-33f536ffbb07`. Windows' recovery screen and https://aka.ms/myrecoverykey list the recovery key ID, matched by its first 8 characters (Microsoft Q&A, https://learn.microsoft.com/en-us/answers/questions/5637575/how-to-find-bitlocker-recovery-key-with-recovery-k). `manage-bde -protectors -get C:` shows the same IDs (https://learn.microsoft.com/windows-server/administration/windows-commands/manage-bde-protectors).
- **Intel RST** (subiquity `subiquity/server/controllers/storage.py`, `has_rst_GET`, at `d05694b1cd06`): RST "remapped NVMe" shows in `/sys/module/ahci/drivers/pci:ahci/*/remapped_nvme` with a count above 0; Linux cannot see those drives (`drivers/ata/ahci.c`, `remapped_nvme_show`). An Intel Matrix (imsm) RAID member disk has the libblkid type `isw_raid_member`. VMD (11th-generation Core and later) works with the kernel's `vmd` driver and needs nothing from us.
- **Windows in BIOS mode** sits on an MBR (`dos`) table; Microsoft's `MBR2GPT.EXE` converts the drive to GPT so Windows starts in UEFI mode (https://learn.microsoft.com/windows/deployment/mbr-to-gpt).
- **Windows' loader** on an ESP is `EFI/Microsoft/Boot/bootmgfw.efi` with `BCD` beside it, and its firmware entry is "Windows Boot Manager" (UEFI firmware lists, and the Windows-like fixture `installer/fixtures/efibootmgr.txt` line format `Boot0000* Windows Boot Manager\tHD(1,GPT,<guid>,…)/\EFI\Microsoft\Boot\bootmgfw.efi`).
- **Windows' clock**: Windows reads the hardware clock as local time unless `HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation` has the DWORD `RealTimeIsUniversal` = 1; systemd keeps it in UTC by default (https://wiki.archlinux.org/title/System_time#UTC_in_Microsoft_Windows).

## Inputs
From the operator's working plan, item 015 ("Alongside mode: Windows-like disk image fixture, largest-gap pick, BitLocker detection, refusals, before-and-after summary; Windows entry in Limine", IN-01 to IN-07 and BT-05), the parts tasks 012 and 013 left for it, and the earlier decisions it names. Checked against `AGENTS.md` and upstream on 2026-11-07. Nothing is copied into the tree.
- **Already built, unchanged:** IN-01 (largest 1 MiB-aligned gap), IN-03 (before-and-after summary), IN-06's 40 GiB floor (the swap file waits for task 026), IN-08, IN-05 (each live ISO boots with its base's loader).
- **IN-02, BitLocker detection, protectors and typed confirmation (operator decision 2026-10-22, layout chosen 2026-11-07):**
  1. Scan every drive except the live medium, not only the target, for partitions `blkid -p` calls `BitLocker`. Read-only, offline.
  2. Classify each with `cryptsetup bitlkDump`: a clear-key protector means suspended (Windows starts without asking); otherwise a TPM protector means Windows will ask for the recovery key at its next start, because Secure Boot is off; no recovery-passphrase protector means no recovery key exists.
  3. One "BitLocker" screen (operator 2026-11-07, instead of one screen per volume) lists every volume on a drive that is not being erased: drive model, partition, size, state, and each recovery key's ID, with where to find the key: https://aka.ms/myrecoverykey, or https://aka.ms/aadrecoverykey for a work or school account.
  4. A typed confirmation, `I have my key`, continues; Esc goes back.
  5. An armed volume with no recovery key: the screen recommends stopping, turning Secure Boot back on, starting Windows and creating a key (`manage-bde -protectors -add C: -RecoveryPassword`), but the typed confirmation still lets the user go on (operator choice 2026-10-22).
  6. The warning says "will", not "may" (operator choice 2026-10-22): turning Secure Boot off will make Windows ask for its recovery key at its next start unless BitLocker was suspended first.
  7. Deviation from 012's interim rule: BitLocker no longer refuses the free space; the confirmation replaces the refusal. Refusing until decrypted (Omarchy's way) stays rejected, because it closes the free space to most Windows 11 laptops.
- **IN-04, OS detection, narrowed (operator 2026-11-07):** Windows only. A read-only look at every EFI System Partition for `EFI/Microsoft/Boot/bootmgfw.efi`, for Limine's Windows entries and the Confirm screen. An MBR drive with an NTFS or BitLocker partition of type `07` counts as BIOS-mode Windows. Dropped: os-prober, Limine entries for other Linux installs, and the earlier rule that a drive with no OS found offers only the erase (task 012 offers free space whenever it is usable).
- **IN-07, refusals with guidance (operator decision 2026-10-22):** VMD proceeds. An `isw_raid_member` drive is left out in both modes: the firmware must switch from RAID/RST to AHCI, and Windows may stop starting unless it is prepared first (Safe Mode once, or the `bcdboot` procedure). RST remapped NVMe makes drives invisible, so the Drive screen says so with the same guidance. On an MBR drive the free space is refused and the erase allowed; with BIOS-mode Windows the reason names `MBR2GPT.EXE` and UEFI mode.
- **BT-05, the Windows entry (operator decision 2026-10-22):** the installer writes the entries itself, not `limine-scan`. Default "Windows": `protocol: efi_boot_entry`, `entry: Windows Boot Manager` (an extra warm reboot, but BitLocker does not ask after Limine updates). Fallback: `protocol: efi`, `path: guid(<Windows ESP's PARTUUID>):/EFI/Microsoft/Boot/bootmgfw.efi`, named so users know BitLocker may ask for its key. PARTUUID, not the FAT serial. Limine 12.9.2 or newer (both images have 12.9.6). Never copy `EFI/Microsoft` onto our ESP and never touch Windows' firmware entry.
- **From task 013:** the guard that Windows Boot Manager's firmware entry survives the install, and the user docs on putting Stackific OS first again after a large Windows update.
- **HW-03 (operator 2026-11-07, docs only):** the hardware clock stays in UTC, systemd's default, and the installer asks nothing; the Windows page explains `RealTimeIsUniversal`.
- **Test of BT-05 (operator 2026-11-07):** a fake Windows Boot Manager in the VM proves both entries start it. Real Windows with BitLocker and swtpm stays on the real-hardware checklist (task 030), which also checks that Windows asks for no key after a Limine update.

## Design
- **Detection, one module per concern:**
  - `installer/src/bitlocker.rs` (new): parses `bitlkDump` into a volume's protectors; `state()` is `Suspended` (any clear key), `Armed` (any TPM, TPM and PIN protector) or `Locked` (passphrase, smart card or startup key only, which Windows asks for as before); `recovery_ids()` is each recovery-passphrase GUID's first 8 characters in uppercase, as Windows shows them. Scans every whole disk `scan` reads except the live medium and zram: `blkid -p -o export` on each partition, then `cryptsetup bitlkDump <partition>` for each `TYPE=BitLocker`.
  - `installer/src/windows.rs` (new): for each partition of type `C12A7328-F81F-11D2-BA4B-00A0C93EC93B` on those disks, mount it read-only (`mount -o ro,nosuid,nodev,noexec`) at `/run/stackific-installer/esp`, check `EFI/Microsoft/Boot/bootmgfw.efi`, unmount (an ESP already mounted is read where it is). Returns the Windows ESPs' PARTUUIDs. Then, from `efibootmgr`'s listing, whether `BootOrder` holds an entry described exactly "Windows Boot Manager" (case-insensitive, as Limine matches) whose `HD(…)` GUID is one of those ESPs. Runs once when Confirm opens (once in unattended mode); the install uses that result, and an erase leaves out the drive it erases.
  - `installer/src/drives.rs`: `PARTTYPE` and `PARTUUID` join `LSBLK_COLUMNS`. `exclusion()` checks `fstype == "isw_raid_member"` before the in-use checks (md may hold the disk). `free_space()` loses its `bitlocker` argument; on a `dos` table with a type `07` partition whose lsblk FSTYPE is `ntfs` or `BitLocker` the reason is BIOS-mode Windows, otherwise "no GPT partition table" as now. A `remapped_nvme()` reads the sysfs glob through `Runner` (the fake answers it).
- **Screens** (80 × 24, prose at 75 columns, texts as snapshot fixtures):
  - **BitLocker**, between Space and Confirm, only when a volume is on a drive not being erased; its top border reads "5 of 6 · BitLocker" (still step 5). Text, fitted by the build: "Secure Boot is off, so Windows will ask for its BitLocker recovery key the next time it starts, unless BitLocker was suspended first." Then one block per volume: `<partition> <size> on <model>` and `Suspended: Windows starts without asking`, `On: Windows will ask for recovery key <ID>` (all IDs), or `On, and no recovery key exists` followed by the advice in Inputs. Then "Find your key at aka.ms/myrecoverykey (work or school: aka.ms/aadrecoverykey).", the typed field, and "Type I have my key to go on." Enter is ready only when the field is exactly `I have my key`. More volumes than fit scroll as a list.
  - **Drive**: an `isw_raid_member` drive's reason is "part of an Intel RST RAID volume: set the firmware's storage mode to AHCI"; when `remapped_nvme` is above 0, one line under the list: "Intel RST hides NVMe drives: set the firmware's storage mode to AHCI, and prepare Windows first, or it may stop starting."
  - **Space**: the help text under the options lists the reasons as they now are (BitLocker leaves the list; BIOS-mode Windows joins).
  - **Confirm**: with Windows found, one more line: "Limine's menu will also start Windows."
- **Unattended:** `[disk] bitlocker = "I have my key"` is required when the BitLocker screen would show; without it the run exits 2 and names each volume and its state on stderr. A value other than the phrase is refused.
- **Limine's menu:** `boot::windows_entries(espuuids, has_nvram_entry) -> String`, shared by both bases, appended after the system's entries:

  ```text

  /Windows
      protocol: efi_boot_entry
      entry: Windows Boot Manager
      comment: Restarts into Windows through the firmware, so BitLocker does not ask for its key.

  /Windows, started directly
      protocol: efi
      path: guid(<PARTUUID>):/EFI/Microsoft/Boot/bootmgfw.efi
      comment: For when "Windows" fails. BitLocker may ask for its recovery key.
  ```

  "Windows" only when the firmware has the entry at install (otherwise Limine would panic); one "started directly" entry per Windows ESP, numbered "(2)", "(3)" after the first. Nothing when no Windows is found.
  - Arch: the installer appends it to `limine.conf` (static, written once).
  - Fedora: the installer writes it to `<target>/etc/kernel/limine-windows.conf` before `kernel-install` runs; `90-stackific-limine.install` appends that file, when present, after the kernel entries each time it writes `limine.conf`. `stackific-boot`'s `Release` goes up.
- **Firmware entry:** unchanged; `efibootmgr --create` keeps "Windows Boot Manager" in `BootOrder` after ours, and `stale()` never touches it. A test proves it.
- **Packages:** Fedora's `stackific-installer.spec` gains `Requires: cryptsetup` and `mount` comes from `util-linux`, already required; Arch's PKGBUILD gains `cryptsetup` in `depends` and a `pkgrel` bump; the Fedora spec's `Release` goes up.

## Changes
- `installer/src/bitlocker.rs` (new), `installer/src/windows.rs` (new), with unit tests.
- `installer/src/drives.rs`: lsblk columns, the RST exclusion, `free_space` without BitLocker and with the BIOS-mode reason, `remapped_nvme`; tests.
- `installer/src/boot.rs`: `windows_entries`; test of its exact text.
- `installer/src/base.rs`, `installer/src/install.rs`: Arch appends the entries to `limine.conf`, Fedora writes `/etc/kernel/limine-windows.conf` before `kernel-install`; tests for both bases.
- `installer/src/answers.rs`: `[disk] bitlocker`, its check; tests.
- `installer/src/main.rs` (if the unattended path needs it), `installer/src/demo.rs`: the demo drive gets a BitLocker volume and a Windows ESP, so `just installer demo` shows the BitLocker screen and the Confirm line.
- `installer/src/ui/bitlocker.rs` (new), `installer/src/ui/mod.rs`, `installer/src/ui/drive.rs`, `installer/src/ui/space.rs`, `installer/src/ui/confirm.rs`, `installer/src/ui/tests.rs`.
- `installer/fixtures/`: `bitlkdump-recovery.txt`, `bitlkdump-two-recovery.txt`, `bitlkdump-clearkey.txt` (each `cryptsetup bitlkDump` of the image named above), `bitlkdump-tpm-no-recovery.txt` (edited from the first: TPM, no recovery passphrase, and the edit said in a comment in the test), an lsblk listing with an `isw_raid_member` drive, an MBR Windows `sfdisk --json`, an `efibootmgr` listing with "Windows Boot Manager"; `installer/fixtures/screens/bitlocker.txt`, `bitlocker-no-key.txt`, `drive-rst.txt` (new), `space.txt`, `confirm-free-space.txt` (changed).
- `arch/packages/stackific-installer/PKGBUILD`: `cryptsetup`, `pkgrel`.
- `fedora/packages/stackific-installer/stackific-installer.spec`: `Requires: cryptsetup`, `Release`.
- `fedora/packages/stackific-boot/90-stackific-limine.install`, `fedora/packages/stackific-boot/stackific-boot.spec`: append `/etc/kernel/limine-windows.conf`; `Release`.
- `tests/vm/boot.sh`: `vm_put <file> <guest path>`, which sends a file through the test shell (the token line, a command such as `head -c <size> > <path>`, then the bytes; bash reads its commands from a socket one byte at a time, so the rest reaches `head`).
- `tests/vm/install.sh`, `tests/vm/installed.sh`: the tests below.
- `tests/vm/screens.sh`: the BitLocker screen, the Drive screen with an RST member, and Confirm with Windows.
- `arch/justfile`, `fedora/justfile` (`test-install`, and `arch screenshots`): download cryptsetup's test images once to `.build/tests/bitlk-images.tar.xz`, checked against the SHA-256 pin, and hand the file to the tests. The URL and checksum are pinned once, in a file both justfiles import (`tests/vars.just`, new).
- `docs/installer.md`: BitLocker, Windows detection, the refusals, Limine's Windows entries and `/etc/kernel/limine-windows.conf`, the tests; "Later" drops task 015's line.
- `site/src/content/docs/user/windows.md` (new), "Next to Windows":
  - before you start: save the BitLocker recovery key (aka.ms/myrecoverykey), know the Windows password, turn off Fast Startup, suspend BitLocker, then turn Secure Boot off in the same restart;
  - making free space (Disk Management);
  - what the installer shows;
  - starting Windows from Limine ("Windows" restarts once into it; "Windows, started directly" when that fails);
  - putting Stackific OS first again after a large Windows update (the firmware's setup, or `sudo efibootmgr --bootorder <ours>,<the rest>`);
  - RST and BIOS-mode Windows (AHCI, `MBR2GPT.EXE`);
  - Windows' clock (`reg add "HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation" /v RealTimeIsUniversal /d 1 /t REG_DWORD /f` in an administrator terminal).
- `site/src/content/docs/user/installation.md`: links the new page; "Limits" keeps the backup advice.
- `release/notes.md`: the installer works next to Windows and BitLocker.
- `AGENTS.md`: the installer line (BitLocker screen and phrase, Windows entries, RST and BIOS-mode refusals, no os-prober); the boot chain line (Windows entries; Fedora's plugin appends `/etc/kernel/limine-windows.conf`); the tests line (cryptsetup's BitLocker images, the fake Windows Boot Manager).
- `site/src/content/docs/prompt-trail/015-installer-windows.md` (new) and `site/src/assets/prompt-trail/015/*.webp` (new): the BitLocker screen, the Drive screen with an RST member, Confirm with the Windows line, Limine's menu with the Windows entries.
- `dev/reviews/015-installer-windows.md` (new).

## Acceptance criteria

Must:
- `just installer test` and `just installer lint` pass. Unit tests cover:
  - each `bitlkDump` fixture's state and key IDs (`64311DEA` for `bitlk-aes-xts-128`; two IDs for two-recovery; suspended for clear key; armed with no key for the TPM fixture);
  - the RST exclusion and its order before the in-use check, `remapped_nvme` above and at 0;
  - the BIOS-mode reason on an MBR Windows table and "no GPT partition table" on an MBR table without Windows;
  - free space next to BitLocker allowed;
  - `windows_entries` for no Windows, one ESP with and without the firmware entry, and two ESPs;
  - the Fedora file and the Arch `limine.conf` text;
  - the unattended phrase required, wrong and right.

  The snapshots match the new and changed screen fixtures. `just installer demo` shows the BitLocker screen and runs to Done.
- `just arch packages` and `just fedora packages` build; `rpm -qpR` of `stackific-installer` lists `cryptsetup`; the PKGBUILD's `depends` has it.
- `just arch iso`, `just fedora iso` build and print their sizes; the Fedora manifest lists `cryptsetup`.
- `just arch test-install` and `just fedora test-install` pass, each with:
  - **MBR Windows:** `/dev/vda` with a `dos` table, partition 1 type `07` holding `bitlk-aes-xts-128.img`. An unattended free-space run exits 2 with `MBR2GPT` on stderr, and `sfdisk --dump` is unchanged. (The erase earlier in the run already proves an MBR drive can be erased.)
  - **BitLocker on the Windows-like GPT:** `bitlk-aes-xts-128.img` written to partition 3 (replacing the fake signature). The ESP holds `EFI/Microsoft/Boot/bootmgfw.efi` (a copy of the image's Limine program) and an empty `BCD`, and the firmware has "Windows Boot Manager" for it (`efibootmgr --create --disk /dev/vda --part 1 --label "Windows Boot Manager" --loader '\EFI\Microsoft\Boot\bootmgfw.efi'`, made before the install).
    - A free-space run without `bitlocker` exits 2, names `vda3` and `64311DEA` on stderr, and changes nothing.
    - The run with `bitlocker = "I have my key"` installs. Every old entry stays byte-identical, partition 3's first 100 MiB have the same SHA-256 as the image, and the Windows ESP's file list and checksums are unchanged.
    - After the install, `efibootmgr` shows `BootOrder` starting with "Stackific OS", then "Windows Boot Manager".
  - **The installed boot:** `limine.conf` ends with the "Windows" and "Windows, started directly" entries naming partition 1's PARTUUID. On Fedora they are still there after the plugin's `kernel-install add` and `remove` and the offline kernel reinstall.
  - **Both Windows entries start the fake Windows:** the test then adds a `limine.conf` beside the fake `bootmgfw.efi`, whose one entry starts the installed kernel from `guid(<our ESP's PARTUUID>)` with `stackific.test=windows` added. Picking "Windows" in Limine's menu (key presses) restarts into it: `/proc/cmdline` has the marker and `efibootmgr`'s `BootCurrent` is the "Windows Boot Manager" entry. Picking "Windows, started directly" also reaches the marker, with `BootCurrent` the "Stackific OS" entry.
  - `limine.png` shows the Windows entries below Stackific OS.
- `just arch screenshots` saves the BitLocker screen, the Drive screen with an `isw_raid_member` drive (an imsm container made in the guest with `IMSM_NO_PLATFORM=1 mdadm --create … --metadata=imsm`, then stopped and wiped), and Confirm with the Windows line. The remapped-NVMe line appears only in its snapshot fixture, because a VM cannot fake it, and the prompt trail says so.
- `just arch test` and `just fedora test` still pass; `just lefthook all` passes (shellcheck on the plugin and the scripts, `just dev check`, the site's checks).

Must not:
- write to any partition, ESP or firmware entry other than our two new partitions and our own firmware entry: no `EFI/Microsoft` on our ESP, no change to "Windows Boot Manager" or `BootOrder` beyond `efibootmgr --create`;
- mount any partition read-write outside the target, mount NTFS at all, or unlock BitLocker;
- use os-prober, `limine-scan`, limine-entry-tool, mdadm assembly or any AUR or COPR package;
- refuse the free space for BitLocker, or change when the free space or the erase is offered other than the RST and BIOS-mode reasons;
- add the installer or Windows support to either aarch64 image (task 018);
- touch files outside the Changes list, or add a dependency this task does not name (`cryptsetup` is the one).

## Tests
- Unit, snapshot and demo: `just installer test`, `just installer lint`, `just installer demo`.
- Packages and images: `just arch packages`, `just arch iso`, `just arch test`, `just fedora packages`, `just fedora iso`, `just fedora test` on the PC.
- Install and installed boot: `just arch test-install`, `just fedora test-install` on the PC. In CI, the x86_64 jobs of `images` and `release`.
- Screenshots for the prompt trail: `just arch screenshots`, plus `limine.png` from `just arch test-install`, each through `just site screenshot`.
- By hand (operator), on a real computer with Windows 11, because only real Windows, its TPM and its firmware show BitLocker's prompt:
  1. In Windows, follow the new "Next to Windows" page: save the recovery key, turn off Fast Startup, suspend BitLocker, shrink C: in Disk Management by at least 40 GiB.
  2. Restart into the firmware's setup, turn Secure Boot off, and boot the x86_64 ISO from the `images` workflow's test release (`gh workflow run images --ref main`, then `v0.0.0-pr.<n>`).
  3. Open "Install Stackific OS". On the BitLocker screen, compare the key ID with the one at aka.ms/myrecoverykey, type `I have my key`, and install into the free space.
  4. Restart. In Limine's menu, pick "Windows": the computer restarts once and Windows starts, without asking for the recovery key if BitLocker was suspended.
  5. In Stackific OS, run `sudo pacman -S limine` (Arch) or `sudo dnf5 reinstall limine` (Fedora), restart, and pick "Windows" again: Windows must not ask for the recovery key.
  6. Report the computer's make and model, its storage mode (AHCI, RST or VMD), and any step that differed.
