//! The stop dialog: it grows out of the bottom border's "Ctrl+C stop" in three frames, over a dimming scrim with a
//! one-cell shadow, and shrinks back into it. Enter stops the install; Esc keeps it going.

use std::sync::atomic::Ordering;

use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::{Modifier, Style};

use super::App;
use super::frame::{Canvas, Hit};
use super::style::blend;
use super::theme::{
    DIALOG, DIALOG_FRAMES, DIALOG_PROSE, FRAME_H, FRAME_W, SCRIM, SHADOW, TERMINAL_BG,
};
use crate::answers::Mode;

/// Whether the dialog is opening or open, and how many of its frames have grown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub open: bool,
    pub frames: u8,
}

impl State {
    /// Opening: from its anchor with Full motion, at once with Reduced.
    pub fn opening(full_motion: bool) -> Self {
        Self {
            open: true,
            frames: if full_motion { 0 } else { DIALOG_FRAMES },
        }
    }

    /// Whether it still grows or shrinks.
    pub fn moving(&self) -> bool {
        (self.open && self.frames < DIALOG_FRAMES) || (!self.open && self.frames > 0)
    }
}

/// One frame of growth or shrinking; a shrunk dialog is gone.
pub fn tick(app: &mut App) {
    let full = app.full_motion();
    let Some(d) = app.dialog.as_mut() else { return };
    if d.open {
        d.frames = (d.frames + 1).min(DIALOG_FRAMES);
        return;
    }
    d.frames = if full { d.frames.saturating_sub(1) } else { 0 };
    if d.frames == 0 {
        app.dialog = None;
    }
}

/// Enter stops the install; Esc closes the dialog and the install goes on.
pub fn key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            if let Some(i) = &app.install {
                i.stop.store(true, Ordering::Relaxed);
            }
            app.dialog = None;
        }
        KeyCode::Esc => {
            if let Some(d) = app.dialog.as_mut() {
                d.open = false;
            }
        }
        _ => {}
    }
}

/// The dialog's rectangle (frame coordinates) after `frames` of growth, from the anchor to its full size.
pub fn rect(frames: u8) -> (u16, u16, u16, u16) {
    let anchor = (2.0, (FRAME_H - 1) as f64, 11.0, 1.0);
    let t = frames as f64 / DIALOG_FRAMES as f64;
    let lerp = |a: f64, b: u16| (a + (b as f64 - a) * t).round() as u16;
    (
        lerp(anchor.0, DIALOG.0),
        lerp(anchor.1, DIALOG.1),
        lerp(anchor.2, DIALOG.2).max(2),
        lerp(anchor.3, DIALOG.3).max(2),
    )
}

/// Draws the scrim, the shadow and the dialog at its current size; its text once it is full grown.
pub fn draw(app: &App, c: &mut Canvas) {
    let Some(d) = app.dialog else { return };
    let look = c.look;
    let (x, y, w, h) = rect(d.frames);
    let inside = |cx: u16, cy: u16| cx >= x && cx < x + w && cy >= y && cy < y + h;
    if look.materials() {
        let scrim = blend(TERMINAL_BG, (0, 0, 0), SCRIM);
        let shadow = blend(TERMINAL_BG, (0, 0, 0), SHADOW);
        for cy in 0..FRAME_H {
            for cx in 0..FRAME_W {
                if inside(cx, cy) {
                    continue;
                }
                let cell = &mut c.buf[(c.frame.x + cx, c.frame.y + cy)];
                let in_shadow = (cx == x + w && cy > y && cy <= y + h)
                    || (cy == y + h && cx > x && cx <= x + w);
                cell.set_style(
                    Style::new()
                        .bg(if in_shadow { shadow } else { scrim })
                        .add_modifier(Modifier::DIM),
                );
            }
        }
    }
    let line = look.help();
    for cy in y..y + h {
        for cx in x..x + w {
            let ch = match (cx == x, cx == x + w - 1, cy == y, cy == y + h - 1) {
                (true, _, true, _) => "┌",
                (_, true, true, _) => "┐",
                (true, _, _, true) => "└",
                (_, true, _, true) => "┘",
                (_, _, true, _) | (_, _, _, true) => "─",
                (true, _, _, _) | (_, true, _, _) => "│",
                _ => " ",
            };
            let style = if ch == " " { look.body() } else { line };
            let cell = &mut c.buf[(c.frame.x + cx, c.frame.y + cy)];
            cell.reset();
            if let Some(bg) = look.surface(false) {
                cell.set_bg(bg);
            }
            c.border(cy, cx, ch, style);
        }
    }
    if d.frames < DIALOG_FRAMES || !d.open {
        return;
    }
    c.border(y, x + 1, " ", line);
    c.border(y + h - 1, x + 1, " ", line);
    let after = c.border(y, x + 2, "Stop the install?", look.title());
    c.border(y, after, " ", line);
    let device = app.install.as_ref().map_or("", |i| i.device.as_str());
    let text = match app.install.as_ref().map(|i| i.mode) {
        Some(Mode::Erase) => {
            format!("{device} stays erased and partly written. No other drive changes.")
        }
        _ => format!(
            "The new partitions stay on {device}, partly written. Nothing else on the drive changes."
        ),
    };
    for (i, l) in super::frame::wrap(&text, DIALOG_PROSE).iter().enumerate() {
        c.border(y + 2 + i as u16, x + 3, l, look.body());
    }
    let mut cx = c.border(y + h - 1, x + 2, "Enter stop", look.body());
    c.hit(
        y + h - 1,
        x + 2,
        10,
        Hit::Key(KeyCode::Enter, KeyModifiers::NONE),
    );
    cx = c.border(y + h - 1, cx, " · ", line);
    let start = cx;
    cx = c.border(y + h - 1, cx, "Esc keep installing", look.body());
    c.hit(
        y + h - 1,
        start,
        cx - start,
        Hit::Key(KeyCode::Esc, KeyModifiers::NONE),
    );
    c.border(y + h - 1, cx, " ", line);
}
