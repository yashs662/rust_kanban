use self::key::Key;

pub mod events;
pub mod key;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum InputEvent {
    /// An input event occurred.
    KeyBoardInput(Key),
    MouseAction(crossterm::event::MouseEvent),
    /// An tick event occurred.
    Tick,
}
