use self::{
    actions::Actions,
    app_helper::{
        handle_general_actions, handle_keybinding_mode, handle_mouse_action,
        handle_user_input_mode, prepare_config_for_new_app,
    },
    kanban::{Board, Card, CardPriority},
    state::{AppStatus, Focus, KeyBindings, UiMode},
};
use crate::{
    app::{actions::Action, kanban::CardStatus},
    constants::{
        DEFAULT_CARD_WARNING_DUE_DATE_DAYS, DEFAULT_TICKRATE, DEFAULT_TOAST_DURATION,
        DEFAULT_UI_MODE, FIELD_NA, FIELD_NOT_SET, IO_EVENT_WAIT_TIME, MAX_NO_BOARDS_PER_PAGE,
        MAX_NO_CARDS_PER_BOARD, MIN_NO_BOARDS_PER_PAGE, MIN_NO_CARDS_PER_BOARD,
        MOUSE_OUT_OF_BOUNDS_COORDINATES, NO_OF_BOARDS_PER_PAGE, NO_OF_CARDS_PER_BOARD,
    },
    inputs::{key::Key, mouse::Mouse},
    io::{
        data_handler::{get_available_local_save_files, get_default_save_directory},
        io_handler::{refresh_visible_boards_and_cards, CloudData},
        logger::{get_logs, RUST_KANBAN_LOGGER},
        IoEvent,
    },
    ui::{
        text_box::TextBox,
        ui_helper,
        widgets::{CommandPaletteWidget, ToastType, ToastWidget},
        TextColorOptions, TextModifierOptions, Theme,
    },
};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use linked_hash_map::LinkedHashMap;
use log::{debug, error, info};
use ratatui::{
    backend::Backend,
    widgets::{ListState, TableState},
    Frame,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    time::{Duration, Instant},
    vec,
};

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
    DeleteCard(Card, (u64, u64)),
    CreateCard(Card, (u64, u64)),
    DeleteBoard(Board),
    MoveCardBetweenBoards(Card, (u64, u64), (u64, u64)),
    MoveCardWithinBoard((u64, u64), usize, usize),
    CreateBoard(Board),
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
}

pub struct App<'a> {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Actions,
    is_loading: bool,
    pub debug_mode: bool,
    pub state: AppState<'a>,
    pub boards: Vec<Board>,
    pub filtered_boards: Vec<Board>,
    pub config: AppConfig,
    pub visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>>,
    pub command_palette: CommandPaletteWidget,
    pub last_io_event_time: Option<Instant>,
    pub all_themes: Vec<Theme>,
    pub current_theme: Theme,
    pub action_history_manager: ActionHistoryManager,
    pub main_menu: MainMenu,
}

impl App<'_> {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>, debug_mode: bool) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();
        let boards = vec![];
        let filtered_boards = vec![];
        let all_themes = Theme::all_default_themes();
        let mut theme = Theme::default();
        let (config, config_errors, prepared_state) =
            prepare_config_for_new_app(state, theme.clone());
        let default_theme = config.default_theme.clone();
        let theme_in_all = all_themes.iter().find(|t| t.name == default_theme);
        if theme_in_all.is_some() {
            theme = theme_in_all.unwrap().clone();
        }

        let mut app = Self {
            io_tx,
            actions,
            is_loading,
            debug_mode,
            state: prepared_state.to_owned(),
            boards,
            filtered_boards,
            config,
            visible_boards_and_cards: LinkedHashMap::new(),
            command_palette: CommandPaletteWidget::new(debug_mode),
            last_io_event_time: None,
            all_themes,
            current_theme: theme,
            action_history_manager: ActionHistoryManager::default(),
            main_menu: MainMenu::default(),
        };
        if !config_errors.is_empty() {
            for error in config_errors {
                app.send_error_toast(error, None);
            }
        }
        app
    }

    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if self.state.app_status == AppStatus::UserInput {
            handle_user_input_mode(self, key).await
        } else if self.state.app_status == AppStatus::KeyBindMode {
            handle_keybinding_mode(self, key).await
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
            self.send_error_toast("Error in handling request please, restart the app", None);
        };
    }

    pub async fn handle_mouse(&mut self, mouse_action: Mouse) -> AppReturn {
        handle_mouse_action(self, mouse_action).await
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
        self.actions = Action::all().into();
        if self.state.ui_mode == UiMode::MainMenu {
            self.main_menu_next();
        } else if self.state.focus == Focus::NoFocus {
            self.state.focus = Focus::Body;
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
        &self.state.focus
    }
    pub fn change_focus(&mut self, focus: Focus) {
        self.state.focus = focus;
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
                if i >= self.config.to_view_list().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.config_state.select(Some(i));
    }
    pub fn config_prv(&mut self) {
        let i = match self.state.config_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.config.to_view_list().len() - 1
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
                if i >= self.main_menu.all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.main_menu_state.select(Some(i));
    }
    pub fn main_menu_prv(&mut self) {
        let i = match self.state.main_menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.main_menu.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.main_menu_state.select(Some(i));
    }
    pub fn load_save_next(&mut self, cloud_mode: bool) {
        let i = match self.state.load_save_state.selected() {
            Some(i) => {
                if cloud_mode {
                    let cloud_save_files = self.state.cloud_data.clone();
                    let cloud_save_files_len = if let Some(cloud_save_files_len) = cloud_save_files
                    {
                        cloud_save_files_len.len()
                    } else {
                        0
                    };
                    if cloud_save_files_len == 0 || i >= cloud_save_files_len - 1 {
                        0
                    } else {
                        i + 1
                    }
                } else {
                    let local_save_files = get_available_local_save_files(&self.config);
                    let local_save_files_len = if let Some(local_save_files_len) = local_save_files
                    {
                        local_save_files_len.len()
                    } else {
                        0
                    };
                    if local_save_files_len == 0 || i >= local_save_files_len - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
            }
            None => 0,
        };
        self.state.load_save_state.select(Some(i));
    }
    pub fn load_save_prv(&mut self, cloud_mode: bool) {
        let i = match self.state.load_save_state.selected() {
            Some(i) => {
                if cloud_mode {
                    let cloud_save_files = self.state.cloud_data.clone();
                    let cloud_save_files_len = if let Some(cloud_save_files_len) = cloud_save_files
                    {
                        cloud_save_files_len.len()
                    } else {
                        0
                    };
                    if i == 0 && cloud_save_files_len != 0 {
                        cloud_save_files_len - 1
                    } else if cloud_save_files_len == 0 {
                        0
                    } else {
                        i - 1
                    }
                } else {
                    let local_save_files = get_available_local_save_files(&self.config);
                    let local_save_files_len = if let Some(local_save_files_len) = local_save_files
                    {
                        local_save_files_len.len()
                    } else {
                        0
                    };
                    if i == 0 && local_save_files_len != 0 {
                        local_save_files_len - 1
                    } else if local_save_files_len == 0 {
                        0
                    } else {
                        i - 1
                    }
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
        self.state.prev_ui_mode = Some(self.state.ui_mode);
        self.state.ui_mode = ui_mode;
        let available_focus_targets = self.state.ui_mode.get_available_targets();
        if !available_focus_targets.contains(&self.state.focus) {
            if available_focus_targets.is_empty() {
                self.state.focus = Focus::NoFocus;
            } else {
                self.state.focus = available_focus_targets[0];
            }
        }
    }
    pub fn edit_keybindings_next(&mut self) {
        let keybinding_iterator = self.config.keybindings.iter();
        let i = match self.state.edit_keybindings_state.selected() {
            Some(i) => {
                if i >= keybinding_iterator.count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.edit_keybindings_state.select(Some(i));
    }
    pub fn edit_keybindings_prv(&mut self) {
        let keybinding_iterator = self.config.keybindings.iter();
        let i = match self.state.edit_keybindings_state.selected() {
            Some(i) => {
                if i == 0 {
                    keybinding_iterator.count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.edit_keybindings_state.select(Some(i));
    }
    pub fn help_next(&mut self) {
        let i = match self.state.help_state.selected() {
            Some(i) => {
                if !self.state.keybinding_store.is_empty() {
                    if i >= (self.state.keybinding_store.len() / 2) - 1 {
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
    pub fn help_prv(&mut self) {
        let i = match self.state.help_state.selected() {
            Some(i) => {
                if !self.state.keybinding_store.is_empty() {
                    if i == 0 {
                        (self.state.keybinding_store.len() / 2) - 1
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
                if i >= UiMode::view_modes_as_string().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.default_view_state.select(Some(i));
    }
    pub fn select_default_view_prv(&mut self) {
        let i = match self.state.default_view_state.selected() {
            Some(i) => {
                if i == 0 {
                    UiMode::view_modes_as_string().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.default_view_state.select(Some(i));
    }
    pub fn command_palette_command_search_prv(&mut self) {
        let i = match self
            .state
            .command_palette_command_search_list_state
            .selected()
        {
            Some(i) => {
                if self.command_palette.command_search_results.is_some() {
                    if i == 0 {
                        self.command_palette
                            .command_search_results
                            .clone()
                            .unwrap()
                            .len()
                            - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state
            .command_palette_command_search_list_state
            .select(Some(i));
    }
    pub fn command_palette_command_search_next(&mut self) {
        let i = match self
            .state
            .command_palette_command_search_list_state
            .selected()
        {
            Some(i) => {
                if self.command_palette.command_search_results.is_some() {
                    if i >= self
                        .command_palette
                        .command_search_results
                        .clone()
                        .unwrap()
                        .len()
                        - 1
                    {
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
        self.state
            .command_palette_command_search_list_state
            .select(Some(i));
    }
    pub fn command_palette_card_search_next(&mut self) {
        let i = match self.state.command_palette_card_search_list_state.selected() {
            Some(i) => {
                if self.command_palette.card_search_results.is_some() {
                    if i >= self
                        .command_palette
                        .card_search_results
                        .clone()
                        .unwrap()
                        .len()
                        - 1
                    {
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
        self.state
            .command_palette_card_search_list_state
            .select(Some(i));
    }
    pub fn command_palette_card_search_prv(&mut self) {
        let i = match self.state.command_palette_card_search_list_state.selected() {
            Some(i) => {
                if self.command_palette.card_search_results.is_some() {
                    if i == 0 {
                        self.command_palette
                            .card_search_results
                            .clone()
                            .unwrap()
                            .len()
                            - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state
            .command_palette_card_search_list_state
            .select(Some(i));
    }
    pub fn command_palette_board_search_next(&mut self) {
        let i = match self
            .state
            .command_palette_board_search_list_state
            .selected()
        {
            Some(i) => {
                if self.command_palette.board_search_results.is_some() {
                    if i >= self
                        .command_palette
                        .board_search_results
                        .clone()
                        .unwrap()
                        .len()
                        - 1
                    {
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
        self.state
            .command_palette_board_search_list_state
            .select(Some(i));
    }
    pub fn command_palette_board_search_prv(&mut self) {
        let i = match self
            .state
            .command_palette_board_search_list_state
            .selected()
        {
            Some(i) => {
                if self.command_palette.board_search_results.is_some() {
                    if i == 0 {
                        self.command_palette
                            .board_search_results
                            .clone()
                            .unwrap()
                            .len()
                            - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state
            .command_palette_board_search_list_state
            .select(Some(i));
    }
    pub fn keybinding_list_maker(&mut self) {
        let keybindings = &self.config.keybindings;
        let default_actions = &self.actions;
        let keybinding_action_iter = keybindings.iter();
        let mut keybinding_action_list: Vec<Vec<String>> = Vec::new();

        for (action, keys) in keybinding_action_iter {
            let mut keybinding_action = Vec::new();
            let mut keybinding_string = String::new();
            for key in keys {
                keybinding_string.push_str(&key.to_string());
                keybinding_string.push(' ');
            }
            keybinding_action.push(keybinding_string);
            let action_translated_string = KeyBindings::str_to_action(keybindings.clone(), action)
                .unwrap_or(&Action::Quit)
                .to_string();
            keybinding_action.push(action_translated_string);
            keybinding_action_list.push(keybinding_action);
        }

        let default_action_iter = default_actions.actions().iter();
        for action in default_action_iter {
            let str_action = action.to_string();
            if !keybinding_action_list.iter().any(|x| x[1] == str_action) {
                let mut keybinding_action = Vec::new();
                let action_keys = action
                    .keys()
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                keybinding_action.push(action_keys);
                keybinding_action.push(str_action);
                keybinding_action_list.push(keybinding_action);
            }
        }
        self.state.keybinding_store = keybinding_action_list;
    }
    pub fn send_info_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Info,
                self.current_theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Info,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn send_error_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Error,
                self.current_theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Error,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn send_warning_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Warning,
                self.current_theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Warning,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn send_loading_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Loading,
                self.current_theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Loading,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn select_card_status_prv(&mut self) {
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
    pub fn select_card_status_next(&mut self) {
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
    pub fn increase_loading_toast_time(&mut self, msg: &str, increase_by: Duration) {
        let toast = self.state.toasts.iter_mut().find(|x| x.message == msg);
        if toast.is_none() {
            debug!("No toast found with message: {}", msg);
            return;
        }
        let toast = toast.unwrap();
        toast.duration += increase_by;
    }
    pub fn select_change_theme_next(&mut self) {
        let i = match self.state.theme_selector_state.selected() {
            Some(i) => {
                if i >= self.all_themes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.theme_selector_state.select(Some(i));
        self.current_theme = self.all_themes[i].clone();
    }
    pub fn select_change_theme_prv(&mut self) {
        let i = match self.state.theme_selector_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.all_themes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.theme_selector_state.select(Some(i));
        self.current_theme = self.all_themes[i].clone();
    }
    pub fn select_create_theme_next(&mut self) {
        let theme_rows_len = Theme::default().to_rows(self).1.len();
        let i = match self.state.theme_editor_state.selected() {
            Some(i) => {
                if i >= theme_rows_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.theme_editor_state.select(Some(i));
    }
    pub fn select_create_theme_prv(&mut self) {
        let theme_rows_len = Theme::default().to_rows(self).1.len();
        let i = match self.state.theme_editor_state.selected() {
            Some(i) => {
                if i == 0 {
                    theme_rows_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.theme_editor_state.select(Some(i));
    }
    pub fn select_edit_style_fg_next(&mut self) {
        let i = match self.state.edit_specific_style_state.0.selected() {
            Some(i) => {
                if i >= TextColorOptions::to_iter().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.edit_specific_style_state.0.select(Some(i));
    }
    pub fn select_edit_style_fg_prv(&mut self) {
        let i = match self.state.edit_specific_style_state.0.selected() {
            Some(i) => {
                if i == 0 {
                    TextColorOptions::to_iter().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.edit_specific_style_state.0.select(Some(i));
    }
    pub fn select_edit_style_bg_next(&mut self) {
        let i = match self.state.edit_specific_style_state.1.selected() {
            Some(i) => {
                if i >= TextColorOptions::to_iter().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.edit_specific_style_state.1.select(Some(i));
    }
    pub fn select_edit_style_bg_prv(&mut self) {
        let i = match self.state.edit_specific_style_state.1.selected() {
            Some(i) => {
                if i == 0 {
                    TextColorOptions::to_iter().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.edit_specific_style_state.1.select(Some(i));
    }
    pub fn select_edit_style_modifier_next(&mut self) {
        let i = match self.state.edit_specific_style_state.2.selected() {
            Some(i) => {
                if i >= TextModifierOptions::to_iter().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.edit_specific_style_state.2.select(Some(i));
    }
    pub fn select_edit_style_modifier_prv(&mut self) {
        let i = match self.state.edit_specific_style_state.2.selected() {
            Some(i) => {
                if i == 0 {
                    TextModifierOptions::to_iter().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.edit_specific_style_state.2.select(Some(i));
    }
    pub fn select_card_priority_next(&mut self) {
        let i = match self.state.card_priority_selector_state.selected() {
            Some(i) => {
                if i >= CardPriority::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.card_priority_selector_state.select(Some(i));
    }
    pub fn select_card_priority_prv(&mut self) {
        let i = match self.state.card_priority_selector_state.selected() {
            Some(i) => {
                if i == 0 {
                    CardPriority::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.card_priority_selector_state.select(Some(i));
    }
    pub fn filter_by_tag_popup_next(&mut self) {
        let all_tags_len = if self.state.all_available_tags.is_some() {
            self.state.all_available_tags.clone().unwrap().len()
        } else {
            0
        };
        if all_tags_len > 0 {
            let i = match self.state.filter_by_tag_list_state.selected() {
                Some(i) => {
                    if i >= all_tags_len - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state.filter_by_tag_list_state.select(Some(i));
        }
    }
    pub fn filter_by_tag_popup_prv(&mut self) {
        let all_tags_len = if self.state.all_available_tags.is_some() {
            self.state.all_available_tags.clone().unwrap().len()
        } else {
            0
        };
        if all_tags_len > 0 {
            let i = match self.state.filter_by_tag_list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        all_tags_len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state.filter_by_tag_list_state.select(Some(i));
        }
    }
    pub fn change_date_format_popup_next(&mut self) {
        let i = match self.state.date_format_selector_state.selected() {
            Some(i) => {
                if i >= DateFormat::get_all_date_formats().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.date_format_selector_state.select(Some(i));
    }
    pub fn change_date_format_popup_prv(&mut self) {
        let i = match self.state.date_format_selector_state.selected() {
            Some(i) => {
                if i == 0 {
                    DateFormat::get_all_date_formats().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.date_format_selector_state.select(Some(i));
    }
    pub fn undo(&mut self) {
        if self.action_history_manager.history_index == 0 {
            self.send_error_toast("No more actions to undo", None);
        } else {
            let history_index = self.action_history_manager.history_index - 1;
            let history = self.action_history_manager.history[history_index].clone();
            match history {
                ActionHistory::DeleteCard(card, board_id) => {
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        board.cards.push(card.clone());
                        self.action_history_manager.history_index -= 1;
                        refresh_visible_boards_and_cards(self);
                        self.send_info_toast(&format!("Undo Delete Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not undo delete card '{}' as the board with id '{:?}' was not found", card.name, board_id), None);
                    }
                }
                ActionHistory::CreateCard(card, board_id) => {
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        board.cards.retain(|c| c.id != card.id);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index -= 1;
                        self.send_info_toast(&format!("Undo Create Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not undo create card '{}' as the board with id '{:?}' was not found", card.name, board_id), None);
                    }
                }
                ActionHistory::MoveCardBetweenBoards(
                    card,
                    moved_from_board_id,
                    moved_to_board_id,
                ) => {
                    if let Some(moved_to_board) =
                        self.boards.iter_mut().find(|b| b.id == moved_to_board_id)
                    {
                        moved_to_board.cards.retain(|c| c.id != card.id);
                    } else {
                        self.send_error_toast(&format!("Could not undo move card '{}' as the board with id '{:?}' was not found", card.name, moved_to_board_id), None);
                        return;
                    }
                    if let Some(moved_from_board) =
                        self.boards.iter_mut().find(|b| b.id == moved_from_board_id)
                    {
                        moved_from_board.cards.push(card.clone());
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index -= 1;
                        self.send_info_toast(&format!("Undo Move Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not undo move card '{}' as the board with id '{:?}' was not found", card.name, moved_from_board_id), None);
                    }
                }
                ActionHistory::MoveCardWithinBoard(board_id, moved_from_index, moved_to_index) => {
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        if moved_from_index >= board.cards.len()
                            || moved_to_index >= board.cards.len()
                        {
                            self.send_error_toast(
                                &format!(
                                    "Could not undo move card '{}' as the index's were invalid",
                                    FIELD_NA
                                ),
                                None,
                            );
                            return;
                        }
                        let card_name = board.cards[moved_to_index].name.clone();
                        board.cards.swap(moved_from_index, moved_to_index);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index -= 1;
                        self.send_info_toast(&format!("Undo Move Card '{}'", card_name), None);
                    } else {
                        self.send_error_toast(&format!("Could not undo move card '{}' as the board with id '{:?}' was not found",FIELD_NA, board_id), None);
                    }
                }
                ActionHistory::DeleteBoard(board) => {
                    self.boards.push(board.clone());
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    self.send_info_toast(&format!("Undo Delete Board '{}'", board.name), None);
                }
                ActionHistory::CreateBoard(board) => {
                    self.boards.retain(|b| b.id != board.id);
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    self.send_info_toast(&format!("Undo Create Board '{}'", board.name), None);
                }
                ActionHistory::EditCard(old_card, _, board_id) => {
                    let mut card_name = String::new();
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        if let Some(card) = board.cards.iter_mut().find(|c| c.id == old_card.id) {
                            *card = old_card.clone();
                            self.action_history_manager.history_index -= 1;
                            card_name = card.name.clone();
                        } else {
                            self.send_error_toast(
                                &format!(
                                    "Could not undo edit card '{}' as the card was not found",
                                    old_card.name
                                ),
                                None,
                            );
                        }
                        if !card_name.is_empty() {
                            self.send_info_toast(&format!("Undo Edit Card '{}'", card_name), None);
                            refresh_visible_boards_and_cards(self);
                        }
                    } else {
                        self.send_error_toast(&format!("Could not undo edit card '{}' as the board with id '{:?}' was not found", old_card.name, board_id), None);
                    }
                }
            }
        }
    }

    pub fn redo(&mut self) {
        if self.action_history_manager.history_index == self.action_history_manager.history.len() {
            self.send_error_toast("No more actions to redo", None);
        } else {
            let history_index = self.action_history_manager.history_index;
            let history = self.action_history_manager.history[history_index].clone();
            match history {
                ActionHistory::DeleteCard(card, board_id) => {
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        board.cards.retain(|c| c.id != card.id);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        self.send_info_toast(&format!("Redo Delete Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not redo delete card '{}' as the board with id '{:?}' was not found", card.name, board_id), None);
                    }
                }
                ActionHistory::CreateCard(card, board_id) => {
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        board.cards.push(card.clone());
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        self.send_info_toast(&format!("Redo Create Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not redo create card '{}' as the board with id '{:?}' was not found", card.name, board_id), None);
                    }
                }
                ActionHistory::MoveCardBetweenBoards(
                    card,
                    moved_from_board_id,
                    moved_to_board_id,
                ) => {
                    if let Some(moved_to_board) =
                        self.boards.iter_mut().find(|b| b.id == moved_to_board_id)
                    {
                        moved_to_board.cards.push(card.clone());
                    } else {
                        self.send_error_toast(&format!("Could not redo move card '{}' as the board with id '{:?}' was not found", card.name, moved_to_board_id), None);
                        return;
                    }
                    if let Some(moved_from_board) =
                        self.boards.iter_mut().find(|b| b.id == moved_from_board_id)
                    {
                        moved_from_board.cards.retain(|c| c.id != card.id);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        self.send_info_toast(&format!("Redo Move Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not redo move card '{}' as the board with id '{:?}' was not found", card.name, moved_from_board_id), None);
                    }
                }
                ActionHistory::MoveCardWithinBoard(board_id, moved_from_index, moved_to_index) => {
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        if moved_from_index >= board.cards.len()
                            || moved_to_index >= board.cards.len()
                        {
                            self.send_error_toast(
                                &format!(
                                    "Could not redo move card '{}' as the index's were invalid",
                                    FIELD_NA
                                ),
                                None,
                            );
                            return;
                        }
                        let card_name = board.cards[moved_to_index].name.clone();
                        board.cards.swap(moved_from_index, moved_to_index);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        self.send_info_toast(&format!("Redo Move Card '{}'", card_name), None);
                    } else {
                        self.send_error_toast(&format!("Could not redo move card '{}' as the board with id '{:?}' was not found", FIELD_NA, board_id), None);
                    }
                }
                ActionHistory::DeleteBoard(board) => {
                    self.boards.retain(|b| b.id != board.id);
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    self.send_info_toast(&format!("Redo Delete Board '{}'", board.name), None);
                }
                ActionHistory::CreateBoard(board) => {
                    self.boards.push(board.clone());
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    self.send_info_toast(&format!("Redo Create Board '{}'", board.name), None);
                }
                ActionHistory::EditCard(_, new_card, board_id) => {
                    let mut card_name = String::new();
                    if let Some(board) = self.boards.iter_mut().find(|b| b.id == board_id) {
                        if let Some(card) = board.cards.iter_mut().find(|c| c.id == new_card.id) {
                            *card = new_card.clone();
                            self.action_history_manager.history_index += 1;
                            card_name = card.name.clone();
                        } else {
                            self.send_error_toast(
                                &format!(
                                    "Could not redo edit card '{}' as the card was not found",
                                    new_card.name
                                ),
                                None,
                            );
                        }
                        if !card_name.is_empty() {
                            self.send_info_toast(&format!("Redo Edit Card '{}'", card_name), None);
                            refresh_visible_boards_and_cards(self);
                        }
                    } else {
                        self.send_error_toast(&format!("Could not redo edit card '{}' as the board with id '{:?}' was not found", new_card.name, board_id), None);
                    }
                }
            }
        }
    }
    pub fn log_next(&mut self) {
        let total_logs = get_logs().len();
        let mut hot_log = RUST_KANBAN_LOGGER.hot_log.lock();
        let i = match hot_log.state.selected() {
            Some(i) => {
                if i >= total_logs - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        hot_log.state.select(Some(i));
    }
    pub fn log_prv(&mut self) {
        let total_logs = get_logs().len();
        let mut hot_log = RUST_KANBAN_LOGGER.hot_log.lock();
        let i = match hot_log.state.selected() {
            Some(i) => {
                if i == 0 {
                    total_logs - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        hot_log.state.select(Some(i));
    }
}

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
            self.items = return_vec.clone();
            return_vec
        } else {
            let return_vec = vec![
                MainMenuItem::View,
                MainMenuItem::Config,
                MainMenuItem::Help,
                MainMenuItem::LoadSaveLocal,
                MainMenuItem::Quit,
            ];
            self.items = return_vec.clone();
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

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum PopupMode {
    ViewCard,
    CommandPalette,
    EditSpecificKeyBinding,
    ChangeUIMode,
    CardStatusSelector,
    EditGeneralConfig,
    SelectDefaultView,
    ChangeDateFormatPopup,
    ChangeTheme,
    EditThemeStyle,
    SaveThemePrompt,
    CustomRGBPromptFG,
    CustomRGBPromptBG,
    ConfirmDiscardCardChanges,
    CardPrioritySelector,
    FilterByTag,
}

impl Display for PopupMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            PopupMode::ViewCard => write!(f, "Card View"),
            PopupMode::CommandPalette => write!(f, "Command Palette"),
            PopupMode::EditSpecificKeyBinding => write!(f, "Edit Specific Key Binding"),
            PopupMode::ChangeUIMode => write!(f, "Change UI Mode"),
            PopupMode::CardStatusSelector => write!(f, "Change Card Status"),
            PopupMode::EditGeneralConfig => write!(f, "Edit General Config"),
            PopupMode::SelectDefaultView => write!(f, "Select Default View"),
            PopupMode::ChangeDateFormatPopup => write!(f, "Change Date Format"),
            PopupMode::ChangeTheme => write!(f, "Change Theme"),
            PopupMode::EditThemeStyle => write!(f, "Edit Theme Style"),
            PopupMode::SaveThemePrompt => write!(f, "Save Theme Prompt"),
            PopupMode::CustomRGBPromptFG => write!(f, "Custom RGB Prompt"),
            PopupMode::CustomRGBPromptBG => write!(f, "Custom RGB Prompt"),
            PopupMode::ConfirmDiscardCardChanges => write!(f, "Confirm Discard Card Changes"),
            PopupMode::CardPrioritySelector => write!(f, "Change Card Priority"),
            PopupMode::FilterByTag => write!(f, "Filter By Tag"),
        }
    }
}

impl PopupMode {
    fn get_available_targets(&self) -> Vec<Focus> {
        match self {
            PopupMode::ViewCard => vec![
                Focus::CardName,
                Focus::CardDescription,
                Focus::CardDueDate,
                Focus::CardPriority,
                Focus::CardStatus,
                Focus::CardTags,
                Focus::CardComments,
                Focus::SubmitButton,
            ],
            PopupMode::CommandPalette => vec![
                Focus::CommandPaletteCommand,
                Focus::CommandPaletteCard,
                Focus::CommandPaletteBoard,
            ],
            PopupMode::EditSpecificKeyBinding => vec![],
            PopupMode::ChangeUIMode => vec![],
            PopupMode::CardStatusSelector => vec![],
            PopupMode::EditGeneralConfig => vec![],
            PopupMode::SelectDefaultView => vec![],
            PopupMode::ChangeDateFormatPopup => vec![],
            PopupMode::ChangeTheme => vec![],
            PopupMode::EditThemeStyle => vec![
                Focus::StyleEditorFG,
                Focus::StyleEditorBG,
                Focus::StyleEditorModifier,
                Focus::SubmitButton,
            ],
            PopupMode::SaveThemePrompt => vec![Focus::SubmitButton, Focus::ExtraFocus],
            PopupMode::CustomRGBPromptFG => vec![Focus::TextInput, Focus::SubmitButton],
            PopupMode::CustomRGBPromptBG => vec![Focus::TextInput, Focus::SubmitButton],
            PopupMode::ConfirmDiscardCardChanges => vec![Focus::SubmitButton, Focus::ExtraFocus],
            PopupMode::CardPrioritySelector => vec![],
            PopupMode::FilterByTag => vec![Focus::FilterByTagPopup, Focus::SubmitButton],
        }
    }

    pub fn render<B>(self, rect: &mut Frame<B>, app: &mut App)
    where
        B: Backend,
    {
        match self {
            PopupMode::ViewCard => {
                ui_helper::render_view_card(rect, app);
            }
            PopupMode::CardStatusSelector => {
                ui_helper::render_change_card_status_popup(rect, app);
            }
            PopupMode::ChangeUIMode => {
                ui_helper::render_change_ui_mode_popup(rect, app);
            }
            PopupMode::CommandPalette => {
                ui_helper::render_command_palette(rect, app);
            }
            PopupMode::EditGeneralConfig => {
                ui_helper::render_edit_config(rect, app);
            }
            PopupMode::EditSpecificKeyBinding => {
                ui_helper::render_edit_specific_keybinding(rect, app);
            }
            PopupMode::SelectDefaultView => {
                ui_helper::render_select_default_view(rect, app);
            }
            PopupMode::ChangeTheme => {
                ui_helper::render_change_theme_popup(rect, app);
            }
            PopupMode::EditThemeStyle => {
                ui_helper::render_edit_specific_style_popup(rect, app);
            }
            PopupMode::SaveThemePrompt => {
                ui_helper::render_save_theme_prompt(rect, app);
            }
            PopupMode::CustomRGBPromptFG | PopupMode::CustomRGBPromptBG => {
                ui_helper::render_custom_rgb_color_prompt(rect, app);
            }
            PopupMode::ConfirmDiscardCardChanges => {
                ui_helper::render_confirm_discard_card_changes(rect, app);
            }
            PopupMode::CardPrioritySelector => {
                ui_helper::render_card_priority_selector(rect, app);
            }
            PopupMode::FilterByTag => {
                ui_helper::render_filter_by_tag_popup(rect, app);
            }
            PopupMode::ChangeDateFormatPopup => {
                ui_helper::render_change_date_format_popup(rect, app);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState<'a> {
    pub app_status: AppStatus,
    pub current_board_id: Option<(u64, u64)>,
    pub current_card_id: Option<(u64, u64)>,
    pub focus: Focus,
    pub previous_focus: Option<Focus>,
    pub current_user_input: String,
    pub main_menu_state: ListState,
    pub config_state: TableState,
    pub new_board_form: Vec<String>,
    pub new_card_form: Vec<String>,
    pub login_form: (Vec<String>, bool),
    pub signup_form: (Vec<String>, bool),
    pub reset_password_form: (Vec<String>, bool),
    pub load_save_state: ListState,
    pub edit_keybindings_state: TableState,
    pub edited_keybinding: Option<Vec<Key>>,
    pub help_state: TableState,
    pub keybinding_store: Vec<Vec<String>>,
    pub default_view_state: ListState,
    pub current_cursor_position: Option<usize>,
    pub toasts: Vec<ToastWidget>,
    pub term_background_color: (u8, u8, u8),
    pub preview_boards_and_cards: Option<Vec<Board>>,
    pub preview_visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>>,
    pub preview_file_name: Option<String>,
    pub popup_mode: Option<PopupMode>,
    pub ui_mode: UiMode,
    pub no_of_cards_to_show: u16,
    pub command_palette_command_search_list_state: ListState,
    pub command_palette_card_search_list_state: ListState,
    pub command_palette_board_search_list_state: ListState,
    pub card_status_selector_state: ListState,
    pub prev_ui_mode: Option<UiMode>,
    pub debug_menu_toggled: bool,
    pub ui_render_time: Option<u128>,
    pub current_mouse_coordinates: (u16, u16),
    pub mouse_focus: Option<Focus>,
    pub mouse_list_index: Option<u16>,
    pub last_mouse_action: Option<Mouse>,
    pub last_mouse_action_time: Option<Instant>,
    pub theme_selector_state: ListState,
    pub theme_being_edited: Theme,
    pub theme_editor_state: TableState,
    pub edit_specific_style_state: (ListState, ListState, ListState),
    pub default_theme_mode: bool,
    pub card_view_list_state: ListState,
    pub card_view_tag_list_state: ListState,
    pub card_view_comment_list_state: ListState,
    pub card_priority_selector_state: ListState,
    pub all_available_tags: Option<Vec<(String, u32)>>,
    pub filter_tags: Option<Vec<String>>,
    pub filter_by_tag_list_state: ListState,
    pub date_format_selector_state: ListState,
    pub log_state: ListState,
    pub user_login_data: UserLoginData,
    pub cloud_data: Option<Vec<CloudData>>,
    pub last_reset_password_link_sent_time: Option<Instant>,
    pub encryption_key_from_arguments: Option<String>,
    pub card_being_edited: Option<((u64, u64), Card)>, // (board_id, card)
    pub card_description_text_buffer: Option<TextBox<'a>>,
    pub config_item_being_edited: Option<usize>,
}

impl Default for AppState<'_> {
    fn default() -> AppState<'static> {
        AppState {
            app_status: AppStatus::default(),
            focus: Focus::NoFocus,
            current_board_id: None,
            current_card_id: None,
            previous_focus: None,
            current_user_input: String::new(),
            main_menu_state: ListState::default(),
            config_state: TableState::default(),
            new_board_form: vec![String::new(), String::new()],
            new_card_form: vec![String::new(), String::new(), String::new()],
            login_form: (vec![String::new(), String::new()], false),
            signup_form: (vec![String::new(), String::new(), String::new()], false),
            reset_password_form: (
                vec![String::new(), String::new(), String::new(), String::new()],
                false,
            ),
            load_save_state: ListState::default(),
            edit_keybindings_state: TableState::default(),
            edited_keybinding: None,
            help_state: TableState::default(),
            keybinding_store: Vec::new(),
            default_view_state: ListState::default(),
            current_cursor_position: None,
            toasts: Vec::new(),
            term_background_color: get_term_bg_color(),
            preview_boards_and_cards: None,
            preview_visible_boards_and_cards: LinkedHashMap::new(),
            preview_file_name: None,
            popup_mode: None,
            ui_mode: DEFAULT_UI_MODE,
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            command_palette_command_search_list_state: ListState::default(),
            command_palette_card_search_list_state: ListState::default(),
            command_palette_board_search_list_state: ListState::default(),
            card_status_selector_state: ListState::default(),
            prev_ui_mode: None,
            debug_menu_toggled: false,
            ui_render_time: None,
            current_mouse_coordinates: MOUSE_OUT_OF_BOUNDS_COORDINATES, // make sure it's out of bounds when mouse mode is disabled
            mouse_focus: None,
            mouse_list_index: None,
            last_mouse_action: None,
            last_mouse_action_time: None,
            theme_selector_state: ListState::default(),
            theme_being_edited: Theme::default(),
            theme_editor_state: TableState::default(),
            edit_specific_style_state: (
                ListState::default(),
                ListState::default(),
                ListState::default(),
            ),
            default_theme_mode: false,
            card_view_list_state: ListState::default(),
            card_view_tag_list_state: ListState::default(),
            card_view_comment_list_state: ListState::default(),
            card_priority_selector_state: ListState::default(),
            all_available_tags: None,
            filter_tags: None,
            filter_by_tag_list_state: ListState::default(),
            date_format_selector_state: ListState::default(),
            log_state: ListState::default(),
            user_login_data: UserLoginData {
                email_id: None,
                auth_token: None,
                user_id: None,
            },
            cloud_data: None,
            last_reset_password_link_sent_time: None,
            encryption_key_from_arguments: None,
            card_being_edited: None,
            card_description_text_buffer: None,
            config_item_being_edited: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserLoginData {
    pub email_id: Option<String>,
    pub auth_token: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum DateFormat {
    DayMonthYear,
    MonthDayYear,
    YearMonthDay,
    #[default]
    DayMonthYearTime,
    MonthDayYearTime,
    YearMonthDayTime,
}

impl DateFormat {
    pub fn to_human_readable_string(&self) -> &str {
        match self {
            DateFormat::DayMonthYear => "DD/MM/YYYY",
            DateFormat::MonthDayYear => "MM/DD/YYYY",
            DateFormat::YearMonthDay => "YYYY/MM/DD",
            DateFormat::DayMonthYearTime => "DD/MM/YYYY-HH:MM:SS",
            DateFormat::MonthDayYearTime => "MM/DD/YYYY-HH:MM:SS",
            DateFormat::YearMonthDayTime => "YYYY/MM/DD-HH:MM:SS",
        }
    }
    pub fn to_parser_string(&self) -> &str {
        match self {
            DateFormat::DayMonthYear => "%d/%m/%Y",
            DateFormat::MonthDayYear => "%m/%d/%Y",
            DateFormat::YearMonthDay => "%Y/%m/%d",
            DateFormat::DayMonthYearTime => "%d/%m/%Y-%H:%M:%S",
            DateFormat::MonthDayYearTime => "%m/%d/%Y-%H:%M:%S",
            DateFormat::YearMonthDayTime => "%Y/%m/%d-%H:%M:%S",
        }
    }
    pub fn get_all_date_formats() -> Vec<DateFormat> {
        vec![
            DateFormat::DayMonthYear,
            DateFormat::MonthDayYear,
            DateFormat::YearMonthDay,
            DateFormat::DayMonthYearTime,
            DateFormat::MonthDayYearTime,
            DateFormat::YearMonthDayTime,
        ]
    }
    pub fn all_formats_with_time() -> Vec<DateFormat> {
        vec![
            DateFormat::DayMonthYearTime,
            DateFormat::MonthDayYearTime,
            DateFormat::YearMonthDayTime,
        ]
    }
    pub fn all_formats_without_time() -> Vec<DateFormat> {
        vec![
            DateFormat::DayMonthYear,
            DateFormat::MonthDayYear,
            DateFormat::YearMonthDay,
        ]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub save_directory: PathBuf,
    pub default_view: UiMode,
    pub always_load_last_save: bool,
    pub save_on_exit: bool,
    pub disable_scroll_bar: bool,
    pub warning_delta: u16,
    pub keybindings: KeyBindings,
    pub tickrate: u64,
    pub no_of_cards_to_show: u16,
    pub no_of_boards_to_show: u16,
    pub enable_mouse_support: bool,
    pub default_theme: String,
    pub date_format: DateFormat,
    pub auto_login: bool,
    pub show_line_numbers: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_view = DEFAULT_UI_MODE;
        let default_theme = Theme::default();
        Self {
            save_directory: get_default_save_directory(),
            default_view,
            always_load_last_save: true,
            save_on_exit: true,
            disable_scroll_bar: false,
            warning_delta: DEFAULT_CARD_WARNING_DUE_DATE_DAYS,
            keybindings: KeyBindings::default(),
            tickrate: DEFAULT_TICKRATE,
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            no_of_boards_to_show: NO_OF_BOARDS_PER_PAGE,
            enable_mouse_support: true,
            default_theme: default_theme.name,
            date_format: DateFormat::default(),
            auto_login: true,
            show_line_numbers: true,
        }
    }
}

impl AppConfig {
    pub fn to_view_list(&self) -> Vec<Vec<String>> {
        vec![
            vec![
                String::from("Save Directory"),
                self.save_directory.to_str().unwrap().to_string(),
            ],
            vec![
                String::from("Select Default View"),
                self.default_view.to_string(),
            ],
            vec![
                String::from("Auto Load Last Save"),
                self.always_load_last_save.to_string(),
            ],
            vec![
                String::from("Auto Save on Exit"),
                self.save_on_exit.to_string(),
            ],
            vec![
                String::from("Disable Scroll Bar"),
                self.disable_scroll_bar.to_string(),
            ],
            vec![String::from("Auto Login"), self.auto_login.to_string()],
            vec![
                String::from("Show Line Numbers"),
                self.show_line_numbers.to_string(),
            ],
            vec![
                String::from("Number of Days to Warn Before Due Date"),
                self.warning_delta.to_string(),
            ],
            vec![String::from("Tickrate"), self.tickrate.to_string()],
            vec![
                String::from("Number of Cards to Show per board"),
                self.no_of_cards_to_show.to_string(),
            ],
            vec![
                String::from("Number of Boards to Show"),
                self.no_of_boards_to_show.to_string(),
            ],
            vec![
                String::from("Enable Mouse Support"),
                self.enable_mouse_support.to_string(),
            ],
            vec![
                String::from("Default Theme"),
                self.default_theme.to_string(),
            ],
            vec![
                String::from("Default Date Format"),
                self.date_format.to_human_readable_string().to_string(),
            ],
            vec![String::from("Edit Keybindings")],
        ]
    }

    pub fn edit_with_string(change_str: &str, app: &mut App) -> Self {
        let mut config = app.config.clone();
        let lines = change_str.lines();
        for line in lines {
            let mut parts = line.split(':');
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            match key {
                "Save Directory" => {
                    let new_path = PathBuf::from(value);
                    if new_path.exists() {
                        config.save_directory = new_path;
                    } else {
                        error!("Invalid path: {}", value);
                        app.send_error_toast(&format!("Invalid path: {}", value), None);
                        app.send_info_toast("Check if the path exists", None);
                    }
                }
                "Select Default View" => {
                    let new_ui_mode = UiMode::from_string(value);
                    if let Some(new_ui_mode) = new_ui_mode {
                        config.default_view = new_ui_mode;
                    } else {
                        error!("Invalid UiMode: {}", value);
                        app.send_error_toast(&format!("Invalid UiMode: {}", value), None);
                        info!("Valid UiModes are: {:?}", UiMode::view_modes_as_string());
                    }
                }
                "Auto Load Last Save" => {
                    if value.to_lowercase() == "true" {
                        config.always_load_last_save = true;
                    } else if value.to_lowercase() == "false" {
                        config.always_load_last_save = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value), None);
                    }
                }
                "Auto Save on Exit" => {
                    if value.to_lowercase() == "true" {
                        config.save_on_exit = true;
                    } else if value.to_lowercase() == "false" {
                        config.save_on_exit = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value), None);
                    }
                }
                "Disable Scroll Bar" => {
                    if value.to_lowercase() == "true" {
                        config.disable_scroll_bar = true;
                    } else if value.to_lowercase() == "false" {
                        config.disable_scroll_bar = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value), None);
                    }
                }
                "Auto Login" => {
                    if value.to_lowercase() == "true" {
                        config.auto_login = true;
                    } else if value.to_lowercase() == "false" {
                        config.auto_login = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value), None);
                    }
                }
                "Show Line Numbers" => {
                    if value.to_lowercase() == "true" {
                        config.show_line_numbers = true;
                    } else if value.to_lowercase() == "false" {
                        config.show_line_numbers = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value), None);
                    }
                }
                "Enable Mouse Support" => {
                    if value.to_lowercase() == "true" {
                        config.enable_mouse_support = true;
                    } else if value.to_lowercase() == "false" {
                        config.enable_mouse_support = false;
                    } else {
                        error!("Invalid boolean: {}", value);
                        app.send_error_toast(&format!("Expected boolean, got: {}", value), None);
                    }
                }
                "Number of Days to Warn Before Due Date" => {
                    let new_delta = value.parse::<u16>();
                    if let Ok(new_delta) = new_delta {
                        config.warning_delta = new_delta;
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(
                            &format!("Expected number of days (integer), got: {}", value),
                            None,
                        );
                    }
                }
                "Tickrate" => {
                    let new_tickrate = value.parse::<u64>();
                    if let Ok(new_tickrate) = new_tickrate {
                        if new_tickrate < 10 {
                            error!(
                                "Tickrate must be greater than 10ms, to avoid overloading the CPU"
                            );
                            app.send_error_toast(
                                "Tickrate must be greater than 10ms, to avoid overloading the CPU",
                                None,
                            );
                        } else if new_tickrate > 1000 {
                            error!("Tickrate must be less than 1000ms");
                            app.send_error_toast("Tickrate must be less than 1000ms", None);
                        } else {
                            config.tickrate = new_tickrate;
                            info!("Tickrate set to {}ms", new_tickrate);
                            info!("Restart the program to apply changes");
                            info!("If experiencing slow input, or stuttering, try adjusting the tickrate");
                            app.send_info_toast(
                                &format!("Tickrate set to {}ms", new_tickrate),
                                None,
                            );
                        }
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(
                            &format!("Expected number of milliseconds (integer), got: {}", value),
                            None,
                        );
                    }
                }
                "Number of Cards to Show per board" => {
                    let new_no_cards = value.parse::<u16>();
                    if let Ok(new_no_cards) = new_no_cards {
                        if new_no_cards < MIN_NO_CARDS_PER_BOARD {
                            error!(
                                "Number of cards must be greater than {}",
                                MIN_NO_CARDS_PER_BOARD
                            );
                            app.send_error_toast(
                                &format!(
                                    "Number of cards must be greater than {}",
                                    MIN_NO_CARDS_PER_BOARD
                                ),
                                None,
                            );
                        } else if new_no_cards > MAX_NO_CARDS_PER_BOARD {
                            error!(
                                "Number of cards must be less than {}",
                                MAX_NO_CARDS_PER_BOARD
                            );
                            app.send_error_toast(
                                &format!(
                                    "Number of cards must be less than {}",
                                    MAX_NO_CARDS_PER_BOARD
                                ),
                                None,
                            );
                        } else {
                            config.no_of_cards_to_show = new_no_cards;
                            app.send_info_toast(
                                &format!(
                                    "Number of cards per board to display set to {}",
                                    new_no_cards
                                ),
                                None,
                            );
                        }
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(
                            &format!("Expected number of cards (integer), got: {}", value),
                            None,
                        );
                    }
                }
                "Number of Boards to Show" => {
                    let new_no_boards = value.parse::<u16>();
                    if let Ok(new_no_boards) = new_no_boards {
                        if new_no_boards < MIN_NO_BOARDS_PER_PAGE {
                            error!(
                                "Number of boards must be greater than {}",
                                MIN_NO_BOARDS_PER_PAGE
                            );
                            app.send_error_toast(
                                &format!(
                                    "Number of boards must be greater than {}",
                                    MIN_NO_BOARDS_PER_PAGE
                                ),
                                None,
                            );
                        } else if new_no_boards > MAX_NO_BOARDS_PER_PAGE {
                            error!(
                                "Number of boards must be less than {}",
                                MAX_NO_BOARDS_PER_PAGE
                            );
                            app.send_error_toast(
                                &format!(
                                    "Number of boards must be less than {}",
                                    MAX_NO_BOARDS_PER_PAGE
                                ),
                                None,
                            );
                        } else {
                            config.no_of_boards_to_show = new_no_boards;
                            app.send_info_toast(
                                &format!("Number of boards to display set to {}", new_no_boards),
                                None,
                            );
                        }
                    } else {
                        error!("Invalid number: {}", value);
                        app.send_error_toast(
                            &format!("Expected number of boards (integer), got: {}", value),
                            None,
                        );
                    }
                }
                "default_theme" => {
                    // TODO: check if theme exists
                }
                "Default Date Format" => {
                    // TODO
                }
                _ => {
                    debug!("Invalid key: {}", key);
                    app.send_error_toast("Something went wrong 😢 ", None);
                    return config;
                }
            }
        }
        refresh_visible_boards_and_cards(app);
        config
    }

    pub fn edit_keybinding(&mut self, key_index: usize, value: Vec<Key>) -> Result<(), String> {
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
            "quit" => self.keybindings.quit = value,
            "next_focus" => self.keybindings.next_focus = value,
            "prev_focus" => self.keybindings.prev_focus = value,
            "open_config_menu" => self.keybindings.open_config_menu = value,
            "up" => self.keybindings.up = value,
            "down" => self.keybindings.down = value,
            "right" => self.keybindings.right = value,
            "left" => self.keybindings.left = value,
            "take_user_input" => self.keybindings.take_user_input = value,
            "stop_user_input" => self.keybindings.stop_user_input = value,
            "hide_ui_element" => self.keybindings.hide_ui_element = value,
            "save_state" => self.keybindings.save_state = value,
            "new_board" => self.keybindings.new_board = value,
            "new_card" => self.keybindings.new_card = value,
            "delete_card" => self.keybindings.delete_card = value,
            "delete_board" => self.keybindings.delete_board = value,
            "change_card_status_to_completed" => {
                self.keybindings.change_card_status_to_completed = value
            }
            "change_card_status_to_active" => self.keybindings.change_card_status_to_active = value,
            "change_card_status_to_stale" => self.keybindings.change_card_status_to_stale = value,
            "reset_ui" => self.keybindings.reset_ui = value,
            "go_to_main_menu" => self.keybindings.go_to_main_menu = value,
            "toggle_command_palette" => self.keybindings.toggle_command_palette = value,
            "clear_all_toasts" => self.keybindings.clear_all_toasts = value,
            "undo" => self.keybindings.undo = value,
            "redo" => self.keybindings.redo = value,
            _ => {
                debug!("Invalid key: {}", key);
                error!("Unable to edit keybinding");
                return Err("Something went wrong 😢 ".to_string());
            }
        }
        Ok(())
    }

    pub fn from_json_string(json_string: &str) -> Result<Self, String> {
        let root = serde_json::from_str(json_string);
        if root.is_err() {
            error!("Unable to recover old config. Resetting to default config");
            debug!("Error: {}", root.unwrap_err());
            return Err("Unable to recover old config. Resetting to default config".to_string());
        }
        let serde_json_object: Value = root.unwrap();
        let save_directory = match serde_json_object["save_directory"].as_str() {
            Some(path) => {
                let path = PathBuf::from(path);
                if path.exists() {
                    path
                } else {
                    error!(
                        "Invalid path: {}, Resetting to default save directory",
                        path.to_str().unwrap()
                    );
                    get_default_save_directory()
                }
            }
            None => {
                error!("Save Directory is not a string, Resetting to default save directory");
                get_default_save_directory()
            }
        };
        let default_view = match serde_json_object["default_view"].as_str() {
            Some(ui_mode) => {
                let ui_mode = UiMode::from_json_string(ui_mode);
                if let Some(ui_mode) = ui_mode {
                    ui_mode
                } else {
                    error!("Invalid UiMode: {:?}, Resetting to default UiMode", ui_mode);
                    DEFAULT_UI_MODE
                }
            }
            None => {
                error!("Default View is not a string, Resetting to default UiMode");
                DEFAULT_UI_MODE
            }
        };
        let always_load_last_save = match serde_json_object["always_load_last_save"].as_bool() {
            Some(always_load_last_save) => always_load_last_save,
            None => {
                error!("Always Load Last Save is not a boolean, Resetting to default value");
                true
            }
        };
        let save_on_exit = match serde_json_object["save_on_exit"].as_bool() {
            Some(save_on_exit) => save_on_exit,
            None => {
                error!("Save on Exit is not a boolean, Resetting to default value");
                true
            }
        };
        let disable_scroll_bar = match serde_json_object["disable_scroll_bar"].as_bool() {
            Some(disable_scroll_bar) => disable_scroll_bar,
            None => {
                error!("Disable Scroll Bar is not a boolean, Resetting to default value");
                false
            }
        };
        let auto_login = match serde_json_object["auto_login"].as_bool() {
            Some(auto_login) => auto_login,
            None => {
                error!("Auto Login is not a boolean, Resetting to default value");
                true
            }
        };
        let warning_delta = match serde_json_object["warning_delta"].as_u64() {
            Some(warning_delta) => warning_delta as u16,
            None => {
                error!("Warning Delta is not a number, Resetting to default value");
                DEFAULT_CARD_WARNING_DUE_DATE_DAYS
            }
        };
        let keybindings = match serde_json_object["keybindings"].as_object() {
            Some(keybindings) => {
                let mut keybindings = keybindings.clone();
                let mut keybindings_object = KeyBindings::default();
                for (key, value) in keybindings.iter_mut() {
                    let mut keybinding = vec![];
                    let value_array = value.as_array_mut();
                    if value_array.is_none() {
                        error!(
                            "Invalid keybinding: {} for key {}, Resetting to default keybinding",
                            value, key
                        );
                        keybindings_object.edit_keybinding(key, vec![Key::Unknown]);
                        continue;
                    }
                    for keybinding_value in value_array.unwrap().iter_mut() {
                        let keybinding_value_str = keybinding_value.as_str();
                        let keybinding_value_obj = keybinding_value.as_object();
                        if let Some(keybinding_value_str) = keybinding_value_str {
                            let keybinding_value = Key::from(keybinding_value_str);
                            if keybinding_value != Key::Unknown {
                                keybinding.push(keybinding_value);
                            } else {
                                error!(
                                    "Invalid keybinding: {} for key {}, Resetting to default keybinding",
                                    keybinding_value_str, key
                                );
                                keybinding = match keybindings_object.get_keybinding(key) {
                                    Some(keybinding) => keybinding.to_vec(),
                                    None => vec![Key::Unknown],
                                }
                            }
                        } else if let Some(keybinding_value_obj) = keybinding_value_obj {
                            let keybinding_value = Key::from(keybinding_value_obj);
                            if keybinding_value != Key::Unknown {
                                keybinding.push(keybinding_value);
                            } else {
                                error!(
                                    "Invalid keybinding: {:?} for key {}, Resetting to default keybinding",
                                    keybinding_value_obj, key
                                );
                                keybinding = match keybindings_object.get_keybinding(key) {
                                    Some(keybinding) => keybinding.to_vec(),
                                    None => vec![Key::Unknown],
                                }
                            }
                        } else {
                            error!(
                                "Invalid keybinding for key {}, Resetting to default keybinding",
                                key
                            );
                            keybinding = match keybindings_object.get_keybinding(key) {
                                Some(keybinding) => keybinding.to_vec(),
                                None => vec![Key::Unknown],
                            }
                        }
                    }
                    keybindings_object.edit_keybinding(key, keybinding);
                }
                keybindings_object
            }
            None => KeyBindings::default(),
        };
        let tickrate = match serde_json_object["tickrate"].as_u64() {
            Some(tickrate) => {
                if !(10..=1000).contains(&tickrate) {
                    error!("Invalid tickrate: {}, It must be between 10 and 1000, Resetting to default tickrate", tickrate);
                    DEFAULT_TICKRATE
                } else {
                    tickrate
                }
            }
            None => {
                error!("Tickrate is not a number, Resetting to default tickrate");
                DEFAULT_TICKRATE
            }
        };
        let no_of_cards_to_show = match serde_json_object["no_of_cards_to_show"].as_u64() {
            Some(no_of_cards_to_show) => {
                if no_of_cards_to_show < MIN_NO_CARDS_PER_BOARD.into()
                    || no_of_cards_to_show > MAX_NO_CARDS_PER_BOARD.into()
                {
                    error!("Invalid number of cards to show: {}, Resetting to default number of cards to show", no_of_cards_to_show);
                    NO_OF_CARDS_PER_BOARD
                } else {
                    no_of_cards_to_show as u16
                }
            }
            None => {
                error!("Number of cards to show is not a number, Resetting to default number of cards to show");
                NO_OF_CARDS_PER_BOARD
            }
        };
        let no_of_boards_to_show = match serde_json_object["no_of_boards_to_show"].as_u64() {
            Some(no_of_boards_to_show) => {
                if no_of_boards_to_show < MIN_NO_BOARDS_PER_PAGE.into()
                    || no_of_boards_to_show > MAX_NO_BOARDS_PER_PAGE.into()
                {
                    error!("Invalid number of boards to show: {}, Resetting to default number of boards to show", no_of_boards_to_show);
                    NO_OF_BOARDS_PER_PAGE
                } else {
                    no_of_boards_to_show as u16
                }
            }
            None => {
                error!("Number of boards to show is not a number, Resetting to default number of boards to show");
                NO_OF_BOARDS_PER_PAGE
            }
        };
        let enable_mouse_support = match serde_json_object["enable_mouse_support"].as_bool() {
            Some(enable_mouse_support) => enable_mouse_support,
            None => {
                error!("Enable Mouse Support is not a boolean, Resetting to default value");
                true
            }
        };
        let default_theme = match serde_json_object["default_theme"].as_str() {
            Some(default_theme) => default_theme.to_string(),
            None => {
                error!("Default Theme is not a string, Resetting to default theme");
                Theme::default().name
            }
        };
        let date_format = match serde_json_object["date_format"].as_str() {
            Some(date_format) => match date_format {
                "DayMonthYear" => DateFormat::DayMonthYear,
                "MonthDayYear" => DateFormat::MonthDayYear,
                "YearMonthDay" => DateFormat::YearMonthDay,
                "DayMonthYearTime" => DateFormat::DayMonthYearTime,
                "MonthDayYearTime" => DateFormat::MonthDayYearTime,
                "YearMonthDayTime" => DateFormat::YearMonthDayTime,
                _ => {
                    error!(
                        "Invalid date format: {}, Resetting to default date format",
                        date_format
                    );
                    DateFormat::default()
                }
            },
            None => {
                error!("Date Format is not a string, Resetting to default date format");
                DateFormat::default()
            }
        };
        let show_line_numbers = match serde_json_object["show_line_numbers"].as_bool() {
            Some(show_line_numbers) => show_line_numbers,
            None => {
                error!("Show Line Numbers is not a boolean, Resetting to default value");
                true
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
            enable_mouse_support,
            default_theme,
            date_format,
            show_line_numbers,
        })
    }
}

pub fn get_term_bg_color() -> (u8, u8, u8) {
    // TODO: make this work on windows and unix
    (0, 0, 0)
}

pub fn date_format_finder(date_string: &str) -> Result<DateFormat, String> {
    let date_formats = DateFormat::get_all_date_formats();
    for date_format in date_formats {
        if DateFormat::all_formats_with_time().contains(&date_format) {
            match NaiveDateTime::parse_from_str(date_string, date_format.to_parser_string()) {
                Ok(_) => return Ok(date_format),
                Err(_) => {
                    continue;
                }
            }
        } else {
            match NaiveDate::parse_from_str(date_string, date_format.to_parser_string()) {
                Ok(_) => return Ok(date_format),
                Err(_) => {
                    continue;
                }
            }
        }
    }
    Err("Invalid date format".to_string())
}

pub fn date_format_converter(date_string: &str, date_format: DateFormat) -> Result<String, String> {
    if date_string == FIELD_NOT_SET || date_string.is_empty() {
        return Ok(date_string.to_string());
    }
    let given_date_format = date_format_finder(date_string)?;
    let all_formats_with_time = DateFormat::all_formats_with_time();
    let all_formats_without_time = DateFormat::all_formats_without_time();
    if all_formats_with_time.contains(&given_date_format)
        && all_formats_without_time.contains(&date_format)
    {
        let naive_date_time =
            NaiveDateTime::parse_from_str(date_string, given_date_format.to_parser_string());
        if let Ok(naive_date_time) = naive_date_time {
            let naive_date = NaiveDate::from_ymd_opt(
                naive_date_time.year(),
                naive_date_time.month(),
                naive_date_time.day(),
            );
            if let Some(naive_date) = naive_date {
                return Ok(naive_date
                    .format(date_format.to_parser_string())
                    .to_string());
            } else {
                Err("Invalid date format".to_string())
            }
        } else {
            Err("Invalid date format".to_string())
        }
    } else if all_formats_without_time.contains(&given_date_format)
        && all_formats_with_time.contains(&date_format)
    {
        let naive_date =
            NaiveDate::parse_from_str(date_string, given_date_format.to_parser_string());
        if let Ok(naive_date) = naive_date {
            let default_time = NaiveTime::from_hms_opt(0, 0, 0);
            if let Some(default_time) = default_time {
                let naive_date_time = NaiveDateTime::new(naive_date, default_time);
                return Ok(naive_date_time
                    .format(date_format.to_parser_string())
                    .to_string());
            } else {
                Err("Invalid date format".to_string())
            }
        } else {
            Err("Invalid date format".to_string())
        }
    } else if all_formats_with_time.contains(&given_date_format)
        && all_formats_with_time.contains(&date_format)
    {
        let naive_date_time =
            NaiveDateTime::parse_from_str(date_string, given_date_format.to_parser_string());
        if let Ok(naive_date_time) = naive_date_time {
            return Ok(naive_date_time
                .format(date_format.to_parser_string())
                .to_string());
        } else {
            Err("Invalid date format".to_string())
        }
    } else if all_formats_without_time.contains(&given_date_format)
        && all_formats_without_time.contains(&date_format)
    {
        let naive_date =
            NaiveDate::parse_from_str(date_string, given_date_format.to_parser_string());
        if let Ok(naive_date) = naive_date {
            return Ok(naive_date
                .format(date_format.to_parser_string())
                .to_string());
        } else {
            Err("Invalid date format".to_string())
        }
    } else {
        Err("Invalid date format".to_string())
    }
}

pub async fn handle_exit(app: &mut App<'_>) {
    if app.config.save_on_exit {
        app.dispatch(IoEvent::AutoSave).await;
    }
}
