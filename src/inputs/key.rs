use crossterm::event::{self, KeyModifiers};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum Key {
    Alt(char),
    AltBackspace,
    AltDelete,
    BackTab,
    Backspace,
    Char(char),
    Ctrl(char),
    CtrlAlt(char),
    CtrlAltDown,
    CtrlAltLeft,
    CtrlAltRight,
    CtrlAltUp,
    CtrlDown,
    CtrlLeft,
    CtrlRight,
    CtrlUp,
    Delete,
    Down,
    End,
    Enter,
    Esc,
    F0,
    F1,
    F10,
    F11,
    F12,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Home,
    Ins,
    Left,
    PageDown,
    PageUp,
    Right,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    ShiftUp,
    Space,
    Tab,
    Unknown,
    Up,
}

impl Key {
    pub fn from_f(n: u8) -> Key {
        match n {
            0 => Key::F0,
            1 => Key::F1,
            2 => Key::F2,
            3 => Key::F3,
            4 => Key::F4,
            5 => Key::F5,
            6 => Key::F6,
            7 => Key::F7,
            8 => Key::F8,
            9 => Key::F9,
            10 => Key::F10,
            11 => Key::F11,
            12 => Key::F12,
            _ => panic!("unknown function key: F{}", n),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Key::Alt(c) => write!(f, "<Alt+{}>", c),
            Key::AltBackspace => write!(f, "<Alt+Backspace>"),
            Key::AltDelete => write!(f, "<Alt+Delete>"),
            Key::BackTab => write!(f, "<Shift+Tab>"),
            Key::Backspace => write!(f, "<Backspace>"),
            Key::Char(c) => write!(f, "<{}>", c),
            Key::Ctrl(c) => write!(f, "<Ctrl+{}>", c),
            Key::CtrlAlt(c) => write!(f, "<Ctrl+Alt+{}>", c),
            Key::CtrlAltDown => write!(f, "<Ctrl+Alt+Down>"),
            Key::CtrlAltLeft => write!(f, "<Ctrl+Alt+Left>"),
            Key::CtrlAltRight => write!(f, "<Ctrl+Alt+Right>"),
            Key::CtrlAltUp => write!(f, "<Ctrl+Alt+Up>"),
            Key::CtrlDown => write!(f, "<Ctrl+Down>"),
            Key::CtrlLeft => write!(f, "<Ctrl+Left>"),
            Key::CtrlRight => write!(f, "<Ctrl+Right>"),
            Key::CtrlUp => write!(f, "<Ctrl+Up>"),
            Key::Delete => write!(f, "<Delete>"),
            Key::Down => write!(f, "<Down>"),
            Key::End => write!(f, "<End>"),
            Key::Enter => write!(f, "<Enter>"),
            Key::Esc => write!(f, "<Esc>"),
            Key::Home => write!(f, "<Home>"),
            Key::Ins => write!(f, "<Ins>"),
            Key::Left => write!(f, "<Left>"),
            Key::PageDown => write!(f, "<PageDown>"),
            Key::PageUp => write!(f, "<PageUp>"),
            Key::Right => write!(f, "<Right>"),
            Key::ShiftDown => write!(f, "<Shift+Down>"),
            Key::ShiftLeft => write!(f, "<Shift+Left>"),
            Key::ShiftRight => write!(f, "<Shift+Right>"),
            Key::ShiftUp => write!(f, "<Shift+Up>"),
            Key::Space => write!(f, "<Space>"),
            Key::Tab => write!(f, "<Tab>"),
            Key::Unknown => write!(f, "<Unknown>"),
            Key::Up => write!(f, "<Up>"),
            _ => write!(f, "<{:?}>", self),
        }
    }
}

impl From<event::KeyEvent> for Key {
    fn from(key_event: event::KeyEvent) -> Self {
        let ctrl = key_event.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key_event.modifiers.contains(KeyModifiers::ALT);
        let shift = key_event.modifiers.contains(KeyModifiers::SHIFT);
        match key_event {
            event::KeyEvent {
                code: event::KeyCode::Esc,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::Esc,

            event::KeyEvent {
                code: event::KeyCode::Backspace,
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if alt {
                    Key::AltBackspace
                } else {
                    Key::Backspace
                }
            }

            event::KeyEvent {
                code: event::KeyCode::Up,
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if ctrl && alt {
                    Key::CtrlAltUp
                } else if ctrl {
                    Key::CtrlUp
                } else if shift {
                    Key::ShiftUp
                } else {
                    Key::Up
                }
            }

            event::KeyEvent {
                code: event::KeyCode::Down,
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if ctrl && alt {
                    Key::CtrlAltDown
                } else if ctrl {
                    Key::CtrlDown
                } else if shift {
                    Key::ShiftDown
                } else {
                    Key::Down
                }
            }

            event::KeyEvent {
                code: event::KeyCode::Left,
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if ctrl && alt {
                    Key::CtrlAltLeft
                } else if ctrl {
                    Key::CtrlLeft
                } else if shift {
                    Key::ShiftLeft
                } else {
                    Key::Left
                }
            }

            event::KeyEvent {
                code: event::KeyCode::Right,
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if ctrl && alt {
                    Key::CtrlAltRight
                } else if ctrl {
                    Key::CtrlRight
                } else if shift {
                    Key::ShiftRight
                } else {
                    Key::Right
                }
            }

            event::KeyEvent {
                code: event::KeyCode::Delete,
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if alt {
                    Key::AltDelete
                } else {
                    Key::Delete
                }
            }

            event::KeyEvent {
                code: event::KeyCode::Home,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::Home,
            event::KeyEvent {
                code: event::KeyCode::End,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::End,
            event::KeyEvent {
                code: event::KeyCode::PageUp,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::PageUp,
            event::KeyEvent {
                code: event::KeyCode::PageDown,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::PageDown,
            event::KeyEvent {
                code: event::KeyCode::Insert,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::Ins,
            event::KeyEvent {
                code: event::KeyCode::F(n),
                kind: event::KeyEventKind::Press,
                ..
            } => Key::from_f(n),
            event::KeyEvent {
                code: event::KeyCode::Enter,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::Enter,
            event::KeyEvent {
                code: event::KeyCode::BackTab,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::BackTab,
            event::KeyEvent {
                code: event::KeyCode::Tab,
                kind: event::KeyEventKind::Press,
                ..
            } => Key::Tab,
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                kind: event::KeyEventKind::Press,
                ..
            } => {
                if ctrl && alt {
                    Key::CtrlAlt(c)
                } else if ctrl {
                    Key::Ctrl(c)
                } else if alt {
                    Key::Alt(c)
                } else if shift {
                    Key::Char(c.to_ascii_uppercase())
                } else {
                    Key::Char(c)
                }
            }
            _ => Key::Unknown,
        }
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        // handle char
        if s.len() == 1 {
            return Key::Char(s.chars().next().unwrap());
        }
        // handle alt+char
        if s.len() == 6 && s.starts_with("<Alt+") && s.ends_with('>') {
            return Key::Alt(s.chars().nth(5).unwrap());
        }
        // handle ctrl+char
        if s.len() == 7 && s.starts_with("<Ctrl+") && s.ends_with('>') {
            return Key::Ctrl(s.chars().nth(6).unwrap());
        }
        // handle ctrl+alt+char
        if s.len() == 10 && s.starts_with("<Ctrl+Alt+") && s.ends_with('>') {
            return Key::CtrlAlt(s.chars().nth(9).unwrap());
        }
        match s {
            "<Alt+Backspace>" => Key::AltBackspace,
            "<Alt+Delete>" => Key::AltDelete,
            "<Backspace>" => Key::Backspace,
            "<Ctrl+Alt+Down>" => Key::CtrlAltDown,
            "<Ctrl+Alt+Left>" => Key::CtrlAltLeft,
            "<Ctrl+Alt+Right>" => Key::CtrlAltRight,
            "<Ctrl+Alt+Up>" => Key::CtrlAltUp,
            "<Ctrl+Down>" => Key::CtrlDown,
            "<Ctrl+Left>" => Key::CtrlLeft,
            "<Ctrl+Right>" => Key::CtrlRight,
            "<Ctrl+Up>" => Key::CtrlUp,
            "<Delete>" => Key::Delete,
            "<Down>" => Key::Down,
            "<End>" => Key::End,
            "<Enter>" => Key::Enter,
            "<Esc>" => Key::Esc,
            "<Home>" => Key::Home,
            "<Ins>" => Key::Ins,
            "<Left>" => Key::Left,
            "<PageDown>" => Key::PageDown,
            "<PageUp>" => Key::PageUp,
            "<Right>" => Key::Right,
            "<Shift+Down>" => Key::ShiftDown,
            "<Shift+Left>" => Key::ShiftLeft,
            "<Shift+Right>" => Key::ShiftRight,
            "<Shift+Up>" => Key::ShiftUp,
            "<Space>" => Key::Space,
            "<Tab>" => Key::Tab,
            "<Unknown>" => Key::Unknown,
            "<Up>" => Key::Up,
            _ => Key::Unknown,
        }
    }
}

impl From<&Map<String, Value>> for Key {
    // TODO: handle more key types
    fn from(value: &Map<String, Value>) -> Self {
        fn char_from_value(val: &Value) -> char {
            val.as_str().and_then(|s| s.chars().next()).unwrap()
        }
        if let Some(char_value) = value.get("Char") {
            Key::Char(char_from_value(char_value))
        } else if let Some(alt_value) = value.get("Alt") {
            Key::Alt(char_from_value(alt_value))
        } else if let Some(ctrl_value) = value.get("Ctrl") {
            Key::Ctrl(char_from_value(ctrl_value))
        } else {
            Key::Unknown
        }
    }
}
