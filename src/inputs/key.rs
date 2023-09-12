use crossterm::event::{self, KeyModifiers};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum Key {
    Esc,

    Backspace,
    AltBackspace,

    Up,
    ShiftUp,
    CtrlUp,
    CtrlAltUp,
    PageUp,

    Down,
    ShiftDown,
    CtrlDown,
    CtrlAltDown,
    PageDown,

    Left,
    ShiftLeft,
    CtrlLeft,
    CtrlAltLeft,

    Right,
    ShiftRight,
    CtrlRight,
    CtrlAltRight,

    Enter,

    Tab,
    BackTab,

    Space,
    Ins,
    Delete,
    Home,
    End,
    F0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Char(char),
    Ctrl(char),
    Alt(char),
    AltDelete,
    CtrlAlt(char),
    Unknown,
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
    pub fn to_digit(&self) -> u8 {
        match self {
            Key::Char(c) => c.to_digit(10).unwrap() as u8,
            _ => panic!("not a digit"),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Key::Alt(' ') => write!(f, "<Alt+Space>"),
            Key::Ctrl(' ') => write!(f, "<Ctrl+Space>"),
            Key::Char(' ') => write!(f, "<Space>"),
            Key::Alt(c) => write!(f, "<Alt+{}>", c),
            Key::Ctrl(c) => write!(f, "<Ctrl+{}>", c),
            Key::Char(c) => write!(f, "<{}>", c),
            Key::Tab => write!(f, "<Tab>"),
            Key::BackTab => write!(f, "<Shift+Tab>"),
            Key::ShiftUp => write!(f, "<Shift+Up>"),
            Key::ShiftDown => write!(f, "<Shift+Down>"),
            Key::ShiftLeft => write!(f, "<Shift+Left>"),
            Key::ShiftRight => write!(f, "<Shift+Right>"),
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
        match s {
            "Enter" => Key::Enter,
            "Tab" => Key::Tab,
            "Backspace" => Key::Backspace,
            "Esc" => Key::Esc,
            "Space" => Key::Space,
            "Left" => Key::Left,
            "Right" => Key::Right,
            "Up" => Key::Up,
            "Down" => Key::Down,
            "Ins" => Key::Ins,
            "Delete" => Key::Delete,
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            "F0" => Key::F0,
            "F1" => Key::F1,
            "F2" => Key::F2,
            "F3" => Key::F3,
            "F4" => Key::F4,
            "F5" => Key::F5,
            "F6" => Key::F6,
            "F7" => Key::F7,
            "F8" => Key::F8,
            "F9" => Key::F9,
            "F10" => Key::F10,
            "F11" => Key::F11,
            "F12" => Key::F12,
            "BackTab" => Key::BackTab,
            "ShiftUp" => Key::ShiftUp,
            "ShiftDown" => Key::ShiftDown,
            "ShiftLeft" => Key::ShiftLeft,
            "ShiftRight" => Key::ShiftRight,
            _ => Key::Unknown,
        }
    }
}

impl From<&Map<String, Value>> for Key {
    fn from(value: &Map<String, Value>) -> Self {
        if value.get("Char").is_some() {
            Key::Char(
                value
                    .get("Char")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap(),
            )
        } else if value.get("Alt").is_some() {
            Key::Alt(
                value
                    .get("Alt")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap(),
            )
        } else if value.get("Ctrl").is_some() {
            Key::Ctrl(
                value
                    .get("Ctrl")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap(),
            )
        } else {
            Key::Unknown
        }
    }
}
