use std::fmt::{self, Display};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumIter)]
pub enum Action {
    ChangeCardStatusToActive,
    ChangeCardStatusToCompleted,
    ChangeCardStatusToStale,
    ChangeCardPriorityToHigh,
    ChangeCardPriorityToMedium,
    ChangeCardPriorityToLow,
    ClearAllToasts,
    Delete,
    DeleteBoard,
    Down,
    Accept,
    GoToMainMenu,
    GoToPreviousUIModeorCancel,
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
            Action::ChangeCardPriorityToHigh => "Change card priority to high",
            Action::ChangeCardPriorityToMedium => "Change card priority to medium",
            Action::ChangeCardPriorityToLow => "Change card priority to low",
            Action::ClearAllToasts => "Clear all toasts",
            Action::Delete => "Delete focused element",
            Action::DeleteBoard => "Delete Board",
            Action::Down => "Go down",
            Action::Accept => "Accept",
            Action::GoToMainMenu => "Go to main menu",
            Action::GoToPreviousUIModeorCancel => "Go to previous mode or cancel",
            Action::HideUiElement => "Hide Focused element",
            Action::Left => "Go left",
            Action::MoveCardDown => "Move card down",
            Action::MoveCardLeft => "Move card left",
            Action::MoveCardRight => "Move card right",
            Action::MoveCardUp => "Move card up",
            Action::NewBoard => "Create new board",
            Action::NewCard => "Create new card in current board",
            Action::NextFocus => "Focus next",
            Action::OpenConfigMenu => "Configure",
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
