//! Welcome: what the installer does, and that nothing is written before the last screen.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::theme::{BODY_ROW, LEFT, TITLE_ROW};
use super::{App, Nav};

/// Enter starts; Esc leaves (on the console, to the live desktop).
pub fn key(_: &mut App, code: KeyCode) -> Nav {
    match code {
        KeyCode::Enter => Nav::Next,
        KeyCode::Esc => Nav::Quit(super::EXIT_LEFT),
        _ => Nav::Stay,
    }
}

/// Welcome's bottom border.
pub fn actions(app: &App) -> Vec<Action> {
    vec![act("Enter start", KeyCode::Enter), app.esc_quit()]
}

/// Draws Welcome.
pub fn draw(app: &App, c: &mut Canvas) {
    let (look, name) = (app.look, &app.env.os.name);
    c.text(
        TITLE_ROW,
        LEFT,
        &format!("Install {name} on this computer."),
        look.title(),
    );
    let n = c.prose(
        BODY_ROW,
        "It goes into a drive's free space, next to what is already there, or onto a whole drive, which it erases first.",
        look.body(),
    );
    c.prose(
        BODY_ROW + n + 1,
        "You pick a keyboard layout, a timezone, your user, the drive and how to use it. Nothing is written until you \
         confirm on the last screen.",
        look.body(),
    );
}
