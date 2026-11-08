---
title: Next to Windows
description: Prepare Windows and BitLocker, install Stackific OS into the free space, start Windows from the boot menu, and keep both clocks right.
sidebar:
  order: 3
---

The installer puts Stackific OS into a drive's free space and leaves Windows as it is: its partitions, its EFI System Partition and its own boot entry. Stackific OS gets its own 4 GiB boot partition, and its boot menu can start Windows. Read this page in Windows, before you start.

## Before you start

:::caution
Turning Secure Boot off will make Windows ask for its BitLocker recovery key the next time it starts, unless you suspend BitLocker first. Save your key before you start.
:::

The ISOs start only with Secure Boot off. Most Windows 11 computers encrypt their drive with BitLocker ("device encryption"), and BitLocker notices the change. In Windows:

1. Save your BitLocker recovery key. A Microsoft account keeps it at https://aka.ms/myrecoverykey (a work or school account at https://aka.ms/aadrecoverykey). Note its key ID and the 48-digit key, somewhere other than this computer. If no key is listed, open an administrator terminal and run `manage-bde -protectors -add C: -RecoveryPassword`, then save the key it prints.
2. Make sure you know your Windows password: Windows Hello (a PIN, a face or a fingerprint) may not work after the recovery key.
3. Turn off Fast Startup: Control Panel, Power Options, "Choose what the power buttons do", "Turn on fast startup (recommended)". Fast Startup leaves Windows' drive half-closed, so Stackific OS should not open it.
4. Make free space: in Disk Management, right-click the Windows partition (C:), choose "Shrink Volume", and shrink it by at least 40 GiB. Leave the space unallocated.
5. Suspend BitLocker for one restart: in an administrator terminal, `manage-bde -protectors -disable C: -RebootCount 1`.
6. Restart into the firmware's setup (often F2 or Del at power-on), turn Secure Boot off, and start the USB stick from the firmware's boot menu.

## What the installer shows

The installer looks at every drive it lists for BitLocker, without unlocking anything. When it finds BitLocker on a drive it keeps, a BitLocker screen comes before the last one and lists each volume:

- **Suspended**: Windows starts without asking for the key.
- **On: Windows will ask for recovery key `<ID>`**: have that key ready. The ID is the first eight characters of the key ID that https://aka.ms/myrecoverykey lists.
- **On, and no recovery key exists**: stop here. Turn Secure Boot back on, start Windows and make a key, as in step 1 above. You can still go on, at the risk of a Windows you cannot unlock.

Type `I have my key` to go on. An unattended install takes the same text as `bitlocker = "I have my key"` under `[disk]`.

The last screen says "Limine's menu will also start Windows" when the installer found Windows' boot files.

Two setups cannot take Stackific OS next to Windows:

- **Intel RST (RAID) storage mode.** The installer leaves out drives in an Intel RST RAID volume, and says so when RST hides NVMe drives. Set the firmware's storage mode (SATA mode) to AHCI. Prepare Windows first, or it may stop starting: run `bcdedit /set {current} safeboot minimal` in an administrator terminal, change the mode, start Windows once in Safe Mode, then run `bcdedit /deletevalue {current} safeboot` and restart.
- **Windows in legacy BIOS mode**, on a drive with an MBR partition table. Convert the drive in Windows with Microsoft's `MBR2GPT.EXE` (`mbr2gpt /convert /allowFullOS`), then switch the firmware to UEFI mode.

Both can still erase a whole drive, which removes Windows from it.

## Starting Windows

Stackific OS's boot menu lists, after Stackific OS:

- **Windows** restarts the computer once and lets the firmware start Windows, as it would without Stackific OS. BitLocker sees no change, so it does not ask for the key.
- **Windows, started directly** starts Windows' loader from the menu, for when "Windows" fails (when Windows' own firmware entry is gone). BitLocker may then ask for its recovery key.

BitLocker suspended with `-RebootCount 1` turns its protection back on by itself once Windows has started. If you suspended it without a count, turn it back on in an administrator terminal: `manage-bde -protectors -enable C:`.

## Putting Stackific OS first again

The installer puts Stackific OS first in the firmware's boot order. A large Windows update can put Windows first again. To start Stackific OS once, open the firmware's boot menu (often F12, F11, F8 or Esc at power-on) and pick "Stackific OS". To put it first again, either:

- in the firmware's setup, move "Stackific OS" to the top of the boot order; or
- in Stackific OS, run `sudo efibootmgr`, note the four-digit number of "Stackific OS" in its list, and run `sudo efibootmgr --bootorder <that number>,<the rest of BootOrder>`, keeping every other number in its order.

## Windows' clock

Stackific OS keeps the hardware clock in UTC, as Linux does, while Windows reads it as local time, so after switching one of them shows the wrong time. Tell Windows to read UTC too: in an administrator terminal in Windows, run

```bat
reg add "HKLM\SYSTEM\CurrentControlSet\Control\TimeZoneInformation" /v RealTimeIsUniversal /d 1 /t REG_DWORD /f
```

and restart Windows.
