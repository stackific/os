//! The install itself, after the confirmation: the eight steps the Installing screen lists, in order, each command
//! and its output logged (never stdin, which carries the password), then the target unmounted. The screens and the
//! unattended run share it.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::answers::{Answers, Mode};
use crate::base::Base;
use crate::os_release::OsRelease;
use crate::run::{Cmd, Output, Runner};
use crate::{boot, copy, disk, system};

/// The steps, as the Installing screen names them.
pub const STEPS: [&str; 8] = [
    "Create the partitions",
    "Format them",
    "Prepare the system partition",
    "Copy the system",
    "Set the keyboard, timezone and computer name",
    "Create your user",
    "Remove the live session's settings",
    "Install the boot loader",
];
/// The step that reports a percentage.
pub const COPY_STEP: usize = 3;

/// Where each command and its output go: /run/stackific-installer.log on the screens, stderr when unattended.
#[derive(Clone)]
pub struct Log(Arc<Mutex<Box<dyn Write + Send>>>);

impl Log {
    /// A log writing to `w`.
    pub fn new(w: impl Write + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(Box::new(w))))
    }

    /// Appends one line; a log that cannot be written never stops the install.
    pub fn line(&self, text: &str) {
        let mut w = self.0.lock().expect("log lock");
        let _ = writeln!(w, "{text}");
        let _ = w.flush();
    }
}

/// What the install reports while it runs.
#[derive(Clone, Debug, PartialEq)]
pub enum Progress {
    /// Step `n` of STEPS starts.
    Step(usize),
    /// The copy's overall percentage.
    Percent(u8),
}

/// Why an install did not finish.
#[derive(Clone, Debug, PartialEq)]
pub enum Failure {
    /// A command exited non-zero: its short form and exit code.
    Command { summary: String, code: i32 },
    /// Something else went wrong, in one line.
    Other(String),
    /// The user stopped it.
    Stopped,
}

impl Failure {
    /// The line the Stopped screen and stderr show.
    pub fn message(&self) -> String {
        match self {
            Failure::Command { summary, code } => format!("{summary} failed (exit {code})"),
            Failure::Other(m) => m.clone(),
            Failure::Stopped => "you stopped it".into(),
        }
    }
}

/// Everything the install needs, fixed at the confirmation.
pub struct Plan<'a> {
    pub answers: &'a Answers,
    pub os: &'a OsRelease,
    pub base: &'a dyn Base,
    /// Where the two partitions go (`disk::layout`), in sectors of `sector` bytes.
    pub layout: disk::Layout,
    pub sector: u64,
    pub target: PathBuf,
}

/// Runs one command, logs it and its output, and turns a non-zero exit into a Failure.
fn exec(run: &dyn Runner, log: &Log, cmd: &Cmd) -> Result<Output, Failure> {
    log.line(&format!("$ {}", cmd.line()));
    let out = run
        .run(cmd)
        .map_err(|e| Failure::Other(format!("{}: {e}", cmd.summary())))?;
    for text in [&out.stdout, &out.stderr] {
        if !text.trim().is_empty() {
            log.line(text.trim_end());
        }
    }
    if out.code != 0 {
        return Err(Failure::Command {
            summary: cmd.summary(),
            code: out.code,
        });
    }
    Ok(out)
}

/// Reads a file through the runner, as a Failure when it cannot.
fn read(run: &dyn Runner, path: &Path) -> Result<String, Failure> {
    run.read(path)
        .map_err(|e| Failure::Other(format!("cannot read {}: {e}", path.display())))
}

/// Writes a file through the runner and logs that it did.
fn write(
    run: &dyn Runner,
    log: &Log,
    path: &Path,
    text: &str,
    owner: Option<(u32, u32)>,
) -> Result<(), Failure> {
    log.line(&format!("write {}", path.display()));
    run.write(path, text, owner)
        .map_err(|e| Failure::Other(format!("cannot write {}: {e}", path.display())))
}

/// Removes a file through the runner and logs that it did.
fn remove(run: &dyn Runner, log: &Log, path: &Path) -> Result<(), Failure> {
    log.line(&format!("remove {}", path.display()));
    run.remove(path)
        .map_err(|e| Failure::Other(format!("cannot remove {}: {e}", path.display())))
}

/// Installs, then unmounts the target whatever happened. `report` hears each step and the copy's percentage; a set
/// `stop` ends the install at the next command (the copy at once).
pub fn install(
    run: &dyn Runner,
    log: &Log,
    plan: &Plan,
    report: &mut dyn FnMut(Progress),
    stop: &AtomicBool,
) -> Result<(), Failure> {
    let result = steps(run, log, plan, report, stop);
    let t = plan.target.to_string_lossy().to_string();
    let mounted = run
        .run(&Cmd::new("findmnt").args(["--mountpoint", &t]))
        .is_ok_and(|o| o.code == 0);
    let unmount = if mounted {
        exec(run, log, &Cmd::new("umount").args(["--recursive", &t])).map(|_| ())
    } else {
        Ok(())
    };
    result.and(unmount)
}

fn steps(
    run: &dyn Runner,
    log: &Log,
    plan: &Plan,
    report: &mut dyn FnMut(Progress),
    stop: &AtomicBool,
) -> Result<(), Failure> {
    let a = plan.answers;
    let dev = a.disk.device.as_str();
    let target = plan.target.as_path();
    let step = |n: usize, report: &mut dyn FnMut(Progress)| {
        if stop.load(Ordering::Relaxed) {
            return Err(Failure::Stopped);
        }
        log.line(&format!("==> {}", STEPS[n]));
        report(Progress::Step(n));
        Ok(())
    };

    step(0, report)?;
    let layout = &plan.layout;
    match a.disk.mode {
        Mode::FreeSpace => exec(run, log, &disk::append(dev, &plan.os.name, layout))?,
        Mode::Erase => exec(run, log, &disk::erase(dev, &plan.os.name, layout))?,
    };
    let starts = [layout.esp.0 * plan.sector, layout.system.0 * plan.sector];
    exec(run, log, &Cmd::new("udevadm").args(["settle"]))?;
    let lsblk = exec(
        run,
        log,
        &Cmd::new("lsblk").args([
            "--json",
            "--bytes",
            "--output",
            "NAME,PATH,TYPE,SIZE,START",
            dev,
        ]),
    )?;
    let [esp, sys] = disk::find_by_start(&lsblk.stdout, starts).map_err(Failure::Other)?;

    step(1, report)?;
    for cmd in disk::format(&esp, &sys, &plan.os.id) {
        exec(run, log, &cmd)?;
    }

    step(2, report)?;
    for cmd in disk::prepare(&esp, &sys, target) {
        exec(run, log, &cmd)?;
    }

    step(COPY_STEP, report)?;
    let rsync = copy::rsync(plan.base.live_root(), target);
    log.line(&format!("$ {}", rsync.line()));
    let out = run
        .stream(
            &rsync,
            &mut |line| {
                copy::percent(line)
                    .into_iter()
                    .for_each(|p| report(Progress::Percent(p)))
            },
            stop,
        )
        .map_err(|e| Failure::Other(format!("rsync: {e}")))?;
    log.line(out.stdout.trim());
    if !out.stderr.trim().is_empty() {
        log.line(out.stderr.trim_end());
    }
    if stop.load(Ordering::Relaxed) {
        return Err(Failure::Stopped);
    }
    if out.code != 0 {
        return Err(Failure::Command {
            summary: rsync.summary(),
            code: out.code,
        });
    }

    step(4, report)?;
    let uuid = |d: &str| {
        exec(
            run,
            log,
            &Cmd::new("blkid").args(["-s", "UUID", "-o", "value", d]),
        )
        .map(|o| o.stdout.trim().to_string())
    };
    let (sys_uuid, esp_uuid) = (uuid(&sys)?, uuid(&esp)?);
    write(
        run,
        log,
        &target.join("etc/fstab"),
        &disk::fstab(&sys_uuid, &esp_uuid),
        None,
    )?;
    let keymap = system::console_keymap(
        &read(run, &target.join(system::KBD_MODEL_MAP))?,
        &a.system.keyboard,
    );
    exec(run, log, &system::firstboot(target, a, &keymap))?;
    write(
        run,
        log,
        &target.join(system::X11_KEYBOARD),
        &system::x11_keyboard(&a.system.keyboard),
        None,
    )?;

    step(5, report)?;
    for cmd in system::user(target, a) {
        exec(run, log, &cmd)?;
    }
    let ids = system::ids(&read(run, &target.join("etc/passwd"))?, &a.user.name)
        .ok_or_else(|| Failure::Other(format!("{} is not in the new passwd", a.user.name)))?;
    write(
        run,
        log,
        &system::mango_path(target, &a.user.name),
        &system::mango_config(&a.system.keyboard),
        Some(ids),
    )?;

    step(6, report)?;
    remove(run, log, &target.join(system::LIVE_SYSUSERS))?;
    let greetd = target.join(system::GREETD);
    write(
        run,
        log,
        &greetd,
        &system::without_initial_session(&read(run, &greetd)?),
        None,
    )?;
    let (files, cmds) = plan.base.live_state(target);
    for f in files {
        remove(run, log, &f)?;
    }
    for cmd in cmds {
        exec(run, log, &cmd)?;
    }

    // After the live state, so the initramfs is built without archiso's mkinitcpio drop-in
    step(7, report)?;
    for cmd in plan.base.initramfs(target) {
        exec(run, log, &cmd)?;
    }
    for cmd in boot::copy_program(target) {
        exec(run, log, &cmd)?;
    }
    let (kernel, initramfs) = plan.base.kernel_files();
    write(
        run,
        log,
        &target.join(boot::CONF),
        &boot::limine_conf(&plan.os.name, kernel, initramfs, &sys_uuid),
        None,
    )?;
    // The firmware entry last, so a failure before it leaves none behind
    let listing = exec(run, log, &boot::entries())?;
    let present = exec(run, log, &boot::partuuids())?;
    for num in boot::stale(&listing.stdout, &present.stdout) {
        exec(run, log, &boot::delete(&num))?;
    }
    let part = boot::partition_number(&esp)
        .ok_or_else(|| Failure::Other(format!("{esp} has no partition number")))?;
    exec(run, log, &boot::create(dev, part, &plan.os.name))?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::base::Arch;
    use crate::run::Fake;

    /// A writer whose bytes a test can read back.
    #[derive(Clone, Default)]
    pub struct Buf(pub Arc<Mutex<Vec<u8>>>);

    impl Write for Buf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl Buf {
        pub fn text(&self) -> String {
            String::from_utf8_lossy(&self.0.lock().unwrap()).into()
        }
    }

    /// Answers for a test install on /dev/vda.
    pub fn answers(mode: Mode) -> Answers {
        let mut a = Answers::default();
        a.disk.device = "/dev/vda".into();
        a.disk.mode = mode;
        a.disk.erase = "vda".into();
        a.system.keyboard = "de".into();
        a.system.timezone = "Europe/Berlin".into();
        a.system.hostname = "stackific-test".into();
        a.user.full_name = "Test User".into();
        a.user.name = "tester".into();
        a.user.password = "correct horse battery".into();
        a
    }

    /// A fake that answers every command an install on /dev/vda runs, for either mode.
    pub fn install_fake() -> Fake {
        let f = Fake::default();
        let t = disk::TARGET;
        let parts = r#"{"blockdevices":[{"name":"vda","path":"/dev/vda","type":"disk","size":1,"start":null,"children":[
            {"name":"vda1","path":"/dev/vda1","type":"part","size":1,"start":2048},
            {"name":"vda2","path":"/dev/vda2","type":"part","size":1,"start":8390656},
            {"name":"vda4","path":"/dev/vda4","type":"part","size":1,"start":25217024},
            {"name":"vda6","path":"/dev/vda6","type":"part","size":1,"start":33605632}]}]}"#;
        for p in [
            "sfdisk --wipe",
            "sfdisk --append",
            "udevadm settle",
            "mkfs.",
            "mkdir -p",
            "mount -o",
            "btrfs subvolume",
            "chattr +C",
        ] {
            f.on(p, Output::ok(""));
        }
        f.on(
            "lsblk --json --bytes --output NAME,PATH,TYPE,SIZE,START /dev/vda",
            Output::ok(parts),
        )
        .on(&format!("umount {t}"), Output::ok(""))
        .on(&format!("umount --recursive {t}"), Output::ok(""))
        .on(&format!("findmnt --mountpoint {t}"), Output::ok("TARGET"))
        .on(
            "rsync ",
            Output::ok("3,620,000,000 100%   80.00MB/s    0:00:43 (xfr#210000, to-chk=0/215000)"),
        )
        .on("blkid -s UUID -o value /dev/vda2", Output::ok("1111-sys\n"))
        .on("blkid -s UUID -o value /dev/vda6", Output::ok("1111-sys\n"))
        .on("blkid -s UUID -o value /dev/vda1", Output::ok("AAAA-ESP\n"))
        .on("blkid -s UUID -o value /dev/vda4", Output::ok("AAAA-ESP\n"))
        .on("systemd-firstboot", Output::ok(""))
        .on("userdel ", Output::ok(""))
        .on("useradd ", Output::ok(""))
        .on("chpasswd ", Output::ok(""))
        .on("usermod ", Output::ok(""))
        .on("pacman --sysroot", Output::ok(""))
        .on("arch-chroot ", Output::ok(""))
        .on("install -D ", Output::ok(""))
        .on("efibootmgr --", Output::ok(""))
        .on(
            "efibootmgr",
            Output::ok(include_str!("../fixtures/efibootmgr.txt")),
        )
        .on(
            "lsblk --noheadings --list --output PARTUUID",
            Output::ok("\n"),
        );
        f.file(&format!("{t}/{}", system::KBD_MODEL_MAP), include_str!("../fixtures/kbd-model-map"))
            .file(&format!("{t}/etc/passwd"), "root:x:0:0::/root:/usr/bin/bash\ntester:x:1000:1000:Test User:/home/tester:/usr/bin/bash\n")
            .file(&format!("{t}/{}", system::GREETD), include_str!("../fixtures/greetd.toml"))
            .file(&format!("{t}/{}", system::LIVE_SYSUSERS), "m live wheel\n");
        f.state().stream_lines = vec![
            "  1,000  10%".into(),
            "  5,000  52%".into(),
            "  9,000 100%".into(),
        ];
        f
    }

    /// Where an install on the 64 GiB test disk puts the partitions: an erase, or the gap after partition 3.
    pub fn layout(mode: Mode) -> disk::Layout {
        match mode {
            Mode::Erase => disk::erase_layout(64 << 30, 512),
            Mode::FreeSpace => disk::Layout::in_gap(&crate::drives::Gap {
                start: 25_217_024,
                end: 134_215_680,
                sector: 512,
            }),
        }
    }

    fn run_install(mode: Mode) -> (Fake, Buf, Vec<Progress>, Result<(), Failure>) {
        let f = install_fake();
        let buf = Buf::default();
        let os = OsRelease::parse("NAME=\"Stackific OS\"\nID=stackific\nID_LIKE=arch\n");
        let a = answers(mode);
        let plan = Plan {
            answers: &a,
            os: &os,
            base: &Arch,
            layout: layout(mode),
            sector: 512,
            target: disk::TARGET.into(),
        };
        let mut seen = Vec::new();
        let r = install(
            &f,
            &Log::new(buf.clone()),
            &plan,
            &mut |p| seen.push(p),
            &AtomicBool::new(false),
        );
        (f, buf, seen, r)
    }

    #[test]
    fn erase_install_runs_every_step_and_never_logs_the_password() {
        let (f, log, seen, r) = run_install(Mode::Erase);
        assert_eq!(r, Ok(()));
        assert!(
            f.state().unexpected.is_empty(),
            "{:?}",
            f.state().unexpected
        );
        assert_eq!(
            seen.iter()
                .filter(|p| matches!(p, Progress::Step(_)))
                .count(),
            8
        );
        assert!(seen.contains(&Progress::Percent(52)));
        let log = log.text();
        assert!(!log.contains("correct horse"), "{log}");
        assert!(
            f.state()
                .stdins
                .iter()
                .any(|s| s == "tester:correct horse battery\n")
        );
        assert!(log.contains("$ mkfs.btrfs -f -L stackific /dev/vda2"));
        let s = f.state();
        let t = disk::TARGET;
        assert_eq!(
            s.files[&PathBuf::from(format!("{t}/home/tester/.config/mango/config.conf"))],
            "source=/etc/mango/config.conf\nbind=SUPER,space,spawn,noctalia msg panel-toggle launcher\nxkb_rules_layout=de\n"
        );
        assert_eq!(
            s.owners[&PathBuf::from(format!("{t}/home/tester/.config/mango/config.conf"))],
            (1000, 1000)
        );
        assert!(
            !s.files
                .contains_key(&PathBuf::from(format!("{t}/{}", system::LIVE_SYSUSERS)))
        );
        assert!(
            !s.files[&PathBuf::from(format!("{t}/{}", system::GREETD))].contains("initial_session")
        );
        assert!(
            s.files[&PathBuf::from(format!("{t}/etc/fstab"))]
                .contains("UUID=AAAA-ESP /boot vfat umask=0077 0 2")
        );
        // `live` goes before our user, so ours gets uid 1000
        let pos = |p: &str| s.calls.iter().position(|c| c.starts_with(p)).unwrap();
        assert!(pos("userdel") < pos("useradd"));
        // The boot loader after the live state (archiso's mkinitcpio drop-in gone), the firmware entry last: the
        // fixture's entry on a partition lsblk no longer lists goes, ours is created on the ESP, partition 1
        assert!(pos("pacman --sysroot") < pos("arch-chroot"));
        assert!(pos("arch-chroot") < pos("install -D"));
        assert_eq!(
            s.files[&PathBuf::from(format!("{t}/boot/limine.conf"))],
            boot::limine_conf(
                "Stackific OS",
                "vmlinuz-linux",
                "initramfs-linux.img",
                "1111-sys"
            )
        );
        assert!(pos("efibootmgr --bootnum 0009 --delete-bootnum") < pos("efibootmgr --create"));
        assert_eq!(
            s.calls[s.calls.len() - 3],
            r#"efibootmgr --create --disk /dev/vda --part 1 --label "Stackific OS" --loader \EFI\limine\BOOTX64.EFI"#
        );
        assert_eq!(s.calls.last().unwrap(), &format!("umount --recursive {t}"));
    }

    #[test]
    fn free_space_install_writes_into_the_gap() {
        let (f, _, _, r) = run_install(Mode::FreeSpace);
        assert_eq!(r, Ok(()));
        assert!(
            f.state().unexpected.is_empty(),
            "{:?}",
            f.state().unexpected
        );
        assert!(f.calls("sfdisk --wipe").is_empty());
        assert_eq!(
            f.calls("mkfs.fat"),
            ["mkfs.fat -F 32 -n STACKIFIC /dev/vda4"]
        );
        assert!(f.calls("efibootmgr --create")[0].contains(" --part 4 "));
    }

    #[test]
    fn a_failing_command_stops_the_install_and_unmounts() {
        let f = install_fake();
        f.state()
            .rules
            .insert(0, ("mkfs.btrfs".into(), Output::fail(1, "boom")));
        let os = OsRelease::parse("NAME=\"Stackific OS\"\nID=stackific\n");
        let a = answers(Mode::Erase);
        let plan = Plan {
            answers: &a,
            os: &os,
            base: &Arch,
            layout: layout(Mode::Erase),
            sector: 512,
            target: disk::TARGET.into(),
        };
        let r = install(
            &f,
            &Log::new(Buf::default()),
            &plan,
            &mut |_| {},
            &AtomicBool::new(false),
        );
        assert_eq!(
            r.unwrap_err().message(),
            "mkfs.btrfs /dev/vda2 failed (exit 1)"
        );
        assert!(f.calls("mount ").is_empty());
        let stopped = install(
            &f,
            &Log::new(Buf::default()),
            &plan,
            &mut |_| {},
            &AtomicBool::new(true),
        );
        assert_eq!(stopped, Err(Failure::Stopped));
    }
}
