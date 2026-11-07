//! The shared boot step: Limine's EFI program on our ESP, its `limine.conf`, and the firmware's entry for it. The
//! kernel and initramfs come first, from the base (`Base::initramfs`). The firmware entry is written here only, at
//! install: nothing writes NVRAM at boot or on updates.

use std::path::Path;

use crate::run::Cmd;

/// The firmware entry's loader on our ESP, in UEFI's notation.
pub const LOADER: &str = r"\EFI\limine\BOOTX64.EFI";

/// Limine's EFI program in the target, as the limine package installs it.
const PROGRAM: &str = "usr/share/limine/BOOTX64.EFI";

/// Where the program goes under the target: the firmware entry's loader, then the firmware's fallback path, which
/// starts the drive when its entry is lost. stackific-boot's pacman hook keeps both current.
const COPIES: [&str; 2] = ["boot/EFI/limine/BOOTX64.EFI", "boot/EFI/BOOT/BOOTX64.EFI"];

/// `limine.conf` under the target: the ESP's root, which Limine searches after the program's own folder, so one file
/// serves both copies.
pub const CONF: &str = "boot/limine.conf";

/// The commands that copy Limine's program from the target's limine package to both places on the ESP.
pub fn copy_program(target: &Path) -> Vec<Cmd> {
    let source = target.join(PROGRAM).to_string_lossy().to_string();
    COPIES
        .iter()
        .map(|c| {
            Cmd::new("install").args([
                "-D",
                "-m",
                "0644",
                &source,
                &target.join(c).to_string_lossy(),
            ])
        })
        .collect()
}

/// Limine's menu: one entry named `name` that starts the base's kernel and initramfs from the ESP, with root on the
/// `@` subvolume of the system partition `root_uuid`.
pub fn limine_conf(name: &str, kernel: &str, initramfs: &str, root_uuid: &str) -> String {
    format!(
        "timeout: 3\ninterface_branding: {name}\n\n/{name}\n    protocol: linux\n    path: boot():/{kernel}\n    \
         module_path: boot():/{initramfs}\n    cmdline: root=UUID={root_uuid} rootflags=subvol=/@ rw\n"
    )
}

/// Every partition's GUID, as `lsblk --noheadings --list --output PARTUUID` prints them.
pub fn partuuids() -> Cmd {
    Cmd::new("lsblk").args(["--noheadings", "--list", "--output", "PARTUUID"])
}

/// The firmware's entries, with their device paths (efibootmgr 18 prints them by default).
pub fn entries() -> Cmd {
    Cmd::new("efibootmgr")
}

/// The boot numbers of the entries that start our loader from a partition this computer no longer has (an earlier
/// install that a reinstall wiped). `listing` is efibootmgr's, `present` lsblk's PARTUUID list.
pub fn stale(listing: &str, present: &str) -> Vec<String> {
    let present: Vec<String> = present.split_whitespace().map(str::to_lowercase).collect();
    listing
        .lines()
        .filter_map(|l| {
            let num = l.strip_prefix("Boot")?.get(..4)?;
            // `HD(<n>,GPT,<partition GUID>,<start>,<size>)/<file>`; firmware may append optional data to the file
            let hd = &l[l.find("HD(")? + 3..];
            let (fields, file) = hd.split_once(")/")?;
            let guid = fields.split(',').nth(2)?.to_lowercase();
            let ours = file.to_lowercase().starts_with(&LOADER.to_lowercase());
            (ours && !present.contains(&guid)).then(|| num.to_string())
        })
        .collect()
}

/// Deletes the firmware entry `num`.
pub fn delete(num: &str) -> Cmd {
    Cmd::new("efibootmgr").args(["--bootnum", num, "--delete-bootnum"])
}

/// Creates the firmware entry `label` for our loader on partition `part` of `drive`; efibootmgr puts it first in the
/// boot order and keeps every other entry after it.
pub fn create(drive: &str, part: u32, label: &str) -> Cmd {
    Cmd::new("efibootmgr").args([
        "--create",
        "--disk",
        drive,
        "--part",
        &part.to_string(),
        "--label",
        label,
        "--loader",
        LOADER,
    ])
}

/// A partition's number from its path: the kernel names partitions `<disk><n>` or `<disk>p<n>` (`/dev/vda4`,
/// `/dev/nvme0n1p5`).
pub fn partition_number(path: &str) -> Option<u32> {
    let digits = path.len() - path.trim_end_matches(|c: char| c.is_ascii_digit()).len();
    path[path.len() - digits..].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// efibootmgr's listing in the test VM (OVMF) before the free-space install: the erase install's entry, Boot0009,
    /// on a partition the Windows-like table replaced.
    const LISTING: &str = include_str!("../fixtures/efibootmgr.txt");

    #[test]
    fn limine_conf_starts_the_kernel_on_the_at_subvolume() {
        assert_eq!(
            limine_conf(
                "Stackific OS",
                "vmlinuz-linux",
                "initramfs-linux.img",
                "1111-sys"
            ),
            "timeout: 3\ninterface_branding: Stackific OS\n\n/Stackific OS\n    protocol: linux\n    \
             path: boot():/vmlinuz-linux\n    module_path: boot():/initramfs-linux.img\n    \
             cmdline: root=UUID=1111-sys rootflags=subvol=/@ rw\n"
        );
    }

    #[test]
    fn only_our_loader_on_a_missing_partition_is_stale() {
        let present = "\n\nGUID-PRESENT\n";
        let gone = LISTING
            .lines()
            .find(|l| l.contains("BOOTX64.EFI") && l.contains("HD("))
            .unwrap();
        assert!(gone.starts_with("Boot"));
        let stale_nums = stale(LISTING, present);
        assert_eq!(stale_nums.len(), 1, "{stale_nums:?}");
        assert!(!stale(LISTING, &format!("{present}\n{}", guid_of(gone))).contains(&stale_nums[0]));
        // Other entries (the firmware's own, another OS's) are never stale
        assert!(stale("Boot0000* Windows Boot Manager\tHD(1,GPT,abc,0x800,0x1)/\\EFI\\Microsoft\\Boot\\bootmgfw.efi\n", "").is_empty());
        assert!(stale("BootOrder: 0001\nBootCurrent: 0001\n", "").is_empty());
    }

    fn guid_of(line: &str) -> String {
        let hd = &line[line.find("HD(").unwrap() + 3..];
        hd.split(',').nth(2).unwrap().to_string()
    }

    #[test]
    fn commands() {
        assert_eq!(
            create("/dev/vda", 4, "Stackific OS").line(),
            r#"efibootmgr --create --disk /dev/vda --part 4 --label "Stackific OS" --loader \EFI\limine\BOOTX64.EFI"#
        );
        assert_eq!(
            delete("0004").line(),
            "efibootmgr --bootnum 0004 --delete-bootnum"
        );
        let copies: Vec<_> = copy_program(Path::new("/t"))
            .iter()
            .map(Cmd::line)
            .collect();
        assert_eq!(
            copies,
            [
                "install -D -m 0644 /t/usr/share/limine/BOOTX64.EFI /t/boot/EFI/limine/BOOTX64.EFI",
                "install -D -m 0644 /t/usr/share/limine/BOOTX64.EFI /t/boot/EFI/BOOT/BOOTX64.EFI"
            ]
        );
        assert_eq!(partition_number("/dev/vda4"), Some(4));
        assert_eq!(partition_number("/dev/nvme0n1p12"), Some(12));
        assert_eq!(partition_number("/dev/vda"), None);
    }
}
