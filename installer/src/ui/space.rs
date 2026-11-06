//! Space: how to use the drive. "Use the free space" comes first when it is usable; otherwise its line says why not,
//! and a drive with no partition table offers only the erase. Below, what is on the drive now.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::list::Cursor;
use super::theme::{
    BODY_ROW, HINT_GAP, LEFT, OPTION, PART_FS, PART_LABEL, PART_NAME, PART_SIZE, ROW, TITLE_ROW,
};
use super::{App, Nav};
use crate::answers::Mode;
use crate::drives::{Drive, human};

/// The options for the selected drive: each mode, and whether it can be picked.
#[derive(Clone, Debug)]
pub struct State {
    pub options: Vec<(Mode, bool)>,
    pub cursor: Cursor,
}

impl Default for State {
    fn default() -> Self {
        Self {
            options: vec![],
            cursor: Cursor::new(0, 2, 0),
        }
    }
}

impl State {
    /// The options that can be picked, in order.
    fn enabled(&self) -> Vec<Mode> {
        self.options
            .iter()
            .filter(|(_, ok)| *ok)
            .map(|(m, _)| *m)
            .collect()
    }

    /// The mode the cursor is on.
    pub fn mode(&self) -> Mode {
        self.enabled()
            .get(self.cursor.sel)
            .copied()
            .unwrap_or(Mode::Erase)
    }
}

/// Builds the options for the drive chosen on Drive.
pub fn enter(app: &mut App) {
    let options = match app.drive.selected() {
        Some(d) if d.table.is_some() => {
            vec![(Mode::FreeSpace, d.free.is_ok()), (Mode::Erase, true)]
        }
        _ => vec![(Mode::Erase, true)],
    };
    let enabled = options.iter().filter(|(_, ok)| *ok).count();
    app.space = State {
        options,
        cursor: Cursor::new(enabled, 2, 0),
    };
}

/// The list keys choose; Enter goes on; Esc goes back.
pub fn key(app: &mut App, code: KeyCode, rows: usize) -> Nav {
    match code {
        KeyCode::Enter => Nav::Next,
        KeyCode::Esc => Nav::Back,
        other => {
            app.space.cursor.key(other, rows);
            Nav::Stay
        }
    }
}

/// Space's bottom border.
pub fn actions(app: &App) -> Vec<Action> {
    vec![
        act(format!("{} choose", app.look.sym("↑↓")), KeyCode::Down),
        act("Enter next", KeyCode::Enter),
        act("Esc back", KeyCode::Esc),
    ]
}

/// "/dev/nvme0n1 (Samsung SSD 990 PRO 1TB, 931.5 GiB)"
pub fn describe(d: &Drive) -> String {
    if d.model.is_empty() {
        format!("{} ({})", d.path, human(d.size))
    } else {
        format!("{} ({}, {})", d.path, d.model, human(d.size))
    }
}

/// One table row: name, size (right-aligned), file system, label.
pub fn part_row(name: &str, bytes: u64, fs: &str, label: &str) -> String {
    let fs_gap = (PART_FS - ROW - PART_NAME) as usize - PART_SIZE;
    format!(
        "{name:<nw$}{size:>sw$}{:fs_gap$}{fs:<fw$}{label}",
        "",
        size = human(bytes),
        nw = PART_NAME as usize,
        sw = PART_SIZE,
        fw = (PART_LABEL - PART_FS) as usize
    )
}

/// The drive's partitions by start, with its largest free space among them when `with_free`; returns the last row.
pub fn draw_parts(c: &mut Canvas, d: &Drive, row: u16, with_free: bool) -> u16 {
    let look = c.look;
    let mut rows: Vec<(u64, String)> = d
        .parts
        .iter()
        .map(|p| {
            (
                p.start.unwrap_or(0) * 512,
                part_row(
                    &p.name,
                    p.size,
                    p.fstype.as_deref().unwrap_or(""),
                    p.label.as_deref().unwrap_or(""),
                ),
            )
        })
        .collect();
    if with_free && let Some(g) = &d.gap {
        rows.push((g.start * g.sector, part_row("free", g.bytes(), "", "")));
    }
    rows.sort_by_key(|(at, _)| *at);
    if rows.is_empty() {
        c.text(
            row,
            ROW,
            if d.table.is_none() {
                "no partition table"
            } else {
                "no partitions"
            },
            look.help(),
        );
        return row;
    }
    for (i, (_, text)) in rows.iter().enumerate() {
        c.text(row + i as u16, ROW, text, look.body());
    }
    row + rows.len() as u16 - 1
}

/// An option's label and what it does to this drive (or why it cannot be picked).
fn option(d: &Drive, mode: Mode) -> (&'static str, String) {
    match (mode, &d.free) {
        (Mode::FreeSpace, Ok(g)) => {
            let place = d
                .part_before(g)
                .map_or("at the start".into(), |p| format!("after {}", p.name));
            (
                "Use the free space",
                format!("the {} {place}; the rest stays", human(g.bytes())),
            )
        }
        (Mode::FreeSpace, Err(why)) => ("Use the free space", why.clone()),
        (Mode::Erase, _) => ("Erase the whole drive", "everything on it is erased".into()),
    }
}

/// Draws Space.
pub fn draw(app: &App, c: &mut Canvas) {
    let look = app.look;
    let Some(d) = app.drive.selected() else {
        return;
    };
    c.text(
        TITLE_ROW,
        LEFT,
        &format!("How to use {}", describe(d)),
        look.title(),
    );
    let s = &app.space;
    // An option that cannot be picked comes first and keeps its place; the cursor's rows follow it
    let disabled = s.options.iter().take_while(|(_, ok)| !ok).count() as u16;
    for (i, (mode, _)) in s.options.iter().take(disabled as usize).enumerate() {
        let (label, why) = option(d, *mode);
        c.text(
            BODY_ROW + i as u16,
            ROW,
            &format!("{label:<w$}", w = OPTION as usize),
            look.help(),
        );
        c.text(BODY_ROW + i as u16, ROW + OPTION, &why, look.warning());
    }
    let enabled = s.enabled();
    s.cursor
        .draw(c, BODY_ROW + disabled, app.pressed, |c, row, i, sel| {
            let (label, what) = option(d, enabled[i]);
            c.text(
                row,
                ROW,
                &format!("{label:<w$}", w = OPTION as usize),
                if sel { look.selected() } else { look.body() },
            );
            c.text(row, ROW + OPTION, &what, look.body());
        });
    let r = BODY_ROW + s.options.len() as u16;
    c.text(r + 1, LEFT, "On it now:", look.body());
    let last = draw_parts(c, d, r + 2, true);
    let hint = "When the free space cannot be used, its line says why: under 40 GiB, no GPT partition table, a table that \
                needs repair, or BitLocker on the drive.";
    c.prose(last + HINT_GAP, hint, look.help());
}
