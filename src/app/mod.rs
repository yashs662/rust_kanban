use self::{
    app_helper::{
        handle_edit_keybinding_mode, handle_general_actions, handle_mouse_action,
        handle_user_input_mode, prepare_config_for_new_app,
    },
    kanban::{Board, Boards, Card, CardPriority},
    state::{AppStatus, Focus, KeyBindings, UiMode},
};
use crate::{
    app::{actions::Action, kanban::CardStatus, state::KeyBindingEnum},
    constants::{
        DEFAULT_CARD_WARNING_DUE_DATE_DAYS, DEFAULT_TICKRATE, DEFAULT_TOAST_DURATION,
        DEFAULT_UI_MODE, FIELD_NA, FIELD_NOT_SET, IO_EVENT_WAIT_TIME, MAX_NO_BOARDS_PER_PAGE,
        MAX_NO_CARDS_PER_BOARD, MAX_TICKRATE, MAX_WARNING_DUE_DATE_DAYS, MIN_NO_BOARDS_PER_PAGE,
        MIN_NO_CARDS_PER_BOARD, MIN_TICKRATE, MIN_WARNING_DUE_DATE_DAYS,
        MOUSE_OUT_OF_BOUNDS_COORDINATES, NO_OF_BOARDS_PER_PAGE, NO_OF_CARDS_PER_BOARD,
    },
    inputs::{key::Key, mouse::Mouse},
    io::{
        data_handler::{self, get_available_local_save_files, get_default_save_directory},
        io_handler::{refresh_visible_boards_and_cards, CloudData},
        logger::{get_logs, RUST_KANBAN_LOGGER},
        IoEvent,
    },
    ui::{
        text_box::TextBox,
        ui_helper,
        widgets::{CloseButtonWidget, CommandPaletteWidget, ToastType, ToastWidget},
        TextColorOptions, TextModifierOptions, Theme,
    },
};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use linked_hash_map::LinkedHashMap;
use log::{debug, error};
use ratatui::{
    widgets::{ListState, TableState},
    Frame,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
    vec,
};
use strum::IntoEnumIterator;
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

pub struct Widgets {
    pub command_palette: CommandPaletteWidget,
    pub close_button_widget: CloseButtonWidget,
    pub toasts: Vec<ToastWidget>,
}

impl Widgets {
    pub fn new(theme: Theme, debug_mode: bool) -> Self {
        Self {
            command_palette: CommandPaletteWidget::new(debug_mode),
            close_button_widget: CloseButtonWidget::new(theme.general_style),
            toasts: vec![],
        }
    }
}

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
    pub visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>>,
    pub last_io_event_time: Option<Instant>,
    pub all_themes: Vec<Theme>,
    pub current_theme: Theme,
    pub action_history_manager: ActionHistoryManager,
    pub main_menu: MainMenu,
    pub widgets: Widgets,
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
        let (config, config_errors, toasts) = prepare_config_for_new_app(theme.clone());
        let default_theme = config.default_theme.clone();
        let theme_in_all = all_themes.iter().find(|t| t.name == default_theme);
        if let Some(theme_in_all) = theme_in_all {
            theme = theme_in_all.clone();
        }
        let mut widgets = Widgets::new(theme.clone(), debug_mode);
        widgets.toasts = toasts;
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
                app.send_error_toast(error, None);
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
            self.send_error_toast("Error in handling request please, restart the app", None);
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
        if self.state.ui_mode == UiMode::MainMenu {
            self.main_menu_next();
        } else if self.state.focus == Focus::NoFocus {
            self.state.set_focus(Focus::Body);
        }
        self.state.app_status = AppStatus::initialized()
    }
    pub fn loaded(&mut self) {
        self.is_loading = false;
    }
    pub fn get_current_focus(&self) -> &Focus {
        &self.state.focus
    }
    pub fn clear_user_input_state(&mut self) {
        self.state.current_user_input = String::new();
        self.state.last_user_input = None;
    }
    pub fn set_config_state(&mut self, config_state: TableState) {
        self.state.app_table_states.config = config_state;
    }
    pub fn config_next(&mut self) {
        let i = match self.state.app_table_states.config.selected() {
            Some(i) => {
                if i >= self.config.to_view_list().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.app_table_states.config.select(Some(i));
    }
    pub fn config_prv(&mut self) {
        let i = match self.state.app_table_states.config.selected() {
            Some(i) => {
                if i == 0 {
                    self.config.to_view_list().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.app_table_states.config.select(Some(i));
    }
    pub fn main_menu_next(&mut self) {
        let i = match self.state.app_list_states.main_menu.selected() {
            Some(i) => {
                if i >= self.main_menu.all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.app_list_states.main_menu.select(Some(i));
    }
    pub fn main_menu_prv(&mut self) {
        let i = match self.state.app_list_states.main_menu.selected() {
            Some(i) => {
                if i == 0 {
                    self.main_menu.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.app_list_states.main_menu.select(Some(i));
    }
    pub fn load_save_next(&mut self, cloud_mode: bool) {
        let i = match self.state.app_list_states.load_save.selected() {
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
        self.state.app_list_states.load_save.select(Some(i));
    }
    pub fn load_save_prv(&mut self, cloud_mode: bool) {
        let i = match self.state.app_list_states.load_save.selected() {
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
        self.state.app_list_states.load_save.select(Some(i));
    }
    pub fn config_state(&self) -> &TableState {
        &self.state.app_table_states.config
    }
    pub fn set_ui_mode(&mut self, ui_mode: UiMode) {
        self.state.prev_ui_mode = Some(self.state.ui_mode);
        self.state.ui_mode = ui_mode;
        let available_focus_targets = self.state.ui_mode.get_available_targets();
        if !available_focus_targets.contains(&self.state.focus) {
            if available_focus_targets.is_empty() {
                self.state.set_focus(Focus::NoFocus);
            } else {
                self.state.set_focus(available_focus_targets[0]);
            }
        }
    }
    pub fn edit_keybindings_next(&mut self) {
        let keybinding_iterator = self.config.keybindings.iter();
        let i = match self.state.app_table_states.edit_keybindings.selected() {
            Some(i) => {
                if i >= keybinding_iterator.count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.app_table_states.edit_keybindings.select(Some(i));
    }
    pub fn edit_keybindings_prv(&mut self) {
        let keybinding_iterator = self.config.keybindings.iter();
        let i = match self.state.app_table_states.edit_keybindings.selected() {
            Some(i) => {
                if i == 0 {
                    keybinding_iterator.count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.app_table_states.edit_keybindings.select(Some(i));
    }
    pub fn help_next(&mut self) {
        let all_keybinds: Vec<_> = self.config.keybindings.iter().collect();
        let i = match self.state.app_table_states.help.selected() {
            Some(i) => {
                if !all_keybinds.is_empty() {
                    if i >= (all_keybinds.len() / 2) - 1 {
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
        self.state.app_table_states.help.select(Some(i));
    }
    pub fn help_prv(&mut self) {
        let all_keybinds: Vec<_> = self.config.keybindings.iter().collect();
        let i = match self.state.app_table_states.help.selected() {
            Some(i) => {
                if !all_keybinds.is_empty() {
                    if i == 0 {
                        (all_keybinds.len() / 2) - 1
                    } else {
                        i - 1
                    }
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.app_table_states.help.select(Some(i));
    }
    pub fn select_default_view_next(&mut self) {
        let i = match self.state.app_list_states.default_view.selected() {
            Some(i) => {
                if i >= UiMode::view_modes_as_string().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.app_list_states.default_view.select(Some(i));
    }
    pub fn select_default_view_prv(&mut self) {
        let i = match self.state.app_list_states.default_view.selected() {
            Some(i) => {
                if i == 0 {
                    UiMode::view_modes_as_string().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.app_list_states.default_view.select(Some(i));
    }
    pub fn command_palette_command_search_prv(&mut self) {
        let i = match self
            .state
            .app_list_states
            .command_palette_command_search
            .selected()
        {
            Some(i) => {
                if self
                    .widgets
                    .command_palette
                    .command_search_results
                    .is_some()
                {
                    if i == 0 {
                        self.widgets
                            .command_palette
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
            .app_list_states
            .command_palette_command_search
            .select(Some(i));
    }
    pub fn command_palette_command_search_next(&mut self) {
        let i = match self
            .state
            .app_list_states
            .command_palette_command_search
            .selected()
        {
            Some(i) => {
                if self
                    .widgets
                    .command_palette
                    .command_search_results
                    .is_some()
                {
                    if i >= self
                        .widgets
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
            .app_list_states
            .command_palette_command_search
            .select(Some(i));
    }
    pub fn command_palette_card_search_next(&mut self) {
        let i = match self
            .state
            .app_list_states
            .command_palette_card_search
            .selected()
        {
            Some(i) => {
                if self.widgets.command_palette.card_search_results.is_some() {
                    if i >= self
                        .widgets
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
            .app_list_states
            .command_palette_card_search
            .select(Some(i));
    }
    pub fn command_palette_card_search_prv(&mut self) {
        let i = match self
            .state
            .app_list_states
            .command_palette_card_search
            .selected()
        {
            Some(i) => {
                if self.widgets.command_palette.card_search_results.is_some() {
                    if i == 0 {
                        self.widgets
                            .command_palette
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
            .app_list_states
            .command_palette_card_search
            .select(Some(i));
    }
    pub fn command_palette_board_search_next(&mut self) {
        let i = match self
            .state
            .app_list_states
            .command_palette_board_search
            .selected()
        {
            Some(i) => {
                if self.widgets.command_palette.board_search_results.is_some() {
                    if i >= self
                        .widgets
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
            .app_list_states
            .command_palette_board_search
            .select(Some(i));
    }
    pub fn command_palette_board_search_prv(&mut self) {
        let i = match self
            .state
            .app_list_states
            .command_palette_board_search
            .selected()
        {
            Some(i) => {
                if self.widgets.command_palette.board_search_results.is_some() {
                    if i == 0 {
                        self.widgets
                            .command_palette
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
            .app_list_states
            .command_palette_board_search
            .select(Some(i));
    }
    pub fn send_info_toast(&mut self, message: &str, custom_duration: Option<Duration>) {
        if let Some(duration) = custom_duration {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Info,
                self.current_theme.clone(),
            ));
        } else {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Info,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn send_error_toast(&mut self, message: &str, custom_duration: Option<Duration>) {
        if let Some(duration) = custom_duration {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Error,
                self.current_theme.clone(),
            ));
        } else {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Error,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn send_warning_toast(&mut self, message: &str, custom_duration: Option<Duration>) {
        if let Some(duration) = custom_duration {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Warning,
                self.current_theme.clone(),
            ));
        } else {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Warning,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn send_loading_toast(&mut self, message: &str, custom_duration: Option<Duration>) {
        if let Some(duration) = custom_duration {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Loading,
                self.current_theme.clone(),
            ));
        } else {
            self.widgets.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Loading,
                self.current_theme.clone(),
            ));
        }
    }
    pub fn select_card_status_prv(&mut self) {
        let i = match self.state.app_list_states.card_status_selector.selected() {
            Some(i) => {
                if i == 0 {
                    CardStatus::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .card_status_selector
            .select(Some(i));
    }
    pub fn select_card_status_next(&mut self) {
        let i = match self.state.app_list_states.card_status_selector.selected() {
            Some(i) => {
                if i >= CardStatus::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .card_status_selector
            .select(Some(i));
    }
    pub fn increase_loading_toast_time(&mut self, msg: &str, increase_by: Duration) {
        let toast = self.widgets.toasts.iter_mut().find(|x| x.message == msg);
        if toast.is_none() {
            debug!("No toast found with message: {}", msg);
            return;
        }
        let toast = toast.unwrap();
        toast.duration += increase_by;
    }
    pub fn select_change_theme_next(&mut self) {
        let i = match self.state.app_list_states.theme_selector.selected() {
            Some(i) => {
                if i >= self.all_themes.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.app_list_states.theme_selector.select(Some(i));
        self.current_theme = self.all_themes[i].clone();
    }
    pub fn select_change_theme_prv(&mut self) {
        let i = match self.state.app_list_states.theme_selector.selected() {
            Some(i) => {
                if i == 0 {
                    self.all_themes.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.app_list_states.theme_selector.select(Some(i));
        self.current_theme = self.all_themes[i].clone();
    }
    pub fn select_create_theme_next(&mut self) {
        let theme_rows_len = Theme::default().to_rows(self).1.len();
        let i = match self.state.app_table_states.theme_editor.selected() {
            Some(i) => {
                if i >= theme_rows_len - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.app_table_states.theme_editor.select(Some(i));
    }
    pub fn select_create_theme_prv(&mut self) {
        let theme_rows_len = Theme::default().to_rows(self).1.len();
        let i = match self.state.app_table_states.theme_editor.selected() {
            Some(i) => {
                if i == 0 {
                    theme_rows_len - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.app_table_states.theme_editor.select(Some(i));
    }
    pub fn select_edit_style_fg_next(&mut self) {
        let i = match self.state.app_list_states.edit_specific_style.0.selected() {
            Some(i) => {
                if i >= TextColorOptions::to_iter().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .edit_specific_style
            .0
            .select(Some(i));
    }
    pub fn select_edit_style_fg_prv(&mut self) {
        let i = match self.state.app_list_states.edit_specific_style.0.selected() {
            Some(i) => {
                if i == 0 {
                    TextColorOptions::to_iter().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .edit_specific_style
            .0
            .select(Some(i));
    }
    pub fn select_edit_style_bg_next(&mut self) {
        let i = match self.state.app_list_states.edit_specific_style.1.selected() {
            Some(i) => {
                if i >= TextColorOptions::to_iter().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .edit_specific_style
            .1
            .select(Some(i));
    }
    pub fn select_edit_style_bg_prv(&mut self) {
        let i = match self.state.app_list_states.edit_specific_style.1.selected() {
            Some(i) => {
                if i == 0 {
                    TextColorOptions::to_iter().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .edit_specific_style
            .1
            .select(Some(i));
    }
    pub fn select_edit_style_modifier_next(&mut self) {
        let i = match self.state.app_list_states.edit_specific_style.2.selected() {
            Some(i) => {
                if i >= TextModifierOptions::to_iter().count() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .edit_specific_style
            .2
            .select(Some(i));
    }
    pub fn select_edit_style_modifier_prv(&mut self) {
        let i = match self.state.app_list_states.edit_specific_style.2.selected() {
            Some(i) => {
                if i == 0 {
                    TextModifierOptions::to_iter().count() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .edit_specific_style
            .2
            .select(Some(i));
    }
    pub fn select_card_priority_next(&mut self) {
        let i = match self.state.app_list_states.card_priority_selector.selected() {
            Some(i) => {
                if i >= CardPriority::all().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .card_priority_selector
            .select(Some(i));
    }
    pub fn select_card_priority_prv(&mut self) {
        let i = match self.state.app_list_states.card_priority_selector.selected() {
            Some(i) => {
                if i == 0 {
                    CardPriority::all().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .card_priority_selector
            .select(Some(i));
    }
    pub fn filter_by_tag_popup_next(&mut self) {
        let all_tags_len = if self.state.all_available_tags.is_some() {
            self.state.all_available_tags.clone().unwrap().len()
        } else {
            0
        };
        if all_tags_len > 0 {
            let i = match self.state.app_list_states.filter_by_tag_list.selected() {
                Some(i) => {
                    if i >= all_tags_len - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.state
                .app_list_states
                .filter_by_tag_list
                .select(Some(i));
        }
    }
    pub fn filter_by_tag_popup_prv(&mut self) {
        let all_tags_len = if self.state.all_available_tags.is_some() {
            self.state.all_available_tags.clone().unwrap().len()
        } else {
            0
        };
        if all_tags_len > 0 {
            let i = match self.state.app_list_states.filter_by_tag_list.selected() {
                Some(i) => {
                    if i == 0 {
                        all_tags_len - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.state
                .app_list_states
                .filter_by_tag_list
                .select(Some(i));
        }
    }
    pub fn change_date_format_popup_next(&mut self) {
        let i = match self.state.app_list_states.date_format_selector.selected() {
            Some(i) => {
                if i >= DateFormat::get_all_date_formats().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .date_format_selector
            .select(Some(i));
    }
    pub fn change_date_format_popup_prv(&mut self) {
        let i = match self.state.app_list_states.date_format_selector.selected() {
            Some(i) => {
                if i == 0 {
                    DateFormat::get_all_date_formats().len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state
            .app_list_states
            .date_format_selector
            .select(Some(i));
    }
    pub fn undo(&mut self) {
        if self.action_history_manager.history_index == 0 {
            self.send_error_toast("No more actions to undo", None);
        } else {
            let history_index = self.action_history_manager.history_index - 1;
            let history = self.action_history_manager.history[history_index].clone();
            match history {
                ActionHistory::DeleteCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.add_card(card.clone());
                        self.action_history_manager.history_index -= 1;
                        refresh_visible_boards_and_cards(self);
                        self.send_info_toast(&format!("Undo Delete Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not undo delete card '{}' as the board with id '{:?}' was not found", card.name, board_id), None);
                    }
                }
                ActionHistory::CreateCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.remove_card_with_id(card.id);
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
                        self.send_error_toast(
                            &format!(
                                "Could not undo move card '{}' as the index's were invalid",
                                card.name
                            ),
                            None,
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
                    self.send_info_toast(&format!("Undo Move Card '{}'", card.name), None);
                }
                ActionHistory::MoveCardWithinBoard(board_id, moved_from_index, moved_to_index) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
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
                        let card_name = board
                            .cards
                            .get_mut_card_with_index(moved_to_index)
                            .unwrap()
                            .name
                            .clone();
                        board.cards.swap(moved_from_index, moved_to_index);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index -= 1;
                        self.send_info_toast(&format!("Undo Move Card '{}'", card_name), None);
                    } else {
                        self.send_error_toast(&format!("Could not undo move card '{}' as the board with id '{:?}' was not found",FIELD_NA, board_id), None);
                    }
                }
                ActionHistory::DeleteBoard(board) => {
                    self.boards.add_board(board.clone());
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    self.send_info_toast(&format!("Undo Delete Board '{}'", board.name), None);
                }
                ActionHistory::CreateBoard(board) => {
                    self.boards.remove_board_with_id(board.id);
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index -= 1;
                    self.send_info_toast(&format!("Undo Create Board '{}'", board.name), None);
                }
                ActionHistory::EditCard(old_card, _, board_id) => {
                    let mut card_name = String::new();
                    let mut card_found = false;
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        if let Some(card) = board.cards.get_mut_card_with_id(old_card.id) {
                            *card = old_card.clone();
                            card_name = card.name.clone();
                            card_found = true;
                        } else {
                            self.send_error_toast(
                                &format!(
                                    "Could not undo edit card '{}' as the card was not found",
                                    old_card.name
                                ),
                                None,
                            );
                        }
                    } else {
                        self.send_error_toast(&format!("Could not undo edit card '{}' as the board with id '{:?}' was not found", old_card.name, board_id), None);
                    }
                    if card_found {
                        self.action_history_manager.history_index -= 1;
                    }
                    if !card_name.is_empty() {
                        self.send_info_toast(&format!("Undo Edit Card '{}'", card_name), None);
                        refresh_visible_boards_and_cards(self);
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
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.remove_card_with_id(card.id);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        self.send_info_toast(&format!("Redo Delete Card '{}'", card.name), None);
                    } else {
                        self.send_error_toast(&format!("Could not redo delete card '{}' as the board with id '{:?}' was not found", card.name, board_id), None);
                    }
                }
                ActionHistory::CreateCard(card, board_id) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        board.cards.add_card(card.clone());
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
                        self.send_error_toast(
                            &format!(
                                "Could not redo move card '{}' as the index's were invalid",
                                card.name
                            ),
                            None,
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
                    self.send_info_toast(&format!("Redo Move Card '{}'", card.name), None);
                }
                ActionHistory::MoveCardWithinBoard(board_id, moved_from_index, moved_to_index) => {
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
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
                        let card_name = board
                            .cards
                            .get_card_with_index(moved_to_index)
                            .unwrap()
                            .name
                            .clone();
                        board.cards.swap(moved_from_index, moved_to_index);
                        refresh_visible_boards_and_cards(self);
                        self.action_history_manager.history_index += 1;
                        self.send_info_toast(&format!("Redo Move Card '{}'", card_name), None);
                    } else {
                        self.send_error_toast(&format!("Could not redo move card '{}' as the board with id '{:?}' was not found", FIELD_NA, board_id), None);
                    }
                }
                ActionHistory::DeleteBoard(board) => {
                    self.boards.remove_board_with_id(board.id);
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    self.send_info_toast(&format!("Redo Delete Board '{}'", board.name), None);
                }
                ActionHistory::CreateBoard(board) => {
                    self.boards.add_board(board.clone());
                    refresh_visible_boards_and_cards(self);
                    self.action_history_manager.history_index += 1;
                    self.send_info_toast(&format!("Redo Create Board '{}'", board.name), None);
                }
                ActionHistory::EditCard(_, new_card, board_id) => {
                    let mut card_name = String::new();
                    let mut card_found = false;
                    if let Some(board) = self.boards.get_mut_board_with_id(board_id) {
                        if let Some(card) = board.cards.get_mut_card_with_id(new_card.id) {
                            *card = new_card.clone();
                            card_name = card.name.clone();
                            card_found = true;
                        } else {
                            self.send_error_toast(
                                &format!(
                                    "Could not redo edit card '{}' as the card was not found",
                                    new_card.name
                                ),
                                None,
                            );
                        }
                    } else {
                        self.send_error_toast(&format!("Could not redo edit card '{}' as the board with id '{:?}' was not found", new_card.name, board_id), None);
                    }
                    if card_found {
                        self.action_history_manager.history_index += 1;
                    }
                    if !card_name.is_empty() {
                        self.send_info_toast(&format!("Redo Edit Card '{}'", card_name), None);
                        refresh_visible_boards_and_cards(self);
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

    pub fn render(self, rect: &mut Frame, app: &mut App) {
        let current_focus = app.state.focus;
        if !self.get_available_targets().contains(&current_focus)
            && !self.get_available_targets().is_empty()
        {
            app.state.set_focus(self.get_available_targets()[0]);
        }
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

#[derive(Debug, Clone, Default)]
pub struct AppListStates {
    pub card_priority_selector: ListState,
    pub card_status_selector: ListState,
    pub card_view_comment_list: ListState,
    pub card_view_list: ListState,
    pub card_view_tag_list: ListState,
    pub command_palette_board_search: ListState,
    pub command_palette_card_search: ListState,
    pub command_palette_command_search: ListState,
    pub date_format_selector: ListState,
    pub default_view: ListState,
    pub edit_specific_style: (ListState, ListState, ListState),
    pub filter_by_tag_list: ListState,
    pub load_save: ListState,
    pub logs: ListState,
    pub main_menu: ListState,
    pub theme_selector: ListState,
}

#[derive(Debug, Clone, Default)]
pub struct AppTableStates {
    pub config: TableState,
    pub edit_keybindings: TableState,
    pub help: TableState,
    pub theme_editor: TableState,
}

#[derive(Debug, Clone)]
pub struct AppFormStates {
    pub login: (Vec<String>, bool),
    pub new_board: Vec<String>,
    pub new_card: Vec<String>,
    pub reset_password: (Vec<String>, bool),
    pub signup: (Vec<String>, bool),
}

impl Default for AppFormStates {
    fn default() -> Self {
        AppFormStates {
            login: (vec![String::new(), String::new()], false),
            new_board: vec![String::new(), String::new()],
            new_card: vec![String::new(), String::new(), String::new()],
            reset_password: (
                vec![String::new(), String::new(), String::new(), String::new()],
                false,
            ),
            signup: (vec![String::new(), String::new(), String::new()], false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState<'a> {
    pub all_available_tags: Option<Vec<(String, u32)>>,
    pub app_form_states: AppFormStates,
    pub app_list_states: AppListStates,
    pub app_status: AppStatus,
    pub app_table_states: AppTableStates,
    pub card_being_edited: Option<((u64, u64), Card)>, // (board_id, card)
    pub card_description_text_buffer: Option<TextBox<'a>>,
    pub card_drag_mode: bool,
    pub cloud_data: Option<Vec<CloudData>>,
    pub config_item_being_edited: Option<usize>,
    pub current_board_id: Option<(u64, u64)>,
    pub current_card_id: Option<(u64, u64)>,
    pub current_cursor_position: Option<usize>,
    pub current_mouse_coordinates: (u16, u16),
    pub current_user_input: String,
    pub debug_menu_toggled: bool,
    pub default_theme_mode: bool,
    pub edited_keybinding: Option<Vec<Key>>,
    pub encryption_key_from_arguments: Option<String>,
    pub filter_tags: Option<Vec<String>>,
    pub focus: Focus,
    pub hovered_board: Option<(u64, u64)>,
    pub hovered_card_dimensions: Option<(u16, u16)>,
    pub hovered_card: Option<((u64, u64), (u64, u64))>,
    pub last_mouse_action: Option<Mouse>,
    pub last_reset_password_link_sent_time: Option<Instant>,
    pub last_user_input: Option<String>,
    pub mouse_focus: Option<Focus>,
    pub mouse_list_index: Option<u16>,
    pub no_of_cards_to_show: u16,
    pub popup_mode: Option<PopupMode>,
    pub prev_focus: Option<Focus>,
    pub prev_ui_mode: Option<UiMode>,
    pub preview_file_name: Option<String>,
    pub preview_visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>>,
    pub previous_mouse_coordinates: (u16, u16),
    pub term_background_color: (u8, u8, u8),
    pub theme_being_edited: Theme,
    pub ui_mode: UiMode,
    pub ui_render_time: Vec<u128>,
    pub user_login_data: UserLoginData,
    // TODO: Improve this, it feels like a hack
    pub path_check_state: PathCheckState,
}

#[derive(Debug, Clone, Default)]
pub struct PathCheckState {
    pub path_last_checked: String,
    pub path_exists: bool,
    pub potential_completion: Option<String>,
    pub recheck_required: bool,
    pub path_check_mode: bool,
}

impl AppState<'_> {
    pub fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }
    pub fn get_card_being_edited(&self) -> Option<((u64, u64), Card)> {
        self.card_being_edited.clone()
    }
    pub fn get_theme_being_edited(&self) -> Theme {
        self.theme_being_edited.clone()
    }
}

impl Default for AppState<'_> {
    fn default() -> AppState<'static> {
        AppState {
            all_available_tags: None,
            app_form_states: AppFormStates::default(),
            app_list_states: AppListStates::default(),
            app_status: AppStatus::default(),
            app_table_states: AppTableStates::default(),
            card_being_edited: None,
            card_description_text_buffer: None,
            card_drag_mode: false,
            cloud_data: None,
            config_item_being_edited: None,
            current_board_id: None,
            current_card_id: None,
            current_cursor_position: None,
            current_mouse_coordinates: MOUSE_OUT_OF_BOUNDS_COORDINATES, // make sure it's out of bounds when mouse mode is disabled
            current_user_input: String::new(),
            debug_menu_toggled: false,
            default_theme_mode: false,
            edited_keybinding: None,
            encryption_key_from_arguments: None,
            filter_tags: None,
            focus: Focus::NoFocus,
            hovered_board: None,
            hovered_card_dimensions: None,
            hovered_card: None,
            last_mouse_action: None,
            last_reset_password_link_sent_time: None,
            last_user_input: None,
            mouse_focus: None,
            mouse_list_index: None,
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            popup_mode: None,
            prev_focus: None,
            prev_ui_mode: None,
            preview_file_name: None,
            preview_visible_boards_and_cards: LinkedHashMap::new(),
            previous_mouse_coordinates: MOUSE_OUT_OF_BOUNDS_COORDINATES,
            term_background_color: get_term_bg_color(),
            theme_being_edited: Theme::default(),
            ui_mode: DEFAULT_UI_MODE,
            ui_render_time: Vec::new(),
            user_login_data: UserLoginData {
                email_id: None,
                auth_token: None,
                refresh_token: None,
                user_id: None,
            },
            path_check_state: PathCheckState::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserLoginData {
    pub auth_token: Option<String>,
    pub email_id: Option<String>,
    pub refresh_token: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum DateFormat {
    DayMonthYear,
    #[default]
    DayMonthYearTime,
    MonthDayYear,
    MonthDayYearTime,
    YearMonthDay,
    YearMonthDayTime,
}

impl DateFormat {
    pub fn to_human_readable_string(&self) -> &str {
        match self {
            DateFormat::DayMonthYear => "DD/MM/YYYY",
            DateFormat::DayMonthYearTime => "DD/MM/YYYY-HH:MM:SS",
            DateFormat::MonthDayYear => "MM/DD/YYYY",
            DateFormat::MonthDayYearTime => "MM/DD/YYYY-HH:MM:SS",
            DateFormat::YearMonthDay => "YYYY/MM/DD",
            DateFormat::YearMonthDayTime => "YYYY/MM/DD-HH:MM:SS",
        }
    }
    pub fn to_parser_string(&self) -> &str {
        match self {
            DateFormat::DayMonthYear => "%d/%m/%Y",
            DateFormat::DayMonthYearTime => "%d/%m/%Y-%H:%M:%S",
            DateFormat::MonthDayYear => "%m/%d/%Y",
            DateFormat::MonthDayYearTime => "%m/%d/%Y-%H:%M:%S",
            DateFormat::YearMonthDay => "%Y/%m/%d",
            DateFormat::YearMonthDayTime => "%Y/%m/%d-%H:%M:%S",
        }
    }
    pub fn from_json_string(json_string: &str) -> Option<DateFormat> {
        match json_string {
            "DayMonthYear" => Some(DateFormat::DayMonthYear),
            "DayMonthYearTime" => Some(DateFormat::DayMonthYearTime),
            "MonthDayYear" => Some(DateFormat::MonthDayYear),
            "MonthDayYearTime" => Some(DateFormat::MonthDayYearTime),
            "YearMonthDay" => Some(DateFormat::YearMonthDay),
            "YearMonthDayTime" => Some(DateFormat::YearMonthDayTime),
            _ => None,
        }
    }
    pub fn from_human_readable_string(human_readable_string: &str) -> Option<DateFormat> {
        match human_readable_string {
            "DD/MM/YYYY" => Some(DateFormat::DayMonthYear),
            "DD/MM/YYYY-HH:MM:SS" => Some(DateFormat::DayMonthYearTime),
            "MM/DD/YYYY" => Some(DateFormat::MonthDayYear),
            "MM/DD/YYYY-HH:MM:SS" => Some(DateFormat::MonthDayYearTime),
            "YYYY/MM/DD" => Some(DateFormat::YearMonthDay),
            "YYYY/MM/DD-HH:MM:SS" => Some(DateFormat::YearMonthDayTime),
            _ => None,
        }
    }
    pub fn get_all_date_formats() -> Vec<DateFormat> {
        vec![
            DateFormat::DayMonthYear,
            DateFormat::DayMonthYearTime,
            DateFormat::MonthDayYear,
            DateFormat::MonthDayYearTime,
            DateFormat::YearMonthDay,
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

impl Display for DateFormat {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_human_readable_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub always_load_last_save: bool,
    pub auto_login: bool,
    pub date_format: DateFormat,
    pub default_theme: String,
    pub default_view: UiMode,
    pub disable_animations: bool,
    pub disable_scroll_bar: bool,
    pub enable_mouse_support: bool,
    pub keybindings: KeyBindings,
    pub no_of_boards_to_show: u16,
    pub no_of_cards_to_show: u16,
    pub save_directory: PathBuf,
    pub save_on_exit: bool,
    pub show_line_numbers: bool,
    pub tickrate: u16,
    pub warning_delta: u16,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_view = DEFAULT_UI_MODE;
        let default_theme = Theme::default();
        Self {
            always_load_last_save: true,
            auto_login: true,
            date_format: DateFormat::default(),
            default_theme: default_theme.name,
            default_view,
            disable_animations: false,
            disable_scroll_bar: false,
            enable_mouse_support: true,
            keybindings: KeyBindings::default(),
            no_of_boards_to_show: NO_OF_BOARDS_PER_PAGE,
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
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
                    ConfigEnum::DefaultTheme => (self.default_theme.clone(), 13),
                    ConfigEnum::DateFormat => (self.date_format.to_string(), 14),
                    ConfigEnum::Keybindings => ("".to_string(), 15),
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
            ConfigEnum::DateFormat => self.date_format.to_string(),
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
                app.send_info_toast("Config updated", None);
            } else {
                app.send_error_toast("Could not write to config file", None);
            }
        } else {
            let error_message = format!("Could not edit config: {}", result.unwrap_err());
            error!("{}", error_message);
            app.send_error_toast(&error_message, None);
        }
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
        let (key, _) = &key_list[key_index];

        if !current_bindings.iter().any(|(k, _)| &k == key) {
            debug!("Invalid key: {}", key);
            error!("Unable to edit keybinding");
            return Err("Unable to edit keybinding 😢 ".to_string());
        }

        for new_value in value.iter() {
            for (k, v) in current_bindings.iter() {
                if v.contains(new_value) && &k != key {
                    error!("Value {} is already assigned to {}", new_value, k);
                    return Err(format!("Value {} is already assigned to {}", new_value, k));
                }
            }
        }

        debug!("Editing keybinding: {} to {:?}", key, value);

        match key {
            KeyBindingEnum::Accept => self.keybindings.accept = value,
            KeyBindingEnum::ChangeCardStatusToActive => {
                self.keybindings.change_card_status_to_active = value;
            }
            KeyBindingEnum::ChangeCardStatusToCompleted => {
                self.keybindings.change_card_status_to_completed = value;
            }
            KeyBindingEnum::ChangeCardStatusToStale => {
                self.keybindings.change_card_status_to_stale = value;
            }
            KeyBindingEnum::ClearAllToasts => {
                self.keybindings.clear_all_toasts = value;
            }
            KeyBindingEnum::DeleteBoard => {
                self.keybindings.delete_board = value;
            }
            KeyBindingEnum::DeleteCard => {
                self.keybindings.delete_card = value;
            }
            KeyBindingEnum::Down => {
                self.keybindings.down = value;
            }
            KeyBindingEnum::GoToMainMenu => {
                self.keybindings.go_to_main_menu = value;
            }
            KeyBindingEnum::GoToPreviousUIModeorCancel => {
                self.keybindings.go_to_previous_ui_mode_or_cancel = value;
            }
            KeyBindingEnum::HideUiElement => {
                self.keybindings.hide_ui_element = value;
            }
            KeyBindingEnum::Left => {
                self.keybindings.left = value;
            }
            KeyBindingEnum::MoveCardDown => {
                self.keybindings.move_card_down = value;
            }
            KeyBindingEnum::MoveCardLeft => {
                self.keybindings.move_card_left = value;
            }
            KeyBindingEnum::MoveCardRight => {
                self.keybindings.move_card_right = value;
            }
            KeyBindingEnum::MoveCardUp => {
                self.keybindings.move_card_up = value;
            }
            KeyBindingEnum::NewBoard => {
                self.keybindings.new_board = value;
            }
            KeyBindingEnum::NewCard => {
                self.keybindings.new_card = value;
            }
            KeyBindingEnum::NextFocus => {
                self.keybindings.next_focus = value;
            }
            KeyBindingEnum::OpenConfigMenu => {
                self.keybindings.open_config_menu = value;
            }
            KeyBindingEnum::PrvFocus => {
                self.keybindings.prv_focus = value;
            }
            KeyBindingEnum::Quit => {
                self.keybindings.quit = value;
            }
            KeyBindingEnum::Redo => {
                self.keybindings.redo = value;
            }
            KeyBindingEnum::ResetUI => {
                self.keybindings.reset_ui = value;
            }
            KeyBindingEnum::Right => {
                self.keybindings.right = value;
            }
            KeyBindingEnum::SaveState => {
                self.keybindings.save_state = value;
            }
            KeyBindingEnum::StopUserInput => {
                self.keybindings.stop_user_input = value;
            }
            KeyBindingEnum::TakeUserInput => {
                self.keybindings.take_user_input = value;
            }
            KeyBindingEnum::ToggleCommandPalette => {
                self.keybindings.toggle_command_palette = value;
            }
            KeyBindingEnum::Undo => {
                self.keybindings.undo = value;
            }
            KeyBindingEnum::Up => {
                self.keybindings.up = value;
            }
        }
        Ok(())
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

    fn json_config_keybindinds_checker(serde_json_object: &Value) -> KeyBindings {
        if let Some(keybindings) = serde_json_object["keybindings"].as_object() {
            let mut default_keybinds = KeyBindings::default();
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
                        default_keybinds.edit_keybinding(key, keybindings);
                    }
                } else {
                    Self::handle_invalid_keybinding(key);
                }
            }
            default_keybinds
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
                    default_config.save_directory
                }
            }
            None => {
                error!("Save Directory is not a string, Resetting to default save directory");
                default_config.save_directory
            }
        };
        let default_view = match serde_json_object["default_view"].as_str() {
            Some(ui_mode) => {
                let ui_mode = UiMode::from_json_string(ui_mode);
                if let Some(ui_mode) = ui_mode {
                    ui_mode
                } else {
                    error!("Invalid UiMode: {:?}, Resetting to default UiMode", ui_mode);
                    default_config.default_view
                }
            }
            None => {
                error!("Default View is not a string, Resetting to default UiMode");
                default_config.default_view
            }
        };
        let keybindings = AppConfig::json_config_keybindinds_checker(&serde_json_object);
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
        let default_theme = match serde_json_object["default_theme"].as_str() {
            Some(default_theme) => default_theme.to_string(),
            None => {
                error!("Default Theme is not a string, Resetting to default theme");
                default_config.default_theme
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
                    default_config.date_format
                }
            },
            None => {
                error!("Date Format is not a string, Resetting to default date format");
                default_config.date_format
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
                let ui_mode = UiMode::from_string(value);
                if ui_mode.is_some() {
                    Ok(())
                } else {
                    Err(format!("Invalid UiMode: {}", value))
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
                let date_format = DateFormat::from_human_readable_string(value);
                if date_format.is_some() {
                    Ok(())
                } else {
                    Err(format!("Invalid DateFormat: {}", value))
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
                config.default_view = UiMode::from_string(value).unwrap();
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
                config.date_format = DateFormat::from_human_readable_string(value).unwrap();
            }
            ConfigEnum::Keybindings => {
                debug!("Keybindings should not be called from edit_config");
                // Keybindings are handled separately
            }
        }
        Ok(())
    }
}

pub fn get_term_bg_color() -> (u8, u8, u8) {
    // TODO: Find a way to get the terminal background color
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

pub async fn handle_exit(app: &mut App<'_>) -> AppReturn {
    if app.config.save_on_exit {
        app.dispatch(IoEvent::AutoSave).await;
    }
    AppReturn::Exit
}
