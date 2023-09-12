use super::state::KeyBindings;
use crate::{app::AppConfig, inputs::key::Key};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    slice::Iter,
};

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
    StopUserInput,
    GoToPreviousUIMode,
    Enter,
    HideUiElement,
    SaveState,
    NewBoard,
    NewCard,
    Delete,
    DeleteBoard,
    ChangeCardStatusToCompleted,
    ChangeCardStatusToActive,
    ChangeCardStatusToStale,
    ResetUI,
    GoToMainMenu,
    ToggleCommandPalette,
    Undo,
    Redo,
    ClearAllToasts,
}

impl Action {
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 31] = [
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
            Action::StopUserInput,
            Action::GoToPreviousUIMode,
            Action::Enter,
            Action::HideUiElement,
            Action::SaveState,
            Action::NewBoard,
            Action::NewCard,
            Action::Delete,
            Action::DeleteBoard,
            Action::ChangeCardStatusToCompleted,
            Action::ChangeCardStatusToActive,
            Action::ChangeCardStatusToStale,
            Action::ResetUI,
            Action::GoToMainMenu,
            Action::ToggleCommandPalette,
            Action::Undo,
            Action::Redo,
            Action::ClearAllToasts,
        ];
        ACTIONS.iter()
    }

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
            Action::StopUserInput => &[Key::Ins],
            Action::GoToPreviousUIMode => &[Key::Esc],
            Action::Enter => &[Key::Enter],
            Action::HideUiElement => &[Key::Char('h')],
            Action::SaveState => &[Key::Ctrl('s')],
            Action::NewBoard => &[Key::Char('b')],
            Action::NewCard => &[Key::Char('n')],
            Action::Delete => &[Key::Char('d')],
            Action::DeleteBoard => &[Key::Char('D')],
            Action::ChangeCardStatusToCompleted => &[Key::Char('1')],
            Action::ChangeCardStatusToActive => &[Key::Char('2')],
            Action::ChangeCardStatusToStale => &[Key::Char('3')],
            Action::ResetUI => &[Key::Char('r')],
            Action::GoToMainMenu => &[Key::Char('m')],
            Action::ToggleCommandPalette => &[Key::Ctrl('p')],
            Action::Undo => &[Key::Ctrl('z')],
            Action::Redo => &[Key::Ctrl('y')],
            Action::ClearAllToasts => &[Key::Char('t')],
        }
    }

    pub fn all() -> Vec<Action> {
        Action::iterator().cloned().collect()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::NextFocus => "Focus next",
            Action::PrvFocus => "Focus previous",
            Action::OpenConfigMenu => "configure",
            Action::Up => "Go up",
            Action::Down => "Go down",
            Action::Right => "Go right",
            Action::Left => "Go left",
            Action::MoveCardUp => "Move card up",
            Action::MoveCardDown => "Move card down",
            Action::MoveCardRight => "Move card right",
            Action::MoveCardLeft => "Move card left",
            Action::TakeUserInput => "Enter input mode",
            Action::StopUserInput => "Stop input mode",
            Action::GoToPreviousUIMode => "Go to previous mode",
            Action::Enter => "Accept",
            Action::HideUiElement => "Hide Focused element",
            Action::SaveState => "Save Kanban state",
            Action::NewBoard => "Create new board",
            Action::NewCard => "Create new card in current board",
            Action::Delete => "Delete focused element",
            Action::DeleteBoard => "Delete Board",
            Action::ChangeCardStatusToCompleted => "Change card status to completed",
            Action::ChangeCardStatusToActive => "Change card status to active",
            Action::ChangeCardStatusToStale => "Change card status to stale",
            Action::ResetUI => "Reset UI",
            Action::GoToMainMenu => "Go to main menu",
            Action::ToggleCommandPalette => "Open command palette",
            Action::Undo => "Undo",
            Action::Redo => "Redo",
            Action::ClearAllToasts => "Clear all toasts",
        };
        write!(f, "{}", str)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    pub fn find(&self, key: Key, config: &AppConfig) -> Option<&Action> {
        let current_bindings = config.keybindings.clone();
        let action_list = &mut Vec::new();
        for (k, _v) in current_bindings.iter() {
            action_list.push(KeyBindings::str_to_action(current_bindings.clone(), k));
        }
        let binding_action = KeyBindings::key_to_action(current_bindings, key);
        if binding_action.is_some() {
            binding_action
        } else {
            let temp_action = Action::iterator()
                .filter(|action| self.0.contains(action))
                .find(|action| action.keys().contains(&key));
            if temp_action.is_some() && !action_list.contains(&temp_action) {
                temp_action
            } else {
                None
            }
        }
    }

    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    fn from(actions: Vec<Action>) -> Self {
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
            .filter(|(_, actions)| actions.len() > 1)
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

        Self(actions)
    }
}
