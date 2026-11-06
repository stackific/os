//! The answers an install needs, from the screens or from an unattended TOML file, and the checks both paths share.

use std::fmt;
use std::path::Path;

use serde::Deserialize;

use crate::run::{Cmd, Runner};

/// XKB's layout list, with each layout's code and name.
pub const EVDEV_LST: &str = "/usr/share/X11/xkb/rules/evdev.lst";
/// The layout the live sessions type with: MangoWM's stock config and the console's default keymap.
pub const LIVE_LAYOUT: &str = "us";
/// The keyboard and timezone lists start with these when the search is empty.
pub const DEFAULT_LAYOUT: &str = "us";
pub const DEFAULT_ZONE: &str = "UTC";

/// How the chosen drive is used.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// Two new partitions in the drive's largest free space; nothing else changes.
    #[default]
    FreeSpace,
    /// A new partition table with our two partitions; everything on the drive is erased.
    Erase,
}

/// Every answer, in the unattended file's shape (`[disk]`, `[system]`, `[user]`).
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Answers {
    pub disk: DiskAnswers,
    pub system: SystemAnswers,
    pub user: UserAnswers,
}

/// `[disk]`: the drive, the mode, and for an erase the typed confirmation (the drive's kernel name).
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DiskAnswers {
    pub device: String,
    pub mode: Mode,
    #[serde(default)]
    pub erase: String,
}

/// `[system]`: an XKB layout code, a timezone and the computer name.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SystemAnswers {
    pub keyboard: String,
    pub timezone: String,
    pub hostname: String,
}

/// `[user]`: the administrator to create.
#[derive(Clone, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct UserAnswers {
    pub full_name: String,
    pub name: String,
    pub password: String,
}

/// Leaves the password out, so no debug print or log can hold it.
impl fmt::Debug for UserAnswers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserAnswers")
            .field("full_name", &self.full_name)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// What the checks compare answers against: the layouts, the timezones and the user names the live root already has.
#[derive(Clone, Debug, Default)]
pub struct Catalog {
    /// XKB layouts as (code, name), in evdev.lst's order.
    pub layouts: Vec<(String, String)>,
    pub zones: Vec<String>,
    pub taken_users: Vec<String>,
}

impl Catalog {
    /// Reads the layouts from evdev.lst, the zones from `timedatectl list-timezones` (local, no daemon involved) and the
    /// users from the live root's /etc/passwd.
    pub fn load(run: &dyn Runner, live_root: &Path) -> Result<Self, String> {
        let evdev = run
            .read(Path::new(EVDEV_LST))
            .map_err(|e| format!("cannot read {EVDEV_LST}: {e}"))?;
        let zones = run
            .run(&Cmd::new("timedatectl").args(["list-timezones"]))
            .map_err(|e| e.to_string())?;
        if zones.code != 0 {
            return Err(format!(
                "timedatectl list-timezones failed: {}",
                zones.stderr.trim()
            ));
        }
        let passwd = run
            .read(&live_root.join("etc/passwd"))
            .map_err(|e| format!("cannot read the live root's passwd: {e}"))?;
        Ok(Self {
            layouts: parse_layouts(&evdev),
            zones: zones
                .stdout
                .lines()
                .map(str::trim)
                .filter(|z| !z.is_empty())
                .map(String::from)
                .collect(),
            taken_users: taken_users(&passwd),
        })
    }

    /// The name of layout `code`, if XKB has it.
    pub fn layout_name(&self, code: &str) -> Option<&str> {
        self.layouts
            .iter()
            .find(|(c, _)| c == code)
            .map(|(_, n)| n.as_str())
    }
}

/// The `! layout` section of evdev.lst: lines of a code and a name.
pub fn parse_layouts(evdev: &str) -> Vec<(String, String)> {
    evdev
        .lines()
        .skip_while(|l| l.trim() != "! layout")
        .skip(1)
        .take_while(|l| !l.trim().is_empty() && !l.starts_with('!'))
        .filter_map(|l| {
            l.trim()
                .split_once(char::is_whitespace)
                .map(|(c, n)| (c.to_string(), n.trim().to_string()))
        })
        .collect()
}

/// User names in a passwd file, except the live user, which the install removes.
pub fn taken_users(passwd: &str) -> Vec<String> {
    passwd
        .lines()
        .filter_map(|l| l.split(':').next())
        .filter(|n| !n.is_empty() && *n != "live")
        .map(String::from)
        .collect()
}

/// The user name the You screen suggests: the first word of the name, lowercased, ASCII letters and digits only.
pub fn suggested_user_name(full_name: &str) -> String {
    let first = full_name.split_whitespace().next().unwrap_or("");
    first
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .take(32)
        .collect()
}

/// A name for the passwd comment field: not empty, no `:` (the field separator) and no control characters.
pub fn check_full_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Type your name.".into());
    }
    if name.contains(':') || name.chars().any(char::is_control) {
        return Err("Your name cannot contain : or control characters.".into());
    }
    Ok(())
}

/// `^[a-z_][a-z0-9_-]{0,31}$`, and not a user the live root already has.
pub fn check_user_name(name: &str, taken: &[String]) -> Result<(), String> {
    if name.is_empty() {
        return Err("Type a user name.".into());
    }
    let mut chars = name.chars();
    let first_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c == '_');
    let rest_ok =
        chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
    if !first_ok || !rest_ok || name.len() > 32 {
        return Err("The user name does not follow the rule below.".into());
    }
    if taken.iter().any(|t| t == name) {
        return Err(format!(
            "The user name {name} belongs to the system. Pick another."
        ));
    }
    Ok(())
}

/// A password that is not empty and, on the screen, typed the same twice.
pub fn check_password(password: &str, confirm: &str) -> Result<(), String> {
    if password.is_empty() {
        return Err("Type a password.".into());
    }
    // chpasswd reads `name:password` lines, so a line break would end the password early
    if password.chars().any(char::is_control) {
        return Err("The password cannot contain control characters.".into());
    }
    if password != confirm {
        return Err("The passwords do not match yet.".into());
    }
    Ok(())
}

/// `^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$`
pub fn check_hostname(name: &str) -> Result<(), String> {
    let ok = !name.is_empty()
        && name.len() <= 63
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-');
    if ok {
        Ok(())
    } else {
        Err("The computer name does not follow the rule below.".into())
    }
}

/// The typed confirmation of an erase: exactly the drive's kernel name (`vda` for /dev/vda).
pub fn check_erase(device: &str, typed: &str) -> Result<(), String> {
    let kernel = device.rsplit('/').next().unwrap_or(device);
    if typed == kernel {
        Ok(())
    } else {
        Err(format!(
            "To erase {device}, the confirmation must be {kernel}."
        ))
    }
}

impl Answers {
    /// Parses an unattended file.
    pub fn from_toml(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }

    /// Every check except the drive's own (drives.rs): the first problem, as one line.
    pub fn check(&self, catalog: &Catalog) -> Result<(), String> {
        if catalog.layout_name(&self.system.keyboard).is_none() {
            return Err(format!(
                "keyboard: {:?} is not an XKB layout code from evdev.lst",
                self.system.keyboard
            ));
        }
        if !catalog.zones.contains(&self.system.timezone) {
            return Err(format!(
                "timezone: {:?} is not a timezone",
                self.system.timezone
            ));
        }
        check_hostname(&self.system.hostname).map_err(|e| format!("hostname: {e}"))?;
        check_full_name(&self.user.full_name).map_err(|e| format!("full_name: {e}"))?;
        check_user_name(&self.user.name, &catalog.taken_users).map_err(|e| format!("name: {e}"))?;
        check_password(&self.user.password, &self.user.password)
            .map_err(|e| format!("password: {e}"))?;
        if self.disk.mode == Mode::Erase {
            check_erase(&self.disk.device, &self.disk.erase).map_err(|e| format!("erase: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVDEV: &str = include_str!("../fixtures/evdev.lst");

    fn catalog() -> Catalog {
        Catalog {
            layouts: parse_layouts(EVDEV),
            zones: vec!["UTC".into(), "Europe/Berlin".into()],
            taken_users: taken_users(
                "root:x:0:0::/root:/usr/bin/bash\nlive:x:1000:1000::/home/live:/usr/bin/bash\n",
            ),
        }
    }

    fn good() -> Answers {
        Answers::from_toml(
            "[disk]\ndevice = \"/dev/vda\"\nmode = \"erase\"\nerase = \"vda\"\n[system]\nkeyboard = \"de\"\n\
             timezone = \"Europe/Berlin\"\nhostname = \"stackific-test\"\n[user]\nfull_name = \"Test User\"\n\
             name = \"tester\"\npassword = \"secret\"\n",
        )
        .unwrap()
    }

    #[test]
    fn layouts_come_from_the_layout_section() {
        let l = parse_layouts(EVDEV);
        assert_eq!(l.len(), 99);
        assert!(l.contains(&("de".into(), "German".into())));
        assert!(l.contains(&("us".into(), "English (US)".into())));
    }

    #[test]
    fn user_names() {
        let taken = catalog().taken_users;
        for good in ["ada", "_svc", "a-b_c9", &"a".repeat(32)] {
            assert_eq!(check_user_name(good, &taken), Ok(()), "{good}");
        }
        for bad in [
            "",
            "Ada",
            "9ada",
            "-ada",
            "ad a",
            "adä",
            &"a".repeat(33),
            "root",
        ] {
            assert!(check_user_name(bad, &taken).is_err(), "{bad}");
        }
        assert_eq!(check_user_name("live", &taken), Ok(()));
        assert_eq!(suggested_user_name("Ada Lovelace"), "ada");
        assert_eq!(suggested_user_name("  Zoë O'Neil"), "zo");
    }

    #[test]
    fn computer_names_passwords_and_names() {
        for good in ["stackific", "a", "stackific-test", "x1", &"a".repeat(63)] {
            assert_eq!(check_hostname(good), Ok(()), "{good}");
        }
        for bad in ["", "-a", "a-", "A", "a.b", "a_b", &"a".repeat(64)] {
            assert!(check_hostname(bad).is_err(), "{bad}");
        }
        assert!(check_password("", "").is_err());
        assert!(check_password("pa\nroot:x", "pa\nroot:x").is_err());
        assert_eq!(
            check_password("x", "y"),
            Err("The passwords do not match yet.".into())
        );
        assert_eq!(check_password("x", "x"), Ok(()));
        assert!(
            check_full_name("").is_err()
                && check_full_name("a:b").is_err()
                && check_full_name("a\nb").is_err()
        );
        assert_eq!(check_full_name("Ada Lovelace"), Ok(()));
    }

    #[test]
    fn unattended_answers() {
        let c = catalog();
        assert_eq!(good().check(&c), Ok(()));
        let mut a = good();
        a.disk.erase = "sda".into();
        assert!(a.check(&c).unwrap_err().starts_with("erase:"));
        a.disk.mode = Mode::FreeSpace;
        assert_eq!(a.check(&c), Ok(()));
        let mut a = good();
        a.system.keyboard = "xx".into();
        assert!(a.check(&c).unwrap_err().starts_with("keyboard:"));
        let mut a = good();
        a.system.timezone = "Mars/Olympus".into();
        assert!(a.check(&c).unwrap_err().starts_with("timezone:"));
        let mut a = good();
        a.user.password.clear();
        assert!(a.check(&c).unwrap_err().starts_with("password:"));
        assert!(Answers::from_toml("[disk]\ndevice = \"/dev/vda\"\nmode = \"wipe\"\n").is_err());
        assert!(
            Answers::from_toml(&format!(
                "{}\nextra = 1\n",
                "[disk]\ndevice = \"x\"\nmode = \"erase\""
            ))
            .is_err()
        );
        assert!(!format!("{:?}", good()).contains("secret"));
    }
}
