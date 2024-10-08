use crate::{
    app::{
        actions::Action,
        handle_exit,
        kanban::{Board, Boards, Card, CardPriority, CardStatus, Cards},
        state::{AppStatus, Focus, KeyBindings, PathCheckState},
        ActionHistory, App, AppConfig, AppReturn, ConfigEnum, DateTimeFormat, MainMenuItem,
    },
    constants::{
        DEFAULT_TOAST_DURATION, FIELD_NOT_SET, IO_EVENT_WAIT_TIME, MOUSE_OUT_OF_BOUNDS_COORDINATES,
    },
    inputs::{key::Key, mouse::Mouse},
    io::{
        data_handler::{get_config, save_theme, write_config},
        io_handler::refresh_visible_boards_and_cards,
        IoEvent,
    },
    ui::{
        text_box::TextBox,
        theme::{Theme, ThemeEnum},
        widgets::{
            command_palette::CommandPaletteWidget,
            toast::{Toast, ToastType},
        },
        PopUp, TextColorOptions, TextModifierOptions, View,
    },
    util::{date_format_converter, date_format_finder, parse_hex_to_rgb},
};
use chrono::NaiveDateTime;
use linked_hash_map::LinkedHashMap;
use log::{debug, error, info, warn};
use ratatui::{style::Color, widgets::ListState};
use std::{fs, path::Path, str::FromStr, time::Duration};
use strum::IntoEnumIterator;

pub fn go_right(app: &mut App) {
    let current_visible_boards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
        app.visible_boards_and_cards.clone();
    let boards: &Boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    let current_board_id = app.state.current_board_id;
    if boards.is_empty() {
        error!("Cannot go right: no boards found");
        app.send_error_toast("Cannot go right: no boards found", None);
        return;
    }
    let current_board_id = if let Some(current_board_id) = current_board_id {
        current_board_id
    } else {
        app.state.current_board_id = boards.get_first_board_id();
        app.state.current_board_id.unwrap()
    };

    match current_visible_boards
        .iter()
        .position(|(board_id, _)| *board_id == current_board_id)
    {
        Some(current_board_index) => {
            if current_board_index == current_visible_boards.len() - 1 {
                match boards.get_board_index(current_board_id) {
                    Some(current_board_index_in_all_boards) => {
                        if current_board_index_in_all_boards == (boards.len() - 1) {
                            app.send_error_toast(
                                "Cannot go right: Already at the last board",
                                None,
                            );
                            return;
                        }
                        if let Some(next_board) =
                            boards.get_board_with_index(current_board_index_in_all_boards + 1)
                        {
                            let next_board_card_ids = next_board.cards.get_all_card_ids();
                            app.visible_boards_and_cards
                                .insert(next_board.id, next_board_card_ids.clone());

                            if let Some((&first_board_id, _)) =
                                app.visible_boards_and_cards.iter().next()
                            {
                                app.visible_boards_and_cards.remove(&first_board_id);
                            }

                            app.state.current_board_id = Some(next_board.id);
                            app.state.current_card_id = next_board_card_ids.first().copied();
                        }
                    }
                    None => {
                        debug!("Cannot go right: current board not found");
                        app.send_error_toast("Cannot go right: Something went wrong", None);
                    }
                }
            } else if let Some((next_board_id, _)) =
                current_visible_boards.iter().nth(current_board_index + 1)
            {
                app.state.current_board_id = Some(*next_board_id);
                if let Some((_, cards)) = current_visible_boards
                    .iter()
                    .find(|(board_id, _)| *board_id == next_board_id)
                {
                    app.state.current_card_id = if cards.is_empty() {
                        None
                    } else {
                        Some(cards[0])
                    };
                }
            }
        }
        None => {
            debug!("Cannot go right: current board not found, trying to assign to the first board");
            if current_visible_boards.is_empty() {
                debug!("Cannot go right: current board not found, no visible boards found");
                app.send_error_toast("Cannot go right: Something went wrong", None);
            } else {
                app.state.current_board_id = Some(*current_visible_boards.keys().next().unwrap());
            }
        }
    }
}

pub fn go_left(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let boards: &Boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    let current_board_id = app.state.current_board_id;
    if boards.is_empty() {
        error!("Cannot go left: no boards");
        app.send_error_toast("Cannot go left: no boards", None);
        return;
    }
    let current_board_id = if let Some(current_board_id) = current_board_id {
        current_board_id
    } else {
        app.state.current_board_id = boards.get_first_board_id();
        app.state.current_board_id.unwrap()
    };

    match current_visible_boards
        .iter()
        .position(|(board_id, _)| *board_id == current_board_id)
    {
        Some(current_board_index) => {
            if current_board_index == 0 {
                match boards.get_board_index(current_board_id) {
                    Some(current_board_index_in_all_boards) => {
                        if current_board_index_in_all_boards == 0 {
                            app.send_error_toast(
                                "Cannot go left: Already at the first board",
                                None,
                            );
                            return;
                        }
                        if let Some(previous_board) =
                            boards.get_board_with_index(current_board_index_in_all_boards - 1)
                        {
                            let previous_board_card_ids = previous_board.cards.get_all_card_ids();
                            let mut new_visible_boards_and_cards: LinkedHashMap<
                                (u64, u64),
                                Vec<(u64, u64)>,
                            > = LinkedHashMap::new();
                            new_visible_boards_and_cards
                                .insert(previous_board.id, previous_board_card_ids.clone());
                            for (board_id, card_ids) in current_visible_boards
                                .iter()
                                .take(current_visible_boards.len() - 1)
                            {
                                new_visible_boards_and_cards.insert(*board_id, card_ids.clone());
                            }
                            app.visible_boards_and_cards = new_visible_boards_and_cards;
                            app.state.current_board_id = Some(previous_board.id);
                            app.state.current_card_id = if previous_board_card_ids.is_empty() {
                                None
                            } else {
                                Some(previous_board_card_ids[0])
                            };
                        }
                    }
                    None => {
                        debug!("Cannot go left: current board not found");
                        app.send_error_toast("Cannot go left: Something went wrong", None);
                    }
                }
            } else if let Some((previous_board_id, _)) =
                current_visible_boards.iter().nth(current_board_index - 1)
            {
                app.state.current_board_id = Some(*previous_board_id);
                if let Some((_, cards)) = current_visible_boards
                    .iter()
                    .find(|(board_id, _)| *board_id == previous_board_id)
                {
                    app.state.current_card_id = if cards.is_empty() {
                        None
                    } else {
                        Some(cards[0])
                    };
                }
            }
        }
        None => {
            debug!("Cannot go left: current board not found, trying to assign to the first board");
            if current_visible_boards.is_empty() {
                debug!("Cannot go left: current board not found, no visible boards found");
                app.send_error_toast("Cannot go left: Something went wrong", None);
            } else {
                app.state.current_board_id = Some(*current_visible_boards.keys().next().unwrap());
            }
        }
    }
}

pub fn go_up(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;
    let boards: &Boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    if current_visible_boards.is_empty() {
        return;
    }
    let current_board_id = if let Some(current_board_id) = current_board_id {
        current_board_id
    } else {
        app.state.current_board_id = boards.get_first_board_id();
        app.state.current_board_id.unwrap()
    };

    let current_card_id = if let Some(current_card_id) = current_card_id {
        current_card_id
    } else {
        match boards.get_board_with_id(current_board_id) {
            Some(current_board) => {
                if current_board.cards.is_empty() {
                    debug!("Cannot go up: current board has no cards");
                    app.send_error_toast("Cannot go up: current board has no cards", None);
                    return;
                }
                current_board.cards.get_first_card_id().unwrap()
            }
            None => {
                debug!("Cannot go up: current board not found");
                app.send_error_toast("Cannot go up: Something went wrong", None);
                return;
            }
        }
    };

    if let Some(current_card_index) = current_visible_boards
        .iter()
        .find(|(board_id, _)| **board_id == current_board_id)
        .unwrap()
        .1
        .iter()
        .position(|card_id| *card_id == current_card_id)
    {
        if current_card_index == 0 {
            if let Some(current_card_index_in_all_cards) = boards
                .get_board_with_id(current_board_id)
                .unwrap()
                .cards
                .get_card_index(current_card_id)
            {
                if current_card_index_in_all_cards == 0 {
                    app.send_error_toast("Cannot go up: Already at the first card", None);
                    return;
                }
                if let Some(board) = boards.get_board_with_id(current_board_id) {
                    if let Some(previous_card) = board
                        .cards
                        .get_card_with_index(current_card_index_in_all_cards - 1)
                    {
                        let previous_card_id = previous_card.id;
                        let previous_cards = board.cards.get_cards_with_range(
                            current_card_index_in_all_cards - 1,
                            current_card_index_in_all_cards - 1
                                + app.config.no_of_cards_to_show as usize,
                        );
                        app.visible_boards_and_cards
                            .entry(current_board_id)
                            .and_modify(|cards| *cards = previous_cards.get_all_card_ids());
                        app.state.current_card_id = Some(previous_card_id);
                    } else {
                        debug!("Cannot go up: Already at the first card");
                        app.send_error_toast("Cannot go up: Already at the first card", None);
                    }
                }
            } else {
                debug!("Cannot go up: current card not found");
                app.send_error_toast("Cannot go up: Something went wrong", None);
            }
        } else if let Some(previous_card_id) = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == current_board_id)
            .unwrap()
            .1
            .get(current_card_index - 1)
        {
            app.state.current_card_id = Some(*previous_card_id);
        } else {
            debug!("Cannot go up: previous card not found");
            app.send_error_toast("Cannot go up: Something went wrong", None);
        }
    } else {
        debug!("Cannot go up: current card not found");
        app.send_error_toast("Cannot go up: Something went wrong", None);
    }
}

pub fn go_down(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;
    let boards: &Boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    if current_visible_boards.is_empty() {
        return;
    }
    let current_board_id = if let Some(current_board_id) = current_board_id {
        current_board_id
    } else {
        app.state.current_board_id = boards.get_first_board_id();
        app.state.current_board_id.unwrap()
    };
    let current_card_id = if let Some(current_card_id) = current_card_id {
        current_card_id
    } else {
        match boards.get_board_with_id(current_board_id) {
            Some(current_board) => {
                if current_board.cards.is_empty() {
                    debug!("Cannot go down: current board has no cards");
                    app.send_error_toast("Cannot go down: Current board has no cards", None);
                    return;
                }
                current_board.cards.get_first_card_id().unwrap()
            }
            None => {
                debug!("Cannot go down: current board not found, trying to get the first board");
                if current_visible_boards.is_empty() {
                    debug!("Cannot go down: current board not found, tried to get the first board, but failed");
                    app.send_error_toast("Cannot go down: Something went wrong", None);
                    return;
                } else {
                    app.state.current_board_id =
                        Some(*current_visible_boards.keys().next().unwrap());
                    app.state.current_card_id =
                        Some(current_visible_boards.values().next().unwrap()[0]);
                    return;
                }
            }
        }
    };
    if let Some(current_card_index) = current_visible_boards
        .iter()
        .find(|(board_id, _)| **board_id == current_board_id)
        .unwrap()
        .1
        .iter()
        .position(|card_id| *card_id == current_card_id)
    {
        if current_card_index == app.config.no_of_cards_to_show as usize - 1 {
            if let Some(current_card_index_in_all_cards) = boards
                .get_board_with_id(current_board_id)
                .unwrap()
                .cards
                .get_card_index(current_card_id)
            {
                if let Some(board) = boards.get_board_with_id(current_board_id) {
                    if let Some(next_card) = board
                        .cards
                        .get_card_with_index(current_card_index_in_all_cards + 1)
                    {
                        let next_card_id = next_card.id;
                        let start_index = current_card_index_in_all_cards + 1;
                        let end_index = current_card_index_in_all_cards
                            + 1
                            + app.config.no_of_cards_to_show as usize;
                        let end_index = if end_index > board.cards.len() {
                            board.cards.len()
                        } else {
                            end_index
                        };
                        let next_card_ids = board
                            .cards
                            .get_cards_with_range(start_index, end_index)
                            .get_all_card_ids();
                        let next_card_ids =
                            if next_card_ids.len() < app.config.no_of_cards_to_show as usize {
                                let mut next_card_ids = next_card_ids;
                                let mut start_index = start_index;
                                while next_card_ids.len() < app.config.no_of_cards_to_show as usize
                                    && start_index > 0
                                {
                                    start_index -= 1;
                                    next_card_ids.insert(
                                        0,
                                        board.cards.get_card_with_index(start_index).unwrap().id,
                                    );
                                }
                                next_card_ids
                            } else {
                                next_card_ids
                            };
                        app.visible_boards_and_cards
                            .entry(current_board_id)
                            .and_modify(|cards| *cards = next_card_ids);
                        app.state.current_card_id = Some(next_card_id);
                    } else {
                        debug!("Cannot go down: Already at the last card");
                        app.send_error_toast("Cannot go down: Already at the last card", None);
                    }
                }
            } else {
                debug!("Cannot go down: current card not found");
                app.send_error_toast("Cannot go down: Something went wrong", None);
            }
        } else if let Some(next_card_id) = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == current_board_id)
            .unwrap()
            .1
            .get(current_card_index + 1)
        {
            app.state.current_card_id = Some(*next_card_id);
        } else {
            debug!("Cannot go down: Already at the last card");
            app.send_error_toast("Cannot go down: Already at the last card", None);
        }
    } else {
        debug!("Cannot go down: current card not found");
        app.send_error_toast("Cannot go down: Something went wrong", None);
    }
}

pub fn prepare_config_for_new_app(theme: Theme) -> (AppConfig, Vec<&'static str>, Vec<Toast>) {
    let mut toasts = vec![];
    let mut errors = vec![];
    match get_config(false) {
        Ok(config) => (config, errors, toasts),
        Err(config_error_msg) => {
            if config_error_msg.contains("Overlapped keybindings found") {
                error!("KeyBindings overlap detected. Please check your config file and fix the keybindings. Using default keybindings for now.");
                errors.push("KeyBindings overlap detected. Please check your config file and fix the keybindings. Using default keybindings for now.");
                toasts.push(Toast::new(
                    config_error_msg,
                    Duration::from_secs(DEFAULT_TOAST_DURATION) * 3,
                    ToastType::Error,
                    theme.clone(),
                ));
                toasts.push(Toast::new("Please check your config file and fix the keybindings. Using default keybindings for now.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Warning, theme.clone()));
                match get_config(true) {
                    Ok(mut new_config) => {
                        new_config.keybindings = KeyBindings::default();
                        (new_config, errors, toasts)
                    }
                    Err(new_config_error) => {
                        error!("Unable to fix keybindings. Please check your config file. Using default config for now.");
                        errors.push("Unable to fix keybindings. Please check your config file. Using default config for now.");
                        toasts.push(Toast::new(
                            new_config_error,
                            Duration::from_secs(DEFAULT_TOAST_DURATION) * 3,
                            ToastType::Error,
                            theme.clone(),
                        ));
                        toasts.push(Toast::new(
                            "Using default config for now.".to_owned(),
                            Duration::from_secs(DEFAULT_TOAST_DURATION),
                            ToastType::Warning,
                            theme,
                        ));
                        (AppConfig::default(), errors, toasts)
                    }
                }
            } else {
                toasts.push(Toast::new(
                    config_error_msg,
                    Duration::from_secs(DEFAULT_TOAST_DURATION),
                    ToastType::Error,
                    theme.clone(),
                ));
                toasts.push(Toast::new(
                    "Using default config for now.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION),
                    ToastType::Info,
                    theme,
                ));
                (AppConfig::default(), errors, toasts)
            }
        }
    }
}

pub async fn handle_user_input_mode(app: &mut App<'_>, key: Key) -> AppReturn {
    reset_mouse(app);
    if key == Key::Esc {
        match app.state.focus {
            Focus::NewBoardName => app.state.text_buffers.board_name.reset(),
            Focus::NewBoardDescription => app.state.text_buffers.board_description.reset(),
            Focus::CardName => app.state.text_buffers.card_name.reset(),
            Focus::CardDescription => app.state.text_buffers.card_description.reset(),
            Focus::EmailIDField => app.state.text_buffers.email_id.reset(),
            Focus::PasswordField => app.state.text_buffers.password.reset(),
            Focus::ConfirmPasswordField => app.state.text_buffers.confirm_password.reset(),
            Focus::ResetPasswordLinkField => app.state.text_buffers.reset_password_link.reset(),
            Focus::CommandPaletteCommand
            | Focus::CommandPaletteBoard
            | Focus::CommandPaletteCard => {
                app.widgets.command_palette.reset(&mut app.state);
            }
            Focus::DTPCalender
            | Focus::DTPMonth
            | Focus::DTPYear
            | Focus::DTPToggleTimePicker
            | Focus::DTPHour
            | Focus::DTPMinute
            | Focus::DTPSecond => {
                app.widgets.date_time_picker.close_date_picker();
            }
            _ => {
                debug!(
                    "No user input handler found for focus: {:?} key Esc",
                    app.state.focus
                );
            }
        }
        if let Some(popup) = app.state.z_stack.checked_control_last() {
            match popup {
                PopUp::CommandPalette => {
                    app.close_popup();
                    if app.widgets.command_palette.already_in_user_input_mode {
                        app.widgets.command_palette.already_in_user_input_mode = false;
                        if let Some(last_focus) = app.widgets.command_palette.last_focus {
                            app.state.set_focus(last_focus);
                        }
                        return AppReturn::Continue;
                    }
                }
                PopUp::ConfirmDiscardCardChanges => {
                    app.close_popup();
                }
                PopUp::ViewCard => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup(PopUp::ConfirmDiscardCardChanges);
                    }
                }
                PopUp::CardPrioritySelector => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup(PopUp::ConfirmDiscardCardChanges);
                    } else {
                        app.close_popup();
                    }
                }
                PopUp::CardStatusSelector => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup(PopUp::ConfirmDiscardCardChanges);
                    } else {
                        app.close_popup();
                    }
                }
                PopUp::CustomHexColorPromptBG | PopUp::CustomHexColorPromptFG => {
                    app.close_popup();
                }
                _ => {}
            }
        }
        app.state.app_status = AppStatus::Initialized;
        app.state.path_check_state = PathCheckState::default();
        info!("Exiting user input mode");
    } else if app.config.keybindings.toggle_command_palette.contains(&key) {
        app.widgets.command_palette.already_in_user_input_mode = true;
        app.widgets.command_palette.last_focus = Some(app.state.focus);
        app.set_popup(PopUp::CommandPalette);
    } else if app.config.keybindings.stop_user_input.contains(&key)
        && !(app.state.z_stack.last() == Some(&PopUp::CommandPalette))
    {
        app.state.app_status = AppStatus::Initialized;
        app.state.path_check_state = PathCheckState::default();
        info!("Exiting user input mode");
    } else {
        // Special Handling for Command Palette

        if app.config.keybindings.toggle_command_palette.contains(&key) {
            app.state.app_status = AppStatus::Initialized;
            app.close_popup();
            return AppReturn::Continue;
        }
        if app.state.z_stack.last() == Some(&PopUp::CommandPalette) {
            let stop_input_mode_keys = &app.config.keybindings.stop_user_input;
            match key {
                // TODO: See if action should be used here instead of key
                Key::Up => {
                    match app.state.focus {
                        Focus::CommandPaletteCommand => {
                            app.command_palette_command_search_prv();
                        }
                        Focus::CommandPaletteCard => {
                            app.command_palette_card_search_prv();
                        }
                        Focus::CommandPaletteBoard => {
                            app.command_palette_board_search_prv();
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                Key::Down => {
                    match app.state.focus {
                        Focus::CommandPaletteCommand => {
                            app.command_palette_command_search_next();
                        }
                        Focus::CommandPaletteCard => {
                            app.command_palette_card_search_next();
                        }
                        Focus::CommandPaletteBoard => {
                            app.command_palette_board_search_next();
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                Key::Enter => {
                    match app.state.focus {
                        Focus::CommandPaletteCommand => {
                            return CommandPaletteWidget::handle_command(app).await
                        }
                        Focus::CommandPaletteCard => {
                            app.close_popup();
                            handle_command_palette_card_selection(app);
                            app.widgets.command_palette.reset(&mut app.state);
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                        Focus::CommandPaletteBoard => {
                            app.close_popup();
                            handle_command_palette_board_selection(app);
                            app.widgets.command_palette.reset(&mut app.state);
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                        Focus::SubmitButton => {
                            if app.state.card_being_edited.is_some() {
                                return handle_edit_card_submit(app);
                            } else {
                                debug!("Submit button pressed in user input mode, dont know what to do");
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    if stop_input_mode_keys.contains(&key) {
                        app.close_popup();
                        app.state.app_status = AppStatus::Initialized;
                        return AppReturn::Continue;
                    } else if app.config.keybindings.next_focus.contains(&key) {
                        handle_next_focus(app);
                        return AppReturn::Continue;
                    } else if app.config.keybindings.prv_focus.contains(&key) {
                        handle_prv_focus(app);
                        return AppReturn::Continue;
                    }
                }
            };
        }

        // Handle user input for anything other than the command palette
        match app.state.focus {
            Focus::NewBoardName => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.board_name.input(key);
                }
            }
            Focus::NewBoardDescription => {
                app.state.text_buffers.board_description.input(key);
            }
            Focus::CardName => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.card_name.input(key);
                }
            }
            Focus::CardDescription => {
                app.state.text_buffers.card_description.input(key);
            }
            Focus::CardDueDate => {
                if app.state.card_being_edited.is_none()
                    && app.state.z_stack.last() == Some(&PopUp::ViewCard)
                {
                    handle_edit_new_card(app);
                }
                app.set_popup(PopUp::DateTimePicker);
            }
            Focus::CardPriority => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else if key == Key::Enter {
                    app.set_popup(PopUp::CardPrioritySelector);
                }
            }
            Focus::CardStatus => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else if key == Key::Enter {
                    app.set_popup(PopUp::CardStatusSelector);
                }
            }
            Focus::CardTags => {
                if let Some((_, current_card)) = &mut app.state.card_being_edited {
                    match key {
                        Key::ShiftRight => {
                            if let Some(current_selected) =
                                app.state.app_list_states.card_view_tag_list.selected()
                            {
                                if !current_card.tags.is_empty() {
                                    let max = current_card.tags.len();
                                    if current_selected < max - 1 {
                                        app.state
                                            .app_list_states
                                            .card_view_tag_list
                                            .select(Some(current_selected + 1));
                                    } else if current_selected != max - 1 {
                                        app.state.app_list_states.card_view_tag_list.select(None);
                                    }
                                }
                            } else if !current_card.tags.is_empty() {
                                app.state.app_list_states.card_view_tag_list.select(Some(0));
                            }
                        }
                        Key::ShiftLeft => {
                            if let Some(current_selected) =
                                app.state.app_list_states.card_view_tag_list.selected()
                            {
                                if !current_card.tags.is_empty() {
                                    let max = current_card.tags.len();
                                    if current_selected > 0 {
                                        app.state
                                            .app_list_states
                                            .card_view_tag_list
                                            .select(Some(current_selected - 1));
                                    } else if current_selected != 0 {
                                        app.state
                                            .app_list_states
                                            .card_view_tag_list
                                            .select(Some(max - 1));
                                    }
                                }
                            } else if !current_card.tags.is_empty() {
                                app.state.app_list_states.card_view_tag_list.select(Some(0));
                            }
                        }
                        Key::Enter => {
                            if let Some(selected_index) =
                                app.state.app_list_states.tag_picker.selected()
                            {
                                if let Some(tag) =
                                    app.widgets.tag_picker.available_tags.get(selected_index)
                                {
                                    if !current_card.tags.contains(tag) {
                                        if let Some(insert_index) =
                                            app.state.app_list_states.card_view_tag_list.selected()
                                        {
                                            if let Some(text_box) = app
                                                .state
                                                .text_buffers
                                                .card_tags
                                                .get_mut(insert_index)
                                            {
                                                text_box.reset();
                                                text_box.insert_str(tag);
                                                current_card.tags[insert_index] = tag.to_owned();
                                            }
                                        } else {
                                            current_card.tags.push(tag.to_owned());
                                            app.state.app_list_states.card_view_tag_list.select(
                                                Some(current_card.tags.len().saturating_sub(1)),
                                            );
                                        }
                                        app.state
                                            .text_buffers
                                            .prepare_tags_and_comments_for_card(current_card);
                                        return AppReturn::Continue;
                                    }
                                }
                            }
                            if let Some(insert_index) =
                                app.state.app_list_states.card_view_tag_list.selected()
                            {
                                if insert_index < current_card.tags.len() {
                                    current_card.tags.insert(insert_index + 1, "".to_owned());
                                } else {
                                    current_card.tags.push("".to_owned());
                                }
                                app.state
                                    .app_list_states
                                    .card_view_tag_list
                                    .select(Some(insert_index + 1));
                            } else {
                                current_card.tags.push("".to_owned());
                                app.state
                                    .app_list_states
                                    .card_view_tag_list
                                    .select(Some(current_card.tags.len().saturating_sub(1)));
                            }
                            app.state
                                .text_buffers
                                .prepare_tags_and_comments_for_card(current_card);
                        }
                        Key::Delete => {
                            if current_card.tags.is_empty() {
                                app.send_error_toast("No tags to delete", None);
                            } else if let Some(delete_index) =
                                app.state.app_list_states.card_view_tag_list.selected()
                            {
                                current_card.tags.remove(delete_index);
                                app.state
                                    .text_buffers
                                    .prepare_tags_and_comments_for_card(current_card);
                                if delete_index != 0 {
                                    app.state
                                        .app_list_states
                                        .card_view_tag_list
                                        .select(Some(delete_index - 1));
                                }
                            }
                        }
                        Key::Up => {
                            if !app.widgets.tag_picker.available_tags.is_empty() {
                                if let Some(selected_index) =
                                    app.state.app_list_states.tag_picker.selected()
                                {
                                    if selected_index == 0 {
                                        app.state.app_list_states.tag_picker.select(Some(
                                            app.widgets
                                                .tag_picker
                                                .available_tags
                                                .len()
                                                .saturating_sub(1),
                                        ));
                                    } else if selected_index
                                        > app.widgets.tag_picker.available_tags.len()
                                    {
                                        app.state.app_list_states.tag_picker.select(Some(0));
                                    } else {
                                        app.state
                                            .app_list_states
                                            .tag_picker
                                            .select(Some(selected_index - 1));
                                    }
                                } else {
                                    app.state.app_list_states.tag_picker.select(Some(0));
                                }
                            }
                        }
                        Key::Down => {
                            if !app.widgets.tag_picker.available_tags.is_empty() {
                                if let Some(selected_index) =
                                    app.state.app_list_states.tag_picker.selected()
                                {
                                    if selected_index
                                        >= app
                                            .widgets
                                            .tag_picker
                                            .available_tags
                                            .len()
                                            .saturating_sub(1)
                                    {
                                        app.state.app_list_states.tag_picker.select(Some(0));
                                    } else {
                                        app.state
                                            .app_list_states
                                            .tag_picker
                                            .select(Some(selected_index + 1));
                                    }
                                } else {
                                    app.state.app_list_states.tag_picker.select(Some(0));
                                }
                            }
                        }
                        _ if app.config.keybindings.next_focus.contains(&key) => {
                            handle_next_focus(app)
                        }
                        _ if app.config.keybindings.prv_focus.contains(&key) => {
                            handle_prv_focus(app)
                        }
                        _ => {
                            if let Some(selected_tag_index) =
                                app.state.app_list_states.card_view_tag_list.selected()
                            {
                                if let Some(current_tag_text_box) =
                                    app.state.text_buffers.card_tags.get_mut(selected_tag_index)
                                {
                                    current_tag_text_box.input(key);
                                    current_card.tags[selected_tag_index] =
                                        current_tag_text_box.get_joined_lines();
                                }
                            } else {
                                app.send_warning_toast(
                                    &format!(
                                        "No Tag selected to edit, use {} or {}",
                                        Key::ShiftRight,
                                        Key::ShiftLeft
                                    ),
                                    None,
                                );
                            }
                        }
                    }
                } else {
                    return AppReturn::Continue;
                }
            }
            Focus::CardComments => {
                if let Some((_, current_card)) = &mut app.state.card_being_edited {
                    let current_selected = app
                        .state
                        .app_list_states
                        .card_view_comment_list
                        .selected()
                        .unwrap_or(0);
                    match key {
                        Key::ShiftRight => {
                            if !current_card.comments.is_empty() {
                                let max = current_card.comments.len();
                                if current_selected < max - 1 {
                                    app.state
                                        .app_list_states
                                        .card_view_comment_list
                                        .select(Some(current_selected + 1));
                                }
                            }
                        }
                        Key::ShiftLeft => {
                            if !current_card.comments.is_empty() {
                                app.state
                                    .app_list_states
                                    .card_view_comment_list
                                    .select(Some(current_selected.saturating_sub(1)));
                            }
                        }
                        Key::Enter => {
                            if let Some(insert_index) =
                                app.state.app_list_states.card_view_comment_list.selected()
                            {
                                current_card
                                    .comments
                                    .insert(insert_index + 1, "".to_owned());
                                app.state
                                    .text_buffers
                                    .prepare_tags_and_comments_for_card(current_card);
                                app.state
                                    .app_list_states
                                    .card_view_comment_list
                                    .select(Some(insert_index + 1));
                            } else {
                                current_card.comments.push("".to_owned());
                                app.state
                                    .text_buffers
                                    .prepare_tags_and_comments_for_card(current_card);
                                app.state
                                    .app_list_states
                                    .card_view_comment_list
                                    .select(Some(current_card.comments.len().saturating_sub(1)));
                            }
                        }
                        Key::Delete => {
                            if current_card.comments.is_empty() {
                                app.send_error_toast("No comments to delete", None);
                            } else if let Some(delete_index) =
                                app.state.app_list_states.card_view_comment_list.selected()
                            {
                                current_card.comments.remove(delete_index);
                                app.state
                                    .text_buffers
                                    .prepare_tags_and_comments_for_card(current_card);
                                if delete_index != 0 {
                                    app.state
                                        .app_list_states
                                        .card_view_comment_list
                                        .select(Some(delete_index - 1));
                                }
                            }
                        }
                        _ if app.config.keybindings.next_focus.contains(&key) => {
                            handle_next_focus(app)
                        }
                        _ if app.config.keybindings.prv_focus.contains(&key) => {
                            handle_prv_focus(app)
                        }
                        _ => {
                            if let Some(selected_comment_index) =
                                app.state.app_list_states.card_view_comment_list.selected()
                            {
                                if let Some(current_comment_text_box) = app
                                    .state
                                    .text_buffers
                                    .card_comments
                                    .get_mut(selected_comment_index)
                                {
                                    current_comment_text_box.input(key);
                                    current_card.comments[selected_comment_index] =
                                        current_comment_text_box.get_joined_lines();
                                }
                            } else {
                                app.send_warning_toast(
                                    &format!(
                                        "No Comment selected to edit, use {} or {}",
                                        Key::ShiftRight,
                                        Key::ShiftLeft
                                    ),
                                    None,
                                );
                            }
                        }
                    }
                } else {
                    return AppReturn::Continue;
                }
            }
            Focus::EmailIDField => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.email_id.input(key);
                }
            }
            Focus::PasswordField => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.password.input(key);
                }
            }
            Focus::ConfirmPasswordField => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.confirm_password.input(key);
                }
            }
            Focus::ResetPasswordLinkField => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.reset_password_link.input(key);
                }
            }
            Focus::CommandPaletteCommand
            | Focus::CommandPaletteBoard
            | Focus::CommandPaletteCard => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.command_palette.input(key);
                }
            }
            Focus::EditGeneralConfigPopup => {
                if app.state.current_view == View::CreateTheme {
                    // Special handling for Create theme as only it uses the general config popup other than config changes
                    app.state.text_buffers.general_config.input(key);
                } else {
                    match key {
                        Key::Right | Key::Tab => {
                            if let Some(potential_completion) =
                                &app.state.path_check_state.potential_completion
                            {
                                app.state
                                    .text_buffers
                                    .general_config
                                    .insert_str(potential_completion);
                            }
                        }
                        Key::Char('%') => {
                            let new_path = app.state.text_buffers.general_config.get_joined_lines();
                            // if path does not start with os sep
                            if !new_path.starts_with(std::path::MAIN_SEPARATOR) {
                                app.send_error_toast(
                                    &format!(
                                        "Path should start with '{}'",
                                        std::path::MAIN_SEPARATOR
                                    ),
                                    None,
                                );
                                return AppReturn::Continue;
                            }
                            // try to create a directory
                            let path = Path::new(&new_path);
                            if path.exists() {
                                app.send_warning_toast("Path already exists", None);
                            } else {
                                match fs::create_dir_all(path) {
                                    Ok(_) => {
                                        app.send_info_toast("Directory created", None);
                                        app.state.path_check_state.potential_completion = None;
                                        app.state.path_check_state.recheck_required = true;
                                    }
                                    Err(e) => {
                                        app.send_error_toast(
                                            &format!("Error creating directory: {}", e),
                                            None,
                                        );
                                    }
                                }
                            }
                            return AppReturn::Continue;
                        }
                        _ => {
                            app.state.text_buffers.general_config.input(key);
                        }
                    }
                }
            }
            Focus::SubmitButton => {
                match key {
                    Key::Enter => {
                        if app.state.z_stack.last() == Some(&PopUp::ViewCard) {
                            if app.state.card_being_edited.is_some() {
                                return handle_edit_card_submit(app);
                            }
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        } else {
                            debug!("Dont know what to do with Submit button in user input mode for popup: {:?}", app.state.z_stack.last());
                        }
                        match app.state.current_view {
                            View::NewCard => {
                                handle_new_card_action(app);
                            }
                            View::NewBoard => {
                                handle_new_board_action(app);
                            }
                            _ => {
                                debug!("Dont know what to do with Submit button in user input mode for view: {:?}", app.state.current_view);
                            }
                        }
                        app.state.app_status = AppStatus::Initialized;
                    }
                    _ if app.config.keybindings.next_focus.contains(&key) => handle_next_focus(app),
                    _ if app.config.keybindings.prv_focus.contains(&key) => handle_prv_focus(app),
                    _ => {
                        debug!("Dont know what to do with key {:?} in user input mode for Submit button", key);
                    }
                }
            }
            Focus::ChangeCardStatusPopup => match key {
                Key::Up => app.select_card_status_prv(),
                Key::Down => app.select_card_status_next(),
                Key::Enter => {
                    handle_change_card_status(app, None);
                }
                _ => {}
            },
            Focus::ChangeCardPriorityPopup => match key {
                Key::Up => app.select_card_priority_prv(),
                Key::Down => app.select_card_priority_next(),
                Key::Enter => {
                    handle_change_card_priority(app, None);
                }
                _ => {}
            },
            Focus::TextInput => {
                let accept_keys = &app.config.keybindings.accept;
                if accept_keys.contains(&key) {
                    match app.state.z_stack.last() {
                        Some(PopUp::CustomHexColorPromptFG) => {
                            return handle_custom_hex_color_prompt(app, true)
                        }
                        Some(PopUp::CustomHexColorPromptBG) => {
                            return handle_custom_hex_color_prompt(app, false)
                        }
                        _ => {
                            debug!(
                                "TextInput is not used in the current popup: {:?}",
                                app.state.z_stack.last()
                            );
                        }
                    }
                } else {
                    match app.state.z_stack.last() {
                        Some(PopUp::CustomHexColorPromptFG) => {
                            app.state.text_buffers.theme_editor_fg_hex.input(key);
                        }
                        Some(PopUp::CustomHexColorPromptBG) => {
                            app.state.text_buffers.theme_editor_bg_hex.input(key);
                        }
                        _ => {
                            debug!(
                                "No user input handler found for focus: {:?}",
                                app.state.focus
                            )
                        }
                    }
                }
            }
            Focus::DTPCalender
            | Focus::DTPMonth
            | Focus::DTPYear
            | Focus::DTPToggleTimePicker
            | Focus::DTPHour
            | Focus::DTPMinute
            | Focus::DTPSecond => {
                handle_date_time_picker_action(app, Some(key), None);
            }
            Focus::NoFocus => {
                if let Some(PopUp::DateTimePicker) = app.state.z_stack.last() {
                    app.state.set_focus(Focus::DTPCalender);
                }
            }
            _ => match key {
                _ if app.config.keybindings.next_focus.contains(&key) => handle_next_focus(app),
                _ if app.config.keybindings.prv_focus.contains(&key) => handle_prv_focus(app),
                _ => {
                    debug!(
                        "No user input handler found for focus: {:?}",
                        app.state.focus
                    );
                }
            },
        }
    }
    AppReturn::Continue
}

pub async fn handle_edit_keybinding_mode(app: &mut App<'_>, key: Key) -> AppReturn {
    // Handle escape key or other keys that should stop user input
    if matches!(key, Key::Esc) {
        app.state.app_status = AppStatus::Initialized;
        app.state.edited_keybinding = None; // Clear edited keybinding on exit
        info!("Exiting user Keybinding input mode");
        return AppReturn::Continue;
    } else if app.config.keybindings.stop_user_input.contains(&key) {
        app.state.app_status = AppStatus::Initialized;
        info!("Exiting user Keybinding input mode");
        return AppReturn::Continue;
    }

    // For any other key, add it to the current keybinding being edited
    match &mut app.state.edited_keybinding {
        Some(keybinding) => keybinding.push(key),
        None => app.state.edited_keybinding = Some(vec![key]),
    }

    AppReturn::Continue
}

pub async fn handle_general_actions(app: &mut App<'_>, key: Key) -> AppReturn {
    if let Some(action) = app.config.keybindings.key_to_action(&key) {
        match action {
            Action::Quit => handle_exit(app).await,
            Action::NextFocus => {
                handle_next_focus(app);
                AppReturn::Continue
            }
            Action::PrvFocus => {
                handle_prv_focus(app);
                AppReturn::Continue
            }
            Action::ResetUI => {
                let default_theme = app.config.default_theme.clone();
                for theme in app.all_themes.iter_mut() {
                    if theme.name == default_theme {
                        app.current_theme = theme.clone();
                    }
                }
                app.widgets.toast_widget.toasts = vec![];
                app.set_view(app.config.default_view);
                app.send_info_toast("UI reset, all toasts cleared", None);
                app.close_popup();
                refresh_visible_boards_and_cards(app);
                AppReturn::Continue
            }
            Action::OpenConfigMenu => {
                if matches!(app.state.current_view, View::ConfigMenu) {
                    handle_go_to_previous_view(app).await;
                } else {
                    app.set_view(View::ConfigMenu);
                }
                if !app.state.z_stack.is_empty() {
                    app.state.z_stack.clear();
                }
                AppReturn::Continue
            }
            Action::Up => {
                reset_mouse(app);
                if let Some(popup) = app.state.z_stack.last() {
                    match popup {
                        PopUp::ChangeView => app.select_default_view_prv(),
                        PopUp::CardStatusSelector => app.select_card_status_prv(),
                        PopUp::SelectDefaultView => app.select_default_view_prv(),
                        PopUp::ChangeTheme => app.select_change_theme_prv(),
                        PopUp::EditThemeStyle => {
                            if app.state.focus == Focus::StyleEditorFG {
                                app.select_edit_style_fg_prv();
                            } else if app.state.focus == Focus::StyleEditorBG {
                                app.select_edit_style_bg_prv();
                            } else if app.state.focus == Focus::StyleEditorModifier {
                                app.select_edit_style_modifier_prv();
                            }
                        }
                        PopUp::SaveThemePrompt => {
                            toggle_focus_between_submit_and_extra(app);
                        }
                        PopUp::ChangeDateFormatPopup => app.change_date_format_popup_prv(),
                        PopUp::FilterByTag => app.filter_by_tag_popup_prv(),
                        PopUp::ViewCard => {
                            if app.state.focus == Focus::CardDescription {
                                app.state.text_buffers.card_description.scroll((-1, 0));
                            }
                        }
                        PopUp::CardPrioritySelector => {
                            app.select_card_priority_prv();
                        }
                        PopUp::DateTimePicker => {
                            handle_date_time_picker_action(app, None, Some(action));
                        }
                        PopUp::TagPicker => {
                            app.tag_picker_prv();
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                match app.state.current_view {
                    View::ConfigMenu => {
                        if app.state.focus == Focus::ConfigTable {
                            app.config_prv();
                        } else {
                            let next_focus_key = app.get_first_next_focus_keybinding();
                            let prev_focus_key = app.get_first_prv_focus_keybinding();
                            app.send_warning_toast(&format!(
                                "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    View::MainMenu => {
                        if app.state.focus == Focus::MainMenu {
                            app.main_menu_prv();
                        } else if app.state.focus == Focus::Help {
                            app.help_prv();
                        } else if app.state.focus == Focus::Log {
                            app.log_prv();
                        } else {
                            let next_focus_key = app.get_first_next_focus_keybinding();
                            let prev_focus_key = app.get_first_prv_focus_keybinding();
                            app.send_warning_toast(&format!(
                                "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    View::LoadLocalSave => {
                        app.load_save_prv(false);
                        app.dispatch(IoEvent::LoadLocalPreview).await;
                    }
                    View::LoadCloudSave => {
                        app.load_save_prv(true);
                        app.dispatch(IoEvent::LoadCloudPreview).await;
                    }
                    View::EditKeybindings => {
                        app.edit_keybindings_prv();
                    }
                    View::CreateTheme => {
                        if app.state.focus == Focus::ThemeEditor {
                            app.select_create_theme_prv();
                        } else if app.state.focus == Focus::SubmitButton {
                            let next_focus_key = app.get_first_next_focus_keybinding();
                            let prev_focus_key = app.get_first_prv_focus_keybinding();
                            app.send_warning_toast(&format!(
                                "Move Focus to the theme editor with {} or {}, to select a style to edit",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    View::NewBoard => {
                        if app.state.focus == Focus::NewBoardDescription {
                            app.state.text_buffers.board_description.scroll((-1, 0))
                        }
                    }
                    View::NewCard => {
                        if app.state.focus == Focus::CardDescription {
                            app.state.text_buffers.card_description.scroll((-1, 0))
                        }
                    }
                    _ => {
                        if app.state.focus == Focus::Body
                            && View::views_with_kanban_board().contains(&app.state.current_view)
                        {
                            go_up(app);
                        } else if app.state.focus == Focus::Help {
                            app.help_prv();
                        } else if app.state.focus == Focus::Log {
                            app.log_prv();
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::Down => {
                reset_mouse(app);
                if let Some(popup) = app.state.z_stack.last() {
                    match popup {
                        PopUp::ChangeView => app.select_default_view_next(),
                        PopUp::CardStatusSelector => app.select_card_status_next(),
                        PopUp::SelectDefaultView => app.select_default_view_next(),
                        PopUp::ChangeTheme => app.select_change_theme_next(),
                        PopUp::EditThemeStyle => {
                            if app.state.focus == Focus::StyleEditorFG {
                                app.select_edit_style_fg_next();
                            } else if app.state.focus == Focus::StyleEditorBG {
                                app.select_edit_style_bg_next();
                            } else if app.state.focus == Focus::StyleEditorModifier {
                                app.select_edit_style_modifier_next();
                            }
                        }
                        PopUp::ChangeDateFormatPopup => app.change_date_format_popup_next(),
                        PopUp::FilterByTag => app.filter_by_tag_popup_next(),
                        PopUp::ViewCard => {
                            if app.state.focus == Focus::CardDescription {
                                app.state.text_buffers.card_description.scroll((1, 0))
                            }
                        }
                        PopUp::SaveThemePrompt => {
                            toggle_focus_between_submit_and_extra(app);
                        }
                        PopUp::CardPrioritySelector => {
                            app.select_card_priority_next();
                        }
                        PopUp::DateTimePicker => {
                            handle_date_time_picker_action(app, None, Some(action));
                        }
                        PopUp::TagPicker => {
                            app.tag_picker_next();
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                match app.state.current_view {
                    View::ConfigMenu => {
                        if app.state.focus == Focus::ConfigTable {
                            app.config_next();
                        } else {
                            let next_focus_key = app.get_first_next_focus_keybinding();
                            let prev_focus_key = app.get_first_prv_focus_keybinding();
                            app.send_warning_toast(&format!(
                                "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    View::MainMenu => {
                        if app.state.focus == Focus::MainMenu {
                            app.main_menu_next();
                        } else if app.state.focus == Focus::Help {
                            app.help_next();
                        } else if app.state.focus == Focus::Log {
                            app.log_next();
                        } else {
                            let next_focus_key = app.get_first_next_focus_keybinding();
                            let prev_focus_key = app.get_first_prv_focus_keybinding();
                            app.send_warning_toast(&format!(
                                "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    View::LoadLocalSave => {
                        app.load_save_next(false);
                        app.dispatch(IoEvent::LoadLocalPreview).await;
                    }
                    View::LoadCloudSave => {
                        app.load_save_next(true);
                        app.dispatch(IoEvent::LoadCloudPreview).await;
                    }
                    View::EditKeybindings => {
                        app.edit_keybindings_next();
                    }
                    View::CreateTheme => {
                        if app.state.focus == Focus::ThemeEditor {
                            app.select_create_theme_next();
                        } else if app.state.focus == Focus::SubmitButton {
                            let next_focus_key = app.get_first_next_focus_keybinding();
                            let prev_focus_key = app.get_first_prv_focus_keybinding();
                            app.send_warning_toast(&format!(
                                "Move Focus to the theme editor with {} or {}, to select a style to edit",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    View::NewBoard => {
                        if app.state.focus == Focus::NewBoardDescription {
                            app.state.text_buffers.board_description.scroll((1, 0))
                        }
                    }
                    View::NewCard => {
                        if app.state.focus == Focus::CardDescription {
                            app.state.text_buffers.card_description.scroll((1, 0))
                        }
                    }
                    _ => {
                        if app.state.focus == Focus::Body {
                            go_down(app);
                        } else if app.state.focus == Focus::Help {
                            app.help_next();
                        } else if app.state.focus == Focus::Log {
                            app.log_next();
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::Right => {
                reset_mouse(app);
                if let Some(popup) = app.state.z_stack.last() {
                    match popup {
                        PopUp::ConfirmDiscardCardChanges => {
                            toggle_focus_between_submit_and_extra(app);
                        }
                        PopUp::DateTimePicker => {
                            handle_date_time_picker_action(app, None, Some(action));
                        }
                        _ => {}
                    }
                } else if app.state.focus == Focus::Body
                    && View::views_with_kanban_board().contains(&app.state.current_view)
                {
                    go_right(app);
                }
                AppReturn::Continue
            }
            Action::Left => {
                reset_mouse(app);
                if let Some(popup) = app.state.z_stack.last() {
                    match popup {
                        PopUp::ConfirmDiscardCardChanges => {
                            toggle_focus_between_submit_and_extra(app);
                        }
                        PopUp::DateTimePicker => {
                            handle_date_time_picker_action(app, None, Some(action));
                        }
                        _ => {}
                    }
                } else if app.state.focus == Focus::Body
                    && View::views_with_kanban_board().contains(&app.state.current_view)
                {
                    go_left(app);
                }
                AppReturn::Continue
            }
            Action::TakeUserInput => {
                match app.state.current_view {
                    View::NewBoard | View::NewCard => {
                        app.state.app_status = AppStatus::UserInput;
                        info!("Taking user input");
                    }
                    _ => {
                        if let Some(popup) = app.state.z_stack.last() {
                            match popup {
                                PopUp::EditGeneralConfig
                                | PopUp::CustomHexColorPromptFG
                                | PopUp::CustomHexColorPromptBG => {
                                    app.state.app_status = AppStatus::UserInput;
                                    info!("Taking user input");
                                }
                                PopUp::EditSpecificKeyBinding => {
                                    app.state.app_status = AppStatus::KeyBindMode;
                                    info!("Taking user Keybinding input");
                                }
                                PopUp::ViewCard => {
                                    handle_edit_new_card(app);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::StopUserInput => {
                if app.state.app_status == AppStatus::UserInput {
                    app.state.app_status = AppStatus::Initialized;
                    info!("Exiting user input mode");
                }
                AppReturn::Continue
            }
            Action::GoToPreviousViewOrCancel => handle_go_to_previous_view(app).await,
            Action::Accept => {
                if let Some(popup) = app.state.z_stack.last() {
                    match popup {
                        PopUp::ChangeView => handle_change_view(app),
                        PopUp::CardStatusSelector => {
                            return handle_change_card_status(app, None);
                        }
                        PopUp::EditGeneralConfig => {
                            if app.state.current_view == View::CreateTheme {
                                handle_create_theme_action(app);
                            } else {
                                handle_edit_general_config(app);
                            }
                        }
                        PopUp::EditSpecificKeyBinding => handle_edit_specific_keybinding(app),
                        PopUp::SelectDefaultView => handle_default_view_selection(app),
                        PopUp::ChangeDateFormatPopup => handle_change_date_format(app),
                        PopUp::ChangeTheme => {
                            return handle_change_theme(app, app.state.default_theme_mode)
                        }
                        PopUp::EditThemeStyle => return handle_create_theme_action(app),
                        PopUp::SaveThemePrompt => handle_save_theme_prompt(app),
                        PopUp::CustomHexColorPromptFG => {
                            return handle_custom_hex_color_prompt(app, true)
                        }
                        PopUp::CustomHexColorPromptBG => {
                            return handle_custom_hex_color_prompt(app, false)
                        }
                        PopUp::ViewCard => return handle_general_actions_view_card(app),
                        PopUp::CommandPalette => {
                            unreachable!("Command palette should not be handled here");
                        }
                        PopUp::ConfirmDiscardCardChanges => match app.state.focus {
                            Focus::SubmitButton => {
                                handle_edit_card_submit(app);
                                app.close_popup();
                            }
                            Focus::ExtraFocus => {
                                app.close_popup();
                            }
                            _ => {}
                        },
                        PopUp::CardPrioritySelector => {
                            return handle_change_card_priority(app, None);
                        }
                        PopUp::FilterByTag => {
                            handle_filter_by_tag(app);
                            return AppReturn::Continue;
                        }
                        PopUp::DateTimePicker => {
                            handle_date_time_picker_action(app, None, Some(action));
                            return AppReturn::Continue;
                        }
                        PopUp::TagPicker => {
                            // This is never reached as tag picker does not handle its own actions
                            return AppReturn::Continue;
                        }
                    }
                    app.close_popup();
                    return AppReturn::Continue;
                }
                match app.state.current_view {
                    View::ConfigMenu => handle_config_menu_action(app),
                    View::MainMenu => match app.state.focus {
                        Focus::MainMenu => handle_main_menu_action(app).await,
                        Focus::Help => {
                            app.set_view(View::HelpMenu);
                            AppReturn::Continue
                        }
                        Focus::Log => {
                            app.set_view(View::LogsOnly);
                            AppReturn::Continue
                        }
                        _ => AppReturn::Continue,
                    },
                    View::NewBoard => {
                        handle_new_board_action(app);
                        AppReturn::Continue
                    }
                    View::NewCard => {
                        handle_new_card_action(app);
                        AppReturn::Continue
                    }
                    View::LoadLocalSave => {
                        app.dispatch(IoEvent::LoadSaveLocal).await;
                        AppReturn::Continue
                    }
                    View::EditKeybindings => {
                        handle_edit_keybindings_action(app);
                        AppReturn::Continue
                    }
                    View::CreateTheme => {
                        handle_create_theme_action(app);
                        AppReturn::Continue
                    }
                    View::Login => {
                        handle_login_action(app).await;
                        AppReturn::Continue
                    }
                    View::SignUp => {
                        handle_signup_action(app).await;
                        AppReturn::Continue
                    }
                    View::ResetPassword => {
                        handle_reset_password_action(app).await;
                        AppReturn::Continue
                    }
                    View::LoadCloudSave => {
                        app.dispatch(IoEvent::LoadSaveCloud).await;
                        AppReturn::Continue
                    }
                    _ => {
                        match app.state.focus {
                            Focus::Help => {
                                app.set_view(View::HelpMenu);
                            }
                            Focus::Log => {
                                app.set_view(View::LogsOnly);
                            }
                            Focus::Title => {
                                app.set_view(View::MainMenu);
                            }
                            _ => {}
                        }
                        if View::views_with_kanban_board().contains(&app.state.current_view)
                            && app.state.focus == Focus::Body
                            && app.state.current_board_id.is_some()
                            && app.state.current_card_id.is_some()
                        {
                            app.set_popup(PopUp::ViewCard);
                        }
                        AppReturn::Continue
                    }
                }
            }
            Action::HideUiElement => {
                let current_focus = app.state.focus;
                match app.state.current_view {
                    View::Zen => {
                        app.set_view(View::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                    View::TitleBody => {
                        if current_focus == Focus::Title {
                            app.set_view(View::Zen);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    View::BodyHelp => {
                        if current_focus == Focus::Help {
                            app.set_view(View::Zen);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    View::BodyLog => {
                        if current_focus == Focus::Log {
                            app.set_view(View::Zen);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    View::TitleBodyHelp => {
                        if current_focus == Focus::Title {
                            app.set_view(View::BodyHelp);
                        } else if current_focus == Focus::Help {
                            app.set_view(View::TitleBody);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    View::TitleBodyLog => {
                        if current_focus == Focus::Title {
                            app.set_view(View::BodyLog);
                        } else if current_focus == Focus::Log {
                            app.set_view(View::TitleBody);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    View::TitleBodyHelpLog => {
                        if current_focus == Focus::Title {
                            app.set_view(View::BodyHelpLog);
                        } else if current_focus == Focus::Help {
                            app.set_view(View::TitleBodyLog);
                        } else if current_focus == Focus::Log {
                            app.set_view(View::TitleBodyHelp);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    View::BodyHelpLog => {
                        if current_focus == Focus::Help {
                            app.set_view(View::BodyLog);
                        } else if current_focus == Focus::Log {
                            app.set_view(View::BodyHelp);
                        } else {
                            app.set_view(View::MainMenu);
                            if app.state.app_list_states.main_menu.selected().is_none() {
                                app.main_menu_next();
                            }
                        }
                    }
                    _ => {}
                };
                AppReturn::Continue
            }
            Action::SaveState => {
                if View::views_with_kanban_board().contains(&app.state.current_view) {
                    app.dispatch(IoEvent::SaveLocalData).await;
                }
                AppReturn::Continue
            }
            Action::NewBoard => {
                if View::views_with_kanban_board().contains(&app.state.current_view) {
                    reset_new_board_form(app);
                    app.set_view(View::NewBoard);
                    app.state.prev_focus = Some(app.state.focus);
                }
                AppReturn::Continue
            }
            Action::NewCard => {
                if View::views_with_kanban_board().contains(&app.state.current_view) {
                    if app.state.current_board_id.is_none() {
                        warn!("No board available to add card to");
                        app.send_warning_toast("No board available to add card to", None);
                        return AppReturn::Continue;
                    }
                    reset_new_card_form(app);
                    app.set_view(View::NewCard);
                    app.state.prev_focus = Some(app.state.focus);
                }
                AppReturn::Continue
            }
            Action::Delete => match app.state.current_view {
                View::LoadLocalSave => {
                    app.dispatch(IoEvent::DeleteLocalSave).await;
                    tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                    app.dispatch(IoEvent::LoadLocalPreview).await;
                    AppReturn::Continue
                }
                View::LoadCloudSave => {
                    app.dispatch(IoEvent::DeleteCloudSave).await;
                    tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                    app.dispatch(IoEvent::GetCloudData).await;
                    tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                    app.dispatch(IoEvent::LoadCloudPreview).await;
                    AppReturn::Continue
                }
                _ => {
                    if !View::views_with_kanban_board().contains(&app.state.current_view) {
                        return AppReturn::Continue;
                    }
                    match app.state.focus {
                        Focus::Body => {
                            if let Some(current_board_id) = app.state.current_board_id {
                                if let Some(current_card_id) = app.state.current_card_id {
                                    match app.boards.get_mut_board_with_id(current_board_id) {
                                        Some(current_board) => {
                                            let current_board_id = current_board.id;
                                            let card_index =
                                                current_board.cards.get_card_index(current_card_id);
                                            if let Some(card_index) = card_index {
                                                let card = current_board
                                                    .cards
                                                    .get_card_with_index(card_index)
                                                    .unwrap()
                                                    .clone();
                                                let card_name = card.name.clone();
                                                current_board
                                                    .cards
                                                    .remove_card_with_id(current_card_id);
                                                if card_index > 0 {
                                                    app.state.current_card_id = Some(
                                                        current_board
                                                            .cards
                                                            .get_card_with_index(card_index - 1)
                                                            .unwrap()
                                                            .id,
                                                    );
                                                } else if !current_board.cards.is_empty() {
                                                    app.state.current_card_id =
                                                        current_board.cards.get_first_card_id();
                                                } else {
                                                    app.state.current_card_id = None;
                                                }
                                                warn!("Deleted card {}", card_name);
                                                app.action_history_manager.new_action(
                                                    ActionHistory::DeleteCard(
                                                        card,
                                                        current_board.id,
                                                    ),
                                                );
                                                app.send_warning_toast(
                                                    &format!("Deleted card {}", card_name),
                                                    None,
                                                );
                                                if let Some(visible_cards) = app
                                                    .visible_boards_and_cards
                                                    .get_mut(&current_board_id)
                                                {
                                                    if let Some(card_index) =
                                                        visible_cards.iter().position(|card_id| {
                                                            *card_id == current_card_id
                                                        })
                                                    {
                                                        visible_cards.remove(card_index);
                                                    }
                                                }
                                                refresh_visible_boards_and_cards(app);
                                            }
                                        }
                                        None => {
                                            debug!("No board available to delete card from");
                                            return AppReturn::Continue;
                                        }
                                    }
                                } else if let Some(board) =
                                    app.boards.get_board_with_id(current_board_id).cloned()
                                {
                                    let board_index =
                                        app.boards.get_board_index(current_board_id).unwrap();
                                    let board_name = board.name.clone();
                                    app.boards.remove_board_with_id(current_board_id);
                                    if board_index > 0 && !app.boards.is_empty() {
                                        app.state.current_board_id = Some(
                                            app.boards
                                                .get_board_with_index(board_index - 1)
                                                .unwrap()
                                                .id,
                                        );
                                    } else {
                                        app.state.current_board_id = None;
                                    }
                                    warn!("Deleted board {}", board_name);
                                    app.action_history_manager
                                        .new_action(ActionHistory::DeleteBoard(board));
                                    app.send_warning_toast(
                                        &format!("Deleted board {}", board_name),
                                        None,
                                    );
                                    app.visible_boards_and_cards.remove(&current_board_id);
                                    refresh_visible_boards_and_cards(app);
                                }
                            }
                            AppReturn::Continue
                        }
                        _ => AppReturn::Continue,
                    }
                }
            },
            Action::DeleteBoard => {
                if !View::views_with_kanban_board().contains(&app.state.current_view) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        if let Some(current_board_id) = app.state.current_board_id {
                            if let Some(board) =
                                app.boards.get_board_with_id(current_board_id).cloned()
                            {
                                let board_index =
                                    app.boards.get_board_index(current_board_id).unwrap();
                                let board_name = board.name.clone();
                                app.boards.remove_board_with_id(current_board_id);
                                if board_index > 0 && !app.boards.is_empty() {
                                    app.state.current_board_id = Some(
                                        app.boards
                                            .get_board_with_index(board_index - 1)
                                            .unwrap()
                                            .id,
                                    );
                                } else {
                                    app.state.current_board_id = None;
                                }
                                warn!("Deleted board {}", board_name);
                                app.action_history_manager
                                    .new_action(ActionHistory::DeleteBoard(board));
                                app.send_warning_toast(
                                    &format!("Deleted board {}", board_name),
                                    None,
                                );
                                app.visible_boards_and_cards.remove(&current_board_id);
                                refresh_visible_boards_and_cards(app);
                            }
                        }
                        AppReturn::Continue
                    }
                    _ => AppReturn::Continue,
                }
            }
            Action::ChangeCardStatusToCompleted => {
                if !View::views_with_kanban_board().contains(&app.state.current_view)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_status(app, Some(CardStatus::Complete))
            }
            Action::ChangeCardStatusToActive => {
                if !View::views_with_kanban_board().contains(&app.state.current_view)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_status(app, Some(CardStatus::Active))
            }
            Action::ChangeCardStatusToStale => {
                if !View::views_with_kanban_board().contains(&app.state.current_view)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_status(app, Some(CardStatus::Stale))
            }
            Action::ChangeCardPriorityToHigh => {
                if !View::views_with_kanban_board().contains(&app.state.current_view)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_priority(app, Some(CardPriority::High))
            }
            Action::ChangeCardPriorityToMedium => {
                if !View::views_with_kanban_board().contains(&app.state.current_view)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_priority(app, Some(CardPriority::Medium))
            }
            Action::ChangeCardPriorityToLow => {
                if !View::views_with_kanban_board().contains(&app.state.current_view)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_priority(app, Some(CardPriority::Low))
            }
            Action::GoToMainMenu => {
                match app.state.current_view {
                    View::NewBoard => {
                        reset_new_board_form(app);
                    }
                    View::NewCard => {
                        reset_new_card_form(app);
                    }
                    View::Login => {
                        reset_login_form(app);
                    }
                    View::SignUp => {
                        reset_signup_form(app);
                    }
                    View::ResetPassword => {
                        reset_reset_password_form(app);
                    }
                    _ => {}
                }
                app.state.current_board_id = None;
                app.state.current_card_id = None;
                app.set_view(View::MainMenu);
                if app.state.app_list_states.main_menu.selected().is_none() {
                    app.state.app_list_states.main_menu.select(Some(0));
                }
                AppReturn::Continue
            }
            Action::MoveCardUp => {
                if !View::views_with_kanban_board().contains(&app.state.current_view) {
                    return AppReturn::Continue;
                }
                if app.state.focus == Focus::Body {
                    if app.state.current_card_id.is_none() {
                        return AppReturn::Continue;
                    } else {
                        let boards: &mut Boards = if app.filtered_boards.is_empty() {
                            &mut app.boards
                        } else {
                            &mut app.filtered_boards
                        };
                        if app.state.current_board_id.is_none() {
                            debug!("Cannot move card up without a current board id");
                            return AppReturn::Continue;
                        }
                        if app.state.current_card_id.is_none() {
                            debug!("Cannot move card up without a current card id");
                            return AppReturn::Continue;
                        }
                        let current_board_id = app.state.current_board_id.unwrap();
                        let current_card_id = app.state.current_card_id.unwrap();
                        match boards.get_mut_board_with_id(current_board_id) {
                            Some(current_board) => {
                                let current_card_index_in_all =
                                    current_board.cards.get_card_index(current_card_id);
                                if current_card_index_in_all.is_none() {
                                    debug!("Cannot move card up without a current card index");
                                    return AppReturn::Continue;
                                }
                                let current_card_index_in_all = current_card_index_in_all.unwrap();
                                if current_card_index_in_all == 0 {
                                    app.send_error_toast(
                                "Cannot move card up, it is already at the top of the board",
                                None,
                                    );
                                    error!("Cannot move card up, it is already at the top of the board");
                                    return AppReturn::Continue;
                                }
                                let current_card_index_in_visible = app.visible_boards_and_cards
                                    [&current_board_id]
                                    .iter()
                                    .position(|card_id| *card_id == current_card_id);
                                if current_card_index_in_visible.is_none() {
                                    debug!(
                                        "Cannot move card up without a current card index in visible cards"
                                    );
                                    return AppReturn::Continue;
                                }
                                let current_card_index_in_visible =
                                    current_card_index_in_visible.unwrap();
                                if current_card_index_in_visible == 0 {
                                    let card_above_id = current_board
                                        .cards
                                        .get_card_with_index(current_card_index_in_all - 1)
                                        .unwrap()
                                        .id;
                                    let mut visible_cards: Vec<(u64, u64)> = vec![];
                                    visible_cards.push(current_card_id);
                                    visible_cards.push(card_above_id);

                                    for card in
                                        app.visible_boards_and_cards[&current_board_id].iter()
                                    {
                                        if *card != current_card_id
                                            && visible_cards.len()
                                                < app.config.no_of_cards_to_show as usize
                                        {
                                            visible_cards.push(*card);
                                        }
                                    }
                                    app.visible_boards_and_cards
                                        .entry(current_board_id)
                                        .and_modify(|cards| *cards = visible_cards);
                                } else {
                                    app.visible_boards_and_cards
                                        .get_mut(&current_board_id)
                                        .unwrap()
                                        .swap(
                                            current_card_index_in_visible,
                                            current_card_index_in_visible - 1,
                                        );
                                }
                                current_board
                                    .cards
                                    .swap(current_card_index_in_all, current_card_index_in_all - 1);
                                app.action_history_manager.new_action(
                                    ActionHistory::MoveCardWithinBoard(
                                        current_board_id,
                                        current_card_index_in_all,
                                        current_card_index_in_all - 1,
                                    ),
                                );
                            }
                            None => {
                                debug!("Cannot move card up without a current board index");
                                return AppReturn::Continue;
                            }
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::MoveCardDown => {
                if !View::views_with_kanban_board().contains(&app.state.current_view) {
                    return AppReturn::Continue;
                }
                if app.state.focus == Focus::Body {
                    if app.state.current_card_id.is_none() {
                        return AppReturn::Continue;
                    } else {
                        let boards: &mut Boards = if app.filtered_boards.is_empty() {
                            &mut app.boards
                        } else {
                            &mut app.filtered_boards
                        };
                        if app.state.current_board_id.is_none() {
                            debug!("Cannot move card down without a current board id");
                            return AppReturn::Continue;
                        }
                        if app.state.current_card_id.is_none() {
                            debug!("Cannot move card down without a current card id");
                            return AppReturn::Continue;
                        }
                        let current_board_id = app.state.current_board_id.unwrap();
                        let current_card_id = app.state.current_card_id.unwrap();
                        let current_board = boards.get_mut_board_with_id(current_board_id);
                        if current_board.is_none() {
                            debug!("Cannot move card down without a current board index");
                            return AppReturn::Continue;
                        }
                        let current_board = current_board.unwrap();
                        let current_card_index_in_all =
                            current_board.cards.get_card_index(current_card_id);
                        if current_card_index_in_all.is_none() {
                            debug!("Cannot move card down without a current card index");
                            return AppReturn::Continue;
                        }
                        let current_card_index_in_all = current_card_index_in_all.unwrap();
                        if current_card_index_in_all == current_board.cards.len() - 1 {
                            app.send_error_toast(
                                "Cannot move card down, it is already at the bottom of the board",
                                None,
                            );
                            error!(
                                "Cannot move card down, it is already at the bottom of the board"
                            );
                            return AppReturn::Continue;
                        }
                        let current_card_index_in_visible = app.visible_boards_and_cards
                            [&current_board_id]
                            .iter()
                            .position(|card_id| *card_id == current_card_id);
                        if current_card_index_in_visible.is_none() {
                            debug!("Cannot move card down without a current card index in visible cards");
                            return AppReturn::Continue;
                        }
                        let current_card_index_in_visible = current_card_index_in_visible.unwrap();
                        if current_card_index_in_visible
                            == app.visible_boards_and_cards[&current_board_id].len() - 1
                        {
                            let card_below_id = current_board
                                .cards
                                .get_card_with_index(current_card_index_in_all + 1)
                                .unwrap()
                                .id;
                            let mut visible_cards: Vec<(u64, u64)> = vec![];
                            visible_cards.push(card_below_id);
                            visible_cards.push(current_card_id);
                            for card in app.visible_boards_and_cards[&current_board_id].iter().rev()
                            {
                                if *card != current_card_id
                                    && visible_cards.len() < app.config.no_of_cards_to_show as usize
                                {
                                    visible_cards.insert(0, *card);
                                }
                            }

                            app.visible_boards_and_cards
                                .entry(current_board_id)
                                .and_modify(|cards| *cards = visible_cards);
                        } else {
                            app.visible_boards_and_cards
                                .get_mut(&current_board_id)
                                .unwrap()
                                .swap(
                                    current_card_index_in_visible,
                                    current_card_index_in_visible + 1,
                                );
                        }
                        current_board
                            .cards
                            .swap(current_card_index_in_all, current_card_index_in_all + 1);
                        app.action_history_manager
                            .new_action(ActionHistory::MoveCardWithinBoard(
                                current_board_id,
                                current_card_index_in_all,
                                current_card_index_in_all + 1,
                            ));
                    }
                }
                AppReturn::Continue
            }
            Action::MoveCardRight => {
                if !View::views_with_kanban_board().contains(&app.state.current_view) {
                    return AppReturn::Continue;
                }
                if app.state.focus == Focus::Body {
                    if app.state.current_card_id.is_none() {
                        return AppReturn::Continue;
                    } else if let Some(current_board_id) = app.state.current_board_id {
                        let boards: &mut Boards = if app.filtered_boards.is_empty() {
                            &mut app.boards
                        } else {
                            &mut app.filtered_boards
                        };
                        let moved_from_board_index = boards.get_board_index(current_board_id);
                        if moved_from_board_index.is_none() {
                            app.send_error_toast(
                                "Something went wrong, could not find the board",
                                None,
                            );
                            debug!("Moved from board index is none");
                            return AppReturn::Continue;
                        }
                        let moved_from_board_index = moved_from_board_index.unwrap();
                        if moved_from_board_index < boards.len() - 1 {
                            let moved_from_board =
                                boards.get_mut_board_with_id(current_board_id).unwrap();
                            let moved_from_board_id = moved_from_board.id;
                            let moved_to_board_index = moved_from_board_index + 1;
                            if let Some(current_card_id) = app.state.current_card_id {
                                let card_index = moved_from_board
                                    .cards
                                    .get_card_index(current_card_id)
                                    .unwrap();
                                let card = moved_from_board
                                    .cards
                                    .remove_card_with_id(current_card_id)
                                    .unwrap();
                                let card_id = card.id;
                                let card_name = card.name.clone();
                                let moved_from_board_cards = moved_from_board.cards.clone();
                                let moved_to_board = boards
                                    .get_mut_board_with_index(moved_to_board_index)
                                    .unwrap();
                                moved_to_board.cards.add_card(card.clone());
                                if moved_to_board.cards.len()
                                    <= app.config.no_of_cards_to_show as usize
                                {
                                    app.visible_boards_and_cards
                                        .entry(moved_to_board.id)
                                        .and_modify(|cards| cards.push(card_id));
                                }
                                app.visible_boards_and_cards
                                    .entry(moved_from_board_id)
                                    .and_modify(|cards| {
                                        cards.retain(|card_id| *card_id != current_card_id)
                                    });
                                let mut moved_to_board_visible_cards: Vec<(u64, u64)> = vec![];
                                let mut moved_from_board_visible_cards: Vec<(u64, u64)> = vec![];
                                for card in moved_to_board.cards.get_all_cards().iter().rev() {
                                    if moved_to_board_visible_cards.len()
                                        < app.config.no_of_cards_to_show as usize
                                    {
                                        moved_to_board_visible_cards.insert(0, card.id);
                                    }
                                }
                                for card in moved_from_board_cards.get_all_cards().iter().rev() {
                                    if moved_from_board_visible_cards.len()
                                        < app.config.no_of_cards_to_show as usize
                                        && !moved_to_board_visible_cards.contains(&card.id)
                                    {
                                        moved_from_board_visible_cards.insert(0, card.id);
                                    }
                                }
                                app.visible_boards_and_cards
                                    .entry(moved_to_board.id)
                                    .and_modify(|cards| *cards = moved_to_board_visible_cards);
                                app.visible_boards_and_cards
                                    .entry(moved_from_board_id)
                                    .and_modify(|cards| *cards = moved_from_board_visible_cards);
                                app.state.current_board_id = Some(moved_to_board.id);

                                let info_msg = &format!(
                                    "Moved card \"{}\" to board \"{}\"",
                                    card_name, moved_to_board.name
                                );
                                app.action_history_manager.new_action(
                                    ActionHistory::MoveCardBetweenBoards(
                                        card.clone(),
                                        moved_from_board_id,
                                        moved_to_board.id,
                                        card_index,
                                        0,
                                    ),
                                );

                                info!("{}", info_msg);
                                app.send_info_toast(info_msg, None);
                            }
                        } else {
                            error!("Cannot move card right as it is the last board");
                            app.send_error_toast(
                                "Cannot move card right as it is the last board",
                                None,
                            );
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::MoveCardLeft => {
                if !View::views_with_kanban_board().contains(&app.state.current_view) {
                    return AppReturn::Continue;
                }
                if app.state.focus == Focus::Body {
                    if app.state.current_card_id.is_none() {
                        return AppReturn::Continue;
                    } else if let Some(current_board) = app.state.current_board_id {
                        let boards: &mut Boards = if app.filtered_boards.is_empty() {
                            &mut app.boards
                        } else {
                            &mut app.filtered_boards
                        };
                        let moved_from_board_index = boards.get_board_index(current_board);
                        if moved_from_board_index.is_none() {
                            app.send_error_toast(
                                "Something went wrong, could not find the board",
                                None,
                            );
                            debug!("Moved from board index is none");
                            return AppReturn::Continue;
                        }
                        let moved_from_board_index = moved_from_board_index.unwrap();
                        let moved_to_board_index = moved_from_board_index - 1;
                        if moved_from_board_index > 0 {
                            if let Some(current_card) = app.state.current_card_id {
                                let moved_from_board = boards
                                    .get_mut_board_with_index(moved_from_board_index)
                                    .unwrap();
                                let card_index =
                                    moved_from_board.cards.get_card_index(current_card).unwrap();
                                let card = moved_from_board
                                    .cards
                                    .remove_card_with_id(current_card)
                                    .unwrap();
                                let moved_from_board_id = moved_from_board.id;
                                let moved_from_board_cards = moved_from_board.cards.clone();
                                let moved_to_board = boards
                                    .get_mut_board_with_index(moved_to_board_index)
                                    .unwrap();
                                let moved_to_board_id = moved_to_board.id;
                                let card_id = card.id;
                                let card_name = card.name.clone();
                                moved_to_board.cards.add_card(card.clone());
                                if moved_to_board.cards.len()
                                    <= app.config.no_of_cards_to_show as usize
                                {
                                    app.visible_boards_and_cards
                                        .entry(moved_to_board_id)
                                        .and_modify(|cards| cards.push(card_id));
                                }
                                app.visible_boards_and_cards
                                    .entry(moved_from_board_id)
                                    .and_modify(|cards| {
                                        cards.retain(|card_id| *card_id != current_card)
                                    });
                                let mut moved_to_board_visible_cards: Vec<(u64, u64)> = vec![];
                                let mut moved_from_board_visible_cards: Vec<(u64, u64)> = vec![];
                                for card in moved_to_board.cards.get_all_cards().iter().rev() {
                                    if moved_to_board_visible_cards.len()
                                        < app.config.no_of_cards_to_show as usize
                                    {
                                        moved_to_board_visible_cards.insert(0, card.id);
                                    }
                                }
                                for card in moved_from_board_cards.get_all_cards().iter().rev() {
                                    if moved_from_board_visible_cards.len()
                                        < app.config.no_of_cards_to_show as usize
                                        && !moved_to_board_visible_cards.contains(&card.id)
                                    {
                                        moved_from_board_visible_cards.insert(0, card.id);
                                    }
                                }
                                app.visible_boards_and_cards
                                    .entry(moved_to_board_id)
                                    .and_modify(|cards| *cards = moved_to_board_visible_cards);
                                app.visible_boards_and_cards
                                    .entry(moved_from_board_id)
                                    .and_modify(|cards| *cards = moved_from_board_visible_cards);
                                app.state.current_board_id = Some(moved_to_board_id);

                                let info_msg = &format!(
                                    "Moved card \"{}\" to board \"{}\"",
                                    card_name, moved_to_board.name
                                );
                                app.action_history_manager.new_action(
                                    ActionHistory::MoveCardBetweenBoards(
                                        card.clone(),
                                        moved_from_board_id,
                                        moved_to_board_id,
                                        card_index,
                                        0,
                                    ),
                                );

                                info!("{}", info_msg);
                                app.send_info_toast(info_msg, None);
                            }
                        } else {
                            error!("Cannot move card left as it is the first board");
                            app.send_error_toast(
                                "Cannot move card left as it is the first board",
                                None,
                            );
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::ToggleCommandPalette => {
                if !app.state.z_stack.contains(&PopUp::CommandPalette) {
                    app.set_popup(PopUp::CommandPalette);
                } else {
                    // move CommandPalette to the top of the z stack if it is not already at the top
                    // safely unwrap as we know that the command palette is in the z stack
                    let command_palette_index = app
                        .state
                        .z_stack
                        .iter()
                        .position(|popup| *popup == PopUp::CommandPalette)
                        .unwrap();
                    if command_palette_index != app.state.z_stack.len() - 1 {
                        app.state.z_stack.remove(command_palette_index);
                        app.set_popup(PopUp::CommandPalette);
                    }
                    match app.state.z_stack.last().unwrap() {
                        PopUp::CommandPalette => {
                            app.close_popup();
                            app.widgets.command_palette.reset(&mut app.state);
                            app.state.app_status = AppStatus::Initialized;
                        }
                        PopUp::ViewCard => {
                            if app.state.card_being_edited.is_some() {
                                app.set_popup(PopUp::ConfirmDiscardCardChanges);
                                app.state.app_status = AppStatus::Initialized;
                            } else {
                                app.set_popup(PopUp::CommandPalette);
                            }
                        }
                        PopUp::ConfirmDiscardCardChanges => {
                            app.close_popup();
                            app.set_popup(PopUp::CommandPalette);
                        }
                        _ => {
                            app.set_popup(PopUp::CommandPalette);
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::Undo => {
                if View::views_with_kanban_board().contains(&app.state.current_view) {
                    app.undo();
                }
                AppReturn::Continue
            }
            Action::Redo => {
                if View::views_with_kanban_board().contains(&app.state.current_view) {
                    app.redo();
                }
                AppReturn::Continue
            }
            Action::ClearAllToasts => {
                app.widgets.toast_widget.toasts.clear();
                info!("Cleared toast messages");
                AppReturn::Continue
            }
        }
    } else {
        // Warn user that they are not in user input mode
        if app.state.card_being_edited.is_some()
            || app.state.current_view == View::NewCard
            || app.state.current_view == View::NewBoard
        {
            let mut keys = String::new();
            for key in app.config.keybindings.take_user_input.iter() {
                keys.push_str(&format!("{}, ", key));
            }
            warn!(
                "You Might want to enter user input mode by pressing any of the following keys: {}",
                keys
            );
            app.send_warning_toast(
                &format!(
                    "You Might want to enter user input mode by pressing any of the following keys: {}",
                    keys
                ),
                Some(Duration::from_secs(5)),
            );
        }
        warn!("No action associated to {}", key);
        app.send_warning_toast(
            &format!("No action associated to {}", key),
            Some(Duration::from_secs(5)),
        );
        AppReturn::Continue
    }
}

fn toggle_focus_between_submit_and_extra(app: &mut App) {
    app.state.set_focus(match app.state.focus {
        Focus::SubmitButton => Focus::ExtraFocus,
        _ => Focus::SubmitButton,
    });
}

pub async fn handle_mouse_action(app: &mut App<'_>, mouse_action: Mouse) -> AppReturn {
    let mut left_button_pressed = false;
    let mut right_button_pressed = false;
    let mut middle_button_pressed = false;
    let mut mouse_scroll_up = false;
    let mut mouse_scroll_down = false;
    let mut mouse_scroll_left = false;
    let mut mouse_scroll_right = false;
    match mouse_action {
        Mouse::Move(x, y) => {
            app.state.previous_mouse_coordinates = app.state.current_mouse_coordinates;
            app.state.current_mouse_coordinates = (x, y);
        }
        Mouse::Drag(x, y) => {
            app.state.current_mouse_coordinates = (x, y);
            let is_invalid_state = !View::views_with_kanban_board()
                .contains(&app.state.current_view)
                || app.state.hovered_card.is_none()
                || app.state.hovered_board.is_none();
            if is_invalid_state {
                return AppReturn::Continue;
            }
            if !app.state.card_drag_mode {
                app.state.card_drag_mode = true;
            }
        }
        Mouse::LeftPress => left_button_pressed = true,
        Mouse::RightPress => right_button_pressed = true,
        Mouse::MiddlePress => middle_button_pressed = true,
        Mouse::ScrollUp => mouse_scroll_up = true,
        Mouse::ScrollDown => mouse_scroll_down = true,
        Mouse::ScrollLeft => mouse_scroll_left = true,
        Mouse::ScrollRight => mouse_scroll_right = true,
        Mouse::Unknown => {}
    }

    if let Some(mouse_action) = &app.state.last_mouse_action {
        match mouse_action {
            Mouse::Drag(_, _) => {
                if left_button_pressed || right_button_pressed || middle_button_pressed {
                    left_button_pressed = false;
                    right_button_pressed = false;
                    middle_button_pressed = false;
                    if app.state.hovered_card.is_some() && app.state.hovered_board.is_some() {
                        move_dragged_card(app);
                        reset_card_drag_mode(app);
                        refresh_visible_boards_and_cards(app);
                    }
                    reset_card_drag_mode(app);
                }
            }
            Mouse::Unknown => {
                reset_card_drag_mode(app);
            }
            _ => {}
        }
    }

    if right_button_pressed {
        return handle_go_to_previous_view(app).await;
    }

    if app.state.focus == Focus::Log {
        if mouse_scroll_down {
            app.log_next();
        }
        if mouse_scroll_up {
            app.log_prv();
        }
    }

    if middle_button_pressed {
        if app.state.z_stack.is_empty() {
            app.set_popup(PopUp::CommandPalette);
        } else {
            match app.state.z_stack.last().unwrap() {
                PopUp::CommandPalette => {
                    app.close_popup();
                    app.widgets.command_palette.reset(&mut app.state);
                    app.state.app_status = AppStatus::Initialized;
                }
                PopUp::ViewCard => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup(PopUp::ConfirmDiscardCardChanges);
                    } else {
                        app.set_popup(PopUp::CommandPalette);
                    }
                }
                PopUp::ConfirmDiscardCardChanges => {
                    app.close_popup();
                    app.set_popup(PopUp::CommandPalette);
                }
                _ => {
                    app.set_popup(PopUp::CommandPalette);
                }
            }
        }
        return AppReturn::Continue;
    }

    if let (Some(popup), Some(mouse_focus)) = (app.state.z_stack.last(), app.state.mouse_focus) {
        match popup {
            PopUp::CommandPalette => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::CommandPaletteCommand => {
                            return CommandPaletteWidget::handle_command(app).await;
                        }
                        Focus::CommandPaletteCard => {
                            handle_command_palette_card_selection(app);
                            app.close_popup();
                            app.widgets.command_palette.reset(&mut app.state);
                            app.state.app_status = AppStatus::Initialized;
                        }
                        Focus::CommandPaletteBoard => {
                            handle_command_palette_board_selection(app);
                            app.close_popup();
                            app.widgets.command_palette.reset(&mut app.state);
                            app.state.app_status = AppStatus::Initialized;
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                            app.widgets.command_palette.reset(&mut app.state);
                            app.state.app_status = AppStatus::Initialized;
                        }
                        _ => {}
                    }
                } else if mouse_scroll_up {
                    match mouse_focus {
                        Focus::CommandPaletteCommand => {
                            app.command_palette_command_search_prv();
                        }
                        Focus::CommandPaletteCard => app.command_palette_card_search_prv(),
                        Focus::CommandPaletteBoard => app.command_palette_board_search_prv(),
                        _ => {}
                    }
                } else if mouse_scroll_down {
                    match mouse_focus {
                        Focus::CommandPaletteCommand => {
                            app.command_palette_command_search_next();
                        }
                        Focus::CommandPaletteCard => {
                            app.command_palette_card_search_next();
                        }
                        Focus::CommandPaletteBoard => {
                            app.command_palette_board_search_next();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::SelectDefaultView => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::SelectDefaultView => {
                            handle_default_view_selection(app);
                            app.close_popup();
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::ChangeView => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::ChangeViewPopup => {
                            handle_change_view(app);
                            app.close_popup();
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::CardStatusSelector => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::ChangeCardStatusPopup => {
                            return handle_change_card_status(app, None);
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::EditGeneralConfig => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::EditGeneralConfigPopup => {
                            app.state.app_status = AppStatus::UserInput;
                        }
                        Focus::CloseButton => {
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                        }
                        Focus::SubmitButton => {
                            if app.state.current_view == View::CreateTheme {
                                handle_create_theme_action(app);
                            } else {
                                handle_edit_general_config(app);
                            }
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::EditSpecificKeyBinding => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::EditSpecificKeyBindingPopup => {
                            app.state.app_status = AppStatus::KeyBindMode;
                        }
                        Focus::CloseButton => {
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                        }
                        Focus::SubmitButton => {
                            handle_edit_specific_keybinding(app);
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::ChangeDateFormatPopup => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::ChangeDateFormatPopup => {
                            handle_change_date_format(app);
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::ChangeTheme => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::ThemeSelector => {
                            handle_change_theme(app, app.state.default_theme_mode);
                            app.close_popup();
                        }
                        Focus::CloseButton => {
                            let config_theme = {
                                let all_themes = Theme::all_default_themes();
                                let default_theme = app.config.default_theme.clone();
                                all_themes.iter().find(|t| t.name == default_theme).cloned()
                            };
                            if let Some(theme) = config_theme {
                                app.current_theme = theme;
                            }
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::EditThemeStyle => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::CloseButton => {
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                        }
                        Focus::SubmitButton
                        | Focus::StyleEditorFG
                        | Focus::StyleEditorBG
                        | Focus::StyleEditorModifier
                        | Focus::ExtraFocus => {
                            handle_create_theme_action(app);
                        }
                        _ => {}
                    }
                } else if mouse_scroll_up {
                    handle_theme_maker_scroll_up(app);
                } else if mouse_scroll_down {
                    handle_theme_maker_scroll_down(app);
                }
            }
            PopUp::SaveThemePrompt => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::SubmitButton | Focus::ExtraFocus => {
                            handle_save_theme_prompt(app);
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::CustomHexColorPromptFG => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::SubmitButton => {
                            handle_custom_hex_color_prompt(app, true);
                        }
                        Focus::TextInput => {
                            app.state.app_status = AppStatus::UserInput;
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::CustomHexColorPromptBG => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::SubmitButton => {
                            handle_custom_hex_color_prompt(app, false);
                        }
                        Focus::TextInput => {
                            app.state.app_status = AppStatus::UserInput;
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        _ => {}
                    }
                }
            }
            PopUp::ViewCard => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::CloseButton => {
                            app.close_popup();
                        }
                        Focus::CardName | Focus::CardDescription | Focus::CardComments => {
                            return handle_edit_new_card(app)
                        }
                        Focus::CardTags => {
                            if app.state.card_being_edited.is_none() {
                                handle_edit_new_card(app);
                            }
                            if left_button_pressed {
                                handle_tag_picker_action(app);
                            } else if mouse_scroll_up {
                                app.tag_picker_prv();
                            } else if mouse_scroll_down {
                                app.tag_picker_next();
                            }
                            return AppReturn::Continue;
                        }
                        Focus::CardPriority => {
                            if app.state.card_being_edited.is_none() {
                                handle_edit_new_card(app);
                            }
                            app.set_popup(PopUp::CardPrioritySelector);
                            return AppReturn::Continue;
                        }
                        Focus::CardStatus => {
                            if app.state.card_being_edited.is_none() {
                                handle_edit_new_card(app);
                            }
                            app.set_popup(PopUp::CardStatusSelector);
                            return AppReturn::Continue;
                        }
                        Focus::CardDueDate => {
                            if app.state.card_being_edited.is_none() {
                                handle_edit_new_card(app);
                            }
                            app.set_popup(PopUp::DateTimePicker);
                            return AppReturn::Continue;
                        }
                        Focus::SubmitButton => return handle_edit_card_submit(app),
                        _ => {}
                    }
                } else if mouse_scroll_down && (mouse_focus == Focus::CardDescription) {
                    app.state.text_buffers.card_description.scroll((1, 0))
                } else if mouse_scroll_up && (mouse_focus == Focus::CardDescription) {
                    app.state.text_buffers.card_description.scroll((-1, 0))
                }
            }
            PopUp::CardPrioritySelector => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::CloseButton => {
                            app.state.app_status = AppStatus::Initialized;
                            if app.state.card_being_edited.is_some() {
                                app.set_popup(PopUp::ConfirmDiscardCardChanges);
                            }
                        }
                        Focus::ChangeCardPriorityPopup => {
                            return handle_change_card_priority(app, None)
                        }
                        _ => {}
                    }
                }
            }
            PopUp::ConfirmDiscardCardChanges => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::CloseButton | Focus::ExtraFocus => {
                            app.close_popup();
                        }
                        Focus::SubmitButton => {
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                            return handle_edit_card_submit(app);
                        }
                        _ => {}
                    }
                }
            }
            PopUp::FilterByTag => {
                if left_button_pressed {
                    match mouse_focus {
                        Focus::FilterByTagPopup => {
                            handle_filter_by_tag(app);
                        }
                        Focus::CloseButton => {
                            app.state.filter_tags = None;
                            app.state.all_available_tags = None;
                            app.state.app_list_states.filter_by_tag_list.select(None);
                            app.close_popup();
                        }
                        Focus::SubmitButton => {
                            handle_filter_by_tag(app);
                            app.close_popup();
                        }
                        _ => {}
                    }
                    if app.state.mouse_focus == Some(Focus::FilterByTagPopup) {
                        handle_filter_by_tag(app);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.filter_tags = None;
                        app.state.all_available_tags = None;
                        app.state.app_list_states.filter_by_tag_list.select(None);
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_filter_by_tag(app);
                        app.close_popup();
                    }
                } else if mouse_scroll_up && app.state.mouse_focus == Some(Focus::FilterByTagPopup)
                {
                    app.filter_by_tag_popup_prv()
                } else if mouse_scroll_down
                    && app.state.mouse_focus == Some(Focus::FilterByTagPopup)
                {
                    app.filter_by_tag_popup_next()
                }
            }
            PopUp::DateTimePicker => {
                if left_button_pressed {
                    handle_date_time_picker_action(app, None, Some(Action::Accept));
                } else if mouse_scroll_up {
                    match app.state.focus {
                        Focus::DTPMonth => app.widgets.date_time_picker.month_next(),
                        Focus::DTPYear => app.widgets.date_time_picker.year_next(),
                        Focus::DTPHour => app.widgets.date_time_picker.move_hours_prv(),
                        Focus::DTPMinute => app.widgets.date_time_picker.move_minutes_prv(),
                        Focus::DTPSecond => app.widgets.date_time_picker.move_seconds_prv(),
                        _ => {}
                    }
                } else if mouse_scroll_down {
                    match app.state.focus {
                        Focus::DTPMonth => app.widgets.date_time_picker.month_prv(),
                        Focus::DTPYear => app.widgets.date_time_picker.year_prv(),
                        Focus::DTPHour => app.widgets.date_time_picker.move_hours_next(),
                        Focus::DTPMinute => app.widgets.date_time_picker.move_minutes_next(),
                        Focus::DTPSecond => app.widgets.date_time_picker.move_seconds_next(),
                        _ => {}
                    }
                }
            }
            PopUp::TagPicker => {
                // This is never reached as tag picker does not handle its own actions
            }
        }
    } else {
        match app.state.current_view {
            View::Zen
            | View::TitleBody
            | View::BodyHelp
            | View::BodyLog
            | View::TitleBodyHelp
            | View::TitleBodyLog
            | View::TitleBodyHelpLog
            | View::BodyHelpLog
            | View::ConfigMenu
            | View::EditKeybindings
            | View::HelpMenu
            | View::NewBoard
            | View::NewCard => {
                if left_button_pressed {
                    if let Some(value) = handle_left_click_for_view(app).await {
                        return value;
                    }
                } else if mouse_scroll_up
                    || mouse_scroll_down
                    || mouse_scroll_right
                    || mouse_scroll_left
                {
                    handle_scroll_for_view(
                        app,
                        mouse_scroll_up,
                        mouse_scroll_down,
                        mouse_scroll_right,
                        mouse_scroll_left,
                    );
                }
            }
            View::MainMenu | View::LogsOnly | View::LoadLocalSave | View::CreateTheme => {
                if left_button_pressed {
                    if let Some(value) = handle_left_click_for_view(app).await {
                        return value;
                    }
                }
            }
            View::Login => {
                if left_button_pressed {
                    handle_login_action(app).await
                }
            }
            View::SignUp => {
                if left_button_pressed {
                    handle_signup_action(app).await
                }
            }
            View::ResetPassword => {
                if left_button_pressed {
                    handle_reset_password_action(app).await
                }
            }
            View::LoadCloudSave => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_previous_view(app).await;
                    } else if app.state.mouse_focus == Some(Focus::LoadSave)
                        && app.state.app_list_states.load_save.selected().is_some()
                        && app.state.cloud_data.is_some()
                    {
                        app.dispatch(IoEvent::LoadCloudPreview).await;
                    }
                }
            }
        }
    }
    app.state.last_mouse_action = Some(mouse_action);
    AppReturn::Continue
}

async fn handle_left_click_for_view(app: &mut App<'_>) -> Option<AppReturn> {
    let prv_view = app.state.current_view;
    app.state.mouse_focus?;
    // TODO: investigate if we really need to have mouse focus separate from focus
    let mouse_focus = app.state.mouse_focus.unwrap();
    match mouse_focus {
        Focus::Title => {
            app.set_view(View::MainMenu);
            if app.state.app_list_states.main_menu.selected().is_none() {
                app.main_menu_next()
            }
        }
        Focus::Body => {
            if !(app.state.current_board_id.is_some() && app.state.current_card_id.is_some()) {
                app.send_error_toast("No card selected", None);
                return Some(AppReturn::Continue);
            }
            app.set_popup(PopUp::ViewCard);
        }
        Focus::Help => {
            app.set_view(View::HelpMenu);
        }
        Focus::Log => {
            app.set_view(View::LogsOnly);
        }
        Focus::ConfigTable => {
            return Some(handle_config_menu_action(app));
        }
        Focus::EditKeybindingsTable => {
            handle_edit_keybindings_action(app);
        }
        Focus::CloseButton => match prv_view {
            View::Zen
            | View::TitleBody
            | View::BodyHelp
            | View::BodyLog
            | View::TitleBodyHelp
            | View::TitleBodyLog
            | View::BodyHelpLog
            | View::TitleBodyHelpLog
            | View::MainMenu => {
                return Some(handle_exit(app).await);
            }
            View::NewBoard => {
                reset_new_board_form(app);
                handle_go_to_previous_view(app).await;
            }
            View::NewCard => {
                reset_new_card_form(app);
                handle_go_to_previous_view(app).await;
            }
            View::CreateTheme => {
                app.state.theme_being_edited = Theme::default();
                handle_go_to_previous_view(app).await;
            }
            _ => {
                handle_go_to_previous_view(app).await;
            }
        },
        Focus::SubmitButton => {
            app.state.set_focus(Focus::SubmitButton);
            match prv_view {
                View::EditKeybindings => {
                    handle_edit_keybindings_action(app);
                }
                View::NewBoard => {
                    handle_new_board_action(app);
                    app.state.app_status = AppStatus::Initialized;
                }
                View::NewCard => {
                    handle_new_card_action(app);
                    app.state.app_status = AppStatus::Initialized;
                }
                View::ConfigMenu => {
                    return Some(handle_config_menu_action(app));
                }
                View::CreateTheme => {
                    return Some(handle_create_theme_action(app));
                }
                _ => {}
            }
        }
        Focus::ExtraFocus => {
            if prv_view == View::ConfigMenu {
                return Some(handle_config_menu_action(app));
            } else if prv_view == View::CreateTheme {
                return Some(handle_create_theme_action(app));
            }
        }
        Focus::MainMenu => {
            return Some(handle_main_menu_action(app).await);
        }
        Focus::NewBoardName
        | Focus::NewBoardDescription
        | Focus::CardName
        | Focus::CardDescription => {
            app.state.app_status = AppStatus::UserInput;
            info!("Taking user input");
        }
        Focus::CardDueDate => {
            if app.state.card_being_edited.is_none() {
                handle_edit_new_card(app);
            }
            app.set_popup(PopUp::DateTimePicker);
        }
        Focus::LoadSave => {
            if app.state.app_list_states.load_save.selected().is_some() {
                app.dispatch(IoEvent::LoadLocalPreview).await;
            }
        }
        Focus::ThemeEditor => {
            return Some(handle_create_theme_action(app));
        }
        _ => {}
    }
    None
}

fn handle_scroll_for_view(
    app: &mut App<'_>,
    mouse_scroll_up: bool,
    mouse_scroll_down: bool,
    mouse_scroll_right: bool,
    mouse_scroll_left: bool,
) {
    if mouse_scroll_up {
        if app.state.mouse_focus == Some(Focus::Body) {
            scroll_up(app);
        } else if app.state.mouse_focus == Some(Focus::Help) {
            app.help_prv();
        } else if app.state.mouse_focus == Some(Focus::ConfigTable) {
            app.config_prv();
        } else if app.state.mouse_focus == Some(Focus::EditKeybindingsTable) {
            app.edit_keybindings_prv();
        } else if app.state.mouse_focus == Some(Focus::NewBoardDescription) {
            app.state.text_buffers.board_description.scroll((-1, 0))
        } else if app.state.mouse_focus == Some(Focus::CardDescription) {
            app.state.text_buffers.card_description.scroll((-1, 0))
        }
    } else if mouse_scroll_down {
        if app.state.mouse_focus == Some(Focus::Body) {
            scroll_down(app);
        } else if app.state.mouse_focus == Some(Focus::Help) {
            app.help_next();
        } else if app.state.mouse_focus == Some(Focus::ConfigTable) {
            app.config_next();
        } else if app.state.mouse_focus == Some(Focus::EditKeybindingsTable) {
            app.edit_keybindings_next();
        } else if app.state.mouse_focus == Some(Focus::NewBoardDescription) {
            app.state.text_buffers.board_description.scroll((1, 0))
        } else if app.state.mouse_focus == Some(Focus::CardDescription) {
            app.state.text_buffers.card_description.scroll((1, 0))
        }
    } else if mouse_scroll_right && app.state.mouse_focus == Some(Focus::Body) {
        scroll_right(app);
    } else if mouse_scroll_left && app.state.mouse_focus == Some(Focus::Body) {
        scroll_left(app);
    }
}

fn move_dragged_card(app: &mut App<'_>) {
    let card_being_dragged = app.state.hovered_card.unwrap();
    let hovered_board_id = app.state.hovered_board.unwrap();
    let card_being_dragged_board = card_being_dragged.0;
    let card_being_dragged_id = card_being_dragged.1;
    if hovered_board_id == card_being_dragged_board {
        if app.state.current_card_id.is_none() {
            debug!("Could not find current card");
            return;
        }
        let hovered_card_id = app.state.current_card_id.unwrap();
        // same board so swap cards
        let hovered_board = app.boards.get_board_with_id(hovered_board_id);
        if hovered_board.is_none() {
            debug!("Could not find hovered board");
            return;
        }
        let hovered_board = hovered_board.unwrap();
        let dragged_card_index = hovered_board.cards.get_card_index(card_being_dragged_id);
        if dragged_card_index.is_none() {
            debug!("Could not find dragged card");
            return;
        }
        let hovered_card_index = hovered_board.cards.get_card_index(hovered_card_id);
        if hovered_card_index.is_none() {
            debug!("Could not find hovered card");
            return;
        }
        let dragged_card_index = dragged_card_index.unwrap();
        let hovered_card_index = hovered_card_index.unwrap();
        if dragged_card_index == hovered_card_index {
            debug!("No need to move card as it is already in the same position");
            return;
        }
        let dragged_card_name = hovered_board
            .cards
            .get_card_with_index(dragged_card_index)
            .unwrap()
            .name
            .clone();
        // swap cards
        app.boards.get_mut_boards().iter_mut().for_each(|board| {
            if board.id == hovered_board_id {
                board.cards.swap(dragged_card_index, hovered_card_index);
            }
        });
        app.action_history_manager
            .new_action(ActionHistory::MoveCardWithinBoard(
                hovered_board_id,
                dragged_card_index,
                hovered_card_index,
            ));
        let info_msg = &format!(
            "Moved card \"{}\" from index {} to index {}",
            dragged_card_name, dragged_card_index, hovered_card_index
        );
        info!("{}", info_msg);
        app.send_info_toast(info_msg, None);
    } else {
        let app_boards = app.boards.clone();
        // different board so remove dragged card from current board and add it to the hovered board at the index of the hovered card and push everything else down
        let dragged_card_board_id = card_being_dragged.0;
        let hovered_board_id = app.state.hovered_board;
        if hovered_board_id.is_none() {
            debug!("Could not find hovered board");
            return;
        }
        let hovered_board_id = hovered_board_id.unwrap();
        let dragged_card_board = app_boards.get_board_with_id(dragged_card_board_id);
        if dragged_card_board.is_none() {
            debug!("Could not find dragged card board");
            return;
        }
        let dragged_card_board = dragged_card_board.unwrap();
        let hovered_board = app_boards.get_board_with_id(hovered_board_id);
        if hovered_board.is_none() {
            debug!("Could not find hovered board");
            return;
        }
        let hovered_board = hovered_board.unwrap();
        let dragged_card_id = card_being_dragged.1;
        let hovered_card_id = app.state.current_card_id;
        let dragged_card_index = dragged_card_board.cards.get_card_index(dragged_card_id);
        if dragged_card_index.is_none() {
            debug!("Could not find dragged card");
            return;
        }
        let dragged_card_index = dragged_card_index.unwrap();
        let dragged_card = dragged_card_board
            .cards
            .get_card_with_index(dragged_card_index)
            .unwrap()
            .clone();
        let dragged_card_name = dragged_card.name.clone();
        if hovered_card_id.is_none() {
            // check if hovered board is empty
            if hovered_board.cards.is_empty() {
                debug!("hovered board is empty");
                // add dragged card to hovered board
                app.boards.get_mut_boards().iter_mut().for_each(|board| {
                    if board.id == hovered_board_id {
                        board.cards.add_card_at_index(0, dragged_card.clone());
                    }
                });
                // remove dragged card from current board
                app.boards.get_mut_boards().iter_mut().for_each(|board| {
                    // check if index is valid
                    if board.id == dragged_card_board_id {
                        if board.cards.len() > dragged_card_index {
                            board.cards.remove_card_with_id(dragged_card_id);
                        } else {
                            debug!("Invalid Index for dragged card, board_id: {:?}, dragged_card_index: {}", dragged_card_board_id, dragged_card_index);
                        }
                    }
                });
                app.action_history_manager
                    .new_action(ActionHistory::MoveCardBetweenBoards(
                        dragged_card,
                        dragged_card_board_id,
                        hovered_board_id,
                        dragged_card_index,
                        0,
                    ));
                let info_msg = &format!(
                    "Moved card \"{}\" to board \"{}\"",
                    dragged_card_name, hovered_board.name
                );
                info!("{}", info_msg);
                app.send_info_toast(info_msg, None);
                return;
            } else {
                debug!("Could not find hovered card");
                let error_msg = "Moving card failed, could not find hovered card";
                error!("{}", error_msg);
                app.send_error_toast(error_msg, None);
                return;
            }
        }
        let hovered_card_id = hovered_card_id.unwrap();
        if hovered_card_id == dragged_card_id {
            // the hovered board is empty just move the dragged card to the hovered
            // board (Special case) as it was the last card that was hovered
            app.boards.get_mut_boards().iter_mut().for_each(|board| {
                if board.id == hovered_board_id {
                    board.cards.add_card_at_index(0, dragged_card.clone());
                }
            });
            // remove dragged card from current board
            app.boards.get_mut_boards().iter_mut().for_each(|board| {
                if board.id == dragged_card_board_id {
                    board.cards.remove_card_with_id(dragged_card_id);
                }
            });
            app.action_history_manager
                .new_action(ActionHistory::MoveCardBetweenBoards(
                    dragged_card,
                    dragged_card_board_id,
                    hovered_board_id,
                    dragged_card_index,
                    0,
                ));
            let info_msg = &format!(
                "Moved card \"{}\" to board \"{}\"",
                dragged_card_name, hovered_board.name
            );
            info!("{}", info_msg);
            app.send_info_toast(info_msg, None);
            return;
        }
        let hovered_card_index = hovered_board.cards.get_card_index(hovered_card_id);
        if hovered_card_index.is_none() {
            // case when card was hovered over another board so a card from another board was the last hovered card
            if hovered_board.cards.is_empty() {
                debug!("hovered board is empty");
                // add dragged card to hovered board
                app.boards.get_mut_boards().iter_mut().for_each(|board| {
                    if board.id == hovered_board_id {
                        board.cards.add_card_at_index(0, dragged_card.clone());
                    }
                });
                app.boards.get_mut_boards().iter_mut().for_each(|board| {
                    if board.id == dragged_card_board_id {
                        board.cards.remove_card_with_id(dragged_card_id);
                    }
                });
                app.action_history_manager
                    .new_action(ActionHistory::MoveCardBetweenBoards(
                        dragged_card,
                        dragged_card_board_id,
                        hovered_board_id,
                        dragged_card_index,
                        0,
                    ));
                let info_msg = &format!(
                    "Moved card \"{}\" to board \"{}\"",
                    dragged_card_name, hovered_board.name
                );
                info!("{}", info_msg);
                app.send_info_toast(info_msg, None);
            } else {
                // the hovered board is empty just move the dragged card to the hovered board
                app.boards.get_mut_boards().iter_mut().for_each(|board| {
                    if board.id == hovered_board_id {
                        board.cards.add_card_at_index(0, dragged_card.clone());
                    }
                });
                // remove dragged card from current board
                app.boards.get_mut_boards().iter_mut().for_each(|board| {
                    if board.id == dragged_card_board_id {
                        board.cards.remove_card_with_id(dragged_card_id);
                    }
                });
                app.action_history_manager
                    .new_action(ActionHistory::MoveCardBetweenBoards(
                        dragged_card,
                        dragged_card_board_id,
                        hovered_board_id,
                        dragged_card_index,
                        0,
                    ));
                let info_msg = &format!(
                    "Moved card \"{}\" to board \"{}\"",
                    dragged_card_name, hovered_board.name
                );
                info!("{}", info_msg);
            }
        } else {
            let hovered_card_index = hovered_card_index.unwrap();
            let dragged_card = dragged_card_board
                .cards
                .get_card_with_index(dragged_card_index)
                .unwrap();
            // remove dragged card from current board
            app.boards.get_mut_boards().iter_mut().for_each(|board| {
                if board.id == dragged_card_board_id {
                    board.cards.remove_card_with_id(dragged_card_id);
                }
            });
            // add dragged card to hovered board
            app.boards.get_mut_boards().iter_mut().for_each(|board| {
                if board.id == hovered_board_id {
                    board
                        .cards
                        .add_card_at_index(hovered_card_index, dragged_card.clone());
                }
            });
            app.action_history_manager
                .new_action(ActionHistory::MoveCardBetweenBoards(
                    dragged_card.clone(),
                    dragged_card_board_id,
                    hovered_board_id,
                    dragged_card_index,
                    hovered_card_index,
                ));
            let info_msg = &format!(
                "Moved card \"{}\" to board \"{}\"",
                dragged_card_name, hovered_board.name
            );
            info!("{}", info_msg);
            app.send_info_toast(info_msg, None);
        }
    }
}

pub fn reset_card_drag_mode(app: &mut App) {
    app.state.card_drag_mode = false;
    app.state.hovered_board = None;
    app.state.hovered_card = None;
    app.state.hovered_card_dimensions = None;
}

fn handle_config_menu_action(app: &mut App) -> AppReturn {
    fn reset_config(app: &mut App, reset_keybindings: bool, warning_message: &str) {
        let keybindings = app.config.keybindings.clone();
        app.config = AppConfig::default();
        app.current_theme = Theme::default();
        if !reset_keybindings {
            app.config.keybindings = keybindings;
        }
        app.state.set_focus(Focus::ConfigTable);
        app.state.app_table_states.config.select(Some(0));
        let write_config_status = write_config(&app.config);
        if write_config_status.is_err() {
            error!(
                "Error writing config file: {}",
                write_config_status.clone().unwrap_err()
            );
            app.send_error_toast(
                &format!(
                    "Error writing config file: {}",
                    write_config_status.unwrap_err()
                ),
                None,
            );
        } else {
            warn!("{}", warning_message);
            app.send_warning_toast(warning_message, None);
        }
    }

    if app.state.focus == Focus::SubmitButton {
        reset_config(app, true, "Reset Config and KeyBindings to default");
        return AppReturn::Continue;
    } else if app.state.focus == Focus::ExtraFocus {
        reset_config(app, false, "Reset Config to default");
        return AppReturn::Continue;
    }
    let app_config_list = &app.config.to_view_list();
    if app.state.app_table_states.config.selected().unwrap_or(0) < app_config_list.len() {
        let default_config_item = String::from("");
        let config_item = &app_config_list
            [app.state.app_table_states.config.selected().unwrap_or(0)]
        .first()
        .unwrap_or(&default_config_item);
        let config_enum = ConfigEnum::from_str(config_item);
        if config_enum.is_err() {
            error!("Error checking which config item is being edited");
            return AppReturn::Continue;
        }
        let config_enum = config_enum.unwrap();
        match config_enum {
            ConfigEnum::Keybindings => {
                app.set_view(View::EditKeybindings);
                if app
                    .state
                    .app_table_states
                    .edit_keybindings
                    .selected()
                    .is_none()
                {
                    app.edit_keybindings_next();
                }
            }
            ConfigEnum::DefaultView => {
                if app.state.app_list_states.default_view.selected().is_none() {
                    app.select_default_view_next();
                }
                app.set_popup(PopUp::SelectDefaultView);
            }
            ConfigEnum::AlwaysLoadLastSave
            | ConfigEnum::SaveOnExit
            | ConfigEnum::DisableScrollBar
            | ConfigEnum::DisableAnimations
            | ConfigEnum::AutoLogin
            | ConfigEnum::ShowLineNumbers
            | ConfigEnum::EnableMouseSupport => {
                AppConfig::edit_config(
                    app,
                    config_enum,
                    &app.config.get_toggled_value_as_string(config_enum),
                );
            }
            ConfigEnum::DefaultTheme => {
                app.state.default_theme_mode = true;
                app.set_popup(PopUp::ChangeTheme);
            }
            ConfigEnum::DateFormat => {
                app.set_popup(PopUp::ChangeDateFormatPopup);
            }
            ConfigEnum::DatePickerCalenderFormat => {
                AppConfig::edit_config(
                    app,
                    config_enum,
                    &app.config.get_toggled_value_as_string(config_enum),
                );
                app.widgets
                    .date_time_picker
                    .set_calender_type(app.config.date_picker_calender_format.clone());
            }
            _ => {
                app.set_popup(PopUp::EditGeneralConfig);
            }
        }
    } else {
        debug!(
            "Config item being edited {} is not in the AppConfig list",
            app.state.app_table_states.config.selected().unwrap_or(0)
        );
    }
    AppReturn::Continue
}

async fn handle_main_menu_action(app: &mut App<'_>) -> AppReturn {
    if let Some(selected_index) = app.state.app_list_states.main_menu.selected() {
        let selected_item = app.main_menu.from_index(selected_index);
        match selected_item {
            MainMenuItem::Quit => return handle_exit(app).await,
            MainMenuItem::Config => {
                app.set_view(View::ConfigMenu);
                if app.state.app_table_states.config.selected().is_none() {
                    app.config_next();
                }
            }
            MainMenuItem::View => {
                app.set_view(app.config.default_view);
            }
            MainMenuItem::Help => {
                app.set_view(View::HelpMenu);
            }
            MainMenuItem::LoadSaveLocal => {
                app.set_view(View::LoadLocalSave);
            }
            MainMenuItem::LoadSaveCloud => {
                if app.main_menu.logged_in {
                    app.set_view(View::LoadCloudSave);
                    reset_preview_boards(app);
                    app.dispatch(IoEvent::GetCloudData).await;
                }
            }
        }
    }
    AppReturn::Continue
}

fn handle_default_view_selection(app: &mut App) {
    let all_views = View::all_views_as_string();
    let current_selected_view = app
        .state
        .app_list_states
        .default_view
        .selected()
        .unwrap_or(0);
    if current_selected_view < all_views.len() {
        let selected_view = &all_views[current_selected_view];
        app.config.default_view = View::from_string(selected_view).unwrap_or(View::MainMenu);
        AppConfig::edit_config(app, ConfigEnum::DefaultView, selected_view);
        app.state.app_list_states.default_view.select(Some(0));
        if app.state.app_table_states.config.selected().is_none() {
            app.config_next();
        }
        if !app.state.z_stack.is_empty() {
            app.state.z_stack.pop();
        }
    } else {
        debug!(
            "Selected view {} is not in the list of all View",
            current_selected_view
        );
    }
}

fn handle_change_date_format(app: &mut App) {
    let all_date_formats = DateTimeFormat::get_all_date_formats();
    let current_selected_format = app
        .state
        .app_list_states
        .date_format_selector
        .selected()
        .unwrap_or(0);
    if current_selected_format < all_date_formats.len() {
        let selected_format = &all_date_formats[current_selected_format];
        app.config.date_time_format = *selected_format;
        AppConfig::edit_config(
            app,
            ConfigEnum::DateFormat,
            selected_format.to_human_readable_string(),
        );
        app.state
            .app_list_states
            .date_format_selector
            .select(Some(0));
        if app.state.app_table_states.config.selected().is_none() {
            app.config_next();
        }
        if !app.state.z_stack.is_empty() {
            app.state.z_stack.pop();
        }
    } else {
        debug!(
            "Selected format {} is not in the list of all date formats",
            current_selected_format
        );
    }
}

fn handle_change_view(app: &mut App) {
    let current_index = app
        .state
        .app_list_states
        .default_view
        .selected()
        .unwrap_or(0);
    let all_views = View::all_views_as_string()
        .iter()
        .filter_map(|s| View::from_string(s))
        .collect::<Vec<View>>();

    let current_index = if current_index >= all_views.len() {
        all_views.len() - 1
    } else {
        current_index
    };
    let selected_view = all_views[current_index];
    app.set_view(selected_view);
}

fn handle_edit_keybindings_action(app: &mut App) {
    if app
        .state
        .app_table_states
        .edit_keybindings
        .selected()
        .is_some()
        && app.state.focus != Focus::SubmitButton
    {
        app.set_popup(PopUp::EditSpecificKeyBinding);
    } else if app.state.focus == Focus::SubmitButton {
        app.config.keybindings = KeyBindings::default();
        warn!("Reset keybindings to default");
        app.send_warning_toast("Reset keybindings to default", None);
        app.state.set_focus(Focus::EditKeybindingsTable);
        app.state.app_table_states.edit_keybindings.select(Some(0));
        let write_config_status = write_config(&app.config);
        if let Err(error_message) = write_config_status {
            error!("Error writing config: {}", error_message);
            app.send_error_toast(&format!("Error writing config: {}", error_message), None);
        }
    }
}

pub async fn handle_go_to_previous_view(app: &mut App<'_>) -> AppReturn {
    if let Some(popup) = app.state.z_stack.last() {
        match popup {
            PopUp::EditGeneralConfig => {
                if app.state.current_view == View::CreateTheme {
                    app.close_popup();
                } else {
                    app.set_view(View::ConfigMenu);
                    if app.state.app_table_states.config.selected().is_none() {
                        app.config_next()
                    }
                }
                app.state.text_buffers.general_config.reset();
            }
            PopUp::EditSpecificKeyBinding => {
                app.set_view(View::EditKeybindings);
                app.state.app_table_states.edit_keybindings.select(Some(0));
            }
            PopUp::ViewCard => {
                app.close_popup();
            }
            PopUp::ConfirmDiscardCardChanges => {
                app.close_popup();
            }
            PopUp::FilterByTag => {
                app.state.filter_tags = None;
                app.state.all_available_tags = None;
                app.state.app_list_states.filter_by_tag_list.select(None);
            }
            PopUp::ChangeTheme => {
                let config_theme = {
                    let all_themes = Theme::all_default_themes();
                    let default_theme = app.config.default_theme.clone();
                    all_themes.iter().find(|t| t.name == default_theme).cloned()
                };
                if let Some(theme) = config_theme {
                    app.current_theme = theme;
                }
            }
            _ => {}
        }
        app.close_popup();
        if app.state.app_status == AppStatus::UserInput {
            app.state.app_status = AppStatus::Initialized;
        }
        return AppReturn::Continue;
    }
    match app.state.current_view {
        View::MainMenu => handle_exit(app).await,
        View::EditKeybindings => {
            app.set_view(View::ConfigMenu);
            if app.state.app_table_states.config.selected().is_none() {
                app.config_next()
            }
            AppReturn::Continue
        }
        View::LoadLocalSave => {
            app.state.app_list_states.load_save = ListState::default();
            go_to_previous_view_without_extras(app);
            AppReturn::Continue
        }
        View::Login => {
            reset_login_form(app);
            go_to_previous_view_without_extras(app);
            AppReturn::Continue
        }
        View::SignUp => {
            reset_signup_form(app);
            go_to_previous_view_without_extras(app);
            AppReturn::Continue
        }
        View::ResetPassword => {
            reset_reset_password_form(app);
            go_to_previous_view_without_extras(app);
            AppReturn::Continue
        }
        View::CreateTheme => {
            app.state.theme_being_edited = Theme::default();
            go_to_previous_view_without_extras(app);
            AppReturn::Continue
        }
        _ => {
            go_to_previous_view_without_extras(app);
            AppReturn::Continue
        }
    }
}

fn go_to_previous_view_without_extras(app: &mut App) {
    if app.state.prev_view == Some(app.state.current_view) {
        app.set_view(View::MainMenu);
        if app.state.app_list_states.main_menu.selected().is_none() {
            app.main_menu_next();
        }
    } else {
        let prev_view = app.state.prev_view.unwrap_or(View::MainMenu);
        app.set_view(prev_view);
        if app.state.app_list_states.main_menu.selected().is_none() {
            app.main_menu_next();
        }
    }
}

fn handle_change_card_status(app: &mut App, status: Option<CardStatus>) -> AppReturn {
    let selected_status = if let Some(status) = status {
        status
    } else {
        let current_index = app
            .state
            .app_list_states
            .card_status_selector
            .selected()
            .unwrap_or(0);
        let all_statuses = CardStatus::all();

        let current_index = if current_index >= all_statuses.len() {
            all_statuses.len() - 1
        } else {
            current_index
        };
        all_statuses[current_index].clone()
    };

    if let Some(card_being_edited) = &mut app.state.card_being_edited {
        let card = &mut card_being_edited.1;
        if selected_status == CardStatus::Complete {
            card.date_completed = chrono::Local::now()
                .format(app.config.date_time_format.to_parser_string())
                .to_string();
        } else {
            card.date_completed = FIELD_NOT_SET.to_string();
        }
        card.card_status = selected_status;
        app.close_popup();
        app.state.set_focus(Focus::CardStatus);
        return AppReturn::Continue;
    } else if let Some(current_board_id) = app.state.current_board_id {
        let mut card_found = String::new();
        let boards: &mut Boards = if app.filtered_boards.is_empty() {
            &mut app.boards
        } else {
            &mut app.filtered_boards
        };
        if let Some(current_board) = boards.get_mut_board_with_id(current_board_id) {
            if let Some(current_card_id) = app.state.current_card_id {
                if let Some(current_card) =
                    current_board.cards.get_mut_card_with_id(current_card_id)
                {
                    let temp_old_card = current_card.clone();
                    current_card.card_status = selected_status.clone();
                    if current_card.card_status == CardStatus::Complete {
                        current_card.date_completed = chrono::Local::now()
                            .format(app.config.date_time_format.to_parser_string())
                            .to_string();
                    } else {
                        current_card.date_completed = FIELD_NOT_SET.to_string();
                    }
                    current_card.date_modified = chrono::Local::now()
                        .format(app.config.date_time_format.to_parser_string())
                        .to_string();
                    app.action_history_manager
                        .new_action(ActionHistory::EditCard(
                            temp_old_card,
                            current_card.clone(),
                            current_board_id,
                        ));
                    info!(
                        "Changed status to \"{}\" for card \"{}\"",
                        selected_status, current_card.name
                    );
                    card_found.clone_from(&current_card.name);
                    app.close_popup();
                }
            }
        }
        if !card_found.is_empty() {
            app.send_info_toast(
                &format!(
                    "Changed status to \"{}\" for card \"{}\"",
                    selected_status, card_found
                ),
                None,
            );
        } else {
            app.send_error_toast("Error Could not find current card", None);
        }
    }
    AppReturn::Continue
}

fn handle_change_card_priority(app: &mut App, priority: Option<CardPriority>) -> AppReturn {
    let selected_priority = if let Some(priority) = priority {
        priority
    } else {
        let current_index = app
            .state
            .app_list_states
            .card_priority_selector
            .selected()
            .unwrap_or(0);
        let all_priorities = CardPriority::all();

        let current_index = if current_index >= all_priorities.len() {
            all_priorities.len() - 1
        } else {
            current_index
        };
        all_priorities[current_index].clone()
    };

    if let Some(card_being_edited) = &mut app.state.card_being_edited {
        card_being_edited.1.priority = selected_priority;
        app.close_popup();
        app.state.set_focus(Focus::CardPriority);
        return AppReturn::Continue;
    } else if let Some(current_board_id) = app.state.current_board_id {
        let mut card_found = String::new();
        let boards: &mut Boards = if app.filtered_boards.is_empty() {
            &mut app.boards
        } else {
            &mut app.filtered_boards
        };
        if let Some(current_board) = boards.get_mut_board_with_id(current_board_id) {
            if let Some(current_card_id) = app.state.current_card_id {
                if let Some(current_card) =
                    current_board.cards.get_mut_card_with_id(current_card_id)
                {
                    let temp_old_card = current_card.clone();
                    current_card.priority = selected_priority.clone();
                    current_card.date_modified = chrono::Local::now()
                        .format(app.config.date_time_format.to_parser_string())
                        .to_string();
                    app.action_history_manager
                        .new_action(ActionHistory::EditCard(
                            temp_old_card,
                            current_card.clone(),
                            current_board_id,
                        ));
                    info!(
                        "Changed priority to \"{}\" for card \"{}\"",
                        selected_priority, current_card.name
                    );
                    card_found.clone_from(&current_card.name);
                    app.close_popup();
                }
            }
        }
        if !card_found.is_empty() {
            app.send_info_toast(
                &format!(
                    "Changed priority to \"{}\" for card \"{}\"",
                    selected_priority, card_found
                ),
                None,
            );
        } else {
            app.send_error_toast("Error Could not find current card", None);
        }
    }
    AppReturn::Continue
}

fn handle_edit_general_config(app: &mut App) {
    let config_item_index = app.state.app_table_states.config.selected().unwrap_or(0);
    let config_item_list = AppConfig::to_view_list(&app.config);
    let config_item = &config_item_list[config_item_index];
    let default_key = String::from("");
    let config_item_key = config_item.first().unwrap_or(&default_key);
    let config_enum = ConfigEnum::from_str(config_item_key);
    if config_enum.is_err() {
        error!("Error checking which config item is being edited");
        return;
    }
    let config_enum = config_enum.unwrap();
    let new_value = app.state.text_buffers.general_config.get_joined_lines();
    if new_value.is_empty() {
        error!(
            "Could not find new value for config item {}",
            config_item_key
        );
        app.send_error_toast(
            &format!(
                "Could not find new value for config item {}",
                config_item_key
            ),
            None,
        );
        return;
    }
    AppConfig::edit_config(app, config_enum, &new_value);
    app.state.app_table_states.config.select(Some(0));
    app.state.text_buffers.general_config.reset();
    app.set_view(View::ConfigMenu);
    refresh_visible_boards_and_cards(app);
}

fn handle_edit_specific_keybinding(app: &mut App) {
    if let Some(edited_keybinding) = &app.state.edited_keybinding {
        let selected = app
            .state
            .app_table_states
            .edit_keybindings
            .selected()
            .unwrap();
        if selected < app.config.keybindings.iter().count() {
            match app.config.edit_keybinding(selected, edited_keybinding) {
                Err(e) => {
                    app.send_error_toast(&format!("Error editing Keybinding: {}", e), None);
                }
                Ok(keybinding_enum) => {
                    let value = edited_keybinding
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                    app.send_info_toast(
                        &format!("Keybinding for {} updated to {}", keybinding_enum, value),
                        None,
                    );
                }
            }
        } else {
            error!("Selected Keybinding with id {} not found", selected);
            app.send_error_toast("Selected Keybinding not found", None);
            app.state.app_table_states.edit_keybindings.select(None);
        }
        app.set_view(View::EditKeybindings);
        if app
            .state
            .app_table_states
            .edit_keybindings
            .selected()
            .is_none()
        {
            app.edit_keybindings_next()
        }
        app.state.edited_keybinding = None;
        let write_config_status = write_config(&app.config);
        if let Err(error_message) = write_config_status {
            error!("Error writing config: {}", error_message);
            app.send_error_toast(&format!("Error writing config: {}", error_message), None);
        }
    } else {
        app.set_view(View::EditKeybindings);
        if app
            .state
            .app_table_states
            .edit_keybindings
            .selected()
            .is_none()
        {
            app.edit_keybindings_next()
        }
    }
}

fn handle_new_board_action(app: &mut App) {
    if app.state.focus == Focus::SubmitButton {
        let new_board_name = app.state.text_buffers.board_name.get_joined_lines();
        let new_board_name = new_board_name.trim();
        let new_board_description = app.state.text_buffers.board_description.get_joined_lines();
        let new_board_description = new_board_description.trim();
        let mut same_name_exists = false;
        for board in app.boards.get_boards().iter() {
            if board.name == new_board_name {
                same_name_exists = true;
                break;
            }
        }
        if !new_board_name.is_empty() && !same_name_exists {
            let new_board = Board::new(new_board_name, new_board_description);
            app.boards.add_board(new_board.clone());
            app.action_history_manager
                .new_action(ActionHistory::CreateBoard(new_board.clone()));
            app.state.current_board_id = Some(new_board.id);
            app.set_view(
                *app.state
                    .prev_view
                    .as_ref()
                    .unwrap_or(&app.config.default_view),
            );
        } else {
            warn!("New board name is empty or already exists");
            app.send_warning_toast("New board name is empty or already exists", None);
        }
        app.set_view(
            *app.state
                .prev_view
                .as_ref()
                .unwrap_or(&app.config.default_view),
        );
        if let Some(previous_focus) = &app.state.prev_focus {
            app.state.set_focus(*previous_focus);
        }
        refresh_visible_boards_and_cards(app);
        reset_new_board_form(app);
    } else if app.state.app_status == AppStatus::Initialized {
        app.state.app_status = AppStatus::UserInput;
    }
    if !app.filtered_boards.is_empty() {
        app.state.filter_tags = None;
        app.send_warning_toast("Filter Reset", None);
    }
}

fn handle_general_actions_view_card(app: &mut App) -> AppReturn {
    match app.state.focus {
        Focus::CardPriority => {
            if app.state.card_being_edited.is_none() {
                handle_edit_new_card(app);
            }
            app.set_popup(PopUp::CardPrioritySelector);
            AppReturn::Continue
        }
        Focus::CardStatus => {
            if app.state.card_being_edited.is_none() {
                handle_edit_new_card(app);
            }
            app.set_popup(PopUp::CardStatusSelector);
            AppReturn::Continue
        }
        Focus::CardName | Focus::CardDescription | Focus::CardTags | Focus::CardComments => {
            handle_edit_new_card(app)
        }
        Focus::CardDueDate => {
            if app.state.card_being_edited.is_none() {
                handle_edit_new_card(app);
            }
            app.set_popup(PopUp::DateTimePicker);
            AppReturn::Continue
        }
        Focus::SubmitButton => handle_edit_card_submit(app),
        _ => AppReturn::Continue,
    }
}

fn handle_tag_picker_action(app: &mut App) {
    if let (Some((_, card_being_edited)), Some(card_tag_selected), Some(selected_tag_index)) = (
        &mut app.state.card_being_edited,
        app.state.app_list_states.card_view_tag_list.selected(),
        app.state.app_list_states.tag_picker.selected(),
    ) {
        if card_tag_selected < card_being_edited.tags.len()
            && selected_tag_index < app.widgets.tag_picker.available_tags.len()
        {
            let selected_tag = app.widgets.tag_picker.available_tags[selected_tag_index].clone();
            card_being_edited.tags[card_tag_selected] = selected_tag;
        }
        app.close_popup();
    }
}

fn handle_date_time_picker_action(app: &mut App, key: Option<Key>, action: Option<Action>) {
    let action = key.map_or_else(|| action, |key| app.config.keybindings.key_to_action(&key));
    if let Some(action) = action {
        match action {
            Action::Up => match app.state.focus {
                Focus::DTPCalender => app.widgets.date_time_picker.calender_move_up(),
                Focus::DTPMonth => app.widgets.date_time_picker.month_prv(),
                Focus::DTPYear => app.widgets.date_time_picker.year_prv(),
                Focus::DTPHour => app.widgets.date_time_picker.move_hours_prv(),
                Focus::DTPMinute => app.widgets.date_time_picker.move_minutes_prv(),
                Focus::DTPSecond => app.widgets.date_time_picker.move_seconds_prv(),
                _ => {}
            },
            Action::Down => match app.state.focus {
                Focus::DTPCalender => app.widgets.date_time_picker.calender_move_down(),
                Focus::DTPMonth => app.widgets.date_time_picker.month_next(),
                Focus::DTPYear => app.widgets.date_time_picker.year_next(),
                Focus::DTPHour => app.widgets.date_time_picker.move_hours_next(),
                Focus::DTPMinute => app.widgets.date_time_picker.move_minutes_next(),
                Focus::DTPSecond => app.widgets.date_time_picker.move_seconds_next(),
                _ => {}
            },
            Action::Left => match app.state.focus {
                Focus::DTPCalender => app.widgets.date_time_picker.move_left(),
                Focus::DTPMonth => app.widgets.date_time_picker.month_prv(),
                Focus::DTPYear => app.widgets.date_time_picker.year_prv(),
                Focus::DTPMinute => app.state.set_focus(Focus::DTPHour),
                Focus::DTPSecond => app.state.set_focus(Focus::DTPMinute),
                _ => {}
            },
            Action::Right => match app.state.focus {
                Focus::DTPCalender => app.widgets.date_time_picker.move_right(),
                Focus::DTPMonth => app.widgets.date_time_picker.month_next(),
                Focus::DTPYear => app.widgets.date_time_picker.year_next(),
                Focus::DTPHour => app.state.set_focus(Focus::DTPMinute),
                Focus::DTPMinute => app.state.set_focus(Focus::DTPSecond),
                _ => {}
            },
            Action::Accept => match app.state.focus {
                Focus::DTPCalender
                | Focus::DTPMonth
                | Focus::DTPYear
                | Focus::DTPHour
                | Focus::DTPMinute
                | Focus::DTPSecond => {
                    if let Some((_, card)) = &mut app.state.card_being_edited {
                        if let Some(selected_date) = app.widgets.date_time_picker.selected_date_time
                        {
                            card.due_date = selected_date
                                .format(app.config.date_time_format.to_parser_string())
                                .to_string();
                        } else {
                            card.due_date = chrono::Local::now()
                                .naive_local()
                                .format(app.config.date_time_format.to_parser_string())
                                .to_string()
                        }
                        debug!("Changed due date to {}", card.due_date);
                    }
                    app.widgets.date_time_picker.close_date_picker();
                }
                Focus::DTPToggleTimePicker => {
                    if app.widgets.date_time_picker.time_picker_active {
                        app.widgets.date_time_picker.close_time_picker();
                    } else {
                        app.widgets.date_time_picker.open_time_picker();
                    }
                }
                Focus::NoFocus => {
                    app.widgets.date_time_picker.close_date_picker();
                }
                _ => {}
            },
            Action::NextFocus => handle_next_focus(app),
            Action::PrvFocus => handle_prv_focus(app),
            _ => {} // Handle other actions or do nothing
        }
    } else {
        debug!(
            "Tried to handle date time picker action with no action, key: {:?}, action: {:?}",
            key, action
        );
    }
}

fn handle_new_card_action(app: &mut App) {
    if app.state.focus == Focus::SubmitButton {
        let new_card_name = app.state.text_buffers.card_name.get_joined_lines();
        let new_card_name = new_card_name.trim();
        let new_card_description = app.state.text_buffers.card_description.get_joined_lines();
        let new_card_description = new_card_description.trim();

        let corrected_date_time_format =
            DateTimeFormat::add_time_to_date_format(app.config.date_time_format);

        let new_card_due_date = app
            .widgets
            .date_time_picker
            .get_date_time_as_string(corrected_date_time_format);
        let new_card_due_date = new_card_due_date.trim();
        let mut same_name_exists = false;
        let current_board_id = app.state.current_board_id.unwrap_or((0, 0));
        let current_board = app.boards.get_board_with_id(current_board_id);
        if let Some(current_board) = current_board {
            for card in current_board.cards.get_all_cards() {
                if card.name == new_card_name {
                    same_name_exists = true;
                    break;
                }
            }
        } else {
            debug!("Current board not found");
            app.send_error_toast("Something went wrong", None);
            app.set_view(
                *app.state
                    .prev_view
                    .as_ref()
                    .unwrap_or(&app.config.default_view),
            );
            return;
        }

        if new_card_name.is_empty() || same_name_exists {
            warn!("New card name is empty or already exists");
            app.send_warning_toast("New card name is empty or already exists", None);
            return;
        }

        let new_card = Card::new(
            new_card_name,
            new_card_description,
            new_card_due_date,
            CardPriority::Low,
            vec![],
            vec![],
            app.config.date_time_format,
        );
        let current_board = app.boards.get_mut_board_with_id(current_board_id);
        if let Some(current_board) = current_board {
            current_board.cards.add_card(new_card.clone());
            app.state.current_card_id = Some(new_card.id);
            app.action_history_manager
                .new_action(ActionHistory::CreateCard(new_card, current_board.id));
        } else {
            debug!("Current board not found");
            app.send_error_toast("Something went wrong", None);
            app.set_view(
                *app.state
                    .prev_view
                    .as_ref()
                    .unwrap_or(&app.config.default_view),
            );
            return;
        }
        app.set_view(
            *app.state
                .prev_view
                .as_ref()
                .unwrap_or(&app.config.default_view),
        );

        if let Some(previous_focus) = &app.state.prev_focus {
            app.state.set_focus(*previous_focus);
        }
        refresh_visible_boards_and_cards(app);
        reset_new_card_form(app);
    } else if app.state.focus == Focus::CardDueDate {
        app.set_popup(PopUp::DateTimePicker);
    } else if app.state.app_status == AppStatus::Initialized {
        app.state.app_status = AppStatus::UserInput;
    }
    if !app.filtered_boards.is_empty() {
        app.state.filter_tags = None;
        app.state.all_available_tags = None;
        app.state.app_list_states.filter_by_tag_list.select(None);
        app.send_warning_toast("Filter Reset", None);
    }
}

fn scroll_up(app: &mut App) {
    if app.visible_boards_and_cards.is_empty() {
        refresh_visible_boards_and_cards(app);
        return;
    }
    if app.state.current_board_id.is_none() {
        debug!("No current board id found");
        return;
    }
    let current_board_id = app.state.current_board_id.unwrap();
    let boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    let current_board = boards.get_board_with_id(current_board_id);
    if current_board.is_none() {
        debug!("No current board found in all boards");
        return;
    }
    let current_board = current_board.unwrap();
    let current_visible_cards = app.visible_boards_and_cards.get(&current_board_id);
    if current_visible_cards.is_none() {
        debug!("No current visible cards found");
        refresh_visible_boards_and_cards(app);
        return;
    }
    let current_visible_cards = current_visible_cards.unwrap();
    if current_visible_cards.is_empty() {
        debug!("Current visible cards is empty");
        return;
    }
    let all_card_ids = &current_board.cards.get_all_card_ids();
    let current_window_start_index = all_card_ids
        .iter()
        .position(|&c| c == current_visible_cards[0]);
    if current_window_start_index.is_none() {
        debug!("No current window start index found");
        return;
    }
    let current_window_start_index = current_window_start_index.unwrap();
    if current_window_start_index == 0 {
        return;
    }
    let new_window_start_index = current_window_start_index - 1;
    let new_window_end_index = new_window_start_index + app.config.no_of_cards_to_show as usize;
    let new_window = all_card_ids[new_window_start_index..new_window_end_index].to_vec();
    let board_in_visible = app.visible_boards_and_cards.get_mut(&current_board_id);
    if board_in_visible.is_none() {
        debug!("Board not found in visible boards");
        return;
    }
    let board_in_visible = board_in_visible.unwrap();
    *board_in_visible = new_window;
}

fn scroll_down(app: &mut App) {
    if app.visible_boards_and_cards.is_empty() {
        refresh_visible_boards_and_cards(app);
        return;
    }
    if app.state.current_board_id.is_none() {
        debug!("No current board id found");
        return;
    }
    let boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    let current_board_id = app.state.current_board_id.unwrap();
    let current_board = boards.get_board_with_id(current_board_id);
    if current_board.is_none() {
        debug!("No current board found in all boards");
        return;
    }
    let current_board = current_board.unwrap();
    let current_visible_cards = app.visible_boards_and_cards.get(&current_board_id);
    if current_visible_cards.is_none() {
        debug!("No current visible cards found");
        refresh_visible_boards_and_cards(app);
        return;
    }
    let current_visible_cards = current_visible_cards.unwrap();
    if current_visible_cards.is_empty() {
        debug!("Current visible cards is empty");
        return;
    }
    let all_card_ids = &current_board.cards.get_all_card_ids();
    let current_window_end_index = all_card_ids
        .iter()
        .position(|&c| c == current_visible_cards[current_visible_cards.len() - 1]);
    if current_window_end_index.is_none() {
        debug!("No current window end index found");
        return;
    }
    let current_window_end_index = current_window_end_index.unwrap();
    if current_window_end_index == all_card_ids.len() - 1 {
        return;
    }
    let new_window_end_index = current_window_end_index + 1;
    let new_window_start_index =
        new_window_end_index - (app.config.no_of_cards_to_show - 1) as usize;
    let new_window = all_card_ids[new_window_start_index..=new_window_end_index].to_vec();
    let board_in_visible = app.visible_boards_and_cards.get_mut(&current_board_id);
    if board_in_visible.is_none() {
        debug!("Board not found in visible boards");
        return;
    }
    let board_in_visible = board_in_visible.unwrap();
    *board_in_visible = new_window;
}

fn scroll_right(app: &mut App) {
    if app.state.current_board_id.is_none() {
        debug!("No current board id found");
        return;
    }
    let last_board_in_visible = app.visible_boards_and_cards.keys().last();
    if last_board_in_visible.is_none() {
        debug!("No last board in visible boards found");
        return;
    }
    let boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    let last_board_in_visible = last_board_in_visible.unwrap();
    let last_board_index = boards.get_board_index(*last_board_in_visible);
    if last_board_index.is_none() {
        debug!("No last board index found");
        return;
    }
    let last_board_index = last_board_index.unwrap();
    if last_board_index == boards.len() - 1 {
        return;
    }
    let next_board_index = last_board_index + 1;
    let next_board = boards.get_board_with_index(next_board_index);
    if next_board.is_none() {
        debug!("No next board found");
        return;
    }
    let next_board = next_board.unwrap();
    let next_board_card_ids = next_board.cards.get_all_card_ids();
    let next_board_card_ids = if next_board_card_ids.len() > app.config.no_of_cards_to_show as usize
    {
        next_board_card_ids[0..app.config.no_of_cards_to_show as usize].to_vec()
    } else {
        next_board_card_ids
    };
    let mut new_visible_boards_and_cards = app.visible_boards_and_cards.clone();
    new_visible_boards_and_cards.insert(next_board.id, next_board_card_ids);
    let first_board_in_visible = app.visible_boards_and_cards.keys().next();
    if first_board_in_visible.is_none() {
        debug!("No first board in visible boards found");
        return;
    }
    let first_board_in_visible = first_board_in_visible.unwrap();
    new_visible_boards_and_cards.remove(first_board_in_visible);
    app.visible_boards_and_cards = new_visible_boards_and_cards;
}

fn scroll_left(app: &mut App) {
    if app.state.current_board_id.is_none() {
        debug!("No current board id found");
        return;
    }
    let first_board_in_visible = app.visible_boards_and_cards.keys().next();
    if first_board_in_visible.is_none() {
        debug!("No first board in visible boards found");
        return;
    }
    let boards = if app.filtered_boards.is_empty() {
        &app.boards
    } else {
        &app.filtered_boards
    };
    let first_board_in_visible = first_board_in_visible.unwrap();
    let first_board_index = boards.get_board_index(*first_board_in_visible);
    if first_board_index.is_none() {
        debug!("No first board index found");
        return;
    }
    let first_board_index = first_board_index.unwrap();
    if first_board_index == 0 {
        return;
    }
    let previous_board_index = first_board_index - 1;
    let previous_board = boards.get_board_with_index(previous_board_index);
    if previous_board.is_none() {
        debug!("No previous board found");
        return;
    }
    let previous_board = previous_board.unwrap();
    let previous_board_card_ids = previous_board.cards.get_all_card_ids();
    let previous_board_card_ids =
        if previous_board_card_ids.len() > app.config.no_of_cards_to_show as usize {
            previous_board_card_ids[0..app.config.no_of_cards_to_show as usize].to_vec()
        } else {
            previous_board_card_ids
        };
    let mut new_visible_boards_and_cards = LinkedHashMap::new();
    new_visible_boards_and_cards.insert(previous_board.id, previous_board_card_ids);
    let last_board_in_visible = app.visible_boards_and_cards.keys().last();
    if last_board_in_visible.is_none() {
        debug!("No last board in visible boards found");
        return;
    }
    let last_board_in_visible = last_board_in_visible.unwrap();
    for (board_id, card_ids) in app.visible_boards_and_cards.iter() {
        if board_id == last_board_in_visible {
            break;
        }
        new_visible_boards_and_cards.insert(*board_id, card_ids.clone());
    }
    app.visible_boards_and_cards = new_visible_boards_and_cards;
}

fn reset_mouse(app: &mut App) {
    app.state.current_mouse_coordinates = MOUSE_OUT_OF_BOUNDS_COORDINATES;
    app.state.mouse_focus = None;
}

fn handle_change_theme(app: &mut App, default_theme_mode: bool) -> AppReturn {
    if default_theme_mode {
        app.state.default_theme_mode = false;
        if let Some(config_item_index) = app.state.app_table_states.config.selected() {
            if app.config.to_view_list()[config_item_index]
                .first()
                .unwrap()
                == "Default Theme"
            {
                if let Some(theme_index) = app.state.app_list_states.theme_selector.selected() {
                    if theme_index < app.all_themes.len() {
                        let theme = app.all_themes[theme_index].clone();
                        app.config.default_theme.clone_from(&theme.name);
                        AppConfig::edit_config(app, ConfigEnum::DefaultTheme, theme.name.as_str());
                    } else {
                        debug!("Theme index {} is not in the theme list", theme_index);
                    }
                }
            }
            app.close_popup();
            AppReturn::Continue
        } else {
            debug!("No config index found");
            app.send_error_toast("Something went wrong", None);
            app.close_popup();
            AppReturn::Continue
        }
    } else {
        let selected_item_index = match app.state.app_list_states.theme_selector.selected() {
            Some(index) => index,
            None => {
                debug!("No selected item index found");
                app.send_error_toast("Something went wrong", None);
                app.close_popup();
                return AppReturn::Continue;
            }
        };

        let selected_theme = app
            .all_themes
            .get(selected_item_index)
            .unwrap_or_else(|| &app.all_themes[0])
            .clone();

        app.send_info_toast(
            &format!("Theme changed to \"{}\"", selected_theme.name),
            None,
        );
        app.current_theme = selected_theme;
        app.close_popup();
        AppReturn::Continue
    }
}

fn handle_create_theme_action(app: &mut App) -> AppReturn {
    if let Some(popup) = app.state.z_stack.last() {
        match popup {
            PopUp::EditGeneralConfig => {
                if app.state.text_buffers.general_config.is_empty() {
                    app.send_error_toast("Theme name cannot be empty", None);
                    app.close_popup();
                    return AppReturn::Continue;
                } else {
                    let mut theme_name_duplicate = false;
                    for theme in app.all_themes.iter() {
                        if theme.name == app.state.text_buffers.general_config.get_joined_lines() {
                            theme_name_duplicate = true;
                            break;
                        }
                    }
                    if theme_name_duplicate {
                        app.send_error_toast("Theme name already exists", None);
                        app.close_popup();
                        return AppReturn::Continue;
                    }
                    app.state.theme_being_edited.name =
                        app.state.text_buffers.general_config.get_joined_lines();
                }
                app.close_popup();
            }
            PopUp::EditThemeStyle => {
                match app.state.focus {
                    Focus::SubmitButton => {
                        let all_color_options =
                            TextColorOptions::iter().collect::<Vec<TextColorOptions>>();
                        let all_modifier_options =
                            TextModifierOptions::iter().collect::<Vec<TextModifierOptions>>();
                        for list_state in app.state.app_list_states.edit_specific_style.iter_mut() {
                            if list_state.selected().is_none() {
                                list_state.select(Some(0));
                            }
                        }
                        let selected_fg_index =
                            app.state.app_list_states.edit_specific_style[0].selected();
                        let selected_bg_index =
                            app.state.app_list_states.edit_specific_style[1].selected();
                        let selected_modifier_index =
                            app.state.app_list_states.edit_specific_style[2].selected();

                        let theme_style_being_edited_index =
                            app.state.app_table_states.theme_editor.selected();
                        if theme_style_being_edited_index.is_none() {
                            debug!("No theme style being edited index found");
                            app.send_error_toast("Something went wrong", None);
                            app.close_popup();
                            return AppReturn::Continue;
                        }
                        let theme_style_being_edited_index =
                            theme_style_being_edited_index.unwrap();

                        let fg_color = if let TextColorOptions::HEX(_, _, _) =
                            all_color_options[selected_fg_index.unwrap()]
                        {
                            let fg_hex_value = app
                                .state
                                .text_buffers
                                .theme_editor_fg_hex
                                .get_joined_lines();

                            if let Some((r, g, b)) = parse_hex_to_rgb(&fg_hex_value) {
                                Color::Rgb(r, g, b)
                            } else {
                                app.send_error_toast("Invalid hex value", None);
                                return AppReturn::Continue;
                            }
                        } else {
                            Color::from(all_color_options[selected_fg_index.unwrap()])
                        };
                        let bg_color = if let TextColorOptions::HEX(_, _, _) =
                            all_color_options[selected_bg_index.unwrap()]
                        {
                            let bg_hex_value = app
                                .state
                                .text_buffers
                                .theme_editor_bg_hex
                                .get_joined_lines();

                            if let Some((r, g, b)) = parse_hex_to_rgb(&bg_hex_value) {
                                Color::Rgb(r, g, b)
                            } else {
                                app.send_error_toast("Invalid hex value", None);
                                return AppReturn::Continue;
                            }
                        } else {
                            Color::from(all_color_options[selected_bg_index.unwrap()])
                        };
                        let modifier = ratatui::style::Modifier::from(
                            all_modifier_options[selected_modifier_index.unwrap()].clone(),
                        );
                        let theme_enum = ThemeEnum::iter().nth(theme_style_being_edited_index);
                        if theme_enum.is_none() {
                            debug!(
                                "No theme enum found for index {}",
                                theme_style_being_edited_index
                            );
                            app.send_error_toast("Something went wrong", None);
                            app.close_popup();
                            return AppReturn::Continue;
                        }
                        let theme_enum = theme_enum.unwrap();
                        Theme::update_style(
                            app.state.theme_being_edited.get_mut_style(theme_enum),
                            Some(fg_color),
                            Some(bg_color),
                            Some(modifier),
                        );
                        app.state.app_list_states.edit_specific_style[0].select(None);
                        app.state.app_list_states.edit_specific_style[1].select(None);
                        app.state.app_list_states.edit_specific_style[2].select(None);
                        app.state.text_buffers.theme_editor_fg_hex.reset();
                        app.state.text_buffers.theme_editor_bg_hex.reset();
                        app.close_popup();
                        app.state.set_focus(Focus::ThemeEditor);
                    }
                    Focus::StyleEditorFG => {
                        let selected_index =
                            app.state.app_list_states.edit_specific_style[0].selected();
                        if selected_index.is_none() {
                            return AppReturn::Continue;
                        }
                        let selected_index = selected_index.unwrap();
                        let all_color_options =
                            TextColorOptions::iter().collect::<Vec<TextColorOptions>>();
                        let selected_color = &all_color_options[selected_index];
                        if let TextColorOptions::HEX(_, _, _) = selected_color {
                            app.set_popup(PopUp::CustomHexColorPromptFG);
                            app.state.set_focus(Focus::TextInput);
                            return AppReturn::Continue;
                        }
                    }
                    Focus::StyleEditorBG => {
                        let selected_index =
                            app.state.app_list_states.edit_specific_style[1].selected();
                        if selected_index.is_none() {
                            return AppReturn::Continue;
                        }
                        let selected_index = selected_index.unwrap();
                        let all_color_options =
                            TextColorOptions::iter().collect::<Vec<TextColorOptions>>();
                        let selected_color = &all_color_options[selected_index];
                        if let TextColorOptions::HEX(_, _, _) = selected_color {
                            app.set_popup(PopUp::CustomHexColorPromptBG);
                            app.state.set_focus(Focus::TextInput);
                            return AppReturn::Continue;
                        }
                    }
                    _ => {}
                }
                return AppReturn::Continue;
            }
            _ => {}
        }
    } else if app.state.focus == Focus::SubmitButton {
        let all_default_theme_names = Theme::all_default_themes()
            .iter()
            .map(|theme| theme.name.clone())
            .collect::<Vec<String>>();
        if all_default_theme_names.contains(&app.state.theme_being_edited.name) {
            app.send_error_toast("Theme name cannot be the same as \"Default Theme\"", None);
            return AppReturn::Continue;
        }
        app.set_popup(PopUp::SaveThemePrompt);
    } else if app.state.focus == Focus::ThemeEditor
        && app.state.app_table_states.theme_editor.selected().is_some()
    {
        let selected_item_index = app.state.app_table_states.theme_editor.selected().unwrap();
        if selected_item_index == 0 {
            app.state.app_table_states.config.select(None);
            app.set_popup(PopUp::EditGeneralConfig);
        } else {
            app.set_popup(PopUp::EditThemeStyle);
        }
    } else if app.state.focus == Focus::ExtraFocus {
        app.state.theme_being_edited = Theme::default();
        app.state.text_buffers.theme_editor_fg_hex.reset();
        app.state.text_buffers.theme_editor_bg_hex.reset();
        app.send_info_toast("Editor reset to default", None);
    }
    AppReturn::Continue
}

fn handle_next_focus(app: &mut App) {
    if app.config.enable_mouse_support {
        reset_mouse(app)
    }
    let available_targets = if let Some(popup) = app.state.z_stack.checked_control_last() {
        popup.get_available_targets()
    } else {
        app.state.current_view.get_available_targets()
    };
    if !available_targets.contains(&app.state.focus) {
        if available_targets.is_empty() {
            app.state.set_focus(Focus::NoFocus);
        } else {
            app.state.set_focus(available_targets[0]);
        }
        return;
    }
    let mut next_focus = app.state.focus.next(&available_targets);
    // Special Handling
    if app.state.card_being_edited.is_none()
        && app.state.z_stack.last() == Some(&PopUp::ViewCard)
        && next_focus == Focus::SubmitButton
    {
        next_focus = Focus::CardName;
    }
    if app.state.z_stack.last() == Some(&PopUp::DateTimePicker)
        && !app.widgets.date_time_picker.time_picker_active
    {
        match next_focus {
            Focus::DTPHour | Focus::DTPMinute | Focus::DTPSecond => {
                next_focus = Focus::DTPCalender;
            }
            _ => {}
        }
    }

    if next_focus != Focus::NoFocus {
        app.state.set_focus(next_focus);
    }
    if app.state.z_stack.last() == Some(&PopUp::CommandPalette) {
        CommandPaletteWidget::reset_list_states(&mut app.state)
    }
}

fn handle_prv_focus(app: &mut App) {
    if app.config.enable_mouse_support {
        reset_mouse(app)
    }
    let available_targets = if let Some(popup) = app.state.z_stack.checked_control_last() {
        PopUp::get_available_targets(popup)
    } else {
        View::get_available_targets(&app.state.current_view)
    };
    if !available_targets.contains(&app.state.focus) {
        if available_targets.is_empty() {
            app.state.set_focus(Focus::NoFocus);
        } else {
            app.state
                .set_focus(available_targets[available_targets.len() - 1]);
        }
        return;
    }
    let mut prv_focus = app.state.focus.prev(&available_targets);

    // Special Handling
    if app.state.card_being_edited.is_none()
        && app.state.z_stack.last() == Some(&PopUp::ViewCard)
        && prv_focus == Focus::SubmitButton
    {
        prv_focus = Focus::CardComments;
    }
    if app.state.z_stack.last() == Some(&PopUp::DateTimePicker)
        && !app.widgets.date_time_picker.time_picker_active
    {
        match prv_focus {
            Focus::DTPHour | Focus::DTPMinute | Focus::DTPSecond => {
                prv_focus = Focus::DTPCalender;
            }
            _ => {}
        }
    }

    if prv_focus != Focus::NoFocus {
        app.state.set_focus(prv_focus);
    }
    if app.state.z_stack.last() == Some(&PopUp::CommandPalette) {
        CommandPaletteWidget::reset_list_states(&mut app.state)
    }
}

fn handle_save_theme_prompt(app: &mut App) {
    if app.state.focus == Focus::SubmitButton {
        let theme_name = app.state.theme_being_edited.name.clone();
        let save_theme_status = save_theme(app.state.theme_being_edited.clone());
        if save_theme_status.is_err() {
            debug!("Failed to save theme: {}", save_theme_status.unwrap_err());
            app.send_error_toast("Failed to save theme", None);
            return;
        } else {
            app.send_info_toast(&format!("Saved theme {}", theme_name), None);
        }
    } else {
        app.send_warning_toast("Theme not saved to file, it will be lost on exit, You can still use it in the current session",
        Some(Duration::from_secs(5)));
    }
    let default_view = app.config.default_view;
    app.set_view(default_view);
    app.all_themes.push(app.state.theme_being_edited.clone());
    app.state.theme_being_edited = Theme::default();
    app.state.text_buffers.theme_editor_fg_hex.reset();
    app.state.text_buffers.theme_editor_bg_hex.reset();
    app.close_popup();
    handle_prv_focus(app);
}

fn handle_custom_hex_color_prompt(app: &mut App, fg: bool) -> AppReturn {
    let fg_hex_value = app
        .state
        .text_buffers
        .theme_editor_fg_hex
        .get_joined_lines();
    let bg_hex_value = app
        .state
        .text_buffers
        .theme_editor_bg_hex
        .get_joined_lines();

    let hex_value: &str = if fg {
        fg_hex_value.trim()
    } else {
        bg_hex_value.trim()
    };

    // validate hex value
    if parse_hex_to_rgb(hex_value).is_none() {
        app.send_error_toast("Invalid hex value", None);
        return AppReturn::Continue;
    };

    app.close_popup();
    AppReturn::Continue
}

fn handle_theme_maker_scroll_up(app: &mut App) {
    let style_index = if app.state.focus == Focus::StyleEditorFG {
        0
    } else if app.state.focus == Focus::StyleEditorBG {
        1
    } else if app.state.focus == Focus::StyleEditorModifier {
        2
    } else {
        debug!("Invalid focus found for theme maker scroll up");
        return;
    };
    if app.state.app_list_states.edit_specific_style[style_index]
        .selected()
        .is_none()
    {
        app.state.app_list_states.edit_specific_style[style_index].select_first();
    }
    let current_selected_index = app.state.app_list_states.edit_specific_style[style_index]
        .selected()
        .unwrap();
    let total_length = if app.state.focus == Focus::StyleEditorModifier {
        TextModifierOptions::iter().count()
    } else {
        TextColorOptions::iter().count()
    };
    let next_selection = if current_selected_index > 0 {
        current_selected_index - 1
    } else {
        total_length - 1
    };
    app.state.app_list_states.edit_specific_style[style_index].select(Some(next_selection));
}

fn handle_theme_maker_scroll_down(app: &mut App) {
    let style_index = if app.state.focus == Focus::StyleEditorFG {
        0
    } else if app.state.focus == Focus::StyleEditorBG {
        1
    } else if app.state.focus == Focus::StyleEditorModifier {
        2
    } else {
        debug!("Invalid focus found for theme maker scroll down");
        return;
    };
    if app.state.app_list_states.edit_specific_style[style_index]
        .selected()
        .is_none()
    {
        app.state.app_list_states.edit_specific_style[style_index].select_first();
    }
    let current_selected_index = app.state.app_list_states.edit_specific_style[style_index]
        .selected()
        .unwrap();
    let total_length = if app.state.focus == Focus::StyleEditorModifier {
        TextModifierOptions::iter().count()
    } else {
        TextColorOptions::iter().count()
    };
    let next_selection = if current_selected_index < total_length - 1 {
        current_selected_index + 1
    } else {
        0
    };
    app.state.app_list_states.edit_specific_style[style_index].select(Some(next_selection));
}

fn handle_edit_new_card(app: &mut App) -> AppReturn {
    app.state.app_status = AppStatus::UserInput;
    if app.state.current_board_id.is_none() || app.state.current_card_id.is_none() {
        app.send_error_toast("No card selected to edit", None);
        app.close_popup();
        return AppReturn::Continue;
    }
    let board = app
        .boards
        .get_board_with_id(app.state.current_board_id.unwrap());
    if board.is_none() {
        app.send_error_toast("No board found for editing card", None);
        app.close_popup();
        return AppReturn::Continue;
    }
    let card = board
        .unwrap()
        .cards
        .get_card_with_id(app.state.current_card_id.unwrap());
    if card.is_none() {
        app.send_error_toast("No card found for editing", None);
        app.close_popup();
        return AppReturn::Continue;
    }
    let card = card.unwrap();
    if app.state.card_being_edited.is_some()
        && app.state.card_being_edited.as_ref().unwrap().1.id == card.id
    {
        return AppReturn::Continue;
    }
    app.state.card_being_edited = Some((app.state.current_board_id.unwrap(), card.clone()));
    app.state.text_buffers.card_name.reset();
    app.state.text_buffers.card_name.insert_str(&card.name);
    // To avoid reversing the order of the description we create a new TextBox, as insert_str reverses the order (adds them one by one)
    app.state.text_buffers.card_description =
        TextBox::from_string_with_newline_sep(card.description.clone(), false);
    app.state.text_buffers.card_tags = Vec::new();
    card.tags.iter().for_each(|tag| {
        app.state
            .text_buffers
            .card_tags
            .push(TextBox::from_string_with_newline_sep(tag.to_string(), true));
    });
    app.state.text_buffers.card_comments = Vec::new();
    card.comments.iter().for_each(|comment| {
        app.state
            .text_buffers
            .card_comments
            .push(TextBox::from_string_with_newline_sep(
                comment.to_string(),
                true,
            ));
    });
    if card.due_date != FIELD_NOT_SET && !card.due_date.is_empty() {
        if let Ok(current_format) = date_format_finder(card.due_date.trim()) {
            app.widgets.date_time_picker.selected_date_time = match NaiveDateTime::parse_from_str(
                card.due_date.trim(),
                current_format.to_parser_string(),
            ) {
                Ok(date_time) => Some(date_time),
                Err(_) => None,
            };
        }
    }
    info!("Editing Card '{}'", card.name);
    app.send_info_toast(&format!("Editing Card '{}'", card.name), None);
    AppReturn::Continue
}

fn handle_edit_card_submit(app: &mut App) -> AppReturn {
    let mut send_warning_toast = false;
    let mut warning_due_date = String::new();
    if app.state.current_board_id.is_none() {
        return AppReturn::Continue;
    }
    if app.state.current_card_id.is_none() {
        return AppReturn::Continue;
    }
    let board = app
        .boards
        .get_mut_board_with_id(app.state.current_board_id.unwrap());
    if board.is_none() {
        return AppReturn::Continue;
    }
    let board = board.unwrap();
    let card = board
        .cards
        .get_mut_card_with_id(app.state.current_card_id.unwrap());
    if card.is_none() {
        return AppReturn::Continue;
    }
    let card = card.unwrap();
    let mut edited_card = if let Some(edited_card) = &app.state.card_being_edited {
        edited_card.1.clone()
    } else {
        debug!("No card being edited found");
        return AppReturn::Continue;
    };
    let card_due_date = edited_card.due_date.clone();
    let parsed_due_date = date_format_converter(card_due_date.trim(), app.config.date_time_format);
    let parsed_date = match parsed_due_date {
        Ok(date) => {
            if date.is_empty() {
                FIELD_NOT_SET.to_string()
            } else {
                date
            }
        }
        Err(_) => {
            if card_due_date.trim() != FIELD_NOT_SET {
                send_warning_toast = true;
                warning_due_date = card_due_date;
            }
            FIELD_NOT_SET.to_string()
        }
    };
    edited_card.due_date = parsed_date;
    edited_card.date_modified = chrono::Local::now()
        .format(app.config.date_time_format.to_parser_string())
        .to_string();
    app.action_history_manager
        .new_action(ActionHistory::EditCard(
            card.clone(),
            edited_card.clone(),
            board.id,
        ));
    *card = edited_card;

    let description = app.state.text_buffers.card_description.get_joined_lines();
    card.description = description;

    let card_name = app.state.text_buffers.card_name.get_joined_lines();
    card.name.clone_from(&card_name);
    app.state.card_being_edited = None;
    if send_warning_toast {
        let all_date_formats = DateTimeFormat::get_all_date_formats()
            .iter()
            .map(|x| x.to_human_readable_string())
            .collect::<Vec<&str>>()
            .join(", ");
        app.send_warning_toast(
            &format!(
                "Invalid date format '{}'. Please use any of the following {}. Date has been reset and other changes have been saved.",
                warning_due_date, all_date_formats
            ),
            Some(Duration::from_secs(10)),
        );
        warn!(
            "Invalid date format '{}'. Please use any of the following {}. Date has been reset and other changes have been saved.",
            warning_due_date, all_date_formats
        );
    }
    app.send_info_toast(&format!("Changes to Card '{}' saved", card_name), None);
    app.state.set_focus(Focus::CardName);
    app.state.app_status = AppStatus::Initialized;
    let calculated_tags = app.calculate_tags();
    if calculated_tags.is_empty() {
        app.state.all_available_tags = None;
    } else {
        app.state.all_available_tags = Some(calculated_tags);
    };
    if !app.filtered_boards.is_empty() {
        filter_boards(app);
    }
    AppReturn::Continue
}

fn handle_filter_by_tag(app: &mut App) {
    match app.state.focus {
        Focus::FilterByTagPopup => {
            let selected_index = app.state.app_list_states.filter_by_tag_list.selected();
            if selected_index.is_none() {
                return;
            }
            let selected_index = selected_index.unwrap();
            let all_tags = app.state.all_available_tags.clone();
            if all_tags.is_none() {
                debug!("No tags found to select");
                return;
            }
            let all_tags = all_tags.unwrap();
            if selected_index >= all_tags.len() {
                debug!("Selected index is out of bounds");
                return;
            }
            let selected_tag = all_tags[selected_index].0.clone();
            if app.state.filter_tags.is_some() {
                let mut filter_tags = app.state.filter_tags.clone().unwrap();
                if filter_tags.contains(&selected_tag) {
                    app.send_warning_toast(
                        &format!("Removed tag \"{}\" from filter", selected_tag),
                        None,
                    );
                    filter_tags.retain(|tag| tag != &selected_tag);
                } else {
                    app.send_info_toast(&format!("Added tag \"{}\" to filter", selected_tag), None);
                    filter_tags.push(selected_tag);
                }
                app.state.filter_tags = Some(filter_tags);
            } else {
                let filter_tags = vec![selected_tag.clone()];
                app.send_info_toast(&format!("Added tag \"{}\" to filter", selected_tag), None);
                app.state.filter_tags = Some(filter_tags);
            }
        }
        Focus::SubmitButton => filter_boards(app),
        _ => {}
    }
}

fn filter_boards(app: &mut App) {
    if app.state.filter_tags.is_none() {
        app.send_warning_toast("No tags selected to filter", None);
        app.close_popup();
        return;
    }
    let all_boards = app.boards.clone();
    app.state.current_board_id = None;
    app.state.current_card_id = None;
    let filter_tags = app.state.filter_tags.clone().unwrap();
    let mut filtered_boards = Vec::new();
    for board in all_boards.get_boards() {
        let mut filtered_cards = Vec::new();
        for card in board.cards.get_all_cards() {
            let mut card_tags = card.tags.clone();
            card_tags.retain(|tag| filter_tags.contains(&tag.to_lowercase()));
            if !card_tags.is_empty() {
                filtered_cards.push(card.clone());
            }
        }
        if !filtered_cards.is_empty() {
            filtered_boards.push(Board {
                id: board.id,
                name: board.name.clone(),
                description: board.description.clone(),
                cards: Cards::from(filtered_cards),
            });
        }
    }
    app.filtered_boards = Boards::from(filtered_boards);
    refresh_visible_boards_and_cards(app);
    app.send_info_toast(
        &format!(
            "Filtered by {} tags",
            app.state.filter_tags.clone().unwrap().len()
        ),
        None,
    );
    app.close_popup();
    app.state.app_list_states.filter_by_tag_list.select(None);
}

fn handle_command_palette_card_selection(app: &mut App) {
    reset_mouse(app);
    refresh_visible_boards_and_cards(app);
    let card_details_index = app
        .state
        .app_list_states
        .command_palette_card_search
        .selected();
    if card_details_index.is_none() {
        return;
    }
    let card_details_index = card_details_index.unwrap();
    let all_card_details = app.widgets.command_palette.card_search_results.clone();
    if all_card_details.is_none() {
        debug!("No card details found to select");
        return;
    }
    let all_card_details = all_card_details.unwrap();
    if card_details_index >= all_card_details.len() {
        debug!("Selected index is out of bounds");
        return;
    }
    let card_id = all_card_details[card_details_index].1;
    app.state.current_board_id = Some(app.boards.find_board_with_card_id(card_id).unwrap().1.id);
    app.state.current_card_id = Some(card_id);
    app.set_popup(PopUp::ViewCard);
}

fn handle_command_palette_board_selection(app: &mut App) {
    reset_mouse(app);
    refresh_visible_boards_and_cards(app);
    let board_details_index = app
        .state
        .app_list_states
        .command_palette_board_search
        .selected();
    if board_details_index.is_none() {
        return;
    }
    let board_details_index = board_details_index.unwrap();
    let all_board_details = app.widgets.command_palette.board_search_results.clone();
    if all_board_details.is_none() {
        debug!("No board details found to select");
        return;
    }
    let all_board_details = all_board_details.unwrap();
    if board_details_index >= all_board_details.len() {
        debug!("Selected index is out of bounds");
        return;
    }
    let board_id = all_board_details[board_details_index].1;
    let mut number_of_times_to_go_right = 0;
    for (board_index, board) in app.boards.get_boards().iter().enumerate() {
        if board.id == board_id {
            number_of_times_to_go_right = board_index;
            break;
        }
    }
    for _ in 0..number_of_times_to_go_right {
        go_right(app);
    }
    app.state.set_focus(Focus::Body);
}

pub async fn handle_login_submit_action(app: &mut App<'_>) {
    app.dispatch(IoEvent::Login(
        app.state.text_buffers.email_id.get_joined_lines(),
        app.state.text_buffers.password.get_joined_lines(),
    ))
    .await;
}

pub async fn handle_signup_submit_action(app: &mut App<'_>) {
    app.dispatch(IoEvent::SignUp(
        app.state.text_buffers.email_id.get_joined_lines(),
        app.state.text_buffers.password.get_joined_lines(),
        app.state.text_buffers.confirm_password.get_joined_lines(),
    ))
    .await;
}

pub async fn handle_reset_password_submit_action(app: &mut App<'_>) {
    app.dispatch(IoEvent::ResetPassword(
        app.state
            .text_buffers
            .reset_password_link
            .get_joined_lines(),
        app.state.text_buffers.password.get_joined_lines(),
        app.state.text_buffers.confirm_password.get_joined_lines(),
    ))
    .await;
}

pub async fn handle_send_reset_password_link_action(app: &mut App<'_>) {
    app.dispatch(IoEvent::SendResetPasswordEmail(
        app.state.text_buffers.email_id.get_joined_lines(),
    ))
    .await;
}

fn reset_new_board_form(app: &mut App) {
    app.state.text_buffers.board_name.reset();
    app.state.text_buffers.board_description.reset();
}

fn reset_new_card_form(app: &mut App) {
    app.state.text_buffers.card_name.reset();
    app.state.text_buffers.card_description.reset();
    app.widgets.date_time_picker.reset();
}

fn reset_login_form(app: &mut App) {
    app.state.text_buffers.email_id.reset();
    app.state.text_buffers.password.reset();
    app.state.show_password = false;
}

fn reset_signup_form(app: &mut App) {
    app.state.text_buffers.email_id.reset();
    app.state.text_buffers.password.reset();
    app.state.text_buffers.confirm_password.reset();
    app.state.show_password = false;
}

fn reset_reset_password_form(app: &mut App) {
    app.state.text_buffers.email_id.reset();
    app.state.text_buffers.reset_password_link.reset();
    app.state.text_buffers.password.reset();
    app.state.text_buffers.confirm_password.reset();
    app.state.show_password = false;
}

async fn handle_login_action(app: &mut App<'_>) {
    match app.state.focus {
        Focus::CloseButton => {
            app.state.theme_being_edited = Theme::default();
            reset_login_form(app);
            handle_go_to_previous_view(app).await;
            exit_user_input_mode(app);
        }
        Focus::SubmitButton => {
            handle_login_submit_action(app).await;
            exit_user_input_mode(app);
        }
        Focus::ExtraFocus => {
            app.state.show_password = !app.state.show_password;
        }
        Focus::Title => {
            app.set_view(View::MainMenu);
            reset_login_form(app);
            exit_user_input_mode(app);
        }
        Focus::EmailIDField | Focus::PasswordField => {
            enter_user_input_mode(app);
        }
        _ => {}
    };
}

fn exit_user_input_mode(app: &mut App) {
    if app.state.app_status == AppStatus::UserInput {
        app.state.app_status = AppStatus::Initialized;
        info!("Exiting user input mode");
    }
}

async fn handle_signup_action(app: &mut App<'_>) {
    match app.state.focus {
        Focus::CloseButton => {
            reset_signup_form(app);
            handle_go_to_previous_view(app).await;
            exit_user_input_mode(app);
        }
        Focus::SubmitButton => {
            handle_signup_submit_action(app).await;
            exit_user_input_mode(app);
        }
        Focus::ExtraFocus => {
            app.state.show_password = !app.state.show_password;
        }
        Focus::Title => {
            app.set_view(View::MainMenu);
            reset_signup_form(app);
            exit_user_input_mode(app);
        }
        Focus::EmailIDField | Focus::PasswordField | Focus::ConfirmPasswordField => {
            enter_user_input_mode(app);
        }
        _ => {}
    }
}

fn enter_user_input_mode(app: &mut App) {
    if app.state.app_status != AppStatus::UserInput {
        app.state.app_status = AppStatus::UserInput;
        info!("Taking user input");
    }
}

async fn handle_reset_password_action(app: &mut App<'_>) {
    match app.state.focus {
        Focus::CloseButton => {
            reset_reset_password_form(app);
            handle_go_to_previous_view(app).await;
            exit_user_input_mode(app);
        }
        Focus::Title => {
            app.set_view(View::MainMenu);
            reset_reset_password_form(app);
            exit_user_input_mode(app);
        }
        Focus::EmailIDField
        | Focus::ResetPasswordLinkField
        | Focus::PasswordField
        | Focus::ConfirmPasswordField => {
            enter_user_input_mode(app);
        }
        Focus::ExtraFocus => {
            app.state.show_password = !app.state.show_password;
        }
        Focus::SubmitButton => {
            handle_reset_password_submit_action(app).await;
        }
        Focus::SendResetPasswordLinkButton => {
            handle_send_reset_password_link_action(app).await;
        }
        _ => {}
    }
}

pub fn reset_preview_boards(app: &mut App) {
    app.preview_boards_and_cards = None;
    app.state.preview_file_name = None;
    app.state.preview_visible_boards_and_cards = LinkedHashMap::new();
}
