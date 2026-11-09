//! stackific-installer: installs Stackific OS from a live session into a drive's free space or onto a whole drive,
//! interactively (a ratatui text interface) or from an unattended TOML file. One program for both bases: the disk,
//! copy and system steps are shared, and each base's differences sit behind `base::Base`.

mod answers;
mod base;
mod bitlocker;
mod boot;
mod copy;
mod demo;
mod disk;
mod drives;
mod install;
mod os_release;
mod run;
mod snapper;
mod system;
mod ui;
mod windows;

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use answers::{Answers, Catalog};
use install::{Failure, Log, Plan};
use os_release::OsRelease;
use run::{Real, Runner};
use ui::style::Look;

const HELP: &str = "\
Usage: stackific-installer [--console] [--config <file>] [--demo [--slow]]

Installs this system into a drive's free space or onto a whole drive.

  --config <file>  install unattended with the answers in a TOML file
  --console        run on the Linux console (the boot menu's Install entry)
  --demo           walk every screen and a simulated install: no root, no disk
  --slow           with --demo: run all motion ten times slower
  --help           show this help

Exit status: 0 installed, 1 a step failed, 2 refused or invalid answers (nothing written),
3 left before the confirmation (nothing written), 130 stopped by the user.
";

/// The parsed command line.
#[derive(Debug, Default, PartialEq)]
struct Args {
    config: Option<String>,
    console: bool,
    demo: bool,
    slow: bool,
    help: bool,
}

/// Reads the flags by hand: there are five.
fn parse(args: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--config" => a.config = Some(it.next().ok_or("--config needs a file")?.clone()),
            "--console" => a.console = true,
            "--demo" => a.demo = true,
            "--slow" => a.slow = true,
            "--help" | "-h" => a.help = true,
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    Ok(a)
}

/// Refusals before any screen: exit 2 with one line.
fn refuse(msg: &str) -> i32 {
    eprintln!("stackific-installer: {msg}");
    2
}

/// The effective uid, from /proc (the standard library has no geteuid).
fn euid() -> Option<u32> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    status
        .lines()
        .find_map(|l| l.strip_prefix("Uid:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

fn main() {
    std::process::exit(real_main());
}

fn real_main() -> i32 {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a = match parse(&args) {
        Ok(a) => a,
        Err(e) => return refuse(&format!("{e}\n\n{HELP}")),
    };
    if a.help {
        print!("{HELP}");
        return 0;
    }
    let env_var = |k: &str| std::env::var(k).unwrap_or_default();
    let no_color = !env_var("NO_COLOR").is_empty();
    if a.demo {
        let os = OsRelease::parse(demo::OS_RELEASE);
        let mut look = Look::from_env(&env_var("TERM"), &env_var("COLORTERM"), no_color, os.accent);
        look.console |= a.console;
        return demo::run(a.console, a.slow, look).unwrap_or_else(|e| refuse(&e.to_string()));
    }

    // Checks: root, UEFI, a known base, its live root mounted
    if euid() != Some(0) {
        return refuse("Run it as root: run0 stackific-installer");
    }
    if !Path::new("/sys/firmware/efi").exists() {
        return refuse("this computer did not start in UEFI mode, which Stackific OS needs");
    }
    let os = OsRelease::parse(&std::fs::read_to_string("/etc/os-release").unwrap_or_default());
    let Some(base) = base::detect(&os) else {
        return refuse(&format!(
            "no installer steps for this system (ID_LIKE={:?})",
            os.id_like
        ));
    };
    let base: Arc<dyn base::Base> = base.into();
    let mounts = std::fs::read_to_string("/proc/self/mountinfo").unwrap_or_default();
    let live = base.live_root().to_string_lossy().to_string();
    if !mounts
        .lines()
        .any(|l| l.split(' ').nth(4) == Some(live.as_str()))
    {
        return refuse(&format!(
            "the live image's root is not mounted at {live}; run this from the live session"
        ));
    }
    let run: Arc<dyn Runner> = Arc::new(Real);
    let catalog = match Catalog::load(&*run, base.live_root()) {
        Ok(c) => c,
        Err(e) => return refuse(&e),
    };

    if let Some(file) = a.config {
        return unattended(&*run, &os, &*base, &catalog, &file);
    }
    let log = match std::fs::File::create(ui::stopped::LOG) {
        Ok(f) => Log::new(f),
        Err(e) => return refuse(&format!("cannot write {}: {e}", ui::stopped::LOG)),
    };
    let look = Look::from_env(&env_var("TERM"), &env_var("COLORTERM"), no_color, os.accent);
    let env = ui::Env {
        run,
        os,
        base,
        catalog,
        log,
        console: a.console,
    };
    ui::run(ui::App::new(env, look), false).unwrap_or_else(|e| {
        eprintln!("stackific-installer: {e}");
        1
    })
}

/// The unattended install: the same checks as the screens, then the same steps, logged to stderr.
fn unattended(
    run: &dyn Runner,
    os: &OsRelease,
    base: &dyn base::Base,
    catalog: &Catalog,
    file: &str,
) -> i32 {
    let answers = match std::fs::read_to_string(file)
        .map_err(|e| e.to_string())
        .and_then(|t| Answers::from_toml(&t))
    {
        Ok(a) => a,
        Err(e) => return refuse(&format!("{file}: {e}")),
    };
    if let Err(e) = answers.check(catalog, &os.name) {
        return refuse(&format!("{file}: {e}"));
    }
    let (listed, excluded) = match drives::scan(run, base.live_medium()) {
        Ok(d) => d,
        Err(e) => return refuse(&e),
    };
    let device = &answers.disk.device;
    let Some(drive) = listed.iter().find(|d| &d.path == device) else {
        let why = excluded
            .iter()
            .find(|x| &x.path == device)
            .map_or("not a drive".into(), |x| x.reason.clone());
        return refuse(&format!("{device} cannot be used: {why}"));
    };
    let layout = match disk::layout(drive, answers.disk.mode) {
        Ok(l) => l,
        Err(why) => return refuse(&format!("the free space on {device} cannot be used: {why}")),
    };
    // BitLocker and Windows on every listed drive, except the one an erase removes
    let erased = (answers.disk.mode == answers::Mode::Erase).then_some(device.as_str());
    let volumes = bitlocker::scan(run, &listed, erased);
    if !volumes.is_empty() && answers.disk.bitlocker != bitlocker::PHRASE {
        return refuse(&bitlocker::refusal(&volumes));
    }
    let found = windows::find(run, &listed, erased);
    let plan = Plan {
        answers: &answers,
        os,
        base,
        layout,
        sector: drive.sector,
        target: disk::TARGET.into(),
        windows: boot::windows_entries(&found.esps, found.firmware_entry),
    };
    let log = Log::new(std::io::stderr());
    match install::install(run, &log, &plan, &mut |_| {}, &AtomicBool::new(false)) {
        Ok(()) => {
            eprintln!("stackific-installer: {} is on {device}", os.name);
            0
        }
        Err(Failure::Stopped) => 130,
        Err(f) => {
            eprintln!("stackific-installer: the install stopped: {}", f.message());
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags() {
        let s = |v: &[&str]| v.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        assert_eq!(
            parse(&s(&["--config", "a.toml"]))
                .unwrap()
                .config
                .as_deref(),
            Some("a.toml")
        );
        assert!(
            parse(&s(&["--demo", "--slow", "--console"]))
                .is_ok_and(|a| a.demo && a.slow && a.console)
        );
        assert!(parse(&s(&["--config"])).is_err());
        assert!(parse(&s(&["--force"])).is_err());
    }
}
