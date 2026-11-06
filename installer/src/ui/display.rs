//! Display, the first screen: text size, motion and contrast, each changing the screen as it is pressed. On the
//! console, text size is the console font, picked for the screen's resolution and set with `setfont`; in foot, which
//! opens at the same physical size on every monitor, it is foot's own Ctrl and + and −.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, act};
use super::list::Cursor;
use super::theme::{BODY_ROW, DISPLAY_LABEL, ROW, TITLE_ROW};
use super::{App, Env, Nav};
use crate::run::Cmd;

/// The framebuffer's size in pixels, `<width>,<height>`.
pub const FB_SIZE: &str = "/sys/class/graphics/fb0/virtual_size";

/// A console font of kbd's, and its glyph size in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Font {
    pub label: &'static str,
    pub file: &'static str,
    pub w: u32,
    pub h: u32,
}

/// The sizes, from the kernel's built-in 8 × 16 (also kbd's default8x16) to 24 × 32.
pub const FONTS: [Font; 4] = [
    Font {
        label: "Small",
        file: "default8x16",
        w: 8,
        h: 16,
    },
    Font {
        label: "Medium",
        file: "sun12x22",
        w: 12,
        h: 22,
    },
    Font {
        label: "Large",
        file: "latarcyrheb-sun32",
        w: 16,
        h: 32,
    },
    Font {
        label: "Extra large",
        file: "solar24x32",
        w: 24,
        h: 32,
    },
];

/// The fonts that leave at least 80 × 24 characters on a screen of `px` pixels.
pub fn offered(px: (u32, u32)) -> Vec<usize> {
    (0..FONTS.len())
        .filter(|&i| px.0 / FONTS[i].w >= 80 && px.1 / FONTS[i].h >= 24)
        .collect()
}

/// The first pick, by the screen's height (under 1000 pixels Small, under 1500 Medium, under 2200 Large, else Extra
/// large), lowered to the largest size that is offered.
pub fn pick(px: (u32, u32)) -> usize {
    let by_height = match px.1 {
        h if h < 1000 => 0,
        h if h < 1500 => 1,
        h if h < 2200 => 2,
        _ => 3,
    };
    offered(px)
        .into_iter()
        .filter(|&i| i <= by_height)
        .max()
        .unwrap_or(0)
}

/// The three settings' rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Row {
    TextSize,
    Motion,
    Contrast,
}

/// Display's state: the rows that can be selected, the settings, and the console's fonts.
#[derive(Clone, Debug)]
pub struct State {
    pub rows: Vec<Row>,
    pub cursor: Cursor,
    pub full_motion: bool,
    pub high: bool,
    pub screen_px: Option<(u32, u32)>,
    /// Offered fonts (indexes into FONTS) and the one in use (an index into `offered`).
    pub offered: Vec<usize>,
    pub font: usize,
}

impl State {
    /// On the console: Text size selectable, Reduced motion, the font picked for the framebuffer. In foot: Full
    /// motion, and Text size only names foot's keys.
    pub fn new(env: &Env) -> Self {
        let screen_px = env
            .run
            .read(std::path::Path::new(FB_SIZE))
            .ok()
            .and_then(|s| {
                let (w, h) = s.trim().split_once(',')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            });
        let offered = screen_px.map_or_else(|| vec![0], offered);
        let font = screen_px.map_or(0, |px| {
            offered.iter().position(|&i| i == pick(px)).unwrap_or(0)
        });
        let rows = if env.console {
            vec![Row::TextSize, Row::Motion, Row::Contrast]
        } else {
            vec![Row::Motion, Row::Contrast]
        };
        Self {
            cursor: Cursor::new(rows.len(), rows.len(), 0),
            rows,
            full_motion: !env.console,
            high: false,
            screen_px,
            offered,
            font,
        }
    }
}

/// Sets the console font in use (a no-op in foot).
pub fn apply_font(app: &mut App) {
    if app.env.console {
        let file = FONTS[app.display.offered[app.display.font]].file;
        // A font that cannot be set leaves the current one, which is readable enough to go on
        let _ = app.env.run.run(&Cmd::new("setfont").args([file]));
    }
}

/// Selects a row by its index among the selectable rows (a mouse click).
pub fn select(app: &mut App, i: usize) {
    app.display.cursor.select(i);
}

/// ←→ change the selected setting at once; ↑↓ choose; Enter goes on; Esc leaves.
pub fn key(app: &mut App, code: KeyCode) -> Nav {
    let d = &mut app.display;
    match (code, d.rows[d.cursor.sel]) {
        (KeyCode::Left | KeyCode::Right, Row::TextSize) => {
            let next = if code == KeyCode::Right {
                (d.font + 1).min(d.offered.len() - 1)
            } else {
                d.font.saturating_sub(1)
            };
            if next != d.font {
                d.font = next;
                apply_font(app);
            }
        }
        // Two values each, so either arrow switches to the other one
        (KeyCode::Left | KeyCode::Right, Row::Motion) => d.full_motion = !d.full_motion,
        (KeyCode::Left | KeyCode::Right, Row::Contrast) => {
            d.high = !d.high;
            app.look.high = d.high;
        }
        (KeyCode::Up, _) => d.cursor.step(-1),
        (KeyCode::Down, _) => d.cursor.step(1),
        (KeyCode::Enter, _) => return Nav::Next,
        (KeyCode::Esc, _) => return Nav::Quit(super::EXIT_LEFT),
        _ => {}
    }
    Nav::Stay
}

/// Display's bottom border.
pub fn actions(app: &App) -> Vec<Action> {
    let l = &app.look;
    vec![
        act(format!("{} choose", l.sym("↑↓")), KeyCode::Down),
        act(format!("{} change", l.sym("←→")), KeyCode::Right),
        act("Enter next", KeyCode::Enter),
        app.esc_quit(),
    ]
}

/// The value a row shows, between arrows.
fn value(app: &App, row: Row) -> String {
    let (l, r) = (app.look.sym("◀"), app.look.sym("▶"));
    let d = &app.display;
    let v = match row {
        Row::TextSize => {
            let f = FONTS[d.offered[d.font]];
            format!("{}  {} × {}", f.label, f.w, f.h)
        }
        Row::Motion => (if d.full_motion { "Full" } else { "Reduced" }).into(),
        Row::Contrast => (if d.high { "High" } else { "Standard" }).into(),
    };
    format!("{l} {v} {r}")
}

/// Draws Display.
pub fn draw(app: &App, c: &mut Canvas) {
    let look = app.look;
    c.text(
        TITLE_ROW,
        super::theme::LEFT,
        "Make this easy to read. Changes show at once.",
        look.title(),
    );
    let first = if app.env.console {
        BODY_ROW
    } else {
        c.text(BODY_ROW, ROW, "Text size", look.body());
        c.text(
            BODY_ROW,
            ROW + DISPLAY_LABEL,
            &format!("Ctrl and + bigger, Ctrl and {} smaller", look.sym("−")),
            look.body(),
        );
        BODY_ROW + 1
    };
    let d = &app.display;
    d.cursor.draw(c, first, app.pressed, |c, r, i, sel| {
        let row = d.rows[i];
        let label = match row {
            Row::TextSize => "Text size",
            Row::Motion => "Motion",
            Row::Contrast => "Contrast",
        };
        let v = value(app, row);
        let style = if sel { look.selected() } else { look.body() };
        c.text(
            r,
            ROW,
            &format!("{label:<w$}", w = DISPLAY_LABEL as usize),
            look.body(),
        );
        c.text(r, ROW + DISPLAY_LABEL, &v, style);
    });
    let hint = if app.env.console {
        match d.screen_px {
            Some((w, h)) => format!(
                "Text size was picked for this {w} × {h} screen. Sizes that leave less than 80 × 24 characters are not offered."
            ),
            None => {
                "The screen's size is unknown, so only the built-in text size is offered.".into()
            }
        }
    } else {
        "The text opened at the same physical size on every screen, from a 13-inch laptop to an ultrawide monitor.".into()
    };
    c.prose(BODY_ROW + 4, &hint, look.help());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_pick_goes_by_height() {
        let name = |px| FONTS[pick(px)].label;
        assert_eq!(name((3840, 2160)), "Large");
        assert_eq!(name((2560, 1600)), "Large");
        assert_eq!(name((1920, 1080)), "Medium");
        assert_eq!(name((1366, 768)), "Small");
        assert_eq!(name((5120, 2880)), "Extra large");
        // 1024 × 768: 16 × 32 and 24 × 32 would leave 64 × 24 and 42 × 24 characters
        assert_eq!(offered((1024, 768)), [0, 1]);
    }
}
