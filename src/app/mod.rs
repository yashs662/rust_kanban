use linked_hash_map::LinkedHashMap;
use log::{debug, error, info};
use ratatui::widgets::{ListState, TableState};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
    time::{Duration, Instant},
    vec,
};

use self::{
    actions::Actions,
    app_helper::{
        handle_general_actions, handle_keybind_mode, handle_mouse_action, handle_user_input_mode,
        prepare_config_for_new_app,
    },
    kanban::{Board, Card, CardPriority},
    state::{AppStatus, Focus, KeyBindings, UiMode},
};
use crate::{
    app::{actions::Action, kanban::CardStatus},
    constants::{
        DEFAULT_CARD_WARNING_DUE_DATE_DAYS, DEFAULT_TICKRATE, DEFAULT_TOAST_DURATION,
        IO_EVENT_WAIT_TIME, MAX_NO_BOARDS_PER_PAGE, MAX_NO_CARDS_PER_BOARD, MIN_NO_BOARDS_PER_PAGE,
        MIN_NO_CARDS_PER_BOARD, MOUSE_OUT_OF_BOUNDS_COORDINATES, NO_OF_BOARDS_PER_PAGE,
        NO_OF_CARDS_PER_BOARD,
    },
    inputs::{key::Key, mouse::Mouse},
    io::{
        data_handler::{
            get_available_local_savefiles, get_config, get_default_save_directory,
            get_default_ui_mode,
        },
        handler::refresh_visible_boards_and_cards,
        IoEvent,
    },
    ui::{
        widgets::{CommandPaletteWidget, ToastType, ToastWidget},
        TextColorOptions, TextModifierOptions, Theme,
    },
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

/// The main application, containing the state
pub struct App {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Actions,
    is_loading: bool,
    pub state: AppState,
    pub boards: Vec<Board>,
    pub filtered_boards: Vec<Board>,
    pub config: AppConfig,
    pub config_item_being_edited: Option<usize>,
    pub card_being_edited: Option<(u128, Card)>, // (board_id, card)
    pub visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>>,
    pub command_palette: CommandPaletteWidget,
    pub last_io_event_time: Option<Instant>,
    pub all_themes: Vec<Theme>,
    pub theme: Theme,
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let mut state = AppState::default();
        let boards = vec![];
        let filtered_boards = vec![];
        let all_themes = Theme::all_default_themes();
        let mut theme = Theme::default();
        let config = prepare_config_for_new_app(&mut state, theme.clone());
        let default_theme = config.default_theme.clone();
        for t in all_themes.iter() {
            if t.name == default_theme {
                theme = t.clone();
                break;
            }
        }

        Self {
            io_tx,
            actions,
            is_loading,
            state,
            boards,
            filtered_boards,
            config,
            config_item_being_edited: None,
            card_being_edited: None,
            visible_boards_and_cards: LinkedHashMap::new(),
            command_palette: CommandPaletteWidget::new(),
            last_io_event_time: None,
            all_themes,
            theme,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        // check if we are in a user input mode
        if self.state.app_status == AppStatus::UserInput {
            handle_user_input_mode(self, key).await
        } else if self.state.app_status == AppStatus::KeyBindMode {
            handle_keybind_mode(self, key).await
        } else {
            handle_general_actions(self, key).await
        }
    }
    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        // check if last_io_event_time is more thant current time + IO_EVENT_WAIT_TIME in ms
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
        // Update contextual actions
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
                if i >= self.config.to_list().len() - 1 {
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
                    self.config.to_list().len() - 1
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
                let local_save_files = get_available_local_savefiles();
                let local_save_files_len = if let Some(local_save_files_len) = local_save_files {
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
            None => 0,
        };
        self.state.load_save_state.select(Some(i));
    }
    pub fn load_save_previous(&mut self) {
        let i = match self.state.load_save_state.selected() {
            Some(i) => {
                let local_save_files = get_available_local_savefiles();
                let local_save_files_len = if let Some(local_save_files_len) = local_save_files {
                    local_save_files_len.len()
                } else {
                    0
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
        self.state.prev_ui_mode = Some(self.state.ui_mode);
        self.state.ui_mode = ui_mode;
        let available_focus_targets = self.state.ui_mode.get_available_targets();
        if !available_focus_targets.contains(&self.state.focus) {
            // check if available focus targets is empty
            if available_focus_targets.is_empty() {
                self.state.focus = Focus::NoFocus;
            } else {
                self.state.focus = available_focus_targets[0];
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
                if self.command_palette.search_results.is_some() {
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
                if self.command_palette.search_results.is_some() {
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
                keybind_string.push(' ');
            }
            keybind_action.push(keybind_string);
            let action_translated_string = KeyBindings::str_to_action(keybinds.clone(), action)
                .unwrap_or(&Action::Quit)
                .to_string();
            keybind_action.push(action_translated_string);
            keybind_action_list.push(keybind_action);
        }

        let default_action_iter = default_actions.actions().iter();
        // append to keybind_action_list if the keybind is not already in the list
        for action in default_action_iter {
            let str_action = action.to_string();
            if !keybind_action_list.iter().any(|x| x[1] == str_action) {
                let mut keybind_action = Vec::new();
                let action_keys = action
                    .keys()
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
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Info,
                self.theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Info,
                self.theme.clone(),
            ));
        }
    }
    pub fn send_error_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Error,
                self.theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Error,
                self.theme.clone(),
            ));
        }
    }
    pub fn send_warning_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Warning,
                self.theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Warning,
                self.theme.clone(),
            ));
        }
    }
    pub fn send_loading_toast(&mut self, message: &str, duration: Option<Duration>) {
        if let Some(duration) = duration {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                duration,
                ToastType::Loading,
                self.theme.clone(),
            ));
        } else {
            self.state.toasts.push(ToastWidget::new(
                message.to_string(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Loading,
                self.theme.clone(),
            ));
        }
    }
    pub fn select_card_status_prev(&mut self) {
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
    }
    pub fn select_change_theme_prev(&mut self) {
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
    pub fn select_create_theme_prev(&mut self) {
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
    pub fn select_edit_style_fg_prev(&mut self) {
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
    pub fn select_edit_style_bg_prev(&mut self) {
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
    pub fn select_edit_style_modifier_prev(&mut self) {
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
    pub fn select_card_priority_prev(&mut self) {
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
    pub fn filter_by_tag_popup_prev(&mut self) {
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MainMenuItem {
    View,
    Config,
    Help,
    LoadSave,
    Quit,
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
    pub items: Vec<MainMenuItem>,
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
            _ => MainMenuItem::Quit,
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
    ChangeTheme,
    ThemeEditor,
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
            PopupMode::ChangeTheme => write!(f, "Change Theme"),
            PopupMode::ThemeEditor => write!(f, "Edit Theme Style"),
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
                Focus::CardDescription,
                Focus::CardDueDate,
                Focus::CardPriority,
                Focus::CardStatus,
                Focus::CardTags,
                Focus::CardComments,
                Focus::SubmitButton,
            ],
            PopupMode::CommandPalette => vec![],
            PopupMode::EditSpecificKeyBinding => vec![],
            PopupMode::ChangeUIMode => vec![],
            PopupMode::CardStatusSelector => vec![],
            PopupMode::EditGeneralConfig => vec![],
            PopupMode::SelectDefaultView => vec![],
            PopupMode::ChangeTheme => vec![],
            PopupMode::ThemeEditor => vec![
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
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub app_status: AppStatus,
    pub current_board_id: Option<u128>,
    pub current_card_id: Option<u128>,
    pub focus: Focus,
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
    pub preview_file_name: Option<String>,
    pub popup_mode: Option<PopupMode>,
    pub ui_mode: UiMode,
    pub no_of_cards_to_show: u16,
    pub command_palette_list_state: ListState,
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
    pub all_available_tags: Option<Vec<String>>,
    pub filter_tags: Option<Vec<String>>,
    pub filter_by_tag_list_state: ListState,
}

impl Default for AppState {
    fn default() -> AppState {
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
            preview_file_name: None,
            popup_mode: None,
            ui_mode: get_default_ui_mode(),
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            command_palette_list_state: ListState::default(),
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
    pub enable_mouse_support: bool,
    pub default_theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_view = UiMode::TitleBodyHelpLog;
        let default_theme = Theme::default();
        Self {
            save_directory: get_default_save_directory(),
            default_view,
            always_load_last_save: true,
            save_on_exit: true,
            disable_scrollbars: false,
            warning_delta: DEFAULT_CARD_WARNING_DUE_DATE_DAYS,
            keybindings: KeyBindings::default(),
            tickrate: DEFAULT_TICKRATE,
            no_of_cards_to_show: NO_OF_CARDS_PER_BOARD,
            no_of_boards_to_show: NO_OF_BOARDS_PER_PAGE,
            enable_mouse_support: true,
            default_theme: default_theme.name,
        }
    }
}

impl AppConfig {
    pub fn to_list(&self) -> Vec<Vec<String>> {
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
                String::from("Disable Scrollbars"),
                self.disable_scrollbars.to_string(),
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
                    // check if the new path is valid
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
                "Disable Scrollbars" => {
                    if value.to_lowercase() == "true" {
                        config.disable_scrollbars = true;
                    } else if value.to_lowercase() == "false" {
                        config.disable_scrollbars = false;
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
                        // make sure tickrate is not too low or too high
                        if new_tickrate < 10 {
                            error!(
                                "Tickrate must be greater than 10ms, to avoid overloading the CPU"
                            );
                            app.send_error_toast(
                                "Tickrate must be greater than 50ms, to avoid overloading the CPU",
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
        // make sure key is not empty, or already assigned

        let get_config_status = get_config(false);
        let config = if let Ok(config) = get_config_status {
            config
        } else {
            debug!("Error getting config: {}", get_config_status.unwrap_err());
            AppConfig::default()
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
            "change_card_status_to_completed" => {
                self.keybindings.change_card_status_to_completed = value
            }
            "change_card_status_to_active" => self.keybindings.change_card_status_to_active = value,
            "change_card_status_to_stale" => self.keybindings.change_card_status_to_stale = value,
            "reset_ui" => self.keybindings.reset_ui = value,
            "go_to_main_menu" => self.keybindings.go_to_main_menu = value,
            "toggle_command_palette" => self.keybindings.toggle_command_palette = value,
            "clear_all_toasts" => self.keybindings.clear_all_toasts = value,
            _ => {
                debug!("Invalid key: {}", key);
                error!("Unable to edit keybinding");
                return Err("Something went wrong 😢 ".to_string());
            }
        }
        Ok(())
    }
}

pub fn get_term_bg_color() -> (u8, u8, u8) {
    // TODO: make this work on windows and unix
    (0, 0, 0)
}
