//! Done: where the system went, and how to start it.

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
    let name = &app.env.os.name;
    let text = format!(
        "Remove the USB stick, then restart the computer to start {name}. If it starts something else, pick \"{name}\" \
         in its boot menu."
    );
    c.prose(BODY_ROW, &text, app.look.body());
}
