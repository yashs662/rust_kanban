use std::collections::HashMap;
use std::fmt::{self, Display};
use std::slice::Iter;

use log::debug;

use super::state::KeyBindings;
use crate::app::AppConfig;
use crate::inputs::key::Key;
use crate::io::data_handler::get_config;

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Quit,
    NextFocus,
    PrvFocus,
    OpenConfigMenu,
    Up,
    Down,
    Right,
    Left,
    MoveCardUp,
    MoveCardDown,
    MoveCardRight,
    MoveCardLeft,
    TakeUserInput,
    GoToPreviousUIMode,
    Enter,
    HideUiElement,
    SaveState,
    NewBoard,
    NewCard,
    DeleteCard,
    DeleteBoard,
    ChangeCardStatusToCompleted,
    ChangeCardStatusToActive,
    ChangeCardStatusToStale,
    ResetUI,
    GoToMainMenu,
    ToggleCommandPalette,
    ClearAllToasts
}

impl Action {
    /// All available actions
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 28] = [
            Action::Quit,
            Action::NextFocus,
            Action::PrvFocus,
            Action::OpenConfigMenu,
            Action::Up,
            Action::Down,
            Action::Right,
            Action::Left,
            Action::MoveCardUp,
            Action::MoveCardDown,
            Action::MoveCardRight,
            Action::MoveCardLeft,
            Action::TakeUserInput,
            Action::GoToPreviousUIMode,
            Action::Enter,
            Action::HideUiElement,
            Action::SaveState,
            Action::NewBoard,
            Action::NewCard,
            Action::DeleteCard,
            Action::DeleteBoard,
            Action::ChangeCardStatusToCompleted,
            Action::ChangeCardStatusToActive,
            Action::ChangeCardStatusToStale,
            Action::ResetUI,
            Action::GoToMainMenu,
            Action::ToggleCommandPalette,
            Action::ClearAllToasts
        ];
        ACTIONS.iter()
    }

    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::NextFocus => &[Key::Tab],
            Action::PrvFocus => &[Key::BackTab],
            Action::OpenConfigMenu => &[Key::Char('c')],
            Action::Up => &[Key::Up],
            Action::Down => &[Key::Down],
            Action::Right => &[Key::Right],
            Action::Left => &[Key::Left],
            Action::MoveCardUp => &[Key::ShiftUp],
            Action::MoveCardDown => &[Key::ShiftDown],
            Action::MoveCardRight => &[Key::ShiftRight],
            Action::MoveCardLeft => &[Key::ShiftLeft],
            Action::TakeUserInput => &[Key::Char('i')],
            Action::GoToPreviousUIMode => &[Key::Esc],
            Action::Enter => &[Key::Enter],
            Action::HideUiElement => &[Key::Char('h')],
            Action::SaveState => &[Key::Ctrl('s')],
            Action::NewBoard => &[Key::Char('b')],
            Action::NewCard => &[Key::Char('n')],
            Action::DeleteCard => &[Key::Char('d')],
            Action::DeleteBoard => &[Key::Char('D')],
            Action::ChangeCardStatusToCompleted => &[Key::Char('1')],
            Action::ChangeCardStatusToActive => &[Key::Char('2')],
            Action::ChangeCardStatusToStale => &[Key::Char('3')],
            Action::ResetUI => &[Key::Char('r')],
            Action::GoToMainMenu => &[Key::Char('m')],
            Action::ToggleCommandPalette => &[Key::Ctrl('p')],
            Action::ClearAllToasts => &[Key::Char('t')]
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
            Action::PrvFocus => "Focus previous",
            Action::OpenConfigMenu => "Open config Menu",
            Action::Up => "Go up",
            Action::Down => "Go down",
            Action::Right => "Go right",
            Action::Left => "Go left",
            Action::MoveCardUp => "Move card up",
            Action::MoveCardDown => "Move card down",
            Action::MoveCardRight => "Move card right",
            Action::MoveCardLeft => "Move card left",
            Action::TakeUserInput => "Enter input mode",
            Action::GoToPreviousUIMode => "Go to previous mode",
            Action::Enter => "Accept",
            Action::HideUiElement => "Hide Focused element",
            Action::SaveState => "Save Kanban state",
            Action::NewBoard => "Create new board",
            Action::NewCard => "Create new card in current board",
            Action::DeleteCard => "Delete focused element",
            Action::DeleteBoard => "Delete Board",
            Action::ChangeCardStatusToCompleted => "Change card status to completed",
            Action::ChangeCardStatusToActive => "Change card status to active",
            Action::ChangeCardStatusToStale => "Change card status to stale",
            Action::ResetUI => "Reset UI",
            Action::GoToMainMenu => "Go to main menu",
            Action::ToggleCommandPalette => "Open command palette",
            Action::ClearAllToasts => "Clear all toasts"
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
        let get_config_status = get_config(false);
        let config = if get_config_status.is_err() {
            debug!("Error getting config: {}", get_config_status.unwrap_err());
            AppConfig::default()
        } else {
            get_config_status.unwrap()
        };
        let current_bindings = config.keybindings.clone();
        let action_list = &mut Vec::new();
        for (k, _v) in current_bindings.iter() {
            action_list.push(KeyBindings::str_to_action(
                current_bindings.clone(),
                k.clone(),
            ));
        }
        let binding_action = KeyBindings::key_to_action(config.keybindings.clone(), key);
        if binding_action.is_some() {
            return binding_action;
        } else {
            let temp_action = Action::iterator()
                .filter(|action| self.0.contains(action))
                .find(|action| action.keys().contains(&key));
            if temp_action.is_some() && !action_list.contains(&temp_action) {
                return temp_action;
            } else {
                return None;
            }
        }
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
        let _actions: Actions = vec![Action::Quit, Action::NextFocus, Action::PrvFocus].into();
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
