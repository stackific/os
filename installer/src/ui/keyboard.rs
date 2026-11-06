//! Keyboard: the installed system's XKB layout, from evdev.lst, through the type-to-search list Timezone shares.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::list::SearchList;
use super::theme::{BODY_ROW, CODE, HINT_GAP, LEFT, ROW, TITLE_ROW};
use super::{App, Nav};
use crate::answers::LIVE_LAYOUT;

/// Typing searches, the list keys choose, Enter goes on once something matches, Esc goes back.
pub fn key(app: &mut App, code: KeyCode, rows: usize) -> Nav {
    search_key(&mut app.keyboard, code, rows)
}

/// The keys of a search screen (Keyboard, Timezone).
pub fn search_key(list: &mut SearchList, code: KeyCode, rows: usize) -> Nav {
    match code {
        KeyCode::Enter if list.selected().is_some() => Nav::Next,
        KeyCode::Esc => Nav::Back,
        KeyCode::Enter => Nav::Stay,
        other => {
            list.key(other, rows);
            Nav::Stay
        }
    }
}

/// A search screen's bottom border.
pub fn search_actions(app: &App, list: &SearchList) -> Vec<Action> {
    let mut a = vec![
        act(format!("{} choose", app.look.sym("↑↓")), KeyCode::Down),
        act("type to search", KeyCode::Null),
    ];
    if list.selected().is_some() {
        a.push(act("Enter next", KeyCode::Enter));
    }
    a.push(act("Esc back", KeyCode::Esc));
    a
}

/// Keyboard's bottom border.
pub fn actions(app: &App) -> Vec<Action> {
    search_actions(app, &app.keyboard)
}

/// Draws a search screen: its title, the search field, the list (a code column when items have names), the count;
/// returns the row of the count.
pub fn draw_search(app: &App, c: &mut Canvas, title: &str, list: &SearchList) -> u16 {
    let look = app.look;
    c.text(TITLE_ROW, LEFT, title, look.title());
    let end = c.text(
        BODY_ROW,
        LEFT,
        &format!("Search: {}", list.query),
        look.body(),
    );
    c.text(BODY_ROW, end, look.sym("▏"), look.mark());
    let first = BODY_ROW + 2;
    list.cursor.draw(c, first, app.pressed, |c, r, i, sel| {
        let (code, name) = &list.items[list.matches[i]];
        let style = if sel { look.selected() } else { look.body() };
        let text = if name.is_empty() {
            code.clone()
        } else {
            format!("{code:<w$}{name}", w = CODE as usize)
        };
        c.text(r, ROW, &text, style);
    });
    let count = first + list.cursor.shown() as u16 + 1;
    c.text(count, LEFT, &list.count(), look.help());
    count
}

/// Draws Keyboard: the hint says the layout is for the installed system (left out on the console).
pub fn draw(app: &App, c: &mut Canvas) {
    let count = draw_search(
        app,
        c,
        "Keyboard layout for the installed system",
        &app.keyboard,
    );
    if !app.env.console {
        let hint = format!(
            "It takes effect on the installed system. Here, you keep typing with the live session's layout ({LIVE_LAYOUT})."
        );
        c.prose(count + HINT_GAP, &hint, app.look.help());
    }
}
