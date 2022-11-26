use std::fmt::{self, Formatter, Display};
use std::path::PathBuf;

use log::{debug, info};
use log::{error, warn};
use serde::{Serialize, Deserialize};
use tui::widgets::ListState;

use self::actions::Actions;
use self::state::AppState;
use self::state::Focus;
use self::state::UiMode;
use self::kanban::Board;
use crate::app::actions::Action;
use crate::constants::DB_NAME;
use crate::inputs::key::Key;
use crate::io::data_handler::write_config;
use crate::io::{IoEvent, handler, data_handler};

pub mod actions;
pub mod state;
pub mod ui;
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
    state: AppState,
    focus: Focus,
    ui_mode: UiMode,
    boards: Vec<kanban::Board>,
    current_user_input: String,
    prev_ui_mode: UiMode,
    pub config_state: ListState,
    pub main_menu: MainMenu,
    config: AppConfig,
    config_item_being_edited: Option<usize>,
    current_board: String,
    current_card: String
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();
        let focus = Focus::Title;
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
            config_state: ListState::default(),
            main_menu: MainMenu::default(),
            config: AppConfig::default(),
            config_item_being_edited: None,
            current_board: "None".to_string(),
            current_card: "None".to_string()
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        // check if we are in a user input mode
        if self.state == AppState::UserInput {
            // append to current user input if key is not enter else change state to Initialized
            if key != Key::Enter {
                let mut current_key = key.to_string();
                if current_key == "<Space>" {
                    current_key = " ".to_string();
                } else if current_key == "<ShiftEnter>" {
                    current_key = "\n".to_string();
                } else if current_key == "<Backspace>" {
                    self.current_user_input.pop();
                    return AppReturn::Continue;
                } else if current_key.starts_with("<") && current_key.ends_with(">") {
                    current_key = current_key[1..current_key.len() - 1].to_string();
                } else {
                    // do nothing
                }
                self.current_user_input.push_str(&current_key);
            } else {
                self.state = AppState::Initialized;
                info!("Exiting user input mode");
                debug!("User input: {}", self.current_user_input);
            }
            return AppReturn::Continue;
        } else {
            if let Some(action) = self.actions.find(key) {
                match action {
                    Action::Quit => AppReturn::Exit,
                    // Action::Sleep => {
                    //     if let Some(duration) = self.state.duration().cloned() {
                    //         // Sleep is an I/O action, we dispatch on the IO channel that's run on another thread
                    //         self.dispatch(IoEvent::Sleep(duration)).await
                    //     }
                    //     AppReturn::Continue
                    // }
                    Action::NextFocus => {
                        self.focus = self.focus.next(&UiMode::get_available_tabs(&self.ui_mode));
                        AppReturn::Continue
                    }
                    Action::PreviousFocus => {
                        self.focus = self.focus.prev(&UiMode::get_available_tabs(&self.ui_mode));
                        AppReturn::Continue
                    }
                    Action::SetUiMode => {
                        let new_ui_mode = UiMode::from_number(key.to_digit() as u8);
                        let available_tabs = UiMode::get_available_tabs(&new_ui_mode);
                        // check if focus is still available in the new ui_mode if not set it to the first available tab
                        if !available_tabs.contains(&self.focus.current().to_owned()) {
                            self.focus = Focus::from_str(available_tabs[0].as_str());
                        }
                        debug!("Setting ui_mode to {}", new_ui_mode.to_string());
                        self.ui_mode = new_ui_mode;
                        AppReturn::Continue
                    }
                    Action::ToggleConfig => {
                        if self.ui_mode == UiMode::Config {
                            self.ui_mode = self.prev_ui_mode.clone();
                        } else {
                            self.prev_ui_mode = self.ui_mode.clone();
                            self.ui_mode = UiMode::Config;
                        }
                        AppReturn::Continue
                    }
                    Action::GoUp => {
                        if self.ui_mode == UiMode::Config {
                            self.config_previous();
                        }
                        else if self.ui_mode == UiMode::MainMenu {
                            self.main_menu_previous();
                        }
                        AppReturn::Continue
                    }
                    Action::GoDown => {
                        if self.ui_mode == UiMode::Config {
                            self.config_next();
                        }
                        else if self.ui_mode == UiMode::MainMenu {
                            self.main_menu_next();
                        }
                        AppReturn::Continue
                    }
                    Action::TakeUserInput => {
                        self.state = AppState::UserInput;
                        info!("Taking user input");
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
                            _ => {
                                self.ui_mode = UiMode::MainMenu;
                                AppReturn::Continue
                            }
                        }
                    }
                    Action::Enter => {
                        match self.ui_mode {
                            UiMode::Config => {
                                self.prev_ui_mode = self.ui_mode.clone();
                                self.ui_mode = UiMode::EditConfig;
                                debug!("Setting ui_mode to {}", self.ui_mode.to_string());
                                self.config_item_being_edited = Some(self.config_state.selected().unwrap_or(0));
                                AppReturn::Continue
                            }
                            UiMode::EditConfig => {
                                let config_item_index = self.config_state.selected().unwrap_or(0);
                                let config_item_list = AppConfig::to_list(&self.config);
                                let config_item = &config_item_list[config_item_index];
                                // split the config item on : and get the first part
                                let config_item_key = config_item.split(":").collect::<Vec<&str>>()[0];
                                let new_value = self.current_user_input.clone();
                                // if new value is not empty update the config
                                if !new_value.is_empty() {
                                    let config_string = format!("{}: {}", config_item_key, new_value);
                                    debug!("Setting config to {}", config_string);
                                    let app_config = AppConfig::edit_with_string(&config_string, self);
                                    self.config = app_config.clone();
                                    write_config(&app_config);

                                    // reset everything
                                    self.config_state = ListState::default();
                                    self.config_item_being_edited = None;
                                    self.current_user_input = String::new();
                                    self.ui_mode = UiMode::Config;
                                }
                                AppReturn::Continue
                            }
                            UiMode::MainMenu => {
                                let selected = self.main_menu.state.selected().unwrap_or(0);
                                let selected_item = &self.main_menu.items[selected];
                                match selected_item {
                                    MainMenuItems::Quit => {
                                        AppReturn::Exit
                                    }
                                    MainMenuItems::Config => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = UiMode::Config;
                                        debug!("Setting ui_mode to {}", self.ui_mode.to_string());
                                        AppReturn::Continue
                                    }
                                    MainMenuItems::View => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = self.config.default_view.clone();
                                        debug!("Setting ui_mode to {}", self.ui_mode.to_string());
                                        AppReturn::Continue
                                    }
                                    MainMenuItems::Help => {
                                        self.prev_ui_mode = self.ui_mode.clone();
                                        self.ui_mode = UiMode::HelpMenu;
                                        debug!("Setting ui_mode to {}", self.ui_mode.to_string());
                                        AppReturn::Continue
                                    }
                                }
                            }
                            _ => {
                                AppReturn::Continue
                            }
                        }
                    }
                    Action::Hide => {
                        let current_focus = Focus::from_str(self.focus.current());
                        let current_ui_mode = self.ui_mode.clone();
                        // hide the current focus by switching to a view where it is not available
                        // for example if current uimode is Title and focus is on Title then switch to Zen
                        if current_ui_mode == UiMode::Zen {
                            self.ui_mode = UiMode::MainMenu;
                        } else if current_ui_mode == UiMode::TitleBody {
                            if current_focus == Focus::Title {
                                self.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.ui_mode = UiMode::MainMenu;
                            }
                        } else if current_ui_mode == UiMode::BodyHelp {
                            if current_focus == Focus::Help {
                                self.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.ui_mode = UiMode::MainMenu;
                            }
                        } else if current_ui_mode == UiMode::BodyLog {
                            if current_focus == Focus::Log {
                                self.ui_mode = UiMode::Zen;
                                self.focus = Focus::Body;
                            } else {
                                self.ui_mode = UiMode::MainMenu;
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
                            }
                        }
                        AppReturn::Continue
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
    pub fn state(&self) -> &AppState {
        &self.state
    }
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }
    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = Action::all()
        .into();
        self.state = AppState::initialized()
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
    pub fn set_current_user_input(&mut self, input: String) {
        let new_input = input;
        debug!("Setting current user input to {}", new_input);
        self.current_user_input = new_input;
    }
    pub fn clear_current_user_input(&mut self) {
        self.current_user_input = String::new();
    }
    pub fn set_config_state(&mut self, config_state: ListState) {
        self.config_state = config_state;
    }
    pub fn config_next(&mut self) {
        let i = match self.config_state.selected() {
            Some(i) => {
                if i >= self.config.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.config_state.select(Some(i));
    }
    pub fn config_previous(&mut self) {
        let i = match self.config_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.config.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.config_state.select(Some(i));
    }
    pub fn main_menu_next(&mut self) {
        let i = match self.main_menu.state.selected() {
            Some(i) => {
                if i >= self.main_menu.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.main_menu.state.select(Some(i));
    }
    pub fn main_menu_previous(&mut self) {
        let i = match self.main_menu.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.main_menu.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.main_menu.state.select(Some(i));
    }
    pub fn config_state(&self) -> &ListState {
        &self.config_state
    }
    pub fn set_ui_mode(&mut self, ui_mode: UiMode) {
        self.ui_mode = ui_mode;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub db_path: PathBuf,
    pub default_view: UiMode
}

impl AppConfig {
    pub fn default() -> Self {
        let db_path = handler::get_config_dir().join(DB_NAME);
        let default_view = UiMode::TitleBodyHelpLog;
        Self {
            db_path,
            default_view
        }
    }

    pub fn to_list(&self) -> Vec<String> {
        vec![
            format!("db_path: {}", self.db_path.to_str().unwrap()),
            format!("default_view: {}", self.default_view.to_string()),
        ]
    }

    pub fn edit_with_string(change_str: &str, app: &App) -> Self {
        let mut config = app.config.clone();
        let mut lines = change_str.lines();
        while let Some(line) = lines.next() {
            let mut parts = line.split(":");
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            debug!("Editing config with key: {} and value: {}", key, value);
            match key {
                "db_path" => {
                    let new_path = PathBuf::from(value);
                    // check if the new path is valid
                    if new_path.exists() {
                        config.db_path = new_path;
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
                _ => {
                    return config;
                }
            }
        }
        debug!("Config: {:?}", config);
        config
    }

    pub fn len(&self) -> usize {
        self.to_list().len()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MainMenuItems {
    View,
    Config,
    Help,
    Quit
}

impl Display for MainMenuItems {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MainMenuItems::View => write!(f, "View your Boards"),
            MainMenuItems::Config => write!(f, "Configure"),
            MainMenuItems::Help => write!(f, "Help"),
            MainMenuItems::Quit => write!(f, "Quit"),
        }
    }
}

pub struct MainMenu {
    pub state: ListState,
    pub items: Vec<MainMenuItems>
}

impl MainMenu {
    fn default() -> Self {
        let items = vec![
            MainMenuItems::View,
            MainMenuItems::Config,
            MainMenuItems::Help,
            MainMenuItems::Quit
        ];
        Self {
            state: ListState::default(),
            items: items
        }
    }
}