use super::{TextColorOptions, Theme};
use crate::{
    app::{
        app_helper::reset_preview_boards,
        handle_exit,
        state::{AppState, AppStatus, Focus, PopupMode, UiMode},
        App, AppReturn, DateTimeFormat,
    },
    constants::{
        DATE_TIME_PICKER_ANIM_DURATION, MIN_DATE_PICKER_HEIGHT, MIN_DATE_PICKER_WIDTH,
        RANDOM_SEARCH_TERM, TIME_PICKER_WIDTH, TOAST_FADE_IN_TIME, TOAST_FADE_OUT_TIME,
    },
    io::{io_handler::refresh_visible_boards_and_cards, IoEvent},
    util::lerp_between,
};
use chrono::{Datelike, NaiveDate, Timelike};
use log::{debug, error, info};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::Arc,
    time::Duration,
    vec,
};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use tokio::time::Instant;

trait Widget {
    fn update(app: &mut App);
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
        DateTimePickerWidget::update(&mut app);
    }
}

pub struct Widgets<'a> {
    pub command_palette: CommandPaletteWidget,
    pub close_button: CloseButtonWidget,
    pub toasts: Vec<ToastWidget>,
    pub date_time_picker: DateTimePickerWidget<'a>,
}

impl<'a> Widgets<'a> {
    pub fn new(theme: Theme, debug_mode: bool, calender_type: CalenderType) -> Self {
        Self {
            command_palette: CommandPaletteWidget::new(debug_mode),
            close_button: CloseButtonWidget::new(theme.general_style),
            toasts: vec![],
            date_time_picker: DateTimePickerWidget::new(calender_type),
        }
    }
}

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
}

impl Widget for ToastWidget {
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
}

impl Widget for CloseButtonWidget {
    fn update(app: &mut App) {
        if app.state.focus == Focus::CloseButton {
            let theme = app.current_theme.clone();
            let disable_animations = app.config.disable_animations;
            let widget = &mut app.widgets.close_button;
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

    pub fn reset(&mut self, app_state: &mut AppState) {
        self.board_search_results = None;
        self.card_search_results = None;
        self.command_search_results = None;
        self.last_search_string = RANDOM_SEARCH_TERM.to_string();
        app_state.text_buffers.command_palette.reset();
        Self::reset_list_states(app_state);
    }

    pub fn reset_list_states(app_state: &mut AppState) {
        app_state
            .app_list_states
            .command_palette_command_search
            .select(None);
        app_state
            .app_list_states
            .command_palette_card_search
            .select(None);
        app_state
            .app_list_states
            .command_palette_board_search
            .select(None);
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
                        app.close_popup();
                        app.set_popup_mode(PopupMode::ChangeUIMode);
                    }
                    CommandPaletteActions::ChangeCurrentCardStatus => {
                        if !UiMode::view_modes().contains(&app.state.ui_mode) {
                            app.send_error_toast("Cannot change card status in this view", None);
                            return AppReturn::Continue;
                        }
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
                                        app.close_popup();
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
                    }
                    CommandPaletteActions::ChangeCurrentCardPriority => {
                        if !UiMode::view_modes().contains(&app.state.ui_mode) {
                            app.send_error_toast("Cannot change card priority in this view", None);
                            return AppReturn::Continue;
                        }
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
                                        app.close_popup();
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
                        app.close_popup();
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
                        } else {
                            app.close_popup();
                            app.set_popup_mode(PopupMode::FilterByTag);
                            app.state.all_available_tags = Some(tags);
                        }
                    }
                    CommandPaletteActions::ClearFilter => {
                        if app.filtered_boards.is_empty() {
                            app.send_warning_toast("No filters to clear", None);
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
                        app.close_popup();
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
                app.widgets.command_palette.reset(&mut app.state);
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
        if app.state.z_stack.last() != Some(&PopupMode::CustomHexColorPromptFG)
            || app.state.z_stack.last() != Some(&PopupMode::CustomHexColorPromptBG)
        {
            app.state.app_status = AppStatus::Initialized;
        }
        AppReturn::Continue
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

impl Widget for CommandPaletteWidget {
    fn update(app: &mut App) {
        if let Some(PopupMode::CommandPalette) = app.state.z_stack.last() {
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
        // remove no commands found
        all.retain(|action| !matches!(action, Self::NoCommandsFound));

        if cfg!(debug_assertions) || debug_mode {
            all
        } else {
            all.into_iter()
                .filter(|action| !matches!(action, Self::DebugMenu))
                .collect()
        }
    }
}

#[derive(Debug)]
enum WidgetAnimState {
    Closed,
    Closing,
    Open,
    Opening,
}

impl WidgetAnimState {
    fn complete_current_stage(&self) -> Self {
        match self {
            Self::Closed => Self::Closed,
            Self::Closing => Self::Closed,
            Self::Open => Self::Open,
            Self::Opening => Self::Open,
        }
    }
}

// enum AnimType {
//     Slide,
//     // Implement Pulse for close button (get rid of old janky solution used
//     // in close button widget, refer DateTimePickerWidget for example)
//     // Pulse(Duration),
// }

#[derive(Serialize, Deserialize, Debug, Clone, Default, EnumString, Display)]
pub enum CalenderType {
    #[default]
    SundayFirst,
    MondayFirst,
}

type CalculatedMouseCoordsCache = Option<(Vec<(Rect, u8)>, chrono::NaiveDateTime, Rect)>;

pub struct DateTimePickerWidget<'a> {
    pub time_picker_active: bool,
    pub anchor: Option<(u16, u16)>,
    pub viewport_corrected_anchor: Option<(u16, u16)>,
    date_picker_anim_state: WidgetAnimState,
    time_picker_anim_state: WidgetAnimState,
    calender_type: CalenderType,
    pub widget_height: u16,
    pub widget_width: u16,
    pub date_target_height: u16,
    pub date_target_width: u16,
    pub time_target_width: u16,
    pub selected_date_time: Option<chrono::NaiveDateTime>,
    last_anim_tick: Instant,
    pub styled_date_lines: (Vec<Line<'a>>, Option<chrono::NaiveDateTime>),
    pub styled_time_lines: (Vec<Line<'a>>, Option<chrono::NaiveDateTime>),
    pub calculated_mouse_coords: CalculatedMouseCoordsCache,
    pub current_viewport: Option<Rect>,
    pub current_render_area: Option<Rect>,
}

impl<'a> DateTimePickerWidget<'a> {
    pub fn new(calender_type: CalenderType) -> Self {
        Self {
            time_picker_active: false,
            anchor: None,
            viewport_corrected_anchor: None,
            date_picker_anim_state: WidgetAnimState::Closed,
            time_picker_anim_state: WidgetAnimState::Closed,
            calender_type,
            widget_height: MIN_DATE_PICKER_HEIGHT,
            widget_width: MIN_DATE_PICKER_WIDTH,
            date_target_height: MIN_DATE_PICKER_HEIGHT,
            date_target_width: MIN_DATE_PICKER_WIDTH,
            time_target_width: TIME_PICKER_WIDTH,
            selected_date_time: None,
            last_anim_tick: Instant::now(),
            styled_date_lines: (vec![], None),
            styled_time_lines: (vec![], None),
            calculated_mouse_coords: None,
            current_viewport: None,
            current_render_area: None,
        }
    }

    pub fn set_calender_type(&mut self, calender_type: CalenderType) {
        self.calender_type = calender_type;
    }

    pub fn open_date_picker(&mut self) {
        if matches!(self.date_picker_anim_state, WidgetAnimState::Closed)
            || matches!(self.date_picker_anim_state, WidgetAnimState::Closing)
        {
            self.time_picker_active = false;
            self.time_picker_anim_state = WidgetAnimState::Closed;
            self.date_picker_anim_state = WidgetAnimState::Opening;
            self.last_anim_tick = Instant::now();
        }
    }

    pub fn close_date_picker(&mut self) {
        if matches!(self.date_picker_anim_state, WidgetAnimState::Open)
            || matches!(self.date_picker_anim_state, WidgetAnimState::Opening)
        {
            self.time_picker_active = false;
            self.time_picker_anim_state = WidgetAnimState::Closed;
            self.date_picker_anim_state = WidgetAnimState::Closing;
            self.last_anim_tick = Instant::now();
        }
    }

    pub fn reset(&mut self) {
        self.time_picker_active = false;
        self.anchor = None;
        self.viewport_corrected_anchor = None;
        self.date_picker_anim_state = WidgetAnimState::Closed;
        self.time_picker_anim_state = WidgetAnimState::Closed;
        self.selected_date_time = None;
        self.widget_height = MIN_DATE_PICKER_HEIGHT;
        self.widget_width = MIN_DATE_PICKER_WIDTH;
        self.date_target_height = MIN_DATE_PICKER_HEIGHT;
        self.date_target_width = MIN_DATE_PICKER_WIDTH;
        self.styled_date_lines = (vec![], None);
        self.styled_time_lines = (vec![], None);
        self.current_viewport = None;
        debug!("DateTimePickerWidget reset");
    }

    pub fn open_time_picker(&mut self) {
        if !self.time_picker_active {
            self.time_picker_anim_state = WidgetAnimState::Opening;
            self.last_anim_tick = Instant::now();
            self.time_picker_active = true;
        }
    }

    pub fn close_time_picker(&mut self) {
        if self.time_picker_active {
            self.time_picker_anim_state = WidgetAnimState::Closing;
            self.last_anim_tick = Instant::now();
            self.time_picker_active = false;
        }
    }

    fn num_days_in_month(year: i32, month: u32) -> Option<u32> {
        // the first day of the next month...
        let (y, m) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let d = match NaiveDate::from_ymd_opt(y, m, 1) {
            Some(d) => d,
            None => return None,
        };
        d.pred_opt().map(|d| d.day())
    }

    fn adjust_selected_date_with_days(&mut self, days: i64) {
        if let Some(current_date) = self.selected_date_time {
            self.selected_date_time = current_date.checked_add_signed(chrono::Duration::days(days));
        } else {
            debug!("No selected date time found, defaulting to current date time");
            self.selected_date_time = chrono::Local::now()
                .naive_local()
                .checked_add_signed(chrono::Duration::days(days));
        }
    }

    fn adjust_selected_date_with_months(&mut self, months: i64) {
        self.selected_date_time = if let Some(selected_date_time) = self.selected_date_time {
            if months.is_negative() {
                selected_date_time
                    .checked_sub_months(chrono::Months::new(months.unsigned_abs() as u32))
            } else {
                selected_date_time
                    .checked_add_months(chrono::Months::new(months.unsigned_abs() as u32))
            }
        } else {
            debug!("No selected date time found, defaulting to current date time");
            let current_date_time = chrono::Local::now().naive_local();
            if months.is_negative() {
                current_date_time
                    .checked_sub_months(chrono::Months::new(months.unsigned_abs() as u32))
            } else {
                current_date_time
                    .checked_add_months(chrono::Months::new(months.unsigned_abs() as u32))
            }
        };
    }

    fn adjust_selected_date_with_years(&mut self, years: i64) {
        let current_date_time = if let Some(selected_date_time) = self.selected_date_time {
            selected_date_time
        } else {
            debug!("No selected date time found, defaulting to current date time");
            chrono::Local::now().naive_local()
        };
        let current_time = current_date_time.time();
        let modified_years = current_date_time.year() as i64 + years;
        let modified_date = NaiveDate::from_ymd_opt(
            modified_years as i32,
            current_date_time.month(),
            current_date_time.day(),
        );
        if let Some(modified_date) = modified_date {
            self.selected_date_time = Some(modified_date.and_time(current_time));
        } else {
            debug!("Could not adjust the selected date with years");
        }
    }

    fn adjust_selected_date_with_seconds(&mut self, seconds: i64) {
        if let Some(current_date) = self.selected_date_time {
            self.selected_date_time =
                current_date.checked_add_signed(chrono::Duration::seconds(seconds));
        } else {
            debug!("No selected date time found, defaulting to current date time");
            self.selected_date_time = chrono::Local::now()
                .naive_local()
                .checked_add_signed(chrono::Duration::seconds(seconds));
        }
    }

    pub fn calender_move_up(&mut self) {
        self.adjust_selected_date_with_days(-7);
    }

    pub fn move_hours_next(&mut self) {
        self.adjust_selected_date_with_seconds(3600);
    }

    pub fn move_minutes_next(&mut self) {
        self.adjust_selected_date_with_seconds(60);
    }

    pub fn move_seconds_next(&mut self) {
        self.adjust_selected_date_with_seconds(1);
    }

    pub fn calender_move_down(&mut self) {
        self.adjust_selected_date_with_days(7);
    }

    pub fn move_hours_prv(&mut self) {
        self.adjust_selected_date_with_seconds(-3600);
    }

    pub fn move_minutes_prv(&mut self) {
        self.adjust_selected_date_with_seconds(-60);
    }

    pub fn move_seconds_prv(&mut self) {
        self.adjust_selected_date_with_seconds(-1);
    }

    pub fn move_left(&mut self) {
        self.adjust_selected_date_with_days(-1);
    }

    pub fn move_right(&mut self) {
        self.adjust_selected_date_with_days(1);
    }

    pub fn month_prv(&mut self) {
        self.adjust_selected_date_with_months(-1);
    }

    pub fn month_next(&mut self) {
        self.adjust_selected_date_with_months(1);
    }

    pub fn year_prv(&mut self) {
        self.adjust_selected_date_with_years(-1);
    }

    pub fn year_next(&mut self) {
        self.adjust_selected_date_with_years(1);
    }

    /// Returns the styled lines of the dates in the current month,
    /// the adjusted height of the widget and the selected month and year
    pub fn get_styled_lines_of_dates(
        &mut self,
        popup_mode: bool,
        current_theme: &Theme,
    ) -> (String, String) {
        let date = self
            .selected_date_time
            .unwrap_or_else(|| chrono::Local::now().naive_local());
        let current_month = chrono::Month::try_from(date.month() as u8).unwrap();
        let current_year = date.year();
        if self.styled_date_lines.1 == Some(date) {
            return (current_month.name().to_string(), current_year.to_string());
        }
        let first_day_of_month = match self.calender_type {
            CalenderType::MondayFirst => date.with_day(1).unwrap().weekday().number_from_monday(),
            CalenderType::SundayFirst => date.with_day(1).unwrap().weekday().number_from_sunday(),
        };
        let previous_month = current_month.pred();
        let number_of_days_in_previous_month =
            Self::num_days_in_month(current_year, previous_month.number_from_month()).unwrap();
        let number_of_days_in_current_month =
            Self::num_days_in_month(current_year, current_month.number_from_month()).unwrap();
        let num_lines_required = if first_day_of_month == 1 {
            (number_of_days_in_current_month as f32 / 7.0).ceil() as u32
        } else {
            ((number_of_days_in_current_month + first_day_of_month - 1) as f32 / 7.0).ceil() as u32
        };
        let previous_month_padding_required = first_day_of_month != 1;
        let next_month_padding_required = num_lines_required * 7 > number_of_days_in_current_month;

        let inactive_style = current_theme.inactive_text_style;
        let general_style = if popup_mode {
            inactive_style
        } else {
            current_theme.general_style
        };
        let highlight_style = if popup_mode {
            inactive_style
        } else {
            current_theme.keyboard_focus_style
        };

        let mut lines: Vec<Line> = Vec::new();
        let days_line = match self.calender_type {
            CalenderType::MondayFirst => Line::from(vec![
                Span::styled("Mo ", general_style),
                Span::styled("Tu ", general_style),
                Span::styled("We ", general_style),
                Span::styled("Th ", general_style),
                Span::styled("Fr ", general_style),
                Span::styled("Sa ", general_style),
                Span::styled("Su ", general_style),
            ]),
            CalenderType::SundayFirst => Line::from(vec![
                Span::styled("Su ", general_style),
                Span::styled("Mo ", general_style),
                Span::styled("Tu ", general_style),
                Span::styled("We ", general_style),
                Span::styled("Th ", general_style),
                Span::styled("Fr ", general_style),
                Span::styled("Sa ", general_style),
            ]),
        };
        lines.push(days_line);
        let mut current_date = 1;
        for line_num in 0..num_lines_required {
            let mut current_line_spans: Vec<Span> = Vec::new();
            if line_num == 0 {
                if previous_month_padding_required {
                    for pre_month_days in 0..first_day_of_month - 1 {
                        let calc = number_of_days_in_previous_month - first_day_of_month
                            + pre_month_days
                            + 2;
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc), inactive_style));
                    }
                }
                for current_month_days in first_day_of_month..8 {
                    let calc_date = current_month_days - first_day_of_month + 1;
                    current_date = calc_date;
                    if calc_date == date.day() {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), highlight_style));
                    } else {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), general_style));
                    }
                }
            } else if line_num == num_lines_required - 1 {
                for calc_date in current_date + 1..number_of_days_in_current_month + 1 {
                    if calc_date == date.day() {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), highlight_style));
                    } else {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), general_style));
                    }
                }
                if next_month_padding_required {
                    let mut next_month_days = 1;
                    for _ in 0..7 - current_line_spans.len() {
                        current_line_spans.push(Span::styled(
                            format!("{:3}", next_month_days),
                            inactive_style,
                        ));
                        next_month_days += 1;
                    }
                }
            } else {
                for current_month_days in 1..8 {
                    let calc_date = (line_num * 7) + current_month_days - first_day_of_month + 1;
                    current_date = calc_date;
                    if calc_date == date.day() {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), highlight_style));
                    } else {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), general_style));
                    }
                }
            }
            lines.push(Line::from(current_line_spans));
        }
        self.date_target_height = MIN_DATE_PICKER_HEIGHT + lines.len() as u16 + 2; // Extra 2 for header and space below it
        self.date_target_width =
            ((current_month.name().to_string().len() + 3 + current_year.to_string().len() + 3)
                as u16)
                .max(MIN_DATE_PICKER_WIDTH);
        self.styled_date_lines = (lines, Some(date));
        (current_month.name().to_string(), current_year.to_string())
    }

    fn adjust_hour(hour: i64) -> i64 {
        if hour < 0 {
            24 + hour
        } else if hour > 23 {
            hour - 24
        } else {
            hour
        }
    }

    fn adjust_minute_or_second(value: i64) -> i64 {
        if value < 0 {
            60 + value
        } else if value > 59 {
            value - 60
        } else {
            value
        }
    }

    fn create_time_line(
        lines: &mut Vec<Line>,
        hms: (i64, i64, i64),
        styles: (Style, Style, Style),
        current_line: bool,
    ) {
        let hour = if hms.0 < 10 {
            format!("{}  ", hms.0)
        } else {
            format!("{} ", hms.0)
        };
        let minute = if hms.1 < 10 {
            format!(" {}", hms.1)
        } else {
            format!("{}", hms.1)
        };
        let second = if hms.2 < 10 {
            format!("  {}", hms.2)
        } else {
            format!(" {}", hms.2)
        };
        if current_line {
            lines.push(Line::from(vec![
                Span::styled("-- ", styles.0),
                Span::styled("--", styles.1),
                Span::styled(" --", styles.2),
            ]));
            lines.push(Line::from(vec![
                Span::styled(hour.to_string(), styles.0),
                Span::styled(minute.to_string(), styles.1),
                Span::styled(second.to_string(), styles.2),
            ]));
            lines.push(Line::from(vec![
                Span::styled("-- ", styles.0),
                Span::styled("--", styles.1),
                Span::styled(" --", styles.2),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(hour.to_string(), styles.0),
                Span::styled(minute.to_string(), styles.1),
                Span::styled(second.to_string(), styles.2),
            ]));
        }
    }

    pub fn get_styled_lines_of_time(
        &mut self,
        popup_mode: bool,
        current_theme: &Theme,
        current_focus: &Focus,
    ) -> Vec<Line> {
        let date = self
            .selected_date_time
            .unwrap_or_else(|| chrono::Local::now().naive_local());

        if self.styled_time_lines.1 == Some(date) {
            return self.styled_time_lines.0.clone();
        }

        let general_style = if popup_mode {
            current_theme.inactive_text_style
        } else {
            current_theme.general_style
        };
        let highlight_style = if popup_mode {
            current_theme.inactive_text_style
        } else {
            current_theme.keyboard_focus_style
        };
        let hour_style = if current_focus == &Focus::DTPHour {
            highlight_style
        } else {
            general_style
        };
        let minute_style = if current_focus == &Focus::DTPMinute {
            highlight_style
        } else {
            general_style
        };
        let second_style = if current_focus == &Focus::DTPSecond {
            highlight_style
        } else {
            general_style
        };

        let current_time = date.time();
        let current_hours = current_time.hour();
        let current_minutes = current_time.minute();
        let current_seconds = current_time.second();
        let available_height = self.date_target_height.saturating_sub(6); // 2 for border, 2 for extra padding, 2 for current time line
        let num_previous_lines = available_height / 2;
        let num_after_lines = if available_height % 2 == 0 {
            num_previous_lines
        } else {
            num_previous_lines + 1
        };
        let mut lines: Vec<Line> = Vec::new();

        for offset in (1..(num_previous_lines + 1)).rev() {
            let current_hour = Self::adjust_hour(current_hours as i64 - offset as i64);
            let current_minute =
                Self::adjust_minute_or_second(current_minutes as i64 - offset as i64);
            let current_second =
                Self::adjust_minute_or_second(current_seconds as i64 - offset as i64);
            Self::create_time_line(
                &mut lines,
                (current_hour, current_minute, current_second),
                (
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                ),
                false,
            );
        }

        Self::create_time_line(
            &mut lines,
            (
                current_hours as i64,
                current_minutes as i64,
                current_seconds as i64,
            ),
            (hour_style, minute_style, second_style),
            true,
        );

        for offset in 1..=num_after_lines {
            let current_hour = Self::adjust_hour(current_hours as i64 + offset as i64);
            let current_minute =
                Self::adjust_minute_or_second(current_minutes as i64 + offset as i64);
            let current_second =
                Self::adjust_minute_or_second(current_seconds as i64 + offset as i64);
            Self::create_time_line(
                &mut lines,
                (current_hour, current_minute, current_second),
                (
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                ),
                false,
            );
        }
        lines
    }

    fn calculate_mouse_coords_for_dates(&mut self) {
        if self.current_render_area.is_none() {
            debug!("No render area found for calculating mouse coords");
            return;
        }
        let render_area = self.current_render_area.unwrap();
        let top_padding = 4; // border, header, extra space, day line
        let left_padding = 2; // border, margin
        let date = self
            .selected_date_time
            .unwrap_or_else(|| chrono::Local::now().naive_local());
        let current_month = chrono::Month::try_from(date.month() as u8).unwrap();
        let current_year = date.year();
        let first_day_of_month = match self.calender_type {
            CalenderType::MondayFirst => {
                date.with_day(1).unwrap().weekday().number_from_monday() - 1
            } // Starts from 0
            CalenderType::SundayFirst => {
                date.with_day(1).unwrap().weekday().number_from_sunday() - 1
            } // Starts from 0
        };
        let number_of_days_in_current_month =
            Self::num_days_in_month(current_year, current_month.number_from_month()).unwrap();
        let mut record = Vec::new();
        for iter_date in 0..number_of_days_in_current_month as u16 {
            // Calculate the correct row and column taking into account the first day of the month
            let adjusted_iter_date = iter_date + first_day_of_month as u16; // Adjust the iter_date based on the first day of the month
            let row = adjusted_iter_date / 7;
            let col = adjusted_iter_date % 7; // Use adjusted_iter_date for column calculation
            let x = render_area.x + left_padding + (col * 3) - 1; // Column position
            let y = render_area.y + top_padding + row - 1; // Row position
            let rect = Rect::new(x, y, 3, 1);
            record.push((rect, (iter_date + 1) as u8));
        }
        self.calculated_mouse_coords = Some((record, date, render_area));
    }

    pub fn get_date_time_as_string(&self, date_time_format: DateTimeFormat) -> String {
        if let Some(selected_date) = self.selected_date_time {
            selected_date
                .format(date_time_format.to_parser_string())
                .to_string()
        } else {
            "No Date Selected".to_string()
        }
    }

    fn calculate_animation_percentage(&self) -> f32 {
        let milliseconds_passed = self.last_anim_tick.elapsed().as_millis() as f32;
        milliseconds_passed / (DATE_TIME_PICKER_ANIM_DURATION as f32)
    }

    // Update the height for the date picker animation
    fn update_date_picker_height(&mut self, current_percentage: f32, opening: bool) {
        self.widget_height = if opening {
            (MIN_DATE_PICKER_HEIGHT as f32
                + (self.date_target_height as f32 - MIN_DATE_PICKER_HEIGHT as f32)
                    * current_percentage) as u16
        } else {
            (self.date_target_height as f32
                - (self.date_target_height as f32 - MIN_DATE_PICKER_HEIGHT as f32)
                    * current_percentage) as u16
        };
    }

    // Update the width for the time picker animation
    fn update_time_picker_width(&mut self, current_percentage: f32, opening: bool) {
        self.widget_width = if opening {
            self.date_target_width + (self.time_target_width as f32 * current_percentage) as u16
        } else {
            self.date_target_width + self.time_target_width
                - (self.time_target_width as f32 * current_percentage) as u16
        };
    }

    pub fn select_date_in_current_month(&mut self, date_to_select: u8) {
        if let Some(selected_date) = self.selected_date_time {
            self.selected_date_time = selected_date.with_day(date_to_select as u32);
        } else {
            debug!("No selected date time found, defaulting to current date time");
            self.selected_date_time = chrono::Local::now()
                .naive_local()
                .with_day(date_to_select as u32);
        }
    }
}

impl<'a> Widget for DateTimePickerWidget<'a> {
    fn update(app: &mut App) {
        if app.state.z_stack.last() != Some(&PopupMode::DateTimePicker) {
            return;
        }
        let disable_animations = app.config.disable_animations;
        let date_time_picker = &mut app.widgets.date_time_picker;
        match date_time_picker.date_picker_anim_state {
            WidgetAnimState::Opening | WidgetAnimState::Closing => {
                if disable_animations {
                    date_time_picker.date_picker_anim_state = date_time_picker
                        .date_picker_anim_state
                        .complete_current_stage();
                    return;
                }
                let current_percentage = date_time_picker.calculate_animation_percentage();
                let opening = matches!(
                    date_time_picker.date_picker_anim_state,
                    WidgetAnimState::Opening
                );
                if current_percentage < 1.0 {
                    date_time_picker.date_picker_anim_state = if opening {
                        WidgetAnimState::Opening
                    } else {
                        WidgetAnimState::Closing
                    };
                    date_time_picker.update_date_picker_height(current_percentage, opening);
                } else {
                    date_time_picker.date_picker_anim_state = if opening {
                        WidgetAnimState::Open
                    } else {
                        WidgetAnimState::Closed
                    };
                }
            }
            WidgetAnimState::Open => {
                if date_time_picker.date_target_height != date_time_picker.widget_height {
                    date_time_picker.widget_height = date_time_picker.date_target_height;
                }
            }
            WidgetAnimState::Closed => {
                app.state.z_stack.pop();
                date_time_picker.reset();
            }
        }

        match date_time_picker.time_picker_anim_state {
            WidgetAnimState::Opening | WidgetAnimState::Closing => {
                if disable_animations {
                    date_time_picker.time_picker_anim_state = date_time_picker
                        .time_picker_anim_state
                        .complete_current_stage();
                    return;
                }
                let current_percentage = date_time_picker.calculate_animation_percentage();
                let opening = matches!(
                    date_time_picker.time_picker_anim_state,
                    WidgetAnimState::Opening
                );
                if current_percentage < 1.0 {
                    date_time_picker.time_picker_anim_state = if opening {
                        WidgetAnimState::Opening
                    } else {
                        WidgetAnimState::Closing
                    };
                    date_time_picker.update_time_picker_width(current_percentage, opening);
                } else {
                    date_time_picker.time_picker_anim_state = if opening {
                        WidgetAnimState::Open
                    } else {
                        WidgetAnimState::Closed
                    };
                }
            }
            WidgetAnimState::Open => {
                if (date_time_picker.date_target_width + date_time_picker.time_target_width)
                    != date_time_picker.widget_width
                {
                    date_time_picker.widget_width =
                        date_time_picker.date_target_width + date_time_picker.time_target_width;
                }
            }
            WidgetAnimState::Closed => {
                if date_time_picker.widget_width != date_time_picker.date_target_width {
                    date_time_picker.widget_width = date_time_picker.date_target_width;
                }
            }
        }

        if date_time_picker.anchor != date_time_picker.viewport_corrected_anchor {
            if let (Some(anchor), Some(viewport)) =
                (date_time_picker.anchor, date_time_picker.current_viewport)
            {
                debug!("Adjusting the anchor for the date time picker");
                let mut viewport_corrected_anchor = anchor;
                if anchor.1 + date_time_picker.date_target_height > viewport.height {
                    viewport_corrected_anchor.1 =
                        viewport.height - date_time_picker.date_target_height;
                }
                if anchor.0 + date_time_picker.date_target_width > viewport.width {
                    viewport_corrected_anchor.0 =
                        viewport.width - date_time_picker.date_target_width;
                }
                date_time_picker.viewport_corrected_anchor = Some(viewport_corrected_anchor);
            }
        }

        let mut re_calculate = false;
        if let Some((_, calc_date, calc_render_area)) = &date_time_picker.calculated_mouse_coords {
            if let Some(selected_date) = date_time_picker.selected_date_time {
                // check if same month
                if selected_date.month() != calc_date.month() {
                    re_calculate = true;
                }
            } else {
                re_calculate = true;
            }
            if let Some(render_area) = date_time_picker.current_render_area {
                if render_area != *calc_render_area {
                    re_calculate = true;
                }
            }
        } else if date_time_picker.current_render_area.is_some() {
            re_calculate = true;
        }
        if re_calculate {
            date_time_picker.calculate_mouse_coords_for_dates();
        }
    }
}
