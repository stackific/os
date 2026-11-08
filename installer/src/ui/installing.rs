//! Installing: one line per step and the copy's progress bar, while the install runs on its own thread, so input and
//! motion never wait for it. Ctrl+C asks first (the stop dialog).

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use super::frame::{Action, Canvas};
use super::motion::Spring;
use super::theme::{BAR, BODY_ROW, LEFT, MARK, MAX_SPEED, PERCENT, PROGRESS, ROW, STEP, TITLE_ROW};
use super::{App, Screen};
use crate::answers::Mode;
use crate::disk::TARGET;
use crate::install::{self, Failure, Plan, Progress, STEPS};

/// What the install thread sends.
pub enum Msg {
    Progress(Progress),
    Finished(Result<(), Failure>),
}

/// The running install: its messages, its stop flag, where it is, and the bar.
pub struct State {
    pub rx: Receiver<Msg>,
    pub stop: Arc<AtomicBool>,
    pub device: String,
    pub mode: Mode,
    pub step: usize,
    pub percent: u8,
    /// The bar's position in percent: a spring that never moves backwards.
    pub bar: Spring,
}

impl State {
    /// A state with no thread behind it (tests, snapshots).
    pub fn idle(device: &str, mode: Mode) -> (Self, mpsc::Sender<Msg>) {
        let (tx, rx) = mpsc::channel();
        let s = Self {
            rx,
            stop: Arc::new(AtomicBool::new(false)),
            device: device.into(),
            mode,
            step: 0,
            percent: 0,
            bar: Spring::new(PROGRESS, 0.0, MAX_SPEED * 4.0),
        };
        (s, tx)
    }
}

/// Starts the install with the screens' answers on its own thread, and shows Installing.
pub fn start(app: &mut App) {
    let answers = app.answers();
    let Some(drive) = app.drive.selected() else {
        return;
    };
    let (layout, sector) = match crate::disk::layout(drive, answers.disk.mode) {
        Ok(l) => (l, drive.sector),
        Err(_) => return,
    };
    let windows = crate::boot::windows_entries(&app.windows.esps, app.windows.firmware_entry);
    let (state, tx) = State::idle(&answers.disk.device, answers.disk.mode);
    let stop = state.stop.clone();
    let (run, base, os, log) = (
        app.env.run.clone(),
        app.env.base.clone(),
        app.env.os.clone(),
        app.env.log.clone(),
    );
    thread::spawn(move || {
        let plan = Plan {
            answers: &answers,
            os: &os,
            base: &*base,
            layout,
            sector,
            target: TARGET.into(),
            windows,
        };
        let progress = tx.clone();
        let r = install::install(
            &*run,
            &log,
            &plan,
            &mut |p| drop(progress.send(Msg::Progress(p))),
            &stop,
        );
        let _ = tx.send(Msg::Finished(r));
    });
    app.install = Some(state);
    app.go(Screen::Installing);
}

/// Takes the install's news and moves the bar; the end shows Done or Stopped.
pub fn poll(app: &mut App, dt: f64) {
    let Some(s) = app.install.as_mut() else {
        return;
    };
    let mut finished = None;
    while let Ok(m) = s.rx.try_recv() {
        match m {
            Msg::Progress(Progress::Step(n)) => s.step = n,
            Msg::Progress(Progress::Percent(p)) => s.percent = s.percent.max(p),
            Msg::Finished(r) => finished = Some(r),
        }
    }
    s.bar.retarget(s.percent as f64);
    let before = s.bar.pos;
    s.bar.step(dt, false);
    s.bar.pos = s.bar.pos.max(before);
    if let Some(r) = finished {
        let ok = r.is_ok();
        app.outcome = Some(r);
        app.dialog = None;
        app.go(if ok { Screen::Done } else { Screen::Stopped });
    }
}

/// Installing's bottom border: only Ctrl+C.
pub fn actions(_: &App) -> Vec<Action> {
    vec![Action {
        label: "Ctrl+C stop".into(),
        code: KeyCode::Char('c'),
        mods: KeyModifiers::CONTROL,
    }]
}

/// The bar: full cells, then the partial cell in eighths (halves on the console, whose fonts have only those),
/// then the track.
pub fn bar(fraction: f64, console: bool) -> String {
    let filled = fraction.clamp(0.0, 1.0) * BAR as f64;
    let mut full = filled.floor() as usize;
    let mut eighths = ((filled - full as f64) * 8.0).round() as usize;
    if eighths == 8 {
        full += 1;
        eighths = 0;
    }
    let partial = if console {
        if eighths >= 4 { "▌" } else { "" }
    } else {
        ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"][eighths]
    };
    let track = BAR as usize - full - partial.chars().count();
    format!("{}{partial}{}", "█".repeat(full), "░".repeat(track))
}

/// Draws Installing.
pub fn draw(app: &App, c: &mut Canvas) {
    let look = app.look;
    let Some(s) = &app.install else { return };
    c.text(
        TITLE_ROW,
        LEFT,
        &format!("Installing {} on {}", app.env.os.name, s.device),
        look.title(),
    );
    for (i, step) in STEPS.iter().enumerate() {
        let r = BODY_ROW + i as u16;
        let (mark, style) = match i.cmp(&s.step) {
            std::cmp::Ordering::Less => (look.sym("✓"), look.body()),
            std::cmp::Ordering::Equal => (look.sym("▸"), look.title()),
            std::cmp::Ordering::Greater => (" ", look.help()),
        };
        c.text(r, MARK, mark, look.mark());
        c.text(r, ROW, step, style);
        if i == s.step && i == install::COPY_STEP {
            let end = c.text(
                r,
                ROW + STEP,
                &bar(s.bar.pos / 100.0, look.console),
                look.mark(),
            );
            c.text(
                r,
                end + 2,
                &format!("{:>PERCENT$}", format!("{}%", s.percent)),
                look.body(),
            );
        }
    }
    c.text(
        BODY_ROW + STEPS.len() as u16 + 1,
        LEFT,
        "Do not turn the computer off.",
        look.body(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bar_moves_in_eighths() {
        assert_eq!(
            bar(0.52, false),
            format!("{}▏{}", "█".repeat(15), "░".repeat(13))
        );
        assert_eq!(
            bar(15.5 / 29.0, false),
            format!("{}▌{}", "█".repeat(15), "░".repeat(13))
        );
        assert_eq!(
            bar(15.5 / 29.0, true),
            format!("{}▌{}", "█".repeat(15), "░".repeat(13))
        );
        assert_eq!(bar(1.0, false), "█".repeat(29));
        assert_eq!(bar(0.0, true), "░".repeat(29));
    }
}
