//! Stopped: the failing command (or the user's stop), what is left on the drive, and where the log is.

use ratatui::crossterm::event::KeyCode;

use super::App;
use super::frame::{Action, Canvas, act};
use super::theme::{BODY_ROW, LEFT, TITLE_ROW};
use crate::answers::Mode;

/// The interactive run's log.
pub const LOG: &str = "/run/stackific-installer.log";

/// Stopped's bottom border.
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

/// Draws Stopped.
pub fn draw(app: &App, c: &mut Canvas) {
    let look = app.look;
    let why = app
        .outcome
        .as_ref()
        .and_then(|o| o.as_ref().err())
        .map(|f| f.message())
        .unwrap_or_default();
    c.text(
        TITLE_ROW,
        LEFT,
        &format!("The install stopped: {why}."),
        look.error(),
    );
    let (device, mode) = app
        .install
        .as_ref()
        .map_or(("", Mode::FreeSpace), |i| (i.device.as_str(), i.mode));
    let left = match mode {
        Mode::FreeSpace => format!(
            "Only the new partitions on {device} are partly written. No other partition or drive was touched."
        ),
        Mode::Erase => {
            format!("{device} is erased and partly written. No other drive was touched.")
        }
    };
    let n = c.prose(BODY_ROW, &left, look.body());
    c.text(
        BODY_ROW + n + 1,
        LEFT,
        &format!("Details: {LOG}"),
        look.help(),
    );
}
