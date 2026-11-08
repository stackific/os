//! Screen tests: a TestBackend snapshot of every screen against its text fixture, navigation, styles, the console
//! path, the mouse, motion, and the demo walked end to end with the fake runner.

use std::time::Duration;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::style::{Color, Modifier};

use super::style::Look;
use super::{App, Screen, dialog, installing};
use crate::answers::Mode;
use crate::demo;
use crate::install::Failure;
use crate::run::Fake;

const FRAME: f64 = 1.0 / 60.0;

fn look(console: bool, no_color: bool) -> Look {
    if console {
        Look::from_env("linux", "", no_color, Some((0, 122, 85)))
    } else {
        Look::from_env("foot", "truecolor", no_color, Some((0, 122, 85)))
    }
}

fn start(console: bool) -> (Fake, App) {
    let f = demo::fake(Duration::ZERO);
    let a = demo::app(&f, console, look(console, false));
    (f, a)
}

fn settle(app: &mut App) {
    for _ in 0..240 {
        app.tick(FRAME);
    }
}

fn press(app: &mut App, code: KeyCode) {
    app.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
}

fn ctrl_c(app: &mut App) {
    app.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));
}

fn typ(app: &mut App, text: &str) {
    for ch in text.chars() {
        press(app, KeyCode::Char(ch));
    }
}

fn render(app: &mut App, w: u16, h: u16) -> Buffer {
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    t.draw(|f| app.render(f)).unwrap();
    t.backend().buffer().clone()
}

fn lines(b: &Buffer, x: u16, y: u16, w: u16, h: u16) -> Vec<String> {
    (y..y + h)
        .map(|r| {
            (x..x + w)
                .map(|c| b[(c, r)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn fixture(name: &str) -> Vec<String> {
    let path = format!("{}/fixtures/screens/{name}.txt", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .lines()
        .map(String::from)
        .collect()
}

/// Renders at 80 × 24 after the motion settles and compares with the fixture, line by line.
fn snapshot(app: &mut App, name: &str) {
    settle(app);
    let got = lines(&render(app, 80, 24), 0, 0, 80, 24);
    assert_eq!(got, fixture(name), "\n{name}:\n{}", got.join("\n"));
}

/// Walks to the You screen with the drawing's answers (the confirmation one character short).
fn to_you(app: &mut App) {
    press(app, KeyCode::Enter); // Display
    press(app, KeyCode::Enter); // Welcome
    typ(app, "germ");
    press(app, KeyCode::Enter); // Keyboard: de
    typ(app, "berl");
    press(app, KeyCode::Enter); // Timezone: Europe/Berlin
    typ(app, "Ada Lovelace");
    press(app, KeyCode::Tab);
    press(app, KeyCode::Tab);
    typ(app, "brisk lantern orbith");
    press(app, KeyCode::Tab);
    typ(app, "brisk lantern orbit");
}

fn installing_at(app: &mut App, mode: Mode, step: usize, percent: u8) {
    let (mut s, _) = installing::State::idle("/dev/nvme0n1", mode);
    s.step = step;
    s.percent = percent;
    s.bar.snap(percent as f64);
    app.install = Some(s);
    app.go(Screen::Installing);
}

#[test]
fn snapshots_of_the_first_screens() {
    let (_, mut c) = start(true);
    snapshot(&mut c, "display-console");
    let (_, mut a) = start(false);
    snapshot(&mut a, "display");
    press(&mut a, KeyCode::Enter);
    snapshot(&mut a, "welcome");
    press(&mut a, KeyCode::Enter);
    typ(&mut a, "germ");
    snapshot(&mut a, "keyboard");
    press(&mut a, KeyCode::Enter);
    typ(&mut a, "berl");
    snapshot(&mut a, "timezone");
    press(&mut c, KeyCode::Enter);
    press(&mut c, KeyCode::Enter);
    typ(&mut c, "germ");
    snapshot(&mut c, "keyboard-console");
}

#[test]
fn snapshots_of_the_answers_and_the_drive() {
    let (_, mut a) = start(false);
    to_you(&mut a);
    snapshot(&mut a, "you");
    typ(&mut a, "h");
    press(&mut a, KeyCode::Enter);
    snapshot(&mut a, "drive");
    press(&mut a, KeyCode::Enter);
    snapshot(&mut a, "space");
    press(&mut a, KeyCode::Enter);
    typ(&mut a, "I have my k");
    snapshot(&mut a, "bitlocker");
    typ(&mut a, "ey");
    press(&mut a, KeyCode::Enter);
    snapshot(&mut a, "confirm-free-space");
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.screen, Screen::BitLocker);
    press(&mut a, KeyCode::Esc);
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Enter);
    typ(&mut a, "nvme0n1");
    snapshot(&mut a, "confirm-erase");
}

#[test]
fn a_long_file_system_name_keeps_a_space_before_the_label() {
    let row = super::space::part_row("vda3", 10 << 30, "BitLocker", "DESKTOP-NPM7RCA H: 7/4/2019");
    assert!(
        row.ends_with(" BitLocker DESKTOP-NPM7RCA H: 7/4/2019"),
        "{row}"
    );
    assert!(
        super::space::part_row("vda1", 100 << 20, "vfat", "SYSTEM").ends_with(" vfat    SYSTEM")
    );
}

#[test]
fn snapshots_of_bitlocker_without_a_key_and_of_rst() {
    let (f, mut a) = start(false);
    f.state().rules.insert(
        0,
        (
            "cryptsetup bitlkDump".into(),
            crate::run::Output::ok(include_str!("../../fixtures/bitlkdump-tpm-no-recovery.txt")),
        ),
    );
    to_you(&mut a);
    typ(&mut a, "h");
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.screen, Screen::BitLocker);
    // Enter waits for the phrase
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.screen, Screen::BitLocker);
    snapshot(&mut a, "bitlocker-no-key");
    // An Intel RST member is left out with guidance; hidden NVMe drives get a line
    let (f, mut a) = start(false);
    let rst = include_str!("../../fixtures/lsblk.json").replacen(
        r#""tran": "sata",
      "model": "CT500MX500SSD1",
      "fstype": null"#,
        r#""tran": "sata",
      "model": "CT500MX500SSD1",
      "fstype": "isw_raid_member""#,
        1,
    );
    assert_ne!(rst, include_str!("../../fixtures/lsblk.json"));
    {
        let mut s = f.state();
        s.rules.insert(
            0,
            (
                "lsblk --json --bytes --output NAME,PATH,TYPE,SIZE,RO".into(),
                crate::run::Output::ok(&rst),
            ),
        );
        s.rules
            .insert(0, ("sh -c ".into(), crate::run::Output::ok("1\n")));
    }
    to_you(&mut a);
    typ(&mut a, "h");
    press(&mut a, KeyCode::Enter);
    snapshot(&mut a, "drive-rst");
}

#[test]
fn snapshots_of_the_install() {
    let (_, mut a) = start(false);
    installing_at(&mut a, Mode::FreeSpace, 3, 52);
    snapshot(&mut a, "installing");
    a.dialog = Some(dialog::State {
        open: true,
        frames: 3,
    });
    snapshot(&mut a, "installing-stop");
    a.dialog = None;
    a.outcome = Some(Ok(()));
    a.go(Screen::Done);
    snapshot(&mut a, "done");
    a.outcome = Some(Err(Failure::Command {
        summary: "mkfs.btrfs /dev/nvme0n1p5".into(),
        code: 1,
    }));
    a.go(Screen::Stopped);
    snapshot(&mut a, "stopped");
    let (_, mut c) = start(true);
    installing_at(&mut c, Mode::FreeSpace, 7, 100);
    c.outcome = Some(Ok(()));
    c.go(Screen::Done);
    snapshot(&mut c, "done-console");
}

#[test]
fn a_small_terminal_and_a_large_one() {
    let (_, mut a) = start(false);
    let mut want = fixture("small");
    want.resize(18, String::new());
    assert_eq!(lines(&render(&mut a, 60, 18), 0, 0, 60, 18), want);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    typ(&mut a, "germ");
    settle(&mut a);
    let b = render(&mut a, 200, 60);
    assert_eq!(lines(&b, 60, 18, 80, 24), fixture("keyboard"));
    assert!(lines(&b, 0, 0, 200, 18).iter().all(String::is_empty));
}

#[test]
fn colour_contrast_and_selection_styles() {
    // NO_COLOR: no cell has a colour
    let f = demo::fake(Duration::ZERO);
    let mut a = demo::app(&f, false, look(false, true));
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    settle(&mut a);
    let b = render(&mut a, 80, 24);
    assert!(
        b.content
            .iter()
            .all(|c| c.fg == Color::Reset && c.bg == Color::Reset)
    );
    // Selection: the accent and reverse video, not colour alone
    let (_, mut a) = start(false);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    settle(&mut a);
    let b = render(&mut a, 80, 24);
    let row = lines(&b, 0, 0, 80, 24)
        .iter()
        .position(|l| l.contains("English (US)"))
        .unwrap() as u16;
    let cell = &b[(5, row)];
    assert_eq!(cell.fg, Color::Rgb(0, 122, 85));
    assert!(cell.modifier.contains(Modifier::REVERSED));
    // Contrast: High takes effect in the same frame, and nothing is dim
    let (_, mut a) = start(false);
    let dim = |b: &Buffer| b.content.iter().any(|c| c.modifier.contains(Modifier::DIM));
    assert!(dim(&render(&mut a, 80, 24)));
    press(&mut a, KeyCode::Down);
    press(&mut a, KeyCode::Right);
    for screen in [Screen::Display, Screen::Welcome, Screen::Keyboard] {
        a.go(screen);
        settle(&mut a);
        let b = render(&mut a, 80, 24);
        assert!(!dim(&b), "{screen:?} has dim cells");
        assert!(
            b.content.iter().all(|c| c.bg == Color::Reset),
            "{screen:?} has a tint"
        );
    }
}

#[test]
fn navigation_keeps_answers_and_shows_next_only_when_valid() {
    let (f, mut a) = start(false);
    to_you(&mut a);
    let footer = |a: &mut App| lines(&render(a, 80, 24), 0, 23, 80, 1)[0].clone();
    assert!(!footer(&mut a).contains("Enter next"));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.screen, Screen::You);
    typ(&mut a, "h");
    assert!(footer(&mut a).contains("Enter next"));
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.screen, Screen::Drive);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.screen, Screen::You);
    assert_eq!(
        a.you.fields,
        [
            "Ada Lovelace",
            "ada",
            "brisk lantern orbith",
            "brisk lantern orbith",
            "stackific"
        ]
        .map(String::from)
    );
    // Ctrl+C before the install leaves at once, with nothing written
    ctrl_c(&mut a);
    assert_eq!(a.exit, Some(3));
    let writes = [
        "sfdisk --wipe",
        "sfdisk --append",
        "mkfs",
        "mount",
        "rsync",
        "chroot",
    ];
    assert!(
        f.state()
            .calls
            .iter()
            .all(|c| !writes.iter().any(|w| c.starts_with(w)))
    );
}

/// Walks every screen to Confirm in `mode` and starts the install.
fn install(a: &mut App, mode: Mode) {
    to_you(a);
    typ(a, "h");
    press(a, KeyCode::Enter);
    press(a, KeyCode::Enter);
    if mode == Mode::Erase {
        press(a, KeyCode::Down);
    }
    press(a, KeyCode::Enter);
    if mode == Mode::Erase {
        // The BitLocker volume is on the drive the erase removes: no BitLocker screen
        assert_eq!(a.screen, Screen::Confirm);
        typ(a, "nvme0n1");
    } else {
        assert_eq!(a.screen, Screen::BitLocker);
        typ(a, "I have my key");
        press(a, KeyCode::Enter);
    }
    press(a, KeyCode::Enter);
    assert_eq!(a.screen, Screen::Installing);
}

fn wait_for_end(a: &mut App) {
    for _ in 0..2000 {
        a.tick(FRAME);
        if a.outcome.is_some() {
            return;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    panic!("the install did not end");
}

#[test]
fn the_demo_walks_every_screen_and_installs_both_ways() {
    for mode in [Mode::FreeSpace, Mode::Erase] {
        let (f, mut a) = start(false);
        install(&mut a, mode);
        wait_for_end(&mut a);
        assert_eq!(a.screen, Screen::Done, "{:?}", a.outcome);
        press(&mut a, KeyCode::Enter);
        assert_eq!(a.exit, Some(0));
        let s = f.state();
        assert!(
            s.unexpected.is_empty(),
            "unexpected commands: {:?}",
            s.unexpected
        );
        let wrote_table = s
            .calls
            .iter()
            .filter(|c| c.starts_with("sfdisk --wipe") || c.starts_with("sfdisk --append"))
            .count();
        assert_eq!(wrote_table, 1);
    }
}

#[test]
fn ctrl_c_during_the_install_asks_and_esc_keeps_installing() {
    let f = demo::fake(Duration::from_millis(5));
    let mut a = demo::app(&f, false, look(false, false));
    install(&mut a, Mode::FreeSpace);
    ctrl_c(&mut a);
    assert!(a.dialog.is_some_and(|d| d.open));
    settle_frames(&mut a, 3);
    let b = render(&mut a, 80, 24);
    assert!(
        lines(&b, 0, 0, 80, 24)
            .iter()
            .any(|l| l.contains("Stop the install?"))
    );
    press(&mut a, KeyCode::Esc);
    settle_frames(&mut a, 3);
    assert!(a.dialog.is_none());
    wait_for_end(&mut a);
    assert_eq!(a.outcome, Some(Ok(())));
    // Enter in the dialog stops it: Stopped, the bell once, exit 130
    let f = demo::fake(Duration::from_millis(20));
    let mut a = demo::app(&f, false, look(false, false));
    install(&mut a, Mode::FreeSpace);
    ctrl_c(&mut a);
    press(&mut a, KeyCode::Enter);
    wait_for_end(&mut a);
    assert_eq!(
        (a.screen, a.outcome.clone()),
        (Screen::Stopped, Some(Err(Failure::Stopped)))
    );
    assert!(std::mem::take(&mut a.bell));
    a.tick(FRAME);
    assert!(!a.bell);
    press(&mut a, KeyCode::Enter);
    assert_eq!(a.exit, Some(130));
}

fn settle_frames(a: &mut App, n: usize) {
    for _ in 0..n {
        a.tick(FRAME);
    }
}

#[test]
fn the_console_path() {
    let (f, mut a) = start(true);
    // The first pick is set at once; ←→ set each offered size with setfont
    assert_eq!(f.calls("setfont"), ["setfont latarcyrheb-sun32"]);
    press(&mut a, KeyCode::Right);
    for _ in 0..3 {
        press(&mut a, KeyCode::Left);
    }
    assert_eq!(
        f.calls("setfont"),
        [
            "setfont latarcyrheb-sun32",
            "setfont solar24x32",
            "setfont latarcyrheb-sun32",
            "setfont sun12x22",
            "setfont default8x16"
        ]
    );
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.exit, Some(3));
    let (f, mut a) = start(true);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Esc);
    assert_eq!(a.exit, Some(3));
    assert!(f.state().calls.iter().all(|c| !c.starts_with("sfdisk --")));
    // Motion starts Reduced on the console: screens change at once
    let (_, mut a) = start(true);
    press(&mut a, KeyCode::Enter);
    assert!(!a.animating());
}

#[test]
fn the_drive_list_is_read_again() {
    let (f, mut a) = start(false);
    to_you(&mut a);
    typ(&mut a, "h");
    let scans = |f: &Fake| f.calls("lsblk --json --bytes --output NAME,").len();
    press(&mut a, KeyCode::Enter);
    assert_eq!(scans(&f), 1);
    a.handle(Event::FocusGained);
    assert_eq!(scans(&f), 2);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Esc);
    assert_eq!(scans(&f), 3);
}

fn mouse(a: &mut App, kind: MouseEventKind, column: u16, row: u16) {
    a.handle(Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }));
}

#[test]
fn the_mouse_presses_releases_and_cancels() {
    let (_, mut a) = start(false);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    settle(&mut a);
    render(&mut a, 80, 24);
    let first = a.keyboard.cursor.sel;
    // Press on the third row: highlighted, not selected; release on it: selected
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), 10, 8);
    assert!(a.pressed.is_some());
    assert_eq!(a.keyboard.cursor.sel, first);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), 10, 8);
    assert_eq!(a.keyboard.cursor.sel, 2);
    // Moving off before the release cancels
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), 10, 9);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), 10, 2);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), 10, 9);
    assert_eq!(a.keyboard.cursor.sel, 2);
    // A click on "Esc back" in the bottom border acts like Esc
    render(&mut a, 80, 24);
    let footer = lines(&render(&mut a, 80, 24), 0, 23, 80, 1)[0].clone();
    let col = footer
        .chars()
        .collect::<Vec<_>>()
        .windows(3)
        .position(|w| w.iter().collect::<String>() == "Esc")
        .unwrap() as u16;
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), col + 1, 23);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), col + 1, 23);
    assert_eq!(a.screen, Screen::Welcome);
}

#[test]
fn a_wheel_flick_lands_where_the_projection_says() {
    let (_, mut a) = start(false);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    render(&mut a, 80, 24);
    // Five wheel events 10 ms apart: 5 rows, then 125 rows/s × 0.099 ≈ 12 more
    for _ in 0..5 {
        mouse(&mut a, MouseEventKind::ScrollDown, 10, 8);
        a.tick(0.01);
    }
    assert_eq!(a.keyboard.cursor.sel, 5);
    a.tick(0.1);
    let v = 5.0 / 0.04;
    assert_eq!(
        a.keyboard.cursor.sel,
        5 + super::motion::projection(v, 0.99).round() as usize
    );
}

#[test]
fn motion_slides_glides_and_holds() {
    let (_, mut a) = start(false);
    let dx = |a: &App| (a.screen.place() * super::theme::SLIDE_COLUMNS - a.camera.pos).round();
    press(&mut a, KeyCode::Enter);
    a.tick(FRAME);
    assert!(dx(&a) > 0.0, "forward slides in from the right");
    settle(&mut a);
    assert_eq!(dx(&a), 0.0);
    press(&mut a, KeyCode::Enter);
    settle(&mut a);
    press(&mut a, KeyCode::Esc);
    a.tick(FRAME);
    assert!(dx(&a) < 0.0, "back slides out to the right");
    // Reduced motion: at once
    let (_, mut a) = start(false);
    press(&mut a, KeyCode::Right);
    assert!(!a.full_motion());
    press(&mut a, KeyCode::Enter);
    assert_eq!(dx(&a), 0.0);
    // A held ↓ speeds up after half a second
    let (_, mut a) = start(false);
    press(&mut a, KeyCode::Enter);
    press(&mut a, KeyCode::Enter);
    for _ in 0..20 {
        press(&mut a, KeyCode::Down);
        a.tick(0.03);
    }
    // 17 repeats before 0.5 s move one row each, the 3 after it three each
    assert_eq!(a.keyboard.cursor.sel, 17 + 3 * 3);
    // The dialog grows from its anchor and returns to it
    assert_eq!(dialog::rect(0), (2, 23, 11, 2));
    assert_eq!(dialog::rect(3), super::theme::DIALOG);
}
