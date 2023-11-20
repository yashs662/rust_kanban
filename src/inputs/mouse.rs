use crossterm::event;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, Serialize, Deserialize)]
pub enum Mouse {
    Drag(u16, u16),
    LeftPress,
    MiddlePress,
    Move(u16, u16),
    RightPress,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    ScrollUp,
    Unknown,
}

impl Display for Mouse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Mouse::Drag(x, y) => write!(f, "<Mouse::Drag({}, {})>", x, y),
            Mouse::LeftPress => write!(f, "<Mouse::Left>"),
            Mouse::MiddlePress => write!(f, "<Mouse::Middle>"),
            Mouse::Move(x, y) => write!(f, "<Mouse::Move({}, {})>", x, y),
            Mouse::RightPress => write!(f, "<Mouse::Right>"),
            Mouse::ScrollDown => write!(f, "<Mouse::ScrollDown>"),
            Mouse::ScrollLeft => write!(f, "<Mouse::Ctrl + ScrollUp>"),
            Mouse::ScrollRight => write!(f, "<Mouse::Ctrl + ScrollDown>"),
            Mouse::ScrollUp => write!(f, "<Mouse::ScrollUp>"),
            Mouse::Unknown => write!(f, "<Mouse::Unknown>"),
        }
    }
}

impl From<event::MouseEvent> for Mouse {
    fn from(mouse_event: event::MouseEvent) -> Self {
        match mouse_event {
            event::MouseEvent {
                kind: event::MouseEventKind::Up(event::MouseButton::Left),
                ..
            } => Mouse::LeftPress,
            event::MouseEvent {
                kind: event::MouseEventKind::Up(event::MouseButton::Right),
                ..
            } => Mouse::RightPress,
            event::MouseEvent {
                kind: event::MouseEventKind::Up(event::MouseButton::Middle),
                ..
            } => Mouse::MiddlePress,
            event::MouseEvent {
                kind: event::MouseEventKind::ScrollDown,
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => Mouse::ScrollLeft,
            event::MouseEvent {
                kind: event::MouseEventKind::ScrollUp,
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => Mouse::ScrollRight,
            event::MouseEvent {
                kind: event::MouseEventKind::ScrollUp,
                ..
            } => Mouse::ScrollUp,
            event::MouseEvent {
                kind: event::MouseEventKind::ScrollDown,
                ..
            } => Mouse::ScrollDown,
            event::MouseEvent {
                kind: event::MouseEventKind::Moved,
                column,
                row,
                ..
            } => Mouse::Move(column, row),
            event::MouseEvent {
                kind: event::MouseEventKind::Drag(event::MouseButton::Left),
                column,
                row,
                ..
            } => Mouse::Drag(column, row),
            _ => Mouse::Unknown,
        }
    }
}
