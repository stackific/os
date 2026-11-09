//! snapper on the installed system (task 026): the root config the installer writes itself, because snapper's
//! create-config refuses the `/.snapshots` that `@snapshots` is already mounted on, the list of configs, and the
//! "Fresh install" snapshot that ends every install.

use std::path::Path;

use crate::run::Cmd;

/// The root config, under the target.
pub const ROOT: &str = "etc/snapper/configs/root";
/// Its text: snapper's template with ranged number limits and `FREE_LIMIT`, so snapper deletes down towards the
/// minimum once free space drops under 20 %, with no quotas (`SPACE_LIMIT` needs them) and no timeline.
pub const ROOT_CONFIG: &str = r#"# Written by stackific-installer: snapper's config for / on @, its snapshots in @snapshots at /.snapshots.
# Snapshots come from package updates (pre and post) and by hand; cleanup keeps 2 to 10 of them, 4 to 10 important
# ones, and deletes down to the minimum while less than 20 % of the file system is free.
SUBVOLUME="/"
FSTYPE="btrfs"
QGROUP=""
FREE_LIMIT="0.2"
ALLOW_USERS=""
ALLOW_GROUPS=""
SYNC_ACL="no"
BACKGROUND_COMPARISON="yes"
NUMBER_CLEANUP="yes"
NUMBER_MIN_AGE="1800"
NUMBER_LIMIT="2-10"
NUMBER_LIMIT_IMPORTANT="4-10"
TIMELINE_CREATE="no"
TIMELINE_CLEANUP="yes"
EMPTY_PRE_POST_CLEANUP="yes"
EMPTY_PRE_POST_MIN_AGE="1800"
"#;
/// The list of configs (`Base::snapper_configs` names the base's file): only root, never /home.
pub const CONFIGS: &str =
    "# Written by stackific-installer: snapper's configs\nSNAPPER_CONFIGS=\"root\"\n";

/// The "Fresh install" snapshot of the target's root, taken from the live system with snapper's `--root` after the
/// boot setup: important, so number cleanup keeps it among the last 4 to 10 important snapshots.
pub fn fresh_install(target: &Path) -> Cmd {
    Cmd::new("snapper")
        .args(["--no-dbus", "--root", &*target.to_string_lossy()])
        .args([
            "-c",
            "root",
            "create",
            "-t",
            "single",
            "-d",
            "Fresh install",
            "-c",
            "number",
            "-u",
            "important=yes",
        ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_config_and_fresh_install() {
        for line in [
            "SUBVOLUME=\"/\"\n",
            "NUMBER_LIMIT=\"2-10\"\n",
            "NUMBER_LIMIT_IMPORTANT=\"4-10\"\n",
            "FREE_LIMIT=\"0.2\"\n",
            "QGROUP=\"\"\n",
            "TIMELINE_CREATE=\"no\"\n",
        ] {
            assert!(ROOT_CONFIG.contains(line), "{line}");
        }
        assert!(CONFIGS.ends_with("\nSNAPPER_CONFIGS=\"root\"\n"));
        assert_eq!(
            fresh_install(Path::new("/t")).line(),
            r#"snapper --no-dbus --root /t -c root create -t single -d "Fresh install" -c number -u important=yes"#
        );
    }
}
