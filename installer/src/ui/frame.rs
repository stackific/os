//! The centered 80 × 24 frame: one surface with the program's name and the step in its top border, the keys that
//! work in its bottom border, and the canvas screens draw on (frame coordinates, shifted by the slide, clipped to the
//! inside). A terminal smaller than the frame gets one short message instead.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::style::Look;
use super::theme::{FIRST_ROW, FRAME_H, FRAME_W, LAST_ROW, LEFT, PROSE};

/// Something the mouse can press: a list row (its item index), a field, or a bottom-border action (its key).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hit {
    Row(usize),
    Field(usize),
    Key(KeyCode, KeyModifiers),
}

/// A bottom-border action: what it does, and the key that does it.
#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub label: String,
    pub code: KeyCode,
    pub mods: KeyModifiers,
}

/// An action for a key without modifiers.
pub fn act(label: impl Into<String>, code: KeyCode) -> Action {
    Action {
        label: label.into(),
        code,
        mods: KeyModifiers::NONE,
    }
}

/// The frame's place in `area`: centered, or None when the terminal is smaller than the frame.
pub fn place(area: Rect) -> Option<Rect> {
    (area.width >= FRAME_W && area.height >= FRAME_H).then(|| {
        Rect::new(
            area.x + (area.width - FRAME_W) / 2,
            area.y + (area.height - FRAME_H) / 2,
            FRAME_W,
            FRAME_H,
        )
    })
}

/// Greedy word wrap at `width`, keeping a one-letter word ("a") off a line's end.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<Vec<&str>> = vec![vec![]];
    for word in text.split_whitespace() {
        let line = lines.last_mut().expect("one line");
        let len: usize =
            line.iter().map(|w| w.chars().count() + 1).sum::<usize>() + word.chars().count();
        if !line.is_empty() && len > width {
            let carry = if line.last().is_some_and(|w| w.chars().count() == 1) {
                line.pop()
            } else {
                None
            };
            lines.push(carry.into_iter().chain([word]).collect());
        } else {
            line.push(word);
        }
    }
    lines
        .into_iter()
        .map(|l| l.join(" "))
        .filter(|l| !l.is_empty())
        .collect()
}

/// Where screens draw: frame coordinates (column 0 is the left border), content shifted by `dx` and clipped to the
/// frame's inside. It also collects what the mouse can press.
pub struct Canvas<'a> {
    pub buf: &'a mut Buffer,
    pub frame: Rect,
    pub dx: i32,
    pub look: Look,
    pub hits: Vec<(Rect, Hit)>,
}

impl Canvas<'_> {
    /// Draws content text at a frame row and column; returns the column after it. Every glyph used is one cell wide.
    pub fn text(&mut self, row: u16, col: u16, s: &str, style: Style) -> u16 {
        let mut c = col as i32 + self.dx;
        if (FIRST_ROW..=LAST_ROW).contains(&row) {
            for ch in s.chars() {
                if (1..FRAME_W as i32 - 1).contains(&c) {
                    let cell = &mut self.buf[(self.frame.x + c as u16, self.frame.y + row)];
                    cell.set_char(ch);
                    cell.set_style(style);
                }
                c += 1;
            }
        }
        col + s.chars().count() as u16
    }

    /// Draws on the borders: not shifted, not clipped to the inside.
    pub fn border(&mut self, row: u16, col: u16, s: &str, style: Style) -> u16 {
        for (i, ch) in s.chars().enumerate() {
            let x = col + i as u16;
            if x < FRAME_W {
                let cell = &mut self.buf[(self.frame.x + x, self.frame.y + row)];
                cell.set_char(ch);
                cell.set_style(style);
            }
        }
        col + s.chars().count() as u16
    }

    /// Adds `style` to `width` content cells from a row and column (shifted like text).
    pub fn restyle(&mut self, row: u16, col: u16, width: u16, style: Style) {
        for i in 0..width {
            let c = (col + i) as i32 + self.dx;
            if (FIRST_ROW..=LAST_ROW).contains(&row) && (1..FRAME_W as i32 - 1).contains(&c) {
                let cell = &mut self.buf[(self.frame.x + c as u16, self.frame.y + row)];
                cell.set_style(style);
            }
        }
    }

    /// Prose from LEFT, wrapped; returns the number of rows it took.
    pub fn prose(&mut self, row: u16, text: &str, style: Style) -> u16 {
        let lines = wrap(text, PROSE);
        for (i, l) in lines.iter().enumerate() {
            self.text(row + i as u16, LEFT, l, style);
        }
        lines.len() as u16
    }

    /// Records a pressable area on a row (frame coordinates, unshifted: the mouse acts on where things settle).
    pub fn hit(&mut self, row: u16, col: u16, width: u16, hit: Hit) {
        self.hits.push((
            Rect::new(self.frame.x + col, self.frame.y + row, width, 1),
            hit,
        ));
    }
}

/// Draws the frame: the surface, the top border with the title and the step, and the bottom border with the actions.
/// `pressed` is the action under a pressed mouse button.
pub fn draw(c: &mut Canvas, title: &str, step: &str, actions: &[Action], pressed: Option<Hit>) {
    let look = c.look;
    if let Some(bg) = look.surface(false) {
        for y in 0..FRAME_H {
            let band = look.surface(y == FRAME_H - 1).unwrap_or(bg);
            for x in 0..FRAME_W {
                c.buf[(c.frame.x + x, c.frame.y + y)].set_bg(band);
            }
        }
    }
    let line = look.help();
    let fill = |from: u16, to: u16| "─".repeat(to.saturating_sub(from) as usize);
    // Top: ┌ <title> ─── <step> ┐
    let mut x = c.border(0, 0, "┌ ", line);
    x = c.border(0, x, title, look.title());
    x = c.border(0, x, " ", line);
    let right = FRAME_W - 2 - step.chars().count() as u16 - 1;
    c.border(0, x, &fill(x, right), line);
    c.border(0, right, " ", line);
    c.border(0, right + 1, step, look.body());
    c.border(0, FRAME_W - 2, " ┐", line);
    for y in 1..FRAME_H - 1 {
        c.border(y, 0, "│", line);
        c.border(y, FRAME_W - 1, "│", line);
    }
    // Bottom: └ <action> · <action> ───┘
    let y = FRAME_H - 1;
    let mut x = c.border(y, 0, "└ ", line);
    for (i, a) in actions.iter().enumerate() {
        if i > 0 {
            x = c.border(y, x, " · ", line);
        }
        let hit = Hit::Key(a.code, a.mods);
        let style = if pressed == Some(hit) {
            look.body().patch(look.pressed())
        } else {
            look.body()
        };
        let start = x;
        x = c.border(y, x, &a.label, style);
        c.hit(y, start, x - start, hit);
    }
    x = c.border(y, x, " ", line);
    c.border(y, x, &fill(x, FRAME_W - 1), line);
    c.border(y, FRAME_W - 1, "┘", line);
}

/// The message for a terminal smaller than the frame, top left.
pub fn too_small(buf: &mut Buffer, area: Rect, title: &str, look: &Look) {
    let lines = [
        (title.to_string(), look.title()),
        (String::new(), look.body()),
        (
            "Make the window bigger or the text smaller".into(),
            look.body(),
        ),
        (
            format!("(Ctrl and {}): the installer needs 80 × 24.", look.sym("−")),
            look.body(),
        ),
        (
            format!("It is {} × {} now.", area.width, area.height),
            look.body(),
        ),
        (String::new(), look.body()),
        ("Ctrl+C quits.".into(), look.help()),
    ];
    for (i, (text, style)) in lines.iter().enumerate() {
        if (i as u16) < area.height {
            buf.set_stringn(
                area.x + 1,
                area.y + i as u16,
                text,
                area.width.saturating_sub(1) as usize,
                *style,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_keeps_one_letter_words_off_line_ends() {
        let w = wrap(
            "It goes into a drive's free space, next to what is already there, or onto a whole drive, which it erases first.",
            75,
        );
        assert_eq!(
            w,
            [
                "It goes into a drive's free space, next to what is already there, or onto",
                "a whole drive, which it erases first."
            ]
        );
        let w = wrap(
            "You pick a keyboard layout, a timezone, your user, the drive and how to use it. Nothing is written until you confirm on the last screen.",
            75,
        );
        assert_eq!(
            w[0],
            "You pick a keyboard layout, a timezone, your user, the drive and how to use"
        );
    }

    #[test]
    fn the_frame_centers() {
        assert_eq!(
            place(Rect::new(0, 0, 200, 60)),
            Some(Rect::new(60, 18, 80, 24))
        );
        assert_eq!(
            place(Rect::new(0, 0, 80, 24)),
            Some(Rect::new(0, 0, 80, 24))
        );
        assert_eq!(place(Rect::new(0, 0, 79, 24)), None);
    }
}
