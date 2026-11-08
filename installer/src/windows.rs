//! Windows next to us (IN-04, narrowed to Windows): its loader on an EFI System Partition, found read-only, and
//! whether the firmware has its "Windows Boot Manager" entry, for Limine's Windows entries (`boot::windows_entries`)
//! and the Confirm screen.

use crate::drives::Drive;
use crate::run::{Cmd, Runner};

/// The EFI System Partition's GPT type, as lsblk prints it.
const ESP_TYPE: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
/// Windows' loader on its ESP (FAT, so the case does not matter).
pub const LOADER: &str = "EFI/Microsoft/Boot/bootmgfw.efi";
/// The firmware entry Windows creates, which Limine's `efi_boot_entry` restarts into.
pub const ENTRY: &str = "Windows Boot Manager";
/// Where a Windows ESP is mounted read-only to look for the loader.
const MOUNT: &str = "/run/stackific-installer/esp";

/// The Windows installs found.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Windows {
    /// Each Windows ESP's partition GUID, lowercase.
    pub esps: Vec<String>,
    /// Whether `BootOrder` holds "Windows Boot Manager" on one of those ESPs; Limine's `efi_boot_entry` needs it.
    pub firmware_entry: bool,
}

/// Looks at every ESP on `drives`, except on the drive `erased`, for Windows' loader: one already mounted where it is,
/// any other mounted read-only for the look and unmounted again. Then reads the firmware's entries.
pub fn find(run: &dyn Runner, drives: &[Drive], erased: Option<&str>) -> Windows {
    let ok = |cmd: Cmd| run.run(&cmd).is_ok_and(|o| o.code == 0);
    let mut esps = Vec::new();
    for d in drives.iter().filter(|d| Some(d.path.as_str()) != erased) {
        for p in &d.parts {
            let (Some(kind), Some(uuid)) = (&p.parttype, &p.partuuid) else {
                continue;
            };
            if !kind.eq_ignore_ascii_case(ESP_TYPE) {
                continue;
            }
            let has_loader =
                |dir: &str| ok(Cmd::new("test").args(["-f", &format!("{dir}/{LOADER}")]));
            let found = match p.mountpoints.iter().flatten().next() {
                Some(dir) => has_loader(dir),
                None => {
                    let mounted = ok(Cmd::new("mkdir").args(["-p", MOUNT]))
                        && ok(Cmd::new("mount").args([
                            "-o",
                            "ro,nosuid,nodev,noexec",
                            &p.path,
                            MOUNT,
                        ]));
                    let found = mounted && has_loader(MOUNT);
                    if mounted {
                        ok(Cmd::new("umount").args([MOUNT]));
                    }
                    found
                }
            };
            if found {
                esps.push(uuid.to_lowercase());
            }
        }
    }
    let listing = run
        .run(&Cmd::new("efibootmgr"))
        .map(|o| o.stdout)
        .unwrap_or_default();
    let firmware_entry = has_entry(&listing, &esps);
    Windows {
        esps,
        firmware_entry,
    }
}

/// Whether efibootmgr's listing has, in `BootOrder`, an entry described exactly "Windows Boot Manager" (ignoring
/// case, as Limine matches it) whose `HD(…)` device path names one of `esps`.
pub fn has_entry(listing: &str, esps: &[String]) -> bool {
    let order: Vec<&str> = listing
        .lines()
        .find_map(|l| l.strip_prefix("BootOrder:"))
        .map(|o| o.trim().split(',').collect())
        .unwrap_or_default();
    listing.lines().any(|l| {
        let Some(rest) = l.strip_prefix("Boot") else {
            return false;
        };
        let (num, rest) = rest.split_at(rest.len().min(4));
        let Some((desc, path)) = rest.trim_start_matches(['*', ' ']).split_once('\t') else {
            return false;
        };
        let guid = path
            .find("HD(")
            .and_then(|i| path[i + 3..].split(',').nth(2))
            .map(str::to_lowercase);
        order.contains(&num)
            && desc.trim().eq_ignore_ascii_case(ENTRY)
            && guid.is_some_and(|g| esps.contains(&g))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drives::parse_lsblk;
    use crate::run::{Fake, Output};

    const WIN: &str = "6a1f0c3e-2b4d-4e5f-9a8b-7c6d5e4f3a2b";

    #[test]
    fn the_firmware_entry_must_be_in_the_boot_order_on_a_windows_esp() {
        let listing = include_str!("../fixtures/efibootmgr-windows.txt");
        let esps = [WIN.to_string()];
        assert!(has_entry(listing, &esps));
        assert!(!has_entry(listing, &["0000".into()]));
        assert!(!has_entry(
            &listing.replace("BootOrder: 0009,000A,", "BootOrder: 0009,"),
            &esps
        ));
        assert!(!has_entry(
            include_str!("../fixtures/efibootmgr.txt"),
            &esps
        ));
    }

    #[test]
    fn find_looks_on_each_esp_read_only() {
        let devs = parse_lsblk(include_str!("../fixtures/lsblk.json")).unwrap();
        let nvme = devs.iter().find(|d| d.name == "nvme0n1").unwrap().clone();
        let drive = Drive {
            path: nvme.path.clone(),
            name: nvme.name.clone(),
            size: nvme.size,
            sector: 512,
            model: String::new(),
            parts: nvme.children.clone(),
            table: None,
            gap: None,
            free: Err(String::new()),
        };
        let f = Fake::default();
        f.on("mkdir -p", Output::ok(""))
            .on(
                "mount -o ro,nosuid,nodev,noexec /dev/nvme0n1p1 /run/stackific-installer/esp",
                Output::ok(""),
            )
            .on(
                "test -f /run/stackific-installer/esp/EFI/Microsoft/Boot/bootmgfw.efi",
                Output::ok(""),
            )
            .on("umount /run/stackific-installer/esp", Output::ok(""))
            .on(
                "efibootmgr",
                Output::ok(include_str!("../fixtures/efibootmgr-windows.txt")),
            );
        let w = find(&f, std::slice::from_ref(&drive), None);
        assert_eq!(
            w,
            Windows {
                esps: vec![WIN.into()],
                firmware_entry: true
            }
        );
        assert!(
            f.state().unexpected.is_empty(),
            "{:?}",
            f.state().unexpected
        );
        // The erased drive is not looked at; an ESP without the loader is no Windows
        assert_eq!(
            find(&f, std::slice::from_ref(&drive), Some("/dev/nvme0n1"))
                .esps
                .len(),
            0
        );
        f.state()
            .rules
            .insert(0, ("test -f".into(), Output::fail(1, "")));
        assert_eq!(find(&f, &[drive], None), Windows::default());
    }
}
