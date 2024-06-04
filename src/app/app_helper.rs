use super::{
    actions::Action,
    date_format_converter, handle_exit,
    kanban::{Board, Boards, Card, CardPriority, CardStatus, Cards},
    state::{AppStatus, Focus, UiMode},
    App, AppReturn, DateFormat, MainMenuItem, PopupMode,
};
use crate::{
    app::{state::KeyBindings, ActionHistory, AppConfig, ConfigEnum, PathCheckState},
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
        text_box::{TextBox, TextBoxScroll},
        widgets::{CommandPaletteWidget, ToastType, ToastWidget},
        TextColorOptions, TextModifierOptions, Theme,
    },
};
use chrono::Utc;
use linked_hash_map::LinkedHashMap;
use log::{debug, error, info, warn};
use ratatui::{style::Color, widgets::ListState};
use std::{fs, path::Path, str::FromStr, time::Duration};

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
    let current_board_index = current_visible_boards
        .iter()
        .position(|(board_id, _)| *board_id == current_board_id);
    if current_board_index.is_none() {
        debug!("Cannot go right: current board not found, trying to assign to the first board");
        if current_visible_boards.is_empty() {
            debug!("Cannot go right: current board not found, no visible boards found");
            app.send_error_toast("Cannot go right: Something went wrong", None);
            return;
        } else {
            app.state.current_board_id = Some(*current_visible_boards.keys().next().unwrap());
        }
    }
    let current_board_index = current_board_index.unwrap();
    if current_board_index == current_visible_boards.len() - 1 {
        let current_board_index_in_all_boards = boards.get_board_index(current_board_id);
        if current_board_index_in_all_boards.is_none() {
            debug!("Cannot go right: current board not found");
            app.send_error_toast("Cannot go right: Something went wrong", None);
            return;
        }
        let current_board_index_in_all_boards = current_board_index_in_all_boards.unwrap();
        if current_board_index_in_all_boards == (boards.len() - 1) {
            app.send_error_toast("Cannot go right: Already at the last board", None);
            return;
        }
        let next_board_index = current_board_index_in_all_boards + 1;
        let next_board = &boards.get_board_with_index(next_board_index).unwrap();
        let next_board_card_ids: Vec<(u64, u64)> = next_board.cards.get_all_card_ids();
        app.visible_boards_and_cards
            .insert(next_board.id, next_board_card_ids.clone());
        let first_board_id = *app.visible_boards_and_cards.iter().next().unwrap().0;
        app.visible_boards_and_cards.remove(&first_board_id);
        app.state.current_board_id = Some(next_board.id);
        if next_board_card_ids.is_empty() {
            app.state.current_card_id = None;
        } else {
            app.state.current_card_id = Some(next_board_card_ids[0]);
        }
    } else {
        let next_board_id = *current_visible_boards
            .iter()
            .nth(current_board_index + 1)
            .unwrap()
            .0;
        app.state.current_board_id = Some(next_board_id);
        let current_board_cards = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == next_board_id)
            .unwrap()
            .1
            .clone();
        if current_board_cards.is_empty() {
            app.state.current_card_id = None;
        } else {
            app.state.current_card_id = Some(current_board_cards[0]);
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
    let current_board_index = current_visible_boards
        .iter()
        .position(|(board_id, _)| *board_id == current_board_id);
    if current_board_index.is_none() {
        debug!("Cannot go left: current board not found");
        app.send_error_toast("Cannot go left: Something went wrong", None);
        return;
    }
    let current_board_index = current_board_index.unwrap();
    if current_board_index == 0 {
        let current_board_index_in_all_boards = boards.get_board_index(current_board_id);
        if current_board_index_in_all_boards.is_none() {
            debug!("Cannot go left: current board not found");
            app.send_error_toast("Cannot go left: Something went wrong", None);
            return;
        }
        let current_board_index_in_all_boards = current_board_index_in_all_boards.unwrap();
        if current_board_index_in_all_boards == 0 {
            app.send_error_toast("Cannot go left: Already at the first board", None);
            return;
        }
        let previous_board_index = current_board_index_in_all_boards - 1;
        let previous_board = boards.get_board_with_index(previous_board_index).unwrap();
        let previous_board_card_ids: Vec<(u64, u64)> = previous_board.cards.get_all_card_ids();
        let mut new_visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
            LinkedHashMap::new();
        new_visible_boards_and_cards.insert(previous_board.id, previous_board_card_ids.clone());
        for (board_id, card_ids) in current_visible_boards
            .iter()
            .take(current_visible_boards.len() - 1)
        {
            new_visible_boards_and_cards.insert(*board_id, card_ids.clone());
        }
        app.visible_boards_and_cards = new_visible_boards_and_cards;
        app.state.current_board_id = Some(previous_board.id);
        if previous_board_card_ids.is_empty() {
            app.state.current_card_id = None;
        } else {
            app.state.current_card_id = Some(previous_board_card_ids[0]);
        }
    } else {
        let previous_board_id = *current_visible_boards
            .iter()
            .nth(current_board_index - 1)
            .unwrap()
            .0;
        app.state.current_board_id = Some(previous_board_id);
        let current_visible_cards = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == previous_board_id)
            .unwrap()
            .1
            .clone();
        if current_visible_cards.is_empty() {
            app.state.current_card_id = None;
        } else {
            app.state.current_card_id = Some(current_visible_cards[0]);
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
        let current_board = boards.get_board_with_id(current_board_id);
        if current_board.is_none() {
            debug!("Cannot go up: current board not found");
            app.send_error_toast("Cannot go up: Something went wrong", None);
            return;
        }
        let current_board = current_board.unwrap();
        if current_board.cards.is_empty() {
            debug!("Cannot go up: current board has no cards");
            app.send_error_toast("Cannot go up: current board has no cards", None);
            return;
        }
        current_board.cards.get_first_card_id().unwrap()
    };
    let current_card_index = current_visible_boards
        .iter()
        .find(|(board_id, _)| **board_id == current_board_id)
        .unwrap()
        .1
        .iter()
        .position(|card_id| *card_id == current_card_id);
    if current_card_index.is_none() {
        debug!("Cannot go up: current card not found");
        app.send_error_toast("Cannot go up: Something went wrong", None);
        return;
    }
    let current_card_index = current_card_index.unwrap();
    if current_card_index == 0 {
        let current_card_index_in_all_cards = boards
            .get_board_with_id(current_board_id)
            .unwrap()
            .cards
            .get_card_index(current_card_id);
        if current_card_index_in_all_cards.is_none() {
            debug!("Cannot go up: current card not found");
            app.send_error_toast("Cannot go up: Something went wrong", None);
            return;
        }
        let current_card_index_in_all_cards = current_card_index_in_all_cards.unwrap();
        if current_card_index_in_all_cards == 0 {
            app.send_error_toast("Cannot go up: Already at the first card", None);
            return;
        }
        let previous_card_index = current_card_index_in_all_cards - 1;
        let previous_card_id = boards
            .get_board_with_id(current_board_id)
            .unwrap()
            .cards
            .get_card_with_index(previous_card_index)
            .unwrap()
            .id;
        let previous_cards = boards
            .get_board_with_id(current_board_id)
            .unwrap()
            .cards
            .get_cards_with_range(
                previous_card_index,
                previous_card_index + app.config.no_of_cards_to_show as usize,
            );
        let mut visible_boards_and_cards = app.visible_boards_and_cards.clone();
        visible_boards_and_cards
            .entry(current_board_id)
            .and_modify(|cards| {
                *cards = previous_cards.get_all_card_ids();
            });
        app.visible_boards_and_cards = visible_boards_and_cards;
        app.state.current_card_id = Some(previous_card_id);
    } else {
        let previous_card_id = *current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == current_board_id)
            .unwrap()
            .1
            .get(current_card_index - 1)
            .unwrap_or(&(0, 0));
        if previous_card_id == (0, 0) {
            debug!("Cannot go up: previous card not found");
            app.send_error_toast("Cannot go up: Something went wrong", None);
        } else {
            app.state.current_card_id = Some(previous_card_id);
        }
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
        let current_board = boards.get_board_with_id(current_board_id);
        if current_board.is_none() {
            debug!("Cannot go down: current board not found, trying to get the first board");
            if current_visible_boards.is_empty() {
                debug!("Cannot go down: current board not found, tried to get the first board, but failed");
                app.send_error_toast("Cannot go down: Something went wrong", None);
                return;
            } else {
                app.state.current_board_id = Some(*current_visible_boards.keys().next().unwrap());
                app.state.current_card_id =
                    Some(current_visible_boards.values().next().unwrap()[0]);
                return;
            }
        }
        let current_board = current_board.unwrap();
        if current_board.cards.is_empty() {
            debug!("Cannot go down: current board has no cards");
            app.send_error_toast("Cannot go down: Current board has no cards", None);
            return;
        }
        current_board.cards.get_first_card_id().unwrap()
    };
    let current_card_index = current_visible_boards
        .iter()
        .find(|(board_id, _)| **board_id == current_board_id)
        .unwrap()
        .1
        .iter()
        .position(|card_id| *card_id == current_card_id);
    if current_card_index.is_none() {
        debug!("Cannot go down: current card not found");
        app.send_error_toast("Cannot go down: Something went wrong", None);
        return;
    }
    let current_card_index = current_card_index.unwrap();
    let current_board = boards.get_board_with_id(current_board_id).unwrap();
    if current_card_index == current_board.cards.len() - 1 {
        app.send_error_toast("Cannot go down: Already at the last card", None);
        return;
    }
    if current_card_index == app.config.no_of_cards_to_show as usize - 1 {
        let current_card_index_in_all_cards = current_board.cards.get_card_index(current_card_id);
        if current_card_index_in_all_cards.is_none() {
            debug!("Cannot go down: current card not found");
            app.send_error_toast("Cannot go down: Something went wrong", None);
            return;
        }
        let current_card_index_in_all_cards = current_card_index_in_all_cards.unwrap();
        let next_card_index = current_card_index_in_all_cards + 1;
        if next_card_index >= current_board.cards.len() {
            app.send_error_toast("Cannot go down: Already at the last card", None);
            return;
        }
        let next_card_id = current_board
            .cards
            .get_card_with_index(next_card_index)
            .unwrap()
            .id;
        let start_index = next_card_index - 1;
        let end_index = next_card_index - 1 + app.config.no_of_cards_to_show as usize;
        let end_index = if end_index > current_board.cards.len() {
            current_board.cards.len()
        } else {
            end_index
        };
        let next_card_ids = current_board
            .cards
            .get_cards_with_range(start_index, end_index)
            .get_all_card_ids();
        let next_card_ids = if next_card_ids.len() < app.config.no_of_cards_to_show as usize {
            let mut next_card_ids = next_card_ids;
            let mut start_index = start_index;
            while next_card_ids.len() < app.config.no_of_cards_to_show as usize && start_index > 0 {
                start_index -= 1;
                next_card_ids.insert(
                    0,
                    current_board
                        .cards
                        .get_card_with_index(start_index)
                        .unwrap()
                        .id,
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
        let next_card_id = *current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == current_board_id)
            .unwrap()
            .1
            .get(current_card_index + 1)
            .unwrap_or(&(0, 0));
        if next_card_id != (0, 0) {
            app.state.current_card_id = Some(next_card_id);
        }
    }
}

pub fn prepare_config_for_new_app(
    theme: Theme,
) -> (AppConfig, Vec<&'static str>, Vec<ToastWidget>) {
    let mut toasts = vec![];
    let mut errors = vec![];
    let get_config_status = get_config(false);
    if let Err(config_error_msg) = get_config_status {
        if config_error_msg.contains("Overlapped keybindings found") {
            error!("KeyBindings overlap detected. Please check your config file and fix the keybindings. Using default keybindings for now.");
            errors.push("KeyBindings overlap detected. Please check your config file and fix the keybindings. Using default keybindings for now.");
            toasts.push(ToastWidget::new(
                config_error_msg,
                Duration::from_secs(DEFAULT_TOAST_DURATION) * 3,
                ToastType::Error,
                theme.clone(),
            ));
            toasts.push(ToastWidget::new("Please check your config file and fix the keybindings. Using default keybindings for now.".to_owned(),
                Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Warning, theme.clone()));
            let new_config = get_config(true);
            if let Err(new_config_error) = new_config {
                error!("Unable to fix keybindings. Please check your config file. Using default config for now.");
                errors.push("Unable to fix keybindings. Please check your config file. Using default config for now.");
                toasts.push(ToastWidget::new(
                    new_config_error,
                    Duration::from_secs(DEFAULT_TOAST_DURATION) * 3,
                    ToastType::Error,
                    theme.clone(),
                ));
                toasts.push(ToastWidget::new(
                    "Using default config for now.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION),
                    ToastType::Warning,
                    theme,
                ));
                (AppConfig::default(), errors, toasts)
            } else {
                let mut unwrapped_new_config = new_config.unwrap();
                unwrapped_new_config.keybindings = KeyBindings::default();
                (unwrapped_new_config, errors, toasts)
            }
        } else {
            toasts.push(ToastWidget::new(
                config_error_msg,
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Error,
                theme.clone(),
            ));
            toasts.push(ToastWidget::new(
                "Using default config for now.".to_owned(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Info,
                theme,
            ));
            (AppConfig::default(), errors, toasts)
        }
    } else {
        (get_config_status.unwrap(), errors, toasts)
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
            Focus::CardDueDate => app.state.text_buffers.card_due_date.reset(),
            Focus::EmailIDField => app.state.text_buffers.email_id.reset(),
            Focus::PasswordField => app.state.text_buffers.password.reset(),
            Focus::ConfirmPasswordField => app.state.text_buffers.confirm_password.reset(),
            Focus::ResetPasswordLinkField => app.state.text_buffers.reset_password_link.reset(),
            Focus::CommandPaletteCommand
            | Focus::CommandPaletteBoard
            | Focus::CommandPaletteCard => {
                app.state.text_buffers.command_palette.reset();
            }
            _ => {
                debug!(
                    "No user input handler found for focus: {:?} key Esc",
                    app.state.focus
                );
            }
        }
        if app.state.popup_mode.is_some() {
            match app.state.popup_mode.unwrap() {
                PopupMode::CommandPalette => {
                    app.close_popup();
                    if app.widgets.command_palette.already_in_user_input_mode {
                        app.widgets.command_palette.already_in_user_input_mode = false;
                        if app.widgets.command_palette.last_focus.is_some() {
                            app.state
                                .set_focus(app.widgets.command_palette.last_focus.unwrap());
                        }
                        return AppReturn::Continue;
                    }
                }
                PopupMode::ConfirmDiscardCardChanges => {
                    if app.state.card_being_edited.is_some() {
                        warn!(
                            "Discarding changes to card '{}'",
                            app.state.card_being_edited.as_ref().unwrap().1.name
                        );
                        app.send_warning_toast(
                            &format!(
                                "Discarding changes to card '{}'",
                                app.state.card_being_edited.as_ref().unwrap().1.name
                            ),
                            None,
                        );
                    }
                    app.close_popup();
                    app.state.card_being_edited = None;
                    // Move this behaviour to clove popup function
                    app.state.app_status = AppStatus::Initialized;
                }
                PopupMode::ViewCard => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                    }
                }
                PopupMode::CardPrioritySelector => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                    } else {
                        app.close_popup();
                    }
                }
                PopupMode::CardStatusSelector => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                    } else {
                        app.close_popup();
                    }
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
        app.set_popup_mode(PopupMode::CommandPalette);
    } else if app.config.keybindings.stop_user_input.contains(&key)
        && !app
            .state
            .popup_mode
            .is_some_and(|popup_mode| popup_mode == PopupMode::CommandPalette)
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
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            let stop_input_mode_keys = &app.config.keybindings.stop_user_input;
            match key {
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
                            handle_command_palette_card_selection(app);
                            app.close_popup();
                            app.state.text_buffers.command_palette.reset();
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                        Focus::CommandPaletteBoard => {
                            handle_command_palette_board_selection(app);
                            app.close_popup();
                            app.state.text_buffers.command_palette.reset();
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
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else {
                    app.state.text_buffers.card_due_date.input(key);
                }
            }
            Focus::CardPriority => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else if key == Key::Enter {
                    app.set_popup_mode(PopupMode::CardPrioritySelector);
                }
            }
            Focus::CardStatus => {
                if app.config.keybindings.next_focus.contains(&key) {
                    handle_next_focus(app);
                } else if app.config.keybindings.prv_focus.contains(&key) {
                    handle_prv_focus(app);
                } else if key == Key::Enter {
                    app.set_popup_mode(PopupMode::CardStatusSelector);
                }
            }
            Focus::CardTags => {
                if app.state.card_being_edited.is_none() {
                    // cannot add tag or comment to a new card
                    return AppReturn::Continue;
                }
                let current_card = &mut app.state.card_being_edited.as_mut().unwrap().1;
                let current_selected = app
                    .state
                    .app_list_states
                    .card_view_tag_list
                    .selected()
                    .unwrap_or(0);
                match key {
                    Key::ShiftRight => {
                        let max = current_card.tags.len();
                        if current_selected < max - 1 {
                            app.state
                                .app_list_states
                                .card_view_tag_list
                                .select(Some(current_selected + 1));
                        }
                    }
                    Key::ShiftLeft => {
                        app.state
                            .app_list_states
                            .card_view_tag_list
                            .select(Some(current_selected.saturating_sub(1)));
                    }
                    Key::Enter => {
                        if let Some(insert_index) =
                            app.state.app_list_states.card_view_tag_list.selected()
                        {
                            current_card.tags.insert(insert_index + 1, "".to_owned());
                            app.state
                                .text_buffers
                                .prepare_tags_and_comments_for_card(current_card);
                            app.state
                                .app_list_states
                                .card_view_tag_list
                                .select(Some(insert_index + 1));
                        } else {
                            current_card.tags.push("".to_owned());
                            app.state
                                .text_buffers
                                .prepare_tags_and_comments_for_card(current_card);
                            app.state
                                .app_list_states
                                .card_view_tag_list
                                .select(Some(current_card.tags.len().saturating_sub(1)));
                        }
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
                    _ if app.config.keybindings.next_focus.contains(&key) => handle_next_focus(app),
                    _ if app.config.keybindings.prv_focus.contains(&key) => handle_prv_focus(app),
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
            }
            Focus::CardComments => {
                if app.state.card_being_edited.is_none() {
                    // cannot add tag or comment to a new card
                    return AppReturn::Continue;
                }
                let current_card = &mut app.state.card_being_edited.as_mut().unwrap().1;
                let current_selected = app
                    .state
                    .app_list_states
                    .card_view_comment_list
                    .selected()
                    .unwrap_or(0);
                match key {
                    Key::ShiftRight => {
                        let max = current_card.comments.len();
                        if current_selected < max - 1 {
                            app.state
                                .app_list_states
                                .card_view_comment_list
                                .select(Some(current_selected + 1));
                        }
                    }
                    Key::ShiftLeft => {
                        app.state
                            .app_list_states
                            .card_view_comment_list
                            .select(Some(current_selected.saturating_sub(1)));
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
                    _ if app.config.keybindings.next_focus.contains(&key) => handle_next_focus(app),
                    _ if app.config.keybindings.prv_focus.contains(&key) => handle_prv_focus(app),
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
                if app.state.ui_mode == UiMode::CreateTheme {
                    // Special handling for Create theme as only it uses the general config popup other than config changes
                    app.state.text_buffers.general_config.input(key);
                } else {
                    match key {
                        Key::Right | Key::Tab => {
                            if app.state.path_check_state.potential_completion.is_some() {
                                app.state.text_buffers.general_config.insert_str(
                                    app.state
                                        .path_check_state
                                        .potential_completion
                                        .as_ref()
                                        .unwrap(),
                                );
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
                        if app.state.popup_mode.is_some() {
                            match app.state.popup_mode.unwrap() {
                                PopupMode::ViewCard if app.state.card_being_edited.is_some() => {
                                    return handle_edit_card_submit(app);
                                }
                                _ => {
                                    debug!("Dont know what to do with Submit button in user input mode for popup mode: {:?}", app.state.popup_mode);
                                }
                            }
                            app.state.app_status = AppStatus::Initialized;
                            return AppReturn::Continue;
                        }
                        match app.state.ui_mode {
                            UiMode::NewCard => {
                                handle_new_card_action(app);
                            }
                            UiMode::NewBoard => {
                                handle_new_board_action(app);
                            }
                            _ => {
                                debug!("Dont know what to do with Submit button in user input mode for ui mode: {:?}", app.state.ui_mode);
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
            // Focus::TextInput => {

            // }
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
    match key {
        Key::Esc => {
            app.state.app_status = AppStatus::Initialized;
            app.state.edited_keybinding = None;
            info!("Exiting user Keybinding input mode");
        }
        _ => {
            if app.config.keybindings.stop_user_input.contains(&key) {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user Keybinding input mode");
                return AppReturn::Continue;
            }
            if app.state.edited_keybinding.is_some() {
                let keybinding = app.state.edited_keybinding.as_mut().unwrap();
                keybinding.push(key);
            } else {
                app.state.edited_keybinding = Some(vec![key]);
            }
        }
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
                app.widgets.toasts = vec![];
                app.set_ui_mode(app.config.default_ui_mode);
                app.send_info_toast("UI reset, all toasts cleared", None);
                app.close_popup();
                refresh_visible_boards_and_cards(app);
                AppReturn::Continue
            }
            Action::OpenConfigMenu => {
                match app.state.ui_mode {
                    UiMode::ConfigMenu => {
                        handle_go_to_previous_ui_mode(app).await;
                    }
                    _ => {
                        app.set_ui_mode(UiMode::ConfigMenu);
                    }
                }
                if app.state.popup_mode.is_some() {
                    app.close_popup();
                }
                AppReturn::Continue
            }
            Action::Up => {
                reset_mouse(app);
                if app.state.popup_mode.is_some() {
                    let popup_mode = app.state.popup_mode.as_ref().unwrap();
                    match popup_mode {
                        PopupMode::ChangeUIMode => app.select_default_view_prv(),
                        PopupMode::CardStatusSelector => app.select_card_status_prv(),
                        PopupMode::SelectDefaultView => app.select_default_view_prv(),
                        PopupMode::ChangeTheme => app.select_change_theme_prv(),
                        PopupMode::EditThemeStyle => {
                            if app.state.focus == Focus::StyleEditorFG {
                                app.select_edit_style_fg_prv();
                            } else if app.state.focus == Focus::StyleEditorBG {
                                app.select_edit_style_bg_prv();
                            } else if app.state.focus == Focus::StyleEditorModifier {
                                app.select_edit_style_modifier_prv();
                            }
                        }
                        PopupMode::ChangeDateFormatPopup => app.change_date_format_popup_prv(),
                        PopupMode::FilterByTag => app.filter_by_tag_popup_prv(),
                        PopupMode::ViewCard => {
                            if app.state.focus == Focus::CardDescription {
                                app.state
                                    .text_buffers
                                    .card_description
                                    .scroll(TextBoxScroll::Delta { rows: -1, cols: 0 });
                            }
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                match app.state.ui_mode {
                    UiMode::ConfigMenu => {
                        if app.state.focus == Focus::ConfigTable {
                            app.config_prv();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .first()
                                .unwrap_or(&Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prv_focus
                                .first()
                                .unwrap_or(&Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::MainMenu => {
                        if app.state.focus == Focus::MainMenu {
                            app.main_menu_prv();
                        } else if app.state.focus == Focus::Help {
                            app.help_prv();
                        } else if app.state.focus == Focus::Log {
                            app.log_prv();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .first()
                                .unwrap_or(&Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prv_focus
                                .first()
                                .unwrap_or(&Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::LoadLocalSave => {
                        app.load_save_prv(false);
                        app.dispatch(IoEvent::LoadLocalPreview).await;
                    }
                    UiMode::LoadCloudSave => {
                        app.load_save_prv(true);
                        app.dispatch(IoEvent::LoadCloudPreview).await;
                    }
                    UiMode::EditKeybindings => {
                        app.edit_keybindings_prv();
                    }
                    UiMode::CreateTheme => {
                        if app.state.focus == Focus::ThemeEditor {
                            app.select_create_theme_prv();
                        } else if app.state.focus == Focus::SubmitButton {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .first()
                                .unwrap_or(&Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prv_focus
                                .first()
                                .unwrap_or(&Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the theme editor with {} or {}, to select a style to edit",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::NewBoard => {
                        if app.state.focus == Focus::NewBoardDescription {
                            app.state
                                .text_buffers
                                .board_description
                                .scroll(TextBoxScroll::Delta { rows: -1, cols: 0 })
                        }
                    }
                    UiMode::NewCard => {
                        if app.state.focus == Focus::CardDescription {
                            app.state
                                .text_buffers
                                .card_description
                                .scroll(TextBoxScroll::Delta { rows: -1, cols: 0 })
                        }
                    }
                    _ => {
                        if app.state.focus == Focus::Body
                            && UiMode::view_modes().contains(&app.state.ui_mode)
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
                if app.state.popup_mode.is_some() {
                    let popup_mode = app.state.popup_mode.as_ref().unwrap();
                    match popup_mode {
                        PopupMode::ChangeUIMode => app.select_default_view_next(),
                        PopupMode::CardStatusSelector => app.select_card_status_next(),
                        PopupMode::SelectDefaultView => app.select_default_view_next(),
                        PopupMode::ChangeTheme => app.select_change_theme_next(),
                        PopupMode::EditThemeStyle => {
                            if app.state.focus == Focus::StyleEditorFG {
                                app.select_edit_style_fg_next();
                            } else if app.state.focus == Focus::StyleEditorBG {
                                app.select_edit_style_bg_next();
                            } else if app.state.focus == Focus::StyleEditorModifier {
                                app.select_edit_style_modifier_next();
                            }
                        }
                        PopupMode::ChangeDateFormatPopup => app.change_date_format_popup_next(),
                        PopupMode::FilterByTag => app.filter_by_tag_popup_next(),
                        PopupMode::ViewCard => {
                            if app.state.focus == Focus::CardDescription {
                                app.state
                                    .text_buffers
                                    .card_description
                                    .scroll(TextBoxScroll::Delta { rows: 1, cols: 0 })
                            }
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                match app.state.ui_mode {
                    UiMode::ConfigMenu => {
                        if app.state.focus == Focus::ConfigTable {
                            app.config_next();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .first()
                                .unwrap_or(&Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prv_focus
                                .first()
                                .unwrap_or(&Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::MainMenu => {
                        if app.state.focus == Focus::MainMenu {
                            app.main_menu_next();
                        } else if app.state.focus == Focus::Help {
                            app.help_next();
                        } else if app.state.focus == Focus::Log {
                            app.log_next();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .first()
                                .unwrap_or(&Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prv_focus
                                .first()
                                .unwrap_or(&Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::LoadLocalSave => {
                        app.load_save_next(false);
                        app.dispatch(IoEvent::LoadLocalPreview).await;
                    }
                    UiMode::LoadCloudSave => {
                        app.load_save_next(true);
                        app.dispatch(IoEvent::LoadCloudPreview).await;
                    }
                    UiMode::EditKeybindings => {
                        app.edit_keybindings_next();
                    }
                    UiMode::CreateTheme => {
                        if app.state.focus == Focus::ThemeEditor {
                            app.select_create_theme_next();
                        } else if app.state.focus == Focus::SubmitButton {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .first()
                                .unwrap_or(&Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prv_focus
                                .first()
                                .unwrap_or(&Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the theme editor with {} or {}, to select a style to edit",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::NewBoard => {
                        if app.state.focus == Focus::NewBoardDescription {
                            app.state
                                .text_buffers
                                .board_description
                                .scroll(TextBoxScroll::Delta { rows: 1, cols: 0 })
                        }
                    }
                    UiMode::NewCard => {
                        if app.state.focus == Focus::CardDescription {
                            app.state
                                .text_buffers
                                .card_description
                                .scroll(TextBoxScroll::Delta { rows: 1, cols: 0 })
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
                if app.state.focus == Focus::Body
                    && UiMode::view_modes().contains(&app.state.ui_mode)
                    && app.state.popup_mode.is_none()
                {
                    go_right(app);
                } else if app.state.popup_mode.is_some()
                    && (app.state.popup_mode.unwrap() == PopupMode::ConfirmDiscardCardChanges
                        || app.state.popup_mode.unwrap() == PopupMode::SaveThemePrompt)
                {
                    if app.state.focus == Focus::SubmitButton {
                        app.state.set_focus(Focus::ExtraFocus);
                    } else {
                        app.state.set_focus(Focus::SubmitButton);
                    }
                }
                AppReturn::Continue
            }
            Action::Left => {
                reset_mouse(app);
                if app.state.focus == Focus::Body
                    && UiMode::view_modes().contains(&app.state.ui_mode)
                    && app.state.popup_mode.is_none()
                {
                    go_left(app);
                } else if app.state.popup_mode.is_some()
                    && (app.state.popup_mode.unwrap() == PopupMode::ConfirmDiscardCardChanges
                        || app.state.popup_mode.unwrap() == PopupMode::SaveThemePrompt)
                {
                    if app.state.focus == Focus::SubmitButton {
                        app.state.set_focus(Focus::ExtraFocus);
                    } else {
                        app.state.set_focus(Focus::SubmitButton);
                    }
                }
                AppReturn::Continue
            }
            Action::TakeUserInput => {
                match app.state.ui_mode {
                    UiMode::NewBoard | UiMode::NewCard => {
                        app.state.app_status = AppStatus::UserInput;
                        info!("Taking user input");
                    }
                    _ => {
                        if app.state.popup_mode.is_some() {
                            match app.state.popup_mode.unwrap() {
                                PopupMode::EditGeneralConfig
                                | PopupMode::CustomRGBPromptFG
                                | PopupMode::CustomRGBPromptBG => {
                                    app.state.app_status = AppStatus::UserInput;
                                    info!("Taking user input");
                                }
                                PopupMode::EditSpecificKeyBinding => {
                                    app.state.app_status = AppStatus::KeyBindMode;
                                    info!("Taking user Keybinding input");
                                }
                                PopupMode::ViewCard => {
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
            Action::GoToPreviousUIModeorCancel => handle_go_to_previous_ui_mode(app).await,
            Action::Accept => {
                if app.state.popup_mode.is_some() {
                    let popup_mode = app.state.popup_mode.as_ref().unwrap();
                    match popup_mode {
                        PopupMode::ChangeUIMode => handle_change_ui_mode(app),
                        PopupMode::CardStatusSelector => {
                            return handle_change_card_status(app, None);
                        }
                        PopupMode::EditGeneralConfig => {
                            if app.state.ui_mode == UiMode::CreateTheme {
                                handle_create_theme_action(app);
                            } else {
                                handle_edit_general_config(app);
                            }
                        }
                        PopupMode::EditSpecificKeyBinding => handle_edit_specific_keybinding(app),
                        PopupMode::SelectDefaultView => handle_default_view_selection(app),
                        PopupMode::ChangeDateFormatPopup => handle_change_date_format(app),
                        PopupMode::ChangeTheme => {
                            return handle_change_theme(app, app.state.default_theme_mode)
                        }
                        PopupMode::EditThemeStyle => return handle_create_theme_action(app),
                        PopupMode::SaveThemePrompt => handle_save_theme_prompt(app),
                        PopupMode::CustomRGBPromptFG => return handle_custom_rgb_prompt(app, true),
                        PopupMode::CustomRGBPromptBG => {
                            return handle_custom_rgb_prompt(app, false)
                        }
                        PopupMode::ViewCard => match app.state.focus {
                            Focus::CardPriority => {
                                if app.state.card_being_edited.is_none() {
                                    handle_edit_new_card(app);
                                }
                                app.set_popup_mode(PopupMode::CardPrioritySelector);
                                return AppReturn::Continue;
                            }
                            Focus::CardStatus => {
                                if app.state.card_being_edited.is_none() {
                                    handle_edit_new_card(app);
                                }
                                app.set_popup_mode(PopupMode::CardStatusSelector);
                                return AppReturn::Continue;
                            }
                            Focus::CardName
                            | Focus::CardDescription
                            | Focus::CardDueDate
                            | Focus::CardTags
                            | Focus::CardComments => return handle_edit_new_card(app),
                            Focus::SubmitButton => {
                                return handle_edit_card_submit(app);
                            }
                            _ => {}
                        },
                        PopupMode::CommandPalette => {
                            // not required to handle here as the command palette is handled in the user input mode
                        }
                        PopupMode::ConfirmDiscardCardChanges => match app.state.focus {
                            Focus::SubmitButton => {
                                handle_edit_card_submit(app);
                                app.close_popup();
                            }
                            Focus::ExtraFocus => {
                                if app.state.card_being_edited.is_some() {
                                    warn!(
                                        "Discarding changes to card '{}'",
                                        app.state.card_being_edited.as_ref().unwrap().1.name
                                    );
                                    app.send_warning_toast(
                                        &format!(
                                            "Discarding changes to card '{}'",
                                            app.state.card_being_edited.as_ref().unwrap().1.name
                                        ),
                                        None,
                                    );
                                }
                                app.close_popup();
                                app.state.card_being_edited = None;
                            }
                            _ => {}
                        },
                        PopupMode::CardPrioritySelector => {
                            return handle_change_card_priority(app, None);
                        }
                        PopupMode::FilterByTag => {
                            handle_filter_by_tag(app);
                            return AppReturn::Continue;
                        }
                    }
                    app.close_popup();
                    return AppReturn::Continue;
                }
                match app.state.ui_mode {
                    UiMode::ConfigMenu => handle_config_menu_action(app),
                    UiMode::MainMenu => match app.state.focus {
                        Focus::MainMenu => handle_main_menu_action(app).await,
                        Focus::Help => {
                            app.set_ui_mode(UiMode::HelpMenu);
                            AppReturn::Continue
                        }
                        Focus::Log => {
                            app.set_ui_mode(UiMode::LogsOnly);
                            AppReturn::Continue
                        }
                        _ => AppReturn::Continue,
                    },
                    UiMode::NewBoard => {
                        handle_new_board_action(app);
                        AppReturn::Continue
                    }
                    UiMode::NewCard => {
                        handle_new_card_action(app);
                        AppReturn::Continue
                    }
                    UiMode::LoadLocalSave => {
                        app.dispatch(IoEvent::LoadSaveLocal).await;
                        AppReturn::Continue
                    }
                    UiMode::EditKeybindings => {
                        handle_edit_keybindings_action(app);
                        AppReturn::Continue
                    }
                    UiMode::CreateTheme => {
                        handle_create_theme_action(app);
                        AppReturn::Continue
                    }
                    UiMode::Login => {
                        handle_login_action(app).await;
                        AppReturn::Continue
                    }
                    UiMode::SignUp => {
                        handle_signup_action(app).await;
                        AppReturn::Continue
                    }
                    UiMode::ResetPassword => {
                        handle_reset_password_action(app).await;
                        AppReturn::Continue
                    }
                    UiMode::LoadCloudSave => {
                        app.dispatch(IoEvent::LoadSaveCloud).await;
                        AppReturn::Continue
                    }
                    _ => {
                        match app.state.focus {
                            Focus::Help => {
                                app.set_ui_mode(UiMode::HelpMenu);
                            }
                            Focus::Log => {
                                app.set_ui_mode(UiMode::LogsOnly);
                            }
                            Focus::Title => {
                                app.set_ui_mode(UiMode::MainMenu);
                            }
                            _ => {}
                        }
                        if UiMode::view_modes().contains(&app.state.ui_mode)
                            && app.state.focus == Focus::Body
                            && app.state.current_board_id.is_some()
                            && app.state.current_card_id.is_some()
                        {
                            app.set_popup_mode(PopupMode::ViewCard);
                        }
                        AppReturn::Continue
                    }
                }
            }
            Action::HideUiElement => {
                let current_focus = app.state.focus;
                let current_ui_mode = app.state.ui_mode;
                if current_ui_mode == UiMode::Zen {
                    app.set_ui_mode(UiMode::MainMenu);
                    if app.state.app_list_states.main_menu.selected().is_none() {
                        app.main_menu_next();
                    }
                } else if current_ui_mode == UiMode::TitleBody {
                    if current_focus == Focus::Title {
                        app.set_ui_mode(UiMode::Zen);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::BodyHelp {
                    if current_focus == Focus::Help {
                        app.set_ui_mode(UiMode::Zen);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::BodyLog {
                    if current_focus == Focus::Log {
                        app.set_ui_mode(UiMode::Zen);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::TitleBodyHelp {
                    if current_focus == Focus::Title {
                        app.set_ui_mode(UiMode::BodyHelp);
                    } else if current_focus == Focus::Help {
                        app.set_ui_mode(UiMode::TitleBody);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::TitleBodyLog {
                    if current_focus == Focus::Title {
                        app.set_ui_mode(UiMode::BodyLog);
                    } else if current_focus == Focus::Log {
                        app.set_ui_mode(UiMode::TitleBody);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::TitleBodyHelpLog {
                    if current_focus == Focus::Title {
                        app.set_ui_mode(UiMode::BodyHelpLog);
                    } else if current_focus == Focus::Help {
                        app.set_ui_mode(UiMode::TitleBodyLog);
                    } else if current_focus == Focus::Log {
                        app.set_ui_mode(UiMode::TitleBodyHelp);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::BodyHelpLog {
                    if current_focus == Focus::Help {
                        app.set_ui_mode(UiMode::BodyLog);
                    } else if current_focus == Focus::Log {
                        app.set_ui_mode(UiMode::BodyHelp);
                    } else {
                        app.set_ui_mode(UiMode::MainMenu);
                        if app.state.app_list_states.main_menu.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::SaveState => {
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    app.dispatch(IoEvent::SaveLocalData).await;
                }
                AppReturn::Continue
            }
            Action::NewBoard => {
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    reset_new_board_form(app);
                    app.set_ui_mode(UiMode::NewBoard);
                    app.state.prev_focus = Some(app.state.focus);
                }
                AppReturn::Continue
            }
            Action::NewCard => {
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    if app.state.current_board_id.is_none() {
                        warn!("No board available to add card to");
                        app.send_warning_toast("No board available to add card to", None);
                        return AppReturn::Continue;
                    }
                    reset_new_card_form(app);
                    app.set_ui_mode(UiMode::NewCard);
                    app.state.prev_focus = Some(app.state.focus);
                }
                AppReturn::Continue
            }
            Action::Delete => match app.state.ui_mode {
                UiMode::LoadLocalSave => {
                    app.dispatch(IoEvent::DeleteLocalSave).await;
                    tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                    app.dispatch(IoEvent::LoadLocalPreview).await;
                    AppReturn::Continue
                }
                UiMode::LoadCloudSave => {
                    app.dispatch(IoEvent::DeleteCloudSave).await;
                    tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                    app.dispatch(IoEvent::GetCloudData).await;
                    tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                    app.dispatch(IoEvent::LoadCloudPreview).await;
                    AppReturn::Continue
                }
                _ => {
                    if !UiMode::view_modes().contains(&app.state.ui_mode) {
                        return AppReturn::Continue;
                    }
                    match app.state.focus {
                        Focus::Body => {
                            if let Some(current_board_id) = app.state.current_board_id {
                                if let Some(current_card_id) = app.state.current_card_id {
                                    let current_board =
                                        app.boards.get_mut_board_with_id(current_board_id);
                                    if current_board.is_none() {
                                        debug!("No board available to delete card from");
                                        return AppReturn::Continue;
                                    }
                                    let current_board = current_board.unwrap();
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
                                        current_board.cards.remove_card_with_id(current_card_id);
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
                                            ActionHistory::DeleteCard(card, current_board.id),
                                        );
                                        app.send_warning_toast(
                                            &format!("Deleted card {}", card_name),
                                            None,
                                        );
                                        if let Some(visible_cards) =
                                            app.visible_boards_and_cards.get_mut(&current_board_id)
                                        {
                                            if let Some(card_index) = visible_cards
                                                .iter()
                                                .position(|card_id| *card_id == current_card_id)
                                            {
                                                visible_cards.remove(card_index);
                                            }
                                        }
                                        refresh_visible_boards_and_cards(app);
                                    }
                                } else if let Some(current_board_id) = app.state.current_board_id {
                                    let board =
                                        app.boards.get_board_with_id(current_board_id).cloned();
                                    if let Some(board) = board {
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
                            }
                            AppReturn::Continue
                        }
                        _ => AppReturn::Continue,
                    }
                }
            },
            Action::DeleteBoard => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        if let Some(current_board_id) = app.state.current_board_id {
                            let board = app.boards.get_board_with_id(current_board_id).cloned();
                            if let Some(board) = board {
                                let board_index =
                                    app.boards.get_board_index(current_board_id).unwrap();
                                let board_name = board.name.clone();
                                app.boards.remove_board_with_id(current_board_id);
                                if board_index > 0 {
                                    app.state.current_board_id = Some(
                                        app.boards
                                            .get_board_with_index(board_index - 1)
                                            .unwrap()
                                            .id,
                                    );
                                } else if board_index < app.boards.len() {
                                    app.state.current_board_id = Some(
                                        app.boards.get_board_with_index(board_index).unwrap().id,
                                    );
                                } else {
                                    app.state.current_board_id = None;
                                }
                                app.visible_boards_and_cards.remove(&current_board_id);
                                warn!("Deleted board: {}", board_name);
                                app.action_history_manager
                                    .new_action(ActionHistory::DeleteBoard(board));
                                app.send_warning_toast(
                                    &format!("Deleted board: {}", board_name),
                                    None,
                                );
                            }
                        }
                        AppReturn::Continue
                    }
                    _ => AppReturn::Continue,
                }
            }
            Action::ChangeCardStatusToCompleted => {
                if !UiMode::view_modes().contains(&app.state.ui_mode)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_status(app, Some(CardStatus::Complete))
            }
            Action::ChangeCardStatusToActive => {
                if !UiMode::view_modes().contains(&app.state.ui_mode)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_status(app, Some(CardStatus::Active))
            }
            Action::ChangeCardStatusToStale => {
                if !UiMode::view_modes().contains(&app.state.ui_mode)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_status(app, Some(CardStatus::Stale))
            }
            Action::ChangeCardPriorityToHigh => {
                if !UiMode::view_modes().contains(&app.state.ui_mode)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_priority(app, Some(CardPriority::High))
            }
            Action::ChangeCardPriorityToMedium => {
                if !UiMode::view_modes().contains(&app.state.ui_mode)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_priority(app, Some(CardPriority::Medium))
            }
            Action::ChangeCardPriorityToLow => {
                if !UiMode::view_modes().contains(&app.state.ui_mode)
                    || app.state.focus != Focus::Body
                {
                    return AppReturn::Continue;
                };
                handle_change_card_priority(app, Some(CardPriority::Low))
            }
            Action::GoToMainMenu => {
                match app.state.ui_mode {
                    UiMode::NewBoard => {
                        reset_new_board_form(app);
                    }
                    UiMode::NewCard => {
                        reset_new_card_form(app);
                    }
                    UiMode::Login => {
                        reset_login_form(app);
                    }
                    UiMode::SignUp => {
                        reset_signup_form(app);
                    }
                    UiMode::ResetPassword => {
                        reset_reset_password_form(app);
                    }
                    _ => {}
                }
                app.state.current_board_id = None;
                app.state.current_card_id = None;
                app.set_ui_mode(UiMode::MainMenu);
                if app.state.app_list_states.main_menu.selected().is_none() {
                    app.state.app_list_states.main_menu.select(Some(0));
                }
                AppReturn::Continue
            }
            Action::MoveCardUp => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
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
                        let current_board = boards.get_mut_board_with_id(current_board_id);
                        if current_board.is_none() {
                            debug!("Cannot move card up without a current board index");
                            return AppReturn::Continue;
                        }
                        let current_board = current_board.unwrap();
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
                        let current_card_index_in_visible = current_card_index_in_visible.unwrap();
                        if current_card_index_in_visible == 0 {
                            let card_above_id = current_board
                                .cards
                                .get_card_with_index(current_card_index_in_all - 1)
                                .unwrap()
                                .id;
                            let mut visible_cards: Vec<(u64, u64)> = vec![];
                            visible_cards.push(current_card_id);
                            visible_cards.push(card_above_id);

                            for card in app.visible_boards_and_cards[&current_board_id].iter() {
                                if *card != current_card_id
                                    && visible_cards.len() < app.config.no_of_cards_to_show as usize
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
                        app.action_history_manager
                            .new_action(ActionHistory::MoveCardWithinBoard(
                                current_board_id,
                                current_card_index_in_all,
                                current_card_index_in_all - 1,
                            ));
                    }
                }
                AppReturn::Continue
            }
            Action::MoveCardDown => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
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
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
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
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
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
                if app.state.popup_mode.is_none() {
                    app.set_popup_mode(PopupMode::CommandPalette);
                } else {
                    match app.state.popup_mode.unwrap() {
                        PopupMode::CommandPalette => {
                            app.close_popup();
                            app.state.text_buffers.command_palette.reset();
                            app.state.app_status = AppStatus::Initialized;
                        }
                        PopupMode::ViewCard => {
                            if app.state.card_being_edited.is_some() {
                                app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                                app.state.app_status = AppStatus::Initialized;
                            } else {
                                app.set_popup_mode(PopupMode::CommandPalette);
                            }
                        }
                        PopupMode::ConfirmDiscardCardChanges => {
                            if app.state.card_being_edited.is_some() {
                                warn!(
                                    "Discarding changes to card '{}'",
                                    app.state.card_being_edited.as_ref().unwrap().1.name
                                );
                                app.send_warning_toast(
                                    &format!(
                                        "Discarding changes to card '{}'",
                                        app.state.card_being_edited.as_ref().unwrap().1.name
                                    ),
                                    None,
                                );
                            }
                            app.close_popup();
                            app.state.card_being_edited = None;
                            app.set_popup_mode(PopupMode::CommandPalette);
                        }
                        _ => {
                            app.set_popup_mode(PopupMode::CommandPalette);
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::Undo => {
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    app.undo();
                }
                AppReturn::Continue
            }
            Action::Redo => {
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    app.redo();
                }
                AppReturn::Continue
            }
            Action::ClearAllToasts => {
                app.widgets.toasts.clear();
                info!("Cleared toast messages");
                AppReturn::Continue
            }
        }
    } else {
        // Warn user that they are not in user input mode
        if app.state.card_being_edited.is_some()
            || app.state.ui_mode == UiMode::NewCard
            || app.state.ui_mode == UiMode::NewBoard
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
            let current_ui_mode = app.state.ui_mode;
            let is_invalid_state = !UiMode::view_modes().contains(&current_ui_mode)
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
        return handle_go_to_previous_ui_mode(app).await;
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
        if app.state.popup_mode.is_none() {
            app.set_popup_mode(PopupMode::CommandPalette);
        } else {
            match app.state.popup_mode.unwrap() {
                PopupMode::CommandPalette => {
                    app.close_popup();
                    app.state.text_buffers.command_palette.reset();
                    app.state.app_status = AppStatus::Initialized;
                }
                PopupMode::ViewCard => {
                    if app.state.card_being_edited.is_some() {
                        app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                    } else {
                        app.set_popup_mode(PopupMode::CommandPalette);
                    }
                }
                PopupMode::ConfirmDiscardCardChanges => {
                    if app.state.card_being_edited.is_some() {
                        warn!(
                            "Discarding changes to card '{}'",
                            app.state.card_being_edited.as_ref().unwrap().1.name
                        );
                        app.send_warning_toast(
                            &format!(
                                "Discarding changes to card '{}'",
                                app.state.card_being_edited.as_ref().unwrap().1.name
                            ),
                            None,
                        );
                    }
                    app.close_popup();
                    app.state.card_being_edited = None;
                    app.set_popup_mode(PopupMode::CommandPalette);
                }
                _ => {
                    app.set_popup_mode(PopupMode::CommandPalette);
                }
            }
        }
        return AppReturn::Continue;
    }

    if app.state.popup_mode.is_some() {
        let popup_mode = app.state.popup_mode.unwrap();
        match popup_mode {
            PopupMode::CommandPalette => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    match app.state.mouse_focus.unwrap() {
                        Focus::CommandPaletteCommand => {
                            return CommandPaletteWidget::handle_command(app).await;
                        }
                        Focus::CommandPaletteCard => {
                            handle_command_palette_card_selection(app);
                            app.close_popup();
                            app.state.text_buffers.command_palette.reset();
                            app.state.app_status = AppStatus::Initialized;
                        }
                        Focus::CommandPaletteBoard => {
                            handle_command_palette_board_selection(app);
                            app.close_popup();
                            app.state.text_buffers.command_palette.reset();
                            app.state.app_status = AppStatus::Initialized;
                        }
                        Focus::CloseButton => {
                            app.close_popup();
                            app.state.text_buffers.command_palette.reset();
                            app.state.app_status = AppStatus::Initialized;
                        }
                        _ => {}
                    }
                } else if mouse_scroll_up && app.state.mouse_focus.is_some() {
                    match app.state.mouse_focus.unwrap() {
                        Focus::CommandPaletteCommand => {
                            app.command_palette_command_search_prv();
                        }
                        Focus::CommandPaletteCard => app.command_palette_card_search_prv(),
                        Focus::CommandPaletteBoard => app.command_palette_board_search_prv(),
                        _ => {}
                    }
                } else if mouse_scroll_down && app.state.mouse_focus.is_some() {
                    match app.state.mouse_focus.unwrap() {
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
            PopupMode::SelectDefaultView => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SelectDefaultView) {
                        handle_default_view_selection(app);
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                    }
                }
            }
            PopupMode::ChangeUIMode => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ChangeUiModePopup) {
                        handle_change_ui_mode(app);
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                    }
                }
            }
            PopupMode::CardStatusSelector => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ChangeCardStatusPopup) {
                        return handle_change_card_status(app, None);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                    }
                }
            }
            PopupMode::EditGeneralConfig => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::EditGeneralConfigPopup) {
                        app.state.app_status = AppStatus::UserInput;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        if app.state.ui_mode == UiMode::CreateTheme {
                            handle_create_theme_action(app);
                        } else {
                            handle_edit_general_config(app);
                        }
                        app.state.app_status = AppStatus::Initialized;
                        app.close_popup();
                    }
                }
            }
            PopupMode::EditSpecificKeyBinding => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::EditSpecificKeyBindingPopup) {
                        app.state.app_status = AppStatus::KeyBindMode;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.close_popup();
                        handle_edit_specific_keybinding(app);
                    }
                }
            }
            PopupMode::ChangeDateFormatPopup => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ChangeDateFormatPopup) {
                        handle_change_date_format(app);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                    }
                }
            }
            PopupMode::ChangeTheme => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ThemeSelector) {
                        handle_change_theme(app, app.state.default_theme_mode);
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        let config_theme = {
                            let all_themes = Theme::all_default_themes();
                            let default_theme = app.config.default_theme.clone();
                            all_themes.iter().find(|t| t.name == default_theme).cloned()
                        };
                        if config_theme.is_some() {
                            app.current_theme = config_theme.unwrap();
                        }
                        app.close_popup();
                    }
                }
            }
            PopupMode::EditThemeStyle => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.close_popup();
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton)
                        || app.state.mouse_focus == Some(Focus::StyleEditorFG)
                        || app.state.mouse_focus == Some(Focus::StyleEditorBG)
                        || app.state.mouse_focus == Some(Focus::StyleEditorModifier)
                        || app.state.mouse_focus == Some(Focus::ExtraFocus)
                    {
                        handle_create_theme_action(app);
                    }
                } else if mouse_scroll_up {
                    handle_theme_maker_scroll_up(app);
                } else if mouse_scroll_down {
                    handle_theme_maker_scroll_down(app);
                }
            }
            PopupMode::SaveThemePrompt => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SubmitButton)
                        || app.state.mouse_focus == Some(Focus::ExtraFocus)
                    {
                        handle_save_theme_prompt(app);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                    }
                }
            }
            PopupMode::CustomRGBPromptFG => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_custom_rgb_prompt(app, true);
                    } else if app.state.mouse_focus == Some(Focus::TextInput) {
                        app.state.app_status = AppStatus::UserInput;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                    }
                }
            }
            PopupMode::CustomRGBPromptBG => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_custom_rgb_prompt(app, false);
                    } else if app.state.mouse_focus == Some(Focus::TextInput) {
                        app.state.app_status = AppStatus::UserInput;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.close_popup();
                        app.state.app_status = AppStatus::Initialized;
                    }
                }
            }
            PopupMode::ViewCard => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    match app.state.mouse_focus.unwrap() {
                        Focus::CloseButton => {
                            app.close_popup();
                            app.state.app_status = AppStatus::Initialized;
                            if app.state.card_being_edited.is_some() {
                                app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                            }
                        }
                        Focus::CardName
                        | Focus::CardDescription
                        | Focus::CardTags
                        | Focus::CardComments
                        | Focus::CardDueDate => return handle_edit_new_card(app),
                        Focus::CardPriority => {
                            if app.state.card_being_edited.is_none() {
                                handle_edit_new_card(app);
                            }
                            app.set_popup_mode(PopupMode::CardPrioritySelector);
                            return AppReturn::Continue;
                        }
                        Focus::CardStatus => {
                            if app.state.card_being_edited.is_none() {
                                handle_edit_new_card(app);
                            }
                            app.set_popup_mode(PopupMode::CardStatusSelector);
                            return AppReturn::Continue;
                        }
                        Focus::SubmitButton => return handle_edit_card_submit(app),
                        _ => {}
                    }
                } else if mouse_scroll_down
                    && app.state.mouse_focus.is_some()
                    && app.state.mouse_focus.unwrap() == Focus::CardDescription
                {
                    app.state
                        .text_buffers
                        .card_description
                        .scroll(TextBoxScroll::Delta { rows: 1, cols: 0 })
                } else if mouse_scroll_up
                    && app.state.mouse_focus.is_some()
                    && app.state.mouse_focus.unwrap() == Focus::CardDescription
                {
                    app.state
                        .text_buffers
                        .card_description
                        .scroll(TextBoxScroll::Delta { rows: -1, cols: 0 })
                }
            }
            PopupMode::CardPrioritySelector => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    match app.state.mouse_focus.unwrap() {
                        Focus::CloseButton => {
                            app.state.app_status = AppStatus::Initialized;
                            if app.state.card_being_edited.is_some() {
                                app.set_popup_mode(PopupMode::ConfirmDiscardCardChanges);
                            }
                        }
                        Focus::ChangeCardPriorityPopup => {
                            return handle_change_card_priority(app, None)
                        }
                        _ => {}
                    }
                }
            }
            PopupMode::ConfirmDiscardCardChanges => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    match app.state.focus {
                        Focus::CloseButton => {
                            app.close_popup();
                            app.state.app_status = AppStatus::Initialized;
                            if app.state.card_being_edited.is_some() {
                                if app.state.card_being_edited.is_some() {
                                    warn!(
                                        "Discarding changes to card '{}'",
                                        app.state.card_being_edited.as_ref().unwrap().1.name
                                    );
                                    app.send_warning_toast(
                                        &format!(
                                            "Discarding changes to card '{}'",
                                            app.state.card_being_edited.as_ref().unwrap().1.name
                                        ),
                                        None,
                                    );
                                }
                                app.close_popup();
                                app.state.card_being_edited = None;
                            }
                        }
                        Focus::SubmitButton => {
                            app.state.app_status = AppStatus::Initialized;
                            app.close_popup();
                            return handle_edit_card_submit(app);
                        }
                        Focus::ExtraFocus => {
                            app.state.app_status = AppStatus::Initialized;
                            if app.state.card_being_edited.is_some() {
                                warn!(
                                    "Discarding changes to card '{}'",
                                    app.state.card_being_edited.as_ref().unwrap().1.name
                                );
                                app.send_warning_toast(
                                    &format!(
                                        "Discarding changes to card '{}'",
                                        app.state.card_being_edited.as_ref().unwrap().1.name
                                    ),
                                    None,
                                );
                            }
                            app.close_popup();
                            app.state.card_being_edited = None;
                        }
                        _ => {}
                    }
                }
            }
            PopupMode::FilterByTag => {
                if left_button_pressed {
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
        }
    } else {
        match app.state.ui_mode {
            UiMode::Zen
            | UiMode::TitleBody
            | UiMode::BodyHelp
            | UiMode::BodyLog
            | UiMode::TitleBodyHelp
            | UiMode::TitleBodyLog
            | UiMode::TitleBodyHelpLog
            | UiMode::BodyHelpLog
            | UiMode::ConfigMenu
            | UiMode::EditKeybindings
            | UiMode::HelpMenu
            | UiMode::NewBoard
            | UiMode::NewCard => {
                if left_button_pressed {
                    if let Some(value) = handle_left_click_for_ui_mode_mouse_action(app).await {
                        return value;
                    }
                } else if mouse_scroll_up
                    || mouse_scroll_down
                    || mouse_scroll_right
                    || mouse_scroll_left
                {
                    handle_scroll_for_ui_mode_mouse_action(
                        app,
                        mouse_scroll_up,
                        mouse_scroll_down,
                        mouse_scroll_right,
                        mouse_scroll_left,
                    );
                }
            }
            UiMode::MainMenu | UiMode::LogsOnly | UiMode::LoadLocalSave | UiMode::CreateTheme => {
                if left_button_pressed {
                    if let Some(value) = handle_left_click_for_ui_mode_mouse_action(app).await {
                        return value;
                    }
                }
            }
            UiMode::Login => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    handle_login_action(app).await
                }
            }
            UiMode::SignUp => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    handle_signup_action(app).await
                }
            }
            UiMode::ResetPassword => {
                if left_button_pressed && app.state.mouse_focus.is_some() {
                    handle_reset_password_action(app).await
                }
            }
            UiMode::LoadCloudSave => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_previous_ui_mode(app).await;
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

async fn handle_left_click_for_ui_mode_mouse_action(app: &mut App<'_>) -> Option<AppReturn> {
    let prv_ui_mode = app.state.ui_mode;
    app.state.mouse_focus?;
    // TODO: investigate if we really need to have mouse focus separate from focus
    let mouse_focus = app.state.mouse_focus.unwrap();
    match mouse_focus {
        Focus::Title => {
            app.set_ui_mode(UiMode::MainMenu);
            if app.state.app_list_states.main_menu.selected().is_none() {
                app.main_menu_next()
            }
        }
        Focus::Body => {
            if !(app.state.current_board_id.is_some() && app.state.current_card_id.is_some()) {
                app.send_error_toast("No card selected", None);
                return Some(AppReturn::Continue);
            }
            app.set_popup_mode(PopupMode::ViewCard);
        }
        Focus::Help => {
            app.set_ui_mode(UiMode::HelpMenu);
        }
        Focus::Log => {
            app.set_ui_mode(UiMode::LogsOnly);
        }
        Focus::ConfigTable => {
            return Some(handle_config_menu_action(app));
        }
        Focus::EditKeybindingsTable => {
            handle_edit_keybindings_action(app);
        }
        Focus::CloseButton => match prv_ui_mode {
            UiMode::Zen
            | UiMode::TitleBody
            | UiMode::BodyHelp
            | UiMode::BodyLog
            | UiMode::TitleBodyHelp
            | UiMode::TitleBodyLog
            | UiMode::BodyHelpLog
            | UiMode::TitleBodyHelpLog
            | UiMode::MainMenu => {
                return Some(handle_exit(app).await);
            }
            UiMode::NewBoard => {
                reset_new_board_form(app);
                handle_go_to_previous_ui_mode(app).await;
            }
            UiMode::NewCard => {
                reset_new_card_form(app);
                handle_go_to_previous_ui_mode(app).await;
            }
            UiMode::CreateTheme => {
                app.state.theme_being_edited = Theme::default();
            }
            _ => {
                handle_go_to_previous_ui_mode(app).await;
            }
        },
        Focus::SubmitButton => {
            app.state.set_focus(Focus::SubmitButton);
            match prv_ui_mode {
                UiMode::EditKeybindings => {
                    handle_edit_keybindings_action(app);
                }
                UiMode::NewBoard => {
                    handle_new_board_action(app);
                    app.state.app_status = AppStatus::Initialized;
                }
                UiMode::NewCard => {
                    handle_new_card_action(app);
                    app.state.app_status = AppStatus::Initialized;
                }
                UiMode::ConfigMenu => {
                    return Some(handle_config_menu_action(app));
                }
                UiMode::CreateTheme => {
                    return Some(handle_create_theme_action(app));
                }
                _ => {}
            }
        }
        Focus::ExtraFocus => {
            if prv_ui_mode == UiMode::ConfigMenu {
                return Some(handle_config_menu_action(app));
            } else if prv_ui_mode == UiMode::CreateTheme {
                return Some(handle_create_theme_action(app));
            }
        }
        Focus::MainMenu => {
            return Some(handle_main_menu_action(app).await);
        }
        Focus::NewBoardName
        | Focus::NewBoardDescription
        | Focus::CardName
        | Focus::CardDescription
        | Focus::CardDueDate => {
            app.state.app_status = AppStatus::UserInput;
            info!("Taking user input");
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

fn handle_scroll_for_ui_mode_mouse_action(
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
            app.state
                .text_buffers
                .board_description
                .scroll(TextBoxScroll::Delta { rows: -1, cols: 0 })
        } else if app.state.mouse_focus == Some(Focus::CardDescription) {
            app.state
                .text_buffers
                .card_description
                .scroll(TextBoxScroll::Delta { rows: -1, cols: 0 })
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
            app.state
                .text_buffers
                .board_description
                .scroll(TextBoxScroll::Delta { rows: 1, cols: 0 })
        } else if app.state.mouse_focus == Some(Focus::CardDescription) {
            app.state
                .text_buffers
                .card_description
                .scroll(TextBoxScroll::Delta { rows: 1, cols: 0 })
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
    app.state.config_item_being_edited =
        Some(app.state.app_table_states.config.selected().unwrap_or(0));
    let app_config_list = &app.config.to_view_list();
    if app.state.config_item_being_edited.unwrap_or(0) < app_config_list.len() {
        let default_config_item = String::from("");
        let config_item = &app_config_list[app.state.config_item_being_edited.unwrap_or(0)]
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
                app.set_ui_mode(UiMode::EditKeybindings);
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
                app.set_popup_mode(PopupMode::SelectDefaultView);
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
                app.set_popup_mode(PopupMode::ChangeTheme);
            }
            ConfigEnum::DateFormat => {
                app.set_popup_mode(PopupMode::ChangeDateFormatPopup);
            }
            _ => {
                app.set_popup_mode(PopupMode::EditGeneralConfig);
            }
        }
    } else {
        debug!(
            "Config item being edited {} is not in the AppConfig list",
            app.state.config_item_being_edited.unwrap_or(0)
        );
    }
    AppReturn::Continue
}

async fn handle_main_menu_action(app: &mut App<'_>) -> AppReturn {
    if app.state.app_list_states.main_menu.selected().is_some() {
        let selected_index = app.state.app_list_states.main_menu.selected().unwrap();
        let selected_item = app.main_menu.from_index(selected_index);
        match selected_item {
            MainMenuItem::Quit => return handle_exit(app).await,
            MainMenuItem::Config => {
                app.set_ui_mode(UiMode::ConfigMenu);
                if app.state.app_table_states.config.selected().is_none() {
                    app.config_next();
                }
            }
            MainMenuItem::View => {
                app.set_ui_mode(app.config.default_ui_mode);
            }
            MainMenuItem::Help => {
                app.set_ui_mode(UiMode::HelpMenu);
            }
            MainMenuItem::LoadSaveLocal => {
                app.set_ui_mode(UiMode::LoadLocalSave);
            }
            MainMenuItem::LoadSaveCloud => {
                if app.main_menu.logged_in {
                    app.set_ui_mode(UiMode::LoadCloudSave);
                    reset_preview_boards(app);
                    app.dispatch(IoEvent::GetCloudData).await;
                }
            }
        }
    }
    AppReturn::Continue
}

fn handle_default_view_selection(app: &mut App) {
    let all_ui_modes = UiMode::view_modes_as_string();
    let current_selected_mode = app
        .state
        .app_list_states
        .default_view
        .selected()
        .unwrap_or(0);
    if current_selected_mode < all_ui_modes.len() {
        let selected_mode = &all_ui_modes[current_selected_mode];
        app.config.default_ui_mode = UiMode::from_string(selected_mode).unwrap_or(UiMode::MainMenu);
        AppConfig::edit_config(app, ConfigEnum::DefaultView, selected_mode);
        app.state.app_list_states.default_view.select(Some(0));
        if app.state.app_table_states.config.selected().is_none() {
            app.config_next();
        }
        if app.state.popup_mode.is_some() {
            app.close_popup();
        }
    } else {
        debug!(
            "Selected mode {} is not in the list of all UI modes",
            current_selected_mode
        );
    }
}

fn handle_change_date_format(app: &mut App) {
    let all_date_formats = DateFormat::get_all_date_formats();
    let current_selected_format = app
        .state
        .app_list_states
        .date_format_selector
        .selected()
        .unwrap_or(0);
    if current_selected_format < all_date_formats.len() {
        let selected_format = &all_date_formats[current_selected_format];
        app.config.date_format = *selected_format;
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
        if app.state.popup_mode.is_some() {
            app.close_popup();
        }
    } else {
        debug!(
            "Selected format {} is not in the list of all date formats",
            current_selected_format
        );
    }
}

fn handle_change_ui_mode(app: &mut App) {
    let current_index = app
        .state
        .app_list_states
        .default_view
        .selected()
        .unwrap_or(0);
    let all_ui_modes = UiMode::view_modes_as_string()
        .iter()
        .filter_map(|s| UiMode::from_string(s))
        .collect::<Vec<UiMode>>();

    let current_index = if current_index >= all_ui_modes.len() {
        all_ui_modes.len() - 1
    } else {
        current_index
    };
    let selected_ui_mode = all_ui_modes[current_index];
    app.set_ui_mode(selected_ui_mode);
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
        app.set_popup_mode(PopupMode::EditSpecificKeyBinding);
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

pub async fn handle_go_to_previous_ui_mode(app: &mut App<'_>) -> AppReturn {
    if app.state.popup_mode.is_some() {
        match app.state.popup_mode.unwrap() {
            PopupMode::EditGeneralConfig => {
                if app.state.ui_mode == UiMode::CreateTheme {
                    app.close_popup();
                } else {
                    app.set_ui_mode(UiMode::ConfigMenu);
                    if app.state.app_table_states.config.selected().is_none() {
                        app.config_next()
                    }
                }
                app.state.text_buffers.general_config.reset();
            }
            PopupMode::EditSpecificKeyBinding => {
                app.set_ui_mode(UiMode::EditKeybindings);
                app.state.app_table_states.edit_keybindings.select(Some(0));
            }
            PopupMode::ViewCard => {
                if app.state.card_being_edited.is_some() {
                    warn!(
                        "Discarding changes to card '{}'",
                        app.state.card_being_edited.as_ref().unwrap().1.name
                    );
                    app.send_warning_toast(
                        &format!(
                            "Discarding changes to card '{}'",
                            app.state.card_being_edited.as_ref().unwrap().1.name
                        ),
                        None,
                    );
                }
                app.close_popup();
                app.state.card_being_edited = None;
            }
            PopupMode::ConfirmDiscardCardChanges => {
                if app.state.card_being_edited.is_some() {
                    warn!(
                        "Discarding changes to card '{}'",
                        app.state.card_being_edited.as_ref().unwrap().1.name
                    );
                    app.send_warning_toast(
                        &format!(
                            "Discarding changes to card '{}'",
                            app.state.card_being_edited.as_ref().unwrap().1.name
                        ),
                        None,
                    );
                }
                app.close_popup();
                app.state.card_being_edited = None;
            }
            PopupMode::FilterByTag => {
                app.state.filter_tags = None;
                app.state.all_available_tags = None;
                app.state.app_list_states.filter_by_tag_list.select(None);
            }
            PopupMode::ChangeTheme => {
                let config_theme = {
                    let all_themes = Theme::all_default_themes();
                    let default_theme = app.config.default_theme.clone();
                    all_themes.iter().find(|t| t.name == default_theme).cloned()
                };
                if config_theme.is_some() {
                    app.current_theme = config_theme.unwrap();
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
    match app.state.ui_mode {
        UiMode::MainMenu => handle_exit(app).await,
        UiMode::EditKeybindings => {
            app.set_ui_mode(UiMode::ConfigMenu);
            if app.state.app_table_states.config.selected().is_none() {
                app.config_next()
            }
            AppReturn::Continue
        }
        UiMode::LoadLocalSave => {
            app.state.app_list_states.load_save = ListState::default();
            go_to_previous_ui_mode_without_extras(app);
            AppReturn::Continue
        }
        UiMode::Login => {
            reset_login_form(app);
            go_to_previous_ui_mode_without_extras(app);
            AppReturn::Continue
        }
        UiMode::SignUp => {
            reset_signup_form(app);
            go_to_previous_ui_mode_without_extras(app);
            AppReturn::Continue
        }
        UiMode::ResetPassword => {
            reset_reset_password_form(app);
            go_to_previous_ui_mode_without_extras(app);
            AppReturn::Continue
        }
        _ => {
            go_to_previous_ui_mode_without_extras(app);
            AppReturn::Continue
        }
    }
}

fn go_to_previous_ui_mode_without_extras(app: &mut App) {
    if app.state.prev_ui_mode == Some(app.state.ui_mode) {
        app.set_ui_mode(UiMode::MainMenu);
        if app.state.app_list_states.main_menu.selected().is_none() {
            app.main_menu_next();
        }
    } else {
        let prev_ui_mode = app.state.prev_ui_mode.unwrap_or(UiMode::MainMenu);
        app.set_ui_mode(prev_ui_mode);
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

    if app.state.card_being_edited.is_some() {
        let card_being_edited = app.state.card_being_edited.clone().unwrap();
        let card_coordinates = card_being_edited.0;
        let mut card = card_being_edited.1;
        if selected_status == CardStatus::Complete {
            card.date_completed = Utc::now().to_string();
        } else {
            card.date_completed = FIELD_NOT_SET.to_string();
        }
        card.card_status = selected_status;
        app.state.card_being_edited = Some((card_coordinates, card));
        app.set_popup_mode(PopupMode::ViewCard);
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
                        current_card.date_completed = Utc::now().to_string();
                    } else {
                        current_card.date_completed = FIELD_NOT_SET.to_string();
                    }
                    current_card.date_modified = Utc::now().to_string();
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

    if app.state.card_being_edited.is_some() {
        let card_being_edited = app.state.card_being_edited.clone().unwrap();
        let card_coordinates = card_being_edited.0;
        let mut card = card_being_edited.1;
        card.priority = selected_priority;
        app.state.card_being_edited = Some((card_coordinates, card));
        app.set_popup_mode(PopupMode::ViewCard);
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
                    current_card.date_modified = Utc::now().to_string();
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
    app.state.config_item_being_edited = None;
    app.state.text_buffers.general_config.reset();
    app.set_ui_mode(UiMode::ConfigMenu);
    refresh_visible_boards_and_cards(app);
}

fn handle_edit_specific_keybinding(app: &mut App) {
    if app.state.edited_keybinding.is_some() {
        let selected = app
            .state
            .app_table_states
            .edit_keybindings
            .selected()
            .unwrap();
        if selected < app.config.keybindings.iter().count() {
            let result = app.config.edit_keybinding(
                selected,
                app.state.edited_keybinding.clone().unwrap_or_default(),
            );
            if let Err(e) = result {
                app.send_error_toast(&format!("Error editing Keybinding: {}", e), None);
            } else {
                let mut key_list = vec![];
                for (k, v) in app.config.keybindings.iter() {
                    key_list.push((k, v));
                }
                let (key, _) = &key_list[selected];
                let key_string = key.to_string();
                let value = app.state.edited_keybinding.clone().unwrap_or_default();
                let value = value
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                app.send_info_toast(
                    &format!("Keybinding for {} updated to {}", key_string, value),
                    None,
                );
            }
        } else {
            error!("Selected Keybinding with id {} not found", selected);
            app.send_error_toast("Selected Keybinding not found", None);
            app.state.edited_keybinding = None;
            app.state.app_table_states.edit_keybindings.select(None);
        }
        app.set_ui_mode(UiMode::EditKeybindings);
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
        app.set_ui_mode(UiMode::EditKeybindings);
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
            app.set_ui_mode(
                *app.state
                    .prev_ui_mode
                    .as_ref()
                    .unwrap_or(&app.config.default_ui_mode),
            );
        } else {
            warn!("New board name is empty or already exists");
            app.send_warning_toast("New board name is empty or already exists", None);
        }
        app.set_ui_mode(
            *app.state
                .prev_ui_mode
                .as_ref()
                .unwrap_or(&app.config.default_ui_mode),
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

fn handle_new_card_action(app: &mut App) {
    if app.state.focus == Focus::SubmitButton {
        let new_card_name = app.state.text_buffers.card_name.get_joined_lines();
        let new_card_name = new_card_name.trim();
        let new_card_description = app.state.text_buffers.card_description.get_joined_lines();
        let new_card_description = new_card_description.trim();
        let new_card_due_date = app.state.text_buffers.card_due_date.get_joined_lines();
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
            app.set_ui_mode(
                *app.state
                    .prev_ui_mode
                    .as_ref()
                    .unwrap_or(&app.config.default_ui_mode),
            );
            return;
        }
        let parsed_due_date =
            date_format_converter(new_card_due_date.trim(), app.config.date_format);
        if parsed_due_date.is_err() {
            let all_date_formats = DateFormat::get_all_date_formats()
                .iter()
                .map(|x| x.to_human_readable_string())
                .collect::<Vec<&str>>()
                .join(", ");
            app.send_warning_toast(
                &format!(
                    "Invalid date format '{}'. Please use any of the following {}. Date has been reset and other changes have been saved.",
                    &new_card_due_date, all_date_formats
                ),
                Some(Duration::from_secs(10)),
            );
            warn!("Invalid date format '{}'. Please use any of the following {}. Date has been reset and other changes have been saved.",
            &new_card_due_date, all_date_formats);
        }
        let parsed_date = if let Ok(parsed_due_date) = parsed_due_date {
            parsed_due_date
        } else {
            FIELD_NOT_SET.to_string()
        };
        if !new_card_name.is_empty() && !same_name_exists {
            let new_card = Card::new(
                new_card_name,
                new_card_description,
                &parsed_date,
                CardPriority::Low,
                vec![],
                vec![],
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
                app.set_ui_mode(
                    *app.state
                        .prev_ui_mode
                        .as_ref()
                        .unwrap_or(&app.config.default_ui_mode),
                );
                return;
            }
            app.set_ui_mode(
                *app.state
                    .prev_ui_mode
                    .as_ref()
                    .unwrap_or(&app.config.default_ui_mode),
            );
        } else {
            warn!("New card name is empty or already exists");
            app.send_warning_toast("New card name is empty or already exists", None);
        }

        if let Some(previous_focus) = &app.state.prev_focus {
            app.state.set_focus(*previous_focus);
        }
        refresh_visible_boards_and_cards(app);
        reset_new_card_form(app);
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
        let config_index = app.state.app_table_states.config.selected();
        if config_index.is_some() {
            let config_item_index = &app.state.config_item_being_edited;
            let list_items = app.config.to_view_list();
            let config_item_name = if config_item_index.is_some() {
                list_items[config_item_index.unwrap()].first().unwrap()
            } else {
                // NOTE: This is temporary, as only the Theme editor uses this other than config
                "Theme Name"
            };
            if config_item_name == "Default Theme" {
                let theme_index = app
                    .state
                    .app_list_states
                    .theme_selector
                    .selected()
                    .unwrap_or(0);
                if theme_index < app.all_themes.len() {
                    let theme = app.all_themes[theme_index].clone();
                    app.config.default_theme.clone_from(&theme.name);
                    AppConfig::edit_config(app, ConfigEnum::DefaultTheme, theme.name.as_str());
                } else {
                    debug!("Theme index {} is not in the theme list", theme_index);
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
        let selected_item_index = app.state.app_list_states.theme_selector.selected();
        if selected_item_index.is_none() {
            debug!("No selected item index found");
            app.send_error_toast("Something went wrong", None);
            app.close_popup();
            return AppReturn::Continue;
        }
        let selected_item_index = selected_item_index.unwrap();
        let selected_theme = if selected_item_index < app.all_themes.len() {
            app.all_themes[selected_item_index].clone()
        } else {
            debug!("Selected item index is out of bounds");
            app.all_themes[0].clone()
        };
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
    if app.state.popup_mode.is_some() {
        match app.state.popup_mode.unwrap() {
            PopupMode::EditGeneralConfig => {
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
            PopupMode::EditThemeStyle => {
                match app.state.focus {
                    Focus::SubmitButton => {
                        let all_color_options =
                            TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
                        let all_modifier_options =
                            TextModifierOptions::to_iter().collect::<Vec<TextModifierOptions>>();
                        if app
                            .state
                            .app_list_states
                            .edit_specific_style
                            .0
                            .selected()
                            .is_none()
                        {
                            app.state
                                .app_list_states
                                .edit_specific_style
                                .0
                                .select(Some(0));
                        }
                        if app
                            .state
                            .app_list_states
                            .edit_specific_style
                            .1
                            .selected()
                            .is_none()
                        {
                            app.state
                                .app_list_states
                                .edit_specific_style
                                .1
                                .select(Some(0));
                        }
                        if app
                            .state
                            .app_list_states
                            .edit_specific_style
                            .2
                            .selected()
                            .is_none()
                        {
                            app.state
                                .app_list_states
                                .edit_specific_style
                                .2
                                .select(Some(0));
                        }
                        let selected_fg_index =
                            app.state.app_list_states.edit_specific_style.0.selected();
                        let selected_bg_index =
                            app.state.app_list_states.edit_specific_style.1.selected();
                        let selected_modifier_index =
                            app.state.app_list_states.edit_specific_style.2.selected();

                        let theme_style_bring_edited_index =
                            app.state.app_table_states.theme_editor.selected();
                        if theme_style_bring_edited_index.is_none() {
                            debug!("No theme style being edited index found");
                            app.send_error_toast("Something went wrong", None);
                            app.close_popup();
                            return AppReturn::Continue;
                        }
                        let theme_style_bring_edited_index =
                            theme_style_bring_edited_index.unwrap();
                        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
                            [theme_style_bring_edited_index];

                        let mut fg_color = all_color_options[selected_fg_index.unwrap()].clone();
                        let mut bg_color = all_color_options[selected_bg_index.unwrap()].clone();
                        let modifier =
                            all_modifier_options[selected_modifier_index.unwrap()].to_modifier();

                        if let TextColorOptions::RGB(_, _, _) = fg_color {
                            if !app.state.text_buffers.theme_editor_fg_rgb.is_empty() {
                                let input = app
                                    .state
                                    .text_buffers
                                    .theme_editor_fg_rgb
                                    .get_joined_lines();
                                let split_input =
                                    input.split(',').map(|s| s.to_string().trim().to_string());
                                if split_input.clone().count() == 3 {
                                    let mut input_is_valid = true;
                                    for i in split_input.clone() {
                                        if i.parse::<u8>().is_err() {
                                            input_is_valid = false;
                                        }
                                    }
                                    if input_is_valid {
                                        let r = split_input
                                            .clone()
                                            .next()
                                            .unwrap()
                                            .parse::<u8>()
                                            .unwrap();
                                        let g = split_input
                                            .clone()
                                            .nth(1)
                                            .unwrap()
                                            .parse::<u8>()
                                            .unwrap();
                                        let b = split_input
                                            .clone()
                                            .nth(2)
                                            .unwrap()
                                            .parse::<u8>()
                                            .unwrap();
                                        fg_color = TextColorOptions::RGB(r, g, b);
                                    }
                                }
                            }
                        }
                        if let TextColorOptions::RGB(_, _, _) = bg_color {
                            if !app.state.text_buffers.theme_editor_bg_rgb.is_empty() {
                                let input = app
                                    .state
                                    .text_buffers
                                    .theme_editor_bg_rgb
                                    .get_joined_lines();
                                let split_input =
                                    input.split(',').map(|s| s.to_string().trim().to_string());
                                if split_input.clone().count() == 3 {
                                    let mut input_is_valid = true;
                                    for i in split_input.clone() {
                                        if i.parse::<u8>().is_err() {
                                            input_is_valid = false;
                                        }
                                    }
                                    if input_is_valid {
                                        let r = split_input
                                            .clone()
                                            .next()
                                            .unwrap()
                                            .parse::<u8>()
                                            .unwrap();
                                        let g = split_input
                                            .clone()
                                            .nth(1)
                                            .unwrap()
                                            .parse::<u8>()
                                            .unwrap();
                                        let b = split_input
                                            .clone()
                                            .nth(2)
                                            .unwrap()
                                            .parse::<u8>()
                                            .unwrap();
                                        bg_color = TextColorOptions::RGB(r, g, b);
                                    }
                                }
                            }
                        }
                        let parsed_fg_color = fg_color.to_color().unwrap_or(Color::Reset);
                        let parsed_bg_color = bg_color.to_color().unwrap_or(Color::Reset);

                        app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                            theme_style_being_edited,
                            Some(parsed_fg_color),
                            Some(parsed_bg_color),
                            Some(modifier),
                        );
                        app.close_popup();
                        app.state.set_focus(Focus::ThemeEditor);
                    }
                    Focus::StyleEditorFG => {
                        let selected_index =
                            app.state.app_list_states.edit_specific_style.0.selected();
                        if selected_index.is_none() {
                            return AppReturn::Continue;
                        }
                        let selected_index = selected_index.unwrap();
                        let all_color_options =
                            TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
                        let selected_color = &all_color_options[selected_index];
                        if let TextColorOptions::RGB(_, _, _) = selected_color {
                            app.set_popup_mode(PopupMode::CustomRGBPromptFG);
                            app.state.set_focus(Focus::TextInput);
                            return AppReturn::Continue;
                        }
                    }
                    Focus::StyleEditorBG => {
                        let selected_index =
                            app.state.app_list_states.edit_specific_style.1.selected();
                        if selected_index.is_none() {
                            return AppReturn::Continue;
                        }
                        let selected_index = selected_index.unwrap();
                        let all_color_options =
                            TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
                        let selected_color = &all_color_options[selected_index];
                        if let TextColorOptions::RGB(_, _, _) = selected_color {
                            app.set_popup_mode(PopupMode::CustomRGBPromptBG);
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
        app.set_popup_mode(PopupMode::SaveThemePrompt);
    } else if app.state.focus == Focus::ThemeEditor
        && app.state.app_table_states.theme_editor.selected().is_some()
    {
        let selected_item_index = app.state.app_table_states.theme_editor.selected().unwrap();
        if selected_item_index == 0 {
            app.set_popup_mode(PopupMode::EditGeneralConfig);
        } else {
            app.set_popup_mode(PopupMode::EditThemeStyle);
        }
    } else if app.state.focus == Focus::ExtraFocus {
        app.state.theme_being_edited = Theme::default();
        // TODO: Handle this
        // app.clear_user_input_state();
        app.send_info_toast("Theme reset to default", None);
    }
    AppReturn::Continue
}

fn handle_next_focus(app: &mut App) {
    if app.config.enable_mouse_support {
        reset_mouse(app)
    }
    let available_targets = if app.state.popup_mode.is_some() {
        PopupMode::get_available_targets(&app.state.popup_mode.unwrap())
    } else {
        UiMode::get_available_targets(&app.state.ui_mode)
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
    if app.state.card_being_edited.is_none()
        && app.state.popup_mode.is_some()
        && app.state.popup_mode.unwrap() == PopupMode::ViewCard
        && next_focus == Focus::SubmitButton
    {
        next_focus = Focus::CardName;
    }
    if next_focus != Focus::NoFocus {
        app.state.set_focus(next_focus);
    }
    if app.state.popup_mode == Some(PopupMode::CommandPalette) {
        app.state
            .app_list_states
            .command_palette_command_search
            .select(None);
        app.state
            .app_list_states
            .command_palette_card_search
            .select(None);
        app.state
            .app_list_states
            .command_palette_board_search
            .select(None);
    }
}

fn handle_prv_focus(app: &mut App) {
    if app.config.enable_mouse_support {
        reset_mouse(app)
    }
    let available_targets = if app.state.popup_mode.is_some() {
        PopupMode::get_available_targets(&app.state.popup_mode.unwrap())
    } else {
        UiMode::get_available_targets(&app.state.ui_mode)
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
    if app.state.card_being_edited.is_none()
        && app.state.popup_mode.is_some()
        && app.state.popup_mode.unwrap() == PopupMode::ViewCard
        && prv_focus == Focus::SubmitButton
    {
        prv_focus = Focus::CardComments;
    }
    if prv_focus != Focus::NoFocus {
        app.state.set_focus(prv_focus);
    }
    if app.state.popup_mode == Some(PopupMode::CommandPalette) {
        app.state
            .app_list_states
            .command_palette_command_search
            .select(None);
        app.state
            .app_list_states
            .command_palette_card_search
            .select(None);
        app.state
            .app_list_states
            .command_palette_board_search
            .select(None);
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
    }
    app.all_themes.push(app.state.theme_being_edited.clone());
    app.state.theme_being_edited = Theme::default();
    app.close_popup();
    handle_prv_focus(app);
}

fn handle_custom_rgb_prompt(app: &mut App, fg: bool) -> AppReturn {
    if app.state.focus == Focus::TextInput {
        app.state.app_status = AppStatus::UserInput;
    } else if app.state.focus == Focus::SubmitButton {
        let fg_rgb_values = app
            .state
            .text_buffers
            .theme_editor_fg_rgb
            .get_joined_lines();
        let bg_rgb_values = app
            .state
            .text_buffers
            .theme_editor_bg_rgb
            .get_joined_lines();

        let rgb_values: Vec<&str> = if fg {
            fg_rgb_values.trim().split(',').collect::<Vec<&str>>()
        } else {
            bg_rgb_values.trim().split(',').collect::<Vec<&str>>()
        };

        let rgb_values = rgb_values.iter().map(|x| x.trim()).collect::<Vec<&str>>();

        if rgb_values.len() != 3 {
            app.send_error_toast("Invalid RGB format. Please enter the RGB values in the format x,y,z where x,y,z are three digit numbers from 0 to 255", None);
            return AppReturn::Continue;
        }
        let rgb_values = rgb_values
            .iter()
            .map(|x| x.parse::<u8>())
            .collect::<Result<Vec<u8>, _>>();
        if rgb_values.is_err() {
            app.send_error_toast("Invalid RGB format. Please enter the RGB values in the format x,y,z where x,y,z are three digit numbers from 0 to 255", None);
            return AppReturn::Continue;
        }
        let rgb_values = rgb_values.unwrap();
        let all_color_options = TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
        let selected_index = app.state.app_list_states.edit_specific_style.0.selected();
        if selected_index.is_none() {
            debug!("No selected index found");
            app.send_error_toast("Something went wrong", None);
        }
        let selected_index = selected_index.unwrap();
        if selected_index >= all_color_options.len() {
            debug!("Selected index is out of bounds");
            app.send_error_toast("Something went wrong", None);
        }
        let theme_style_bring_edited_index = app.state.app_table_states.theme_editor.selected();
        if theme_style_bring_edited_index.is_none() {
            debug!("No theme style being edited index found");
            app.send_error_toast("Something went wrong", None);
        }
        let theme_style_bring_edited_index = theme_style_bring_edited_index.unwrap();
        if theme_style_bring_edited_index >= app.state.theme_being_edited.to_vec_str().len() {
            debug!("Theme style being edited index is out of bounds");
            app.send_error_toast("Something went wrong", None);
        }
        let theme_style_being_edited =
            app.state.theme_being_edited.to_vec_str()[theme_style_bring_edited_index];
        if fg {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                Some(Color::Rgb(rgb_values[0], rgb_values[1], rgb_values[2])),
                None,
                None,
            );
        } else {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                None,
                Some(Color::Rgb(rgb_values[0], rgb_values[1], rgb_values[2])),
                None,
            );
        }
        app.set_popup_mode(PopupMode::EditThemeStyle);
    }
    AppReturn::Continue
}

fn handle_theme_maker_scroll_up(app: &mut App) {
    if app.state.focus == Focus::StyleEditorFG {
        let current_index = app.state.app_list_states.edit_specific_style.0.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state
                .app_list_states
                .edit_specific_style
                .0
                .select(Some(0));
        }
        let current_index = app
            .state
            .app_list_states
            .edit_specific_style
            .0
            .selected()
            .unwrap();
        let selector_index = if current_index > 0 {
            current_index - 1
        } else {
            total_length - 1
        };
        app.state
            .app_list_states
            .edit_specific_style
            .0
            .select(Some(selector_index));
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
            [app.state.app_table_states.theme_editor.selected().unwrap()];
        if TextColorOptions::to_iter().nth(selector_index).is_some() {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                TextColorOptions::to_iter()
                    .nth(selector_index)
                    .unwrap()
                    .to_color(),
                None,
                None,
            );
        }
    } else if app.state.focus == Focus::StyleEditorBG {
        let current_index = app.state.app_list_states.edit_specific_style.1.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state
                .app_list_states
                .edit_specific_style
                .1
                .select(Some(0));
        }
        let current_index = app
            .state
            .app_list_states
            .edit_specific_style
            .1
            .selected()
            .unwrap();
        let selector_index = if current_index > 0 {
            current_index - 1
        } else {
            total_length - 1
        };
        app.state
            .app_list_states
            .edit_specific_style
            .1
            .select(Some(selector_index));
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
            [app.state.app_table_states.theme_editor.selected().unwrap()];
        if TextColorOptions::to_iter().nth(selector_index).is_some() {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                None,
                TextColorOptions::to_iter()
                    .nth(selector_index)
                    .unwrap()
                    .to_color(),
                None,
            );
        }
    } else if app.state.focus == Focus::StyleEditorModifier {
        let current_index = app.state.app_list_states.edit_specific_style.2.selected();
        let total_length = TextModifierOptions::to_iter().count();
        if current_index.is_none() {
            app.state
                .app_list_states
                .edit_specific_style
                .2
                .select(Some(0));
        }
        let current_index = app
            .state
            .app_list_states
            .edit_specific_style
            .2
            .selected()
            .unwrap();
        let selector_index = if current_index > 0 {
            current_index - 1
        } else {
            total_length - 1
        };
        app.state
            .app_list_states
            .edit_specific_style
            .2
            .select(Some(selector_index));
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
            [app.state.app_table_states.theme_editor.selected().unwrap()];
        if TextModifierOptions::to_iter().nth(selector_index).is_some() {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                None,
                None,
                Some(
                    TextModifierOptions::to_iter()
                        .nth(selector_index)
                        .unwrap()
                        .to_modifier(),
                ),
            );
        }
    }
}

fn handle_theme_maker_scroll_down(app: &mut App) {
    if app.state.focus == Focus::StyleEditorFG {
        let current_index = app.state.app_list_states.edit_specific_style.0.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state
                .app_list_states
                .edit_specific_style
                .0
                .select(Some(0));
        }
        let current_index = app
            .state
            .app_list_states
            .edit_specific_style
            .0
            .selected()
            .unwrap();
        let selector_index = if current_index < total_length - 1 {
            current_index + 1
        } else {
            0
        };
        app.state
            .app_list_states
            .edit_specific_style
            .0
            .select(Some(selector_index));
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
            [app.state.app_table_states.theme_editor.selected().unwrap()];
        if TextColorOptions::to_iter().nth(selector_index).is_some() {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                TextColorOptions::to_iter()
                    .nth(selector_index)
                    .unwrap()
                    .to_color(),
                None,
                None,
            );
        }
    } else if app.state.focus == Focus::StyleEditorBG {
        let current_index = app.state.app_list_states.edit_specific_style.1.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state
                .app_list_states
                .edit_specific_style
                .1
                .select(Some(0));
        }
        let current_index = app
            .state
            .app_list_states
            .edit_specific_style
            .1
            .selected()
            .unwrap();
        let selector_index = if current_index < total_length - 1 {
            current_index + 1
        } else {
            0
        };
        app.state
            .app_list_states
            .edit_specific_style
            .1
            .select(Some(selector_index));
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
            [app.state.app_table_states.theme_editor.selected().unwrap()];
        if TextColorOptions::to_iter().nth(selector_index).is_some() {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                None,
                TextColorOptions::to_iter()
                    .nth(selector_index)
                    .unwrap()
                    .to_color(),
                None,
            );
        }
    } else if app.state.focus == Focus::StyleEditorModifier {
        let current_index = app.state.app_list_states.edit_specific_style.2.selected();
        let total_length = TextModifierOptions::to_iter().count();
        if current_index.is_none() {
            app.state
                .app_list_states
                .edit_specific_style
                .2
                .select(Some(0));
        }
        let current_index = app
            .state
            .app_list_states
            .edit_specific_style
            .2
            .selected()
            .unwrap();
        let selector_index = if current_index < total_length - 1 {
            current_index + 1
        } else {
            0
        };
        app.state
            .app_list_states
            .edit_specific_style
            .2
            .select(Some(selector_index));
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
            [app.state.app_table_states.theme_editor.selected().unwrap()];
        if TextModifierOptions::to_iter().nth(selector_index).is_some() {
            app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                theme_style_being_edited,
                None,
                None,
                Some(
                    TextModifierOptions::to_iter()
                        .nth(selector_index)
                        .unwrap()
                        .to_modifier(),
                ),
            );
        }
    }
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
    app.state.text_buffers.card_due_date.reset();
    app.state
        .text_buffers
        .card_due_date
        .insert_str(&card.due_date);
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
    let mut edited_card = app.state.card_being_edited.as_ref().unwrap().1.clone();
    let card_due_date = edited_card.due_date.clone();
    let parsed_due_date = date_format_converter(card_due_date.trim(), app.config.date_format);
    let parsed_date = match parsed_due_date {
        Ok(date) => {
            if date.is_empty() {
                FIELD_NOT_SET.to_string()
            } else {
                date
            }
        }
        Err(_) => {
            send_warning_toast = true;
            warning_due_date = card_due_date;
            FIELD_NOT_SET.to_string()
        }
    };
    edited_card.due_date = parsed_date;
    edited_card.date_modified = Utc::now().to_string();
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
        let all_date_formats = DateFormat::get_all_date_formats()
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
    let calculated_tags = CommandPaletteWidget::calculate_tags(app);
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
    let mut number_of_times_to_go_right = 0;
    let mut number_of_times_to_go_down = 0;
    for (board_index, board) in app.boards.get_boards().iter().enumerate() {
        for (card_index, card) in board.cards.get_all_cards().iter().enumerate() {
            if card.id == card_id {
                number_of_times_to_go_right = board_index;
                number_of_times_to_go_down = card_index;
                break;
            }
        }
    }
    for _ in 0..number_of_times_to_go_right {
        go_right(app);
    }
    for _ in 0..number_of_times_to_go_down {
        go_down(app);
    }
    app.set_popup_mode(PopupMode::ViewCard);
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
    // TODO: Maybe animate the border of the board for better visibility
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
    app.state.text_buffers.card_due_date.reset();
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
            handle_go_to_previous_ui_mode(app).await;
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::SubmitButton => {
            handle_login_submit_action(app).await;
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::ExtraFocus => {
            app.state.show_password = !app.state.show_password;
        }
        Focus::Title => {
            app.set_ui_mode(UiMode::MainMenu);
            reset_login_form(app);
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::EmailIDField | Focus::PasswordField => {
            if app.state.app_status != AppStatus::UserInput {
                app.state.app_status = AppStatus::UserInput;
                info!("Taking user input");
            }
        }
        _ => {}
    };
}

async fn handle_signup_action(app: &mut App<'_>) {
    match app.state.focus {
        Focus::CloseButton => {
            reset_signup_form(app);
            handle_go_to_previous_ui_mode(app).await;
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::SubmitButton => {
            handle_signup_submit_action(app).await;
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::ExtraFocus => {
            app.state.show_password = !app.state.show_password;
        }
        Focus::Title => {
            app.set_ui_mode(UiMode::MainMenu);
            reset_signup_form(app);
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::EmailIDField | Focus::PasswordField | Focus::ConfirmPasswordField => {
            if app.state.app_status != AppStatus::UserInput {
                app.state.app_status = AppStatus::UserInput;
                info!("Taking user input");
            }
        }
        _ => {}
    }
}

async fn handle_reset_password_action(app: &mut App<'_>) {
    match app.state.focus {
        Focus::CloseButton => {
            reset_reset_password_form(app);
            handle_go_to_previous_ui_mode(app).await;
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::Title => {
            app.set_ui_mode(UiMode::MainMenu);
            reset_reset_password_form(app);
            if app.state.app_status == AppStatus::UserInput {
                app.state.app_status = AppStatus::Initialized;
                info!("Exiting user input mode");
            }
        }
        Focus::EmailIDField
        | Focus::ResetPasswordLinkField
        | Focus::PasswordField
        | Focus::ConfirmPasswordField => {
            if app.state.app_status != AppStatus::UserInput {
                app.state.app_status = AppStatus::UserInput;
                info!("Taking user input");
            }
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
