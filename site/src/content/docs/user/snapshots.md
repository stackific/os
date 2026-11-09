---
title: Snapshots and restore
description: How Stackific OS keeps snapshots of the system, how to undo an update, and how to restore a system that no longer starts from the USB stick.
sidebar:
  order: 5
---

Stackific OS keeps snapshots of the system with snapper, and Btrfs Assistant restores them. A snapshot is a frozen copy of the system as it was at one moment; it takes almost no space until the system changes.

## What is kept

- **Fresh install**: the system exactly as the installer left it. It is the first snapshot and is marked important.
- **Before and after every update**: each time packages are installed, updated or removed, a snapshot is taken before and one after.
- Snapper keeps up to 10 snapshots (and up to 10 important ones, such as those around kernel updates). When less than a fifth of the disk is free, it deletes older ones, down to 2 (4 important), so snapshots never fill the disk.

Snapshots cover the system, not your files: your home folder, logs, caches, Flatpak apps, virtual machines and containers are left out, so restoring a snapshot never takes back your own work. Snapshots are not a backup either: they are on the same disk as the system. Keep copies of your files on another drive.

## Undo an update

1. Open **Btrfs Assistant** from the app launcher and enter your password.
2. On the **Snapper** tab, choose **Restore**, then the `root` config.
3. Pick the snapshot from before the update (its description names the command that ran) and choose **Restore**.
4. Restart. The system is back as it was; the restored-over system is kept as a backup subvolume, which Btrfs Assistant's **Subvolumes** tab can delete.

## Restore a system that no longer starts

Start the computer from the Stackific OS USB stick and choose **Try**. Your disk is encrypted, so unlock it first:

1. Open **Disks**, select your drive and the large encrypted partition, choose the unlock button and enter your login password. Or, in a terminal: `sudo cryptsetup open /dev/<partition> restore` (Disks shows the partition's name, such as `nvme0n1p6`).
2. Tell Btrfs Assistant where the snapshots of the system are. In a terminal, with the name of the unlocked device (`/dev/mapper/restore`, or the one Disks shows):

   ```sh
   echo "root = \"@snapshots,@,$(sudo blkid -s UUID -o value /dev/mapper/restore)\"" | sudo tee -a /etc/btrfs-assistant.conf
   ```

3. Open **Btrfs Assistant**, choose the **Snapper** tab, then **Restore**, pick a snapshot (for example **Fresh install**, or the one before the last update) and choose **Restore**.
4. Restart and take out the USB stick.

The live session forgets the line in step 2 when it shuts down.

## Swap

Stackific OS swaps to compressed memory (zram) as large as the computer's RAM, and to no swap file. Hibernation is not available.
