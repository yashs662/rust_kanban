use crate::app::state::UiMode;

pub const REFRESH_TOKEN_FILE_NAME: &str = "kanban_token";
pub const REFRESH_TOKEN_SEPARATOR: &str = "<<>>";
pub const APP_TITLE: &str = "Rust 🦀 Kanban";
pub const CONFIG_DIR_NAME: &str = "rust_kanban";
pub const CONFIG_FILE_NAME: &str = "config.json";
pub const DEFAULT_BOARD_TITLE_LENGTH: u16 = 20;
pub const DEFAULT_CARD_TITLE_LENGTH: u16 = 20;
pub const DEFAULT_CARD_WARNING_DUE_DATE_DAYS: u16 = 3;
pub const MAX_WARNING_DUE_DATE_DAYS: u16 = 30;
pub const MIN_WARNING_DUE_DATE_DAYS: u16 = 1;
pub const DEFAULT_TICKRATE: u16 = 50;
pub const MIN_TICKRATE: u16 = 10;
pub const MAX_TICKRATE: u16 = 1000;
pub const DEFAULT_TOAST_DURATION: u64 = 5;
pub const DEFAULT_UI_MODE: UiMode = UiMode::TitleBodyHelpLog;
pub const ENCRYPTION_KEY_FILE_NAME: &str = "kanban_encryption_key";
pub const FIELD_NA: &str = "N/A";
pub const FIELD_NOT_SET: &str = "Not Set";
pub const HIDDEN_PASSWORD_SYMBOL: char = '•';
pub const IO_EVENT_WAIT_TIME: u64 = 5; // ms
pub const LIST_SELECTED_SYMBOL: &str = ">> ";
pub const LOGIN_FORM_DEFAULT_STATE: ([&str; 2], bool) = (["", ""], false);
pub const MAX_NO_BOARDS_PER_PAGE: u16 = 5;
pub const MAX_NO_CARDS_PER_BOARD: u16 = 4;
pub const MAX_TOASTS_TO_DISPLAY: usize = 5;
pub const MIN_NO_BOARDS_PER_PAGE: u16 = 1;
pub const MIN_NO_CARDS_PER_BOARD: u16 = 1;
pub const MIN_TERM_HEIGHT: u16 = 30;
pub const MIN_TERM_WIDTH: u16 = 110;
pub const MOUSE_OUT_OF_BOUNDS_COORDINATES: (u16, u16) = (9999, 9999);
pub const NEW_BOARD_FORM_DEFAULT_STATE: [&str; 2] = ["", ""];
pub const NEW_CARD_FORM_DEFAULT_STATE: [&str; 3] = ["", "", ""];
pub const NO_OF_BOARDS_PER_PAGE: u16 = 3;
pub const NO_OF_CARDS_PER_BOARD: u16 = 2;
pub const PATTERN_CHANGE_INTERVAL: u64 = 1000; // ms
pub const RANDOM_SEARCH_TERM: &str = "iibnigivirneiivure";
pub const RESET_PASSWORD_FORM_DEFAULT_STATE: ([&str; 4], bool) = (["", "", "", ""], false);
pub const SAMPLE_TEXT: &str = "Sample Text";
pub const SAVE_DIR_NAME: &str = "kanban_saves";
pub const SAVE_FILE_NAME: &str = "kanban";
pub const SAVE_FILE_REGEX: &str = r"^kanban_\d{2}-\d{2}-\d{4}_v\d+.json";
pub const SCREEN_TO_TOAST_WIDTH_RATIO: u16 = 3; // 1/3rd of the screen width
pub const SIGNUP_FORM_DEFAULT_STATE: ([&str; 3], bool) = (["", "", ""], false);
pub const THEME_DIR_NAME: &str = "themes";
pub const THEME_FILE_NAME: &str = "kanban_theme";
pub const TOAST_FADE_IN_TIME: u64 = 200;
pub const TOAST_FADE_OUT_TIME: u64 = 400;
pub const SPINNER_FRAMES: [&str; 7] = [
    "[    ]", "[=   ]", "[==  ]", "[=== ]", "[ ===]", "[  ==]", "[   =]",
];

// Cloud Stuff
pub const MAX_PASSWORD_LENGTH: usize = 32;
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const MIN_TIME_BETWEEN_SENDING_RESET_LINK: u64 = 60; // seconds
pub const SUPABASE_ANON_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImtjcGtiZG9ic3lydGthd29jdWR6Iiwicm9sZSI6ImFub24iLCJpYXQiOjE2ODA5NDkzOTksImV4cCI6MTk5NjUyNTM5OX0.N1jDZ2rFUDw9VtQbGQhBjonI0zy10lfJL-O2rBJlUOs";
pub const SUPABASE_URL: &str = "https://kcpkbdobsyrtkawocudz.supabase.co";

