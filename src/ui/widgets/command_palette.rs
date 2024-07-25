use crate::{
    app::{
        app_helper::reset_preview_boards,
        handle_exit,
        state::{AppState, AppStatus, Focus},
        App, AppReturn,
    },
    constants::RANDOM_SEARCH_TERM,
    io::{io_handler::refresh_visible_boards_and_cards, IoEvent},
    ui::{widgets::Widget, PopUp, View},
};
use log::{debug, error, info};
use std::{
    collections::HashMap,
    fmt::{self, Display},
    vec,
};
use strum::{EnumIter, EnumString, IntoEnumIterator};

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
                        app.set_view(View::ConfigMenu);
                        app.state.app_table_states.config.select(Some(0));
                    }
                    CommandPaletteActions::MainMenu => {
                        app.close_popup();
                        app.set_view(View::MainMenu);
                        app.state.app_list_states.main_menu.select(Some(0));
                    }
                    CommandPaletteActions::HelpMenu => {
                        app.close_popup();
                        app.set_view(View::HelpMenu);
                        app.state.app_table_states.help.select(Some(0));
                    }
                    CommandPaletteActions::SaveKanbanState => {
                        app.close_popup();
                        app.dispatch(IoEvent::SaveLocalData).await;
                    }
                    CommandPaletteActions::NewBoard => {
                        if View::views_with_kanban_board().contains(&app.state.current_view) {
                            app.close_popup();
                            app.set_view(View::NewBoard);
                        } else {
                            app.close_popup();
                            app.send_error_toast("Cannot create a new board in this view", None);
                        }
                    }
                    CommandPaletteActions::NewCard => {
                        if View::views_with_kanban_board().contains(&app.state.current_view) {
                            if app.state.current_board_id.is_none() {
                                app.send_error_toast("No board Selected / Available", None);
                                app.close_popup();
                                app.state.app_status = AppStatus::Initialized;
                                return AppReturn::Continue;
                            }
                            app.close_popup();
                            app.set_view(View::NewCard);
                        } else {
                            app.close_popup();
                            app.send_error_toast("Cannot create a new card in this view", None);
                        }
                    }
                    CommandPaletteActions::ResetUI => {
                        app.close_popup();
                        app.set_view(app.config.default_view);
                        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
                    }
                    CommandPaletteActions::ChangeView => {
                        app.close_popup();
                        app.set_popup(PopUp::ChangeView);
                    }
                    CommandPaletteActions::ChangeCurrentCardStatus => {
                        if !View::views_with_kanban_board().contains(&app.state.current_view) {
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
                                        app.set_popup(PopUp::CardStatusSelector);
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
                        if !View::views_with_kanban_board().contains(&app.state.current_view) {
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
                                        app.set_popup(PopUp::CardPrioritySelector);
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
                        app.set_view(View::LoadLocalSave);
                    }
                    CommandPaletteActions::DebugMenu => {
                        app.state.debug_menu_toggled = !app.state.debug_menu_toggled;
                        app.close_popup();
                    }
                    CommandPaletteActions::ChangeTheme => {
                        app.close_popup();
                        app.set_popup(PopUp::ChangeTheme);
                    }
                    CommandPaletteActions::CreateATheme => {
                        app.set_view(View::CreateTheme);
                        app.close_popup();
                    }
                    CommandPaletteActions::FilterByTag => {
                        let tags = Self::calculate_tags(app);
                        if tags.is_empty() {
                            app.send_warning_toast("No tags found to filter with", None);
                        } else {
                            app.close_popup();
                            app.set_popup(PopUp::FilterByTag);
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
                        app.set_popup(PopUp::ChangeDateFormatPopup);
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
                        app.set_view(View::Login);
                        app.close_popup();
                    }
                    CommandPaletteActions::Logout => {
                        app.dispatch(IoEvent::Logout).await;
                        app.close_popup();
                    }
                    CommandPaletteActions::SignUp => {
                        app.set_view(View::SignUp);
                        app.close_popup();
                    }
                    CommandPaletteActions::ResetPassword => {
                        app.set_view(View::ResetPassword);
                        app.close_popup();
                    }
                    CommandPaletteActions::SyncLocalData => {
                        app.dispatch(IoEvent::SyncLocalData).await;
                        app.close_popup();
                    }
                    CommandPaletteActions::LoadASaveCloud => {
                        if app.state.user_login_data.auth_token.is_some() {
                            app.set_view(View::LoadCloudSave);
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
        if app.state.z_stack.last() != Some(&PopUp::CustomHexColorPromptFG)
            || app.state.z_stack.last() != Some(&PopUp::CustomHexColorPromptBG)
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
        if let Some(PopUp::CommandPalette) = app.state.z_stack.last() {
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
    ChangeView,
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
            Self::ChangeView => write!(f, "Change View"),
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
