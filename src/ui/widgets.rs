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
use ratatui::style::{Color, Style};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
};
use strum::{EnumIter, EnumString, IntoEnumIterator};
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
        let term_background_color = if let Some(bg_color) = app.current_theme.general_style.bg {
            TextColorOptions::from(bg_color).to_rgb()
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
    pub command_search_results: Option<Vec<CommandPaletteActions>>,
    pub last_focus: Option<Focus>,
    pub last_search_string: String,
}

impl CommandPaletteWidget {
    pub fn new(debug_mode: bool) -> Self {
        let available_commands = CommandPaletteActions::all(debug_mode);
        Self {
            already_in_user_input_mode: false,
            available_commands,
            board_search_results: None,
            card_search_results: None,
            command_search_results: None,
            last_focus: None,
            last_search_string: RANDOM_SEARCH_TERM.to_string(),
        }
    }

    pub async fn handle_command(app: &mut App<'_>) -> AppReturn {
        if let Some(command_index) = app
            .state
            .app_list_states
            .command_palette_command_search
            .selected()
        {
            if let Some(command) =
                if let Some(search_results) = &app.widgets.command_palette.command_search_results {
                    search_results.get(command_index)
                } else {
                    None
                }
            {
                match command {
                    CommandPaletteActions::Quit => {
                        info!("Quitting");
                        return handle_exit(app).await;
                    }
                    CommandPaletteActions::ConfigMenu => {
                        app.close_popup();
                        app.set_ui_mode(UiMode::ConfigMenu);
                        app.state.app_table_states.config.select(Some(0));
                    }
                    CommandPaletteActions::MainMenu => {
                        app.close_popup();
                        app.set_ui_mode(UiMode::MainMenu);
                        app.state.app_list_states.main_menu.select(Some(0));
                    }
                    CommandPaletteActions::HelpMenu => {
                        app.close_popup();
                        app.set_ui_mode(UiMode::HelpMenu);
                        app.state.app_table_states.help.select(Some(0));
                    }
                    CommandPaletteActions::SaveKanbanState => {
                        app.close_popup();
                        app.dispatch(IoEvent::SaveLocalData).await;
                    }
                    CommandPaletteActions::NewBoard => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            app.close_popup();
                            app.set_ui_mode(UiMode::NewBoard);
                        } else {
                            app.close_popup();
                            app.send_error_toast("Cannot create a new board in this view", None);
                        }
                    }
                    CommandPaletteActions::NewCard => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            if app.state.current_board_id.is_none() {
                                app.send_error_toast("No board Selected / Available", None);
                                app.close_popup();
                                app.state.app_status = AppStatus::Initialized;
                                return AppReturn::Continue;
                            }
                            app.close_popup();
                            app.set_ui_mode(UiMode::NewCard);
                        } else {
                            app.close_popup();
                            app.send_error_toast("Cannot create a new card in this view", None);
                        }
                    }
                    CommandPaletteActions::ResetUI => {
                        app.close_popup();
                        app.set_ui_mode(app.config.default_ui_mode);
                        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
                    }
                    CommandPaletteActions::ChangeUIMode => {
                        app.set_popup_mode(PopupMode::ChangeUIMode);
                    }
                    CommandPaletteActions::ChangeCurrentCardStatus => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            if let Some(current_board_id) = app.state.current_board_id {
                                if let Some(current_board) =
                                    app.boards.get_mut_board_with_id(current_board_id)
                                {
                                    if let Some(current_card_id) = app.state.current_card_id {
                                        if current_board
                                            .cards
                                            .get_card_with_id(current_card_id)
                                            .is_some()
                                        {
                                            app.set_popup_mode(PopupMode::CardStatusSelector);
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
                            app.close_popup();
                            app.send_error_toast("Cannot change card status in this view", None);
                        }
                    }
                    CommandPaletteActions::ChangeCurrentCardPriority => {
                        if UiMode::view_modes().contains(&app.state.ui_mode) {
                            if let Some(current_board_id) = app.state.current_board_id {
                                if let Some(current_board) =
                                    app.boards.get_mut_board_with_id(current_board_id)
                                {
                                    if let Some(current_card_id) = app.state.current_card_id {
                                        if current_board
                                            .cards
                                            .get_card_with_id(current_card_id)
                                            .is_some()
                                        {
                                            app.set_popup_mode(PopupMode::CardPrioritySelector);
                                            app.state.app_status = AppStatus::Initialized;
                                            app.state
                                                .app_list_states
                                                .card_priority_selector
                                                .select(Some(0));
                                            return AppReturn::Continue;
                                        }
                                    }
                                }
                            }
                            app.send_error_toast("Could not find current card", None);
                        } else {
                            app.close_popup();
                            app.send_error_toast("Cannot change card priority in this view", None);
                        }
                    }
                    CommandPaletteActions::LoadASaveLocal => {
                        app.close_popup();
                        reset_preview_boards(app);
                        app.set_ui_mode(UiMode::LoadLocalSave);
                    }
                    CommandPaletteActions::DebugMenu => {
                        app.state.debug_menu_toggled = !app.state.debug_menu_toggled;
                        app.close_popup();
                    }
                    CommandPaletteActions::ChangeTheme => {
                        app.set_popup_mode(PopupMode::ChangeTheme);
                    }
                    CommandPaletteActions::CreateATheme => {
                        app.set_ui_mode(UiMode::CreateTheme);
                        app.close_popup();
                    }
                    CommandPaletteActions::FilterByTag => {
                        let tags = Self::calculate_tags(app);
                        if tags.is_empty() {
                            app.send_warning_toast("No tags found to filter with", None);
                            app.close_popup();
                        } else {
                            app.set_popup_mode(PopupMode::FilterByTag);
                            app.state.all_available_tags = Some(tags);
                        }
                    }
                    CommandPaletteActions::ClearFilter => {
                        if app.filtered_boards.is_empty() {
                            app.send_warning_toast("No filters to clear", None);
                            app.close_popup();
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        } else {
                            app.send_info_toast("All Filters Cleared", None);
                        }
                        app.state.filter_tags = None;
                        app.state.all_available_tags = None;
                        app.state.app_list_states.filter_by_tag_list.select(None);
                        app.close_popup();
                        app.filtered_boards.reset();
                        refresh_visible_boards_and_cards(app);
                    }
                    CommandPaletteActions::ChangeDateFormat => {
                        app.set_popup_mode(PopupMode::ChangeDateFormatPopup);
                    }
                    CommandPaletteActions::NoCommandsFound => {
                        app.close_popup();
                        app.state.app_status = AppStatus::Initialized;
                        return AppReturn::Continue;
                    }
                    CommandPaletteActions::Login => {
                        if app.state.user_login_data.auth_token.is_some() {
                            app.send_error_toast("Already logged in", None);
                            app.close_popup();
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                        app.set_ui_mode(UiMode::Login);
                        app.close_popup();
                    }
                    CommandPaletteActions::Logout => {
                        app.dispatch(IoEvent::Logout).await;
                        app.close_popup();
                    }
                    CommandPaletteActions::SignUp => {
                        app.set_ui_mode(UiMode::SignUp);
                        app.close_popup();
                    }
                    CommandPaletteActions::ResetPassword => {
                        app.set_ui_mode(UiMode::ResetPassword);
                        app.close_popup();
                    }
                    CommandPaletteActions::SyncLocalData => {
                        app.dispatch(IoEvent::SyncLocalData).await;
                        app.close_popup();
                    }
                    CommandPaletteActions::LoadASaveCloud => {
                        if app.state.user_login_data.auth_token.is_some() {
                            app.set_ui_mode(UiMode::LoadCloudSave);
                            reset_preview_boards(app);
                            app.dispatch(IoEvent::GetCloudData).await;
                            app.close_popup();
                        } else {
                            error!("Not logged in");
                            app.send_error_toast("Not logged in", None);
                            app.close_popup();
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                    }
                }
                app.state.text_buffers.command_palette.reset();
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
        AppReturn::Continue
    }

    fn update(app: &mut App) {
        if let Some(PopupMode::CommandPalette) = app.state.popup_mode {
            if app
                .state
                .text_buffers
                .command_palette
                .get_joined_lines()
                .to_lowercase()
                == app.widgets.command_palette.last_search_string
            {
                return;
            }
            let current_search_string = app.state.text_buffers.command_palette.get_joined_lines();
            let current_search_string = current_search_string.to_lowercase();
            let search_results = app
                .widgets
                .command_palette
                .available_commands
                .iter()
                .filter(|action| {
                    action
                        .to_string()
                        .to_lowercase()
                        .contains(&current_search_string)
                })
                .cloned()
                .collect::<Vec<CommandPaletteActions>>();

            // Making sure the results which start with the search string are shown first
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
                for board in app.boards.get_boards() {
                    for card in board.cards.get_all_cards() {
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
            if card_search_results.is_empty() {
                app.widgets.command_palette.card_search_results = None;
            } else {
                app.widgets.command_palette.card_search_results = Some(card_search_results.clone());
            }

            let mut board_search_results: Vec<(String, (u64, u64))> = vec![];
            if !current_search_string.is_empty() {
                for board in app.boards.get_boards() {
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
            if board_search_results.is_empty() {
                app.widgets.command_palette.board_search_results = None;
            } else {
                app.widgets.command_palette.board_search_results =
                    Some(board_search_results.clone());
            }

            app.widgets.command_palette.command_search_results = Some(command_search_results);
            app.widgets.command_palette.last_search_string = current_search_string;
            if let Some(search_results) = &app.widgets.command_palette.command_search_results {
                if !search_results.is_empty() {
                    app.state
                        .app_list_states
                        .command_palette_command_search
                        .select(Some(0));
                }
            }
        }
    }

    pub fn calculate_tags(app: &App) -> Vec<(String, u32)> {
        let mut tags: Vec<String> = vec![];
        for board in app.boards.get_boards() {
            for card in board.cards.get_all_cards() {
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

#[derive(Clone, Debug, PartialEq, EnumIter, EnumString)]
pub enum CommandPaletteActions {
    ChangeCurrentCardStatus,
    ChangeCurrentCardPriority,
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
            Self::ChangeCurrentCardPriority => write!(f, "Change Current Card Priority"),
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
        let mut all = CommandPaletteActions::iter().collect::<Vec<Self>>();
        // sort
        all.sort_by_key(|a| a.to_string());

        if cfg!(debug_assertions) || debug_mode {
            all
        } else {
            all.into_iter()
                .filter(|action| !matches!(action, Self::DebugMenu))
                .collect()
        }
    }
}
