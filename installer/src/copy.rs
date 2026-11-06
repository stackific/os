//! The copy (IN-10, offline): rsync from the live image's pristine root to the target, its overall percentage from
//! `--info=progress2`, and its stop.

use std::path::Path;

use crate::run::Cmd;

/// `rsync -aHAXx --numeric-ids --info=progress2` from the live root to the target. `--no-inc-recursive` makes rsync
/// count every file first, so progress2's percentage is of the whole copy and only grows. `/boot` is left out: it is
/// empty in the live root (mkarchiso empties it) and is the FAT ESP here, which cannot take owners, ACLs or xattrs.
pub fn rsync(live_root: &Path, target: &Path) -> Cmd {
    Cmd::new("rsync")
        .args([
            "-aHAXx",
            "--numeric-ids",
            "--info=progress2",
            "--no-inc-recursive",
            "--exclude=/boot/",
        ])
        .args([
            format!("{}/", live_root.display()),
            format!("{}/", target.display()),
        ])
}

/// The overall percentage in one progress2 line, such as `  1,234,567  52%   10.00MB/s    0:00:10 (xfr#12, to-chk=3/9)`.
pub fn percent(line: &str) -> Option<u8> {
    line.split_whitespace()
        .find_map(|w| w.strip_suffix('%')?.parse::<u8>().ok())
        .map(|p| p.min(100))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_and_progress() {
        let c = rsync(
            Path::new("/run/archiso/airootfs"),
            Path::new("/run/stackific-installer/target"),
        );
        assert_eq!(
            c.line(),
            "rsync -aHAXx --numeric-ids --info=progress2 --no-inc-recursive --exclude=/boot/ /run/archiso/airootfs/ /run/stackific-installer/target/"
        );
        assert_eq!(
            percent("  1,234,567  52%   10.00MB/s    0:00:10 (xfr#12, to-chk=3/9)"),
            Some(52)
        );
        assert_eq!(
            percent("3,620,000,000 100%   80.00MB/s    0:00:43 (xfr#210000, to-chk=0/215000)"),
            Some(100)
        );
        assert_eq!(percent("sending incremental file list"), None);
    }
}
