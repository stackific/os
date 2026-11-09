//! The demo: every screen and a simulated install against fixture drives, through the fake runner, so it runs on
//! any machine without root and without touching a disk (`just installer demo`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::answers::{Catalog, EVDEV_LST};
use crate::base::{Arch, Base};
use crate::disk::TARGET;
use crate::install::Log;
use crate::os_release::OsRelease;
use crate::run::{Cmd, Fake, Output};
use crate::system;
use crate::ui::{App, Env, display, style::Look};

/// The os-release the demo shows.
pub const OS_RELEASE: &str = "NAME=\"Stackific OS\"\nID=stackific\nID_LIKE=arch\nDEFAULT_HOSTNAME=stackific\nANSI_COLOR=\"38;2;0;122;85\"\n";

/// A fake that answers everything the screens and an install on the fixture drives run. `delay` paces the copy.
pub fn fake(delay: Duration) -> Fake {
    let f = Fake::default();
    let lsblk = include_str!("../fixtures/lsblk.json");
    let protective = r#"{"partitiontable":{"label":"dos","partitions":[{"start":1,"size":1953525167,"type":"ee"}]}}"#;
    // The new partitions where an install would put them: an erase (1 MiB, then 4 GiB later) or the free space
    let new = r#"{"blockdevices":[{"name":"D","path":"/dev/D","type":"disk","size":1,"start":null,"children":[
        {"name":"Dp1","path":"/dev/Dp1","type":"part","size":1,"start":2048},
        {"name":"Dp2","path":"/dev/Dp2","type":"part","size":1,"start":8390656},
        {"name":"Dp4","path":"/dev/Dp4","type":"part","size":1,"start":1701865472},
        {"name":"Dp5","path":"/dev/Dp5","type":"part","size":1,"start":1710254080}]}]}"#;
    f.on(
        &format!(
            "lsblk --json --bytes --output {}",
            crate::drives::LSBLK_COLUMNS
        ),
        Output::ok(lsblk),
    )
    .on(
        "findmnt --noheadings --output SOURCE /run/archiso/bootmnt",
        Output::ok("/dev/sdb1\n"),
    )
    .on(
        "sfdisk --json /dev/nvme0n1",
        Output::ok(include_str!("../fixtures/sfdisk-nvme0n1.json")),
    )
    .on(
        "sfdisk --verify /dev/nvme0n1",
        Output::ok("/dev/nvme0n1:\nNo errors detected.\n"),
    )
    .on(
        "sfdisk --json --label-nested dos /dev/nvme0n1",
        Output::ok(protective),
    )
    .on(
        "sfdisk --json /dev/sda",
        Output::fail(
            1,
            "sfdisk: /dev/sda: does not contain a recognized partition table",
        ),
    )
    .on(
        "blkid -p -o export /dev/nvme0n1p1",
        Output::ok("TYPE=vfat\n"),
    )
    .on("blkid -p -o export /dev/nvme0n1p2", Output::fail(2, ""))
    .on(
        "blkid -p -o export /dev/nvme0n1p3",
        Output::ok(include_str!("../fixtures/blkid-bitlocker.txt")),
    )
    // Windows 11's usual protectors: the TPM, and a recovery key
    .on(
        "cryptsetup bitlkDump /dev/nvme0n1p3",
        Output::ok(
            &include_str!("../fixtures/bitlkdump-recovery.txt")
                .replace("VMK protected with passphrase", "VMK protected with TPM"),
        ),
    )
    // Windows' loader on nvme0n1p1, found read-only; no NVMe hidden by Intel RST
    .on("test -f ", Output::ok(""))
    .on("sh -c ", Output::ok(""))
    .on(
        "timedatectl list-timezones",
        Output::ok(include_str!("../fixtures/timezones.txt")),
    )
    .on(
        "lsblk --json --bytes --output NAME,PATH,TYPE,SIZE,START /dev/nvme0n1",
        Output::ok(&new.replace("/dev/D", "/dev/nvme0n1")),
    )
    .on(
        "lsblk --json --bytes --output NAME,PATH,TYPE,SIZE,START /dev/sda",
        Output::ok(
            &new.replace("/dev/Dp", "/dev/sda")
                .replace("/dev/D", "/dev/sda"),
        ),
    )
    .on(
        &format!("findmnt --mountpoint {TARGET}"),
        Output::ok("TARGET\n"),
    )
    .on("blkid -s UUID -o value", Output::ok("0a1b2c3d-demo\n"))
    .on(
        "rsync ",
        Output::ok("3,620,000,000 100%   80.00MB/s    0:00:43 (xfr#210000, to-chk=0/215000)"),
    );
    for ok in [
        "setfont ",
        "sfdisk --wipe",
        "sfdisk --append",
        "udevadm settle",
        "cryptsetup luksFormat ",
        "cryptsetup open ",
        "cryptsetup status ",
        "cryptsetup close ",
        "mkfs.",
        "mkdir -p",
        "mount -o",
        "btrfs subvolume",
        "chattr +C",
        "chmod 750 ",
        "snapper --no-dbus --root ",
        "umount ",
        "systemd-firstboot ",
        "userdel ",
        "chroot ",
        "chpasswd ",
        "usermod ",
        "env SNAP_PAC_SKIP=y pacman --sysroot",
        "arch-chroot ",
        "install -D ",
        "efibootmgr --",
    ] {
        f.on(ok, Output::ok(""));
    }
    f.on(
        "efibootmgr",
        Output::ok(include_str!("../fixtures/efibootmgr-windows.txt")),
    )
    .on(
        "lsblk --noheadings --list --output PARTUUID",
        Output::ok("\n"),
    );
    f.file(EVDEV_LST, include_str!("../fixtures/evdev.lst"))
        .file(display::FB_SIZE, "3840,2160\n")
        .file("/run/archiso/airootfs/etc/passwd", "root:x:0:0::/root:/usr/bin/bash\nlive:x:1000:1000:Live user:/home/live:/usr/bin/bash\n")
        .file(&format!("{TARGET}/{}", system::KBD_MODEL_MAP), include_str!("../fixtures/kbd-model-map"))
        .file(&format!("{TARGET}/{}", system::GREETD), include_str!("../fixtures/greetd.toml"))
        .file(&format!("{TARGET}/{}", system::LIVE_SYSUSERS), "m live wheel\n");
    {
        let mut s = f.state();
        s.effects.push(("chroot ".into(), add_user));
        s.stream_lines = (0..=100u64)
            .step_by(2)
            .map(|p| {
                format!(
                    "{:>13} {p:>3}%   80.00MB/s    0:00:{:02}",
                    p * 36_200_000,
                    p / 2
                )
            })
            .collect();
        s.delay = delay;
    }
    f
}

/// What `chroot <root> useradd … <name>` leaves in the root's passwd: the user, with uid and gid 1000.
fn add_user(cmd: &Cmd, files: &mut HashMap<PathBuf, String>) {
    let root = cmd.args.first().cloned().unwrap_or_default();
    let name = cmd.args.last().cloned().unwrap_or_default();
    files
        .entry(PathBuf::from(root).join("etc/passwd"))
        .or_default()
        .push_str(&format!("{name}:x:1000:1000::/home/{name}:/usr/bin/bash\n"));
}

/// The demo's App: the fixture drives and lists, on the console's screens with `console`.
pub fn app(f: &Fake, console: bool, look: Look) -> App {
    let os = OsRelease::parse(OS_RELEASE);
    let base: Arc<dyn Base> = Arc::new(Arch);
    let catalog = Catalog::load(f, base.live_root()).expect("demo fixtures");
    // On a fake system the user name `tester` should stay free: the demo's passwd has only root and live
    let env = Env {
        run: Arc::new(f.clone()),
        os,
        base,
        catalog,
        log: Log::new(std::io::sink()),
        console,
    };
    App::new(env, look)
}

/// Runs the demo in this terminal.
pub fn run(console: bool, slow: bool, look: Look) -> std::io::Result<i32> {
    let f = fake(Duration::from_millis(60));
    let code = crate::ui::run(app(&f, console, look), slow)?;
    let unexpected = f.state().unexpected.clone();
    if !unexpected.is_empty() {
        eprintln!(
            "demo: commands the fake runner was not told to expect:\n  {}",
            unexpected.join("\n  ")
        );
    }
    Ok(code)
}
