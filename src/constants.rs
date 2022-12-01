use tui::style::{Style, Color, Modifier};

pub const FIELD_NOT_SET: &str = "Not Set";
pub const CONFIG_FILE_NAME: &str = "config.json";
pub const CONFIG_DIR_NAME: &str = "rust_kanban";
pub const SAVE_DIR_NAME: &str = "kanban_saves";
pub const SAVE_FILE_NAME: &str = "kanban";
pub const NO_OF_BOARDS_PER_PAGE: u16 = 3;
pub const NO_OF_CARDS_PER_BOARD: u16 = 2;
pub const DEFAULT_BOARD_TITLE_LENGTH: u16 = 20;
pub const DEFAULT_CARD_TITLE_LENGTH: u16 = 20;
pub const APP_TITLE: &str = "Rust 🦀 Kanban";
pub const MIN_TERM_WIDTH: u16 = 110;
pub const MIN_TERM_HEIGHT: u16 = 30;
pub const LIST_SELECTED_SYMBOL: &str = ">> ";

// Style

pub const DEFAULT_STYLE: Style = Style{
    fg: Some(Color::White),
    bg: Some(Color::Reset),
    add_modifier: Modifier::empty(),
    sub_modifier: Modifier::empty(),
};
pub const LIST_SELECT_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::LightMagenta),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const CARD_DATE_DUE_STYLE: Style = Style {
    fg: Some(Color::LightRed),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const CARD_STATUS_STYLE: Style = Style {
    fg: Some(Color::LightGreen),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const FOCUSED_ELEMENT_STYLE: Style = Style {
    fg: Some(Color::LightCyan),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const NON_FOCUSED_ELEMENT_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const HELP_KEY_STYLE: Style = Style {
    fg: Some(Color::LightCyan),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const HELP_DESCRIPTION_STYLE: Style = Style {
    fg: Some(Color::White),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_ERROR_STYLE: Style = Style {
    fg: Some(Color::LightRed),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_DEBUG_STYLE: Style = Style {
    fg: Some(Color::LightGreen),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_WARN_STYLE: Style = Style {
    fg: Some(Color::LightYellow),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_TRACE_STYLE: Style = Style {
    fg: Some(Color::Gray),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const LOG_INFO_STYLE: Style = Style {
    fg: Some(Color::LightCyan),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const PROGRESS_BAR_STYLE: Style = Style {
    fg: Some(Color::LightGreen),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};
pub const ERROR_TEXT_STYLE: Style = Style {
    fg: Some(Color::LightRed),
    bg: Some(Color::Black),
    add_modifier: Modifier::BOLD,
    sub_modifier: Modifier::empty(),
};