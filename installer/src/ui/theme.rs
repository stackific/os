//! Every spacing, colour and timing of the screens, each with why it has its value, so the screens stay consistent.

use super::motion::Tuning;

/// The frame is exactly 80 × 24, the smallest terminal everyone has: it reads the same on an ultrawide monitor, a
/// 13-inch laptop or a full-screen console, centered in whatever is around it.
pub const FRAME_W: u16 = 80;
pub const FRAME_H: u16 = 24;
/// Content starts at the same column on every screen: the border, then two blank columns.
pub const LEFT: u16 = 3;
/// The last content column: one blank column before the right border.
pub const RIGHT: u16 = 77;
/// Prose wraps here (LEFT to RIGHT), a comfortable line length that leaves the right margin blank.
pub const PROSE: usize = 75;
/// Rows inside the frame: 1 to 22; the borders take 0 and 23.
pub const FIRST_ROW: u16 = 1;
pub const LAST_ROW: u16 = 22;
/// The screen's title row and where its body starts (one blank row between groups).
pub const TITLE_ROW: u16 = 2;
pub const BODY_ROW: u16 = 4;
/// Rows a list shows at most, so its count and hint always fit below it.
pub const LIST_ROWS: usize = 10;
/// After a list or a table: one blank row, then the count; hints sit two blank rows below the last group.
pub const HINT_GAP: u16 = 3;

/// Columns of the rows. A marker takes two columns before the row's text.
pub const MARK: u16 = LEFT;
pub const ROW: u16 = LEFT + 2;
/// Display: label, then value.
pub const DISPLAY_LABEL: u16 = 14;
/// You: label, then the field.
pub const YOU_LABEL: u16 = 17;
/// Keyboard: the code column.
pub const CODE: u16 = 8;
/// Drive: path, size, free space, then the model.
pub const DRIVE_PATH: u16 = 15;
pub const DRIVE_SIZE: u16 = 11;
pub const DRIVE_FREE: u16 = 13;
/// Space and Confirm: the option label; partition name, size (right-aligned), file system, then label.
pub const OPTION: u16 = 23;
pub const PART_NAME: u16 = 14;
pub const PART_SIZE: usize = 9;
pub const PART_FS: u16 = 31;
pub const PART_LABEL: u16 = 39;
/// Confirm: where the typed drive name starts.
pub const TYPED: u16 = 30;
/// Installing: the step label's width, the progress bar's width, and the percentage's (right-aligned).
pub const STEP: u16 = 26;
pub const BAR: u16 = 29;
pub const PERCENT: usize = 4;
/// The stop dialog, in frame coordinates: centered across, level with the step list.
pub const DIALOG: (u16, u16, u16, u16) = (12, 4, 55, 6);
/// Its text wraps inside its own margins.
pub const DIALOG_PROSE: usize = 50;
/// It grows out of its anchor, the bottom border's "Ctrl+C stop", in this many frames.
pub const DIALOG_FRAMES: u8 = 3;

/// Frames per second while something moves; none while nothing does.
pub const FPS: f64 = 60.0;
/// Motion steps at most one cell per frame, for smoothness.
pub const MAX_SPEED: f64 = FPS;
/// Screen slides: critically damped (no bounce: keys carry no momentum), 0.3 s, over a short distance, so the
/// direction shows without the text racing past.
pub const SLIDE: Tuning = Tuning {
    damping: 1.0,
    response: 0.3,
};
pub const SLIDE_COLUMNS: f64 = 6.0;
/// The selection marker gliding between rows.
pub const GLIDE: Tuning = Tuning {
    damping: 1.0,
    response: 0.15,
};
/// The progress bar, which never moves backwards.
pub const PROGRESS: Tuning = Tuning {
    damping: 1.0,
    response: 0.4,
};
/// The rubber band past a list's edge: one row, then back.
pub const BAND: Tuning = Tuning {
    damping: 1.0,
    response: 0.2,
};
/// A held ↑ or ↓ speeds up after this long, to this many rows per repeat.
pub const HOLD_AFTER: f64 = 0.5;
pub const HOLD_ROWS: usize = 3;
/// Key repeats come closer together than this; a longer pause ends a hold.
pub const REPEAT_GAP: f64 = 0.15;
/// Wheel events this close together form a flick; it needs this many events to count as one.
pub const FLICK_GAP: f64 = 0.06;
pub const FLICK_EVENTS: usize = 3;
/// The guide's "fast" deceleration rate per millisecond, for flicks.
pub const DECELERATION: f64 = 0.99;
/// `--slow` runs every motion this many times slower, to review it frame by frame.
pub const SLOW: f64 = 10.0;

/// The terminal's background, assumed: foot's default (#242424), because a terminal does not report its colours
/// without a round trip. Tints are blended over it.
pub const TERMINAL_BG: (u8, u8, u8) = (0x24, 0x24, 0x24);
/// The frame's surface: the accent at 6 % over the terminal, a slight lift.
pub const SURFACE_TINT: f64 = 0.06;
/// The bottom border's band: heavier, at 14 %.
pub const BAND_TINT: f64 = 0.14;
/// The scrim behind the dialog dims what it covers to this share; the shadow darkens to this share.
pub const SCRIM: f64 = 0.45;
pub const SHADOW: f64 = 0.5;
