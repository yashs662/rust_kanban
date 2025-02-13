use crate::{
    app::{
        actions::Action,
        app_helper::{
            handle_edit_keybinding_mode, handle_general_actions, handle_mouse_action,
            handle_user_input_mode, prepare_config_for_new_app,
        },
        kanban::{Board, Boards, Card, CardPriority, CardStatus},
        state::{AppStatus, Focus, KeyBindingEnum, KeyBindings},
    },
    constants::{
        DEFAULT_CARD_WARNING_DUE_DATE_DAYS, DEFAULT_NO_OF_BOARDS_PER_PAGE,
        DEFAULT_NO_OF_CARDS_PER_BOARD, DEFAULT_TICKRATE, DEFAULT_VIEW, FIELD_NA,
        IO_EVENT_WAIT_TIME, MAX_NO_BOARDS_PER_PAGE, MAX_NO_CARDS_PER_BOARD, MAX_TICKRATE,
        MAX_WARNING_DUE_DATE_DAYS, MIN_NO_BOARDS_PER_PAGE, MIN_NO_CARDS_PER_BOARD, MIN_TICKRATE,
        MIN_WARNING_DUE_DATE_DAYS,
    },
    inputs::{key::Key, mouse::Mouse},
    io::{
        data_handler::{self, get_available_local_save_files, get_default_save_directory},
        io_handler::refresh_visible_boards_and_cards,
        logger::{get_logs, RUST_KANBAN_LOGGER},
        IoEvent,
    },
    ui::{
        text_box::TextBox,
        theme::Theme,
        widgets::{date_time_picker::CalenderType, Widgets},
        PopUp, TextColorOptions, TextModifierOptions, View,
    },
    util::{send_error_toast, send_error_toast_with_duration, send_info_toast, send_warning_toast},
};
use linked_hash_map::LinkedHashMap;
use log::{debug, error, warn};
use ratatui::widgets::TableState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::AppState;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
    vec,
};
use strum::{EnumString, IntoEnumIterator};
use strum_macros::EnumIter;

pub mod actions;
pub mod app_helper;
pub mod kanban;
pub mod state;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionHistory {
    /// card, board_id
    DeleteCard(Card, (u64, u64)),
    /// card, board_id
    CreateCard(Card, (u64, u64)),
    /// board
    DeleteBoard(Board),
    /// card, moved_from_board_id, moved_to_board_id, moved_from_index, moved_to_index
    MoveCardBetweenBoards(Card, (u64, u64), (u64, u64), usize, usize),
    /// board_id, moved_from_index, moved_to_index
    MoveCardWithinBoard((u64, u64), usize, usize),
    /// board
    CreateBoard(Board),
    /// old_card, new_card, board_id
    EditCard(Card, Card, (u64, u64)),
}

#[derive(Default)]
pub struct ActionHistoryManager {
    pub history: Vec<ActionHistory>,
    pub history_index: usize,
}

impl ActionHistoryManager {
    pub fn new_action(&mut self, action: ActionHistory) {
        if self.history_index != self.history.len() {
            self.history.truncate(self.history_index);
        }
        self.history.push(action);
        self.history_index += 1;
    }
    pub fn reset(&mut self) {
        self.history.clear();
        self.history_index = 0;
    }
}

/// Hashmap of board id to a vector of card id's
pub type VisibleBoardsAndCards = LinkedHashMap<(u64, u64), Vec<(u64, u64)>>;

pub struct App<'a> {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Vec<Action>,
    is_loading: bool,
    pub debug_mode: bool,
    pub state: AppState<'a>,
    pub boards: Boards,
    pub filtered_boards: Boards,
    pub preview_boards_and_cards: Option<Boards>,
    pub config: AppConfig,
    pub visible_boards_and_cards: VisibleBoardsAndCards,
    pub last_io_event_time: Option<Instant>,
    pub all_themes: Vec<Theme>,
    pub current_theme: Theme,
    pub action_history_manager: ActionHistoryManager,
    pub main_menu: MainMenu,
    pub widgets: Widgets<'a>,
}

impl App<'_> {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>, debug_mode: bool) -> Self {
        let actions = vec![Action::Quit];
        let is_loading = false;
        let state = AppState::default();
        let boards = Boards::default();
        let filtered_boards = Boards::default();
        let all_themes = Theme::all_default_themes();
        let mut theme = Theme::default();
        let (config, config_errors, toasts) = prepare_config_for_new_app();
        let default_theme = config.default_theme.clone();
        let theme_in_all = all_themes.iter().find(|t| t.name == default_theme);
        if let Some(theme_in_all) = theme_in_all {
            theme = theme_in_all.clone();
        }
        let mut widgets = Widgets::new(
            theme.clone(),
            debug_mode,
            config.date_picker_calender_format.clone(),
        );
        widgets.toast_widget.toasts = toasts;
        let mut app = Self {
            io_tx,
            actions,
            is_loading,
            debug_mode,
            state,
            boards,
            filtered_boards,
            preview_boards_and_cards: None,
            config,
            visible_boards_and_cards: LinkedHashMap::new(),
            last_io_event_time: None,
            all_themes,
            current_theme: theme,
            action_history_manager: ActionHistoryManager::default(),
            main_menu: MainMenu::default(),
            widgets,
        };
        if !config_errors.is_empty() {
            for error in config_errors {
                send_error_toast(&mut app.widgets.toast_widget, error);
            }
        }
        app
    }

    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if self.state.app_status == AppStatus::UserInput {
            handle_user_input_mode(self, key).await
        } else if self.state.app_status == AppStatus::KeyBindMode {
            handle_edit_keybinding_mode(self, key).await
        } else {
            handle_general_actions(self, key).await
        }
    }
    pub async fn dispatch(&mut self, action: IoEvent) {
        self.is_loading = true;
        if self
            .last_io_event_time
            .unwrap_or_else(|| Instant::now() - Duration::from_millis(IO_EVENT_WAIT_TIME + 10))
            + Duration::from_millis(IO_EVENT_WAIT_TIME)
            > Instant::now()
        {
            tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
        }
        self.last_io_event_time = Some(Instant::now());
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            debug!("Error from dispatch {}", e);
            error!("Error in handling request please, restart the app");
            send_error_toast(
                &mut self.widgets.toast_widget,
                "Error in handling request please, restart the app",
            );
        };
    }

    pub async fn handle_mouse(&mut self, mouse_action: Mouse) -> AppReturn {
        if self.config.enable_mouse_support {
            handle_mouse_action(self, mouse_action).await
        } else {
            AppReturn::Continue
        }
    }
    pub fn get_first_keybinding(&self, keybinding_enum: KeyBindingEnum) -> Option<String> {
        self.config
            .keybindings
            .get_keybindings(keybinding_enum)
            .and_then(|keys| keys.first().cloned())
            .map(|key| key.to_string())
    }
    pub fn status(&self) -> &AppStatus {
        &self.state.app_status
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn initialized(&mut self) {
        self.actions = Action::all();
        self.state.set_focus(Focus::Body);
        self.state.app_status = AppStatus::initialized()
    }
    pub fn loaded(&mut self) {
        self.is_loading = false;
    }
    pub fn get_current_focus(&self) -> &Focus {
        &self.state.focus
    }
    pub fn set_config_state(&mut self, config_state: TableState) {
        self.state.app_table_states.config = config_state;
    }
    pub fn get_config_state(&self) -> &TableState {
        &self.state.app_table_states.config
    }
    pub fn undo(&mut self) {
        if self.action_history_manager.history_index == 0 {
            send_error_toast(&mut self.widgets.toast_widget, "No more actions to undo");
        } else {
            let history_index = self.action_history_manager.history_index - 1;
            let history = self.action_history_manager.history[history_index].clone();
            match history {
                ActionHistory::DeleteCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.add_card(card.clone());
                        self.action_history_manager.history_index -= 1;
                        refresh_visible_boards_and_cards(self);
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Undo Delete Card '{}'", card.name),
                        );
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not undo delete card '{}' as the board with id '{:?}' was not found", card.name, board_id)
                        );
                    }
                }
                ActionHistory::CreateCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.remove_card_with_id(card.id);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index -= 1;
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Undo Create Card '{}'", card.name),
                        );
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not undo create card '{}' as the board with id '{:?}' was not found", card.name, board_id)
                        );
                    }
                }
                ActionHistory::MoveCardBetweenBoards(
                    card,
                    moved_from_board_id,
                    moved_to_board_id,
                    moved_from_index,
                    moved_to_index,
                ) => {
                    let moved_to_board = self.boards.get_board_with_id(moved_to_board_id);
                    let moved_from_board = self.boards.get_board_with_id(moved_from_board_id);
                    if moved_to_board.is_none() || moved_from_board.is_none() {
                        debug!("Could not undo move card '{}' as the move to board with id '{:?}' or the move from board with id '{:?}' was not found", card.name, moved_to_board_id, moved_from_board_id);
                        return;
                    }

                    let moved_from_board = moved_from_board.unwrap();
                    if moved_from_index > moved_from_board.cards.len() {
                        debug!("bad index for undo move card, from board {:?}, to board {:?}, from index {}, to index {}", moved_from_board_id, moved_to_board_id, moved_from_index, moved_to_index);
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!(
                                "Could not undo move card '{}' as the index's were invalid",
                                card.name
                            ),
                        );
                    }

                    let moved_to_board = self
                        .boards
                        .get_mut_board_with_id(moved_to_board_id)
                        .unwrap();
                    moved_to_board.cards.remove_card_with_id(card.id);

                    let moved_from_board = self
                        .boards
                        .get_mut_board_with_id(moved_from_board_id)
                        .unwrap();
                    moved_from_board
                        .cards
                        .add_card_at_index(moved_from_index, card.clone());

                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    send_info_toast(
                        &mut self.widgets.toast_widget,
                        &format!("Undo Move Card '{}'", card.name),
                    );
                }
                ActionHistory::MoveCardWithinBoard(board_id, moved_from_index, moved_to_index) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        if moved_from_index >= board.cards.len()
                            || moved_to_index >= board.cards.len()
                        {
                            send_error_toast(
                                &mut self.widgets.toast_widget,
                                &format!(
                                    "Could not undo move card '{}' as the index's were invalid",
                                    FIELD_NA
                                ),
                            );
                            return;
                        }
                        let card_name = board
                            .cards
                            .get_mut_card_with_index(moved_to_index)
                            .unwrap()
                            .name
                            .clone();
                        board.cards.swap(moved_from_index, moved_to_index);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index -= 1;
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Undo Move Card '{}'", card_name),
                        );
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not undo move card '{}' as the board with id '{:?}' was not found",FIELD_NA, board_id)
                        );
                    }
                }
                ActionHistory::DeleteBoard(board) => {
                    self.boards.add_board(board.clone());
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    send_info_toast(
                        &mut self.widgets.toast_widget,
                        &format!("Undo Delete Board '{}'", board.name),
                    );
                }
                ActionHistory::CreateBoard(board) => {
                    self.boards.remove_board_with_id(board.id);
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    send_info_toast(
                        &mut self.widgets.toast_widget,
                        &format!("Undo Create Board '{}'", board.name),
                    );
                }
                ActionHistory::EditCard(old_card, _, board_id) => {
                    let mut card_name = String::new();
                    let mut card_found = false;
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        if let Some(card) = board.cards.get_mut_card_with_id(old_card.id) {
                            *card = old_card.clone();
                            card_name.clone_from(&card.name);
                            card_found = true;
                        } else {
                            send_error_toast(
                                &mut self.widgets.toast_widget,
                                &format!(
                                    "Could not undo edit card '{}' as the card was not found",
                                    old_card.name
                                ),
                            );
                        }
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not undo edit card '{}' as the board with id '{:?}' was not found", old_card.name, board_id)
                        );
                    }
                    if card_found {
                        self.action_history_manager.history_index -= 1;
                    }
                    if !card_name.is_empty() {
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Undo Edit Card '{}'", card_name),
                        );
                        refresh_visible_boards_and_cards(self);
                    }
                }
            }
        }
    }

    pub fn redo(&mut self) {
        if self.action_history_manager.history_index == self.action_history_manager.history.len() {
            send_error_toast(&mut self.widgets.toast_widget, "No more actions to redo");
        } else {
            let history_index = self.action_history_manager.history_index;
            let history = self.action_history_manager.history[history_index].clone();
            match history {
                ActionHistory::DeleteCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.remove_card_with_id(card.id);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Redo Delete Card '{}'", card.name),
                        );
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not redo delete card '{}' as the board with id '{:?}' was not found", card.name, board_id)
                        );
                    }
                }
                ActionHistory::CreateCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.add_card(card.clone());
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Redo Create Card '{}'", card.name),
                        );
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not redo create card '{}' as the board with id '{:?}' was not found", card.name, board_id)
                        );
                    }
                }
                ActionHistory::MoveCardBetweenBoards(
                    card,
                    moved_from_board_id,
                    moved_to_board_id,
                    moved_from_index,
                    moved_to_index,
                ) => {
                    let moved_to_board = self.boards.get_board_with_id(moved_to_board_id);
                    let moved_from_board = self.boards.get_board_with_id(moved_from_board_id);
                    if moved_to_board.is_none() || moved_from_board.is_none() {
                        debug!("Could not undo move card '{}' as the move to board with id '{:?}' or the move from board with id '{:?}' was not found", card.name, moved_to_board_id, moved_from_board_id);
                        return;
                    }

                    let moved_to_board = moved_to_board.unwrap();
                    if moved_to_index > moved_to_board.cards.len() {
                        debug!("bad index for redo move card, from board {:?}, to board {:?}, from index {}, to index {}", moved_from_board_id, moved_to_board_id, moved_from_index, moved_to_index);
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!(
                                "Could not redo move card '{}' as the index's were invalid",
                                card.name
                            ),
                        );
                        return;
                    }

                    let moved_from_board = self
                        .boards
                        .get_mut_board_with_id(moved_from_board_id)
                        .unwrap();
                    moved_from_board.cards.remove_card_with_id(card.id);

                    let moved_to_board = self
                        .boards
                        .get_mut_board_with_id(moved_to_board_id)
                        .unwrap();
                    moved_to_board
                        .cards
                        .add_card_at_index(moved_to_index, card.clone());

                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    send_info_toast(
                        &mut self.widgets.toast_widget,
                        &format!("Redo Move Card '{}'", card.name),
                    );
                }
                ActionHistory::MoveCardWithinBoard(board_id, moved_from_index, moved_to_index) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        if moved_from_index >= board.cards.len()
                            || moved_to_index >= board.cards.len()
                        {
                            send_error_toast(
                                &mut self.widgets.toast_widget,
                                &format!(
                                    "Could not redo move card '{}' as the index's were invalid",
                                    FIELD_NA
                                ),
                            );
                            return;
                        }
                        let card_name = board
                            .cards
                            .get_card_with_index(moved_to_index)
                            .unwrap()
                            .name
                            .clone();
                        board.cards.swap(moved_from_index, moved_to_index);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Redo Move Card '{}'", card_name),
                        );
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not redo move card '{}' as the board with id '{:?}' was not found", FIELD_NA, board_id)
                        );
                    }
                }
                ActionHistory::DeleteBoard(board) => {
                    self.boards.remove_board_with_id(board.id);
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    send_info_toast(
                        &mut self.widgets.toast_widget,
                        &format!("Redo Delete Board '{}'", board.name),
                    );
                }
                ActionHistory::CreateBoard(board) => {
                    self.boards.add_board(board.clone());
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    send_info_toast(
                        &mut self.widgets.toast_widget,
                        &format!("Redo Create Board '{}'", board.name),
                    );
                }
                ActionHistory::EditCard(_, new_card, board_id) => {
                    let mut card_name = String::new();
                    let mut card_found = false;
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        if let Some(card) = board.cards.get_mut_card_with_id(new_card.id) {
                            *card = new_card.clone();
                            card_name.clone_from(&card.name);
                            card_found = true;
                        } else {
                            send_error_toast(
                                &mut self.widgets.toast_widget,
                                &format!(
                                    "Could not redo edit card '{}' as the card was not found",
                                    new_card.name
                                ),
                            );
                        }
                    } else {
                        send_error_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Could not redo edit card '{}' as the board with id '{:?}' was not found", new_card.name, board_id)
                        );
                    }
                    if card_found {
                        self.action_history_manager.history_index += 1;
                    }
                    if !card_name.is_empty() {
                        send_info_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Redo Edit Card '{}'", card_name),
                        );
                        refresh_visible_boards_and_cards(self);
                    }
                }
            }
        }
    }
    fn select_next(current_index: Option<usize>, items_len: usize) -> usize {
        match current_index {
            Some(i) => {
                if i >= items_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        }
    }
    fn select_previous(current_index: Option<usize>, items_len: usize) -> usize {
        match current_index {
            Some(i) => {
                if i == 0 {
                    items_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        }
    }
    pub fn config_next(&mut self) {
        let i = Self::select_next(
            self.state.app_table_states.config.selected(),
            self.config.to_view_list().len(),
        );
        self.state.app_table_states.config.select(Some(i));
    }
    pub fn config_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_table_states.config.selected(),
            self.config.to_view_list().len(),
        );
        self.state.app_table_states.config.select(Some(i));
    }
    pub fn main_menu_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.main_menu.selected(),
            self.main_menu.all().len(),
        );
        self.state.app_list_states.main_menu.select(Some(i));
    }
    pub fn main_menu_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.main_menu.selected(),
            self.main_menu.all().len(),
        );
        self.state.app_list_states.main_menu.select(Some(i));
    }
    pub fn load_save_next(&mut self, cloud_mode: bool) {
        let items_len = if cloud_mode {
            self.state.cloud_data.as_ref().map_or(0, |data| data.len())
        } else {
            get_available_local_save_files(&self.config).map_or(0, |files| files.len())
        };
        let i = Self::select_next(self.state.app_list_states.load_save.selected(), items_len);
        self.state.app_list_states.load_save.select(Some(i));
    }
    pub fn load_save_prv(&mut self, cloud_mode: bool) {
        let items_len = if cloud_mode {
            self.state.cloud_data.as_ref().map_or(0, |data| data.len())
        } else {
            get_available_local_save_files(&self.config).map_or(0, |files| files.len())
        };
        let i = Self::select_previous(self.state.app_list_states.load_save.selected(), items_len);
        self.state.app_list_states.load_save.select(Some(i));
    }
    pub fn edit_keybindings_next(&mut self) {
        let keybinding_count = self.config.keybindings.iter().count();
        let i = Self::select_next(
            self.state.app_table_states.edit_keybindings.selected(),
            keybinding_count,
        );
        self.state.app_table_states.edit_keybindings.select(Some(i));
    }
    pub fn edit_keybindings_prv(&mut self) {
        let keybinding_count = self.config.keybindings.iter().count();
        let i = Self::select_previous(
            self.state.app_table_states.edit_keybindings.selected(),
            keybinding_count,
        );
        self.state.app_table_states.edit_keybindings.select(Some(i));
    }
    pub fn help_next(&mut self) {
        let all_keybindings: Vec<_> = self.config.keybindings.iter().collect();
        let i = Self::select_next(
            self.state.app_table_states.help.selected(),
            all_keybindings.len() / 2,
        );
        self.state.app_table_states.help.select(Some(i));
    }
    pub fn help_prv(&mut self) {
        let all_keybindings: Vec<_> = self.config.keybindings.iter().collect();
        let i = Self::select_previous(
            self.state.app_table_states.help.selected(),
            all_keybindings.len() / 2,
        );
        self.state.app_table_states.help.select(Some(i));
    }
    pub fn select_default_view_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.default_view.selected(),
            View::all_views_as_string().len(),
        );
        self.state.app_list_states.default_view.select(Some(i));
    }
    pub fn select_default_view_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.default_view.selected(),
            View::all_views_as_string().len(),
        );
        self.state.app_list_states.default_view.select(Some(i));
    }
    pub fn command_palette_command_search_prv(&mut self) {
        if let Some(results) = &self.widgets.command_palette.command_search_results {
            let i = Self::select_previous(
                self.state
                    .app_list_states
                    .command_palette_command_search
                    .selected(),
                results.len(),
            );
            self.state
                .app_list_states
                .command_palette_command_search
                .select(Some(i));
        }
    }
    pub fn command_palette_command_search_next(&mut self) {
        if let Some(results) = &self.widgets.command_palette.command_search_results {
            let i = Self::select_next(
                self.state
                    .app_list_states
                    .command_palette_command_search
                    .selected(),
                results.len(),
            );
            self.state
                .app_list_states
                .command_palette_command_search
                .select(Some(i));
        }
    }
    pub fn command_palette_card_search_next(&mut self) {
        if let Some(results) = &self.widgets.command_palette.card_search_results {
            let i = Self::select_next(
                self.state
                    .app_list_states
                    .command_palette_card_search
                    .selected(),
                results.len(),
            );
            self.state
                .app_list_states
                .command_palette_card_search
                .select(Some(i));
        }
    }
    pub fn command_palette_card_search_prv(&mut self) {
        if let Some(results) = &self.widgets.command_palette.card_search_results {
            let i = Self::select_previous(
                self.state
                    .app_list_states
                    .command_palette_card_search
                    .selected(),
                results.len(),
            );
            self.state
                .app_list_states
                .command_palette_card_search
                .select(Some(i));
        }
    }
    pub fn command_palette_board_search_next(&mut self) {
        if let Some(results) = &self.widgets.command_palette.board_search_results {
            let i = Self::select_next(
                self.state
                    .app_list_states
                    .command_palette_board_search
                    .selected(),
                results.len(),
            );
            self.state
                .app_list_states
                .command_palette_board_search
                .select(Some(i));
        }
    }
    pub fn command_palette_board_search_prv(&mut self) {
        if let Some(results) = &self.widgets.command_palette.board_search_results {
            let i = Self::select_previous(
                self.state
                    .app_list_states
                    .command_palette_board_search
                    .selected(),
                results.len(),
            );
            self.state
                .app_list_states
                .command_palette_board_search
                .select(Some(i));
        }
    }
    pub fn select_card_status_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.card_status_selector.selected(),
            CardStatus::all().len(),
        );
        self.state
            .app_list_states
            .card_status_selector
            .select(Some(i));
    }
    pub fn select_card_status_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.card_status_selector.selected(),
            CardStatus::all().len(),
        );
        self.state
            .app_list_states
            .card_status_selector
            .select(Some(i));
    }
    pub fn select_change_theme_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.theme_selector.selected(),
            self.all_themes.len(),
        );
        self.state.app_list_states.theme_selector.select(Some(i));
        self.current_theme = self.all_themes[i].clone();
    }
    pub fn select_change_theme_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.theme_selector.selected(),
            self.all_themes.len(),
        );
        self.state.app_list_states.theme_selector.select(Some(i));
        self.current_theme = self.all_themes[i].clone();
    }
    pub fn select_create_theme_next(&mut self) {
        // popup doesn't matter here, as we only want the length of the rows
        let theme_rows_len = Theme::default().to_rows(self, true).1.len();
        let i = Self::select_next(
            self.state.app_table_states.theme_editor.selected(),
            theme_rows_len,
        );
        self.state.app_table_states.theme_editor.select(Some(i));
    }
    pub fn select_create_theme_prv(&mut self) {
        // popup doesn't matter here, as we only want the length of the rows
        let theme_rows_len = Theme::default().to_rows(self, true).1.len();
        let i = Self::select_previous(
            self.state.app_table_states.theme_editor.selected(),
            theme_rows_len,
        );
        self.state.app_table_states.theme_editor.select(Some(i));
    }
    pub fn select_edit_style_fg_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.edit_specific_style[0].selected(),
            TextColorOptions::iter().count(),
        );
        self.state.app_list_states.edit_specific_style[0].select(Some(i));
    }
    pub fn select_edit_style_fg_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.edit_specific_style[0].selected(),
            TextColorOptions::iter().count(),
        );
        self.state.app_list_states.edit_specific_style[0].select(Some(i));
    }
    pub fn select_edit_style_bg_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.edit_specific_style[1].selected(),
            TextColorOptions::iter().count(),
        );
        self.state.app_list_states.edit_specific_style[1].select(Some(i));
    }
    pub fn select_edit_style_bg_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.edit_specific_style[1].selected(),
            TextColorOptions::iter().count(),
        );
        self.state.app_list_states.edit_specific_style[1].select(Some(i));
    }
    pub fn select_edit_style_modifier_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.edit_specific_style[2].selected(),
            TextModifierOptions::iter().count(),
        );
        self.state.app_list_states.edit_specific_style[2].select(Some(i));
    }
    pub fn select_edit_style_modifier_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.edit_specific_style[2].selected(),
            TextModifierOptions::iter().count(),
        );
        self.state.app_list_states.edit_specific_style[2].select(Some(i));
    }
    pub fn select_card_priority_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.card_priority_selector.selected(),
            CardPriority::all().len(),
        );
        self.state
            .app_list_states
            .card_priority_selector
            .select(Some(i));
    }
    pub fn select_card_priority_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.card_priority_selector.selected(),
            CardPriority::all().len(),
        );
        self.state
            .app_list_states
            .card_priority_selector
            .select(Some(i));
    }
    pub fn filter_by_tag_popup_next(&mut self) {
        let all_tags_len = self
            .state
            .all_available_tags
            .as_ref()
            .map_or(0, |tags| tags.len());
        if all_tags_len > 0 {
            let i = Self::select_next(
                self.state.app_list_states.filter_by_tag_list.selected(),
                all_tags_len,
            );
            self.state
                .app_list_states
                .filter_by_tag_list
                .select(Some(i));
        }
    }
    pub fn filter_by_tag_popup_prv(&mut self) {
        let all_tags_len = self
            .state
            .all_available_tags
            .as_ref()
            .map_or(0, |tags| tags.len());
        if all_tags_len > 0 {
            let i = Self::select_previous(
                self.state.app_list_states.filter_by_tag_list.selected(),
                all_tags_len,
            );
            self.state
                .app_list_states
                .filter_by_tag_list
                .select(Some(i));
        }
    }
    pub fn change_date_format_popup_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.date_format_selector.selected(),
            DateTimeFormat::get_all_date_formats().len(),
        );
        self.state
            .app_list_states
            .date_format_selector
            .select(Some(i));
    }
    pub fn change_date_format_popup_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.date_format_selector.selected(),
            DateTimeFormat::get_all_date_formats().len(),
        );
        self.state
            .app_list_states
            .date_format_selector
            .select(Some(i));
    }
    pub fn log_next(&mut self) {
        let total_logs = get_logs().len();
        let mut hot_log = RUST_KANBAN_LOGGER.hot_log.lock();
        let i = Self::select_next(hot_log.state.selected(), total_logs);
        hot_log.state.select(Some(i));
    }
    pub fn log_prv(&mut self) {
        let total_logs = get_logs().len();
        let mut hot_log = RUST_KANBAN_LOGGER.hot_log.lock();
        let i = Self::select_previous(hot_log.state.selected(), total_logs);
        hot_log.state.select(Some(i));
    }
    pub fn tag_picker_next(&mut self) {
        let i = Self::select_next(
            self.state.app_list_states.tag_picker.selected(),
            self.widgets.tag_picker.available_tags.len(),
        );
        self.state.app_list_states.tag_picker.select(Some(i));
    }
    pub fn tag_picker_prv(&mut self) {
        let i = Self::select_previous(
            self.state.app_list_states.tag_picker.selected(),
            self.widgets.tag_picker.available_tags.len(),
        );
        self.state.app_list_states.tag_picker.select(Some(i));
    }
    pub fn set_popup(&mut self, popup: PopUp) {
        if self.state.z_stack.contains(&popup) {
            debug!(
                "Popup already set: {:?}, z_stack {:?}",
                popup, self.state.z_stack
            );
            return;
        }
        self.state.z_stack.push(popup);
        let available_focus_targets = popup.get_available_targets();
        if !available_focus_targets.contains(&self.state.focus) {
            if available_focus_targets.is_empty() {
                self.state.set_focus(Focus::NoFocus);
            } else if available_focus_targets.len() > 1
                && available_focus_targets[0] == Focus::Title
            {
                self.state.set_focus(available_focus_targets[1]);
            } else {
                self.state.set_focus(available_focus_targets[0]);
            }
        }
        match popup {
            PopUp::ViewCard => {
                if self.state.current_board_id.is_none() || self.state.current_card_id.is_none() {
                    send_error_toast_with_duration(
                        &mut self.widgets.toast_widget,
                        "No card selected",
                        Duration::from_secs(1),
                    );
                    return;
                }
                if let Some(current_board) = self
                    .boards
                    .get_board_with_id(self.state.current_board_id.unwrap())
                {
                    if let Some(current_card) = current_board
                        .cards
                        .get_card_with_id(self.state.current_card_id.unwrap())
                    {
                        self.state.set_focus(Focus::CardName);
                        self.state.text_buffers.card_name =
                            TextBox::from_string_with_newline_sep(current_card.name.clone(), true);
                        self.state.text_buffers.card_description =
                            TextBox::from_string_with_newline_sep(
                                current_card.description.clone(),
                                false,
                            );
                    } else {
                        send_error_toast_with_duration(
                            &mut self.widgets.toast_widget,
                            "No card selected",
                            Duration::from_secs(1),
                        );
                    }
                } else {
                    send_error_toast_with_duration(
                        &mut self.widgets.toast_widget,
                        "No board selected",
                        Duration::from_secs(1),
                    );
                }
            }
            PopUp::CommandPalette => {
                self.widgets.command_palette.reset(&mut self.state);
                self.state.app_status = AppStatus::UserInput;
                self.state.set_focus(Focus::CommandPaletteCommand);
            }
            PopUp::CardStatusSelector => {
                self.state.set_focus(Focus::ChangeCardStatusPopup);
            }
            PopUp::CardPrioritySelector => {
                self.state.set_focus(Focus::ChangeCardPriorityPopup);
            }
            PopUp::EditGeneralConfig => {
                self.state.set_focus(Focus::EditGeneralConfigPopup);
            }
            PopUp::CustomHexColorPromptBG | PopUp::CustomHexColorPromptFG => {
                self.state.set_focus(Focus::TextInput);
                self.state.app_status = AppStatus::UserInput;
            }
            PopUp::DateTimePicker => {
                self.widgets.date_time_picker.open_date_picker();
            }
            _ => {
                debug!("No special logic for setting popup: {:?}", popup);
            }
        }
    }

    pub fn close_popup(&mut self) {
        if let Some(popup) = self.state.z_stack.pop() {
            match popup {
                PopUp::CustomHexColorPromptBG | PopUp::CustomHexColorPromptFG => {
                    self.state.app_status = AppStatus::Initialized;
                }
                PopUp::ViewCard => {
                    self.state.app_status = AppStatus::Initialized;
                    if self.state.card_being_edited.is_some() {
                        self.set_popup(PopUp::ConfirmDiscardCardChanges);
                    }
                }
                PopUp::ConfirmDiscardCardChanges => {
                    self.state.app_status = AppStatus::Initialized;
                    if let Some(card) = &self.state.card_being_edited {
                        warn!("Discarding changes to card '{}'", card.1.name);
                        send_warning_toast(
                            &mut self.widgets.toast_widget,
                            &format!("Discarding changes to card '{}'", card.1.name),
                        );
                        self.state.card_being_edited = None;
                    }
                }
                PopUp::DateTimePicker => {
                    self.widgets.date_time_picker.close_date_picker();
                }
                _ => {}
            }
        }
    }

    pub fn set_view(&mut self, view: View) {
        if let Some(prv_view) = self.state.prev_view {
            if prv_view == view {
                self.state.prev_view = None;
            } else {
                self.state.prev_view = Some(self.state.current_view);
            }
        } else {
            self.state.prev_view = Some(self.state.current_view);
        }
        self.state.current_view = view;
        let available_focus_targets = self.state.current_view.get_available_targets();
        if !available_focus_targets.contains(&self.state.focus) {
            if available_focus_targets.is_empty() {
                self.state.set_focus(Focus::NoFocus);
            } else if available_focus_targets.len() > 1
                && available_focus_targets[0] == Focus::Title
            {
                self.state.set_focus(available_focus_targets[1]);
            } else {
                self.state.set_focus(available_focus_targets[0]);
            }
        }
        match view {
            View::Login => {
                self.state.text_buffers.email_id.reset();
                self.state.text_buffers.password.reset();
            }
            View::SignUp => {
                self.state.text_buffers.email_id.reset();
                self.state.text_buffers.password.reset();
                self.state.text_buffers.confirm_password.reset();
            }
            View::ResetPassword => {
                self.state.text_buffers.email_id.reset();
                self.state.text_buffers.password.reset();
                self.state.text_buffers.confirm_password.reset();
                self.state.text_buffers.reset_password_link.reset();
            }
            View::CreateTheme => {
                self.state.text_buffers.general_config.reset();
                self.state.app_table_states.theme_editor.select(Some(0));
            }
            View::ConfigMenu => self.state.app_table_states.config.select(Some(0)),
            _ => {
                debug!("No special logic for setting view: {:?}", view);
            }
        }
    }

    pub fn calculate_tags(&self) -> Vec<(String, u32)> {
        let mut tags: Vec<(String, String)> = vec![];
        for board in self.boards.get_boards() {
            for card in board.cards.get_all_cards() {
                for tag in &card.tags {
                    if tag.is_empty() {
                        continue;
                    }
                    tags.push((tag.clone(), tag.to_lowercase()));
                }
            }
        }

        let count_hash: HashMap<String, (String, u32)> =
            tags.iter()
                .fold(HashMap::new(), |mut acc, (original, lower)| {
                    let entry = acc.entry(lower.clone()).or_insert((original.clone(), 0));
                    entry.1 += 1;
                    acc
                });

        let mut tags: Vec<(String, u32)> = count_hash
            .iter()
            .map(|(_, (original, count))| (original.clone(), *count))
            .collect();

        tags.sort_by(|a, b| {
            if a.1 == b.1 {
                a.0.to_lowercase().cmp(&b.0.to_lowercase())
            } else {
                b.1.cmp(&a.1)
            }
        });

        tags
    }
}

// TODO: Refactor to keep all structs and enums separate from other code (maybe? think about this)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MainMenuItem {
    View,
    Config,
    Help,
    LoadSaveLocal,
    LoadSaveCloud,
    Quit,
}

impl Display for MainMenuItem {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MainMenuItem::View => write!(f, "View your Boards"),
            MainMenuItem::Config => write!(f, "Configure"),
            MainMenuItem::Help => write!(f, "Help"),
            MainMenuItem::LoadSaveLocal => write!(f, "Load a Save (local)"),
            MainMenuItem::LoadSaveCloud => write!(f, "Load a Save (cloud)"),
            MainMenuItem::Quit => write!(f, "Quit"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MainMenu {
    pub items: Vec<MainMenuItem>,
    pub logged_in: bool,
}

impl Default for MainMenu {
    fn default() -> Self {
        MainMenu {
            items: vec![
                MainMenuItem::View,
                MainMenuItem::Config,
                MainMenuItem::Help,
                MainMenuItem::LoadSaveLocal,
                MainMenuItem::Quit,
            ],
            logged_in: false,
        }
    }
}

impl MainMenu {
    pub fn all(&mut self) -> Vec<MainMenuItem> {
        if self.logged_in {
            let return_vec = vec![
                MainMenuItem::View,
                MainMenuItem::Config,
                MainMenuItem::Help,
                MainMenuItem::LoadSaveLocal,
                MainMenuItem::LoadSaveCloud,
                MainMenuItem::Quit,
            ];
            self.items.clone_from(&return_vec);
            return_vec
        } else {
            let return_vec = vec![
                MainMenuItem::View,
                MainMenuItem::Config,
                MainMenuItem::Help,
                MainMenuItem::LoadSaveLocal,
                MainMenuItem::Quit,
            ];
            self.items.clone_from(&return_vec);
            return_vec
        }
    }

    pub fn from_index(&self, index: usize) -> MainMenuItem {
        if self.logged_in {
            match index {
                0 => MainMenuItem::View,
                1 => MainMenuItem::Config,
                2 => MainMenuItem::Help,
                3 => MainMenuItem::LoadSaveLocal,
                4 => MainMenuItem::LoadSaveCloud,
                5 => MainMenuItem::Quit,
                _ => MainMenuItem::Quit,
            }
        } else {
            match index {
                0 => MainMenuItem::View,
                1 => MainMenuItem::Config,
                2 => MainMenuItem::Help,
                3 => MainMenuItem::LoadSaveLocal,
                4 => MainMenuItem::Quit,
                _ => MainMenuItem::Quit,
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq, EnumString)]
pub enum DateTimeFormat {
    DayMonthYear,
    #[default]
    DayMonthYearTime,
    MonthDayYear,
    MonthDayYearTime,
    YearMonthDay,
    YearMonthDayTime,
}

impl DateTimeFormat {
    pub fn to_human_readable_string(&self) -> &str {
        match self {
            DateTimeFormat::DayMonthYear => "DD/MM/YYYY",
            DateTimeFormat::DayMonthYearTime => "DD/MM/YYYY-HH:MM:SS",
            DateTimeFormat::MonthDayYear => "MM/DD/YYYY",
            DateTimeFormat::MonthDayYearTime => "MM/DD/YYYY-HH:MM:SS",
            DateTimeFormat::YearMonthDay => "YYYY/MM/DD",
            DateTimeFormat::YearMonthDayTime => "YYYY/MM/DD-HH:MM:SS",
        }
    }
    pub fn to_parser_string(&self) -> &str {
        match self {
            DateTimeFormat::DayMonthYear => "%d/%m/%Y",
            DateTimeFormat::DayMonthYearTime => "%d/%m/%Y-%H:%M:%S",
            DateTimeFormat::MonthDayYear => "%m/%d/%Y",
            DateTimeFormat::MonthDayYearTime => "%m/%d/%Y-%H:%M:%S",
            DateTimeFormat::YearMonthDay => "%Y/%m/%d",
            DateTimeFormat::YearMonthDayTime => "%Y/%m/%d-%H:%M:%S",
        }
    }
    pub fn from_json_string(json_string: &str) -> Option<DateTimeFormat> {
        match DateTimeFormat::from_str(json_string) {
            Ok(date_time_format) => Some(date_time_format),
            Err(_) => None,
        }
    }
    pub fn from_human_readable_string(human_readable_string: &str) -> Option<DateTimeFormat> {
        match human_readable_string {
            "DD/MM/YYYY" => Some(DateTimeFormat::DayMonthYear),
            "DD/MM/YYYY-HH:MM:SS" => Some(DateTimeFormat::DayMonthYearTime),
            "MM/DD/YYYY" => Some(DateTimeFormat::MonthDayYear),
            "MM/DD/YYYY-HH:MM:SS" => Some(DateTimeFormat::MonthDayYearTime),
            "YYYY/MM/DD" => Some(DateTimeFormat::YearMonthDay),
            "YYYY/MM/DD-HH:MM:SS" => Some(DateTimeFormat::YearMonthDayTime),
            _ => None,
        }
    }
    pub fn get_all_date_formats() -> Vec<DateTimeFormat> {
        vec![
            DateTimeFormat::DayMonthYear,
            DateTimeFormat::DayMonthYearTime,
            DateTimeFormat::MonthDayYear,
            DateTimeFormat::MonthDayYearTime,
            DateTimeFormat::YearMonthDay,
            DateTimeFormat::YearMonthDayTime,
        ]
    }
    pub fn all_formats_with_time() -> Vec<DateTimeFormat> {
        vec![
            DateTimeFormat::DayMonthYearTime,
            DateTimeFormat::MonthDayYearTime,
            DateTimeFormat::YearMonthDayTime,
        ]
    }
    pub fn all_formats_without_time() -> Vec<DateTimeFormat> {
        vec![
            DateTimeFormat::DayMonthYear,
            DateTimeFormat::MonthDayYear,
            DateTimeFormat::YearMonthDay,
        ]
    }
    pub fn add_time_to_date_format(date_format: DateTimeFormat) -> DateTimeFormat {
        match date_format {
            DateTimeFormat::DayMonthYear => DateTimeFormat::DayMonthYearTime,
            DateTimeFormat::MonthDayYear => DateTimeFormat::MonthDayYearTime,
            DateTimeFormat::YearMonthDay => DateTimeFormat::YearMonthDayTime,
            _ => date_format,
        }
    }
    pub fn remove_time_from_date_format(date_format: DateTimeFormat) -> DateTimeFormat {
        match date_format {
            DateTimeFormat::DayMonthYearTime => DateTimeFormat::DayMonthYear,
            DateTimeFormat::MonthDayYearTime => DateTimeFormat::MonthDayYear,
            DateTimeFormat::YearMonthDayTime => DateTimeFormat::YearMonthDay,
            _ => date_format,
        }
    }
}

impl Display for DateTimeFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_human_readable_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub always_load_last_save: bool,
    pub auto_login: bool,
    pub date_time_format: DateTimeFormat,
    pub default_theme: String,
    pub default_view: View,
    pub disable_animations: bool,
    pub disable_scroll_bar: bool,
    pub enable_mouse_support: bool,
    pub keybindings: KeyBindings,
    pub no_of_boards_to_show: u16,
    pub no_of_cards_to_show: u16,
    pub date_picker_calender_format: CalenderType,
    pub save_directory: PathBuf,
    pub save_on_exit: bool,
    pub show_line_numbers: bool,
    pub tickrate: u16,
    pub warning_delta: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_view = DEFAULT_VIEW;
        let default_theme = Theme::default();
        Self {
            always_load_last_save: true,
            auto_login: true,
            date_time_format: DateTimeFormat::default(),
            default_theme: default_theme.name,
            default_view,
            disable_animations: false,
            disable_scroll_bar: false,
            enable_mouse_support: true,
            keybindings: KeyBindings::default(),
            no_of_boards_to_show: DEFAULT_NO_OF_BOARDS_PER_PAGE,
            no_of_cards_to_show: DEFAULT_NO_OF_CARDS_PER_BOARD,
            date_picker_calender_format: CalenderType::default(),
            save_directory: get_default_save_directory(),
            save_on_exit: true,
            show_line_numbers: true,
            tickrate: DEFAULT_TICKRATE,
            warning_delta: DEFAULT_CARD_WARNING_DUE_DATE_DAYS,
        }
    }
}

impl AppConfig {
    pub fn to_view_list(&self) -> Vec<Vec<String>> {
        // Custom ordering
        let mut view_list = ConfigEnum::iter()
            .map(|enum_variant| {
                let (value, index) = match enum_variant {
                    ConfigEnum::SaveDirectory => {
                        (self.save_directory.to_string_lossy().to_string(), 0)
                    }
                    ConfigEnum::DefaultView => (self.default_view.to_string(), 1),
                    ConfigEnum::AlwaysLoadLastSave => (self.always_load_last_save.to_string(), 2),
                    ConfigEnum::SaveOnExit => (self.save_on_exit.to_string(), 3),
                    ConfigEnum::DisableScrollBar => (self.disable_scroll_bar.to_string(), 4),
                    ConfigEnum::DisableAnimations => (self.disable_animations.to_string(), 5),
                    ConfigEnum::AutoLogin => (self.auto_login.to_string(), 6),
                    ConfigEnum::ShowLineNumbers => (self.show_line_numbers.to_string(), 7),
                    ConfigEnum::EnableMouseSupport => (self.enable_mouse_support.to_string(), 8),
                    ConfigEnum::WarningDelta => (self.warning_delta.to_string(), 9),
                    ConfigEnum::Tickrate => (self.tickrate.to_string(), 10),
                    ConfigEnum::NoOfCardsToShow => (self.no_of_cards_to_show.to_string(), 11),
                    ConfigEnum::NoOfBoardsToShow => (self.no_of_boards_to_show.to_string(), 12),
                    ConfigEnum::DatePickerCalenderFormat => {
                        (self.date_picker_calender_format.to_string(), 13)
                    }
                    ConfigEnum::DefaultTheme => (self.default_theme.clone(), 14),
                    ConfigEnum::DateFormat => (self.date_time_format.to_string(), 15),
                    ConfigEnum::Keybindings => ("".to_string(), 16),
                };
                (enum_variant.to_string(), value.to_string(), index)
            })
            .collect::<Vec<(String, String, usize)>>();

        view_list.sort_by(|a, b| a.2.cmp(&b.2));
        view_list
            .iter()
            .map(|(key, value, _)| vec![key.to_owned(), value.to_owned()])
            .collect::<Vec<Vec<String>>>()
    }

    pub fn get_value_as_string(&self, config_enum: ConfigEnum) -> String {
        match config_enum {
            ConfigEnum::AlwaysLoadLastSave => self.always_load_last_save.to_string(),
            ConfigEnum::AutoLogin => self.auto_login.to_string(),
            ConfigEnum::DateFormat => self.date_time_format.to_string(),
            ConfigEnum::DefaultTheme => self.default_theme.clone(),
            ConfigEnum::DefaultView => self.default_view.to_string(),
            ConfigEnum::DisableAnimations => self.disable_animations.to_string(),
            ConfigEnum::DisableScrollBar => self.disable_scroll_bar.to_string(),
            ConfigEnum::EnableMouseSupport => self.enable_mouse_support.to_string(),
            ConfigEnum::Keybindings => {
                // This should never be called
                debug!("Keybindings should not be called from get_value_as_str");
                "".to_string()
            }
            ConfigEnum::NoOfBoardsToShow => self.no_of_boards_to_show.to_string(),
            ConfigEnum::NoOfCardsToShow => self.no_of_cards_to_show.to_string(),
            ConfigEnum::DatePickerCalenderFormat => self.date_picker_calender_format.to_string(),
            ConfigEnum::SaveDirectory => self.save_directory.to_string_lossy().to_string(),
            ConfigEnum::SaveOnExit => self.save_on_exit.to_string(),
            ConfigEnum::ShowLineNumbers => self.show_line_numbers.to_string(),
            ConfigEnum::Tickrate => self.tickrate.to_string(),
            ConfigEnum::WarningDelta => self.warning_delta.to_string(),
        }
    }

    pub fn get_toggled_value_as_string(&self, config_enum: ConfigEnum) -> String {
        match config_enum {
            ConfigEnum::AlwaysLoadLastSave => (!self.always_load_last_save).to_string(),
            ConfigEnum::AutoLogin => (!self.auto_login).to_string(),
            ConfigEnum::DisableAnimations => (!self.disable_animations).to_string(),
            ConfigEnum::DisableScrollBar => (!self.disable_scroll_bar).to_string(),
            ConfigEnum::EnableMouseSupport => (!self.enable_mouse_support).to_string(),
            ConfigEnum::SaveOnExit => (!self.save_on_exit).to_string(),
            ConfigEnum::ShowLineNumbers => (!self.show_line_numbers).to_string(),
            ConfigEnum::DatePickerCalenderFormat => match self.date_picker_calender_format {
                CalenderType::MondayFirst => CalenderType::SundayFirst.to_string(),
                CalenderType::SundayFirst => CalenderType::MondayFirst.to_string(),
            },
            _ => {
                debug!("Invalid config enum to toggle: {}", config_enum);
                "".to_string()
            }
        }
    }

    pub fn edit_config(app: &mut App, config_enum: ConfigEnum, edited_value: &str) {
        let mut config_copy = app.config.clone();
        let result = config_enum.edit_config(&mut config_copy, edited_value);
        if result.is_ok() {
            let write_status = data_handler::write_config(&config_copy);
            if write_status.is_ok() {
                app.config = config_copy;
                send_info_toast(&mut app.widgets.toast_widget, "Config updated");
            } else {
                send_error_toast(
                    &mut app.widgets.toast_widget,
                    "Could not write to config file",
                );
            }
        } else {
            let error_message = format!("Could not edit config: {}", result.unwrap_err());
            error!("{}", error_message);
            send_error_toast(&mut app.widgets.toast_widget, &error_message);
        }
    }

    pub fn edit_keybinding(
        &mut self,
        key_index: usize,
        value: &[Key],
    ) -> Result<KeyBindingEnum, String> {
        let current_bindings = &self.keybindings;

        let mut key_list = vec![];
        for (k, v) in current_bindings.iter() {
            key_list.push((k, v));
        }
        if key_index >= key_list.len() {
            debug!("Invalid key index: {}", key_index);
            error!("Unable to edit keybinding");
            return Err("Unable to edit keybinding 😢 ".to_string());
        }

        key_list.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

        let (key, _) = key_list[key_index];

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
            KeyBindingEnum::Accept => self.keybindings.accept = value.to_vec(),
            KeyBindingEnum::ChangeCardStatusToActive => {
                self.keybindings.change_card_status_to_active = value.to_vec();
            }
            KeyBindingEnum::ChangeCardStatusToCompleted => {
                self.keybindings.change_card_status_to_completed = value.to_vec();
            }
            KeyBindingEnum::ChangeCardStatusToStale => {
                self.keybindings.change_card_status_to_stale = value.to_vec();
            }
            KeyBindingEnum::ChangeCardPriorityToHigh => {
                self.keybindings.change_card_priority_to_high = value.to_vec();
            }
            KeyBindingEnum::ChangeCardPriorityToLow => {
                self.keybindings.change_card_priority_to_low = value.to_vec();
            }
            KeyBindingEnum::ChangeCardPriorityToMedium => {
                self.keybindings.change_card_priority_to_medium = value.to_vec();
            }
            KeyBindingEnum::ClearAllToasts => {
                self.keybindings.clear_all_toasts = value.to_vec();
            }
            KeyBindingEnum::DeleteBoard => {
                self.keybindings.delete_board = value.to_vec();
            }
            KeyBindingEnum::DeleteCard => {
                self.keybindings.delete_card = value.to_vec();
            }
            KeyBindingEnum::Down => {
                self.keybindings.down = value.to_vec();
            }
            KeyBindingEnum::GoToMainMenu => {
                self.keybindings.go_to_main_menu = value.to_vec();
            }
            KeyBindingEnum::GoToPreviousViewOrCancel => {
                self.keybindings.go_to_previous_view_or_cancel = value.to_vec();
            }
            KeyBindingEnum::HideUiElement => {
                self.keybindings.hide_ui_element = value.to_vec();
            }
            KeyBindingEnum::Left => {
                self.keybindings.left = value.to_vec();
            }
            KeyBindingEnum::MoveCardDown => {
                self.keybindings.move_card_down = value.to_vec();
            }
            KeyBindingEnum::MoveCardLeft => {
                self.keybindings.move_card_left = value.to_vec();
            }
            KeyBindingEnum::MoveCardRight => {
                self.keybindings.move_card_right = value.to_vec();
            }
            KeyBindingEnum::MoveCardUp => {
                self.keybindings.move_card_up = value.to_vec();
            }
            KeyBindingEnum::NewBoard => {
                self.keybindings.new_board = value.to_vec();
            }
            KeyBindingEnum::NewCard => {
                self.keybindings.new_card = value.to_vec();
            }
            KeyBindingEnum::NextFocus => {
                self.keybindings.next_focus = value.to_vec();
            }
            KeyBindingEnum::OpenConfigMenu => {
                self.keybindings.open_config_menu = value.to_vec();
            }
            KeyBindingEnum::PrvFocus => {
                self.keybindings.prv_focus = value.to_vec();
            }
            KeyBindingEnum::Quit => {
                self.keybindings.quit = value.to_vec();
            }
            KeyBindingEnum::Redo => {
                self.keybindings.redo = value.to_vec();
            }
            KeyBindingEnum::ResetUI => {
                self.keybindings.reset_ui = value.to_vec();
            }
            KeyBindingEnum::Right => {
                self.keybindings.right = value.to_vec();
            }
            KeyBindingEnum::SaveState => {
                self.keybindings.save_state = value.to_vec();
            }
            KeyBindingEnum::StopUserInput => {
                self.keybindings.stop_user_input = value.to_vec();
            }
            KeyBindingEnum::TakeUserInput => {
                self.keybindings.take_user_input = value.to_vec();
            }
            KeyBindingEnum::ToggleCommandPalette => {
                self.keybindings.toggle_command_palette = value.to_vec();
            }
            KeyBindingEnum::Undo => {
                self.keybindings.undo = value.to_vec();
            }
            KeyBindingEnum::Up => {
                self.keybindings.up = value.to_vec();
            }
        }
        Ok(key)
    }

    fn get_bool_or_default(
        serde_json_object: &serde_json::Value,
        config_enum: ConfigEnum,
        default: bool,
    ) -> bool {
        match serde_json_object[config_enum.to_json_key()].as_bool() {
            Some(value) => value,
            None => {
                error!(
                    "{} is not a boolean (true/false), Resetting to default value",
                    config_enum.to_json_key()
                );
                default
            }
        }
    }

    fn get_u16_or_default(
        serde_json_object: &serde_json::Value,
        config_enum: ConfigEnum,
        default: u16,
        min: Option<u16>,
        max: Option<u16>,
    ) -> u16 {
        match serde_json_object[config_enum.to_json_key()].as_u64() {
            Some(value) => {
                if let Some(min) = min {
                    if value < min as u64 {
                        error!(
                            "Invalid value: {} for {}, It must be greater than {}, Resetting to default value",
                            value, config_enum.to_json_key(), min
                        );
                        return default;
                    }
                }
                if let Some(max) = max {
                    if value > max as u64 {
                        error!(
                            "Invalid value: {} for {}, It must be less than {}, Resetting to default value",
                            value, config_enum.to_json_key(), max
                        );
                        return default;
                    }
                }
                value as u16
            }
            None => {
                error!(
                    "{} is not a number, Resetting to default value",
                    config_enum.to_json_key()
                );
                default
            }
        }
    }

    fn handle_invalid_keybinding(key: &str) {
        error!(
            "Invalid keybinding for key {}, Resetting to default keybinding",
            key
        );
    }

    fn json_config_keybindings_checker(serde_json_object: &Value) -> KeyBindings {
        if let Some(keybindings) = serde_json_object["keybindings"].as_object() {
            let mut default_keybindings = KeyBindings::default();
            for (key, value) in keybindings.iter() {
                let mut keybindings = vec![];
                if let Some(value_array) = value.as_array() {
                    for keybinding_value in value_array {
                        if let Some(keybinding_value_str) = keybinding_value.as_str() {
                            let keybinding_value = Key::from(keybinding_value_str);
                            if keybinding_value != Key::Unknown {
                                keybindings.push(keybinding_value);
                            } else {
                                Self::handle_invalid_keybinding(key);
                            }
                        } else if let Some(keybinding_value_obj) = keybinding_value.as_object() {
                            let keybinding_value = Key::from(keybinding_value_obj);
                            if keybinding_value != Key::Unknown {
                                keybindings.push(keybinding_value);
                            } else {
                                Self::handle_invalid_keybinding(key);
                            }
                        } else {
                            Self::handle_invalid_keybinding(key);
                        }
                    }
                    if keybindings.is_empty() {
                        Self::handle_invalid_keybinding(key);
                    } else {
                        default_keybindings.edit_keybinding(key, keybindings);
                    }
                } else {
                    Self::handle_invalid_keybinding(key);
                }
            }
            default_keybindings
        } else {
            KeyBindings::default()
        }
    }

    pub fn from_json_string(json_string: &str) -> Result<Self, String> {
        let root = serde_json::from_str(json_string);
        if root.is_err() {
            error!("Unable to recover old config. Resetting to default config");
            debug!("Error: {}", root.unwrap_err());
            return Err("Unable to recover old config. Resetting to default config".to_string());
        }
        let serde_json_object: Value = root.unwrap();
        let default_config = AppConfig::default();
        let save_directory =
            match serde_json_object[ConfigEnum::SaveDirectory.to_json_key()].as_str() {
                Some(path) => {
                    let path = PathBuf::from(path);
                    if path.exists() {
                        path
                    } else {
                        error!(
                            "Invalid path: {}, Resetting to default save directory",
                            path.to_str().unwrap()
                        );
                        default_config.save_directory
                    }
                }
                None => {
                    error!("Save Directory is not a string, Resetting to default save directory");
                    default_config.save_directory
                }
            };
        let default_view = match serde_json_object[ConfigEnum::DefaultView.to_json_key()].as_str() {
            Some(view) => {
                let view = View::from_str(view);
                if let Ok(view) = view {
                    view
                } else {
                    error!("Invalid View: {:?}, Resetting to default View", view);
                    default_config.default_view
                }
            }
            None => {
                error!("Default View is not a string, Resetting to default View");
                default_config.default_view
            }
        };
        let keybindings = AppConfig::json_config_keybindings_checker(&serde_json_object);
        let always_load_last_save = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::AlwaysLoadLastSave,
            default_config.always_load_last_save,
        );
        let save_on_exit = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::SaveOnExit,
            default_config.save_on_exit,
        );
        let disable_scroll_bar = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::DisableScrollBar,
            default_config.disable_scroll_bar,
        );
        let auto_login = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::AutoLogin,
            default_config.auto_login,
        );
        let show_line_numbers = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::ShowLineNumbers,
            default_config.show_line_numbers,
        );
        let disable_animations = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::DisableAnimations,
            default_config.disable_animations,
        );
        let enable_mouse_support = AppConfig::get_bool_or_default(
            &serde_json_object,
            ConfigEnum::EnableMouseSupport,
            default_config.enable_mouse_support,
        );
        let warning_delta = AppConfig::get_u16_or_default(
            &serde_json_object,
            ConfigEnum::WarningDelta,
            default_config.warning_delta,
            Some(1),
            None,
        );
        let tickrate = AppConfig::get_u16_or_default(
            &serde_json_object,
            ConfigEnum::Tickrate,
            default_config.tickrate,
            Some(MIN_TICKRATE),
            Some(MAX_TICKRATE),
        );
        let no_of_cards_to_show = AppConfig::get_u16_or_default(
            &serde_json_object,
            ConfigEnum::NoOfCardsToShow,
            default_config.no_of_cards_to_show,
            Some(MIN_NO_CARDS_PER_BOARD),
            Some(MAX_NO_CARDS_PER_BOARD),
        );
        let no_of_boards_to_show = AppConfig::get_u16_or_default(
            &serde_json_object,
            ConfigEnum::NoOfBoardsToShow,
            default_config.no_of_boards_to_show,
            Some(MIN_NO_BOARDS_PER_PAGE),
            Some(MAX_NO_BOARDS_PER_PAGE),
        );
        let default_theme = match serde_json_object[ConfigEnum::DefaultTheme.to_json_key()].as_str()
        {
            Some(default_theme) => default_theme.to_string(),
            None => {
                error!("Default Theme is not a string, Resetting to default theme");
                default_config.default_theme
            }
        };
        let date_format = match serde_json_object[ConfigEnum::DateFormat.to_json_key()].as_str() {
            Some(date_format) => match DateTimeFormat::from_str(date_format) {
                Ok(date_format) => date_format,
                Err(date_format_parse_error) => {
                    error!(
                        "Invalid date format: {}, Resetting to default date format",
                        date_format
                    );
                    debug!("Error: {}", date_format_parse_error);
                    default_config.date_time_format
                }
            },
            None => {
                error!("Date Format is not a string, Resetting to default date format");
                default_config.date_time_format
            }
        };
        let date_picker_calender_format =
            match serde_json_object[ConfigEnum::DatePickerCalenderFormat.to_json_key()].as_str() {
                Some(calender_format) => match CalenderType::from_str(calender_format) {
                    Ok(calender_format) => calender_format,
                    Err(calender_format_parse_error) => {
                        error!(
                            "Invalid calender format: {}, Resetting to default calender format",
                            calender_format
                        );
                        debug!("Error: {}", calender_format_parse_error);
                        CalenderType::default()
                    }
                },
                None => {
                    error!("Calender Format is not a string, Resetting to default calender format");
                    CalenderType::default()
                }
            };
        Ok(Self {
            save_directory,
            default_view,
            always_load_last_save,
            save_on_exit,
            disable_scroll_bar,
            auto_login,
            warning_delta,
            keybindings,
            tickrate,
            no_of_cards_to_show,
            no_of_boards_to_show,
            date_picker_calender_format,
            enable_mouse_support,
            default_theme,
            date_time_format: date_format,
            show_line_numbers,
            disable_animations,
        })
    }
}

#[derive(PartialEq, Copy, Clone, EnumIter)]
pub enum ConfigEnum {
    AlwaysLoadLastSave,
    AutoLogin,
    DateFormat,
    DefaultTheme,
    DefaultView,
    DisableAnimations,
    DisableScrollBar,
    EnableMouseSupport,
    Keybindings,
    NoOfBoardsToShow,
    NoOfCardsToShow,
    DatePickerCalenderFormat,
    SaveDirectory,
    SaveOnExit,
    ShowLineNumbers,
    Tickrate,
    WarningDelta,
}

impl fmt::Display for ConfigEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigEnum::AlwaysLoadLastSave => write!(f, "Auto Load Last Save"),
            ConfigEnum::AutoLogin => write!(f, "Auto Login"),
            ConfigEnum::DateFormat => write!(f, "Date Format"),
            ConfigEnum::DefaultTheme => write!(f, "Default Theme"),
            ConfigEnum::DefaultView => write!(f, "Select Default View"),
            ConfigEnum::DisableAnimations => write!(f, "Disable Animations"),
            ConfigEnum::DisableScrollBar => write!(f, "Disable Scroll Bar"),
            ConfigEnum::EnableMouseSupport => write!(f, "Enable Mouse Support"),
            ConfigEnum::Keybindings => write!(f, "Edit Keybindings"),
            ConfigEnum::NoOfBoardsToShow => write!(f, "Number of Boards to Show"),
            ConfigEnum::NoOfCardsToShow => write!(f, "Number of Cards to Show"),
            ConfigEnum::DatePickerCalenderFormat => write!(f, "Date Picker Calender Format"),
            ConfigEnum::SaveDirectory => write!(f, "Save Directory"),
            ConfigEnum::SaveOnExit => write!(f, "Auto Save on Exit"),
            ConfigEnum::ShowLineNumbers => write!(f, "Show Line Numbers"),
            ConfigEnum::Tickrate => write!(f, "Tickrate"),
            ConfigEnum::WarningDelta => write!(f, "Number of Days to Warn Before Due Date"),
        }
    }
}

impl FromStr for ConfigEnum {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Auto Load Last Save" => Ok(ConfigEnum::AlwaysLoadLastSave),
            "Auto Login" => Ok(ConfigEnum::AutoLogin),
            "Auto Save on Exit" => Ok(ConfigEnum::SaveOnExit),
            "Date Format" => Ok(ConfigEnum::DateFormat),
            "Default Theme" => Ok(ConfigEnum::DefaultTheme),
            "Disable Animations" => Ok(ConfigEnum::DisableAnimations),
            "Disable Scroll Bar" => Ok(ConfigEnum::DisableScrollBar),
            "Edit Keybindings" => Ok(ConfigEnum::Keybindings),
            "Enable Mouse Support" => Ok(ConfigEnum::EnableMouseSupport),
            "Number of Boards to Show" => Ok(ConfigEnum::NoOfBoardsToShow),
            "Number of Cards to Show" => Ok(ConfigEnum::NoOfCardsToShow),
            "Date Picker Calender Format" => Ok(ConfigEnum::DatePickerCalenderFormat),
            "Number of Days to Warn Before Due Date" => Ok(ConfigEnum::WarningDelta),
            "Save Directory" => Ok(ConfigEnum::SaveDirectory),
            "Select Default View" => Ok(ConfigEnum::DefaultView),
            "Show Line Numbers" => Ok(ConfigEnum::ShowLineNumbers),
            "Tickrate" => Ok(ConfigEnum::Tickrate),
            _ => Err(format!("Invalid ConfigEnum: {}", s)),
        }
    }
}

impl ConfigEnum {
    pub fn to_json_key(&self) -> &str {
        match self {
            ConfigEnum::AlwaysLoadLastSave => "always_load_last_save",
            ConfigEnum::AutoLogin => "auto_login",
            ConfigEnum::DateFormat => "date_format",
            ConfigEnum::DefaultTheme => "default_theme",
            ConfigEnum::DefaultView => "default_view",
            ConfigEnum::DisableAnimations => "disable_animations",
            ConfigEnum::DisableScrollBar => "disable_scroll_bar",
            ConfigEnum::EnableMouseSupport => "enable_mouse_support",
            ConfigEnum::Keybindings => "keybindings",
            ConfigEnum::NoOfBoardsToShow => "no_of_boards_to_show",
            ConfigEnum::NoOfCardsToShow => "no_of_cards_to_show",
            ConfigEnum::DatePickerCalenderFormat => "date_picker_calender_format",
            ConfigEnum::SaveDirectory => "save_directory",
            ConfigEnum::SaveOnExit => "save_on_exit",
            ConfigEnum::ShowLineNumbers => "show_line_numbers",
            ConfigEnum::Tickrate => "tickrate",
            ConfigEnum::WarningDelta => "warning_delta",
        }
    }

    pub fn validate_value(&self, value: &str) -> Result<(), String> {
        match self {
            ConfigEnum::SaveDirectory => {
                let path = PathBuf::from(value);
                if path.try_exists().is_ok() && path.try_exists().unwrap() && path.is_dir() {
                    Ok(())
                } else {
                    Err(format!("Invalid path: {}", value))
                }
            }
            ConfigEnum::DefaultView => {
                let view = View::from_string(value);
                if view.is_some() {
                    Ok(())
                } else {
                    Err(format!("Invalid View: {}", value))
                }
            }
            ConfigEnum::AlwaysLoadLastSave
            | ConfigEnum::AutoLogin
            | ConfigEnum::DisableAnimations
            | ConfigEnum::DisableScrollBar
            | ConfigEnum::EnableMouseSupport
            | ConfigEnum::SaveOnExit
            | ConfigEnum::ShowLineNumbers => {
                let check = value.parse::<bool>();
                if check.is_ok() {
                    Ok(())
                } else {
                    Err(format!("Invalid boolean: {}", value))
                }
            }
            ConfigEnum::NoOfBoardsToShow
            | ConfigEnum::NoOfCardsToShow
            | ConfigEnum::Tickrate
            | ConfigEnum::WarningDelta => {
                let min_value = match self {
                    ConfigEnum::WarningDelta => MIN_WARNING_DUE_DATE_DAYS,
                    ConfigEnum::Tickrate => MIN_TICKRATE,
                    ConfigEnum::NoOfCardsToShow => MIN_NO_CARDS_PER_BOARD,
                    ConfigEnum::NoOfBoardsToShow => MIN_NO_BOARDS_PER_PAGE,
                    _ => 0,
                };
                let max_value = match self {
                    ConfigEnum::WarningDelta => MAX_WARNING_DUE_DATE_DAYS,
                    ConfigEnum::Tickrate => MAX_TICKRATE,
                    ConfigEnum::NoOfCardsToShow => MAX_NO_CARDS_PER_BOARD,
                    ConfigEnum::NoOfBoardsToShow => MAX_NO_BOARDS_PER_PAGE,
                    _ => 0,
                };
                let check = value.parse::<u16>();
                if check.is_ok() {
                    let value = check.unwrap();
                    if value >= min_value && value <= max_value {
                        Ok(())
                    } else {
                        Err(format!(
                            "Invalid number: {}, It must be between {} and {}",
                            value, min_value, max_value
                        ))
                    }
                } else {
                    Err(format!("Invalid number: {}", value))
                }
            }
            ConfigEnum::DefaultTheme => {
                // TODO: check if theme exists
                Ok(())
            }
            ConfigEnum::DateFormat => {
                let date_format = DateTimeFormat::from_human_readable_string(value);
                if date_format.is_some() {
                    Ok(())
                } else {
                    Err(format!("Invalid DateFormat: {}", value))
                }
            }
            ConfigEnum::DatePickerCalenderFormat => {
                let calender_format = CalenderType::try_from(value);
                if calender_format.is_ok() {
                    Ok(())
                } else {
                    Err(format!("Invalid CalenderFormat: {}", value))
                }
            }
            ConfigEnum::Keybindings => {
                debug!("Keybindings should not be called from validate_value");
                // Keybindings are handled separately
                Ok(())
            }
        }
    }

    pub fn edit_config(&self, config: &mut AppConfig, value: &str) -> Result<(), String> {
        let value = value.trim();
        self.validate_value(value)?;
        // No need to be safe, since the value has been validated
        match self {
            ConfigEnum::SaveDirectory => {
                config.save_directory = PathBuf::from(value);
            }
            ConfigEnum::DefaultView => {
                config.default_view = View::from_string(value).unwrap();
            }
            ConfigEnum::AlwaysLoadLastSave => {
                config.always_load_last_save = value.parse::<bool>().unwrap();
            }
            ConfigEnum::SaveOnExit => {
                config.save_on_exit = value.parse::<bool>().unwrap();
            }
            ConfigEnum::DisableScrollBar => {
                config.disable_scroll_bar = value.parse::<bool>().unwrap();
            }
            ConfigEnum::AutoLogin => {
                config.auto_login = value.parse::<bool>().unwrap();
            }
            ConfigEnum::ShowLineNumbers => {
                config.show_line_numbers = value.parse::<bool>().unwrap();
            }
            ConfigEnum::DisableAnimations => {
                config.disable_animations = value.parse::<bool>().unwrap();
            }
            ConfigEnum::EnableMouseSupport => {
                config.enable_mouse_support = value.parse::<bool>().unwrap();
            }
            ConfigEnum::WarningDelta => {
                config.warning_delta = value.parse::<u16>().unwrap();
            }
            ConfigEnum::Tickrate => {
                config.tickrate = value.parse::<u16>().unwrap();
            }
            ConfigEnum::NoOfCardsToShow => {
                config.no_of_cards_to_show = value.parse::<u16>().unwrap();
            }
            ConfigEnum::NoOfBoardsToShow => {
                config.no_of_boards_to_show = value.parse::<u16>().unwrap();
            }
            ConfigEnum::DefaultTheme => {
                config.default_theme = value.to_string();
            }
            ConfigEnum::DateFormat => {
                config.date_time_format =
                    DateTimeFormat::from_human_readable_string(value).unwrap();
            }
            ConfigEnum::DatePickerCalenderFormat => {
                config.date_picker_calender_format = CalenderType::try_from(value).unwrap();
            }
            ConfigEnum::Keybindings => {
                debug!("Keybindings should not be called from edit_config");
                // Keybindings are handled separately
            }
        }
        Ok(())
    }
}

pub async fn handle_exit(app: &mut App<'_>) -> AppReturn {
    if app.config.save_on_exit {
        app.dispatch(IoEvent::AutoSave).await;
    }
    AppReturn::Exit
}
