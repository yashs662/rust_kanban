use self::{key::Key, mouse::Mouse};

pub mod events;
pub mod key;
pub mod mouse;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum InputEvent {
    /// An input event occurred.
    KeyBoardInput(Key),
    MouseAction(Mouse),
    /// An tick event occurred.
    Tick,
}
