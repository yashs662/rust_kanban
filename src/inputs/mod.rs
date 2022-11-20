use self::key::Key;

pub mod events;
pub mod key;

pub enum InputEvent<'a> {
    /// An input event occurred.
    Input(Key<'a>),
    /// An tick event occurred.
    Tick,
}
