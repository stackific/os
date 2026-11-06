//! Done: where the system went, and that the drive does not start on its own yet (the boot loader comes in task 013).

use ratatui::crossterm::event::KeyCode;

use super::App;
use super::frame::{Action, Canvas, act};
use super::theme::{BODY_ROW, LEFT, TITLE_ROW};

/// Done's bottom border: Enter quits; on the console the unit then starts the live desktop.
pub fn actions(app: &App) -> Vec<Action> {
    vec![act(
        if app.env.console {
            "Enter open the live desktop"
        } else {
            "Enter quit"
        },
        KeyCode::Enter,
    )]
}

/// Draws Done.
pub fn draw(app: &App, c: &mut Canvas) {
    let device = app.install.as_ref().map_or("", |i| i.device.as_str());
    c.text(
        TITLE_ROW,
        LEFT,
        &format!("{} is on {device}.", app.env.os.name),
        app.look.title(),
    );
    let text = "The boot loader is not in this build yet, so the drive does not start on its own. Keep using the live \
                session; a later build adds the boot loader.";
    c.prose(BODY_ROW, text, app.look.body());
}
