//! The shared disk step: our two partitions (a 4 GiB FAT32 EFI System Partition and the btrfs system partition),
//! either on a new GPT or appended into the chosen gap, then formatting, the subvolume layout (SN-01), the mounts
//! under the target and the fstab that matches them.

use std::path::{Path, PathBuf};

use crate::drives::Gap;
use crate::run::Cmd;

/// GPT type GUIDs (UEFI specification; util-linux's list).
pub const ESP_TYPE: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";
pub const LINUX_TYPE: &str = "0FC63DAF-8483-4772-8E79-3D69D8477DE4";
/// Our ESP's size (BT-01): room for Limine, several kernels and their initramfs files.
pub const ESP_BYTES: u64 = 4 << 30;
/// Where the new system is mounted while it is written.
pub const TARGET: &str = "/run/stackific-installer/target";
/// Every subvolume is mounted with these (SN-01).
pub const MOUNT_OPTIONS: &str = "noatime,compress=zstd:1";

/// The flat subvolume layout (SN-01): every one a top-level sibling of `@`, and its mount point. `@snapshots` holds
/// snapper's snapshots of `@` (task 026), so restoring `@` keeps them.
pub const SUBVOLUMES: [(&str, &str); 13] = [
    ("@", "/"),
    ("@home", "/home"),
    ("@root", "/root"),
    ("@srv", "/srv"),
    ("@cache", "/var/cache"),
    ("@tmp", "/var/tmp"),
    ("@log", "/var/log"),
    ("@swap", "/swap"),
    ("@flatpak", "/var/lib/flatpak"),
    ("@libvirt", "/var/lib/libvirt/images"),
    ("@containers", "/var/lib/containers"),
    ("@docker", "/var/lib/docker"),
    ("@snapshots", "/.snapshots"),
];
/// NOCOW subvolumes (`chattr +C` on their empty roots): VM images and the future swap file, which btrfs then never
/// compresses or copies on write.
pub const NOCOW: [&str; 2] = ["@libvirt", "@swap"];

/// The two new partitions' places on the disk, in sectors: (start, size), the ESP first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub esp: (u64, u64),
    pub system: (u64, u64),
}

impl Layout {
    /// Both partitions inside an aligned gap: the ESP at its start, the system partition to its end.
    pub fn in_gap(gap: &Gap) -> Self {
        let esp = ESP_BYTES / gap.sector;
        Self {
            esp: (gap.start, esp),
            system: (gap.start + esp, gap.end - gap.start - esp),
        }
    }
}

/// The FAT label: the os-release ID in capitals, at most 11 characters (FAT's limit).
pub fn fat_label(id: &str) -> String {
    id.to_uppercase().chars().take(11).collect()
}

/// The partition names: `<NAME> boot` and `<NAME>`.
fn names(name: &str) -> (String, String) {
    (format!("{name} boot"), name.to_string())
}

/// The two partitions' lines of an sfdisk script, at explicit sectors: sfdisk aligns only values it picks itself.
fn lines(name: &str, layout: &Layout) -> String {
    let (esp, sys) = names(name);
    format!(
        "start={}, size={}, type={ESP_TYPE}, name=\"{esp}\"\nstart={}, size={}, type={LINUX_TYPE}, name=\"{sys}\"\n",
        layout.esp.0, layout.esp.1, layout.system.0, layout.system.1
    )
}

/// The sfdisk command that erases the drive: a new GPT whose first usable sector is the ESP's start, then both
/// partitions as `erase_layout` places them. `--wipe always` clears old signatures on the disk,
/// `--wipe-partitions always` inside the new partitions.
pub fn erase(device: &str, name: &str, layout: &Layout) -> Cmd {
    let script = format!(
        "label: gpt\nfirst-lba: {}\n{}",
        layout.esp.0,
        lines(name, layout)
    );
    Cmd::new("sfdisk")
        .args(["--wipe", "always", "--wipe-partitions", "always", device])
        .stdin(script)
}

/// The sfdisk command that appends both partitions inside the gap, in one write. `--append` keeps every existing
/// entry as it is, `--wipe never` keeps the disk's signatures, and `--wipe-partitions always` clears old signatures
/// only inside the new partitions.
pub fn append(device: &str, name: &str, layout: &Layout) -> Cmd {
    Cmd::new("sfdisk")
        .args([
            "--append",
            "--wipe",
            "never",
            "--wipe-partitions",
            "always",
            device,
        ])
        .stdin(lines(name, layout))
}

/// Where an install puts the two partitions on `drive` in `mode`, or why it cannot.
pub fn layout(drive: &crate::drives::Drive, mode: crate::answers::Mode) -> Result<Layout, String> {
    match mode {
        crate::answers::Mode::FreeSpace => drive
            .free
            .as_ref()
            .map(Layout::in_gap)
            .map_err(Clone::clone),
        crate::answers::Mode::Erase => Ok(erase_layout(drive.size, drive.sector)),
    }
}

/// Where an erase puts the partitions on a drive of `size` bytes with `sector`-byte sectors: the first at 1 MiB, the
/// last ending at the last usable sector (before the backup GPT) rounded down to 1 MiB.
pub fn erase_layout(size: u64, sector: u64) -> Layout {
    let grain = ALIGN / sector;
    let last_usable = size / sector - 2 - (16384 / sector);
    let end = (last_usable + 1) / grain * grain;
    let esp = ESP_BYTES / sector;
    Layout {
        esp: (grain, esp),
        system: (grain + esp, end - grain - esp),
    }
}

/// New partitions start on this boundary (IN-01).
const ALIGN: u64 = crate::drives::ALIGN_BYTES;

/// Finds the new partitions' device paths in `lsblk --json --output NAME,PATH,TYPE,SIZE,START <device>` by where they
/// start, in bytes; lsblk counts START in 512-byte sectors whatever the drive's sector size.
pub fn find_by_start(lsblk_json: &str, starts: [u64; 2]) -> Result<[String; 2], String> {
    let devs = crate::drives::parse_lsblk(lsblk_json)?;
    let parts: Vec<_> = devs.iter().flat_map(|d| &d.children).collect();
    let find = |s: u64| {
        let at = parts
            .iter()
            .find(|p| p.start.is_some_and(|start| start * 512 == s));
        at.map(|p| p.path.clone())
            .ok_or(format!("no new partition starts at byte {s}"))
    };
    Ok([find(starts[0])?, find(starts[1])?])
}

/// The system partition's LUKS2 mapping while the installer writes it (BT-09); the installed system's initramfs opens
/// it as `luks-<uuid>` itself, so the name only has to be fixed for the clean-up.
pub const MAPPER: &str = "stackific-installer-root";
/// The mapping's device, where btrfs goes.
pub const MAPPER_PATH: &str = "/dev/mapper/stackific-installer-root";

/// The commands that put LUKS2 on the system partition with cryptsetup's defaults (argon2id, 2 s) and open it at
/// MAPPER_PATH. `--key-file -` reads stdin to its end, a newline included, so the passphrase goes in without one.
pub fn encrypt(system: &str, passphrase: &str) -> [Cmd; 2] {
    [
        Cmd::new("cryptsetup")
            .args([
                "luksFormat",
                "--type",
                "luks2",
                "--batch-mode",
                "--key-file",
                "-",
                system,
            ])
            .stdin(passphrase.to_string()),
        Cmd::new("cryptsetup")
            .args(["open", "--key-file", "-", system, MAPPER])
            .stdin(passphrase.to_string()),
    ]
}

/// /etc/crypttab: the system partition's LUKS2 container by UUID, named as the initramfs names it, with no key file
/// (the passphrase is asked). Fedora's dracut copies it into the initramfs and then adds no unlock job of its own
/// beside systemd's (that job fails with an error line next to the prompt); elsewhere it only documents the container.
pub fn crypttab(luks_uuid: &str) -> String {
    format!(
        "# /etc/crypttab, written by stackific-installer: the system partition's LUKS2 container\n\
         luks-{luks_uuid} UUID={luks_uuid} none\n"
    )
}

/// The mkfs commands for the ESP and the system partition (the LUKS2 mapping on it).
pub fn format(esp: &str, system: &str, id: &str) -> [Cmd; 2] {
    [
        Cmd::new("mkfs.fat").args(["-F", "32", "-n", &fat_label(id), esp]),
        Cmd::new("mkfs.btrfs").args(["-f", "-L", id, system]),
    ]
}

/// The path of a mount point under the target.
pub fn under(target: &Path, mount_point: &str) -> PathBuf {
    target.join(mount_point.trim_start_matches('/'))
}

/// Creates the subvolumes on the system partition's top level (mounted at the target for the moment), marks the
/// NOCOW ones and keeps users out of the snapshots (snapper's own mode for `.snapshots`), then mounts `@`, every other subvolume and the ESP, in that order, under the target.
pub fn prepare(esp: &str, system: &str, target: &Path) -> Vec<Cmd> {
    let t = target.to_string_lossy().to_string();
    let top = |v: &str| format!("{t}/{v}");
    let mut cmds = vec![
        Cmd::new("mkdir").args(["-p", &t]),
        Cmd::new("mount").args(["-o", "subvolid=5", system, &t]),
        Cmd::new("btrfs")
            .args(["subvolume", "create"])
            .args(SUBVOLUMES.iter().map(|(v, _)| top(v))),
        Cmd::new("chattr")
            .args(["+C"])
            .args(NOCOW.iter().map(|v| top(v))),
        Cmd::new("chmod").args(["750", &top("@snapshots")]),
        Cmd::new("umount").args([&t]),
        Cmd::new("mount").args(["-o", &format!("subvol=/@,{MOUNT_OPTIONS}"), system, &t]),
    ];
    let dirs = SUBVOLUMES.iter().skip(1).map(|(_, m)| *m).chain(["/boot"]);
    cmds.push(
        Cmd::new("mkdir")
            .args(["-p"])
            .args(dirs.map(|m| under(target, m).to_string_lossy().to_string())),
    );
    for (v, m) in SUBVOLUMES.iter().skip(1) {
        cmds.push(
            Cmd::new("mount")
                .args(["-o", &format!("subvol=/{v},{MOUNT_OPTIONS}"), system])
                .args([under(target, m).to_string_lossy()]),
        );
    }
    cmds.push(
        Cmd::new("mount")
            .args(["-o", "umask=0077", esp])
            .args([under(target, "/boot").to_string_lossy()]),
    );
    cmds
}

/// /etc/fstab by UUID, so systemd-gpt-auto-generator never has to guess (BT-01): one line per subvolume, then /boot.
pub fn fstab(system_uuid: &str, esp_uuid: &str) -> String {
    let mut s = String::from(
        "# /etc/fstab, written by stackific-installer: the btrfs subvolumes and our ESP, by UUID\n",
    );
    for (v, m) in SUBVOLUMES {
        s += &format!("UUID={system_uuid} {m} btrfs subvol=/{v},{MOUNT_OPTIONS} 0 0\n");
    }
    s += &format!("UUID={esp_uuid} /boot vfat umask=0077 0 2\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erase_script() {
        // The 64 GiB test disk: partition 1 at 1 MiB, partition 2 to the last usable sector rounded down to 1 MiB
        let c = erase("/dev/vda", "Stackific OS", &erase_layout(64 << 30, 512));
        assert_eq!(
            c.line(),
            "sfdisk --wipe always --wipe-partitions always /dev/vda"
        );
        assert_eq!(
            c.stdin.unwrap(),
            "label: gpt\nfirst-lba: 2048\n\
             start=2048, size=8388608, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, name=\"Stackific OS boot\"\n\
             start=8390656, size=125825024, type=0FC63DAF-8483-4772-8E79-3D69D8477DE4, name=\"Stackific OS\"\n"
        );
        // 4K sectors: the same bytes
        let l = erase_layout(64 << 30, 4096);
        assert_eq!(
            (l.esp.0 * 4096, l.system.0 * 4096),
            (1 << 20, (1 << 20) + ESP_BYTES)
        );
    }

    #[test]
    fn free_space_script() {
        let gap = Gap {
            start: 1_701_865_472,
            end: 1_953_523_712,
            sector: 512,
        };
        let l = Layout::in_gap(&gap);
        assert_eq!(
            l,
            Layout {
                esp: (1_701_865_472, 8_388_608),
                system: (1_710_254_080, 243_269_632)
            }
        );
        let c = append("/dev/nvme0n1", "Stackific OS", &l);
        assert_eq!(
            c.line(),
            "sfdisk --append --wipe never --wipe-partitions always /dev/nvme0n1"
        );
        assert_eq!(
            c.stdin.unwrap(),
            "start=1701865472, size=8388608, type=C12A7328-F81F-11D2-BA4B-00A0C93EC93B, name=\"Stackific OS boot\"\n\
             start=1710254080, size=243269632, type=0FC63DAF-8483-4772-8E79-3D69D8477DE4, name=\"Stackific OS\"\n"
        );
        // 4K sectors: the same 4 GiB
        assert_eq!(
            Layout::in_gap(&Gap {
                start: 256,
                end: 25_600_000,
                sector: 4096
            })
            .esp,
            (256, 1_048_576)
        );
    }

    #[test]
    fn subvolumes_mounts_and_fstab() {
        let names: Vec<_> = SUBVOLUMES.iter().map(|(v, _)| *v).collect();
        assert_eq!(
            names,
            [
                "@",
                "@home",
                "@root",
                "@srv",
                "@cache",
                "@tmp",
                "@log",
                "@swap",
                "@flatpak",
                "@libvirt",
                "@containers",
                "@docker",
                "@snapshots"
            ]
        );
        let cmds: Vec<String> = prepare("/dev/vda1", "/dev/vda2", Path::new(TARGET))
            .iter()
            .map(Cmd::line)
            .collect();
        assert_eq!(
            cmds[1],
            "mount -o subvolid=5 /dev/vda2 /run/stackific-installer/target"
        );
        assert!(cmds[2].starts_with("btrfs subvolume create /run/stackific-installer/target/@ /run/stackific-installer/target/@home"));
        assert_eq!(
            cmds[3],
            "chattr +C /run/stackific-installer/target/@libvirt /run/stackific-installer/target/@swap"
        );
        assert_eq!(
            cmds[4],
            "chmod 750 /run/stackific-installer/target/@snapshots"
        );
        assert_eq!(
            cmds[6],
            "mount -o subvol=/@,noatime,compress=zstd:1 /dev/vda2 /run/stackific-installer/target"
        );
        assert!(cmds.contains(&"mount -o subvol=/@snapshots,noatime,compress=zstd:1 /dev/vda2 /run/stackific-installer/target/.snapshots".into()));
        assert!(cmds.contains(&"mount -o subvol=/@libvirt,noatime,compress=zstd:1 /dev/vda2 /run/stackific-installer/target/var/lib/libvirt/images".into()));
        assert_eq!(
            cmds.last().unwrap(),
            "mount -o umask=0077 /dev/vda1 /run/stackific-installer/target/boot"
        );
        let f = fstab("S", "E");
        assert_eq!(
            f.lines().filter(|l| l.contains(" btrfs subvol=/@")).count(),
            13
        );
        assert!(
            f.contains("UUID=S /.snapshots btrfs subvol=/@snapshots,noatime,compress=zstd:1 0 0\n")
        );
        assert!(f.contains("UUID=S / btrfs subvol=/@,noatime,compress=zstd:1 0 0\n"));
        assert!(f.contains(
            "UUID=S /var/lib/docker btrfs subvol=/@docker,noatime,compress=zstd:1 0 0\n"
        ));
        assert!(f.ends_with("UUID=E /boot vfat umask=0077 0 2\n"));
        assert_eq!(fat_label("stackific"), "STACKIFIC");
        assert_eq!(fat_label("averyverylongid"), "AVERYVERYLO");
    }

    #[test]
    fn new_partitions_by_start() {
        let json = r#"{"blockdevices":[{"name":"vda","path":"/dev/vda","type":"disk","size":1,"start":null,
            "children":[{"name":"vda4","path":"/dev/vda4","type":"part","size":1,"start":100},
            {"name":"vda6","path":"/dev/vda6","type":"part","size":1,"start":200}]}]}"#;
        assert_eq!(
            find_by_start(json, [100 * 512, 200 * 512]).unwrap(),
            ["/dev/vda4".to_string(), "/dev/vda6".into()]
        );
        assert!(find_by_start(json, [100 * 512, 300 * 512]).is_err());
        // An erase of the 64 GiB test disk: what sfdisk made of the erase script there
        let l = erase_layout(64 << 30, 512);
        assert_eq!(
            l,
            Layout {
                esp: (2048, 8_388_608),
                system: (8_390_656, 125_825_024)
            }
        );
    }
}
