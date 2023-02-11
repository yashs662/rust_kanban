use std::fmt::{
    self,
    Display,
    Formatter
};
use serde::{
    Serialize,
    Deserialize
};

use crossterm::event;

/// Represents an key.
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum Key {
    Enter,
    Tab,
    Backspace,
    Esc,

    Left,
    Right,
    Up,
    Down,

    Ins,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,

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
    BackTab,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
    Unknown,
}

impl Key {
    /// If exit
    pub fn is_exit(&self) -> bool {
        matches!(self, Key::Ctrl('c') | Key::Char('q') | Key::Esc)
    }
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
        // check if char is a digit if so return it
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
        match key_event {
            event::KeyEvent {
                code: event::KeyCode::Esc,
                ..
            } => Key::Esc,
            event::KeyEvent {
                code: event::KeyCode::Backspace,
                ..
            } => Key::Backspace,
            event::KeyEvent {
                code: event::KeyCode::Left,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => Key::ShiftLeft,
            event::KeyEvent {
                code: event::KeyCode::Left,
                ..
            } => Key::Left,
            event::KeyEvent {
                code: event::KeyCode::Right,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => Key::ShiftRight,
            event::KeyEvent {
                code: event::KeyCode::Right,
                ..
            } => Key::Right,
            event::KeyEvent {
                code: event::KeyCode::Up,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => Key::ShiftUp,
            event::KeyEvent {
                code: event::KeyCode::Up,
                ..
            } => Key::Up,
            event::KeyEvent {
                code: event::KeyCode::Down,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => Key::ShiftDown,
            event::KeyEvent {
                code: event::KeyCode::Down,
                ..
            } => Key::Down,
            event::KeyEvent {
                code: event::KeyCode::Home,
                ..
            } => Key::Home,
            event::KeyEvent {
                code: event::KeyCode::End,
                ..
            } => Key::End,
            event::KeyEvent {
                code: event::KeyCode::PageUp,
                ..
            } => Key::PageUp,
            event::KeyEvent {
                code: event::KeyCode::PageDown,
                ..
            } => Key::PageDown,
            event::KeyEvent {
                code: event::KeyCode::Delete,
                ..
            } => Key::Delete,
            event::KeyEvent {
                code: event::KeyCode::Insert,
                ..
            } => Key::Ins,
            event::KeyEvent {
                code: event::KeyCode::F(n),
                ..
            } => Key::from_f(n),
            event::KeyEvent {
                code: event::KeyCode::Enter,
                ..
            } => Key::Enter,

            event::KeyEvent {
                code: event::KeyCode::BackTab,
                modifiers: event::KeyModifiers::SHIFT,
                ..
            } => Key::BackTab,

            event::KeyEvent {
                code: event::KeyCode::Tab,
                ..
            } => Key::Tab,

            // First check for char + modifier
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::ALT,
                ..
            } => Key::Alt(c),
            event::KeyEvent {
                code: event::KeyCode::Char(c),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => Key::Ctrl(c),

            event::KeyEvent {
                code: event::KeyCode::Char(c),
                ..
            } => Key::Char(c),

            _ => Key::Unknown,
        }
    }
}
