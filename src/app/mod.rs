use linked_hash_map::LinkedHashMap;
use std::time::{Duration, Instant};
use std::{
    env,
    vec
};
use std::fmt::{
    self,
    Formatter,
    Display
};
use std::path::PathBuf;

use chrono::{NaiveDateTime, NaiveDate};
use log::{
    debug,
    info,
    error,
    warn
};
use serde::{
    Serialize, 
    Deserialize
};
use tui::widgets::{ListState, TableState};

use self::actions::Actions;
use self::state::{
    AppStatus,
    Focus,
    UiMode,
    KeyBindings,
};
use self::kanban::{
    Board,
    Card,
    CardPriority
};
use crate::app::actions::Action;
use crate::app::kanban::CardStatus;
use crate::constants::{
    SAVE_DIR_NAME,
    FIELD_NOT_SET,
    DEFAULT_CARD_WARNING_DUE_DATE_DAYS,
    DEFAULT_TICKRATE,
    DEFAULT_TOAST_DURATION,
    NO_OF_CARDS_PER_BOARD,
    MIN_NO_CARDS_PER_BOARD,
    MAX_NO_CARDS_PER_BOARD,
    NO_OF_BOARDS_PER_PAGE,
    MIN_NO_BOARDS_PER_PAGE,
    MAX_NO_BOARDS_PER_PAGE, IO_EVENT_WAIT_TIME,
};
use crate::inputs::key::Key;
use crate::io::data_handler::{
    write_config,
    get_available_local_savefiles,
    get_config, export_kanban_to_json
};
use crate::io::handler::{refresh_visible_boards_and_cards};
use crate::io::{
    IoEvent,
    data_handler
};
use crate::ui::widgets::{
    ToastWidget,
    ToastType,
    CommandPalette,
    CommandPaletteActions
};

pub mod actions;
pub mod state;
pub mod kanban;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Actions,
    is_loading: bool,
    pub state: AppState,
    pub focus: Focus,
    pub boards: Vec<Board>,
    pub config: AppConfig,
    pub config_item_being_edited: Option<usize>,
    pub visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>>,
    pub command_palette: CommandPalette,
    pub last_io_event_time: Option<Instant>,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let mut state = AppState::default();
        let focus = Focus::NoFocus;
        let boards = vec![];
        let get_config_status = get_config(false);
        let config = if get_config_status.is_err() {
            let config_error_msg = get_config_status.unwrap_err();
            if config_error_msg.contains("Overlapped keybinds found") {
                error!("Keybinds overlap detected. Please check your config file and fix the keybinds. Using default keybinds for now.");
                state.toasts.push(ToastWidget::new(config_error_msg, Duration::from_secs(DEFAULT_TOAST_DURATION) * 3, ToastType::Error));
                state.toasts.push(ToastWidget::new("Please check your config file and fix the keybinds. Using default keybinds for now.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Warning));
                let new_config = get_config(true);
                if new_config.is_err() {
                    error!("Unable to fix keybinds. Please check your config file. Using default config for now.");
                    state.toasts.push(ToastWidget::new(new_config.unwrap_err(), Duration::from_secs(DEFAULT_TOAST_DURATION) * 3, ToastType::Error));
                    state.toasts.push(ToastWidget::new("Using default config for now.".to_owned(),
                        Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Warning));
                    AppConfig::default()
                } else {
                    let mut unwrapped_new_config = new_config.unwrap();
                    unwrapped_new_config.keybindings = KeyBindings::default();
                    unwrapped_new_config
                }
            } else {
                state.toasts.push(ToastWidget::new(config_error_msg, Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Error));
                state.toasts.push(ToastWidget::new("Using default config for now.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Info));
                AppConfig::default()
            }
        } else {
            get_config_status.unwrap()
        };

        Self {
            io_tx,
            actions,
            is_loading,
            state,
            focus,
            boards: boards,
            config: config,
            config_item_being_edited: None,
            visible_boards_and_cards: LinkedHashMap::new(),
            command_palette: CommandPalette::new(),
            last_io_event_time: None,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        // check if we are in a user input mode
        if self.state.app_status == AppStatus::UserInput {
            // append to current user input if key is not enter else change state to Initialized
            if key != Key::Enter && key != Key::Esc {
                if self.config.keybindings.toggle_command_palette.contains(&key) {
                    self.state.app_status = AppStatus::Initialized;
                    self.state.popup_mode = None;
                }
                if self.state.popup_mode.is_some() && self.state.popup_mode.unwrap() == PopupMode::CommandPalette {
                    if key == Key::Up {
                        self.command_palette_up();
                        return AppReturn::Continue;
                    } else if key == Key::Down {
                        self.command_palette_down();
                        return AppReturn::Continue;
                    }
                }
                let mut current_key = key.to_string();
                if key == Key::Char(' ') {
                    current_key = " ".to_string();
                } else if key == Key::Ctrl('n') {
                    current_key = "\n".to_string();
                } else if key == Key::Tab {
                    current_key = "  ".to_string();
                } else if key == Key::Backspace {
                    match self.state.ui_mode {
                        UiMode::NewBoard => {
                            match self.focus {
                                Focus::NewBoardName => {
                                    if self.state.current_cursor_position.is_some() {
                                        let current_cursor_position = self.state.current_cursor_position.unwrap();
                                        if current_cursor_position > 0 {
                                            self.state.new_board_form[0].remove(current_cursor_position - 1);
                                            self.state.current_cursor_position = Some(current_cursor_position - 1);
                                        }
                                    } else {
                                        self.state.new_board_form[0].pop();
                                    }
                                }
                                Focus::NewBoardDescription => {
                                    if self.state.current_cursor_position.is_some() {
                                        let current_cursor_position = self.state.current_cursor_position.unwrap();
                                        if current_cursor_position > 0 {
                                            self.state.new_board_form[1].remove(current_cursor_position - 1);
                                            self.state.current_cursor_position = Some(current_cursor_position - 1);
                                        }
                                    } else {
                                        self.state.new_board_form[1].pop();
                                    }
                                }
                                _ => {}
                            }
                        }
                        UiMode::NewCard => {
                            match self.focus {
                                Focus::NewCardName => {
                                    if self.state.current_cursor_position.is_some() {
                                        let current_cursor_position = self.state.current_cursor_position.unwrap();
                                        if current_cursor_position > 0 {
                                            self.state.new_card_form[0].remove(current_cursor_position - 1);
                                            self.state.current_cursor_position = Some(current_cursor_position - 1);
                                        }
                                    } else {
                                        self.state.new_card_form[0].pop();
                                    }
                                }
                                Focus::NewCardDescription => {
                                    if self.state.current_cursor_position.is_some() {
                                        let current_cursor_position = self.state.current_cursor_position.unwrap();
                                        if current_cursor_position > 0 {
                                            self.state.new_card_form[1].remove(current_cursor_position - 1);
                                            self.state.current_cursor_position = Some(current_cursor_position - 1);
                                        }
                                    } else {
                                        self.state.new_card_form[1].pop();
                                    }
                                }
                                Focus::NewCardDueDate => {
                                    if self.state.current_cursor_position.is_some() {
                                        let current_cursor_position = self.state.current_cursor_position.unwrap();
                                        if current_cursor_position > 0 {
                                            self.state.new_card_form[2].remove(current_cursor_position - 1);
                                            self.state.current_cursor_position = Some(current_cursor_position - 1);
                                        }
                                    } else {
                                        self.state.new_card_form[2].pop();
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            if self.state.current_cursor_position.is_some() {
                                let current_cursor_position = self.state.current_cursor_position.unwrap();
                                if current_cursor_position > 0 {
                                    self.state.current_user_input.remove(current_cursor_position - 1);
                                    self.state.current_cursor_position = Some(current_cursor_position - 1);
                                }
                            } else {
                                self.state.current_user_input.pop();
                            }
                        }
                    };
                    current_key = "".to_string();
                } else if key == Key::Left {
                    match self.state.ui_mode {
                        UiMode::NewBoard => {
                            match self.focus {
                                Focus::NewBoardName => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_board_form[0].len());
                                    } else if self.state.current_cursor_position.unwrap() > 0 {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() - 1);
                                    } else {
                                        self.state.current_cursor_position = Some(0);
                                    }
                                }
                                Focus::NewBoardDescription => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_board_form[1].len());
                                    } else if self.state.current_cursor_position.unwrap() > 0 {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() - 1);
                                    } else {
                                        self.state.current_cursor_position = Some(0);
                                    }
                                }
                                _ => {}
                            }
                        }
                        UiMode::NewCard => {
                            match self.focus {
                                Focus::NewCardName => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[0].len());
                                    } else if self.state.current_cursor_position.unwrap() > 0 {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() - 1);
                                    } else {
                                        self.state.current_cursor_position = Some(0);
                                    }
                                }
                                Focus::NewCardDescription => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[1].len());
                                    } else if self.state.current_cursor_position.unwrap() > 0 {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() - 1);
                                    } else {
                                        self.state.current_cursor_position = Some(0);
                                    }
                                }
                                Focus::NewCardDueDate => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[2].len());
                                    } else if self.state.current_cursor_position.unwrap() > 0 {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() - 1);
                                    } else {
                                        self.state.current_cursor_position = Some(0);
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            if self.state.current_cursor_position.is_none() {
                                self.state.current_cursor_position = Some(self.state.current_user_input.len());
                            } else if self.state.current_cursor_position.unwrap() > 0 {
                                self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() - 1);
                            } else {
                                self.state.current_cursor_position = Some(0);
                            }
                        }
                    };
                    current_key = "".to_string();
                } else if key == Key::Right {
                    match self.state.ui_mode {
                        UiMode::NewBoard => {
                            match self.focus {
                                Focus::NewBoardName => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_board_form[0].len());
                                    } else if self.state.current_cursor_position.unwrap() < self.state.new_board_form[0].len() {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() + 1);
                                    } else {
                                        self.state.current_cursor_position = Some(self.state.new_board_form[0].len());
                                    }
                                }
                                Focus::NewBoardDescription => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_board_form[1].len());
                                    } else if self.state.current_cursor_position.unwrap() < self.state.new_board_form[1].len() {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() + 1);
                                    } else {
                                        self.state.current_cursor_position = Some(self.state.new_board_form[1].len());
                                    }
                                }
                                _ => {}
                            }
                        }
                        UiMode::NewCard => {
                            match self.focus {
                                Focus::NewCardName => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[0].len());
                                    } else if self.state.current_cursor_position.unwrap() < self.state.new_card_form[0].len() {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() + 1);
                                    } else {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[0].len());
                                    }
                                }
                                Focus::NewCardDescription => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[1].len());
                                    } else if self.state.current_cursor_position.unwrap() < self.state.new_card_form[1].len() {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() + 1);
                                    } else {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[1].len());
                                    }
                                }
                                Focus::NewCardDueDate => {
                                    if self.state.current_cursor_position.is_none() {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[2].len());
                                    } else if self.state.current_cursor_position.unwrap() < self.state.new_card_form[2].len() {
                                        self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() + 1);
                                    } else {
                                        self.state.current_cursor_position = Some(self.state.new_card_form[2].len());
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            if self.state.current_cursor_position.is_none() {
                                self.state.current_cursor_position = Some(self.state.current_user_input.len());
                            } else if self.state.current_cursor_position.unwrap() < self.state.current_user_input.len() {
                                self.state.current_cursor_position = Some(self.state.current_cursor_position.unwrap() + 1);
                            } else {
                                self.state.current_cursor_position = Some(self.state.current_user_input.len());
                            }
                        }
                    };
                    current_key = "".to_string();
                } else if key == Key::Home {
                    match self.state.ui_mode {
                        UiMode::NewBoard => {
                            match self.focus {
                                Focus::NewBoardName => {
                                    self.state.current_cursor_position = Some(0);
                                }
                                Focus::NewBoardDescription => {
                                    self.state.current_cursor_position = Some(0);
                                }
                                _ => {}
                            }
                        }
                        UiMode::NewCard => {
                            match self.focus {
                                Focus::NewCardName => {
                                    self.state.current_cursor_position = Some(0);
                                }
                                Focus::NewCardDescription => {
                                    self.state.current_cursor_position = Some(0);
                                }
                                Focus::NewCardDueDate => {
                                    self.state.current_cursor_position = Some(0);
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            self.state.current_cursor_position = Some(0);
                        }
                    };
                    current_key = "".to_string();
                } else if key == Key::End {
                    match self.state.ui_mode {
                        UiMode::NewBoard => {
                            match self.focus {
                                Focus::NewBoardName => {
                                    self.state.current_cursor_position = Some(self.state.new_board_form[0].len());
                                }
                                Focus::NewBoardDescription => {
                                    self.state.current_cursor_position = Some(self.state.new_board_form[1].len());
                                }
                                _ => {}
                            }
                        }
                        UiMode::NewCard => {
                            match self.focus {
                                Focus::NewCardName => {
                                    self.state.current_cursor_position = Some(self.state.new_card_form[0].len());
                                }
                                Focus::NewCardDescription => {
                                    self.state.current_cursor_position = Some(self.state.new_card_form[1].len());
                                }
                                Focus::NewCardDueDate => {
                                    self.state.current_cursor_position = Some(self.state.new_card_form[2].len());
                                }
                                _ => {}
                            }
                        }
                        _ => {
                            self.state.current_cursor_position = Some(self.state.current_user_input.len());
                        }
                    };
                    current_key = "".to_string();
                } else if current_key.starts_with("<") && current_key.ends_with(">") {
                    current_key = current_key[1..current_key.len() - 1].to_string();
                }
                if current_key == "" {
                    return AppReturn::Continue;
                }
                if self.focus == Focus::NewBoardName {
                    let cursor_position = self.state.current_cursor_position.unwrap_or(0);
                    self.state.new_board_form[0].insert(cursor_position, current_key.chars().next().unwrap());
                    self.state.current_cursor_position = Some(cursor_position + 1);
                } else if self.focus == Focus::NewBoardDescription {
                    let cursor_position = self.state.current_cursor_position.unwrap_or(0);
                    self.state.new_board_form[1].insert(cursor_position, current_key.chars().next().unwrap());
                    self.state.current_cursor_position = Some(cursor_position + 1);
                } else if self.focus == Focus::NewCardName {
                    let cursor_position = self.state.current_cursor_position.unwrap_or(0);
                    self.state.new_card_form[0].insert(cursor_position, current_key.chars().next().unwrap());
                    self.state.current_cursor_position = Some(cursor_position + 1);
                } else if self.focus == Focus::NewCardDescription {
                    let current_cursor_position = self.state.current_cursor_position.unwrap_or(0);
                    self.state.new_card_form[1].insert(current_cursor_position, current_key.chars().next().unwrap());
                    self.state.current_cursor_position = Some(current_cursor_position + 1);
                } else if self.focus == Focus::NewCardDueDate {
                    let current_cursor_position = self.state.current_cursor_position.unwrap_or(0);
                    self.state.new_card_form[2].insert(current_cursor_position, current_key.chars().next().unwrap());
                    self.state.current_cursor_position = Some(current_cursor_position + 1);
                } else {
                    let current_cursor_position = self.state.current_cursor_position.unwrap_or(0);
                    self.state.current_user_input.insert(current_cursor_position, current_key.chars().next().unwrap());
                    self.state.current_cursor_position = Some(current_cursor_position + 1);
                }
            } else if key == Key::Esc {
                if self.focus == Focus::NewBoardName {
                    self.state.new_board_form[0] = "".to_string();
                } else if self.focus == Focus::NewBoardDescription {
                    self.state.new_board_form[1] = "".to_string();
                } else if self.focus == Focus::NewCardName {
                    self.state.new_card_form[0] = "".to_string();
                } else if self.focus == Focus::NewCardDescription {
                    self.state.new_card_form[1] = "".to_string();
                } else if self.focus == Focus::NewCardDueDate {
                    self.state.new_card_form[2] = "".to_string();
                } else {
                    self.state.current_user_input = "".to_string();
                }
                if self.state.popup_mode.is_some() && self.state.popup_mode.unwrap() == PopupMode::CommandPalette {
                    self.state.popup_mode = None;
                }
                self.state.app_status = AppStatus::Initialized;
                self.state.current_cursor_position = None;
                info!("Exiting user input mode");
            } else {
                if key == Key::Enter && self.state.popup_mode.is_some() && self.state.popup_mode.unwrap() == PopupMode::CommandPalette {
                    if self.state.command_palette_list_state.selected().is_some() {
                        let command_index = self.state.command_palette_list_state.selected().unwrap();
                        let command = if self.command_palette.search_results.is_some() {
                            self.command_palette.search_results.as_ref().unwrap().get(command_index)
                        } else {
                            None
                        };
                        if command.is_some() {
                            match command.unwrap() {
                                CommandPaletteActions::ExportToJSON => {
                                    let export_result = export_kanban_to_json(&self.boards);
                                    if export_result.is_ok() {
                                        let msg = format!("Exported JSON to {}", export_result.unwrap());
                                        self.send_info_toast(&msg, None);
                                        info!("{}", msg);
                                    } else {
                                        let msg = format!("Failed to export JSON: {}", export_result.unwrap_err());
                                        self.send_error_toast(&msg, None);
                                        error!("{}", msg);
                                    }
                                    self.state.popup_mode = None;
                                },
                                CommandPaletteActions::Quit => {
                                    info!("Quitting");
                                    return AppReturn::Exit;
                                },
                                CommandPaletteActions::OpenConfigMenu => {
                                    self.state.popup_mode = None;
                                    self.state.ui_mode = UiMode::ConfigMenu;
                                    self.state.config_state.select(Some(0));
                                    self.focus = Focus::ConfigTable;
                                },
                                CommandPaletteActions::OpenMainMenu => {
                                    self.state.popup_mode = None;
                                    self.state.ui_mode = UiMode::MainMenu;
                                    self.state.main_menu_state.select(Some(0));
                                    self.focus = Focus::MainMenu;
                                },
                                CommandPaletteActions::OpenHelpMenu => {
                                    self.state.popup_mode = None;
                                    self.state.ui_mode = UiMode::HelpMenu;
                                    self.state.help_state.select(Some(0));
                                    self.focus = Focus::Body;
                                },
                                CommandPaletteActions::SaveKanbanState => {
                                    self.state.popup_mode = None;
                                    self.dispatch(IoEvent::SaveLocalData).await;
                                },
                                CommandPaletteActions::NewBoard => {
                                    if UiMode::view_modes().contains(&self.state.ui_mode) {
                                        self.state.popup_mode = None;
                                        self.state.ui_mode = UiMode::NewBoard;
                                        self.focus = Focus::NewBoardName;
                                    } else {
                                        self.state.popup_mode = None;
                                        self.send_error_toast("Cannot create a new board in this view", None);
                                    }
                                },
                                CommandPaletteActions::NewCard => {
                                    if UiMode::view_modes().contains(&self.state.ui_mode) {
                                        if self.state.current_board_id.is_none() {
                                            self.send_error_toast("No board Selected / Available", None);
                                            self.state.popup_mode = None;
                                            self.state.app_status = AppStatus::Initialized;
                                            return AppReturn::Continue;
                                        }
                                        self.state.popup_mode = None;
                                        self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                        self.state.ui_mode = UiMode::NewCard;
                                        self.focus = Focus::NewCardName;
                                    } else {
                                        self.state.popup_mode = None;
                                        self.send_error_toast("Cannot create a new card in this view", None);
                                    }
                                },
                                CommandPaletteActions::ResetUI => {
                                    self.state.popup_mode = None;
                                    let default_view = self.config.default_view.clone();
                                    self.state.ui_mode = default_view;
                                },
                                CommandPaletteActions::ChangeUIMode => {
                                    self.state.popup_mode = Some(PopupMode::ChangeUIMode);
                                },
                                CommandPaletteActions::ChangeCurrentCardStatus => {
                                    if UiMode::view_modes().contains(&self.state.ui_mode) {
                                        if let Some(current_board_id) = self.state.current_board_id {
                                            if let Some(current_board) = self.boards.iter_mut().find(|b| b.id == current_board_id) {
                                                if let Some(current_card_id) = self.state.current_card_id {
                                                    if let Some(_) = current_board.cards.iter_mut().find(|c| c.id == current_card_id) {
                                                        self.state.popup_mode = Some(PopupMode::ChangeCurrentCardStatus);
                                                        self.state.app_status = AppStatus::Initialized;
                                                        self.state.card_status_selector_state.select(Some(0));
                                                        return AppReturn::Continue;
                                                    }
                                                }
                                            }
                                        }
                                        self.send_error_toast("Could not find current card", None);
                                    } else {
                                        self.state.popup_mode = None;
                                        self.send_error_toast("Cannot change card status in this view", None);
                                    }
                                },
                                CommandPaletteActions::LoadASave => {
                                    self.state.popup_mode = None;
                                    self.state.ui_mode = UiMode::LoadSave;
                                },
                                CommandPaletteActions::DebugMenu => {
                                    self.state.debug_menu_toggled = !self.state.debug_menu_toggled;
                                    self.state.popup_mode = None;
                                }
                            }
                            self.state.current_user_input = "".to_string();
                        }
                    }
                }
                self.state.app_status = AppStatus::Initialized;
                self.state.current_cursor_position = None;
                info!("Exiting user input mode");
            }
            return AppReturn::Continue;
        } else if self.state.app_status == AppStatus::KeyBindMode {
            if key != Key::Enter && key != Key::Esc {
                if self.state.edited_keybinding.is_some() {
                    let keybinding = self.state.edited_keybinding.as_mut().unwrap();
                    keybinding.push(key);
                } else {
                    self.state.edited_keybinding = Some(vec![key]);
                }
            } else if key == Key::Enter {
                self.state.app_status = AppStatus::Initialized;
                info!("Exiting user keybind input mode");
            } else if key == Key::Esc {
                self.state.app_status = AppStatus::Initialized;
                self.state.edited_keybinding = None;
                info!("Exiting user keybind input mode");
            }
            AppReturn::Continue
        } else {
            if let Some(action) = self.actions.find(key) {
                // check if the current focus is in the available focus list for the current ui mode if not assign it to the first
                if UiMode::get_available_targets(&self.state.ui_mode)
                    .iter()
                    .find(|x| *x == &self.focus)
                    .is_none() {
                        self.focus = UiMode::get_available_targets(&self.state.ui_mode)[0];
                }
                match action {
                    Action::Quit => {
                        let get_config_status = get_config(false);
                        let config = if get_config_status.is_err() {
                            debug!("Error getting config: {}", get_config_status.unwrap_err());
                            AppConfig::default()
                        } else {
                            get_config_status.unwrap()
                        };
                        if config.save_on_exit {
                            self.dispatch(IoEvent::AutoSave).await;
                        }
                        AppReturn::Exit
                    }
                    Action::NextFocus => {
                        let current_focus = self.focus.clone();
                        let next_focus = self.focus.next(&UiMode::get_available_targets(&self.state.ui_mode));
                        // check if the next focus is the same as the current focus or NoFocus if so set back to the first focus
                        if next_focus == current_focus || next_focus == Focus::NoFocus {
                            self.focus = current_focus;
                        } else {
                            self.focus = next_focus;
                        }
                        AppReturn::Continue
                    }
                    Action::PrvFocus => {
                        let current_focus = self.focus.clone();
                        let next_focus = self.focus.prev(&UiMode::get_available_targets(&self.state.ui_mode));
                        // check if the next focus is the same as the current focus or NoFocus if so set back to the first focus
                        if next_focus == current_focus || next_focus == Focus::NoFocus {
                            self.focus = current_focus;
                        } else {
                            self.focus = next_focus;
                        }
                        AppReturn::Continue
                    }
                    Action::ResetUI => {
                        let new_ui_mode = self.config.default_view.clone();
                        let available_focus_targets = UiMode::get_available_targets(&new_ui_mode);
                        // check if focus is still available in the new ui_mode if not set it to the first available tab
                        if !available_focus_targets.contains(&self.focus) {
                            // check if available focus targets is empty
                            if available_focus_targets.is_empty() {
                                self.focus = Focus::NoFocus;
                            } else {
                                self.focus = available_focus_targets[0];
                            }
                        }
                        self.state.ui_mode = new_ui_mode;
                        self.state.popup_mode = None;
                        AppReturn::Continue
                    }
                    Action::OpenConfigMenu => {
                        match self.state.ui_mode {
                            UiMode::ConfigMenu => {
                                // check if the prv ui mode is the same as the current ui mode
                                if self.state.prev_ui_mode.is_some() && self.state.prev_ui_mode.as_ref().unwrap() == &UiMode::ConfigMenu {
                                    self.state.ui_mode = self.config.default_view.clone();
                                } else {
                                    self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                }
                            },
                            _ => {
                                self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                self.state.ui_mode = UiMode::ConfigMenu;
                                if self.state.config_state.selected().is_none() {
                                    self.config_next()
                                }
                                let available_focus_targets = self.state.ui_mode.get_available_targets();
                                if !available_focus_targets.contains(&self.focus) {
                                    // check if available focus targets is empty
                                    if available_focus_targets.is_empty() {
                                        self.focus = Focus::NoFocus;
                                    } else {
                                        self.focus = available_focus_targets[0];
                                    }
                                }
                            }
                        }
                        if self.state.popup_mode.is_some() {
                            self.state.popup_mode = None;
                        }
                        AppReturn::Continue
                    }
                    Action::Up => {
                        if self.state.popup_mode.is_some() {
                            if self.state.popup_mode.as_ref().unwrap() == &PopupMode::ChangeUIMode {
                                self.select_default_view_prev();
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::ChangeCurrentCardStatus {
                                self.select_current_card_status_prev();
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::SelectDefaultView{
                                self.select_default_view_prev();
                            }
                            return AppReturn::Continue;
                        }
                        match self.state.ui_mode {
                            UiMode::ConfigMenu => {
                                if self.focus == Focus::ConfigTable {
                                    self.config_previous();
                                } else {
                                    let next_focus_key = self.config.keybindings.next_focus.get(0).unwrap_or_else(|| &Key::Tab);
                                    let prev_focus_key = self.config.keybindings.prev_focus.get(0).unwrap_or_else(|| &Key::BackTab);
                                    self.send_warning_toast(&format!(
                                        "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                        next_focus_key, prev_focus_key), None);
                                }
                            }
                            UiMode::MainMenu => {
                                if self.focus == Focus::MainMenu {
                                    self.main_menu_previous();
                                } else if self.focus == Focus::MainMenuHelp {
                                    self.help_prev();
                                } else {
                                    let next_focus_key = self.config.keybindings.next_focus.get(0).unwrap_or_else(|| &Key::Tab);
                                    let prev_focus_key = self.config.keybindings.prev_focus.get(0).unwrap_or_else(|| &Key::BackTab);
                                    self.send_warning_toast(&format!(
                                        "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                        next_focus_key, prev_focus_key), None);
                                }
                            }
                            UiMode::LoadSave => {
                                self.load_save_previous();
                                self.dispatch(IoEvent::LoadPreview).await;
                            }
                            UiMode::EditKeybindings => {
                                self.edit_keybindings_prev();
                            }
                            _ => {
                                if self.focus == Focus::Body {
                                    self.dispatch(IoEvent::GoUp).await;
                                } else if self.focus == Focus::Help {
                                    self.help_prev();
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::Down => {
                        if self.state.popup_mode.is_some() {
                            if self.state.popup_mode.as_ref().unwrap() == &PopupMode::ChangeUIMode {
                                self.select_default_view_next();
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::ChangeCurrentCardStatus {
                                self.select_current_card_status_next();
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::SelectDefaultView{
                                self.select_default_view_next();
                            }
                            return AppReturn::Continue;
                        }
                        match self.state.ui_mode {
                            UiMode::ConfigMenu => {
                                if self.focus == Focus::ConfigTable {
                                    self.config_next();
                                } else {
                                    let next_focus_key = self.config.keybindings.next_focus.get(0).unwrap_or_else(|| &Key::Tab);
                                    let prev_focus_key = self.config.keybindings.prev_focus.get(0).unwrap_or_else(|| &Key::BackTab);
                                    self.send_warning_toast(&format!(
                                        "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                        next_focus_key, prev_focus_key), None);
                                }
                            },
                            UiMode::MainMenu => {
                                if self.focus == Focus::MainMenu {
                                    self.main_menu_next();
                                } else if self.focus == Focus::MainMenuHelp {
                                    self.help_next();
                                } else {
                                    let next_focus_key = self.config.keybindings.next_focus.get(0).unwrap_or_else(|| &Key::Tab);
                                    let prev_focus_key = self.config.keybindings.prev_focus.get(0).unwrap_or_else(|| &Key::BackTab);
                                    self.send_warning_toast(&format!(
                                        "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                        next_focus_key, prev_focus_key), None);
                                }
                            },
                            UiMode::LoadSave => {
                                self.load_save_next();
                                self.dispatch(IoEvent::LoadPreview).await;
                            },
                            UiMode::EditKeybindings => {
                                self.edit_keybindings_next();
                            },
                            _ => {
                                if self.focus == Focus::Body {
                                    self.dispatch(IoEvent::GoDown).await;
                                } else if self.focus == Focus::Help {
                                    self.help_next();
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::Right => {
                        if self.focus == Focus::Body && self.state.ui_mode != UiMode::LoadSave{
                            self.dispatch(IoEvent::GoRight).await;
                        }
                        AppReturn::Continue
                    }
                    Action::Left => {
                        if self.focus == Focus::Body && self.state.ui_mode != UiMode::LoadSave{
                            self.dispatch(IoEvent::GoLeft).await;
                        }
                        AppReturn::Continue
                    }
                    Action::TakeUserInput => {
                        match self.state.ui_mode {
                            UiMode::NewBoard | UiMode::NewCard => {
                                self.state.app_status = AppStatus::UserInput;
                                info!("Taking user input");
                            },
                            _ => {
                                if self.state.popup_mode.is_some() {
                                    if self.state.popup_mode.unwrap() == PopupMode::EditGeneralConfig {
                                        self.state.app_status = AppStatus::UserInput;
                                        info!("Taking user input");
                                    } else if self.state.popup_mode.unwrap() == PopupMode::EditSpecificKeyBinding {
                                        self.state.app_status = AppStatus::KeyBindMode;
                                        info!("Taking user keybind input");
                                    }
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::GoToPreviousUIMode => {
                        if self.state.popup_mode.is_some() {
                            if self.state.popup_mode.unwrap() == PopupMode::EditGeneralConfig {
                                self.state.ui_mode = UiMode::ConfigMenu;
                                if self.state.config_state.selected().is_none() {
                                    self.config_next()
                                }
                                self.state.current_user_input = String::new();
                                self.state.current_cursor_position = None;
                            } else if self.state.popup_mode.unwrap() == PopupMode::EditSpecificKeyBinding {
                                self.state.ui_mode = UiMode::EditKeybindings;
                                if self.state.edit_keybindings_state.selected().is_none() {
                                    self.edit_keybindings_next();
                                }
                            }
                            self.state.popup_mode = None;
                            if self.state.app_status == AppStatus::UserInput {
                                self.state.app_status = AppStatus::Initialized;
                            }
                            return AppReturn::Continue;
                        }
                        match self.state.ui_mode {
                            UiMode::ConfigMenu => {
                                if self.state.prev_ui_mode == Some(UiMode::ConfigMenu) {
                                    self.state.prev_ui_mode = None;
                                    self.state.ui_mode = self.config.default_view.clone();
                                } else {
                                    self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                    self.state.prev_ui_mode = Some(UiMode::ConfigMenu);
                                }
                                AppReturn::Continue
                            }
                            UiMode::MainMenu => {
                                AppReturn::Exit
                            }
                            UiMode::EditKeybindings => {
                                self.state.ui_mode = UiMode::ConfigMenu;
                                if self.state.config_state.selected().is_none() {
                                    self.config_next()
                                }
                                AppReturn::Continue
                            }
                            _ => {
                                if self.state.ui_mode == UiMode::LoadSave {
                                    self.state.load_save_state = ListState::default();
                                }
                                // check if previous ui mode is the same as the current ui mode
                                if self.state.prev_ui_mode == Some(self.state.ui_mode.clone()) {
                                    self.state.ui_mode = UiMode::MainMenu;
                                    if self.state.main_menu_state.selected().is_none() {
                                        self.main_menu_next();
                                    }
                                } else {
                                    self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &UiMode::MainMenu).clone();
                                    if self.state.main_menu_state.selected().is_none() {
                                        self.main_menu_next();
                                    }
                                }
                                AppReturn::Continue
                            }
                        }
                    }
                    Action::Enter => {
                        if self.state.popup_mode.is_some() {
                            if self.state.popup_mode.as_ref().unwrap() == &PopupMode::ChangeUIMode {
                                let current_index = self.state.default_view_state.selected().unwrap_or(0);
                                // UiMode::all() has strings map all of them to UiMode using UiMode::from_string which returns an option<UiMode>
                                let all_ui_modes = UiMode::all()
                                    .iter()
                                    .map(|s| UiMode::from_string(s))
                                    .filter(|s| s.is_some())
                                    .map(|s| s.unwrap())
                                    .collect::<Vec<UiMode>>();

                                // make sure index is in bounds
                                let current_index = if current_index >= all_ui_modes.len() {
                                    all_ui_modes.len() - 1
                                } else {
                                    current_index
                                };
                                let selected_ui_mode = all_ui_modes[current_index].clone();
                                self.state.ui_mode = selected_ui_mode;
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::ChangeCurrentCardStatus {
                                let current_index = self.state.card_status_selector_state.selected().unwrap_or(0);
                                let all_statuses = CardStatus::all();

                                let current_index = if current_index >= all_statuses.len() {
                                    all_statuses.len() - 1
                                } else {
                                    current_index
                                };
                                let selected_status = all_statuses[current_index].clone();
                                // find current board from self.boards
                                if let Some(current_board_id) = self.state.current_board_id {
                                    if let Some(current_board) = self.boards.iter_mut().find(|b| b.id == current_board_id) {
                                        if let Some(current_card_id) = self.state.current_card_id {
                                            if let Some(current_card) = current_board.cards.iter_mut().find(|c| c.id == current_card_id) {
                                                current_card.card_status = selected_status;
                                                self.state.popup_mode = None;
                                                return AppReturn::Continue;
                                            }
                                        }
                                    }
                                }
                                self.send_error_toast("Error Could not find current card", None);
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::EditGeneralConfig {
                                let config_item_index = self.state.config_state.selected().unwrap_or(0);
                                let config_item_list = AppConfig::to_list(&self.config);
                                let config_item = config_item_list[config_item_index].clone();
                                // key is the second item in the list
                                let default_key = String::from("");
                                let config_item_key = config_item.get(0).unwrap_or_else(|| &default_key);
                                let new_value = self.state.current_user_input.clone();
                                // if new value is not empty update the config
                                if !new_value.is_empty() {
                                    let config_string = format!("{}: {}", config_item_key, new_value);
                                    let app_config = AppConfig::edit_with_string(&config_string, self);
                                    self.config = app_config.clone();
                                    let write_config_status = write_config(&app_config);
                                    if write_config_status.is_err() {
                                        error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                        self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                    } else {
                                        self.send_info_toast("Config updated Successfully", None);
                                    }

                                    // reset everything
                                    self.state.config_state.select(Some(0));
                                    self.config_item_being_edited = None;
                                    self.state.current_user_input = String::new();
                                    self.state.ui_mode = UiMode::ConfigMenu;
                                    if self.state.config_state.selected().is_none() {
                                        self.config_next();
                                    }
                                }
                                self.state.config_state.select(Some(0));
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::EditSpecificKeyBinding {
                                if self.state.edited_keybinding.is_some() {
                                    let selected = self.state.edit_keybindings_state.selected().unwrap();
                                    if selected < self.config.keybindings.iter().count() {
                                        let result = self.config.edit_keybinding(selected, self.state.edited_keybinding.clone().unwrap_or_else(|| vec![]));
                                        if result.is_err() {
                                            self.send_error_toast(&result.unwrap_err(), None);
                                        } else {
                                            let mut key_list = vec![];
                                            for (k, v) in self.config.keybindings.iter() {
                                                key_list.push((k, v));
                                            }
                                            let (key, _) = key_list[selected];
                                            let key_string = key.to_string();
                                            let value = self.state.edited_keybinding.clone().unwrap_or_else(|| vec![]);
                                            let value = value.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(" ");
                                            self.send_info_toast(&format!("Keybind for {} updated to {}", key_string, value), None);
                                        }
                                    } else {
                                        error!("Selected keybind with id {} not found", selected);
                                        self.send_error_toast("Selected keybind not found", None);
                                        self.state.edited_keybinding = None;
                                        self.state.edit_keybindings_state.select(None);
                                    }
                                    self.state.ui_mode = UiMode::EditKeybindings;
                                    if self.state.edit_keybindings_state.selected().is_none() {
                                        self.edit_keybindings_next()
                                    }
                                    self.state.edited_keybinding = None;
                                    let write_config_status = write_config(&self.config);
                                    if write_config_status.is_err() {
                                        error!("Error writing config: {}", write_config_status.clone().unwrap_err());
                                        self.send_error_toast(&write_config_status.unwrap_err(), None);
                                    }
                                } else {
                                    self.state.ui_mode = UiMode::EditKeybindings;
                                    if self.state.edit_keybindings_state.selected().is_none() {
                                        self.edit_keybindings_next()
                                    }
                                }
                                self.keybind_list_maker();
                            } else if self.state.popup_mode.as_ref().unwrap() == &PopupMode::SelectDefaultView{
                                let all_ui_modes = UiMode::all();
                                let current_selected_mode = self.state.default_view_state.selected().unwrap_or(0);
                                if current_selected_mode < all_ui_modes.len() {
                                    let selected_mode = &all_ui_modes[current_selected_mode];
                                    self.config.default_view = UiMode::from_string(&selected_mode).unwrap_or(UiMode::MainMenu);
                                    self.state.prev_ui_mode = Some(self.config.default_view.clone());
                                    let config_string = format!("{}: {}", "Select Default View", selected_mode);
                                    let app_config = AppConfig::edit_with_string(&config_string, self);
                                    self.config = app_config.clone();
                                    let write_config_status = write_config(&app_config);
                                    if write_config_status.is_err() {
                                        error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                        self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                    } else {
                                        self.send_info_toast("Config updated Successfully", None);
                                    }

                                    // reset everything
                                    self.state.default_view_state.select(Some(0));
                                    self.state.ui_mode = UiMode::ConfigMenu;
                                    if self.state.config_state.selected().is_none() {
                                        self.config_next();
                                    }
                                    if self.state.popup_mode.is_some() {
                                        self.state.popup_mode = None;
                                    }
                                } else {
                                    debug!("Selected mode {} is not in the list of all UI modes", current_selected_mode);
                                }
                            }
                            self.state.popup_mode = None;
                            return AppReturn::Continue;
                        }
                        match self.state.ui_mode {
                            UiMode::ConfigMenu => {
                                if self.focus == Focus::SubmitButton {
                                    self.config = AppConfig::default();
                                    self.focus = Focus::ConfigTable;
                                    self.state.config_state.select(Some(0));
                                    let write_config_status = write_config(&self.config);
                                    if write_config_status.is_err() {
                                        error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                        self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                    } else {
                                        warn!("Reset Config and Keybinds to default");
                                        self.send_warning_toast("Reset Config and Keybinds to default", None);
                                    }
                                    self.keybind_list_maker();
                                    return AppReturn::Continue;
                                } else if self.focus == Focus::ExtraFocus {
                                    // make a copy of the keybinds and reset only config and then write the config
                                    let keybinds = self.config.keybindings.clone();
                                    self.config = AppConfig::default();
                                    self.config.keybindings = keybinds;
                                    self.focus = Focus::ConfigTable;
                                    self.state.config_state.select(Some(0));
                                    let write_config_status = write_config(&self.config);
                                    if write_config_status.is_err() {
                                        error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                        self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                    } else {
                                        warn!("Reset Config to default");
                                        self.send_warning_toast("Reset Config to default", None);
                                    }
                                    return AppReturn::Continue;
                                }
                                self.config_item_being_edited = Some(self.state.config_state.selected().unwrap_or(0));
                                // check if the config_item_being_edited index is in the AppConfig list and the value in the list is Edit Keybindings
                                let app_config_list = &self.config.to_list();
                                if self.config_item_being_edited.unwrap_or(0) < app_config_list.len() {
                                    let default_config_item = String::from("");
                                    let config_item = &app_config_list[self.config_item_being_edited.unwrap_or(0)].first().unwrap_or_else(|| &default_config_item);
                                    if *config_item == "Edit Keybindings" {
                                        self.state.ui_mode = UiMode::EditKeybindings;
                                        if self.state.edit_keybindings_state.selected().is_none() {
                                            self.edit_keybindings_next();
                                        }
                                    } else if *config_item == "Select Default View" {
                                        if self.state.default_view_state.selected().is_none() {
                                            self.select_default_view_next();
                                        }
                                        self.state.popup_mode = Some(PopupMode::SelectDefaultView);
                                    } else if *config_item == "Auto Save on Exit" {
                                        let save_on_exit = self.config.save_on_exit;
                                        self.config.save_on_exit = !save_on_exit;
                                        let config_string = format!("{}: {}", "Auto Save on Exit", self.config.save_on_exit);
                                        let app_config = AppConfig::edit_with_string(&config_string, self);
                                        self.config = app_config.clone();
                                        let write_config_status = write_config(&app_config);
                                        if write_config_status.is_err() {
                                            error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                            self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                        } else {
                                            self.send_info_toast("Config updated Successfully", None);
                                        }
                                    } else if *config_item == "Auto Load Last Save" {
                                        let always_load_last_save = self.config.always_load_last_save;
                                        self.config.always_load_last_save = !always_load_last_save;
                                        let config_string = format!("{}: {}", "Auto Load Last Save", self.config.always_load_last_save);
                                        let app_config = AppConfig::edit_with_string(&config_string, self);
                                        self.config = app_config.clone();
                                        let write_config_status = write_config(&app_config);
                                        if write_config_status.is_err() {
                                            error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                            self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                        } else {
                                            self.send_info_toast("Config updated Successfully", None);
                                        }
                                    } else if *config_item == "Disable Scrollbars" {
                                        let disable_scrollbars = self.config.disable_scrollbars;
                                        self.config.disable_scrollbars = !disable_scrollbars;
                                        let config_string = format!("{}: {}", "Disable Scrollbars", self.config.disable_scrollbars);
                                        let app_config = AppConfig::edit_with_string(&config_string, self);
                                        self.config = app_config.clone();
                                        let write_config_status = write_config(&app_config);
                                        if write_config_status.is_err() {
                                            error!("Error writing config file: {}", write_config_status.clone().unwrap_err());
                                            self.send_error_toast(&format!("Error writing config file: {}", write_config_status.unwrap_err()), None);
                                        } else {
                                            self.send_info_toast("Config updated Successfully", None);
                                        }
                                    } else {
                                        self.state.popup_mode = Some(PopupMode::EditGeneralConfig);
                                    }
                                } else {
                                    debug!("Config item being edited {} is not in the AppConfig list", self.config_item_being_edited.unwrap_or(0));
                                }
                                AppReturn::Continue
                            }
                            UiMode::MainMenu => {
                                match self.focus {
                                    Focus::MainMenu => {
                                        let selected = self.state.main_menu_state.selected().unwrap_or(0);
                                        let selected_item = MainMenu::from_index(selected);
                                        self.state.main_menu_state.select(Some(0));
                                        match selected_item {
                                            MainMenuItem::Quit => {
                                                AppReturn::Exit
                                            }
                                            MainMenuItem::Config => {
                                                self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                                self.state.ui_mode = UiMode::ConfigMenu;
                                                if self.state.config_state.selected().is_none() {
                                                    self.config_next();
                                                }
                                                AppReturn::Continue
                                            }
                                            MainMenuItem::View => {
                                                self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                                self.state.ui_mode = self.config.default_view.clone();
                                                AppReturn::Continue
                                            }
                                            MainMenuItem::Help => {
                                                self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                                self.state.ui_mode = UiMode::HelpMenu;
                                                AppReturn::Continue
                                            }
                                            MainMenuItem::LoadSave => {
                                                self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                                self.state.ui_mode = UiMode::LoadSave;
                                                AppReturn::Continue
                                            }
                                        }
                                    },
                                    Focus::MainMenuHelp => {
                                        self.state.ui_mode = UiMode::HelpMenu;
                                        AppReturn::Continue
                                    },
                                    Focus::Log => {
                                        self.state.ui_mode = UiMode::LogsOnly;
                                        AppReturn::Continue
                                    },
                                    _ => {
                                        AppReturn::Continue
                                    }
                                }
                            }
                            UiMode::NewBoard => {
                                if self.focus == Focus::SubmitButton {
                                    // check if self.state.new_board_form[0] is not empty or is not the same as any of the existing boards
                                    let new_board_name = self.state.new_board_form[0].clone();
                                    let new_board_description = self.state.new_board_form[1].clone();
                                    let mut same_name_exists = false;
                                    for board in self.boards.iter() {
                                        if board.name == new_board_name {
                                            same_name_exists = true;
                                            break;
                                        }
                                    }
                                    if !new_board_name.is_empty() && !same_name_exists {
                                        let new_board = Board::new(new_board_name, new_board_description);
                                        self.boards.push(new_board.clone());
                                        self.state.current_board_id = Some(new_board.id);
                                        self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                    } else {
                                        warn!("New board name is empty or already exists");
                                        self.send_warning_toast("New board name is empty or already exists", None);
                                    }
                                    self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                    if let Some(previous_focus) = &self.state.previous_focus {
                                        self.focus = previous_focus.clone();
                                    }
                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                }
                                AppReturn::Continue
                            }
                            UiMode::NewCard => {
                                if self.focus == Focus::SubmitButton {
                                    // check if self.state.new_card_form[0] is not empty or is not the same as any of the existing cards
                                    let new_card_name = self.state.new_card_form[0].clone();
                                    let new_card_description = self.state.new_card_form[1].clone();
                                    let new_card_due_date = self.state.new_card_form[2].clone();
                                    let mut same_name_exists = false;
                                    let current_board_id = self.state.current_board_id.unwrap_or(0);
                                    let current_board = self.boards.iter().find(|board| board.id == current_board_id);
                                    if let Some(current_board) = current_board {
                                        for card in current_board.cards.iter() {
                                            if card.name == new_card_name {
                                                same_name_exists = true;
                                                break;
                                            }
                                        }
                                    } else {
                                        error!("Current board not found");
                                        self.send_error_toast("Current board not found", None);
                                        self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                        return AppReturn::Continue;
                                    }
                                    // check if due date is empty or is a valid date
                                    let due_date = if new_card_due_date.is_empty() {
                                        Some(FIELD_NOT_SET.to_string())
                                    } else {
                                        match NaiveDateTime::parse_from_str(&new_card_due_date, "%d/%m/%Y-%H:%M:%S") {
                                            Ok(due_date) => {
                                                debug!("Due date: {}", due_date);
                                                let new_due = due_date.to_string();
                                                // the date is in the format 2023-01-20 14:10:00 convert it to 20/01/2023-14:10:00
                                                let new_due = new_due.replace("-", "/");
                                                let new_due = new_due.replace(" ", "-");
                                                debug!("New due date: {}", new_due);
                                                Some(new_due)
                                            },
                                            Err(_) => {
                                                // cehck if the user has not put the time if not put the default time
                                                match NaiveDate::parse_from_str(&new_card_due_date, "%d/%m/%Y") {
                                                    Ok(due_date) => {
                                                        debug!("Due date: {}", due_date);
                                                        let new_due = due_date.to_string();
                                                        // the date is in the format 2023-01-20 14:10:00 convert it to 20/01/2023-14:10:00
                                                        let new_due = new_due.replace("-", "/");
                                                        let new_due = new_due.replace(" ", "-");
                                                        let new_due = format!("{}-{}", new_due, "12:00:00");
                                                        debug!("New due date: {}", new_due);
                                                        Some(new_due)
                                                    },
                                                    Err(e) => {
                                                        debug!("Invalid due date: {}", e);
                                                        debug!("Due date: {}", new_card_due_date);
                                                        None
                                                    }
                                                }
                                            }
                                        }
                                    };
                                    if due_date.is_none() {
                                        warn!("Invalid due date");
                                        self.send_warning_toast("Invalid due date", None);
                                        self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                        return AppReturn::Continue;
                                    }
                                    if !new_card_name.is_empty() && !same_name_exists {
                                        let new_card = Card::new(new_card_name, new_card_description, due_date.unwrap().to_string(),
                                            CardPriority::Low, vec![], vec![]);
                                        let current_board = self.boards.iter_mut().find(|board| board.id == current_board_id);
                                        if let Some(current_board) = current_board {
                                            current_board.cards.push(new_card.clone());
                                            self.state.current_card_id = Some(new_card.id);
                                        } else {
                                            error!("Current board not found");
                                            self.send_error_toast("Current board not found", None);
                                            self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                            return AppReturn::Continue;
                                        }
                                        self.state.ui_mode = self.state.prev_ui_mode.as_ref().unwrap_or_else(|| &self.config.default_view).clone();
                                    } else {
                                        warn!("New card name is empty or already exists");
                                        self.send_warning_toast("New card name is empty or already exists", None);
                                    }

                                    if let Some(previous_focus) = &self.state.previous_focus {
                                        self.focus = previous_focus.clone();
                                    }
                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                }
                                AppReturn::Continue
                            }
                            UiMode::LoadSave => {
                                self.dispatch(IoEvent::LoadSave).await;
                                AppReturn::Continue
                            }
                            UiMode::EditKeybindings => {
                                if self.state.edit_keybindings_state.selected().is_some() && self.focus != Focus::SubmitButton {
                                    self.state.popup_mode = Some(PopupMode::EditSpecificKeyBinding);
                                } else if self.focus == Focus::SubmitButton {
                                    self.config.keybindings = KeyBindings::default();
                                    warn!("Reset keybindings to default");
                                    self.send_warning_toast("Reset keybindings to default", None);
                                    self.focus = Focus::NoFocus;
                                    self.state.edit_keybindings_state.select(None);
                                    let write_config_status = write_config(&self.config);
                                    if write_config_status.is_err() {
                                        error!("Error writing config: {}", write_config_status.clone().unwrap_err());
                                        self.send_error_toast(&write_config_status.unwrap_err(), None);
                                    }
                                    self.keybind_list_maker();
                                }
                                AppReturn::Continue
                            }
                            _ => {
                                match self.focus {
                                    Focus::Help => {
                                        self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                        self.state.ui_mode = UiMode::HelpMenu;
                                    },
                                    Focus::Log => {
                                        self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
                                        self.state.ui_mode = UiMode::LogsOnly;
                                    },
                                    _ => {}
                                }
                                if UiMode::view_modes().contains(&self.state.ui_mode) && self.focus == Focus::Body{
                                    // check if there is a current card
                                    if let Some(current_card_id) = self.state.current_card_id {
                                        if let Some(current_board_id) = self.state.current_board_id {
                                            // check if the current card is in the current board
                                            let current_board = self.boards.iter().find(|board| board.id == current_board_id);
                                            if let Some(current_board) = current_board {
                                                let current_card = current_board.cards.iter().find(|card| card.id == current_card_id);
                                                if let Some(_) = current_card {
                                                    self.state.popup_mode = Some(PopupMode::CardView);
                                                } else {
                                                    // if the current card is not in the current board then set the current card to None
                                                    self.state.current_card_id = None;
                                                }
                                            } else {
                                                // if the current board is not found then set the current card to None
                                                self.state.current_card_id = None;
                                            }
                                        } else {
                                            // if the current board is not found then set the current card to None
                                            self.state.current_card_id = None;
                                        }
                                    }
                                }
                                AppReturn::Continue
                            }
                        }
                    }
                    Action::HideUiElement => {
                        let current_focus = Focus::from_str(self.focus.to_str());
                        let current_ui_mode = self.state.ui_mode.clone();
                        // hide the current focus by switching to a view where it is not available
                        // for example if current uimode is Title and focus is on Title then switch to Zen
                        if current_ui_mode == UiMode::Zen {
                            self.state.ui_mode = UiMode::MainMenu;
                            if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                        } else if current_ui_mode == UiMode::TitleBody {
                            if current_focus == Focus::Title {
                                self.state.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        } else if current_ui_mode == UiMode::BodyHelp {
                            if current_focus == Focus::Help {
                                self.state.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        } else if current_ui_mode == UiMode::BodyLog {
                            if current_focus == Focus::Log {
                                self.state.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        } else if current_ui_mode == UiMode::TitleBodyHelp {
                            if current_focus == Focus::Title {
                                self.state.ui_mode = UiMode::BodyHelp;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Help {
                                self.state.ui_mode = UiMode::TitleBody;
                                self.focus = Focus::Title;
                            }
                            else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        } else if current_ui_mode == UiMode::TitleBodyLog {
                            if current_focus == Focus::Title {
                                self.state.ui_mode = UiMode::BodyLog;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Log {
                                self.state.ui_mode = UiMode::TitleBody;
                                self.focus = Focus::Title;
                            }
                            else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        } else if current_ui_mode == UiMode::TitleBodyHelpLog {
                            if current_focus == Focus::Title {
                                self.state.ui_mode = UiMode::BodyHelpLog;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Help {
                                self.state.ui_mode = UiMode::TitleBodyLog;
                                self.focus = Focus::Title;
                            }
                            else if current_focus == Focus::Log {
                                self.state.ui_mode = UiMode::TitleBodyHelp;
                                self.focus = Focus::Title;
                            }
                            else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        } else if current_ui_mode == UiMode::BodyHelpLog {
                            if current_focus == Focus::Help {
                                self.state.ui_mode = UiMode::BodyLog;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Log {
                                self.state.ui_mode = UiMode::BodyHelp;
                                self.focus = Focus::Body;
                            }
                            else {
                                self.state.ui_mode = UiMode::MainMenu;
                                if self.state.main_menu_state.selected().is_none() {
                                    self.main_menu_next();
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::SaveState => {
                        self.dispatch(IoEvent::SaveLocalData).await;
                        AppReturn::Continue
                    }
                    Action::NewBoard => {
                        // check if current ui_mode is in UiMode::view_modes()
                        if UiMode::view_modes().contains(&self.state.ui_mode) {
                            self.state.new_board_form = vec![String::new(), String::new()];
                            self.set_ui_mode(UiMode::NewBoard);
                            self.state.previous_focus = Some(self.focus.clone());
                        }
                        AppReturn::Continue
                    }
                    Action::NewCard => {
                        if UiMode::view_modes().contains(&self.state.ui_mode) {
                            // check if current board is not empty
                            if self.state.current_board_id.is_none() {
                                warn!("No board available to add card to");
                                self.send_warning_toast("No board available to add card to",None);
                                return AppReturn::Continue;
                            }
                            self.state.new_card_form = vec![String::new(), String::new(), String::new()];
                            self.set_ui_mode(UiMode::NewCard);
                            self.state.previous_focus = Some(self.focus.clone());
                        }
                        AppReturn::Continue
                    }
                    Action::DeleteCard => {
                        match self.state.ui_mode {
                            UiMode::LoadSave => {
                                // run delete task in background
                                self.dispatch(IoEvent::DeleteSave).await;
                                tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                                self.dispatch(IoEvent::LoadPreview).await;
                                AppReturn::Continue
                            }
                            _ => {
                                if !UiMode::view_modes().contains(&self.state.ui_mode) {
                                    return AppReturn::Continue;
                                }
                                match self.focus {
                                    Focus::Body => {
                                        // delete the current card
                                        if let Some(current_board) = self.state.current_board_id {
                                            // find index of current board id in self.boards
                                            let index = self.boards.iter().position(|board| board.id == current_board);
                                            if let Some(current_card) = self.state.current_card_id {
                                                let card_index = self.boards[index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                                if let Some(card_index) = card_index {
                                                    let card_name = self.boards[index.unwrap()].cards[card_index].name.clone();
                                                    self.boards[index.unwrap()].cards.remove(card_index);
                                                    // if index is > 0, set current card to previous card, else set to next card, else set to None
                                                    if card_index > 0 {
                                                        self.state.current_card_id = Some(self.boards[index.unwrap()].cards[card_index - 1].id);
                                                    } else if self.boards[index.unwrap()].cards.len() > 0 {
                                                        self.state.current_card_id = Some(self.boards[index.unwrap()].cards[0].id);
                                                    } else {
                                                        self.state.current_card_id = None;
                                                    }
                                                    warn!("Deleted card {}", card_name);
                                                    self.send_warning_toast(&format!("Deleted card {}", card_name), None);
                                                    // remove card_id from self.visible_boards_and_cards if it is there, where visible_boards_and_cards is a LinkedHashMap of board_id to a vector of card_ids
                                                    if let Some(visible_cards) = self.visible_boards_and_cards.get_mut(&current_board) {
                                                        if let Some(card_index) = visible_cards.iter().position(|card_id| *card_id == current_card) {
                                                            visible_cards.remove(card_index);
                                                        }
                                                    }
                                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                                }

                                            } else if let Some(current_board) = self.state.current_board_id {
                                                // find index of current board id in self.boards
                                                let index = self.boards.iter().position(|board| board.id == current_board);
                                                if let Some(index) = index {
                                                    let board_name = self.boards[index].name.clone();
                                                    self.boards.remove(index);
                                                    // if index is > 0, set current board to previous board, else set to next board, else set to None
                                                    if index > 0 {
                                                        self.state.current_board_id = Some(self.boards[index - 1].id);
                                                    } else if self.boards.len() > 0 {
                                                        self.state.current_board_id = Some(self.boards[0].id);
                                                    } else {
                                                        self.state.current_board_id = None;
                                                    }
                                                    warn!("Deleted board {}", board_name);
                                                    self.send_warning_toast(&format!("Deleted board {}", board_name), None);
                                                    // remove board_id from self.visible_boards_and_cards if it is there
                                                    self.visible_boards_and_cards.remove(&current_board);
                                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                                }
                                            }
                                        }
                                        AppReturn::Continue
                                    },
                                    _ => AppReturn::Continue   
                                }
                            }
                        }
                    }
                    Action::DeleteBoard => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        match self.focus {
                            Focus::Body => {
                                // delete the current board from self.boards
                                if let Some(current_board) = self.state.current_board_id.clone() {
                                    // find index of current board id in self.boards
                                    let index = self.boards.iter().position(|board| board.id == current_board);
                                    if let Some(index) = index {
                                        let board_name = self.boards[index].name.clone();
                                        self.boards.remove(index);
                                        // if index is > 0, set current board to previous board, else set to next board, else set to None
                                        if index > 0 {
                                            self.state.current_board_id = Some(self.boards[index - 1].id.clone());
                                        } else if index < self.boards.len() {
                                            self.state.current_board_id = Some(self.boards[index].id.clone());
                                        } else {
                                            self.state.current_board_id = None;
                                        }
                                        self.visible_boards_and_cards.remove(&current_board);
                                        warn!("Deleted board: {}", board_name);
                                        self.send_warning_toast(&format!("Deleted board: {}", board_name), None);
                                    }
                                }
                                AppReturn::Continue
                            },
                            _ => AppReturn::Continue
                        }
                    }
                    Action::ChangeCardStatusToCompleted => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        // check if focus is on body
                        if self.focus != Focus::Body {
                            return AppReturn::Continue;
                        }
                        // get the current card and change its status to complete
                        if let Some(current_board) = self.state.current_board_id {
                            // find index of current board id in self.boards
                            let index = self.boards.iter().position(|board| board.id == current_board);
                            if let Some(current_card) = self.state.current_card_id {
                                let card_index = self.boards[index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                if let Some(card_index) = card_index {
                                    self.boards[index.unwrap()].cards[card_index].card_status = CardStatus::Complete;
                                    info!("Changed status to Completed for card {}", self.boards[index.unwrap()].cards[card_index].name);
                                    self.send_info_toast(&format!("Changed status to Completed for card {}", self.boards[index.unwrap()].cards[card_index].name), None);
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::ChangeCardStatusToActive => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        // check if focus is on body
                        if self.focus != Focus::Body {
                            return AppReturn::Continue;
                        }
                        // get the current card and change its status to active
                        if let Some(current_board) = self.state.current_board_id {
                            // find index of current board id in self.boards
                            let index = self.boards.iter().position(|board| board.id == current_board);
                            if let Some(current_card) = self.state.current_card_id {
                                let card_index = self.boards[index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                if let Some(card_index) = card_index {
                                    self.boards[index.unwrap()].cards[card_index].card_status = CardStatus::Active;
                                    info!("Changed status to Active for card {}", self.boards[index.unwrap()].cards[card_index].name);
                                    self.send_info_toast(&format!("Changed status to Active for card {}", self.boards[index.unwrap()].cards[card_index].name), None);
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::ChangeCardStatusToStale => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        // check if focus is on body
                        if self.focus != Focus::Body {
                            return AppReturn::Continue;
                        }
                        // get the current card and change its status to stale
                        if let Some(current_board) = self.state.current_board_id {
                            // find index of current board id in self.boards
                            let index = self.boards.iter().position(|board| board.id == current_board);
                            if let Some(current_card) = self.state.current_card_id {
                                let card_index = self.boards[index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                if let Some(card_index) = card_index {
                                    self.boards[index.unwrap()].cards[card_index].card_status = CardStatus::Stale;
                                    info!("Changed status to Stale for card {}", self.boards[index.unwrap()].cards[card_index].name);
                                    self.send_info_toast(&format!("Changed status to Stale for card {}", self.boards[index.unwrap()].cards[card_index].name), None);
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::GoToMainMenu => {
                        self.state.current_board_id = None;
                        self.state.current_card_id = None;
                        self.focus = Focus::MainMenu;
                        self.state.ui_mode = UiMode::MainMenu;
                        if self.state.main_menu_state.selected().is_none() {
                            self.state.main_menu_state.select(Some(0));
                        }
                        AppReturn::Continue
                    }
                    Action::MoveCardUp => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        match self.focus {
                            Focus::Body => {
                                if self.state.current_card_id.is_none() {
                                    return AppReturn::Continue;
                                } else {
                                    if let Some(current_board) = self.state.current_board_id {
                                        let index = self.boards.iter().position(|board| board.id == current_board);
                                        if let Some(current_card) = self.state.current_card_id {
                                            let card_index = self.boards[index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                            if let Some(card_index) = card_index {
                                                if card_index > 0 {
                                                    self.boards[index.unwrap()].cards.swap(card_index, card_index - 1);
                                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                                    // check if current card is visible if not set it to the first visible card
                                                    if !self.visible_boards_and_cards[&current_board].contains(&self.state.current_card_id.unwrap()) {
                                                        self.state.current_card_id = Some(self.visible_boards_and_cards[&current_board][0]);
                                                    }
                                                    info!("Moved card {} up", self.boards[index.unwrap()].cards[card_index].name);
                                                    self.send_info_toast(&format!("Moved card {} up", self.boards[index.unwrap()].cards[card_index].name), None);
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::MoveCardDown => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        match self.focus {
                            Focus::Body => {
                                if self.state.current_card_id.is_none() {
                                    return AppReturn::Continue;
                                } else {
                                    if let Some(current_board) = self.state.current_board_id {
                                        let board_index = self.boards.iter().position(|board| board.id == current_board);
                                        if let Some(current_card) = self.state.current_card_id {
                                            let card_index = self.boards[board_index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                            if let Some(card_index) = card_index {
                                                if card_index < self.boards[board_index.unwrap()].cards.len() - 1 {
                                                    self.boards[board_index.unwrap()].cards.swap(card_index, card_index + 1);
                                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                                    // check if current card is visible if not set it to the last visible card
                                                    if !self.visible_boards_and_cards[&current_board].contains(&self.state.current_card_id.unwrap()) {
                                                        self.state.current_card_id = Some(self.visible_boards_and_cards[&current_board][self.visible_boards_and_cards[&current_board].len() - 1]);
                                                    }
                                                    info!("Moved card {} down", self.boards[board_index.unwrap()].cards[card_index].name);
                                                    self.send_info_toast(&format!("Moved card {} down", self.boards[board_index.unwrap()].cards[card_index].name), None);
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::MoveCardRight => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        match self.focus {
                            Focus::Body => {
                                if self.state.current_card_id.is_none() {
                                    return AppReturn::Continue;
                                } else {
                                    if let Some(current_board) = self.state.current_board_id {
                                        let board_index = self.boards.iter().position(|board| board.id == current_board);
                                        // check if board is the last board
                                        if board_index.unwrap() < self.boards.len() - 1 {
                                            if let Some(current_card) = self.state.current_card_id {
                                                let card_index = self.boards[board_index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                                if let Some(card_index) = card_index {
                                                    let card = self.boards[board_index.unwrap()].cards.remove(card_index);
                                                    let card_name = card.name.clone();
                                                    self.boards[board_index.unwrap() + 1].cards.push(card);
                                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                                    self.state.current_board_id = Some(self.boards[board_index.unwrap() + 1].id);
                                                    info!("Moved card {} right", card_name);
                                                    self.send_info_toast(&format!("Moved card {} right", card_name), None);
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::MoveCardLeft => {
                        if !UiMode::view_modes().contains(&self.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        match self.focus {
                            Focus::Body => {
                                if self.state.current_card_id.is_none() {
                                    return AppReturn::Continue;
                                } else {
                                    if let Some(current_board) = self.state.current_board_id {
                                        let board_index = self.boards.iter().position(|board| board.id == current_board);
                                        // check if board is the first board
                                        if board_index.unwrap() > 0 {
                                            if let Some(current_card) = self.state.current_card_id {
                                                let card_index = self.boards[board_index.unwrap()].cards.iter().position(|card| card.id == current_card);
                                                if let Some(card_index) = card_index {
                                                    let card = self.boards[board_index.unwrap()].cards.remove(card_index);
                                                    let card_name = card.name.clone();
                                                    self.boards[board_index.unwrap() - 1].cards.push(card);
                                                    self.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
                                                    self.state.current_board_id = Some(self.boards[board_index.unwrap() - 1].id);
                                                    info!("Moved card {} left", card_name);
                                                    self.send_info_toast(&format!("Moved card {} left", card_name), None);
                                                }
                                            }
                                        }
                                    }
                                }
                            },
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::ToggleCommandPalette => {
                        if self.state.popup_mode.is_none() {
                            self.state.popup_mode = Some(PopupMode::CommandPalette);
                            self.state.app_status = AppStatus::UserInput;
                        } else if self.state.popup_mode == Some(PopupMode::CommandPalette) {
                            self.state.popup_mode = None;
                            self.state.app_status = AppStatus::Initialized;
                        } else {
                            self.state.popup_mode = Some(PopupMode::CommandPalette);
                            self.state.app_status = AppStatus::UserInput;
                        }
                        AppReturn::Continue
                    }
                }
            } else {
                warn!("No action accociated to {}", key);
                self.send_warning_toast(&format!("No action accociated to {}", key), None);
                AppReturn::Continue
            }
        }
    }
    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        // check if last_io_event_time is more thant current time + IO_EVENT_WAIT_TIME in ms
        if self.last_io_event_time.unwrap_or_else(|| Instant::now() - Duration::from_millis(IO_EVENT_WAIT_TIME + 10)) + Duration::from_millis(IO_EVENT_WAIT_TIME) > Instant::now() {
            self.send_error_toast(&format!("Please wait before sending another request - {:?}",action), None);
            tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
        }
        self.last_io_event_time = Some(Instant::now());
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            debug!("Error from dispatch {}", e);
            error!("Error in handling request please, restart the app");
            self.send_error_toast("Error in handling request please, restart the app",None);
        };
    }
    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn status(&self) -> &AppStatus {
        &self.state.app_status
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = Action::all()
        .into();
        if self.state.ui_mode == UiMode::MainMenu {
            self.main_menu_next();
        } else if self.focus == Focus::NoFocus {
            self.focus = Focus::Body;
        }
        self.state.app_status = AppStatus::initialized()
    }
    pub fn set_boards(&mut self, boards: Vec<Board>) {
        self.boards = boards;
    }
    pub fn loaded(&mut self) {
        self.is_loading = false;
    }
    pub fn current_focus(&self) -> &Focus {
        &self.focus
    }
    pub fn change_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }
    pub fn clear_current_user_input(&mut self) {
        self.state.current_user_input = String::new();
    }
    pub fn set_config_state(&mut self, config_state: TableState) {
        self.state.config_state = config_state;
    }
    pub fn config_next(&mut self) {
        let i = match self.state.config_state.selected() {
            Some(i) => {
                if i >= self.config.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.config_state.select(Some(i));
    }
    pub fn config_previous(&mut self) {
        let i = match self.state.config_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.config.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.config_state.select(Some(i));
    }
    pub fn main_menu_next(&mut self) {
        let i = match self.state.main_menu_state.selected() {
            Some(i) => {
                if i >= MainMenu::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.main_menu_state.select(Some(i));
    }
    pub fn main_menu_previous(&mut self) {
        let i = match self.state.main_menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    MainMenu::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.main_menu_state.select(Some(i));
    }
    pub fn load_save_next(&mut self) {
        let i = match self.state.load_save_state.selected() {
            Some(i) => {
                let local_save_files_len = get_available_local_savefiles();
                let local_save_files_len = if local_save_files_len.is_none() {
                    0
                } else {
                    local_save_files_len.unwrap().len()
                };
                if local_save_files_len == 0 {
                    0
                } else if i >= local_save_files_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.load_save_state.select(Some(i));
    }
    pub fn load_save_previous(&mut self) {
        let i = match self.state.load_save_state.selected() {
            Some(i) => {
                let local_save_files_len = get_available_local_savefiles();
                let local_save_files_len = if local_save_files_len.is_none() {
                    0
                } else {
                    local_save_files_len.unwrap().len()
                };
                if local_save_files_len == 0 {
                    0
                } else if i == 0 {
                    local_save_files_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.load_save_state.select(Some(i));
    }
    pub fn config_state(&self) -> &TableState {
        &self.state.config_state
    }
    pub fn set_ui_mode(&mut self, ui_mode: UiMode) {
        self.state.prev_ui_mode = Some(self.state.ui_mode.clone());
        self.state.ui_mode = ui_mode;
        let available_focus_targets = self.state.ui_mode.get_available_targets();
        if !available_focus_targets.contains(&self.focus) {
            // check if available focus targets is empty
            if available_focus_targets.is_empty() {
                self.focus = Focus::NoFocus;
            } else {
                self.focus = available_focus_targets[0];
            }
        }
    }
    pub fn edit_keybindings_next(&mut self) {
        let keybind_iterator = self.config.keybindings.iter();
        let i = match self.state.edit_keybindings_state.selected() {
            Some(i) => {
                if i >= keybind_iterator.count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.edit_keybindings_state.select(Some(i));
    }
    pub fn edit_keybindings_prev(&mut self) {
        let keybind_iterator = self.config.keybindings.iter();
        let i = match self.state.edit_keybindings_state.selected() {
            Some(i) => {
                if i == 0 {
                    keybind_iterator.count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.edit_keybindings_state.select(Some(i));
    }
    pub fn help_next(&mut self) {
        // as the help menu is split into two use only half the length of the keybind store
        let i = match self.state.help_state.selected() {
            Some(i) => {
                if !self.state.keybind_store.is_empty() {
                    if i >= (self.state.keybind_store.len() / 2) - 1 {
                        0
                    } else {
                        i + 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.help_state.select(Some(i));
    }
    pub fn help_prev(&mut self) {
        let i = match self.state.help_state.selected() {
            Some(i) => {
                if !self.state.keybind_store.is_empty() {
                    if i == 0 {
                        (self.state.keybind_store.len() / 2) - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.help_state.select(Some(i));
    }
    pub fn select_default_view_next(&mut self) {
        let i = match self.state.default_view_state.selected() {
            Some(i) => {
                if i >= UiMode::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.default_view_state.select(Some(i));
    }
    pub fn select_default_view_prev(&mut self) {
        let i = match self.state.default_view_state.selected() {
            Some(i) => {
                if i == 0 {
                    UiMode::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.default_view_state.select(Some(i));
    }
    pub fn command_palette_up(&mut self) {
        let i = match self.state.command_palette_list_state.selected() {
            Some(i) => {
                if !self.command_palette.search_results.is_none() {
                    if i == 0 {
                        self.command_palette.search_results.clone().unwrap().len() - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.command_palette_list_state.select(Some(i));
    }
    pub fn command_palette_down(&mut self) {
        let i = match self.state.command_palette_list_state.selected() {
            Some(i) => {
                if !self.command_palette.search_results.is_none() {
                    if i >= self.command_palette.search_results.clone().unwrap().len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.command_palette_list_state.select(Some(i));
    }
    pub fn keybind_list_maker(&mut self) {

        let keybinds = &self.config.keybindings;
        let default_actions = &self.actions;
        let keybind_action_iter = keybinds.iter();
        let mut keybind_action_list: Vec<Vec<String>> = Vec::new();
    
        for (action, keys) in keybind_action_iter {
            let mut keybind_action = Vec::new();
            let mut keybind_string = String::new();
            for key in keys {
                keybind_string.push_str(&key.to_string());
                keybind_string.push_str(" ");
            }
            keybind_action.push(keybind_string);
            let action_translated_string = KeyBindings::str_to_action(keybinds.clone(), action).unwrap_or_else(|| &Action::Quit).to_string();
            keybind_action.push(action_translated_string);
            keybind_action_list.push(keybind_action);
        }
    
        let default_action_iter = default_actions.actions().iter();
        // append to keybind_action_list if the keybind is not already in the list
        for action in default_action_iter {
            let str_action = action.to_string();
            if !keybind_action_list.iter().any(|x| x[1] == str_action) {
                let mut keybind_action = Vec::new();
                let action_keys = action.keys()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                keybind_action.push(action_keys);
                keybind_action.push(str_action);
                keybind_action_list.push(keybind_action);
            }
        }
        self.state.keybind_store = keybind_action_list;
    }

    pub fn send_info_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(message.to_string(), duration, ToastType::Info));
        } else {
            self.state.toasts.push(ToastWidget::new(message.to_string(), Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Info));
        }
    }

    pub fn send_error_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(message.to_string(), duration, ToastType::Error));
        } else {
            self.state.toasts.push(ToastWidget::new(message.to_string(), Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Error));
        }
    }

    pub fn send_warning_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(message.to_string(), duration, ToastType::Warning));
        } else {
            self.state.toasts.push(ToastWidget::new(message.to_string(), Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Warning));
        }
    }

    pub fn select_current_card_status_prev(&mut self) {
        let i = match self.state.card_status_selector_state.selected() {
            Some(i) => {
                if i == 0 {
                    CardStatus::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.card_status_selector_state.select(Some(i));
    }

    pub fn select_current_card_status_next(&mut self) {
        let i = match self.state.card_status_selector_state.selected() {
            Some(i) => {
                if i >= CardStatus::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.card_status_selector_state.select(Some(i));
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MainMenuItem {
    View,
    Config,
    Help,
    LoadSave,
    Quit
}

impl Display for MainMenuItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MainMenuItem::View => write!(f, "View your Boards"),
            MainMenuItem::Config => write!(f, "Configure"),
            MainMenuItem::Help => write!(f, "Help"),
            MainMenuItem::LoadSave => write!(f, "Load a Save"),
            MainMenuItem::Quit => write!(f, "Quit"),
        }
    }
}

pub struct MainMenu {
    pub items: Vec<MainMenuItem>
}

impl MainMenu {
    pub fn all() -> Vec<MainMenuItem> {
        vec![
            MainMenuItem::View,
            MainMenuItem::Config,
            MainMenuItem::Help,
            MainMenuItem::LoadSave,
            MainMenuItem::Quit,
        ]
    }

    pub fn from_index(index: usize) -> MainMenuItem {
        match index {
            0 => MainMenuItem::View,
            1 => MainMenuItem::Config,
            2 => MainMenuItem::Help,
            3 => MainMenuItem::LoadSave,
            4 => MainMenuItem::Quit,
            _ => MainMenuItem::Quit
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum PopupMode {
    CardView,
    CommandPalette,
    EditSpecificKeyBinding,
    ChangeUIMode,
    ChangeCurrentCardStatus,
    EditGeneralConfig,
    SelectDefaultView,
}

impl Display for PopupMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            PopupMode::CardView => write!(f, "Card View"),
            PopupMode::CommandPalette => write!(f, "Command Palette"),
            PopupMode::EditSpecificKeyBinding => write!(f, "Edit Specific Key Binding"),
            PopupMode::ChangeUIMode => write!(f, "Change UI Mode"),
            PopupMode::ChangeCurrentCardStatus => write!(f, "Change Current Card Status"),
            PopupMode::EditGeneralConfig => write!(f, "Edit General Config"),
            PopupMode::SelectDefaultView => write!(f, "Select Default View"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub app_status: AppStatus,
    pub current_board_id: Option<u128>,
    pub current_card_id: Option<u128>,
    pub previous_focus: Option<Focus>,
    pub current_user_input: String,
    pub main_menu_state: ListState,
    pub config_state: TableState,
    pub new_board_form: Vec<String>,
    pub new_card_form: Vec<String>,
    pub load_save_state: ListState,
    pub edit_keybindings_state: TableState,
    pub edited_keybinding: Option<Vec<Key>>,
    pub help_state: TableState,
    pub keybind_store: Vec<Vec<String>>,
    pub default_view_state: ListState,
    pub current_cursor_position: Option<usize>,
    pub toasts: Vec<ToastWidget>,
    pub term_background_color: (u8, u8, u8),
    pub preview_boards_and_cards: Option<Vec<Board>>,
    pub preview_visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>>,
    pub popup_mode: Option<PopupMode>,
    pub ui_mode: UiMode,
    pub no_of_cards_to_show: u16,
    pub command_palette_list_state: ListState,
    pub card_status_selector_state: ListState,
    pub prev_ui_mode: Option<UiMode>,
    pub debug_menu_toggled: bool,
    pub ui_render_time: Option<u128>
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            app_status: AppStatus::default(),
            current_board_id: None,
            current_card_id: None,
            previous_focus: None,
            current_user_input: String::new(),
            main_menu_state: ListState::default(),
            config_state: TableState::default(),
            new_board_form: vec![String::new(), String::new()],
            new_card_form: vec![String::new(), String::new(), String::new()],
            load_save_state: ListState::default(),
            edit_keybindings_state: TableState::default(),
            edited_keybinding: None,
            help_state: TableState::default(),
            keybind_store: Vec::new(),
            default_view_state: ListState::default(),
            current_cursor_position: None,
            toasts: Vec::new(),
            term_background_color: get_term_bg_color(),
            preview_boards_and_cards: None,
            preview_visible_boards_and_cards: LinkedHashMap::new(),
            popup_mode: None,
            ui_mode: data_handler::get_default_ui_mode(),
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            command_palette_list_state: ListState::default(),
            card_status_selector_state: ListState::default(),
            prev_ui_mode: None,
            debug_menu_toggled: false,
            ui_render_time: None
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub save_directory: PathBuf,
    pub default_view: UiMode,
    pub always_load_last_save: bool,
    pub save_on_exit: bool,
    pub disable_scrollbars: bool,
    pub warning_delta: u16,
    pub keybindings: KeyBindings,
    pub tickrate: u64,
    pub no_of_cards_to_show: u16,
    pub no_of_boards_to_show: u16,
}

impl AppConfig {
    pub fn default() -> Self {
        let save_directory = env::temp_dir().join(SAVE_DIR_NAME);
        let default_view = UiMode::TitleBodyHelpLog;
        Self {
            save_directory: save_directory,
            default_view,
            always_load_last_save: true,
            save_on_exit: true,
            disable_scrollbars: false,
            warning_delta: DEFAULT_CARD_WARNING_DUE_DATE_DAYS,
            keybindings: KeyBindings::default(),
            tickrate: DEFAULT_TICKRATE,
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            no_of_boards_to_show: NO_OF_BOARDS_PER_PAGE,
        }
    }

    pub fn to_list(&self) -> Vec<Vec<String>> {
        vec![
            vec![String::from("Save Directory"), self.save_directory.to_str().unwrap().to_string()],
            vec![String::from("Select Default View"), self.default_view.to_string()],
            vec![String::from("Auto Load Last Save"), self.always_load_last_save.to_string()],
            vec![String::from("Auto Save on Exit"), self.save_on_exit.to_string()],
            vec![String::from("Disable Scrollbars"), self.disable_scrollbars.to_string()],
            vec![String::from("Number of Days to Warn Before Due Date"), self.warning_delta.to_string()],
            vec![String::from("Tickrate"), self.tickrate.to_string()],
            vec![String::from("Number of Cards to Show per board"), self.no_of_cards_to_show.to_string()],
            vec![String::from("Number of Boards to Show"), self.no_of_boards_to_show.to_string()],
            vec![String::from("Edit Keybindings")],
        ]
    }

    pub fn edit_with_string(change_str: &str, app: &mut App) -> Self {
        let mut config = app.config.clone();
        let mut lines = change_str.lines();
        while let Some(line) = lines.next() {
            let mut parts = line.split(":");
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            match key {
                "Save Directory" => {
                    let new_path = PathBuf::from(value);
                    // check if the new path is valid
                    if new_path.exists() {
                        config.save_directory = new_path;
                    } else {
                        error!("Invalid path: {}", value);
                        app.send_error_toast(&format!("Invalid path: {}", value),None);
                        app.send_info_toast("Check if the path exists",None);
                    }
                }
                "Select Default View" => {
                    let new_ui_mode = UiMode::from_string(value);
                    if new_ui_mode.is_some() {
                        config.default_view = new_ui_mode.unwrap();
                    } else {
                        error!("Invalid UiMode: {}", value);
                        app.send_error_toast(&format!("Invalid UiMode: {}", value),None);
                        info!("Valid UiModes are: {:?}", UiMode::all());
                    }
                }
                "Auto Load Last Save" => {
                    if value.to_lowercase() == "true" {
                        config.always_load_last_save = true;
                    } else if value.to_lowercase() == "false" {
                        config.always_load_last_save = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value),None);
                    }
                }
                "Auto Save on Exit" => {
                    if value.to_lowercase() == "true" {
                        config.save_on_exit = true;
                    } else if value.to_lowercase() == "false" {
                        config.save_on_exit = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value),None);
                    }
                }
                "Disable Scrollbars" => {
                    if value.to_lowercase() == "true" {
                        config.disable_scrollbars = true;
                    } else if value.to_lowercase() == "false" {
                        config.disable_scrollbars = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value),None);
                    }
                }
                "Number of Days to Warn Before Due Date" => {
                    let new_delta = value.parse::<u16>();
                    if new_delta.is_ok() {
                        config.warning_delta = new_delta.unwrap();
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(&format!("Expected number of days (integer), got: {}", value),None);
                    }
                }
                "Tickrate" => {
                    let new_tickrate = value.parse::<u64>();
                    if new_tickrate.is_ok() {
                        let new_tickrate = new_tickrate.unwrap();
                        // make sure tickrate is not too low or too high
                        if new_tickrate < 50 {
                            error!("Tickrate must be greater than 50ms, to avoid overloading the CPU");
                            app.send_error_toast("Tickrate must be greater than 50ms, to avoid overloading the CPU",None);
                        } else if new_tickrate > 1000 {
                            error!("Tickrate must be less than 1000ms");
                            app.send_error_toast("Tickrate must be less than 1000ms",None);
                        } else {
                            config.tickrate = new_tickrate;
                            info!("Tickrate set to {}ms", new_tickrate);
                            info!("Restart the program to apply changes");
                            info!("If experiencing slow input, or stuttering, try adjusting the tickrate");
                            app.send_info_toast(&format!("Tickrate set to {}ms", new_tickrate),None);
                        }
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(&format!("Expected number of milliseconds (integer), got: {}", value),None);
                    }
                }
                "Number of Cards to Show per board" => {
                    let new_no_cards = value.parse::<u16>();
                    if new_no_cards.is_ok() {
                        let unwrapped = new_no_cards.unwrap();
                        if unwrapped < MIN_NO_CARDS_PER_BOARD {
                            error!("Number of cards must be greater than {}", MIN_NO_CARDS_PER_BOARD);
                            app.send_error_toast(&format!("Number of cards must be greater than {}", MIN_NO_CARDS_PER_BOARD),None);
                        } else if unwrapped > MAX_NO_CARDS_PER_BOARD {
                            error!("Number of cards must be less than {}", MAX_NO_CARDS_PER_BOARD);
                            app.send_error_toast(&format!("Number of cards must be less than {}", MAX_NO_CARDS_PER_BOARD),None);
                        } else {
                            config.no_of_cards_to_show = unwrapped;
                            app.send_info_toast(&format!("Number of cards per board to display set to {}", unwrapped),None);
                        }
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(&format!("Expected number of cards (integer), got: {}", value),None);
                    }
                }
                "Number of Boards to Show" => {
                    let new_no_boards = value.parse::<u16>();
                    if new_no_boards.is_ok() {
                        let unwrapped = new_no_boards.unwrap();
                        if unwrapped < MIN_NO_BOARDS_PER_PAGE {
                            error!("Number of boards must be greater than {}", MIN_NO_BOARDS_PER_PAGE);
                            app.send_error_toast(&format!("Number of boards must be greater than {}", MIN_NO_BOARDS_PER_PAGE),None);
                        } else if unwrapped > MAX_NO_BOARDS_PER_PAGE {
                            error!("Number of boards must be less than {}", MAX_NO_BOARDS_PER_PAGE);
                            app.send_error_toast(&format!("Number of boards must be less than {}", MAX_NO_BOARDS_PER_PAGE),None);
                        } else {
                            config.no_of_boards_to_show = unwrapped;
                            app.send_info_toast(&format!("Number of boards to display set to {}", unwrapped),None);
                        }
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(&format!("Expected number of boards (integer), got: {}", value),None);
                    }
                }
                _ => {
                    debug!("Invalid key: {}", key);
                    app.send_error_toast("Something went wrong 😢 ",None);
                    return config;
                }
            }
        }
        refresh_visible_boards_and_cards(app);
        config
    }

    pub fn edit_keybinding(&mut self, key_index: usize, value: Vec<Key>) -> Result<(), String> {
        // make sure key is not empty, or already assigned
        
        let get_config_status = get_config(false);
        let config = if get_config_status.is_err() {
            debug!("Error getting config: {}", get_config_status.unwrap_err());
            AppConfig::default()
        } else {
            get_config_status.unwrap()
        };
        let current_bindings = config.keybindings;

        // make a list from the keybindings
        let mut key_list = vec![];
        for (k, v) in current_bindings.iter() {
            key_list.push((k, v));
        }
        // check if index is valid
        if key_index >= key_list.len() {
            debug!("Invalid key index: {}", key_index);
            error!("Unable to edit keybinding");
            return Err("Unable to edit keybinding 😢 ".to_string());
        }   
        let (key, _) = key_list[key_index];

        // check if key is present in current bindings if not, return error
        if !current_bindings.iter().any(|(k, _)| k == key) {
            debug!("Invalid key: {}", key);
            error!("Unable to edit keybinding");
            return Err("Unable to edit keybinding 😢 ".to_string());
        }

        for new_value in value.iter() {
            for (k, v) in current_bindings.iter() {
                if v.contains(new_value) && k != key {
                    error!("Value {} is already assigned to {}", new_value, k);
                    return Err(format!("Value {} is already assigned to {}", new_value, k));
                }
            }          
        }

        debug!("Editing keybinding: {} to {:?}", key, value);

        match key {
            "quit" => self.keybindings.quit = value,
            "next_focus" => self.keybindings.next_focus = value,
            "prev_focus" => self.keybindings.prev_focus = value,
            "open_config_menu" => self.keybindings.open_config_menu = value,
            "up" => self.keybindings.up = value,
            "down" => self.keybindings.down = value,
            "right" => self.keybindings.right = value,
            "left" => self.keybindings.left = value,
            "take_user_input" => self.keybindings.take_user_input = value,
            "hide_ui_element" => self.keybindings.hide_ui_element = value,
            "save_state" => self.keybindings.save_state = value,
            "new_board" => self.keybindings.new_board = value,
            "new_card" => self.keybindings.new_card = value,
            "delete_card" => self.keybindings.delete_card = value,
            "delete_board" => self.keybindings.delete_board = value,
            "change_card_status_to_completed" => self.keybindings.change_card_status_to_completed = value,
            "change_card_status_to_active" => self.keybindings.change_card_status_to_active = value,
            "change_card_status_to_stale" => self.keybindings.change_card_status_to_stale = value,
            "reset_ui" => self.keybindings.reset_ui = value,
            _ => {
                debug!("Invalid key: {}", key);
                error!("Unable to edit keybinding");
                return Err("Something went wrong 😢 ".to_string());
            }
        }
        Ok(())
    }
    
    pub fn len(&self) -> usize {
        self.to_list().len()
    }
}

pub fn get_term_bg_color() -> (u8, u8, u8) {
    // TODO: make this work on windows and unix
    (0, 0, 0)
}
