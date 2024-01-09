use super::{TextColorOptions, Theme};
use crate::{
    app::{
        app_helper::reset_preview_boards,
        handle_exit,
        state::{AppStatus, Focus, UiMode},
        App, AppReturn, PopupMode,
    },
    constants::{RANDOM_SEARCH_TERM, TOAST_FADE_IN_TIME, TOAST_FADE_OUT_TIME},
    io::{io_handler::refresh_visible_boards_and_cards, IoEvent},
    util::lerp_between,
};
use log::{debug, error, info};
use ngrammatic::{Corpus, CorpusBuilder, Pad};
use ratatui::style::{Color, Style};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
};
use tokio::time::Instant;

#[derive(Clone, Debug, PartialEq)]
pub struct ToastWidget {
    pub duration: Duration,
    pub message: String,
    pub start_time: Instant,
    pub title: String,
    pub toast_color: (u8, u8, u8),
    pub toast_type: ToastType,
}

impl ToastWidget {
    pub fn new(message: String, duration: Duration, toast_type: ToastType, theme: Theme) -> Self {
        Self {
            duration,
            message,
            start_time: Instant::now(),
            title: toast_type.as_string(),
            toast_color: toast_type.as_color(theme),
            toast_type: toast_type.clone(),
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
            duration,
            message,
            start_time: Instant::now(),
            title,
            toast_color: toast_type.as_color(theme),
            toast_type: toast_type.clone(),
        }
    }

    fn update(app: &mut App) {
        let theme = app.current_theme.clone();
        let term_background_color = if app.current_theme.general_style.bg.is_some() {
            TextColorOptions::from(app.current_theme.general_style.bg.unwrap()).to_rgb()
        } else {
            app.state.term_background_color
        };
        let disable_animations = app.config.disable_animations;
        let toasts = &mut app.widgets.toasts;
        for i in (0..toasts.len()).rev() {
            if toasts[i].start_time.elapsed() > toasts[i].duration {
                toasts.remove(i);
                continue;
            }
            if disable_animations {
                toasts[i].toast_color = toasts[i].toast_type.as_color(theme.clone());
                continue;
            }
            if toasts[i].start_time.elapsed() < Duration::from_millis(TOAST_FADE_IN_TIME) {
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
                toasts[i].toast_color = toasts[i].toast_type.as_color(theme.clone());
            } else {
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Error,
    Info,
    Loading,
    Warning,
}

impl ToastType {
    pub fn as_string(&self) -> String {
        match self {
            Self::Error => "Error".to_string(),
            Self::Info => "Info".to_string(),
            Self::Loading => "Loading".to_string(),
            Self::Warning => "Warning".to_string(),
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

pub struct WidgetManager<'a> {
    pub app: Arc<tokio::sync::Mutex<App<'a>>>,
}

impl WidgetManager<'_> {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> WidgetManager {
        WidgetManager { app }
    }

    pub async fn update(&mut self) {
        let mut app = self.app.lock().await;
        ToastWidget::update(&mut app);
        CommandPaletteWidget::update(&mut app);
        CloseButtonWidget::update(&mut app);
    }
}

#[derive(Debug)]
pub struct CloseButtonWidget {
    start_time: Instant,
    fade_time: f32,
    pub color: (u8, u8, u8),
    offset: f32,
}

impl CloseButtonWidget {
    pub fn new(style: Style) -> Self {
        let color = style.fg.unwrap_or(Color::White);
        let text_color = TextColorOptions::from(color).to_rgb();
        Self {
            start_time: Instant::now(),
            fade_time: 1.0,
            color: text_color,
            offset: 0.8,
        }
    }

    pub fn update(app: &mut App) {
        if app.state.focus == Focus::CloseButton {
            let theme = app.current_theme.clone();
            let disable_animations = app.config.disable_animations;
            let widget = &mut app.widgets.close_button_widget;
            if disable_animations {
                widget.color = TextColorOptions::from(theme.error_text_style.bg.unwrap()).to_rgb();
                return;
            }

            let normal_color = TextColorOptions::from(theme.general_style.bg.unwrap()).to_rgb();
            let hover_color = TextColorOptions::from(theme.error_text_style.bg.unwrap()).to_rgb();
            let total_duration = Duration::from_millis((widget.fade_time * 1000.0) as u64);
            let half_duration = Duration::from_millis((widget.fade_time * 500.0) as u64);

            if widget.start_time.elapsed() > total_duration {
                widget.start_time = Instant::now();
            }

            let mut t = (widget.start_time.elapsed().as_millis() as f32
                / total_duration.as_millis() as f32)
                + widget.offset; // offset to make it overall brighter

            if widget.start_time.elapsed() < half_duration {
                widget.color = lerp_between(normal_color, hover_color, t);
            } else {
                t = t - widget.fade_time - (widget.offset / 4.0); // offset to make it overall brighter
                widget.color = lerp_between(hover_color, normal_color, t);
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandPaletteWidget {
    pub already_in_user_input_mode: bool,
    pub available_commands: Vec<CommandPaletteActions>,
    pub board_search_results: Option<Vec<(String, (u64, u64))>>,
    pub card_search_results: Option<Vec<(String, (u64, u64))>>,
    pub command_palette_actions_corpus: Corpus,
    pub command_search_results: Option<Vec<CommandPaletteActions>>,
    pub last_focus: Option<Focus>,
    pub last_search_string: String,
}

impl CommandPaletteWidget {
    pub fn new(debug_mode: bool) -> Self {
        let available_commands = CommandPaletteActions::all(debug_mode);
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();
        for command in &available_commands {
            corpus.add_text(command.to_string().to_lowercase().as_str());
        }
        Self {
            already_in_user_input_mode: false,
            available_commands,
            board_search_results: None,
            card_search_results: None,
            command_palette_actions_corpus: corpus,
            command_search_results: None,
            last_focus: None,
            last_search_string: RANDOM_SEARCH_TERM.to_string(),
        }
    }

    pub async fn handle_command(app: &mut App<'_>) -> AppReturn {
        if app
            .state
            .app_list_states
            .command_palette_command_search
            .selected()
            .is_some()
        {
            let command_index = app
                .state
                .app_list_states
                .command_palette_command_search
                .selected()
                .unwrap();
            let command = if app.widgets.command_palette.command_search_results.is_some() {
                app.widgets
                    .command_palette
                    .command_search_results
                    .as_ref()
                    .unwrap()
                    .get(command_index)
            } else {
                None
            };
            if command.is_some() {
                match command.unwrap() {
                    CommandPaletteActions::Quit => {
                        info!("Quitting");
                        return handle_exit(app).await;
                    }
                    CommandPaletteActions::ConfigMenu => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::ConfigMenu;
                        app.state.app_table_states.config.select(Some(0));
                        app.state.focus = Focus::ConfigTable;
                    }
                    CommandPaletteActions::MainMenu => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::MainMenu;
                        app.state.app_list_states.main_menu.select(Some(0));
                        app.state.focus = Focus::MainMenu;
                    }
                    CommandPaletteActions::HelpMenu => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.app_table_states.help.select(Some(0));
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
                            app.state.focus = Focus::CardName;
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
                                            app.state
                                                .app_list_states
                                                .card_status_selector
                                                .select(Some(0));
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
                    CommandPaletteActions::LoadASaveLocal => {
                        app.state.popup_mode = None;
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        reset_preview_boards(app);
                        app.state.ui_mode = UiMode::LoadLocalSave;
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
                        if app.filtered_boards.is_empty() {
                            app.send_warning_toast("No filters to clear", None);
                            app.state.popup_mode = None;
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        } else {
                            app.send_info_toast("All Filters Cleared", None);
                        }
                        app.state.filter_tags = None;
                        app.state.all_available_tags = None;
                        app.state.app_list_states.filter_by_tag_list.select(None);
                        app.state.popup_mode = None;
                        app.filtered_boards = vec![];
                        refresh_visible_boards_and_cards(app);
                    }
                    CommandPaletteActions::ChangeDateFormat => {
                        app.state.popup_mode = Some(PopupMode::ChangeDateFormatPopup);
                    }
                    CommandPaletteActions::NoCommandsFound => {
                        app.state.popup_mode = None;
                        app.state.app_status = AppStatus::Initialized;
                        return AppReturn::Continue;
                    }
                    CommandPaletteActions::Login => {
                        if app.state.user_login_data.auth_token.is_some() {
                            app.send_error_toast("Already logged in", None);
                            app.state.popup_mode = None;
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.ui_mode = UiMode::Login;
                        app.state.popup_mode = None;
                        app.state.focus = Focus::EmailIDField;
                    }
                    CommandPaletteActions::Logout => {
                        app.dispatch(IoEvent::Logout).await;
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::SignUp => {
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.focus = Focus::EmailIDField;
                        app.state.ui_mode = UiMode::SignUp;
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::ResetPassword => {
                        app.state.prev_ui_mode = Some(app.state.ui_mode);
                        app.state.focus = Focus::EmailIDField;
                        app.state.ui_mode = UiMode::ResetPassword;
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::SyncLocalData => {
                        app.dispatch(IoEvent::SyncLocalData).await;
                        app.state.popup_mode = None;
                    }
                    CommandPaletteActions::LoadASaveCloud => {
                        if app.state.user_login_data.auth_token.is_some() {
                            app.state.prev_ui_mode = Some(app.state.ui_mode);
                            app.state.ui_mode = UiMode::LoadCloudSave;
                            reset_preview_boards(app);
                            app.dispatch(IoEvent::GetCloudData).await;
                            app.state.popup_mode = None;
                        } else {
                            error!("Not logged in");
                            app.send_error_toast("Not logged in", None);
                            app.state.popup_mode = None;
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                    }
                }
                app.state.current_user_input = "".to_string();
            } else {
                debug!("No command found for the command palette");
            }
        } else {
            return AppReturn::Continue;
        }
        if app.widgets.command_palette.already_in_user_input_mode {
            app.widgets.command_palette.already_in_user_input_mode = false;
            app.widgets.command_palette.last_focus = None;
        }
        app.state.app_status = AppStatus::Initialized;
        app.state.current_user_input = String::new();
        app.state.current_cursor_position = None;
        AppReturn::Continue
    }

    fn update(app: &mut App) {
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            if app.state.current_user_input.to_lowercase()
                == app.widgets.command_palette.last_search_string
            {
                return;
            }
            let current_search_string = app.state.current_user_input.clone().to_lowercase();
            let result = app
                .widgets
                .command_palette
                .command_palette_actions_corpus
                .search(&current_search_string, 0.2);
            let mut search_results = vec![];
            for item in result {
                search_results.push(CommandPaletteActions::from_string(&item.text, true).unwrap());
            }
            let mut command_search_results = if search_results.is_empty() {
                if current_search_string.is_empty() {
                    CommandPaletteActions::all(app.debug_mode)
                } else {
                    let all_actions = CommandPaletteActions::all(app.debug_mode);
                    let mut results = vec![];
                    for action in all_actions {
                        if action
                            .to_string()
                            .to_lowercase()
                            .starts_with(&current_search_string)
                        {
                            results.push(action);
                        }
                    }
                    results
                }
            } else {
                let mut ordered_command_search_results = vec![];
                let mut extra_command_results = vec![];
                for result in search_results {
                    if result
                        .to_string()
                        .to_lowercase()
                        .starts_with(&current_search_string)
                    {
                        ordered_command_search_results.push(result);
                    } else {
                        extra_command_results.push(result);
                    }
                }
                ordered_command_search_results.extend(extra_command_results);
                ordered_command_search_results
            };
            if command_search_results.is_empty() {
                command_search_results = vec![CommandPaletteActions::NoCommandsFound]
            }

            let mut card_search_results: Vec<(String, (u64, u64))> = vec![];
            if !current_search_string.is_empty() {
                for board in &app.boards {
                    for card in &board.cards {
                        let search_helper =
                            if card.name.to_lowercase().contains(&current_search_string) {
                                format!("{} - Matched in Name", card.name)
                            } else if card
                                .description
                                .to_lowercase()
                                .contains(&current_search_string)
                            {
                                format!("{} - Matched in Description", card.name)
                            } else if card
                                .tags
                                .iter()
                                .any(|tag| tag.to_lowercase().contains(&current_search_string))
                            {
                                format!("{} - Matched in Tags", card.name)
                            } else if card.comments.iter().any(|comment| {
                                comment.to_lowercase().contains(&current_search_string)
                            }) {
                                format!("{} - Matched in Comments", card.name)
                            } else {
                                String::new()
                            };
                        if !search_helper.is_empty() {
                            card_search_results.push((search_helper, card.id));
                        }
                    }
                }
            }
            if !card_search_results.is_empty() {
                app.widgets.command_palette.card_search_results = Some(card_search_results.clone());
            }

            let mut board_search_results: Vec<(String, (u64, u64))> = vec![];
            if !current_search_string.is_empty() {
                for board in &app.boards {
                    let search_helper =
                        if board.name.to_lowercase().contains(&current_search_string) {
                            format!("{} - Matched in Name", board.name)
                        } else if board
                            .description
                            .to_lowercase()
                            .contains(&current_search_string)
                        {
                            format!("{} - Matched in Description", board.name)
                        } else {
                            String::new()
                        };
                    if !search_helper.is_empty() {
                        board_search_results.push((search_helper, board.id));
                    }
                }
            }
            if !board_search_results.is_empty() {
                app.widgets.command_palette.board_search_results =
                    Some(board_search_results.clone());
            }

            app.widgets.command_palette.command_search_results = Some(command_search_results);
            app.widgets.command_palette.last_search_string = current_search_string;
            if app.widgets.command_palette.command_search_results.is_some()
                && !app
                    .widgets
                    .command_palette
                    .command_search_results
                    .as_ref()
                    .unwrap()
                    .is_empty()
            {
                app.state
                    .app_list_states
                    .command_palette_command_search
                    .select(Some(0));
            }
        }
    }

    pub fn calculate_tags(app: &App) -> Vec<(String, u32)> {
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
        let mut tags: Vec<(String, u32)> = count_hash
            .iter()
            .map(|(tag, count)| (tag.clone(), *count))
            .collect();
        tags.sort_by(|a, b| {
            if a.1 == b.1 {
                a.0.cmp(&b.0)
            } else {
                b.1.cmp(&a.1)
            }
        });
        tags
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandPaletteActions {
    ChangeCurrentCardStatus,
    ChangeDateFormat,
    ChangeTheme,
    ChangeUIMode,
    ClearFilter,
    ConfigMenu,
    CreateATheme,
    DebugMenu,
    FilterByTag,
    HelpMenu,
    LoadASaveCloud,
    LoadASaveLocal,
    Login,
    Logout,
    MainMenu,
    NewBoard,
    NewCard,
    NoCommandsFound,
    Quit,
    ResetPassword,
    ResetUI,
    SaveKanbanState,
    SignUp,
    SyncLocalData,
}

impl Display for CommandPaletteActions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChangeCurrentCardStatus => write!(f, "Change Current Card Status"),
            Self::ChangeDateFormat => write!(f, "Change Date Format"),
            Self::ChangeTheme => write!(f, "Change Theme"),
            Self::ChangeUIMode => write!(f, "Change UI Mode"),
            Self::ClearFilter => write!(f, "Clear Filter"),
            Self::CreateATheme => write!(f, "Create a Theme"),
            Self::DebugMenu => write!(f, "Toggle Debug Panel"),
            Self::FilterByTag => write!(f, "Filter by Tag"),
            Self::LoadASaveCloud => write!(f, "Load a Save (Cloud)"),
            Self::LoadASaveLocal => write!(f, "Load a Save (Local)"),
            Self::Login => write!(f, "Login"),
            Self::Logout => write!(f, "Logout"),
            Self::NewBoard => write!(f, "New Board"),
            Self::NewCard => write!(f, "New Card"),
            Self::NoCommandsFound => write!(f, "No Commands Found"),
            Self::ConfigMenu => write!(f, "Configure"),
            Self::HelpMenu => write!(f, "Open Help Menu"),
            Self::MainMenu => write!(f, "Open Main Menu"),
            Self::Quit => write!(f, "Quit"),
            Self::ResetPassword => write!(f, "Reset Password"),
            Self::ResetUI => write!(f, "Reset UI"),
            Self::SaveKanbanState => write!(f, "Save Kanban State"),
            Self::SignUp => write!(f, "Sign Up"),
            Self::SyncLocalData => write!(f, "Sync Local Data"),
        }
    }
}

impl CommandPaletteActions {
    pub fn all(debug_mode: bool) -> Vec<Self> {
        let all = vec![
            Self::ChangeCurrentCardStatus,
            Self::ChangeDateFormat,
            Self::ChangeTheme,
            Self::ChangeUIMode,
            Self::ClearFilter,
            Self::ConfigMenu,
            Self::CreateATheme,
            Self::FilterByTag,
            Self::HelpMenu,
            Self::LoadASaveCloud,
            Self::LoadASaveLocal,
            Self::Login,
            Self::Logout,
            Self::MainMenu,
            Self::NewBoard,
            Self::NewCard,
            Self::Quit,
            Self::ResetPassword,
            Self::ResetUI,
            Self::SaveKanbanState,
            Self::SignUp,
            Self::SyncLocalData,
        ];

        if cfg!(debug_assertions) || debug_mode {
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
                "configure" => Some(Self::ConfigMenu),
                "save kanban state" => Some(Self::SaveKanbanState),
                "load a save (local)" => Some(Self::LoadASaveLocal),
                "new board" => Some(Self::NewBoard),
                "new card" => Some(Self::NewCard),
                "reset ui" => Some(Self::ResetUI),
                "open main menu" => Some(Self::MainMenu),
                "open help menu" => Some(Self::HelpMenu),
                "change ui mode" => Some(Self::ChangeUIMode),
                "change current card status" => Some(Self::ChangeCurrentCardStatus),
                "toggle debug panel" => Some(Self::DebugMenu),
                "change theme" => Some(Self::ChangeTheme),
                "create a theme" => Some(Self::CreateATheme),
                "filter by tag" => Some(Self::FilterByTag),
                "clear filter" => Some(Self::ClearFilter),
                "change date format" => Some(Self::ChangeDateFormat),
                "login" => Some(Self::Login),
                "sign up" => Some(Self::SignUp),
                "reset password" => Some(Self::ResetPassword),
                "logout" => Some(Self::Logout),
                "sync local data" => Some(Self::SyncLocalData),
                "load a save (cloud)" => Some(Self::LoadASaveCloud),
                "quit" => Some(Self::Quit),
                _ => None,
            }
        } else {
            match s {
                "Configure" => Some(Self::ConfigMenu),
                "Save Kanban State" => Some(Self::SaveKanbanState),
                "Load a Save (Local)" => Some(Self::LoadASaveLocal),
                "New Board" => Some(Self::NewBoard),
                "New Card" => Some(Self::NewCard),
                "Reset UI" => Some(Self::ResetUI),
                "Open Main Menu" => Some(Self::MainMenu),
                "Open Help Menu" => Some(Self::HelpMenu),
                "Change UI Mode" => Some(Self::ChangeUIMode),
                "Change Current Card Status" => Some(Self::ChangeCurrentCardStatus),
                "Toggle Debug Panel" => Some(Self::DebugMenu),
                "Change Theme" => Some(Self::ChangeTheme),
                "Create a Theme" => Some(Self::CreateATheme),
                "Filter by Tag" => Some(Self::FilterByTag),
                "Clear Filter" => Some(Self::ClearFilter),
                "Change Date Format" => Some(Self::ChangeDateFormat),
                "Login" => Some(Self::Login),
                "Sign Up" => Some(Self::SignUp),
                "Reset Password" => Some(Self::ResetPassword),
                "Logout" => Some(Self::Logout),
                "Sync Local Data" => Some(Self::SyncLocalData),
                "Load a Save (Cloud)" => Some(Self::LoadASaveCloud),
                "Quit" => Some(Self::Quit),
                _ => None,
            }
        }
    }
}
