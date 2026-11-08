//! BitLocker on the drives the installer lists (IN-02): each volume `blkid -p` finds, classified by the protectors
//! `cryptsetup bitlkDump` reads from its metadata, offline and read-only. Secure Boot is off whenever our ISO runs, so
//! Windows will ask for the recovery key of a TPM-protected volume at its next start; the BitLocker screen says which
//! key, and the install needs the typed confirmation `PHRASE`.

use crate::drives::{Drive, human};
use crate::run::{Cmd, Runner};

/// The typed confirmation, on the screen and in the unattended file's `[disk] bitlocker`.
pub const PHRASE: &str = "I have my key";

/// What Windows does with a volume at its next start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// A clear-key protector: BitLocker is suspended (or not yet protecting), so Windows starts without asking.
    Suspended,
    /// A TPM protector: with Secure Boot now off, Windows will ask for the recovery key.
    Armed,
    /// Only a password, smart card or startup key: Windows asks for those as before.
    Locked,
}

/// One BitLocker volume, for the screen and the unattended refusal.
#[derive(Clone, Debug, PartialEq)]
pub struct Volume {
    /// The partition's kernel name ("nvme0n1p3"), its size, and its drive's model.
    pub name: String,
    pub size: u64,
    pub model: String,
    pub state: State,
    /// The recovery keys' IDs as Windows shows them: each recovery-passphrase protector's GUID, first 8 characters,
    /// uppercase. Windows names its key files by the same GUIDs.
    pub recovery: Vec<String>,
}

impl Volume {
    /// One line on what Windows will do, for the screen and stderr.
    pub fn summary(&self) -> String {
        match (self.state, self.recovery.is_empty()) {
            (State::Suspended, _) => "Suspended: Windows starts without asking for the key".into(),
            (State::Armed, true) => "On, and no recovery key exists".into(),
            (State::Armed, false) => format!(
                "On: Windows will ask for recovery key {}",
                self.recovery.join(" or ")
            ),
            (State::Locked, true) => "On, unlocked by its password or key as before".into(),
            (State::Locked, false) => format!(
                "On, unlocked by its password or key; recovery key {}",
                self.recovery.join(" or ")
            ),
        }
    }
}

/// The state and the recovery key IDs from `cryptsetup bitlkDump`'s keyslots (cryptsetup 2.8.8, `lib/bitlk/bitlk.c`).
pub fn classify(dump: &str) -> (State, Vec<String>) {
    let (mut guid, mut protections, mut recovery) = ("", Vec::new(), Vec::new());
    for line in dump.lines().map(str::trim) {
        if let Some(g) = line.strip_prefix("GUID:") {
            guid = g.trim();
        } else if let Some(p) = line.strip_prefix("Protection:") {
            let p = p.trim();
            if p == "VMK protected with recovery passphrase" {
                recovery.push(guid.get(..8).unwrap_or(guid).to_uppercase());
            }
            protections.push(p.to_string());
        }
    }
    let has = |text: &str| protections.iter().any(|p| p.contains(text));
    let state = if has("clear key") {
        State::Suspended
    } else if has("TPM") {
        State::Armed
    } else {
        State::Locked
    };
    (state, recovery)
}

/// Every BitLocker volume on `drives`, except on the drive `erased` (an erase removes it). A volume whose metadata
/// cannot be read counts as armed with no known key, the safe reading.
pub fn scan(run: &dyn Runner, drives: &[Drive], erased: Option<&str>) -> Vec<Volume> {
    let mut found = Vec::new();
    for d in drives.iter().filter(|d| Some(d.path.as_str()) != erased) {
        for p in &d.parts {
            let probe = run.run(&Cmd::new("blkid").args(["-p", "-o", "export", &p.path]));
            if !probe.is_ok_and(|o| o.stdout.lines().any(|l| l == "TYPE=BitLocker")) {
                continue;
            }
            let dump = run
                .run(&Cmd::new("cryptsetup").args(["bitlkDump", &p.path]))
                .ok()
                .filter(|o| o.code == 0);
            let (state, recovery) = dump.map_or((State::Armed, vec![]), |o| classify(&o.stdout));
            found.push(Volume {
                name: p.name.clone(),
                size: p.size,
                model: d.model.clone(),
                state,
                recovery,
            });
        }
    }
    found
}

/// Why the unattended run stops without `[disk] bitlocker = PHRASE`: one line per volume.
pub fn refusal(volumes: &[Volume]) -> String {
    let lines: Vec<String> = volumes
        .iter()
        .map(|v| format!("  {} ({}): {}", v.name, human(v.size), v.summary()))
        .collect();
    format!(
        "BitLocker is on this computer, and Secure Boot is off:\n{}\nSave each recovery key (https://aka.ms/myrecoverykey), then set [disk] bitlocker = \"{PHRASE}\"",
        lines.join("\n")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drives::{Dev, parse_lsblk};
    use crate::run::{Fake, Output};

    #[test]
    fn protectors_give_the_state_and_key_ids() {
        // Volumes Windows made (cryptsetup's test images, bitlkDump 2.8.8)
        let (s, ids) = classify(include_str!("../fixtures/bitlkdump-recovery.txt"));
        assert_eq!((s, ids), (State::Locked, vec!["64311DEA".to_string()]));
        let (_, ids) = classify(include_str!("../fixtures/bitlkdump-two-recovery.txt"));
        assert_eq!(ids, ["E7E48BAE", "B7ADC334"]);
        assert_eq!(
            classify(include_str!("../fixtures/bitlkdump-clearkey.txt")),
            (State::Suspended, vec![])
        );
        // Edited from the first: its passphrase protector made a TPM one, its recovery passphrase removed
        assert_eq!(
            classify(include_str!("../fixtures/bitlkdump-tpm-no-recovery.txt")),
            (State::Armed, vec![])
        );
    }

    #[test]
    fn scan_skips_the_erased_drive_and_reads_each_volume() {
        let devs = parse_lsblk(include_str!("../fixtures/lsblk.json")).unwrap();
        let nvme: Dev = devs.iter().find(|d| d.name == "nvme0n1").unwrap().clone();
        let drive = Drive {
            path: nvme.path.clone(),
            name: nvme.name.clone(),
            size: nvme.size,
            sector: 512,
            model: "Samsung SSD 990 PRO 1TB".into(),
            parts: nvme.children.clone(),
            table: None,
            gap: None,
            free: Err(String::new()),
        };
        let f = Fake::default();
        f.on(
            "blkid -p -o export /dev/nvme0n1p3",
            Output::ok(include_str!("../fixtures/blkid-bitlocker.txt")),
        )
        .on("blkid -p", Output::fail(2, ""))
        .on(
            "cryptsetup bitlkDump /dev/nvme0n1p3",
            Output::ok(include_str!("../fixtures/bitlkdump-recovery.txt")),
        );
        let v = scan(&f, std::slice::from_ref(&drive), None);
        assert_eq!(v.len(), 1);
        assert_eq!(
            (v[0].name.as_str(), v[0].recovery.clone()),
            ("nvme0n1p3", vec!["64311DEA".into()])
        );
        assert!(scan(&f, std::slice::from_ref(&drive), Some("/dev/nvme0n1")).is_empty());
        // Unreadable metadata: armed, no known key
        f.state().rules.insert(
            0,
            (
                "cryptsetup bitlkDump".into(),
                Output::fail(1, "unsupported"),
            ),
        );
        let v = scan(&f, &[drive], None);
        assert_eq!((v[0].state, v[0].recovery.len()), (State::Armed, 0));
        assert_eq!(v[0].summary(), "On, and no recovery key exists");
        assert!(refusal(&v).contains("nvme0n1p3 (811.2 GiB): On, and no recovery key exists"));
        assert!(f.state().unexpected.is_empty());
    }
}
