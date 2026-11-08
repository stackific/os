//! Drive: whole disks only, with their size and largest free space, and every drive left out with its reason. The
//! list is read again each time the screen shows and when the terminal regains focus, so changes made in Disks show.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::list::Cursor;
use super::theme::{
    BODY_ROW, DRIVE_FREE, DRIVE_PATH, DRIVE_SIZE, HINT_GAP, LEFT, LIST_ROWS, ROW, TITLE_ROW,
};
use super::{App, Nav};
use crate::drives::{self, Drive, Excluded, human};

/// The drives read last, and the selection.
#[derive(Clone, Debug)]
pub struct State {
    pub listed: Vec<Drive>,
    pub excluded: Vec<Excluded>,
    pub error: Option<String>,
    pub cursor: Cursor,
    /// Intel RST hides NVMe drives from Linux (IN-07): the guidance line shows, as for an RST member.
    pub rst: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            listed: vec![],
            excluded: vec![],
            error: None,
            cursor: Cursor::new(0, LIST_ROWS, 0),
            rst: false,
        }
    }
}

impl State {
    /// The selected drive.
    pub fn selected(&self) -> Option<&Drive> {
        self.listed.get(self.cursor.sel)
    }
}

/// Reads the drives again, keeping the selection on the same drive when it is still listed.
pub fn rescan(app: &mut App) {
    let keep = app.drive.selected().map(|d| d.path.clone());
    match drives::scan(&*app.env.run, app.env.base.live_medium()) {
        Ok((listed, excluded)) => {
            let sel = keep
                .and_then(|p| listed.iter().position(|d| d.path == p))
                .unwrap_or(0);
            let cursor = Cursor::new(listed.len(), LIST_ROWS, sel);
            app.drive = State {
                listed,
                excluded,
                error: None,
                cursor,
                rst: drives::remapped_nvme(&*app.env.run),
            };
        }
        Err(e) => {
            app.drive = State {
                error: Some(e),
                ..State::default()
            }
        }
    }
}

/// The list keys choose; Enter goes on when a drive is listed; Esc goes back.
pub fn key(app: &mut App, code: KeyCode, rows: usize) -> Nav {
    match code {
        KeyCode::Enter if app.drive.selected().is_some() => Nav::Next,
        KeyCode::Esc => Nav::Back,
        other => {
            app.drive.cursor.key(other, rows);
            Nav::Stay
        }
    }
}

/// Drive's bottom border.
pub fn actions(app: &App) -> Vec<Action> {
    let mut a = vec![act(format!("{} choose", app.look.sym("↑↓")), KeyCode::Down)];
    if app.drive.selected().is_some() {
        a.push(act("Enter next", KeyCode::Enter));
    }
    a.push(act("Esc back", KeyCode::Esc));
    a
}

/// What the free-space column says about a drive.
pub fn free_column(d: &Drive) -> String {
    match (&d.table, &d.gap) {
        (None, _) => "no table".into(),
        (Some(t), _) if t.label != "gpt" => "MBR table".into(),
        (Some(_), Some(g)) => human(g.bytes()),
        (Some(_), None) => "none".into(),
    }
}

/// Draws Drive.
pub fn draw(app: &App, c: &mut Canvas) {
    let (look, s) = (app.look, &app.drive);
    c.text(TITLE_ROW, LEFT, "Drive to install on", look.title());
    let (size_col, free_col) = (ROW + DRIVE_PATH, ROW + DRIVE_PATH + DRIVE_SIZE);
    let mut last = BODY_ROW;
    if let Some(e) = &s.error {
        c.text(
            BODY_ROW,
            LEFT,
            &format!("The drives could not be read: {e}"),
            look.error(),
        );
    } else if s.listed.is_empty() {
        c.text(
            BODY_ROW,
            LEFT,
            &format!("No drive can take {}.", app.env.os.name),
            look.warning(),
        );
    } else {
        c.text(BODY_ROW, size_col, "Size", look.help());
        c.text(BODY_ROW, free_col, "Largest free space", look.help());
        s.cursor.draw(c, BODY_ROW + 1, app.pressed, |c, r, i, sel| {
            let d = &s.listed[i];
            let w = |n: u16| n as usize;
            let line = format!(
                "{:<pw$}{:<sw$}{:<fw$}{}",
                d.path,
                human(d.size),
                free_column(d),
                d.model,
                pw = w(DRIVE_PATH),
                sw = w(DRIVE_SIZE),
                fw = w(DRIVE_FREE)
            );
            c.text(
                r,
                ROW,
                &line,
                if sel { look.selected() } else { look.body() },
            );
        });
        last = BODY_ROW + s.cursor.shown() as u16;
    }
    if !s.excluded.is_empty() {
        let head = last + 2;
        c.text(head, LEFT, "Not listed:", look.body());
        for (i, x) in s.excluded.iter().enumerate() {
            c.text(
                head + 1 + i as u16,
                ROW,
                &format!("{:<w$}{}", x.path, x.reason, w = DRIVE_PATH as usize),
                look.help(),
            );
        }
        last = head + s.excluded.len() as u16;
    }
    if s.rst || s.excluded.iter().any(|x| x.reason == drives::RST_MEMBER) {
        let rst = "Intel RST may hide drives here. To use them, set the firmware's storage mode to AHCI, and prepare \
                   Windows first, or it may stop starting.";
        last += 1 + c.prose(last + 2, rst, look.warning());
    }
    if !app.env.console {
        let hint = "To change partitions first, open Disks from the app launcher. This list updates when you come back.";
        c.prose(last + HINT_GAP, hint, look.help());
    }
}
