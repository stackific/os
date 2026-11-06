//! Timezone: the installed system's zone, from `timedatectl list-timezones`, through the same list as Keyboard.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas};
use super::keyboard::{draw_search, search_actions, search_key};
use super::theme::HINT_GAP;
use super::{App, Nav};

/// The same keys as Keyboard.
pub fn key(app: &mut App, code: KeyCode, rows: usize) -> Nav {
    search_key(&mut app.timezone, code, rows)
}

/// Timezone's bottom border.
pub fn actions(app: &App) -> Vec<Action> {
    search_actions(app, &app.timezone)
}

/// Draws Timezone.
pub fn draw(app: &App, c: &mut Canvas) {
    let count = draw_search(app, c, "Timezone for the installed system", &app.timezone);
    c.prose(
        count + HINT_GAP,
        "An empty search lists every timezone, UTC first.",
        app.look.help(),
    );
}
