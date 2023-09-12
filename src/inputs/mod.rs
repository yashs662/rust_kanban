use self::{key::Key, mouse::Mouse};

pub mod events;
pub mod key;
pub mod mouse;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum InputEvent {
    KeyBoardInput(Key),
    MouseAction(Mouse),
    Tick,
}
