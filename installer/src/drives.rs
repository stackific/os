//! The drives the installer may use: whole disks from `lsblk --json`, their tables from `sfdisk --json` and
//! `--verify`, the protective MBR from `sfdisk --label-nested dos`, and the largest 1 MiB-aligned gap. Each drive left
//! out, and each free space that cannot be used, carries its reason (IN-07's refusals among them).

use std::path::Path;

use serde::Deserialize;

use crate::run::{Cmd, Runner};

/// Why an Intel RST RAID member is left out (IN-07); the Drive screen adds how to switch.
pub const RST_MEMBER: &str = "Intel RST RAID member: set storage mode to AHCI";

/// The smallest gap, or drive, an install accepts (IN-06).
pub const MIN_BYTES: u64 = 40 << 30;
/// New partitions start and end on 1 MiB boundaries (IN-01).
pub const ALIGN_BYTES: u64 = 1 << 20;

/// One device from `lsblk --json --bytes`, with its children.
#[derive(Clone, Debug, Deserialize)]
pub struct Dev {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub size: u64,
    #[serde(default)]
    pub ro: bool,
    pub tran: Option<String>,
    pub model: Option<String>,
    pub fstype: Option<String>,
    pub label: Option<String>,
    /// The partition's type: a GUID on GPT, a hex code such as `0x7` on MBR.
    pub parttype: Option<String>,
    pub partuuid: Option<String>,
    pub start: Option<u64>,
    /// The logical sector size, which sfdisk counts in.
    #[serde(rename = "log-sec", default)]
    pub log_sec: Option<u64>,
    #[serde(default)]
    pub mountpoints: Vec<Option<String>>,
    #[serde(default)]
    pub children: Vec<Dev>,
}

#[derive(Deserialize)]
struct Lsblk {
    blockdevices: Vec<Dev>,
}

/// The lsblk columns the installer reads.
pub const LSBLK_COLUMNS: &str =
    "NAME,PATH,TYPE,SIZE,RO,TRAN,MODEL,FSTYPE,LABEL,PARTTYPE,PARTUUID,START,LOG-SEC,MOUNTPOINTS";

/// Parses `lsblk --json --bytes --output …`.
pub fn parse_lsblk(json: &str) -> Result<Vec<Dev>, String> {
    serde_json::from_str::<Lsblk>(json)
        .map(|l| l.blockdevices)
        .map_err(|e| format!("lsblk: {e}"))
}

/// A partition table from `sfdisk --json`.
#[derive(Clone, Debug, Deserialize)]
pub struct PartTable {
    pub label: String,
    #[serde(default)]
    pub firstlba: u64,
    #[serde(default)]
    pub lastlba: u64,
    #[serde(default = "default_sector")]
    pub sectorsize: u64,
    #[serde(default)]
    pub partitions: Vec<SfPart>,
}

fn default_sector() -> u64 {
    512
}

/// One partition-table entry from `sfdisk --json`.
#[derive(Clone, Debug, Deserialize)]
pub struct SfPart {
    pub start: u64,
    pub size: u64,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize)]
struct Sfdisk {
    partitiontable: PartTable,
}

/// Parses `sfdisk --json`.
pub fn parse_sfdisk(json: &str) -> Result<PartTable, String> {
    serde_json::from_str::<Sfdisk>(json)
        .map(|s| s.partitiontable)
        .map_err(|e| format!("sfdisk: {e}"))
}

/// A run of free sectors, `start` inclusive and `end` exclusive, both on the alignment grain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gap {
    pub start: u64,
    pub end: u64,
    pub sector: u64,
}

impl Gap {
    /// The gap's size in bytes.
    pub fn bytes(&self) -> u64 {
        (self.end - self.start) * self.sector
    }
}

/// The largest aligned gap in a GPT between `firstlba` and `lastlba`: each gap's start rounded up and end rounded
/// down to 1 MiB, small leftovers ignored (IN-01).
pub fn largest_gap(table: &PartTable) -> Option<Gap> {
    let sector = table.sectorsize;
    let grain = (ALIGN_BYTES / sector).max(1);
    let mut used: Vec<(u64, u64)> = table
        .partitions
        .iter()
        .map(|p| (p.start, p.start + p.size))
        .collect();
    used.sort_unstable();
    let mut gaps = Vec::new();
    let mut free_from = table.firstlba;
    for (start, end) in used {
        gaps.push((free_from, start));
        free_from = free_from.max(end);
    }
    gaps.push((free_from, table.lastlba + 1));
    gaps.into_iter()
        .map(|(s, e)| Gap {
            start: s.div_ceil(grain) * grain,
            end: e / grain * grain,
            sector,
        })
        .filter(|g| g.end > g.start)
        .max_by_key(|g| (g.end - g.start, std::cmp::Reverse(g.start)))
}

/// What the drive's free space allows: an install into `Gap`, or the reason it is refused.
pub type FreeSpace = Result<Gap, String>;

/// A drive the installer lists, with what the Drive, Space and Confirm screens show about it.
#[derive(Clone, Debug)]
pub struct Drive {
    pub path: String,
    pub name: String,
    pub size: u64,
    /// The logical sector size: the table's, or lsblk's for a drive without one.
    pub sector: u64,
    pub model: String,
    /// The partitions lsblk sees, by start sector.
    pub parts: Vec<Dev>,
    /// `None` when the drive has no partition table.
    pub table: Option<PartTable>,
    /// The largest aligned gap, usable or not.
    pub gap: Option<Gap>,
    /// Whether an install into the free space is allowed, and why not.
    pub free: FreeSpace,
}

impl Drive {
    /// The partition just before `gap`, if any (lsblk counts in 512-byte sectors, sfdisk in the drive's own).
    pub fn part_before(&self, gap: &Gap) -> Option<&Dev> {
        let at = gap.start * gap.sector;
        self.parts
            .iter()
            .rev()
            .find(|p| p.start.is_some_and(|s| s * 512 < at))
    }
}

/// A drive the installer leaves out, and why.
#[derive(Clone, Debug, PartialEq)]
pub struct Excluded {
    pub path: String,
    pub reason: String,
}

/// Sizes the way every screen shows them: one decimal and a binary unit, such as "931.5 GiB".
pub fn human(bytes: u64) -> String {
    let units = [("TiB", 40), ("GiB", 30), ("MiB", 20), ("KiB", 10)];
    let (unit, shift) = units
        .iter()
        .find(|(_, s)| bytes >= 1 << s)
        .copied()
        .unwrap_or(("B", 0));
    format!("{:.1} {unit}", bytes as f64 / (1u64 << shift) as f64)
}

/// Every mount point of a device and its children ("[SWAP]" for swap).
fn mounts(dev: &Dev) -> Vec<String> {
    let own = dev.mountpoints.iter().flatten().cloned();
    own.chain(dev.children.iter().flat_map(mounts)).collect()
}

/// Whether `path` is the device or one of its children.
pub fn holds(dev: &Dev, path: &str) -> bool {
    dev.path == path || dev.children.iter().any(|c| holds(c, path))
}

/// Why a whole disk cannot be used at all, if it cannot. `live` is the device the live medium is mounted from.
pub fn exclusion(dev: &Dev, live: Option<&str>) -> Option<String> {
    if live.is_some_and(|l| holds(dev, l)) {
        let what = if dev.tran.as_deref() == Some("usb") {
            "the USB stick"
        } else {
            "the drive"
        };
        return Some(format!("{what} this installer runs from"));
    }
    // Before the in-use checks: md may hold the member (IN-07)
    if dev.fstype.as_deref() == Some("isw_raid_member") {
        return Some(RST_MEMBER.into());
    }
    if dev.ro {
        return Some("read-only".into());
    }
    if dev.size < MIN_BYTES {
        return Some(format!("{}, under the 40 GiB minimum", human(dev.size)));
    }
    let m = mounts(dev);
    if m.iter().any(|p| p == "[SWAP]") {
        return Some("in use as swap".into());
    }
    if let Some(p) = m.first() {
        return Some(format!("in use, mounted at {p}"));
    }
    None
}

/// Why the free space on an MBR drive cannot be used when Windows is on it: such a Windows starts in legacy BIOS mode,
/// which our UEFI-only Limine cannot start (IN-07).
pub const BIOS_WINDOWS: &str = "BIOS-mode Windows: convert with MBR2GPT.EXE";

/// Whether an MBR table holds Windows: a partition of type 07 that lsblk sees as NTFS or BitLocker.
pub fn bios_windows(table: &PartTable, parts: &[Dev]) -> bool {
    table.label == "dos"
        && table.partitions.iter().any(|t| {
            t.kind.trim_start_matches("0x") == "7"
                && parts.iter().any(|p| {
                    p.start
                        .is_some_and(|s| s * 512 == t.start * table.sectorsize)
                        && matches!(p.fstype.as_deref(), Some("ntfs" | "BitLocker"))
                })
        })
}

/// Whether the free space may take an install: a clean GPT (`sfdisk --verify` reports no error and warns about
/// nothing), a protective MBR with only its `ee` entry, and a gap of at least 40 GiB. BitLocker is confirmed on its
/// own screen instead (`bitlocker.rs`). `bios_windows` names the reason on an MBR drive with Windows.
pub fn free_space(
    table: &PartTable,
    verify: (&str, &str),
    nested: Option<&PartTable>,
    bios_windows: bool,
) -> FreeSpace {
    if table.label != "gpt" {
        return Err(if bios_windows {
            BIOS_WINDOWS.into()
        } else {
            "no GPT partition table".into()
        });
    }
    let (out, err) = verify;
    let protective = nested.is_some_and(|n| {
        n.partitions.len() == 1 && n.partitions[0].kind.eq_ignore_ascii_case("ee")
    });
    if !out.contains("No errors detected.") || !err.trim().is_empty() || !protective {
        return Err("the partition table needs repair".into());
    }
    match largest_gap(table) {
        Some(g) if g.bytes() >= MIN_BYTES => Ok(g),
        Some(g) => Err(format!(
            "only {} free, under the 40 GiB minimum",
            human(g.bytes())
        )),
        None => Err("no free space".into()),
    }
}

/// Reads every whole disk, splits them into listed and excluded, and probes each listed one's table.
/// `live_medium` is the base's mount point of the boot medium (Arch: /run/archiso/bootmnt).
pub fn scan(run: &dyn Runner, live_medium: &Path) -> Result<(Vec<Drive>, Vec<Excluded>), String> {
    let out = run
        .run(&Cmd::new("lsblk").args(["--json", "--bytes", "--output", LSBLK_COLUMNS]))
        .map_err(|e| e.to_string())?;
    if out.code != 0 {
        return Err(format!("lsblk failed: {}", out.stderr.trim()));
    }
    let live = run
        .run(
            &Cmd::new("findmnt")
                .args(["--noheadings", "--output", "SOURCE"])
                .args([live_medium.to_string_lossy()]),
        )
        .ok()
        .filter(|o| o.code == 0)
        .map(|o| o.stdout.trim().to_string());
    let (mut listed, mut excluded) = (Vec::new(), Vec::new());
    for dev in parse_lsblk(&out.stdout)? {
        // zram is a disk to lsblk, but memory; rom and loop devices are not drives
        if dev.kind != "disk" || dev.name.starts_with("zram") {
            continue;
        }
        match exclusion(&dev, live.as_deref()) {
            Some(reason) => excluded.push(Excluded {
                path: dev.path.clone(),
                reason,
            }),
            None => listed.push(probe(run, dev)?),
        }
    }
    Ok((listed, excluded))
}

/// Reads one disk's table and decides what its free space allows.
pub fn probe(run: &dyn Runner, dev: Dev) -> Result<Drive, String> {
    let sfdisk = |args: &[&str]| {
        run.run(&Cmd::new("sfdisk").args(args).args([&dev.path]))
            .map_err(|e| e.to_string())
    };
    let json = sfdisk(&["--json"])?;
    let table = if json.code == 0 {
        Some(parse_sfdisk(&json.stdout)?)
    } else {
        None
    };
    let mut parts = dev.children.clone();
    parts.sort_by_key(|p| p.start.unwrap_or(0));
    let gap = table
        .as_ref()
        .filter(|t| t.label == "gpt")
        .and_then(largest_gap);
    let free = match &table {
        None => Err("no partition table".into()),
        Some(t) if t.label != "gpt" => free_space(t, ("", ""), None, bios_windows(t, &parts)),
        Some(t) => {
            let verify = sfdisk(&["--verify"])?;
            let nested = sfdisk(&["--json", "--label-nested", "dos"])?;
            let nested = if nested.code == 0 {
                parse_sfdisk(&nested.stdout).ok()
            } else {
                None
            };
            free_space(t, (&verify.stdout, &verify.stderr), nested.as_ref(), false)
        }
    };
    Ok(Drive {
        path: dev.path.clone(),
        name: dev.name.clone(),
        size: dev.size,
        sector: table
            .as_ref()
            .map_or(dev.log_sec.unwrap_or(512), |t| t.sectorsize),
        model: dev.model.clone().unwrap_or_default().trim().to_string(),
        parts,
        table,
        gap,
        free,
    })
}

/// Whether Intel RST hides NVMe drives from Linux: the ahci driver counts them in `remapped_nvme` (subiquity's
/// `has_rst` check). Such drives cannot be listed, so the Drive screen says how to show them (IN-07).
pub fn remapped_nvme(run: &dyn Runner) -> bool {
    run.run(&Cmd::new("sh").args([
        "-c",
        "cat /sys/module/ahci/drivers/pci:ahci/*/remapped_nvme 2>/dev/null",
    ]))
    .is_ok_and(|o| {
        o.stdout
            .split_whitespace()
            .any(|n| n.parse::<u32>().is_ok_and(|n| n > 0))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIB: u64 = 1 << 30;

    fn table(first: u64, last: u64, parts: &[(u64, u64)]) -> PartTable {
        PartTable {
            label: "gpt".into(),
            firstlba: first,
            lastlba: last,
            sectorsize: 512,
            partitions: parts
                .iter()
                .map(|&(start, size)| SfPart {
                    start,
                    size,
                    kind: "x".into(),
                })
                .collect(),
        }
    }

    const PROTECTIVE: &str = r#"{"partitiontable":{"label":"dos","partitions":[{"start":1,"size":134217727,"type":"ee"}]}}"#;
    const HYBRID: &str = r#"{"partitiontable":{"label":"dos","partitions":[{"start":1,"size":2047,"type":"ee"},
        {"start":2048,"size":204800,"type":"c"}]}}"#;

    #[test]
    fn gap_search() {
        // The fixture laptop drive: 120 GiB after the Windows partition
        let t = parse_sfdisk(include_str!("../fixtures/sfdisk-nvme0n1.json")).unwrap();
        let g = largest_gap(&t).unwrap();
        assert_eq!((g.start % 2048, g.end % 2048, g.bytes()), (0, 0, 120 * GIB));
        // Alignment: a gap from an odd sector is rounded up, its end down
        let g = largest_gap(&table(34, 1_000_000, &[(34, 100)])).unwrap();
        assert_eq!((g.start, g.end), (2048, 999_424));
        // The largest of several gaps wins, wherever it is; a numbering hole changes nothing
        let t = table(
            2048,
            134_217_694,
            &[
                (2048, 204_800),
                (206_848, 32_768),
                (4_433_920, 20_971_520),
                (130_000_000, 4_215_680),
            ],
        );
        let g = largest_gap(&t).unwrap();
        assert_eq!((g.start, g.end), (25_405_440, 129_998_848));
        // A gap under 40 GiB is reported, not used
        let t = table(2048, 20 * GIB / 512, &[(2048, 2048)]);
        assert!(
            free_space(
                &t,
                ("No errors detected.", ""),
                parse_sfdisk(PROTECTIVE).ok().as_ref(),
                false
            )
            .unwrap_err()
            .contains("under the 40 GiB minimum")
        );
        // A full disk has no gap (leftovers under 1 MiB are ignored)
        assert_eq!(largest_gap(&table(2048, 999_999, &[(2048, 997_000)])), None);
    }

    #[test]
    fn free_space_refusals() {
        let gpt = table(2048, 134_217_694, &[(2048, 204_800)]);
        let ok = ("No errors detected.\n", "");
        let protective = parse_sfdisk(PROTECTIVE).unwrap();
        assert!(free_space(&gpt, ok, Some(&protective), false).is_ok());
        let mbr = PartTable {
            label: "dos".into(),
            ..gpt.clone()
        };
        assert_eq!(
            free_space(&mbr, ok, None, false),
            Err("no GPT partition table".into())
        );
        let bad = (
            "The backup GPT table is not on the end of the device.\n1 error detected.\n",
            "",
        );
        assert_eq!(
            free_space(&gpt, bad, Some(&protective), false),
            Err("the partition table needs repair".into())
        );
        let warned = (
            "No errors detected.",
            "GPT PMBR size mismatch (2097151 != 4194303) will be corrected by write.",
        );
        assert!(free_space(&gpt, warned, Some(&protective), false).is_err());
        let hybrid = parse_sfdisk(HYBRID).unwrap();
        assert_eq!(
            free_space(&gpt, ok, Some(&hybrid), false),
            Err("the partition table needs repair".into())
        );
        // An MBR table with Windows names BIOS mode and MBR2GPT
        assert_eq!(free_space(&mbr, ok, None, true), Err(BIOS_WINDOWS.into()));
    }

    #[test]
    fn eligibility() {
        let devs = parse_lsblk(include_str!("../fixtures/lsblk-edge.json")).unwrap();
        let reasons: Vec<(String, Option<String>)> = devs
            .iter()
            .filter(|d| d.kind == "disk")
            .map(|d| (d.name.clone(), exclusion(d, Some("/dev/sdb1"))))
            .collect();
        let want = |n: &str| reasons.iter().find(|(d, _)| d == n).unwrap().1.clone();
        assert_eq!(
            want("sdb"),
            Some("the USB stick this installer runs from".into())
        );
        assert_eq!(want("sdd"), Some("read-only".into()));
        assert_eq!(
            want("sdc"),
            Some("29.8 GiB, under the 40 GiB minimum".into())
        );
        assert_eq!(want("sde"), Some("in use, mounted at /mnt".into()));
        assert_eq!(want("sdf"), Some("in use as swap".into()));
        assert_eq!(want("vda"), None);
        // An Intel RST RAID member is left out first, even while md holds it
        let mut member = devs.iter().find(|d| d.name == "sde").unwrap().clone();
        member.fstype = Some("isw_raid_member".into());
        assert_eq!(exclusion(&member, None), Some(RST_MEMBER.into()));
        // zram is a disk to lsblk; rom and loop devices are not disks: scan skips all three
        assert!(devs.iter().any(|d| d.name == "zram0" && d.kind == "disk"));
        assert!(devs.iter().any(|d| d.kind == "rom") && devs.iter().any(|d| d.kind == "loop"));
    }

    #[test]
    fn scan_lists_whole_disks_and_says_why_others_are_left_out() {
        use crate::run::{Fake, Output};
        let f = Fake::default();
        f.on(
            "lsblk --json --bytes --output",
            Output::ok(include_str!("../fixtures/lsblk-edge.json")),
        )
        .on(
            "findmnt --noheadings --output SOURCE /run/archiso/bootmnt",
            Output::ok("/dev/sdb1\n"),
        )
        .on(
            "sfdisk --json /dev/vda",
            Output::fail(1, "no partition table"),
        );
        let (listed, excluded) = scan(&f, Path::new("/run/archiso/bootmnt")).unwrap();
        assert_eq!(
            listed.iter().map(|d| d.path.as_str()).collect::<Vec<_>>(),
            ["/dev/vda"]
        );
        let left: Vec<&str> = excluded.iter().map(|x| x.path.as_str()).collect();
        // zram, the optical drive and the loop device are not drives, so they are not even listed as left out
        assert_eq!(
            left,
            ["/dev/sdb", "/dev/sdc", "/dev/sdd", "/dev/sde", "/dev/sdf"]
        );
        assert!(f.state().unexpected.is_empty());
    }

    #[test]
    fn probe_reads_the_table_and_verify() {
        use crate::run::{Fake, Output};
        let devs = parse_lsblk(include_str!("../fixtures/lsblk.json")).unwrap();
        let nvme = devs.iter().find(|d| d.name == "nvme0n1").unwrap().clone();
        let protective =
            r#"{"partitiontable":{"label":"dos","partitions":[{"start":1,"size":9,"type":"ee"}]}}"#;
        let f = Fake::default();
        f.on(
            "sfdisk --json /dev/nvme0n1",
            Output::ok(include_str!("../fixtures/sfdisk-nvme0n1.json")),
        )
        .on(
            "sfdisk --verify /dev/nvme0n1",
            Output::ok("No errors detected.\n"),
        )
        .on(
            "sfdisk --json --label-nested dos /dev/nvme0n1",
            Output::ok(protective),
        );
        // BitLocker on nvme0n1p3 no longer refuses the free space: its own screen confirms it
        let d = probe(&f, nvme.clone()).unwrap();
        assert_eq!(d.parts[2].fstype.as_deref(), Some("BitLocker"));
        assert_eq!((d.gap.unwrap().bytes(), d.sector), (120 * GIB, 512));
        assert!(d.free.is_ok());
        // No table: sfdisk fails, only an erase is possible, at lsblk's sector size
        let sda = devs.iter().find(|d| d.name == "sda").unwrap().clone();
        f.on(
            "sfdisk --json /dev/sda",
            Output::fail(
                1,
                "sfdisk: /dev/sda: does not contain a recognized partition table",
            ),
        );
        let d = probe(
            &f,
            Dev {
                log_sec: Some(4096),
                ..sda
            },
        )
        .unwrap();
        assert_eq!((d.free, d.sector), (Err("no partition table".into()), 4096));
        // An MBR table: with Windows (type 7, NTFS or BitLocker at the same start) the reason names BIOS mode
        let mbr = r#"{"partitiontable":{"label":"dos","sectorsize":512,"partitions":[{"start":567296,"size":9,"type":"7"}]}}"#;
        f.state()
            .rules
            .insert(0, ("sfdisk --json /dev/nvme0n1".into(), Output::ok(mbr)));
        assert_eq!(
            probe(&f, nvme.clone()).unwrap().free,
            Err(BIOS_WINDOWS.into())
        );
        let mut data = nvme.clone();
        data.children[2].fstype = Some("exfat".into());
        assert_eq!(
            probe(&f, data).unwrap().free,
            Err("no GPT partition table".into())
        );

        assert!(f.state().unexpected.is_empty());
    }

    #[test]
    fn remapped_nvme_counts_hidden_drives() {
        use crate::run::{Fake, Output};
        let f = Fake::default();
        f.on("sh -c", Output::ok("0\n0\n"));
        assert!(!remapped_nvme(&f));
        f.state()
            .rules
            .insert(0, ("sh -c".into(), Output::ok("0\n2\n")));
        assert!(remapped_nvme(&f));
        // No ahci controller: cat finds nothing
        f.state()
            .rules
            .insert(0, ("sh -c".into(), Output::fail(1, "")));
        assert!(!remapped_nvme(&f));
    }

    #[test]
    fn sizes() {
        assert_eq!(human(1_000_204_886_016), "931.5 GiB");
        assert_eq!(human(260 << 20), "260.0 MiB");
        assert_eq!(human(32_017_047_552), "29.8 GiB");
    }
}
