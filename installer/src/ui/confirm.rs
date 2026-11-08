//! Confirm, the last screen before anything is written (IN-03): what is on the drive now and what will be on it
//! afterwards, and the answers. Free space installs on Enter; an erase also needs the drive's kernel name typed.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::space::{describe, draw_parts, part_row};
use super::theme::{BODY_ROW, LEFT, ROW, TITLE_ROW, TYPED};
use super::{App, Nav};
use crate::answers::Mode;
use crate::disk;
use crate::drives::human;

/// The typed confirmation of an erase.
#[derive(Clone, Debug, Default)]
pub struct State {
    pub typed: String,
}

/// Whether Enter installs now: always for free space, for an erase once the typed text is the drive's kernel name.
fn ready(app: &App) -> bool {
    match (app.space.mode(), app.drive.selected()) {
        (Mode::FreeSpace, Some(_)) => true,
        (Mode::Erase, Some(d)) => app.confirm.typed == d.name,
        _ => false,
    }
}

/// Enter installs when ready; on an erase, typing fills the confirmation; Esc goes back.
pub fn key(app: &mut App, code: KeyCode) -> Nav {
    let erase = app.space.mode() == Mode::Erase;
    match code {
        KeyCode::Enter if ready(app) => Nav::Install,
        KeyCode::Esc => Nav::Back,
        KeyCode::Char(ch) if erase => {
            app.confirm.typed.push(ch);
            Nav::Stay
        }
        KeyCode::Backspace if erase => {
            app.confirm.typed.pop();
            Nav::Stay
        }
        _ => Nav::Stay,
    }
}

/// Confirm's bottom border: "Enter install" only when Enter installs.
pub fn actions(app: &App) -> Vec<Action> {
    let mut a = vec![];
    if ready(app) {
        a.push(act("Enter install", KeyCode::Enter));
    }
    a.push(act("Esc back", KeyCode::Esc));
    a
}

/// Draws Confirm.
pub fn draw(app: &App, c: &mut Canvas) {
    let look = app.look;
    let Some(d) = app.drive.selected() else {
        return;
    };
    let name = &app.env.os.name;
    let erase = app.space.mode() == Mode::Erase;
    let Ok(layout) = disk::layout(d, app.space.mode()) else {
        return;
    };
    let (sector, new) = (d.sector, if erase { ["1", "2"] } else { ["new", "new"] });
    if erase {
        c.text(
            TITLE_ROW,
            LEFT,
            &format!("Erase {}?", describe(d)),
            look.title(),
        );
        c.text(BODY_ROW, LEFT, "Now on it, all erased:", look.body());
    } else {
        c.text(
            TITLE_ROW,
            LEFT,
            &format!("Install {name} in the free space on {}?", d.path),
            look.title(),
        );
        c.text(BODY_ROW, LEFT, "Stays as it is:", look.body());
    }
    let last = draw_parts(c, d, BODY_ROW + 1, false);
    let head = if erase {
        "After the install:".to_string()
    } else {
        format!(
            "New, in the {} of free space:",
            human(d.free.as_ref().map_or(0, |g| g.bytes()))
        )
    };
    c.text(last + 2, LEFT, &head, look.body());
    let esp = part_row(
        new[0],
        layout.esp.1 * sector,
        "FAT32",
        &format!("{name} boot"),
    );
    let sys = part_row(new[1], layout.system.1 * sector, "btrfs", name);
    c.text(last + 3, ROW, &esp, look.body());
    c.text(last + 4, ROW, &sys, look.body());
    let a = app.answers();
    let layout_name = app
        .env
        .catalog
        .layout_name(&a.system.keyboard)
        .unwrap_or("");
    let r = last + 6;
    c.text(
        r,
        LEFT,
        &format!(
            "Keyboard {} ({layout_name}) · Timezone {}",
            a.system.keyboard, a.system.timezone
        ),
        look.body(),
    );
    let user = format!(
        "User {} ({}), administrator · Computer {}",
        a.user.full_name, a.user.name, a.system.hostname
    );
    c.text(r + 1, LEFT, &user, look.body());
    if erase {
        let end = c.text(
            r + 3,
            LEFT,
            &format!("Type {} to erase it:", d.name),
            look.title(),
        );
        let at = TYPED.max(end + 2);
        let end = c.text(r + 3, at, &app.confirm.typed, look.body());
        c.text(r + 3, end, look.sym("▏"), look.mark());
    } else {
        c.text(
            r + 3,
            LEFT,
            "Nothing else on the drive changes.",
            look.body(),
        );
    }
    if !app.windows.esps.is_empty() {
        c.text(
            r + 5,
            LEFT,
            "Limine's menu will also start Windows.",
            look.body(),
        );
    }
}
