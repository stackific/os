//! The text interface: the App holds every screen's state, turns key, mouse, focus and resize events into changes
//! at once (no debounce, no wait), advances the springs only while something moves, and renders the centered frame.

pub mod confirm;
pub mod dialog;
pub mod display;
pub mod done;
pub mod drive;
pub mod frame;
pub mod installing;
pub mod keyboard;
pub mod list;
pub mod motion;
pub mod space;
pub mod stopped;
pub mod style;
#[cfg(test)]
mod tests;
pub mod theme;
pub mod timezone;
pub mod welcome;
pub mod you;

use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::layout::Rect;

use crate::answers::{self, Answers, Catalog};
use crate::base::Base;
use crate::install::{Failure, Log};
use crate::os_release::OsRelease;
use crate::run::Runner;
use frame::{Action, Canvas, Hit};
use motion::Spring;
use style::Look;

/// What the screens run against: the runner, the system's names and base, the catalog, and how they were started.
pub struct Env {
    pub run: Arc<dyn Runner>,
    pub os: OsRelease,
    pub base: Arc<dyn Base>,
    pub catalog: Catalog,
    pub log: Log,
    /// On the Linux console, from the boot menu's Install entry.
    pub console: bool,
}

/// The screens, in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Display,
    Welcome,
    Keyboard,
    Timezone,
    You,
    Drive,
    Space,
    Confirm,
    Installing,
    Done,
    Stopped,
}

impl Screen {
    /// The position along the slide track.
    fn place(self) -> f64 {
        match self {
            Screen::Done | Screen::Stopped => 9.0,
            s => s as u8 as f64,
        }
    }

    /// The top border's right side: the screen's name, with "n of 6" for the six steps.
    pub fn label(self) -> String {
        let (n, name) = match self {
            Screen::Display => (0, "Display"),
            Screen::Welcome => (0, "Welcome"),
            Screen::Keyboard => (1, "Keyboard"),
            Screen::Timezone => (2, "Timezone"),
            Screen::You => (3, "You"),
            Screen::Drive => (4, "Drive"),
            Screen::Space => (5, "Space"),
            Screen::Confirm => (6, "Confirm"),
            Screen::Installing => (0, "Installing"),
            Screen::Done => (0, "Done"),
            Screen::Stopped => (0, "Stopped"),
        };
        if n == 0 {
            name.into()
        } else {
            format!("{n} of 6 · {name}")
        }
    }
}

/// Where a key sends the user.
#[derive(Debug, PartialEq)]
pub enum Nav {
    Stay,
    Next,
    Back,
    Quit(i32),
    Install,
}

/// Exit codes (`stackific-installer --help`).
pub const EXIT_LEFT: i32 = 3;
pub const EXIT_FAILED: i32 = 1;
pub const EXIT_STOPPED: i32 = 130;

/// The interface's whole state.
pub struct App {
    pub env: Env,
    pub look: Look,
    pub screen: Screen,
    pub display: display::State,
    pub keyboard: list::SearchList,
    pub timezone: list::SearchList,
    pub you: you::State,
    pub drive: drive::State,
    pub space: space::State,
    pub confirm: confirm::State,
    pub install: Option<installing::State>,
    pub dialog: Option<dialog::State>,
    pub outcome: Option<Result<(), Failure>>,
    /// The slide track's camera: a screen is drawn at its place minus this.
    pub camera: Spring,
    pub pressed: Option<Hit>,
    pub hits: Vec<(Rect, Hit)>,
    /// Seconds since start, advanced by `tick`.
    pub clock: f64,
    pub exit: Option<i32>,
    /// Ring the terminal bell after the next frame (once, on Stopped).
    pub bell: bool,
    held: Option<(KeyCode, f64, f64)>,
    wheel: Vec<(f64, isize)>,
}

impl App {
    /// The App on its first screen, Display. `look` comes from the environment (TERM, COLORTERM, NO_COLOR).
    pub fn new(env: Env, look: Look) -> Self {
        let keyboard = list::SearchList::new(env.catalog.layouts.clone(), answers::DEFAULT_LAYOUT);
        let zones = env
            .catalog
            .zones
            .iter()
            .map(|z| (z.clone(), String::new()))
            .collect();
        let timezone = list::SearchList::new(zones, answers::DEFAULT_ZONE);
        let display = display::State::new(&env);
        let you = you::State::new(&env.os.default_hostname);
        let mut app = Self {
            look: Look {
                high: display.high,
                ..look
            },
            env,
            screen: Screen::Display,
            display,
            keyboard,
            timezone,
            you,
            drive: drive::State::default(),
            space: space::State::default(),
            confirm: confirm::State::default(),
            install: None,
            dialog: None,
            outcome: None,
            camera: Spring::new(theme::SLIDE, 0.0, theme::MAX_SPEED),
            pressed: None,
            hits: vec![],
            clock: 0.0,
            exit: None,
            bell: false,
            held: None,
            wheel: vec![],
        };
        display::apply_font(&mut app);
        app
    }

    /// Whether motion is Full (springs) rather than Reduced (instant).
    pub fn full_motion(&self) -> bool {
        self.display.full_motion
    }

    /// Goes to `screen`; the camera glides there (or jumps, with Reduced motion).
    pub fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.pressed = None;
        self.camera.retarget(screen.place() * theme::SLIDE_COLUMNS);
        if !self.full_motion() {
            self.camera.snap(self.camera.target);
        }
        match screen {
            Screen::Drive => drive::rescan(self),
            Screen::Space => space::enter(self),
            Screen::Confirm => self.confirm = confirm::State::default(),
            Screen::Stopped => self.bell = true,
            _ => {}
        }
    }

    /// The answers the screens hold, for the install.
    pub fn answers(&self) -> Answers {
        let mut a = Answers::default();
        if let Some(d) = self.drive.selected() {
            a.disk.device = d.path.clone();
            a.disk.erase = d.name.clone();
        }
        a.disk.mode = self.space.mode();
        a.system.keyboard = self
            .keyboard
            .selected()
            .map(|s| s.0.clone())
            .unwrap_or_default();
        a.system.timezone = self
            .timezone
            .selected()
            .map(|s| s.0.clone())
            .unwrap_or_default();
        a.system.hostname = self.you.fields[you::HOSTNAME].clone();
        a.user.full_name = self.you.fields[you::NAME].clone();
        a.user.name = self.you.fields[you::USER].clone();
        a.user.password = self.you.fields[you::PASSWORD].clone();
        a
    }

    /// Handles one terminal event.
    pub fn handle(&mut self, ev: Event) {
        match ev {
            Event::Key(k) if k.kind != KeyEventKind::Release => self.key(k),
            Event::Mouse(m) if !self.env.console => self.mouse(m),
            Event::FocusGained if self.screen == Screen::Drive => drive::rescan(self),
            _ => {}
        }
    }

    /// Rows an arrow moves: one, or several once the key has been held for HOLD_AFTER.
    fn arrow_rows(&mut self, code: KeyCode) -> usize {
        let now = self.clock;
        let rows = match self.held {
            Some((c, first, last)) if c == code && now - last < theme::REPEAT_GAP => {
                self.held = Some((c, first, now));
                if now - first >= theme::HOLD_AFTER {
                    theme::HOLD_ROWS
                } else {
                    1
                }
            }
            _ => {
                self.held = Some((code, now, now));
                1
            }
        };
        if matches!(code, KeyCode::Up | KeyCode::Down) {
            rows
        } else {
            1
        }
    }

    /// Handles a key: Ctrl+C first (quit before the install, ask during it), then the dialog, then the screen.
    pub fn key(&mut self, k: KeyEvent) {
        let ctrl_c = k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL);
        if self.dialog.as_ref().is_some_and(|d| d.open) {
            dialog::key(self, k.code);
            return;
        }
        if ctrl_c {
            match self.screen {
                Screen::Installing => {
                    self.dialog = Some(dialog::State::opening(self.full_motion()))
                }
                Screen::Done | Screen::Stopped => self.exit = Some(self.outcome_code()),
                _ => self.exit = Some(EXIT_LEFT),
            }
            return;
        }
        let text = !k
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let code = if text { k.code } else { KeyCode::Null };
        let rows = self.arrow_rows(code);
        let nav = match self.screen {
            Screen::Display => display::key(self, code),
            Screen::Welcome => welcome::key(self, code),
            Screen::Keyboard => keyboard::key(self, code, rows),
            Screen::Timezone => timezone::key(self, code, rows),
            Screen::You => you::key(self, code),
            Screen::Drive => drive::key(self, code, rows),
            Screen::Space => space::key(self, code, rows),
            Screen::Confirm => confirm::key(self, code),
            Screen::Installing => Nav::Stay,
            Screen::Done | Screen::Stopped => {
                if code == KeyCode::Enter {
                    Nav::Quit(self.outcome_code())
                } else {
                    Nav::Stay
                }
            }
        };
        self.navigate(nav);
    }

    /// The exit code the outcome gives.
    pub fn outcome_code(&self) -> i32 {
        match &self.outcome {
            Some(Ok(())) => 0,
            Some(Err(Failure::Stopped)) => EXIT_STOPPED,
            Some(Err(_)) => EXIT_FAILED,
            None => EXIT_LEFT,
        }
    }

    fn navigate(&mut self, nav: Nav) {
        use Screen::*;
        let order = [
            Display, Welcome, Keyboard, Timezone, You, Drive, Space, Confirm,
        ];
        let at = order.iter().position(|s| *s == self.screen);
        match (nav, at) {
            (Nav::Next, Some(i)) if i + 1 < order.len() => self.go(order[i + 1]),
            (Nav::Back, Some(i)) if i > 0 => self.go(order[i - 1]),
            (Nav::Back, _) => self.exit = Some(EXIT_LEFT),
            (Nav::Quit(code), _) => self.exit = Some(code),
            (Nav::Install, _) => installing::start(self),
            _ => {}
        }
    }

    /// Every screen's list, for the springs that move them.
    fn lists(&mut self) -> [&mut list::Cursor; 5] {
        [
            &mut self.display.cursor,
            &mut self.keyboard.cursor,
            &mut self.timezone.cursor,
            &mut self.drive.cursor,
            &mut self.space.cursor,
        ]
    }

    /// The list the wheel and the arrows move on this screen.
    fn cursor(&mut self) -> Option<&mut list::Cursor> {
        match self.screen {
            Screen::Display => Some(&mut self.display.cursor),
            Screen::Keyboard => Some(&mut self.keyboard.cursor),
            Screen::Timezone => Some(&mut self.timezone.cursor),
            Screen::Drive => Some(&mut self.drive.cursor),
            Screen::Space => Some(&mut self.space.cursor),
            _ => None,
        }
    }

    /// The mouse in foot: a press highlights, the release commits, moving off first cancels; the wheel moves the
    /// list, and a flick of the wheel carries on as far as the guide's projection says.
    fn mouse(&mut self, m: MouseEvent) {
        let at = self
            .hits
            .iter()
            .find(|(r, _)| r.contains((m.column, m.row).into()))
            .map(|(_, h)| *h);
        match m.kind {
            MouseEventKind::Down(MouseButton::Left) => self.pressed = at,
            MouseEventKind::Drag(MouseButton::Left) if at != self.pressed => self.pressed = None,
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(hit) = self.pressed.take().filter(|p| Some(*p) == at) {
                    self.click(hit);
                }
            }
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let dir = if m.kind == MouseEventKind::ScrollDown {
                    1
                } else {
                    -1
                };
                if let Some(c) = self.cursor() {
                    c.step(dir);
                }
                self.wheel
                    .retain(|(t, d)| *d == dir && self.clock - *t < theme::FLICK_GAP * 4.0);
                self.wheel.push((self.clock, dir));
            }
            _ => {}
        }
    }

    /// A committed click: a row selects it, a field takes focus, an action acts like its key.
    fn click(&mut self, hit: Hit) {
        match hit {
            Hit::Row(i) => {
                if self.screen == Screen::Display {
                    display::select(self, i);
                } else if let Some(c) = self.cursor() {
                    c.select(i);
                }
            }
            Hit::Field(i) => self.you.focus = i,
            Hit::Key(code, mods) => self.key(KeyEvent::new(code, mods)),
        }
    }

    /// Advances time by `dt` seconds: springs, the dialog's frames, the end of a wheel flick, the install's news.
    pub fn tick(&mut self, dt: f64) {
        self.clock += dt;
        let instant = !self.full_motion();
        self.camera.step(dt, instant);
        for c in self.lists() {
            c.tick(dt, instant);
        }
        if let Some(&(last, dir)) = self.wheel.last()
            && self.clock - last > theme::FLICK_GAP
        {
            let (n, first) = (self.wheel.len(), self.wheel[0].0);
            self.wheel.clear();
            if n >= theme::FLICK_EVENTS && last > first {
                let extra = motion::projection(n as f64 / (last - first), theme::DECELERATION)
                    .round() as isize;
                if let Some(c) = self.cursor() {
                    c.select(c.sel.saturating_add_signed(dir * extra));
                }
            }
        }
        dialog::tick(self);
        installing::poll(self, dt);
    }

    /// Whether anything moves, so the loop draws at the frame rate; otherwise it waits for the next event.
    pub fn animating(&mut self) -> bool {
        let lists = self.lists().iter().any(|c| c.moving());
        !self.camera.settled()
            || lists
            || !self.wheel.is_empty()
            || self.dialog.as_ref().is_some_and(dialog::State::moving)
            || self.install.as_ref().is_some_and(|i| !i.bar.settled())
    }

    /// The bottom border's actions for this screen.
    fn actions(&self) -> Vec<Action> {
        if self.dialog.is_some() {
            return installing::actions(self);
        }
        match self.screen {
            Screen::Display => display::actions(self),
            Screen::Welcome => welcome::actions(self),
            Screen::Keyboard => keyboard::actions(self),
            Screen::Timezone => timezone::actions(self),
            Screen::You => you::actions(self),
            Screen::Drive => drive::actions(self),
            Screen::Space => space::actions(self),
            Screen::Confirm => confirm::actions(self),
            Screen::Installing => installing::actions(self),
            Screen::Done => done::actions(self),
            Screen::Stopped => stopped::actions(self),
        }
    }

    /// The program's title in the top border.
    pub fn title(&self) -> String {
        format!("{} installer", self.env.os.name)
    }

    /// The Esc label on the first two screens: on the console, Esc hands over to the live desktop.
    pub fn esc_quit(&self) -> Action {
        frame::act(
            if self.env.console {
                "Esc try the live desktop"
            } else {
                "Esc quit"
            },
            KeyCode::Esc,
        )
    }

    /// Draws the frame and the screen into `f`.
    pub fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        let Some(rect) = frame::place(area) else {
            frame::too_small(f.buffer_mut(), area, &self.title(), &self.look);
            self.hits.clear();
            return;
        };
        let dx = (self.screen.place() * theme::SLIDE_COLUMNS - self.camera.pos).round() as i32;
        let mut c = Canvas {
            buf: f.buffer_mut(),
            frame: rect,
            dx: 0,
            look: self.look,
            hits: vec![],
        };
        let actions = self.actions();
        frame::draw(
            &mut c,
            &self.title(),
            &self.screen.label(),
            &actions,
            self.pressed,
        );
        c.dx = dx;
        match self.screen {
            Screen::Display => display::draw(self, &mut c),
            Screen::Welcome => welcome::draw(self, &mut c),
            Screen::Keyboard => keyboard::draw(self, &mut c),
            Screen::Timezone => timezone::draw(self, &mut c),
            Screen::You => you::draw(self, &mut c),
            Screen::Drive => drive::draw(self, &mut c),
            Screen::Space => space::draw(self, &mut c),
            Screen::Confirm => confirm::draw(self, &mut c),
            Screen::Installing => installing::draw(self, &mut c),
            Screen::Done => done::draw(self, &mut c),
            Screen::Stopped => stopped::draw(self, &mut c),
        }
        c.dx = 0;
        dialog::draw(self, &mut c);
        self.hits = std::mem::take(&mut c.hits);
    }
}

/// Runs the App in the terminal until it exits; returns its exit code. The mouse and focus reports are on only in a
/// desktop terminal; the console has neither.
pub fn run(mut app: App, slow: bool) -> io::Result<i32> {
    let mut terminal = ratatui::init();
    if !app.env.console {
        execute!(
            io::stdout(),
            event::EnableMouseCapture,
            event::EnableFocusChange
        )?;
    }
    let result = (|| {
        let mut last = Instant::now();
        let speed = if slow { 1.0 / theme::SLOW } else { 1.0 };
        loop {
            terminal.draw(|f| app.render(f))?;
            if std::mem::take(&mut app.bell) {
                io::stdout().write_all(b"\x07")?;
                io::stdout().flush()?;
            }
            if let Some(code) = app.exit {
                return Ok(code);
            }
            let wait = if app.animating() {
                Duration::from_secs_f64(1.0 / theme::FPS)
            } else if app.install.is_some() {
                Duration::from_millis(100)
            } else {
                Duration::from_secs(3600)
            };
            let ev = if event::poll(wait)? {
                Some(event::read()?)
            } else {
                None
            };
            let now = Instant::now();
            app.tick((now - last).as_secs_f64() * speed);
            last = now;
            if let Some(ev) = ev {
                app.handle(ev);
            }
        }
    })();
    if !app.env.console {
        let _ = execute!(
            io::stdout(),
            event::DisableMouseCapture,
            event::DisableFocusChange
        );
    }
    ratatui::restore();
    result
}
