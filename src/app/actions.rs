use super::state::KeyBindings;
use crate::{app::AppConfig, inputs::key::Key};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    slice::Iter,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    ChangeCardStatusToActive,
    ChangeCardStatusToCompleted,
    ChangeCardStatusToStale,
    ClearAllToasts,
    Delete,
    DeleteBoard,
    Down,
    Enter,
    GoToMainMenu,
    GoToPreviousUIMode,
    HideUiElement,
    Left,
    MoveCardDown,
    MoveCardLeft,
    MoveCardRight,
    MoveCardUp,
    NewBoard,
    NewCard,
    NextFocus,
    OpenConfigMenu,
    PrvFocus,
    Quit,
    Redo,
    ResetUI,
    Right,
    SaveState,
    StopUserInput,
    TakeUserInput,
    ToggleCommandPalette,
    Undo,
    Up,
}

impl Action {
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 31] = [
            Action::ChangeCardStatusToActive,
            Action::ChangeCardStatusToCompleted,
            Action::ChangeCardStatusToStale,
            Action::ClearAllToasts,
            Action::Delete,
            Action::DeleteBoard,
            Action::Down,
            Action::Enter,
            Action::GoToMainMenu,
            Action::GoToPreviousUIMode,
            Action::HideUiElement,
            Action::Left,
            Action::MoveCardDown,
            Action::MoveCardLeft,
            Action::MoveCardRight,
            Action::MoveCardUp,
            Action::NewBoard,
            Action::NewCard,
            Action::NextFocus,
            Action::OpenConfigMenu,
            Action::PrvFocus,
            Action::Quit,
            Action::Redo,
            Action::ResetUI,
            Action::Right,
            Action::SaveState,
            Action::StopUserInput,
            Action::TakeUserInput,
            Action::ToggleCommandPalette,
            Action::Undo,
            Action::Up,
        ];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::ChangeCardStatusToActive => &[Key::Char('2')],
            Action::ChangeCardStatusToCompleted => &[Key::Char('1')],
            Action::ChangeCardStatusToStale => &[Key::Char('3')],
            Action::ClearAllToasts => &[Key::Char('t')],
            Action::Delete => &[Key::Char('d')],
            Action::DeleteBoard => &[Key::Char('D')],
            Action::Down => &[Key::Down],
            Action::Enter => &[Key::Enter],
            Action::GoToMainMenu => &[Key::Char('m')],
            Action::GoToPreviousUIMode => &[Key::Esc],
            Action::HideUiElement => &[Key::Char('h')],
            Action::Left => &[Key::Left],
            Action::MoveCardDown => &[Key::ShiftDown],
            Action::MoveCardLeft => &[Key::ShiftLeft],
            Action::MoveCardRight => &[Key::ShiftRight],
            Action::MoveCardUp => &[Key::ShiftUp],
            Action::NewBoard => &[Key::Char('b')],
            Action::NewCard => &[Key::Char('n')],
            Action::NextFocus => &[Key::Tab],
            Action::OpenConfigMenu => &[Key::Char('c')],
            Action::PrvFocus => &[Key::BackTab],
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::Redo => &[Key::Ctrl('y')],
            Action::ResetUI => &[Key::Char('r')],
            Action::Right => &[Key::Right],
            Action::SaveState => &[Key::Ctrl('s')],
            Action::StopUserInput => &[Key::Ins],
            Action::TakeUserInput => &[Key::Char('i')],
            Action::ToggleCommandPalette => &[Key::Ctrl('p')],
            Action::Undo => &[Key::Ctrl('z')],
            Action::Up => &[Key::Up],
        }
    }

    pub fn all() -> Vec<Action> {
        Action::iterator().cloned().collect()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::ChangeCardStatusToActive => "Change card status to active",
            Action::ChangeCardStatusToCompleted => "Change card status to completed",
            Action::ChangeCardStatusToStale => "Change card status to stale",
            Action::ClearAllToasts => "Clear all toasts",
            Action::Delete => "Delete focused element",
            Action::DeleteBoard => "Delete Board",
            Action::Down => "Go down",
            Action::Enter => "Accept",
            Action::GoToMainMenu => "Go to main menu",
            Action::GoToPreviousUIMode => "Go to previous mode",
            Action::HideUiElement => "Hide Focused element",
            Action::Left => "Go left",
            Action::MoveCardDown => "Move card down",
            Action::MoveCardLeft => "Move card left",
            Action::MoveCardRight => "Move card right",
            Action::MoveCardUp => "Move card up",
            Action::NewBoard => "Create new board",
            Action::NewCard => "Create new card in current board",
            Action::NextFocus => "Focus next",
            Action::OpenConfigMenu => "configure",
            Action::PrvFocus => "Focus previous",
            Action::Quit => "Quit",
            Action::Redo => "Redo",
            Action::ResetUI => "Reset UI",
            Action::Right => "Go right",
            Action::SaveState => "Save Kanban state",
            Action::StopUserInput => "Stop input mode",
            Action::TakeUserInput => "Enter input mode",
            Action::ToggleCommandPalette => "Open command palette",
            Action::Undo => "Undo",
            Action::Up => "Go up",
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
