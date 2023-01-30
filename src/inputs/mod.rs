use self::key::Key;

pub mod events;
pub mod key;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum InputEvent {
    /// An input event occurred.
    Input(Key),
    /// An tick event occurred.
    Tick,
}
