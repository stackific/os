//! BitLocker, between Space and Confirm (IN-02): shown only when a BitLocker volume is on a drive the install keeps.
//! Secure Boot is off, so it says what Windows will do at its next start and which recovery key it will ask for, and
//! goes on only once `I have my key` is typed. Esc goes back.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::theme::{BITLOCKER_LAST, BITLOCKER_TYPED, BODY_ROW, LEFT, ROW, TITLE_ROW, TYPED};
use super::{App, Nav};
use crate::answers::Mode;
use crate::bitlocker::{self, PHRASE, State as Lock, Volume};
use crate::drives::human;

/// The volumes found when the screen opened, and the typed confirmation.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub volumes: Vec<Volume>,
    pub typed: String,
}

/// Reads BitLocker on every listed drive except the one an erase removes; whether the screen shows.
pub fn enter(app: &mut App) -> bool {
    let erased = match (app.space.mode(), app.drive.selected()) {
        (Mode::Erase, Some(d)) => Some(d.path.clone()),
        _ => None,
    };
    let volumes = bitlocker::scan(&*app.env.run, &app.drive.listed, erased.as_deref());
    app.bitlocker = State {
        volumes,
        typed: String::new(),
    };
    !app.bitlocker.volumes.is_empty()
}

fn ready(app: &App) -> bool {
    app.bitlocker.typed == PHRASE
}

/// Typing fills the confirmation; Enter goes on once it is the phrase; Esc goes back.
pub fn key(app: &mut App, code: KeyCode) -> Nav {
    match code {
        KeyCode::Enter if ready(app) => Nav::Next,
        KeyCode::Esc => Nav::Back,
        KeyCode::Char(ch) => {
            app.bitlocker.typed.push(ch);
            Nav::Stay
        }
        KeyCode::Backspace => {
            app.bitlocker.typed.pop();
            Nav::Stay
        }
        _ => Nav::Stay,
    }
}

/// BitLocker's bottom border: "Enter next" only once the phrase is typed.
pub fn actions(app: &App) -> Vec<Action> {
    let mut a = vec![];
    if ready(app) {
        a.push(act("Enter next", KeyCode::Enter));
    }
    a.push(act("Esc back", KeyCode::Esc));
    a
}

/// Draws BitLocker: as many volumes as fit above the key's address, then how many more.
pub fn draw(app: &App, c: &mut Canvas) {
    let look = app.look;
    c.text(
        TITLE_ROW,
        LEFT,
        "Save your BitLocker recovery key",
        look.title(),
    );
    let intro = "Secure Boot is off, so Windows will ask for its BitLocker recovery key the next time it starts, \
                 unless BitLocker was suspended first.";
    let mut row = BODY_ROW + c.prose(BODY_ROW, intro, look.body()) + 1;
    let volumes = &app.bitlocker.volumes;
    let mut shown = 0;
    for v in volumes {
        let no_key = v.state == Lock::Armed && v.recovery.is_empty();
        let rows = if no_key { 4 } else { 2 };
        if row + rows > BITLOCKER_LAST {
            break;
        }
        let model = if v.model.is_empty() {
            String::new()
        } else {
            format!(" on {}", v.model)
        };
        c.text(
            row,
            ROW,
            &format!("{} ({}){model}", v.name, human(v.size)),
            look.body(),
        );
        c.text(
            row + 1,
            ROW + 2,
            &v.summary(),
            if no_key { look.warning() } else { look.body() },
        );
        if no_key {
            c.text(
                row + 2,
                ROW + 2,
                "Stop here, turn Secure Boot back on, start Windows and make one:",
                look.body(),
            );
            c.text(
                row + 3,
                ROW + 2,
                "manage-bde -protectors -add C: -RecoveryPassword",
                look.body(),
            );
        }
        row += rows + 1;
        shown += 1;
    }
    if shown < volumes.len() {
        c.text(
            row,
            ROW,
            &format!("and {} more", volumes.len() - shown),
            look.help(),
        );
    }
    c.text(
        BITLOCKER_TYPED - 2,
        LEFT,
        "Find it at aka.ms/myrecoverykey (work or school: aka.ms/aadrecoverykey).",
        look.body(),
    );
    let end = c.text(
        BITLOCKER_TYPED,
        LEFT,
        &format!("Type {PHRASE} to go on:"),
        look.title(),
    );
    let at = TYPED.max(end + 2);
    let end = c.text(BITLOCKER_TYPED, at, &app.bitlocker.typed, look.body());
    c.text(BITLOCKER_TYPED, end, look.sym("▏"), look.mark());
}
