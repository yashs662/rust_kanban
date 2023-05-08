use log::{debug, error, info};
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
};
use tokio::{sync::MutexGuard, time::Instant};

use crate::{
    app::{
        state::{AppStatus, Focus, UiMode},
        App, AppReturn, PopupMode,
    },
    constants::{TOAST_FADE_IN_TIME, TOAST_FADE_OUT_TIME},
    io::{data_handler::export_kanban_to_json, handler::refresh_visible_boards_and_cards, IoEvent},
    lerp_between,
};

use super::{TextColorOptions, Theme};

#[derive(Clone, Debug, PartialEq)]
pub struct ToastWidget {
    pub title: String,
    pub message: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub toast_type: ToastType,
    pub toast_color: (u8, u8, u8),
}

impl ToastWidget {
    pub fn new(message: String, duration: Duration, toast_type: ToastType, theme: Theme) -> Self {
        Self {
            title: toast_type.as_str().to_string(),
            message,
            duration,
            start_time: Instant::now(),
            toast_type: toast_type.clone(),
            toast_color: toast_type.as_color(theme),
        }
    }
    pub fn new_with_title(
        title: String,
        message: String,
        duration: Duration,
        toast_type: ToastType,
        theme: Theme,
    ) -> Self {
        Self {
            title,
            message,
            duration,
            start_time: Instant::now(),
            toast_type: toast_type.clone(),
            toast_color: toast_type.as_color(theme),
        }
    }

    fn update(mut app: MutexGuard<App>) {
        let theme = app.theme.clone();
        let term_background_color = if app.theme.general_style.bg.is_some() {
            TextColorOptions::from(app.theme.general_style.bg.unwrap()).to_rgb()
        } else {
            app.state.term_background_color
        };
        let toasts = &mut app.state.toasts;
        // remove all inactive toasts
        for i in (0..toasts.len()).rev() {
            // based on the toast_type lerp between the toast_type color and 0,0,0 within the TOAST_FADE_TIME which is in milliseconds
            if toasts[i].start_time.elapsed() < Duration::from_millis(TOAST_FADE_IN_TIME) {
                // make the toast fade in use fade in time lerp from 0,0,0 to toast_type color
                let t =
                    toasts[i].start_time.elapsed().as_millis() as f32 / TOAST_FADE_IN_TIME as f32;
                toasts[i].toast_color = lerp_between(
                    term_background_color,
                    toasts[i].toast_type.as_color(theme.clone()),
                    t,
                );
            } else if toasts[i].start_time.elapsed()
                < toasts[i].duration - Duration::from_millis(TOAST_FADE_OUT_TIME)
            {
                // make the toast stay at the toast_type color
                toasts[i].toast_color = toasts[i].toast_type.as_color(theme.clone());
            } else {
                // make the toast fade out use fade out time lerp from toast_type color to 0,0,0
                let t = (toasts[i].start_time.elapsed()
                    - (toasts[i].duration - Duration::from_millis(TOAST_FADE_OUT_TIME)))
                .as_millis() as f32
                    / TOAST_FADE_OUT_TIME as f32;
                toasts[i].toast_color = lerp_between(
                    toasts[i].toast_type.as_color(theme.clone()),
                    term_background_color,
                    t,
                );
            }
            if toasts[i].start_time.elapsed() > toasts[i].duration {
                toasts.remove(i);
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Error,
    Warning,
    Info,
    Loading,
}

impl ToastType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Loading => "Loading",
        }
    }
    pub fn as_color(&self, theme: Theme) -> (u8, u8, u8) {
        match self {
            Self::Error => TextColorOptions::from(
                theme
                    .log_error_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightRed),
            )
            .to_rgb(),
            Self::Warning => TextColorOptions::from(
                theme
                    .log_warn_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightYellow),
            )
            .to_rgb(),
            Self::Info => TextColorOptions::from(
                theme
                    .log_info_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightCyan),
            )
            .to_rgb(),
            Self::Loading => TextColorOptions::from(
                theme
                    .log_debug_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightGreen),
            )
            .to_rgb(),
        }
    }
}

pub struct WidgetManager {
    pub app: Arc<tokio::sync::Mutex<App>>,
}

impl WidgetManager {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn update(&mut self) {
        ToastWidget::update(self.app.lock().await);
        CommandPaletteWidget::update(self.app.lock().await);
    }
}

#[derive(Debug)]
pub struct CommandPaletteWidget {
    pub search_results: Option<Vec<CommandPaletteActions>>,
    pub last_search_string: String,
    pub available_commands: Vec<CommandPaletteActions>,
    pub corpus: Corpus,
}

impl Default for CommandPaletteWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteWidget {
    pub fn new() -> Self {
        let available_commands = CommandPaletteActions::all();
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();
        for command in available_commands {
            corpus.add_text(command.to_string().to_lowercase().as_str());
        }
        Self {
            search_results: None,
            last_search_string: "iibnigivirneiivure".to_string(), // random string that will never be typed as the selected index jumps around when a mouse is used with an empty search string
            available_commands: CommandPaletteActions::all(),
            corpus,
        }
    }

    pub async fn handle_command(app: &mut App) -> AppReturn {
        if app.state.command_palette_list_state.selected().is_some() {
            let command_index = app.state.command_palette_list_state.selected().unwrap();
            let command = if app.command_palette.search_results.is_some() {
                app.command_palette
                    .search_results
                    .as_ref()
                    .unwrap()
                    .get(command_index)
            } else {
                None
            };
            if command.is_some() {
                match command.unwrap() {
                    CommandPaletteActions::ExportToJSON => {
                        let export_result = export_kanban_to_json(&app.boards);
                        if export_result.is_ok() {
                            let msg = format!("Exported JSON to {}", export_result.unwrap());
                            app.send_info_toast(&msg, None);
                            info!("{}", msg);
                        } else {
                            let msg =
                                format!("Failed to export JSON: {}", export_result.unwrap_err());
                            app.send_error_toast(&msg, None);
                            error!("{}", msg);
                        }
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::Quit => {
                        info!("Quitting");
                        return AppReturn::Exit;
                    }
                    CommandPaletteActions::OpenConfigMenu => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::ConfigMenu;
                        app.state.config_state.select(Some(0));
                        app.state.focus = Focus::ConfigTable;
                    }
                    CommandPaletteActions::OpenMainMenu => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::MainMenu;
                        app.state.main_menu_state.select(Some(0));
                        app.state.focus = Focus::MainMenu;
                    }
                    CommandPaletteActions::OpenHelpMenu => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.help_state.select(Some(0));
                        app.state.focus = Focus::Body;
                    }
                    CommandPaletteActions::SaveKanbanState => {
                        app.state.popup_mode = None;
                        app.dispatch(IoEvent::SaveLocalData).await;
                    }
                    CommandPaletteActions::NewBoard => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            app.state.popup_mode = None;
                            app.state.prev_ui_mode = Some(app.state.ui_mode);
                            app.state.ui_mode = UiMode::NewBoard;
                            app.state.focus = Focus::NewBoardName;
                        } else {
                            app.state.popup_mode = None;
                            app.send_error_toast("Cannot create a new board in this view", None);
                        }
                    }
                    CommandPaletteActions::NewCard => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            if app.state.current_board_id.is_none() {
                                app.send_error_toast("No board Selected / Available", None);
                                app.state.popup_mode = None;
                                app.state.app_status = AppStatus::Initialized;
                                return AppReturn::Continue;
                            }
                            app.state.popup_mode = None;
                            app.state.prev_ui_mode = Some(app.state.ui_mode);
                            app.state.ui_mode = UiMode::NewCard;
                            app.state.focus = Focus::NewCardName;
                        } else {
                            app.state.popup_mode = None;
                            app.send_error_toast("Cannot create a new card in this view", None);
                        }
                    }
                    CommandPaletteActions::ResetUI => {
                        app.state.popup_mode = None;
                        app.state.ui_mode = app.config.default_view;
                        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
                    }
                    CommandPaletteActions::ChangeUIMode => {
                        app.state.popup_mode = Some(PopupMode::ChangeUIMode);
                    }
                    CommandPaletteActions::ChangeCurrentCardStatus => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            if let Some(current_board_id) = app.state.current_board_id {
                                if let Some(current_board) =
                                    app.boards.iter_mut().find(|b| b.id == current_board_id)
                                {
                                    if let Some(current_card_id) = app.state.current_card_id {
                                        if current_board
                                            .cards
                                            .iter_mut()
                                            .any(|c| c.id == current_card_id)
                                        {
                                            app.state.popup_mode =
                                                Some(PopupMode::CardStatusSelector);
                                            app.state.app_status = AppStatus::Initialized;
                                            app.state.card_status_selector_state.select(Some(0));
                                            return AppReturn::Continue;
                                        }
                                    }
                                }
                            }
                            app.send_error_toast("Could not find current card", None);
                        } else {
                            app.state.popup_mode = None;
                            app.send_error_toast("Cannot change card status in this view", None);
                        }
                    }
                    CommandPaletteActions::LoadASave => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::LoadSave;
                    }
                    CommandPaletteActions::DebugMenu => {
                        app.state.debug_menu_toggled = !app.state.debug_menu_toggled;
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::ChangeTheme => {
                        app.state.popup_mode = Some(PopupMode::ChangeTheme);
                    }
                    CommandPaletteActions::CreateATheme => {
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::CreateTheme;
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::FilterByTag => {
                        let tags = Self::calculate_tags(app);
                        if tags.is_empty() {
                            app.send_warning_toast("No tags found to filter with", None);
                            app.state.popup_mode = None;
                        } else {
                            app.state.popup_mode = Some(PopupMode::FilterByTag);
                            app.state.all_available_tags = Some(tags);
                        }
                    }
                    CommandPaletteActions::ClearFilter => {
                        app.state.filter_tags = None;
                        app.state.all_available_tags = None;
                        app.state.filter_by_tag_list_state.select(None);
                        app.state.popup_mode = None;
                        app.filtered_boards = vec![];
                        refresh_visible_boards_and_cards(app);
                        app.send_info_toast("All Filters Cleared", None);
                    }
                }
                app.state.current_user_input = "".to_string();
            } else {
                debug!("No command found for the command palette");
            }
        } else {
            debug!("Tried to handle command but no item was selected");
        }
        app.state.app_status = AppStatus::Initialized;
        app.state.current_user_input = String::new();
        app.state.current_cursor_position = None;
        AppReturn::Continue
    }

    fn update(mut app: MutexGuard<App>) {
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            // check if last search string is different from app,.state.current_user_input
            if app.state.current_user_input.to_lowercase() == app.command_palette.last_search_string
            {
                return;
            }
            let current_search_string = app.state.current_user_input.clone().to_lowercase();
            let result = app
                .command_palette
                .corpus
                .search(&current_search_string, 0.2);
            let mut search_results = vec![];
            for item in result {
                search_results.push(CommandPaletteActions::from_string(&item.text, true));
            }
            let search_results: Vec<CommandPaletteActions> =
                search_results.into_iter().flatten().collect();
            // if the search results are empty, then show all commands
            let search_results = if search_results.is_empty() {
                CommandPaletteActions::all()
            } else {
                // sort to keep search results that start with the current search string at the top of the list
                let mut ordered_search_results = vec![];
                let mut extra_results = vec![];
                for result in search_results {
                    if result
                        .to_string()
                        .to_lowercase()
                        .starts_with(&current_search_string)
                    {
                        ordered_search_results.push(result);
                    } else {
                        extra_results.push(result);
                    }
                }
                ordered_search_results.extend(extra_results);
                ordered_search_results
            };
            app.command_palette.search_results = Some(search_results);
            app.command_palette.last_search_string = current_search_string;
            if app.command_palette.search_results.is_some() {
                // if length is > 0 select first item
                if !app
                    .command_palette
                    .search_results
                    .as_ref()
                    .unwrap()
                    .is_empty()
                {
                    app.state.command_palette_list_state.select(Some(0));
                }
            }
        }
    }

    pub fn calculate_tags(app: &App) -> Vec<String> {
        let mut tags: Vec<String> = vec![];
        for board in &app.boards {
            for card in &board.cards {
                for tag in &card.tags {
                    if tag.is_empty() {
                        continue;
                    }
                    tags.push(tag.clone());
                }
            }
        }
        tags = tags.iter().map(|tag| tag.to_lowercase()).collect();
        let count_hash: HashMap<String, u32> = tags.iter().fold(HashMap::new(), |mut acc, tag| {
            *acc.entry(tag.clone()).or_insert(0) += 1;
            acc
        });
        count_hash
            .iter()
            .map(|(tag, count)| format!("{} - {} occurrences", tag, count))
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandPaletteActions {
    ExportToJSON,
    OpenConfigMenu,
    SaveKanbanState,
    LoadASave,
    NewBoard,
    NewCard,
    ResetUI,
    OpenMainMenu,
    OpenHelpMenu,
    ChangeUIMode,
    ChangeCurrentCardStatus,
    DebugMenu,
    ChangeTheme,
    CreateATheme,
    FilterByTag,
    ClearFilter,
    Quit,
}

impl Display for CommandPaletteActions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExportToJSON => write!(f, "Export to JSON"),
            Self::OpenConfigMenu => write!(f, "Open Config Menu"),
            Self::SaveKanbanState => write!(f, "Save Kanban State"),
            Self::LoadASave => write!(f, "Load a Save"),
            Self::NewBoard => write!(f, "New Board"),
            Self::NewCard => write!(f, "New Card"),
            Self::ResetUI => write!(f, "Reset UI"),
            Self::OpenMainMenu => write!(f, "Open Main Menu"),
            Self::OpenHelpMenu => write!(f, "Open Help Menu"),
            Self::ChangeUIMode => write!(f, "Change UI Mode"),
            Self::ChangeCurrentCardStatus => write!(f, "Change Current Card Status"),
            Self::DebugMenu => write!(f, "Toggle Debug Panel"),
            Self::ChangeTheme => write!(f, "Change Theme"),
            Self::CreateATheme => write!(f, "Create a Theme"),
            Self::FilterByTag => write!(f, "Filter by Tag"),
            Self::ClearFilter => write!(f, "Clear Filter"),
            Self::Quit => write!(f, "Quit"),
        }
    }
}

impl CommandPaletteActions {
    pub fn all() -> Vec<Self> {
        let all = vec![
            Self::ExportToJSON,
            Self::OpenConfigMenu,
            Self::SaveKanbanState,
            Self::LoadASave,
            Self::NewBoard,
            Self::NewCard,
            Self::ResetUI,
            Self::OpenMainMenu,
            Self::OpenHelpMenu,
            Self::ChangeUIMode,
            Self::ChangeCurrentCardStatus,
            Self::ChangeTheme,
            Self::CreateATheme,
            Self::FilterByTag,
            Self::ClearFilter,
            Self::Quit,
        ];

        if cfg!(debug_assertions) {
            let mut all = all;
            all.push(Self::DebugMenu);
            all
        } else {
            all
        }
    }

    pub fn from_string(s: &str, lowercase_match: bool) -> Option<Self> {
        if lowercase_match {
            match s.to_lowercase().as_str() {
                "export to json" => Some(Self::ExportToJSON),
                "open config menu" => Some(Self::OpenConfigMenu),
                "save kanban state" => Some(Self::SaveKanbanState),
                "load a save" => Some(Self::LoadASave),
                "new board" => Some(Self::NewBoard),
                "new card" => Some(Self::NewCard),
                "reset ui" => Some(Self::ResetUI),
                "open main menu" => Some(Self::OpenMainMenu),
                "open help menu" => Some(Self::OpenHelpMenu),
                "change ui mode" => Some(Self::ChangeUIMode),
                "change current card status" => Some(Self::ChangeCurrentCardStatus),
                "toggle debug panel" => Some(Self::DebugMenu),
                "change theme" => Some(Self::ChangeTheme),
                "create a theme" => Some(Self::CreateATheme),
                "filter by tag" => Some(Self::FilterByTag),
                "clear filter" => Some(Self::ClearFilter),
                "quit" => Some(Self::Quit),
                _ => None,
            }
        } else {
            match s {
                "Export to JSON" => Some(Self::ExportToJSON),
                "Open Config Menu" => Some(Self::OpenConfigMenu),
                "Save Kanban State" => Some(Self::SaveKanbanState),
                "Load a Save" => Some(Self::LoadASave),
                "New Board" => Some(Self::NewBoard),
                "New Card" => Some(Self::NewCard),
                "Reset UI" => Some(Self::ResetUI),
                "Open Main Menu" => Some(Self::OpenMainMenu),
                "Open Help Menu" => Some(Self::OpenHelpMenu),
                "Change UI Mode" => Some(Self::ChangeUIMode),
                "Change Current Card Status" => Some(Self::ChangeCurrentCardStatus),
                "Toggle Debug Panel" => Some(Self::DebugMenu),
                "Change Theme" => Some(Self::ChangeTheme),
                "Create a Theme" => Some(Self::CreateATheme),
                "Filter by Tag" => Some(Self::FilterByTag),
                "Clear Filter" => Some(Self::ClearFilter),
                "Quit" => Some(Self::Quit),
                _ => None,
            }
        }
    }
}
