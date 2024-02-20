use super::state::KeyBindings;
use crate::inputs::key::Key;
use std::fmt::{self, Display};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter)]
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
    pub fn keys(&self) -> Vec<Key> {
        let default_keybinds = KeyBindings::default();
        // TODO: make the remaining keybinds also customizable
        match self {
            Action::ChangeCardStatusToActive => default_keybinds.change_card_status_to_active,
            Action::ChangeCardStatusToCompleted => default_keybinds.change_card_status_to_completed,
            Action::ChangeCardStatusToStale => default_keybinds.change_card_status_to_stale,
            Action::ClearAllToasts => default_keybinds.clear_all_toasts,
            Action::Delete => default_keybinds.delete_card,
            Action::DeleteBoard => default_keybinds.delete_board,
            Action::Down => default_keybinds.down,
            Action::Enter => vec![Key::Enter],
            Action::GoToMainMenu => default_keybinds.go_to_main_menu,
            Action::GoToPreviousUIMode => vec![Key::Esc],
            Action::HideUiElement => default_keybinds.hide_ui_element,
            Action::Left => default_keybinds.left,
            Action::MoveCardDown => vec![Key::ShiftDown],
            Action::MoveCardLeft => vec![Key::ShiftLeft],
            Action::MoveCardRight => vec![Key::ShiftRight],
            Action::MoveCardUp => vec![Key::ShiftUp],
            Action::NewBoard => default_keybinds.new_board,
            Action::NewCard => default_keybinds.new_card,
            Action::NextFocus => default_keybinds.next_focus,
            Action::OpenConfigMenu => default_keybinds.open_config_menu,
            Action::PrvFocus => default_keybinds.prv_focus,
            Action::Quit => default_keybinds.quit,
            Action::Redo => default_keybinds.redo,
            Action::ResetUI => default_keybinds.reset_ui,
            Action::Right => default_keybinds.right,
            Action::SaveState => default_keybinds.save_state,
            Action::StopUserInput => default_keybinds.stop_user_input,
            Action::TakeUserInput => default_keybinds.take_user_input,
            Action::ToggleCommandPalette => default_keybinds.toggle_command_palette,
            Action::Undo => default_keybinds.undo,
            Action::Up => default_keybinds.up,
        }
    }

    pub fn all() -> Vec<Action> {
        Action::iter().collect()
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
