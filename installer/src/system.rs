//! The shared system steps on the copied root: keyboard, timezone and computer name, the user, and the live
//! session's settings removed. Base-specific live state is `Base::remove_live_state`.

use std::path::{Path, PathBuf};

use crate::answers::Answers;
use crate::run::Cmd;

/// systemd's map from console keymaps to XKB layouts.
pub const KBD_MODEL_MAP: &str = "usr/share/systemd/kbd-model-map";

/// The console keymap for an XKB layout: the first kbd-model-map line whose layout is `layout` with no variant
/// (columns: console keymap, XKB layout, model, variant, options), or `us` when there is none.
pub fn console_keymap(map: &str, layout: &str) -> String {
    map.lines()
        .filter(|l| !l.starts_with('#'))
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .find(|c| c.len() >= 4 && c[1] == layout && c[3] == "-")
        .map_or_else(|| "us".into(), |c| c[0].to_string())
}

/// Timezone, computer name and console keymap through systemd-firstboot, which replaces the live image's files.
pub fn firstboot(target: &Path, a: &Answers, keymap: &str) -> Cmd {
    Cmd::new("systemd-firstboot").args([
        format!("--root={}", target.display()),
        "--force".into(),
        format!("--timezone={}", a.system.timezone),
        format!("--hostname={}", a.system.hostname),
        format!("--keymap={keymap}"),
    ])
}

/// Where systemd-localed keeps the X11 keyboard, which Xorg and desktops read.
pub const X11_KEYBOARD: &str = "etc/X11/xorg.conf.d/00-keyboard.conf";

/// The X11 keyboard file, in the format `localectl set-x11-keymap <layout>` writes (systemd-localed).
pub fn x11_keyboard(layout: &str) -> String {
    format!(
        "# Written by systemd-localed(8), read by systemd-localed and Xorg. It's\n\
         # probably wise not to edit this file manually. Use localectl(1) to\n\
         # update this file.\n\
         Section \"InputClass\"\n        Identifier \"system-keyboard\"\n        MatchIsKeyboard \"on\"\n\
         \x20       Option \"XkbLayout\" \"{layout}\"\nEndSection\n"
    )
}

/// The user's MangoWM config: MangoWM's stock config, then Super+Space for the shell's launcher and our layout on top,
/// so /etc/mango/ stays stock.
pub fn mango_config(layout: &str) -> String {
    format!("source=/etc/mango/config.conf\n{LAUNCHER_BIND}\nxkb_rules_layout={layout}\n")
}

/// Super+Space opens Noctalia's launcher, as in the live session (arch/iso/airootfs/home/live/.config/mango/config.conf).
pub const LAUNCHER_BIND: &str = "bind=SUPER,space,spawn,noctalia msg panel-toggle launcher";

/// The user's MangoWM config path in the target.
pub fn mango_path(target: &Path, user: &str) -> PathBuf {
    target
        .join("home")
        .join(user)
        .join(".config/mango/config.conf")
}

/// The commands that replace the live user `live` with ours, an administrator through `wheel`, and lock root: first the
/// accounts, then the passwords, so a base can relabel the account files in between (`Base::relabel`). `live` goes
/// first, so our user gets its uid 1000. The password reaches chpasswd on stdin only, hashed with yescrypt.
pub fn user(target: &Path, a: &Answers) -> (Vec<Cmd>, Vec<Cmd>) {
    let root = target.to_string_lossy().to_string();
    let u = &a.user;
    let accounts = vec![
        Cmd::new("userdel").args(["--root", &root, "--remove", "live"]),
        // In a chroot rather than with --root: Fedora's useradd copies /etc/skel only that way
        Cmd::new("chroot").args([
            &*root,
            "useradd",
            "--create-home",
            "--groups",
            "wheel",
            "--comment",
            &u.full_name,
            "--shell",
            "/usr/bin/bash",
            &u.name,
        ]),
    ];
    let passwords = vec![
        // An explicit method makes chpasswd hash with libxcrypt itself rather than through PAM, whose pam_unix fails
        // under --root on Fedora (SELinux runs its unix_update helper); yescrypt is both bases' default anyway
        Cmd::new("chpasswd")
            .args(["--root", &root, "--crypt-method", "YESCRYPT"])
            .stdin(format!("{}:{}\n", u.name, u.password)),
        Cmd::new("usermod").args(["--root", &root, "--lock", "root"]),
    ];
    (accounts, passwords)
}

/// A user's uid and gid from a passwd file.
pub fn ids(passwd: &str, user: &str) -> Option<(u32, u32)> {
    passwd
        .lines()
        .map(|l| l.split(':').collect::<Vec<_>>())
        .find(|f| f.len() > 3 && f[0] == user)
        .and_then(|f| Some((f[2].parse().ok()?, f[3].parse().ok()?)))
}

/// The live image's sysusers line that would recreate `live` on the installed system.
pub const LIVE_SYSUSERS: &str = "etc/sysusers.d/stackific-live.conf";
/// greetd's config, whose `[initial_session]` logs `live` in on the live image.
pub const GREETD: &str = "etc/greetd/config.toml";

/// greetd's config without its `[initial_session]` table; every other line, `[terminal]` and `[default_session]`
/// included, stays as it is.
pub fn without_initial_session(config: &str) -> String {
    let mut skipping = false;
    let mut out = String::new();
    for line in config.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            skipping = t == "[initial_session]";
        }
        if !skipping {
            out += line;
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAP: &str = include_str!("../fixtures/kbd-model-map");
    const GREETD_LIVE: &str = include_str!("../fixtures/greetd.toml");

    #[test]
    fn keymaps() {
        assert_eq!(console_keymap(MAP, "de"), "de");
        assert_eq!(console_keymap(MAP, "us"), "us");
        assert_eq!(console_keymap(MAP, "fr"), "fr");
        assert_eq!(console_keymap(MAP, "zz"), "us");
    }

    #[test]
    fn greetd_keeps_everything_but_the_autologin() {
        let out = without_initial_session(GREETD_LIVE);
        assert!(!out.contains("initial_session") && !out.contains("user = \"live\""));
        assert!(out.contains("[terminal]\nvt = 1\n"));
        assert!(out.contains("[default_session]\ncommand = \"/usr/bin/noctalia-greeter-session\"\nuser = \"greeter\"\n"));
        let tail = |s: &str| s[s.find("[default_session]").unwrap()..].to_string();
        assert_eq!(tail(&out), tail(GREETD_LIVE));
    }

    #[test]
    fn mango_keyboard_and_ids() {
        assert_eq!(
            mango_config("de"),
            "source=/etc/mango/config.conf\nbind=SUPER,space,spawn,noctalia msg panel-toggle launcher\nxkb_rules_layout=de\n"
        );
        assert!(x11_keyboard("de").contains("        Option \"XkbLayout\" \"de\"\nEndSection\n"));
        assert_eq!(
            ids(
                "root:x:0:0::/root:/bin/bash\ntester:x:1000:1000:T:/home/tester:/usr/bin/bash\n",
                "tester"
            ),
            Some((1000, 1000))
        );
        assert_eq!(ids("root:x:0:0::/root:/bin/bash\n", "tester"), None);
    }
}
