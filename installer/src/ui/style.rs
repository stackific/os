//! The styles: the accent from os-release's ANSI_COLOR, true colour or not, `NO_COLOR` (no-color.org), High
//! contrast, and the Linux console's symbols. A selection is never shown by colour alone: it is also reverse video.

use ratatui::style::{Color, Modifier, Style};

use super::theme;

/// How the screens look in this terminal, with these settings.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    /// Colour at all (`NO_COLOR` unset).
    pub color: bool,
    /// 24-bit colour, for tints and materials.
    pub truecolor: bool,
    /// The Linux console (`TERM=linux`): ASCII markers and arrow words.
    pub console: bool,
    /// High contrast: no dim text, no tints, bold and reverse video.
    pub high: bool,
    /// The accent, as os-release gives it.
    pub accent: (u8, u8, u8),
}

impl Look {
    /// The look for an environment: `TERM`, `COLORTERM` and `NO_COLOR`. foot reports `TERM=foot`, which run0 passes
    /// on while it drops `COLORTERM`, so foot counts as true colour too.
    pub fn from_env(
        term: &str,
        colorterm: &str,
        no_color: bool,
        accent: Option<(u8, u8, u8)>,
    ) -> Self {
        let console = term == "linux";
        Self {
            color: !no_color,
            truecolor: !no_color
                && !console
                && (matches!(colorterm, "truecolor" | "24bit") || term.starts_with("foot")),
            console,
            high: false,
            accent: accent.unwrap_or((0x00, 0x7a, 0x55)),
        }
    }

    /// Materials (tints, scrim, shadow, faded edges) only with true colour and Standard contrast.
    pub fn materials(&self) -> bool {
        self.truecolor && !self.high
    }

    fn accent_color(&self) -> Color {
        let (r, g, b) = self.accent;
        if self.truecolor {
            Color::Rgb(r, g, b)
        } else {
            Color::Green
        }
    }

    /// Titles: bold.
    pub fn title(&self) -> Style {
        Style::new().add_modifier(Modifier::BOLD)
    }

    /// Body text: the terminal's own.
    pub fn body(&self) -> Style {
        Style::new()
    }

    /// Help and hints: dim, except with High contrast.
    pub fn help(&self) -> Style {
        if self.high {
            Style::new()
        } else {
            Style::new().add_modifier(Modifier::DIM)
        }
    }

    /// The selected row: reverse video in the accent; High contrast drops the colour and adds bold.
    pub fn selected(&self) -> Style {
        let s = Style::new().add_modifier(Modifier::REVERSED);
        match (self.high, self.color) {
            (true, _) => s.add_modifier(Modifier::BOLD),
            (false, true) => s.fg(self.accent_color()),
            (false, false) => s,
        }
    }

    /// A row or action under a pressed mouse button: underlined until the release commits it.
    pub fn pressed(&self) -> Style {
        Style::new().add_modifier(Modifier::UNDERLINED)
    }

    /// The selection marker and done ticks: the accent.
    pub fn mark(&self) -> Style {
        if self.color && !self.high {
            Style::new().fg(self.accent_color())
        } else {
            Style::new().add_modifier(Modifier::BOLD)
        }
    }

    /// Warnings: the terminal's yellow.
    pub fn warning(&self) -> Style {
        if self.color {
            Style::new().fg(Color::Yellow)
        } else {
            Style::new().add_modifier(Modifier::BOLD)
        }
    }

    /// Errors: the terminal's red.
    pub fn error(&self) -> Style {
        if self.color {
            Style::new().fg(Color::Red)
        } else {
            Style::new().add_modifier(Modifier::BOLD)
        }
    }

    /// The frame's surface and its bottom band, as background colours, when materials are on.
    pub fn surface(&self, band: bool) -> Option<Color> {
        self.materials().then(|| {
            blend(
                theme::TERMINAL_BG,
                self.accent,
                if band {
                    theme::BAND_TINT
                } else {
                    theme::SURFACE_TINT
                },
            )
        })
    }

    /// The symbol to draw: on the console, whose fonts lack them, ASCII and words.
    pub fn sym(&self, s: &'static str) -> &'static str {
        if !self.console {
            return s;
        }
        match s {
            "▶" | "▸" => ">",
            "◀" => "<",
            "✓" | "•" => "*",
            "✗" => "x",
            "▏" => "_",
            "↑↓" => "Up/Down",
            "←→" => "Left/Right",
            "−" => "-",
            other => other,
        }
    }
}

/// `a` mixed with `b` at share `t`, as a 24-bit colour.
pub fn blend(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> Color {
    let mix = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * t).round() as u8;
    Color::Rgb(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}
