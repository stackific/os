//! You: the user, an administrator through `wheel`, and the computer name. The user name follows the name's first
//! word until edited; passwords show as dots; the first problem shows under the fields as you type, and "Enter next"
//! appears once every field is valid.

use ratatui::crossterm::event::KeyCode;

use super::frame::{Action, Canvas, Hit, act};
use super::theme::{BODY_ROW, LEFT, ROW, TITLE_ROW, YOU_LABEL};
use super::{App, Nav};
use crate::answers::{self, LIVE_LAYOUT};

/// Field indexes.
pub const NAME: usize = 0;
pub const USER: usize = 1;
pub const PASSWORD: usize = 2;
pub const CONFIRM: usize = 3;
pub const HOSTNAME: usize = 4;
const LABELS: [&str; 5] = [
    "Your name",
    "User name",
    "Password",
    "Confirm",
    "Computer name",
];

/// The fields, the one with focus, and whether the user name was typed by hand.
#[derive(Clone, Debug)]
pub struct State {
    pub fields: [String; 5],
    pub focus: usize,
    pub user_edited: bool,
}

impl State {
    /// Empty fields but the computer name, which starts as os-release's DEFAULT_HOSTNAME.
    pub fn new(hostname: &str) -> Self {
        let mut fields: [String; 5] = Default::default();
        fields[HOSTNAME] = hostname.into();
        Self {
            fields,
            focus: NAME,
            user_edited: false,
        }
    }

    /// Field `i`'s check.
    fn check(&self, i: usize, taken: &[String]) -> Result<(), String> {
        let f = &self.fields;
        match i {
            NAME => answers::check_full_name(&f[NAME]),
            USER => answers::check_user_name(&f[USER], taken),
            PASSWORD => answers::check_password(&f[PASSWORD], &f[PASSWORD]),
            CONFIRM => answers::check_password(&f[PASSWORD], &f[CONFIRM]),
            _ => answers::check_hostname(&f[HOSTNAME]),
        }
    }

    /// Whether every field is valid.
    pub fn valid(&self, taken: &[String]) -> bool {
        (0..LABELS.len()).all(|i| self.check(i, taken).is_ok())
    }

    /// The first problem among the fields so far: those before the focus, and the focused one once typed in.
    pub fn problem(&self, taken: &[String]) -> Option<String> {
        (0..=self.focus)
            .filter(|&i| i < self.focus || !self.fields[i].is_empty())
            .find_map(|i| self.check(i, taken).err())
    }
}

/// Typing edits the focused field; Tab, Shift+Tab, ↑ and ↓ move between fields; Enter goes on once all are valid.
pub fn key(app: &mut App, code: KeyCode) -> Nav {
    let taken = &app.env.catalog.taken_users;
    let y = &mut app.you;
    match code {
        KeyCode::Tab | KeyCode::Down => y.focus = (y.focus + 1) % LABELS.len(),
        KeyCode::BackTab | KeyCode::Up => y.focus = (y.focus + LABELS.len() - 1) % LABELS.len(),
        KeyCode::Enter if y.valid(taken) => return Nav::Next,
        KeyCode::Esc => return Nav::Back,
        KeyCode::Char(ch) => {
            y.fields[y.focus].push(ch);
            edited(y);
        }
        KeyCode::Backspace => {
            y.fields[y.focus].pop();
            edited(y);
        }
        _ => {}
    }
    Nav::Stay
}

/// After an edit: the user name follows the name until it is edited by hand.
fn edited(y: &mut State) {
    match y.focus {
        USER => y.user_edited = !y.fields[USER].is_empty(),
        NAME if !y.user_edited => y.fields[USER] = answers::suggested_user_name(&y.fields[NAME]),
        _ => {}
    }
}

/// You's bottom border: "Enter next" only once every field is valid.
pub fn actions(app: &App) -> Vec<Action> {
    let mut a = vec![act("Tab next field", KeyCode::Tab)];
    if app.you.valid(&app.env.catalog.taken_users) {
        a.push(act("Enter next", KeyCode::Enter));
    }
    a.push(act("Esc back", KeyCode::Esc));
    a
}

/// Draws You.
pub fn draw(app: &App, c: &mut Canvas) {
    let (look, y) = (app.look, &app.you);
    c.text(
        TITLE_ROW,
        LEFT,
        "Your user, an administrator of this computer",
        look.title(),
    );
    for (i, label) in LABELS.iter().enumerate() {
        let r = BODY_ROW + i as u16;
        let shown = if matches!(i, PASSWORD | CONFIRM) {
            look.sym("•").repeat(y.fields[i].chars().count())
        } else {
            y.fields[i].clone()
        };
        let label_style = if i == y.focus {
            look.title()
        } else {
            look.body()
        };
        c.text(r, ROW, label, label_style);
        let end = c.text(r, ROW + YOU_LABEL, &shown, look.body());
        if i == y.focus {
            c.text(r, end, look.sym("▏"), look.mark());
        }
        c.hit(r, ROW, super::theme::RIGHT - ROW, Hit::Field(i));
    }
    if let Some(p) = y.problem(&app.env.catalog.taken_users) {
        c.text(
            BODY_ROW + 6,
            LEFT,
            &format!("{} {p}", look.sym("✗")),
            look.warning(),
        );
    }
    let help = [
        "User name: lowercase letters, digits, - and _, starting with a letter.".to_string(),
        "Computer name: lowercase letters, digits and -, up to 63.".into(),
        format!("Typed with the live session's layout ({LIVE_LAYOUT})."),
    ];
    for (i, h) in help.iter().enumerate() {
        c.text(BODY_ROW + 9 + i as u16, LEFT, h, look.help());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_name_follows_the_name_until_edited() {
        let mut y = State::new("stackific");
        for ch in "Ada Lovelace".chars() {
            y.fields[NAME].push(ch);
            edited(&mut y);
        }
        assert_eq!(y.fields[USER], "ada");
        y.focus = USER;
        y.fields[USER] = "al".into();
        edited(&mut y);
        y.focus = NAME;
        y.fields[NAME].push('x');
        edited(&mut y);
        assert_eq!(y.fields[USER], "al");
    }

    #[test]
    fn problems_show_as_you_go() {
        let mut y = State::new("stackific");
        assert_eq!(y.problem(&[]), None);
        y.fields = [
            "Ada".into(),
            "ada".into(),
            "abcdefgh".into(),
            "abcdefg".into(),
            "stackific".into(),
        ];
        y.focus = CONFIRM;
        assert_eq!(
            y.problem(&[]),
            Some("The passwords do not match yet.".into())
        );
        assert!(!y.valid(&[]));
        y.fields[CONFIRM].push('h');
        assert!(y.valid(&[]) && y.problem(&[]).is_none());
    }
}
