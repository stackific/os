//! Lists: the selection cursor every list shares (scrolling, the gliding marker, the rubber band past an edge), and
//! the type-to-search list that Keyboard and Timezone share, which looks and behaves the same on both.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Canvas, Hit};
use super::motion::Spring;
use super::theme::{self, BAND, GLIDE, LIST_ROWS, MARK, MAX_SPEED};

/// A selection over `len` items, showing `rows` of them from `top`.
#[derive(Clone, Debug)]
pub struct Cursor {
    pub sel: usize,
    pub top: usize,
    pub len: usize,
    pub rows: usize,
    /// The marker glides toward the selection (item units).
    pub marker: Spring,
    /// Rows the list is pushed past its edge (springs back to 0).
    pub band: Spring,
}

impl Cursor {
    /// A cursor on item `sel` of `len`, showing at most `rows`.
    pub fn new(len: usize, rows: usize, sel: usize) -> Self {
        let mut c = Self {
            sel: 0,
            top: 0,
            len,
            rows,
            marker: Spring::new(GLIDE, 0.0, MAX_SPEED),
            band: Spring::new(BAND, 0.0, MAX_SPEED),
        };
        c.reset(len, sel);
        c
    }

    /// New contents: select `sel` at once, no glide.
    pub fn reset(&mut self, len: usize, sel: usize) {
        self.len = len;
        self.sel = sel.min(len.saturating_sub(1));
        self.top = 0;
        self.scroll();
        self.marker.snap(self.sel as f64);
    }

    /// Rows shown now.
    pub fn shown(&self) -> usize {
        self.len.min(self.rows)
    }

    fn scroll(&mut self) {
        if self.sel < self.top {
            self.top = self.sel;
        }
        if self.sel >= self.top + self.rows {
            self.top = self.sel + 1 - self.rows;
        }
        self.top = self.top.min(self.len.saturating_sub(self.rows));
    }

    /// Selects item `i`; the marker glides there.
    pub fn select(&mut self, i: usize) {
        if self.len == 0 {
            return;
        }
        self.sel = i.min(self.len - 1);
        self.scroll();
        self.marker.retarget(self.sel as f64);
    }

    /// Moves by `delta` items; at an edge, nudges the list one row past it instead (the rubber band).
    pub fn step(&mut self, delta: isize) {
        if self.len == 0 {
            return;
        }
        let at_edge = (delta < 0 && self.sel == 0) || (delta > 0 && self.sel + 1 == self.len);
        if at_edge {
            self.band.pos = if delta < 0 { 1.0 } else { -1.0 };
            self.band.retarget(0.0);
            return;
        }
        self.select(self.sel.saturating_add_signed(delta));
    }

    /// The keys every list shares: arrows (`rows` at a time while held), Page Up and Down, Home and End.
    pub fn key(&mut self, code: KeyCode, rows: usize) -> bool {
        let page = self.rows.max(1) as isize;
        match code {
            KeyCode::Up => self.step(-(rows as isize)),
            KeyCode::Down => self.step(rows as isize),
            KeyCode::PageUp => self.step(-page),
            KeyCode::PageDown => self.step(page),
            KeyCode::Home => self.select(0),
            KeyCode::End => self.select(self.len.saturating_sub(1)),
            _ => return false,
        }
        true
    }

    /// Advances the marker and the band.
    pub fn tick(&mut self, dt: f64, instant: bool) {
        self.marker.step(dt, instant);
        self.band.step(dt, instant);
    }

    /// Whether the marker or the band still moves.
    pub fn moving(&self) -> bool {
        !self.marker.settled() || !self.band.settled()
    }

    /// Draws `shown()` rows from `row`: `line(canvas, row, item, selected)` draws one item; the cursor adds the marker,
    /// the faded edges of a scrolled list and the pressable rows.
    pub fn draw(
        &self,
        c: &mut Canvas,
        row: u16,
        pressed: Option<Hit>,
        mut line: impl FnMut(&mut Canvas, u16, usize, bool),
    ) {
        let band = self.band.cell();
        let shown = self.shown();
        for v in 0..shown {
            let item = self.top + v;
            let at = v as i32 + band;
            if at < 0 || at >= shown as i32 {
                continue;
            }
            let r = row + at as u16;
            line(c, r, item, item == self.sel);
            if pressed == Some(Hit::Row(item)) {
                c.restyle(r, theme::ROW, theme::RIGHT - theme::ROW, c.look.pressed());
            }
            let edge = (v == 0 && self.top > 0) || (v + 1 == shown && self.top + shown < self.len);
            if edge && c.look.materials() && item != self.sel {
                c.restyle(
                    r,
                    theme::ROW,
                    theme::RIGHT - theme::ROW,
                    ratatui::style::Style::new().add_modifier(ratatui::style::Modifier::DIM),
                );
            }
            c.hit(r, theme::ROW, theme::RIGHT - theme::ROW, Hit::Row(item));
        }
        if shown > 0 {
            let m = (self.marker.cell() - self.top as i32 + band).clamp(0, shown as i32 - 1);
            let style = c.look.mark();
            c.text(row + m as u16, MARK, c.look.sym("▶"), style);
        }
    }
}

/// A list filtered by what is typed: items are (code, name), such as ("de", "German") or ("Europe/Berlin", "").
#[derive(Clone, Debug)]
pub struct SearchList {
    pub items: Vec<(String, String)>,
    /// The item an empty search starts with.
    pub default: String,
    pub query: String,
    /// Indexes into `items`, in the order shown.
    pub matches: Vec<usize>,
    pub cursor: Cursor,
}

impl SearchList {
    /// A list over `items`, starting with `default` when the search is empty.
    pub fn new(items: Vec<(String, String)>, default: &str) -> Self {
        let mut l = Self {
            items,
            default: default.into(),
            query: String::new(),
            matches: vec![],
            cursor: Cursor::new(0, LIST_ROWS, 0),
        };
        l.refilter();
        l
    }

    /// Filters by code and name, ignoring case: an exact code first, then shorter names, then the list's own order.
    /// An empty search lists everything, the default first.
    pub fn refilter(&mut self) {
        let q = self.query.to_lowercase();
        let name = |i: usize| {
            if self.items[i].1.is_empty() {
                &self.items[i].0
            } else {
                &self.items[i].1
            }
        };
        self.matches = if q.is_empty() {
            let d = self.items.iter().position(|(c, _)| *c == self.default);
            d.into_iter()
                .chain((0..self.items.len()).filter(|i| Some(*i) != d))
                .collect()
        } else {
            let mut m: Vec<usize> = (0..self.items.len())
                .filter(|&i| {
                    self.items[i].0.to_lowercase().contains(&q)
                        || self.items[i].1.to_lowercase().contains(&q)
                })
                .collect();
            m.sort_by_key(|&i| {
                (
                    self.items[i].0.to_lowercase() != q,
                    name(i).chars().count(),
                    i,
                )
            });
            m
        };
        self.cursor.reset(self.matches.len(), 0);
    }

    /// The selected item's code.
    pub fn selected(&self) -> Option<&(String, String)> {
        self.matches.get(self.cursor.sel).map(|&i| &self.items[i])
    }

    /// Typing and its keys: a character re-filters at once, Backspace removes one, the list keys move.
    pub fn key(&mut self, code: KeyCode, rows: usize) -> bool {
        match code {
            KeyCode::Char(ch) => {
                self.query.push(ch);
                self.refilter();
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.refilter();
            }
            other => return self.cursor.key(other, rows),
        }
        true
    }

    /// "3 matches", "1 match", "No match".
    pub fn count(&self) -> String {
        match self.matches.len() {
            0 => "No match. Backspace widens the search.".into(),
            1 => "1 match".into(),
            n => format!("{n} matches"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layouts() -> SearchList {
        SearchList::new(
            crate::answers::parse_layouts(include_str!("../../fixtures/evdev.lst")),
            "us",
        )
    }

    #[test]
    fn search_order() {
        let mut l = layouts();
        assert_eq!(l.selected().unwrap().0, "us");
        for ch in "germ".chars() {
            l.key(KeyCode::Char(ch), 1);
            assert!(
                l.matches
                    .iter()
                    .all(|&i| l.items[i].1.to_lowercase().contains(&l.query)
                        || l.items[i].0.contains(&l.query))
            );
        }
        let codes: Vec<&str> = l.matches.iter().map(|&i| l.items[i].0.as_str()).collect();
        assert_eq!(codes, ["de", "at", "ch"]);
        // An exact code comes first, though other names are shorter
        let mut l = layouts();
        l.key(KeyCode::Char('i'), 1);
        l.key(KeyCode::Char('n'), 1);
        assert_eq!(l.selected().unwrap().0, "in");
        // Each character re-filters, and Backspace widens again
        let mut l = layouts();
        l.key(KeyCode::Char('g'), 1);
        let after_g = l.matches.len();
        l.key(KeyCode::Char('e'), 1);
        assert!(l.matches.len() < after_g);
        l.key(KeyCode::Backspace, 1);
        assert_eq!(l.matches.len(), after_g);
        let zones = SearchList::new(
            vec![
                ("Africa/Abidjan".into(), String::new()),
                ("UTC".into(), String::new()),
            ],
            "UTC",
        );
        assert_eq!(zones.selected().unwrap().0, "UTC");
    }

    #[test]
    fn the_band_and_scrolling() {
        let mut c = Cursor::new(30, 10, 0);
        c.step(-1);
        assert_eq!((c.sel, c.band.pos), (0, 1.0));
        c.tick(0.5, false);
        assert!(!c.moving());
        c.select(29);
        assert_eq!((c.top, c.sel), (20, 29));
        c.step(1);
        assert_eq!(c.band.pos, -1.0);
        c.step(-3);
        assert_eq!(c.sel, 26);
    }
}
