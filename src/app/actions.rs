use std::collections::HashMap;
use std::fmt::{self, Display};
use std::slice::Iter;

use crate::inputs::key::Key;

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Quit,
    NextFocus,
    PreviousFocus,
    SetUiMode,
    ToggleConfig,
    Up,
    Down,
    Right,
    Left,
    TakeUserInput,
    Escape,
    Enter,
    Hide,
    SaveState,
    NewBoard,
    NewCard,
    Delete
}

impl Action {
    /// All available actions
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 17] = [
            Action::Quit,
            Action::NextFocus,
            Action::PreviousFocus,
            Action::SetUiMode,
            Action::ToggleConfig,
            Action::Up,
            Action::Down,
            Action::Right,
            Action::Left,
            Action::TakeUserInput,
            Action::Escape,
            Action::Enter,
            Action::Hide,
            Action::SaveState,
            Action::NewBoard,
            Action::NewCard,
            Action::Delete
        ];
        ACTIONS.iter()
    }

    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::NextFocus => &[Key::Tab],
            Action::PreviousFocus => &[Key::ShiftTab],
            Action::SetUiMode => &[Key::Char('1'), Key::Char('2'), Key::Char('3'),
                                   Key::Char('4'), Key::Char('5'), Key::Char('6'),
                                   Key::Char('7'), Key::Char('8'), Key::Char('9')
                                   ],
            Action::ToggleConfig => &[Key::Char('c')],
            Action::Up => &[Key::Up],
            Action::Down => &[Key::Down],
            Action::Right => &[Key::Right],
            Action::Left => &[Key::Left],
            Action::TakeUserInput => &[Key::Char('i')],
            Action::Escape => &[Key::Esc],
            Action::Enter => &[Key::Enter],
            Action::Hide => &[Key::Char('h')],
            Action::SaveState => &[Key::Ctrl('s')],
            Action::NewBoard => &[Key::Char('b')],
            Action::NewCard => &[Key::Char('n')],
            Action::Delete => &[Key::Char('d')],
        }
    }

    pub fn all() -> Vec<Action> {
        Action::iterator().cloned().collect()
    }
}

/// Could display a user friendly short description of action
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::NextFocus => "Focus next",
            Action::PreviousFocus => "Focus previous",
            Action::SetUiMode => "Set UI mode",
            Action::ToggleConfig => "Open config Menu",
            Action::Up => "Go up",
            Action::Down => "Go down",
            Action::Right => "Go right",
            Action::Left => "Go left",
            Action::TakeUserInput => "Enter input mode",
            Action::Escape => "Go to previous mode",
            Action::Enter => "Accept",
            Action::Hide => "Hide Focused element",
            Action::SaveState => "Save Kanban state",
            Action::NewBoard => "Create new board",
            Action::NewCard => "Create new card in current board",
            Action::Delete => "Delete focused element",
        };
        write!(f, "{}", str)
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        // Check key unicity
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {} with actions {}", key, actions)
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        // Ok, we can create contextual actions
        Self(actions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_action_by_key() {
        let actions: Actions = vec![Action::Quit, Action::NextFocus].into();
        let result = actions.find(Key::Ctrl('c'));
        assert_eq!(result, Some(&Action::Quit));
    }

    #[test]
    fn should_find_action_by_key_not_found() {
        let actions: Actions = vec![Action::Quit, Action::NextFocus].into();
        let result = actions.find(Key::Alt('w'));
        assert_eq!(result, None);
    }

    #[test]
    fn should_create_actions_from_vec() {
        let _actions: Actions = vec![
            Action::Quit,
            Action::NextFocus,
            Action::PreviousFocus,
        ]
        .into();
    }

    #[test]
    #[should_panic]
    fn should_panic_when_create_actions_conflict_key() {
        let _actions: Actions = vec![
            Action::Quit,
            Action::Quit,
            Action::NextFocus,
            Action::NextFocus,
            Action::NextFocus,
        ]
        .into();
    }
}
