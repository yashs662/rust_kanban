use std::collections::BTreeMap;
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

use chrono::NaiveDate;
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
use tui::widgets::ListState;

use self::actions::Actions;
use self::state::AppStatus;
use self::state::Focus;
use self::state::UiMode;
use self::kanban::{
    Board,
    Card,
    CardPriority
};
use crate::app::actions::Action;
use crate::constants::{
    SAVE_DIR_NAME,
    FIELD_NOT_SET,
};
use crate::inputs::key::Key;
use crate::io::data_handler::{
    write_config,
    get_available_local_savefiles
};
use crate::io::{
    IoEvent,
    data_handler
};

pub mod actions;
pub mod state;
pub mod ui;
pub mod kanban;
pub mod ui_helper;

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
    focus: Focus,
    ui_mode: UiMode,
    pub boards: Vec<kanban::Board>,
    current_user_input: String,
    prev_ui_mode: UiMode,
    config: AppConfig,
    config_item_being_edited: Option<usize>,
    pub visible_boards_and_cards: BTreeMap<u128, Vec<u128>>,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();
        let focus = Focus::NoFocus;
        let ui_mode = data_handler::get_default_ui_mode();
        let boards = vec![];


        Self {
            io_tx,
            actions,
            is_loading,
            state,
            focus,
            ui_mode,
            boards: boards,
            current_user_input: String::new(),
            prev_ui_mode: UiMode::Zen,
            config: AppConfig::default(),
            config_item_being_edited: None,
            visible_boards_and_cards: BTreeMap::new(),
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        // check if we are in a user input mode
        if self.state.status == AppStatus::UserInput {
            // append to current user input if key is not enter else change state to Initialized
            if key != Key::Enter && key != Key::Esc {
                let mut current_key = key.to_string();
                if current_key == "<Space>" {
                    current_key = " ".to_string();
                } else if current_key == "<ShiftEnter>" {
                    current_key = "".to_string();
                } else if current_key == "<Tab>" {
                    current_key = "  ".to_string();
                } else if current_key == "<Backspace>" {
                    match self.ui_mode {
                        UiMode::NewBoard => {
                            match self.focus {
                                Focus::NewBoardName => self.state.new_board_form[0].pop(),
                                Focus::NewBoardDescription => self.state.new_board_form[1].pop(),
                                _ => Option::None,
                            }
                        }
                        UiMode::NewCard => {
                            match self.focus {
                                Focus::NewCardName => self.state.new_card_form[0].pop(),
                                Focus::NewCardDescription => self.state.new_card_form[1].pop(),
                                Focus::NewCardDueDate => self.state.new_card_form[2].pop(),
                                _ => Option::None,
                            }
                        }
                        _ => self.current_user_input.pop(),
                    };
                    return AppReturn::Continue;
                } else if current_key.starts_with("<") && current_key.ends_with(">") {
                    current_key = current_key[1..current_key.len() - 1].to_string();
                }

                if self.focus == Focus::NewBoardName {
                    self.state.new_board_form[0].push_str(&current_key);
                } else if self.focus == Focus::NewBoardDescription {
                    self.state.new_board_form[1].push_str(&current_key);
                } else if self.focus == Focus::NewCardName {
                    self.state.new_card_form[0].push_str(&current_key);
                } else if self.focus == Focus::NewCardDescription {
                    self.state.new_card_form[1].push_str(&current_key);
                } else if self.focus == Focus::NewCardDueDate {
                    self.state.new_card_form[2].push_str(&current_key);
                } else {
                    self.current_user_input.push_str(&current_key);
                }
            } else {
                self.state.status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
            return AppReturn::Continue;
        } else {
            if let Some(action) = self.actions.find(key) {
                match action {
                    Action::Quit => AppReturn::Exit,
                    Action::Tab => {
                        let current_focus = self.focus.clone();
                        let next_focus = self.focus.next(&UiMode::get_available_targets(&self.ui_mode));
                        // check if the next focus is the same as the current focus or NoFocus if so set back to the first focus
                        if next_focus == current_focus || next_focus == Focus::NoFocus {
                            self.focus = current_focus;
                        } else {
                            self.focus = next_focus;
                        }
                        AppReturn::Continue
                    }
                    Action::ShiftTab => {
                        let current_focus = self.focus.clone();
                        let next_focus = self.focus.prev(&UiMode::get_available_targets(&self.ui_mode));
                        // check if the next focus is the same as the current focus or NoFocus if so set back to the first focus
                        if next_focus == current_focus || next_focus == Focus::NoFocus {
                            self.focus = current_focus;
                        } else {
                            self.focus = next_focus;
                        }
                        AppReturn::Continue
                    }
                    Action::SetUiMode => {
                        let new_ui_mode = UiMode::from_number(key.to_digit() as u8);
                        if new_ui_mode == UiMode::MainMenu {
                            self.main_menu_next();
                        }
                        let available_focus_targets = UiMode::get_available_targets(&new_ui_mode);
                        // check if focus is still available in the new ui_mode if not set it to the first available tab
                        if !available_focus_targets.contains(&self.focus.to_str().to_string()) {
                            // check if available focus targets is empty
                            if available_focus_targets.is_empty() {
                                self.focus = Focus::NoFocus;
                            } else {
                                self.focus = Focus::from_str(available_focus_targets[0].as_str());
                            }
                        }
                        self.ui_mode = new_ui_mode;
                        AppReturn::Continue
                    }
                    Action::ToggleConfig => {
                        if self.ui_mode == UiMode::Config {
                            self.ui_mode = self.prev_ui_mode.clone();
                        } else {
                            self.prev_ui_mode = self.ui_mode.clone();
                            self.ui_mode = UiMode::Config;
                            let available_focus_targets = self.ui_mode.get_available_targets();
                            if !available_focus_targets.contains(&self.focus.to_str().to_string()) {
                                // check if available focus targets is empty
                                if available_focus_targets.is_empty() {
                                    self.focus = Focus::NoFocus;
                                } else {
                                    self.focus = Focus::from_str(available_focus_targets[0].as_str());
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::Up => {
                        if self.ui_mode == UiMode::Config {
                            self.config_previous();
                        }
                        else if self.ui_mode == UiMode::MainMenu {
                            self.main_menu_previous();
                        }
                        else if self.ui_mode == UiMode::LoadSave {
                            self.load_save_previous();
                        }
                        else {
                            if self.focus == Focus::Body {
                                self.dispatch(IoEvent::GoUp).await;
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::Down => {
                        if self.ui_mode == UiMode::Config {
                            self.config_next();
                        }
                        else if self.ui_mode == UiMode::MainMenu {
                            self.main_menu_next();
                        }
                        else if self.ui_mode == UiMode::LoadSave {
                            self.load_save_next();
                        }
                        else {
                            if self.focus == Focus::Body {
                                self.dispatch(IoEvent::GoDown).await;
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::Right => {
                        if self.focus == Focus::Body {
                            self.dispatch(IoEvent::GoRight).await;
                        }
                        AppReturn::Continue
                    }
                    Action::Left => {
                        if self.focus == Focus::Body {
                            self.dispatch(IoEvent::GoLeft).await;
                        }
                        AppReturn::Continue
                    }
                    Action::TakeUserInput => {
                        match self.ui_mode {
                            UiMode::NewBoard => {
                                self.state.status = AppStatus::UserInput;
                                info!("Taking user input");
                            },
                            UiMode::EditConfig => {
                                self.state.status = AppStatus::UserInput;
                                info!("Taking user input");
                            },
                            UiMode::NewCard => {
                                self.state.status = AppStatus::UserInput;
                                info!("Taking user input");
                            },
                            _ => {}
                        }
                        AppReturn::Continue
                    }
                    Action::Escape => {
                        match self.ui_mode {
                            UiMode::Config => {
                                self.ui_mode = self.prev_ui_mode.clone();
                                AppReturn::Continue
                            }
                            UiMode::EditConfig => {
                                self.ui_mode = UiMode::Config;
                                AppReturn::Continue
                            }
                            UiMode::MainMenu => {
                                AppReturn::Exit
                            }
                            UiMode::NewBoard => {
                                self.ui_mode = self.prev_ui_mode.clone();
                                AppReturn::Continue
                            }
                            _ => {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                                AppReturn::Continue
                            }
                        }
                    }
                    Action::Enter => {
                        match self.ui_mode {
                            UiMode::Config => {
                                self.prev_ui_mode = self.ui_mode.clone();
                                self.ui_mode = UiMode::EditConfig;
                                self.config_item_being_edited = Some(self.state.config_state.selected().unwrap_or(0));
                                AppReturn::Continue
                            }
                            UiMode::EditConfig => {
                                let config_item_index = self.state.config_state.selected().unwrap_or(0);
                                let config_item_list = AppConfig::to_list(&self.config);
                                let config_item = &config_item_list[config_item_index];
                                // split the config item on : and get the first part
                                let config_item_key = config_item.split(":").collect::<Vec<&str>>()[0];
                                let new_value = self.current_user_input.clone();
                                // if new value is not empty update the config
                                if !new_value.is_empty() {
                                    let config_string = format!("{}: {}", config_item_key, new_value);
                                    let app_config = AppConfig::edit_with_string(&config_string, self);
                                    self.config = app_config.clone();
                                    write_config(&app_config);

                                    // reset everything
                                    self.state.config_state = ListState::default();
                                    self.config_item_being_edited = None;
                                    self.current_user_input = String::new();
                                    self.ui_mode = UiMode::Config;
                                }
                                AppReturn::Continue
                            }
                            UiMode::MainMenu => {
                                let selected = self.state.main_menu_state.selected().unwrap_or(0);
                                let selected_item = MainMenu::from_index(selected);
                                match selected_item {
                                    MainMenuItem::Quit => {
                                        AppReturn::Exit
                                    }
                                    MainMenuItem::Config => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = UiMode::Config;
                                        AppReturn::Continue
                                    }
                                    MainMenuItem::View => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = self.config.default_view.clone();
                                        AppReturn::Continue
                                    }
                                    MainMenuItem::Help => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = UiMode::HelpMenu;
                                        AppReturn::Continue
                                    }
                                    MainMenuItem::LoadSave => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = UiMode::LoadSave;
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
                                        self.boards.push(new_board);
                                        self.ui_mode = self.prev_ui_mode.clone();
                                        self.state.new_board_form = vec![String::new(), String::new()];
                                    } else {
                                        warn!("New board name is empty or already exists");
                                    }
                                    self.ui_mode = self.prev_ui_mode.clone();
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
                                        self.ui_mode = self.prev_ui_mode.clone();
                                        return AppReturn::Continue;
                                    }
                                    // check if due date is empty or is a valid date
                                    let due_date = if new_card_due_date.is_empty() {
                                        Some(FIELD_NOT_SET.to_string())
                                    } else {
                                        match NaiveDate::parse_from_str(&new_card_due_date, "%Y-%m-%d") {
                                            Ok(due_date) => Some(due_date.to_string()),
                                            Err(e) => {
                                                debug!("Invalid due date: {}", e);
                                                debug!("Due date: {}", new_card_due_date);
                                                None
                                            }
                                        }
                                    };
                                    if due_date.is_none() {
                                        warn!("Invalid due date");
                                        self.ui_mode = self.prev_ui_mode.clone();
                                        return AppReturn::Continue;
                                    }
                                    if !new_card_name.is_empty() && !same_name_exists {
                                        let new_card = Card::new(new_card_name, new_card_description, due_date.unwrap().to_string(),
                                            CardPriority::Low, vec![], vec![]);
                                        let current_board = self.boards.iter_mut().find(|board| board.id == current_board_id);
                                        if let Some(current_board) = current_board {
                                            current_board.cards.push(new_card);
                                        } else {
                                            error!("Current board not found");
                                            self.ui_mode = self.prev_ui_mode.clone();
                                            return AppReturn::Continue;
                                        }
                                        self.ui_mode = self.prev_ui_mode.clone();
                                    } else {
                                        warn!("New card name is empty or already exists");
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
                                self.ui_mode = self.config.default_view.clone();
                                AppReturn::Continue
                            }
                            _ => {
                                AppReturn::Continue
                            }
                        }
                    }
                    Action::Hide => {
                        let current_focus = Focus::from_str(self.focus.to_str());
                        let current_ui_mode = self.ui_mode.clone();
                        // hide the current focus by switching to a view where it is not available
                        // for example if current uimode is Title and focus is on Title then switch to Zen
                        if current_ui_mode == UiMode::Zen {
                            self.ui_mode = UiMode::MainMenu;
                            self.main_menu_next();
                        } else if current_ui_mode == UiMode::TitleBody {
                            if current_focus == Focus::Title {
                                self.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        } else if current_ui_mode == UiMode::BodyHelp {
                            if current_focus == Focus::Help {
                                self.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        } else if current_ui_mode == UiMode::BodyLog {
                            if current_focus == Focus::Log {
                                self.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        } else if current_ui_mode == UiMode::TitleBodyHelp {
                            if current_focus == Focus::Title {
                                self.ui_mode = UiMode::BodyHelp;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Help {
                                self.ui_mode = UiMode::TitleBody;
                                self.focus = Focus::Title;
                            }
                            else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        } else if current_ui_mode == UiMode::TitleBodyLog {
                            if current_focus == Focus::Title {
                                self.ui_mode = UiMode::BodyLog;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Log {
                                self.ui_mode = UiMode::TitleBody;
                                self.focus = Focus::Title;
                            }
                            else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        } else if current_ui_mode == UiMode::TitleBodyHelpLog {
                            if current_focus == Focus::Title {
                                self.ui_mode = UiMode::BodyHelpLog;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Help {
                                self.ui_mode = UiMode::TitleBodyLog;
                                self.focus = Focus::Title;
                            }
                            else if current_focus == Focus::Log {
                                self.ui_mode = UiMode::TitleBodyHelp;
                                self.focus = Focus::Title;
                            }
                            else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        } else if current_ui_mode == UiMode::BodyHelpLog {
                            if current_focus == Focus::Help {
                                self.ui_mode = UiMode::BodyLog;
                                self.focus = Focus::Body;
                            }
                            else if current_focus == Focus::Log {
                                self.ui_mode = UiMode::BodyHelp;
                                self.focus = Focus::Body;
                            }
                            else {
                                self.ui_mode = UiMode::MainMenu;
                                self.main_menu_next();
                            }
                        }
                        AppReturn::Continue
                    }
                    Action::SaveState => {
                        self.dispatch(IoEvent::SaveLocalData).await;
                        AppReturn::Continue
                    }
                    Action::NewBoard => {
                        self.state.new_board_form = vec![String::new(), String::new()];
                        self.set_ui_mode(UiMode::NewBoard);
                        self.state.previous_focus = Some(self.focus.clone());
                        AppReturn::Continue
                    }
                    Action::NewCard => {
                        // check if current board is not empty
                        if self.state.current_board_id.is_none() {
                            warn!("No board available to add card to");
                            return AppReturn::Continue;
                        }
                        self.state.new_card_form = vec![String::new(), String::new(), String::new()];
                        self.set_ui_mode(UiMode::NewCard);
                        self.state.previous_focus = Some(self.focus.clone());
                        AppReturn::Continue
                    }
                    Action::Delete => {
                        match self.ui_mode {
                            UiMode::LoadSave => {
                                self.dispatch(IoEvent::DeleteSave).await;
                                AppReturn::Continue
                            }
                            _ => {
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
                                                    info!("Deleted card {}", card_name);
                                                    // remove card_id from self.visible_boards_and_cards if it is there, where visible_boards_and_cards is a BtreeMap of board_id to a vector of card_ids
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
                                                    info!("Deleted board {}", board_name);
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
                    Action::AltDelete => {
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
                                        info!("Deleted board: {}", board_name);
                                    }
                                }
                                AppReturn::Continue
                            },
                            _ => AppReturn::Continue
                        }
                    }
                }
            } else {
                warn!("No action accociated to {}", key);
                AppReturn::Continue
            }
        }
    }
    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }
    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn status(&self) -> &AppStatus {
        &self.state.status
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = Action::all()
        .into();
        if self.ui_mode == UiMode::MainMenu {
            self.main_menu_next();
        } else if self.focus == Focus::NoFocus {
            self.focus = Focus::Body;
        }
        self.state.status = AppStatus::initialized()
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
        self.current_user_input = String::new();
    }
    pub fn set_config_state(&mut self, config_state: ListState) {
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
                if i >= get_available_local_savefiles().len() - 1 {
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
                if i == 0 {
                    get_available_local_savefiles().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.load_save_state.select(Some(i));
    }
    pub fn config_state(&self) -> &ListState {
        &self.state.config_state
    }
    pub fn set_ui_mode(&mut self, ui_mode: UiMode) {
        self.prev_ui_mode = self.ui_mode.clone();
        self.ui_mode = ui_mode;
        let available_focus_targets = self.ui_mode.get_available_targets();
        if !available_focus_targets.contains(&self.focus.to_str().to_string()) {
            // check if available focus targets is empty
            if available_focus_targets.is_empty() {
                self.focus = Focus::NoFocus;
            } else {
                self.focus = Focus::from_str(available_focus_targets[0].as_str());
            }
        }
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
            MainMenuItem::LoadSave => write!(f, "Load Save"),
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

#[derive(Debug, Clone)]
pub struct AppState {
    pub status: AppStatus,
    pub current_board_id: Option<u128>,
    pub current_card_id: Option<u128>,
    pub previous_focus: Option<Focus>,
    pub main_menu_state: ListState,
    pub config_state: ListState,
    pub new_board_form: Vec<String>,
    pub new_card_form: Vec<String>,
    pub load_save_state: ListState,
}

impl Default for AppState {
    fn default() -> AppState {
        AppState {
            status: AppStatus::default(),
            current_board_id: None,
            current_card_id: None,
            previous_focus: None,
            main_menu_state: ListState::default(),
            config_state: ListState::default(),
            new_board_form: vec![String::new(), String::new()],
            new_card_form: vec![String::new(), String::new(), String::new()],
            load_save_state: ListState::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub save_directory: PathBuf,
    pub default_view: UiMode,
    pub always_load_latest_save: bool,
}

impl AppConfig {
    pub fn default() -> Self {
        let save_directory = env::temp_dir().join(SAVE_DIR_NAME);
        let default_view = UiMode::TitleBodyHelpLog;
        Self {
            save_directory: save_directory,
            default_view,
            always_load_latest_save: true,
        }
    }

    pub fn to_list(&self) -> Vec<String> {
        vec![
            format!("save_directory: {}", self.save_directory.to_str().unwrap()),
            format!("default_view: {}", self.default_view.to_string()),
            format!("always_load_latest_save: {}", self.always_load_latest_save),
        ]
    }

    pub fn edit_with_string(change_str: &str, app: &App) -> Self {
        let mut config = app.config.clone();
        let mut lines = change_str.lines();
        while let Some(line) = lines.next() {
            let mut parts = line.split(":");
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            match key {
                "save_directory" => {
                    let new_path = PathBuf::from(value);
                    // check if the new path is valid
                    if new_path.exists() {
                        config.save_directory = new_path;
                    } else {
                        warn!("Invalid path: {}", value);
                    }
                }
                "default_view" => {
                    let new_ui_mode = UiMode::from_string(value);
                    if new_ui_mode.is_some() {
                        config.default_view = new_ui_mode.unwrap();
                    } else {
                        warn!("Invalid UiMode: {}", value);
                        info!("Valid UiModes are: {:?}", UiMode::all());
                    }
                }
                "always_load_latest_save" => {
                    if value.to_lowercase() == "true" {
                        config.always_load_latest_save = true;
                    } else if value.to_lowercase() == "false" {
                        config.always_load_latest_save = false;
                    } else {
                        warn!("Invalid boolean: {}", value);
                    }
                }
                _ => {
                    error!("Invalid key: {}", key);
                    return config;
                }
            }
        }
        config
    }

    pub fn len(&self) -> usize {
        self.to_list().len()
    }
}