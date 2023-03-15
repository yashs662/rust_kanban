use std::time::Duration;

use chrono::{NaiveDate, NaiveDateTime};
use linked_hash_map::LinkedHashMap;
use log::{debug, error, info, warn};
use tui::{style::Color, widgets::ListState};

use crate::{
    app::{state::KeyBindings, AppConfig},
    constants::{
        DEFAULT_TOAST_DURATION, FIELD_NOT_SET, IO_EVENT_WAIT_TIME, MOUSE_OUT_OF_BOUNDS_COORDINATES,
        NEW_BOARD_FORM_DEFAULT_STATE, NEW_CARD_FORM_DEFAULT_STATE,
    },
    inputs::{key::Key, mouse::Mouse},
    io::{
        data_handler::{get_config, save_theme, write_config},
        handler::refresh_visible_boards_and_cards,
        IoEvent,
    },
    ui::{
        ui_helper::get_config_items,
        widgets::{CommandPalette, ToastType, ToastWidget},
        TextColorOptions, TextModifierOptions, Theme,
    },
};

use super::{
    actions::Action,
    kanban::{Board, Card, CardPriority, CardStatus},
    state::{AppStatus, Focus, UiMode},
    App, AppReturn, AppState, MainMenu, MainMenuItem, PopupMode,
};

pub fn go_right(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let all_boards = app.boards.clone();
    let current_board_id = app.state.current_board_id;
    // check if current_board_id is set, if not assign to the first board
    // check if all_boards is empty, if so, return
    if all_boards.is_empty() {
        error!("Cannot go right: no boards found");
        app.send_error_toast("Cannot go right: no boards found", None);
        return;
    }
    let current_board_id = if current_board_id.is_none() {
        all_boards[0].id
    } else {
        current_board_id.unwrap()
    };
    // check if the current board is the last one in visible_boards which is a LinkedHashMap of board_id and card_ids
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
            app.state.current_board_id =
                Some(current_visible_boards.keys().next().unwrap().clone());
        }
    }
    let current_board_index = current_board_index.unwrap();
    if current_board_index == current_visible_boards.len() - 1 {
        // we are at the last board, check the index for the current board in all boards, if it is the last one, we cannot go right
        let current_board_index_in_all_boards = all_boards
            .iter()
            .position(|board| board.id == current_board_id);
        if current_board_index_in_all_boards.is_none() {
            debug!("Cannot go right: current board not found");
            app.send_error_toast("Cannot go right: Something went wrong", None);
            return;
        }
        let current_board_index_in_all_boards = current_board_index_in_all_boards.unwrap();
        if current_board_index_in_all_boards == all_boards.len() - 1 {
            // we are at the last board, we cannot go right
            app.send_error_toast("Cannot go right: Already at the last board", None);
            return;
        }
        // we are not at the last board, we can go right
        // get the next NO_OF_BOARDS_PER_PAGE boards
        let next_board_index = current_board_index_in_all_boards + 1;
        let next_board = &all_boards[next_board_index];
        let next_board_card_ids: Vec<u128> = next_board
            .cards
            .clone()
            .iter()
            .map(|card| card.id)
            .collect();
        app.visible_boards_and_cards
            .insert(next_board.id, next_board_card_ids.clone());
        // remove the first board from visible_boards
        let first_board_id = app
            .visible_boards_and_cards
            .iter()
            .nth(0)
            .unwrap()
            .0
            .clone();
        app.visible_boards_and_cards.remove(&first_board_id);
        app.state.current_board_id = Some(next_board.id);
        // reset the current card id to first card of current board from visible_boards if there is any
        if next_board_card_ids.len() > 0 {
            app.state.current_card_id = Some(next_board_card_ids[0]);
        } else {
            app.state.current_card_id = None;
        }
    } else {
        // we are not at the last board, we can go right
        let next_board_id = current_visible_boards
            .iter()
            .nth(current_board_index + 1)
            .unwrap()
            .0
            .clone();
        app.state.current_board_id = Some(next_board_id);
        // reset the current card id to first card of current board from visible_boards if there is any
        let current_board_cards = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == next_board_id)
            .unwrap()
            .1
            .clone();
        if current_board_cards.len() > 0 {
            app.state.current_card_id = Some(current_board_cards[0]);
        } else {
            app.state.current_card_id = None;
        }
    }
}

pub fn go_left(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let all_boards = app.boards.clone();
    let current_board_id = app.state.current_board_id;
    // check if current_board_id is set, if not assign to the first board
    // check if all_boards is empty, if so, return
    if all_boards.is_empty() {
        error!("Cannot go left: no boards");
        app.send_error_toast("Cannot go left: no boards", None);
        return;
    }
    let current_board_id = if current_board_id.is_none() {
        all_boards[0].id
    } else {
        current_board_id.unwrap()
    };
    // check if the current board is the first one in visible_boards which is a LinkedHashMap of board_id and card_ids
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
        // we are at the first board, check the index for the current board in all boards, if it is the first one, we cannot go left
        let current_board_index_in_all_boards = all_boards
            .iter()
            .position(|board| board.id == current_board_id);
        if current_board_index_in_all_boards.is_none() {
            debug!("Cannot go left: current board not found");
            app.send_error_toast("Cannot go left: Something went wrong", None);
            return;
        }
        let current_board_index_in_all_boards = current_board_index_in_all_boards.unwrap();
        if current_board_index_in_all_boards == 0 {
            // we are at the first board, we cannot go left
            app.send_error_toast("Cannot go left: Already at the first board", None);
            return;
        }
        // we are not at the first board, we can go left
        // get the previous NO_OF_BOARDS_PER_PAGE boards
        let previous_board_index = current_board_index_in_all_boards - 1;
        let previous_board = &all_boards[previous_board_index];
        let previous_board_card_ids: Vec<u128> = previous_board
            .cards
            .clone()
            .iter()
            .map(|card| card.id)
            .collect();
        let mut new_visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>> = LinkedHashMap::new();
        new_visible_boards_and_cards.insert(previous_board.id, previous_board_card_ids.clone());
        for (board_id, card_ids) in current_visible_boards
            .iter()
            .take(current_visible_boards.len() - 1)
        {
            new_visible_boards_and_cards.insert(*board_id, card_ids.clone());
        }
        app.visible_boards_and_cards = new_visible_boards_and_cards;
        app.state.current_board_id = Some(previous_board.id);
        // reset the current card id to first card of current board from visible_boards if there is any
        if previous_board_card_ids.len() > 0 {
            app.state.current_card_id = Some(previous_board_card_ids[0]);
        } else {
            app.state.current_card_id = None;
        }
    } else {
        // we are not at the first board, we can go left
        let previous_board_id = current_visible_boards
            .iter()
            .nth(current_board_index - 1)
            .unwrap()
            .0
            .clone();
        app.state.current_board_id = Some(previous_board_id);
        // reset the current card id to first card of current board from visible_boards if there is any
        let current_visible_cards = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == previous_board_id)
            .unwrap()
            .1
            .clone();
        if current_visible_cards.len() > 0 {
            app.state.current_card_id = Some(current_visible_cards[0]);
        } else {
            app.state.current_card_id = None;
        }
    }
}

pub fn go_up(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;
    // check if app.board is empty, if so, return
    if current_visible_boards.is_empty() {
        return;
    }
    let current_board_id = if current_board_id.is_none() {
        app.boards[0].id
    } else {
        current_board_id.unwrap()
    };
    let current_card_id = if current_card_id.is_none() {
        // get the first card of the current board
        let current_board = app.boards.iter().find(|board| board.id == current_board_id);
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
        current_board.cards[0].id
    } else {
        current_card_id.unwrap()
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
        let current_card_index_in_all_cards = app
            .boards
            .iter()
            .find(|board| board.id == current_board_id)
            .unwrap()
            .cards
            .iter()
            .position(|card| card.id == current_card_id);
        if current_card_index_in_all_cards.is_none() {
            debug!("Cannot go up: current card not found");
            app.send_error_toast("Cannot go up: Something went wrong", None);
            return;
        }
        let current_card_index_in_all_cards = current_card_index_in_all_cards.unwrap();
        if current_card_index_in_all_cards == 0 {
            // we are at the first card, we cannot go up
            app.send_error_toast("Cannot go up: Already at the first card", None);
            return;
        }
        // we are not at the first card, we can go up
        // get the previous NO_OF_CARDS_PER_PAGE cards
        let previous_card_index = current_card_index_in_all_cards - 1;
        let previous_card_id = app
            .boards
            .iter()
            .find(|board| board.id == current_board_id)
            .unwrap()
            .cards[previous_card_index]
            .id;
        let previous_cards = app
            .boards
            .iter()
            .find(|board| board.id == current_board_id)
            .unwrap()
            .cards
            [previous_card_index..previous_card_index + app.config.no_of_cards_to_show as usize]
            .to_vec();
        let mut visible_boards_and_cards = app.visible_boards_and_cards.clone();
        // replace the cards of the current board
        visible_boards_and_cards
            .entry(current_board_id)
            .and_modify(|cards| {
                *cards = previous_cards
                    .iter()
                    .map(|card| card.id)
                    .collect::<Vec<u128>>()
            });
        app.visible_boards_and_cards = visible_boards_and_cards;
        app.state.current_card_id = Some(previous_card_id);
    } else {
        // we are not at the first card, we can go up
        let previous_card_id = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == current_board_id)
            .unwrap()
            .1
            .iter()
            .nth(current_card_index - 1)
            .unwrap_or(&0)
            .clone();
        // check if previous_card_id is 0
        if previous_card_id == 0 {
            debug!("Cannot go up: previous card not found");
            app.send_error_toast("Cannot go up: Something went wrong", None);
            return;
        } else {
            app.state.current_card_id = Some(previous_card_id);
        }
    }
}

pub fn go_down(app: &mut App) {
    let current_visible_boards = app.visible_boards_and_cards.clone();
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;
    // check if app.board is empty, if so, return
    if current_visible_boards.is_empty() {
        return;
    }
    let current_board_id = if current_board_id.is_none() {
        app.boards[0].id
    } else {
        current_board_id.unwrap()
    };
    let current_card_id = if current_card_id.is_none() {
        // get the first card of the current board
        let current_board = app.boards.iter().find(|board| board.id == current_board_id);
        if current_board.is_none() {
            debug!("Cannot go down: current board not found, trying to get the first board");
            // check if app.visible_boards_and_cards is empty, if so, return else select the first board and first card
            if current_visible_boards.is_empty() {
                debug!("Cannot go down: current board not found, tried to get the first board, but failed");
                app.send_error_toast("Cannot go down: Something went wrong", None);
                return;
            } else {
                app.state.current_board_id =
                    Some(current_visible_boards.keys().nth(0).unwrap().clone());
                app.state.current_card_id =
                    Some(current_visible_boards.values().nth(0).unwrap()[0]);
                return;
            }
        }
        let current_board = current_board.unwrap();
        if current_board.cards.is_empty() {
            debug!("Cannot go down: current board has no cards");
            app.send_error_toast("Cannot go down: Current board has no cards", None);
            return;
        }
        current_board.cards[0].id
    } else {
        current_card_id.unwrap()
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
    if current_card_index == app.config.no_of_cards_to_show as usize - 1 {
        let current_card_index_in_all_cards = app
            .boards
            .iter()
            .find(|board| board.id == current_board_id)
            .unwrap()
            .cards
            .iter()
            .position(|card| card.id == current_card_id);
        if current_card_index_in_all_cards.is_none() {
            debug!("Cannot go down: current card not found");
            app.send_error_toast("Cannot go down: Something went wrong", None);
            return;
        }
        let current_card_index_in_all_cards = current_card_index_in_all_cards.unwrap();
        if current_card_index_in_all_cards
            == app
                .boards
                .iter()
                .find(|board| board.id == current_board_id)
                .unwrap()
                .cards
                .len()
                - 1
        {
            // we are at the last card, we cannot go down
            app.send_error_toast("Cannot go down: Already at the last card", None);
            return;
        }
        // we are not at the last card, we can go down
        // get the next NO_OF_CARDS_PER_PAGE cards
        let next_card_index = current_card_index_in_all_cards + 1;
        let next_card_id = app
            .boards
            .iter()
            .find(|board| board.id == current_board_id)
            .unwrap()
            .cards[next_card_index]
            .id;
        let start_index = next_card_index - 1;
        let end_index = next_card_index - 1 + app.config.no_of_cards_to_show as usize;
        let end_index = if end_index
            > app
                .boards
                .iter()
                .find(|board| board.id == current_board_id)
                .unwrap()
                .cards
                .len()
        {
            app.boards
                .iter()
                .find(|board| board.id == current_board_id)
                .unwrap()
                .cards
                .len()
        } else {
            end_index
        };
        let next_card_ids = app
            .boards
            .iter()
            .find(|board| board.id == current_board_id)
            .unwrap()
            .cards[start_index..end_index]
            .iter()
            .map(|card| card.id)
            .collect::<Vec<u128>>();
        // if next_card_ids are less tha app.config.no_of_cards_to_show, then add cards before the start index till we have app.config.no_of_cards_to_show cards or we have reached index 0
        let next_card_ids = if next_card_ids.len() < app.config.no_of_cards_to_show as usize {
            let mut next_card_ids = next_card_ids.clone();
            let mut start_index = start_index;
            while next_card_ids.len() < app.config.no_of_cards_to_show as usize && start_index > 0 {
                start_index -= 1;
                next_card_ids.insert(
                    0,
                    app.boards
                        .iter()
                        .find(|board| board.id == current_board_id)
                        .unwrap()
                        .cards[start_index]
                        .id,
                );
            }
            next_card_ids
        } else {
            next_card_ids
        };
        // replace the cards of the current board
        app.visible_boards_and_cards
            .entry(current_board_id)
            .and_modify(|cards| *cards = next_card_ids);
        // set the current card id
        app.state.current_card_id = Some(next_card_id);
    } else {
        // we are not at the last card, we can go down
        let next_card_id = current_visible_boards
            .iter()
            .find(|(board_id, _)| **board_id == current_board_id)
            .unwrap()
            .1
            .iter()
            .nth(current_card_index + 1)
            .unwrap_or(&0)
            .clone();
        // check if next_card_id is not 0
        if next_card_id == 0 {
            return;
        } else {
            app.state.current_card_id = Some(next_card_id);
        }
    }
}

pub fn prepare_config_for_new_app(state: &mut AppState, theme: Theme) -> AppConfig {
    let get_config_status = get_config(false);
    if get_config_status.is_err() {
        let config_error_msg = get_config_status.unwrap_err();
        if config_error_msg.contains("Overlapped keybinds found") {
            error!("Keybinds overlap detected. Please check your config file and fix the keybinds. Using default keybinds for now.");
            state.toasts.push(ToastWidget::new(
                config_error_msg,
                Duration::from_secs(DEFAULT_TOAST_DURATION) * 3,
                ToastType::Error,
                theme.clone(),
            ));
            state.toasts.push(ToastWidget::new("Please check your config file and fix the keybinds. Using default keybinds for now.".to_owned(),
                Duration::from_secs(DEFAULT_TOAST_DURATION), ToastType::Warning, theme.clone()));
            let new_config = get_config(true);
            if new_config.is_err() {
                error!("Unable to fix keybinds. Please check your config file. Using default config for now.");
                state.toasts.push(ToastWidget::new(
                    new_config.unwrap_err(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION) * 3,
                    ToastType::Error,
                    theme.clone(),
                ));
                state.toasts.push(ToastWidget::new(
                    "Using default config for now.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION),
                    ToastType::Warning,
                    theme.clone(),
                ));
                AppConfig::default()
            } else {
                let mut unwrapped_new_config = new_config.unwrap();
                unwrapped_new_config.keybindings = KeyBindings::default();
                unwrapped_new_config
            }
        } else {
            state.toasts.push(ToastWidget::new(
                config_error_msg,
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Error,
                theme.clone(),
            ));
            state.toasts.push(ToastWidget::new(
                "Using default config for now.".to_owned(),
                Duration::from_secs(DEFAULT_TOAST_DURATION),
                ToastType::Info,
                theme.clone(),
            ));
            AppConfig::default()
        }
    } else {
        get_config_status.unwrap()
    }
}

pub async fn handle_user_input_mode(app: &mut App, key: Key) -> AppReturn {
    // append to current user input if key is not enter else change state to Initialized
    if key != Key::Enter && key != Key::Esc {
        if app.config.keybindings.toggle_command_palette.contains(&key) {
            app.state.app_status = AppStatus::Initialized;
            app.state.popup_mode = None;
        }
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            if key == Key::Up {
                app.command_palette_up();
                return AppReturn::Continue;
            } else if key == Key::Down {
                app.command_palette_down();
                return AppReturn::Continue;
            }
        }
        let mut current_key = key.to_string();
        if key == Key::Char(' ') {
            current_key = " ".to_string();
        } else if key == Key::Ctrl('n') {
            current_key = "\n".to_string();
        } else if key == Key::Tab {
            current_key = "  ".to_string();
        } else if key == Key::Backspace {
            match app.state.ui_mode {
                UiMode::NewBoard => match app.state.focus {
                    Focus::NewBoardName => {
                        if app.state.current_cursor_position.is_some() {
                            let current_cursor_position =
                                app.state.current_cursor_position.unwrap();
                            if current_cursor_position > 0 {
                                app.state.new_board_form[0].remove(current_cursor_position - 1);
                                app.state.current_cursor_position =
                                    Some(current_cursor_position - 1);
                            }
                        } else {
                            app.state.new_board_form[0].pop();
                        }
                    }
                    Focus::NewBoardDescription => {
                        if app.state.current_cursor_position.is_some() {
                            let current_cursor_position =
                                app.state.current_cursor_position.unwrap();
                            if current_cursor_position > 0 {
                                app.state.new_board_form[1].remove(current_cursor_position - 1);
                                app.state.current_cursor_position =
                                    Some(current_cursor_position - 1);
                            }
                        } else {
                            app.state.new_board_form[1].pop();
                        }
                    }
                    _ => {}
                },
                UiMode::NewCard => match app.state.focus {
                    Focus::NewCardName => {
                        if app.state.current_cursor_position.is_some() {
                            let current_cursor_position =
                                app.state.current_cursor_position.unwrap();
                            if current_cursor_position > 0 {
                                app.state.new_card_form[0].remove(current_cursor_position - 1);
                                app.state.current_cursor_position =
                                    Some(current_cursor_position - 1);
                            }
                        } else {
                            app.state.new_card_form[0].pop();
                        }
                    }
                    Focus::NewCardDescription => {
                        if app.state.current_cursor_position.is_some() {
                            let current_cursor_position =
                                app.state.current_cursor_position.unwrap();
                            if current_cursor_position > 0 {
                                app.state.new_card_form[1].remove(current_cursor_position - 1);
                                app.state.current_cursor_position =
                                    Some(current_cursor_position - 1);
                            }
                        } else {
                            app.state.new_card_form[1].pop();
                        }
                    }
                    Focus::NewCardDueDate => {
                        if app.state.current_cursor_position.is_some() {
                            let current_cursor_position =
                                app.state.current_cursor_position.unwrap();
                            if current_cursor_position > 0 {
                                app.state.new_card_form[2].remove(current_cursor_position - 1);
                                app.state.current_cursor_position =
                                    Some(current_cursor_position - 1);
                            }
                        } else {
                            app.state.new_card_form[2].pop();
                        }
                    }
                    _ => {}
                },
                _ => {
                    if app.state.current_cursor_position.is_some() {
                        let current_cursor_position = app.state.current_cursor_position.unwrap();
                        if current_cursor_position > 0 {
                            app.state
                                .current_user_input
                                .remove(current_cursor_position - 1);
                            app.state.current_cursor_position = Some(current_cursor_position - 1);
                        }
                    } else {
                        app.state.current_user_input.pop();
                    }
                }
            };
            current_key = "".to_string();
        } else if key == Key::Left {
            match app.state.ui_mode {
                UiMode::NewBoard => match app.state.focus {
                    Focus::NewBoardName => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_board_form[0].len());
                        } else if app.state.current_cursor_position.unwrap() > 0 {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() - 1);
                        } else {
                            app.state.current_cursor_position = Some(0);
                        }
                    }
                    Focus::NewBoardDescription => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_board_form[1].len());
                        } else if app.state.current_cursor_position.unwrap() > 0 {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() - 1);
                        } else {
                            app.state.current_cursor_position = Some(0);
                        }
                    }
                    _ => {}
                },
                UiMode::NewCard => match app.state.focus {
                    Focus::NewCardName => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[0].len());
                        } else if app.state.current_cursor_position.unwrap() > 0 {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() - 1);
                        } else {
                            app.state.current_cursor_position = Some(0);
                        }
                    }
                    Focus::NewCardDescription => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[1].len());
                        } else if app.state.current_cursor_position.unwrap() > 0 {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() - 1);
                        } else {
                            app.state.current_cursor_position = Some(0);
                        }
                    }
                    Focus::NewCardDueDate => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[2].len());
                        } else if app.state.current_cursor_position.unwrap() > 0 {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() - 1);
                        } else {
                            app.state.current_cursor_position = Some(0);
                        }
                    }
                    _ => {}
                },
                _ => {
                    if app.state.current_cursor_position.is_none() {
                        app.state.current_cursor_position =
                            Some(app.state.current_user_input.len());
                    } else if app.state.current_cursor_position.unwrap() > 0 {
                        app.state.current_cursor_position =
                            Some(app.state.current_cursor_position.unwrap() - 1);
                    } else {
                        app.state.current_cursor_position = Some(0);
                    }
                }
            };
            current_key = "".to_string();
        } else if key == Key::Right {
            match app.state.ui_mode {
                UiMode::NewBoard => match app.state.focus {
                    Focus::NewBoardName => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_board_form[0].len());
                        } else if app.state.current_cursor_position.unwrap()
                            < app.state.new_board_form[0].len()
                        {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() + 1);
                        } else {
                            app.state.current_cursor_position =
                                Some(app.state.new_board_form[0].len());
                        }
                    }
                    Focus::NewBoardDescription => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_board_form[1].len());
                        } else if app.state.current_cursor_position.unwrap()
                            < app.state.new_board_form[1].len()
                        {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() + 1);
                        } else {
                            app.state.current_cursor_position =
                                Some(app.state.new_board_form[1].len());
                        }
                    }
                    _ => {}
                },
                UiMode::NewCard => match app.state.focus {
                    Focus::NewCardName => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[0].len());
                        } else if app.state.current_cursor_position.unwrap()
                            < app.state.new_card_form[0].len()
                        {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() + 1);
                        } else {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[0].len());
                        }
                    }
                    Focus::NewCardDescription => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[1].len());
                        } else if app.state.current_cursor_position.unwrap()
                            < app.state.new_card_form[1].len()
                        {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() + 1);
                        } else {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[1].len());
                        }
                    }
                    Focus::NewCardDueDate => {
                        if app.state.current_cursor_position.is_none() {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[2].len());
                        } else if app.state.current_cursor_position.unwrap()
                            < app.state.new_card_form[2].len()
                        {
                            app.state.current_cursor_position =
                                Some(app.state.current_cursor_position.unwrap() + 1);
                        } else {
                            app.state.current_cursor_position =
                                Some(app.state.new_card_form[2].len());
                        }
                    }
                    _ => {}
                },
                _ => {
                    if app.state.current_cursor_position.is_none() {
                        app.state.current_cursor_position =
                            Some(app.state.current_user_input.len());
                    } else if app.state.current_cursor_position.unwrap()
                        < app.state.current_user_input.len()
                    {
                        app.state.current_cursor_position =
                            Some(app.state.current_cursor_position.unwrap() + 1);
                    } else {
                        app.state.current_cursor_position =
                            Some(app.state.current_user_input.len());
                    }
                }
            };
            current_key = "".to_string();
        } else if key == Key::Home {
            match app.state.ui_mode {
                UiMode::NewBoard => match app.state.focus {
                    Focus::NewBoardName => {
                        app.state.current_cursor_position = Some(0);
                    }
                    Focus::NewBoardDescription => {
                        app.state.current_cursor_position = Some(0);
                    }
                    _ => {}
                },
                UiMode::NewCard => match app.state.focus {
                    Focus::NewCardName => {
                        app.state.current_cursor_position = Some(0);
                    }
                    Focus::NewCardDescription => {
                        app.state.current_cursor_position = Some(0);
                    }
                    Focus::NewCardDueDate => {
                        app.state.current_cursor_position = Some(0);
                    }
                    _ => {}
                },
                _ => {
                    app.state.current_cursor_position = Some(0);
                }
            };
            current_key = "".to_string();
        } else if key == Key::End {
            match app.state.ui_mode {
                UiMode::NewBoard => match app.state.focus {
                    Focus::NewBoardName => {
                        app.state.current_cursor_position = Some(app.state.new_board_form[0].len());
                    }
                    Focus::NewBoardDescription => {
                        app.state.current_cursor_position = Some(app.state.new_board_form[1].len());
                    }
                    _ => {}
                },
                UiMode::NewCard => match app.state.focus {
                    Focus::NewCardName => {
                        app.state.current_cursor_position = Some(app.state.new_card_form[0].len());
                    }
                    Focus::NewCardDescription => {
                        app.state.current_cursor_position = Some(app.state.new_card_form[1].len());
                    }
                    Focus::NewCardDueDate => {
                        app.state.current_cursor_position = Some(app.state.new_card_form[2].len());
                    }
                    _ => {}
                },
                _ => {
                    app.state.current_cursor_position = Some(app.state.current_user_input.len());
                }
            };
            current_key = "".to_string();
        } else if current_key.starts_with("<") && current_key.ends_with(">") {
            current_key = current_key[1..current_key.len() - 1].to_string();
        }
        if current_key == "" {
            return AppReturn::Continue;
        }
        if app.state.focus == Focus::NewBoardName {
            let cursor_position = app.state.current_cursor_position.unwrap_or(0);
            app.state.new_board_form[0]
                .insert(cursor_position, current_key.chars().next().unwrap());
            app.state.current_cursor_position = Some(cursor_position + 1);
        } else if app.state.focus == Focus::NewBoardDescription {
            let cursor_position = app.state.current_cursor_position.unwrap_or(0);
            app.state.new_board_form[1]
                .insert(cursor_position, current_key.chars().next().unwrap());
            app.state.current_cursor_position = Some(cursor_position + 1);
        } else if app.state.focus == Focus::NewCardName {
            let cursor_position = app.state.current_cursor_position.unwrap_or(0);
            app.state.new_card_form[0].insert(cursor_position, current_key.chars().next().unwrap());
            app.state.current_cursor_position = Some(cursor_position + 1);
        } else if app.state.focus == Focus::NewCardDescription {
            let current_cursor_position = app.state.current_cursor_position.unwrap_or(0);
            app.state.new_card_form[1]
                .insert(current_cursor_position, current_key.chars().next().unwrap());
            app.state.current_cursor_position = Some(current_cursor_position + 1);
        } else if app.state.focus == Focus::NewCardDueDate {
            let current_cursor_position = app.state.current_cursor_position.unwrap_or(0);
            app.state.new_card_form[2]
                .insert(current_cursor_position, current_key.chars().next().unwrap());
            app.state.current_cursor_position = Some(current_cursor_position + 1);
        } else {
            let current_cursor_position = app.state.current_cursor_position.unwrap_or(0);
            app.state
                .current_user_input
                .insert(current_cursor_position, current_key.chars().next().unwrap());
            app.state.current_cursor_position = Some(current_cursor_position + 1);
        }
    } else if key == Key::Esc {
        if app.state.focus == Focus::NewBoardName {
            app.state.new_board_form[0] = "".to_string();
        } else if app.state.focus == Focus::NewBoardDescription {
            app.state.new_board_form[1] = "".to_string();
        } else if app.state.focus == Focus::NewCardName {
            app.state.new_card_form[0] = "".to_string();
        } else if app.state.focus == Focus::NewCardDescription {
            app.state.new_card_form[1] = "".to_string();
        } else if app.state.focus == Focus::NewCardDueDate {
            app.state.new_card_form[2] = "".to_string();
        } else {
            app.state.current_user_input = "".to_string();
        }
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            app.state.popup_mode = None;
        }
        app.state.app_status = AppStatus::Initialized;
        app.state.current_cursor_position = None;
        info!("Exiting user input mode");
    } else {
        if key == Key::Enter
            && app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            return CommandPalette::handle_command(app).await;
        }
        app.state.app_status = AppStatus::Initialized;
        app.state.current_cursor_position = None;
        info!("Exiting user input mode");
    }
    AppReturn::Continue
}

pub async fn handle_keybind_mode(app: &mut App, key: Key) -> AppReturn {
    if key != Key::Enter && key != Key::Esc {
        if app.state.edited_keybinding.is_some() {
            let keybinding = app.state.edited_keybinding.as_mut().unwrap();
            keybinding.push(key);
        } else {
            app.state.edited_keybinding = Some(vec![key]);
        }
    } else if key == Key::Enter {
        app.state.app_status = AppStatus::Initialized;
        info!("Exiting user keybind input mode");
    } else if key == Key::Esc {
        app.state.app_status = AppStatus::Initialized;
        app.state.edited_keybinding = None;
        info!("Exiting user keybind input mode");
    }
    AppReturn::Continue
}

pub async fn handle_general_actions(app: &mut App, key: Key) -> AppReturn {
    if let Some(action) = app.actions.find(key) {
        // check if the current focus is in the available focus list for the current ui mode if not assign it to the first
        if app.state.popup_mode.is_some() {
            if PopupMode::get_available_targets(&app.state.popup_mode.unwrap())
                .iter()
                .find(|x| *x == &app.state.focus)
                .is_none()
            {
                let available_targets =
                    PopupMode::get_available_targets(&app.state.popup_mode.unwrap());
                if available_targets.len() > 0 {
                    app.state.focus = available_targets[0];
                }
            }
        } else {
            if UiMode::get_available_targets(&app.state.ui_mode)
                .iter()
                .find(|x| *x == &app.state.focus)
                .is_none()
            {
                let available_targets = UiMode::get_available_targets(&app.state.ui_mode);
                if available_targets.len() > 0 {
                    app.state.focus = available_targets[0];
                }
            }
        }
        match action {
            Action::Quit => {
                let get_config_status = get_config(false);
                let config = if get_config_status.is_err() {
                    debug!("Error getting config: {}", get_config_status.unwrap_err());
                    AppConfig::default()
                } else {
                    get_config_status.unwrap()
                };
                if config.save_on_exit {
                    app.dispatch(IoEvent::AutoSave).await;
                }
                return AppReturn::Exit;
            }
            Action::NextFocus => {
                handle_next_focus(app);
                AppReturn::Continue
            }
            Action::PrvFocus => {
                handle_prv_focus(app);
                AppReturn::Continue
            }
            Action::ResetUI => {
                let new_ui_mode = app.config.default_view.clone();
                let available_focus_targets = UiMode::get_available_targets(&new_ui_mode);
                // check if focus is still available in the new ui_mode if not set it to the first available tab
                if !available_focus_targets.contains(&app.state.focus) {
                    // check if available focus targets is empty
                    if available_focus_targets.is_empty() {
                        app.state.focus = Focus::NoFocus;
                    } else {
                        app.state.focus = available_focus_targets[0];
                    }
                }
                let default_theme = app.config.default_theme.clone();
                for theme in app.all_themes.iter_mut() {
                    if theme.name == default_theme {
                        app.theme = theme.clone();
                    }
                }
                app.state.toasts = vec![];
                app.send_info_toast("UI reset, all toasts cleared", None);
                app.state.ui_mode = new_ui_mode;
                app.state.popup_mode = None;
                app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
                AppReturn::Continue
            }
            Action::OpenConfigMenu => {
                match app.state.ui_mode {
                    UiMode::ConfigMenu => {
                        // check if the prv ui mode is the same as the current ui mode
                        if app.state.prev_ui_mode.is_some()
                            && app.state.prev_ui_mode.as_ref().unwrap() == &UiMode::ConfigMenu
                        {
                            app.state.ui_mode = app.config.default_view.clone();
                        } else {
                            app.state.ui_mode = app
                                .state
                                .prev_ui_mode
                                .as_ref()
                                .unwrap_or_else(|| &app.config.default_view)
                                .clone();
                        }
                    }
                    _ => {
                        app.state.prev_ui_mode = Some(app.state.ui_mode.clone());
                        app.state.ui_mode = UiMode::ConfigMenu;
                        if app.state.config_state.selected().is_none() {
                            app.config_next()
                        }
                        let available_focus_targets = app.state.ui_mode.get_available_targets();
                        if !available_focus_targets.contains(&app.state.focus) {
                            // check if available focus targets is empty
                            if available_focus_targets.is_empty() {
                                app.state.focus = Focus::NoFocus;
                            } else {
                                app.state.focus = available_focus_targets[0];
                            }
                        }
                    }
                }
                if app.state.popup_mode.is_some() {
                    app.state.popup_mode = None;
                }
                AppReturn::Continue
            }
            Action::Up => {
                reset_mouse(app);
                if app.state.popup_mode.is_some() {
                    let popup_mode = app.state.popup_mode.as_ref().unwrap();
                    match popup_mode {
                        PopupMode::ChangeUIMode => {
                            app.select_default_view_prev();
                        }
                        PopupMode::ChangeCurrentCardStatus => {
                            app.select_current_card_status_prev();
                        }
                        PopupMode::SelectDefaultView => {
                            app.select_default_view_prev();
                        }
                        PopupMode::ChangeTheme => {
                            app.select_change_theme_prev();
                        }
                        PopupMode::ThemeEditor => {
                            if app.state.focus == Focus::StyleEditorFG {
                                app.select_edit_style_fg_prev();
                            } else if app.state.focus == Focus::StyleEditorBG {
                                app.select_edit_style_bg_prev();
                            } else if app.state.focus == Focus::StyleEditorModifier {
                                app.select_edit_style_modifier_prev();
                            }
                        }
                        _ => {}
                    }
                    return AppReturn::Continue;
                }
                match app.state.ui_mode {
                    UiMode::ConfigMenu => {
                        if app.state.focus == Focus::ConfigTable {
                            app.config_previous();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prev_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::MainMenu => {
                        if app.state.focus == Focus::MainMenu {
                            app.main_menu_previous();
                        } else if app.state.focus == Focus::MainMenuHelp {
                            app.help_prev();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prev_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::LoadSave => {
                        app.load_save_previous();
                        app.dispatch(IoEvent::LoadPreview).await;
                    }
                    UiMode::EditKeybindings => {
                        app.edit_keybindings_prev();
                    }
                    UiMode::CreateTheme => {
                        if app.state.focus == Focus::ThemeEditor {
                            app.select_create_theme_prev();
                        } else if app.state.focus == Focus::SubmitButton {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prev_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Submit Button with {} or {}, to create the theme",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    _ => {
                        if app.state.focus == Focus::Body {
                            go_up(app);
                        } else if app.state.focus == Focus::Help {
                            app.help_prev();
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
                        PopupMode::ChangeUIMode => {
                            app.select_default_view_next();
                        }
                        PopupMode::ChangeCurrentCardStatus => {
                            app.select_current_card_status_next();
                        }
                        PopupMode::SelectDefaultView => {
                            app.select_default_view_next();
                        }
                        PopupMode::ChangeTheme => {
                            app.select_change_theme_next();
                        }
                        PopupMode::ThemeEditor => {
                            if app.state.focus == Focus::StyleEditorFG {
                                app.select_edit_style_fg_next();
                            } else if app.state.focus == Focus::StyleEditorBG {
                                app.select_edit_style_bg_next();
                            } else if app.state.focus == Focus::StyleEditorModifier {
                                app.select_edit_style_modifier_next();
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
                                .get(0)
                                .unwrap_or_else(|| &Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prev_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Config Menu with {} or {}, to select a config option using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::MainMenu => {
                        if app.state.focus == Focus::MainMenu {
                            app.main_menu_next();
                        } else if app.state.focus == Focus::MainMenuHelp {
                            app.help_next();
                        } else {
                            let next_focus_key = app
                                .config
                                .keybindings
                                .next_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prev_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Main Menu with {} or {}, to navigate the menu using the arrow keys",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    UiMode::LoadSave => {
                        app.load_save_next();
                        app.dispatch(IoEvent::LoadPreview).await;
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
                                .get(0)
                                .unwrap_or_else(|| &Key::Tab);
                            let prev_focus_key = app
                                .config
                                .keybindings
                                .prev_focus
                                .get(0)
                                .unwrap_or_else(|| &Key::BackTab);
                            app.send_warning_toast(&format!(
                                "Move Focus to the Submit Button with {} or {}, to create the theme",
                                next_focus_key, prev_focus_key), None);
                        }
                    }
                    _ => {
                        if app.state.focus == Focus::Body {
                            go_down(app);
                        } else if app.state.focus == Focus::Help {
                            app.help_next();
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
                            if app.state.popup_mode.unwrap() == PopupMode::EditGeneralConfig
                                || app.state.popup_mode.unwrap() == PopupMode::CustomRGBPromptFG
                                || app.state.popup_mode.unwrap() == PopupMode::CustomRGBPromptBG
                            {
                                app.state.app_status = AppStatus::UserInput;
                                info!("Taking user input");
                            } else if app.state.popup_mode.unwrap()
                                == PopupMode::EditSpecificKeyBinding
                            {
                                app.state.app_status = AppStatus::KeyBindMode;
                                info!("Taking user keybind input");
                            }
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::GoToPreviousUIMode => handle_go_to_previous_ui_mode(app),
            Action::Enter => {
                if app.state.popup_mode.is_some() {
                    let popup_mode = app.state.popup_mode.as_ref().unwrap();
                    match popup_mode {
                        PopupMode::ChangeUIMode => handle_change_ui_mode(app),
                        PopupMode::ChangeCurrentCardStatus => {
                            return handle_change_current_card_status(app);
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
                        PopupMode::ChangeTheme => {
                            return handle_change_theme(app, app.state.defualt_theme_mode)
                        }
                        PopupMode::ThemeEditor => return handle_create_theme_action(app),
                        PopupMode::SaveThemePrompt => handle_save_theme_prompt(app),
                        PopupMode::CustomRGBPromptFG => handle_custom_rgb_prompt(app, true),
                        PopupMode::CustomRGBPromptBG => handle_custom_rgb_prompt(app, false),
                        PopupMode::CardView => {}
                        PopupMode::CommandPalette => {}
                    }
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                match app.state.ui_mode {
                    UiMode::ConfigMenu => handle_config_menu_action(app),
                    UiMode::MainMenu => match app.state.focus {
                        Focus::MainMenu => handle_main_menu_action(app),
                        Focus::MainMenuHelp => {
                            app.state.ui_mode = UiMode::HelpMenu;
                            AppReturn::Continue
                        }
                        Focus::Log => {
                            app.state.ui_mode = UiMode::LogsOnly;
                            AppReturn::Continue
                        }
                        _ => AppReturn::Continue,
                    },
                    UiMode::NewBoard => {
                        handle_new_board_action(app);
                        AppReturn::Continue
                    }
                    UiMode::NewCard => handle_new_card_action(app),
                    UiMode::LoadSave => {
                        app.dispatch(IoEvent::LoadSave).await;
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
                    _ => {
                        match app.state.focus {
                            Focus::Help => {
                                app.state.prev_ui_mode = Some(app.state.ui_mode.clone());
                                app.state.ui_mode = UiMode::HelpMenu;
                            }
                            Focus::Log => {
                                app.state.prev_ui_mode = Some(app.state.ui_mode.clone());
                                app.state.ui_mode = UiMode::LogsOnly;
                            }
                            _ => {}
                        }
                        if UiMode::view_modes().contains(&app.state.ui_mode)
                            && app.state.focus == Focus::Body
                        {
                            // check if there is a current card
                            if let Some(current_card_id) = app.state.current_card_id {
                                if let Some(current_board_id) = app.state.current_board_id {
                                    // check if the current card is in the current board
                                    let current_board = app
                                        .boards
                                        .iter()
                                        .find(|board| board.id == current_board_id);
                                    if let Some(current_board) = current_board {
                                        let current_card = current_board
                                            .cards
                                            .iter()
                                            .find(|card| card.id == current_card_id);
                                        if let Some(_) = current_card {
                                            app.state.popup_mode = Some(PopupMode::CardView);
                                        } else {
                                            // if the current card is not in the current board then set the current card to None
                                            app.state.current_card_id = None;
                                        }
                                    } else {
                                        // if the current board is not found then set the current card to None
                                        app.state.current_card_id = None;
                                    }
                                } else {
                                    // if the current board is not found then set the current card to None
                                    app.state.current_card_id = None;
                                }
                            }
                        }
                        AppReturn::Continue
                    }
                }
            }
            Action::HideUiElement => {
                let current_focus = Focus::from_str(app.state.focus.to_str());
                let current_ui_mode = app.state.ui_mode.clone();
                // hide the current focus by switching to a view where it is not available
                // for example if current uimode is Title and focus is on Title then switch to Zen
                if current_ui_mode == UiMode::Zen {
                    app.state.ui_mode = UiMode::MainMenu;
                    if app.state.main_menu_state.selected().is_none() {
                        app.main_menu_next();
                    }
                } else if current_ui_mode == UiMode::TitleBody {
                    if current_focus == Focus::Title {
                        app.state.ui_mode = UiMode::Zen;
                        app.state.focus = Focus::Body;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::BodyHelp {
                    if current_focus == Focus::Help {
                        app.state.ui_mode = UiMode::Zen;
                        app.state.focus = Focus::Body;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::BodyLog {
                    if current_focus == Focus::Log {
                        app.state.ui_mode = UiMode::Zen;
                        app.state.focus = Focus::Body;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::TitleBodyHelp {
                    if current_focus == Focus::Title {
                        app.state.ui_mode = UiMode::BodyHelp;
                        app.state.focus = Focus::Body;
                    } else if current_focus == Focus::Help {
                        app.state.ui_mode = UiMode::TitleBody;
                        app.state.focus = Focus::Title;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::TitleBodyLog {
                    if current_focus == Focus::Title {
                        app.state.ui_mode = UiMode::BodyLog;
                        app.state.focus = Focus::Body;
                    } else if current_focus == Focus::Log {
                        app.state.ui_mode = UiMode::TitleBody;
                        app.state.focus = Focus::Title;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::TitleBodyHelpLog {
                    if current_focus == Focus::Title {
                        app.state.ui_mode = UiMode::BodyHelpLog;
                        app.state.focus = Focus::Body;
                    } else if current_focus == Focus::Help {
                        app.state.ui_mode = UiMode::TitleBodyLog;
                        app.state.focus = Focus::Title;
                    } else if current_focus == Focus::Log {
                        app.state.ui_mode = UiMode::TitleBodyHelp;
                        app.state.focus = Focus::Title;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                } else if current_ui_mode == UiMode::BodyHelpLog {
                    if current_focus == Focus::Help {
                        app.state.ui_mode = UiMode::BodyLog;
                        app.state.focus = Focus::Body;
                    } else if current_focus == Focus::Log {
                        app.state.ui_mode = UiMode::BodyHelp;
                        app.state.focus = Focus::Body;
                    } else {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next();
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::SaveState => {
                app.dispatch(IoEvent::SaveLocalData).await;
                AppReturn::Continue
            }
            Action::NewBoard => {
                // check if current ui_mode is in UiMode::view_modes()
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    app.state.new_board_form = NEW_BOARD_FORM_DEFAULT_STATE
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    app.set_ui_mode(UiMode::NewBoard);
                    app.state.previous_focus = Some(app.state.focus.clone());
                }
                AppReturn::Continue
            }
            Action::NewCard => {
                if UiMode::view_modes().contains(&app.state.ui_mode) {
                    // check if current board is not empty
                    if app.state.current_board_id.is_none() {
                        warn!("No board available to add card to");
                        app.send_warning_toast("No board available to add card to", None);
                        return AppReturn::Continue;
                    }
                    app.state.new_card_form = NEW_CARD_FORM_DEFAULT_STATE
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    app.set_ui_mode(UiMode::NewCard);
                    app.state.previous_focus = Some(app.state.focus.clone());
                }
                AppReturn::Continue
            }
            Action::DeleteCard => {
                match app.state.ui_mode {
                    UiMode::LoadSave => {
                        // run delete task in background
                        app.dispatch(IoEvent::DeleteSave).await;
                        tokio::time::sleep(Duration::from_millis(IO_EVENT_WAIT_TIME)).await;
                        app.dispatch(IoEvent::LoadPreview).await;
                        AppReturn::Continue
                    }
                    _ => {
                        if !UiMode::view_modes().contains(&app.state.ui_mode) {
                            return AppReturn::Continue;
                        }
                        match app.state.focus {
                            Focus::Body => {
                                // delete the current card
                                if let Some(current_board) = app.state.current_board_id {
                                    // find index of current board id in app.boards
                                    let index = app
                                        .boards
                                        .iter()
                                        .position(|board| board.id == current_board);
                                    if let Some(current_card) = app.state.current_card_id {
                                        let card_index = app.boards[index.unwrap()]
                                            .cards
                                            .iter()
                                            .position(|card| card.id == current_card);
                                        if let Some(card_index) = card_index {
                                            let card_name = app.boards[index.unwrap()].cards
                                                [card_index]
                                                .name
                                                .clone();
                                            app.boards[index.unwrap()].cards.remove(card_index);
                                            // if index is > 0, set current card to previous card, else set to next card, else set to None
                                            if card_index > 0 {
                                                app.state.current_card_id = Some(
                                                    app.boards[index.unwrap()].cards
                                                        [card_index - 1]
                                                        .id,
                                                );
                                            } else if app.boards[index.unwrap()].cards.len() > 0 {
                                                app.state.current_card_id =
                                                    Some(app.boards[index.unwrap()].cards[0].id);
                                            } else {
                                                app.state.current_card_id = None;
                                            }
                                            warn!("Deleted card {}", card_name);
                                            app.send_warning_toast(
                                                &format!("Deleted card {}", card_name),
                                                None,
                                            );
                                            // remove card_id from app.visible_boards_and_cards if it is there, where visible_boards_and_cards is a LinkedHashMap of board_id to a vector of card_ids
                                            if let Some(visible_cards) =
                                                app.visible_boards_and_cards.get_mut(&current_board)
                                            {
                                                if let Some(card_index) = visible_cards
                                                    .iter()
                                                    .position(|card_id| *card_id == current_card)
                                                {
                                                    visible_cards.remove(card_index);
                                                }
                                            }
                                            app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
                                        }
                                    } else if let Some(current_board) = app.state.current_board_id {
                                        // find index of current board id in app.boards
                                        let index = app
                                            .boards
                                            .iter()
                                            .position(|board| board.id == current_board);
                                        if let Some(index) = index {
                                            let board_name = app.boards[index].name.clone();
                                            app.boards.remove(index);
                                            // if index is > 0, set current board to previous board, else set to next board, else set to None
                                            if index > 0 {
                                                app.state.current_board_id =
                                                    Some(app.boards[index - 1].id);
                                            } else if app.boards.len() > 0 {
                                                app.state.current_board_id = Some(app.boards[0].id);
                                            } else {
                                                app.state.current_board_id = None;
                                            }
                                            warn!("Deleted board {}", board_name);
                                            app.send_warning_toast(
                                                &format!("Deleted board {}", board_name),
                                                None,
                                            );
                                            // remove board_id from app.visible_boards_and_cards if it is there
                                            app.visible_boards_and_cards.remove(&current_board);
                                            app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
                                        }
                                    }
                                }
                                AppReturn::Continue
                            }
                            _ => AppReturn::Continue,
                        }
                    }
                }
            }
            Action::DeleteBoard => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        // delete the current board from app.boards
                        if let Some(current_board) = app.state.current_board_id.clone() {
                            // find index of current board id in app.boards
                            let index = app
                                .boards
                                .iter()
                                .position(|board| board.id == current_board);
                            if let Some(index) = index {
                                let board_name = app.boards[index].name.clone();
                                app.boards.remove(index);
                                // if index is > 0, set current board to previous board, else set to next board, else set to None
                                if index > 0 {
                                    app.state.current_board_id =
                                        Some(app.boards[index - 1].id.clone());
                                } else if index < app.boards.len() {
                                    app.state.current_board_id = Some(app.boards[index].id.clone());
                                } else {
                                    app.state.current_board_id = None;
                                }
                                app.visible_boards_and_cards.remove(&current_board);
                                warn!("Deleted board: {}", board_name);
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
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                // check if focus is on body
                if app.state.focus != Focus::Body {
                    return AppReturn::Continue;
                }
                // get the current card and change its status to complete
                if let Some(current_board) = app.state.current_board_id {
                    // find index of current board id in app.boards
                    let index = app
                        .boards
                        .iter()
                        .position(|board| board.id == current_board);
                    if let Some(current_card) = app.state.current_card_id {
                        let card_index = app.boards[index.unwrap()]
                            .cards
                            .iter()
                            .position(|card| card.id == current_card);
                        if let Some(card_index) = card_index {
                            app.boards[index.unwrap()].cards[card_index].card_status =
                                CardStatus::Complete;
                            info!(
                                "Changed status to Completed for card {}",
                                app.boards[index.unwrap()].cards[card_index].name
                            );
                            app.send_info_toast(
                                &format!(
                                    "Changed status to Completed for card {}",
                                    app.boards[index.unwrap()].cards[card_index].name
                                ),
                                None,
                            );
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::ChangeCardStatusToActive => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                // check if focus is on body
                if app.state.focus != Focus::Body {
                    return AppReturn::Continue;
                }
                // get the current card and change its status to active
                if let Some(current_board) = app.state.current_board_id {
                    // find index of current board id in app.boards
                    let index = app
                        .boards
                        .iter()
                        .position(|board| board.id == current_board);
                    if let Some(current_card) = app.state.current_card_id {
                        let card_index = app.boards[index.unwrap()]
                            .cards
                            .iter()
                            .position(|card| card.id == current_card);
                        if let Some(card_index) = card_index {
                            app.boards[index.unwrap()].cards[card_index].card_status =
                                CardStatus::Active;
                            info!(
                                "Changed status to Active for card {}",
                                app.boards[index.unwrap()].cards[card_index].name
                            );
                            app.send_info_toast(
                                &format!(
                                    "Changed status to Active for card {}",
                                    app.boards[index.unwrap()].cards[card_index].name
                                ),
                                None,
                            );
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::ChangeCardStatusToStale => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                // check if focus is on body
                if app.state.focus != Focus::Body {
                    return AppReturn::Continue;
                }
                // get the current card and change its status to stale
                if let Some(current_board) = app.state.current_board_id {
                    // find index of current board id in app.boards
                    let index = app
                        .boards
                        .iter()
                        .position(|board| board.id == current_board);
                    if let Some(current_card) = app.state.current_card_id {
                        let card_index = app.boards[index.unwrap()]
                            .cards
                            .iter()
                            .position(|card| card.id == current_card);
                        if let Some(card_index) = card_index {
                            app.boards[index.unwrap()].cards[card_index].card_status =
                                CardStatus::Stale;
                            info!(
                                "Changed status to Stale for card {}",
                                app.boards[index.unwrap()].cards[card_index].name
                            );
                            app.send_info_toast(
                                &format!(
                                    "Changed status to Stale for card {}",
                                    app.boards[index.unwrap()].cards[card_index].name
                                ),
                                None,
                            );
                        }
                    }
                }
                AppReturn::Continue
            }
            Action::GoToMainMenu => {
                app.state.current_board_id = None;
                app.state.current_card_id = None;
                app.state.focus = Focus::MainMenu;
                app.state.ui_mode = UiMode::MainMenu;
                if app.state.main_menu_state.selected().is_none() {
                    app.state.main_menu_state.select(Some(0));
                }
                AppReturn::Continue
            }
            Action::MoveCardUp => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        if app.state.current_card_id.is_none() {
                            return AppReturn::Continue;
                        } else {
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
                            let current_board_index_in_all_boards = app
                                .boards
                                .iter()
                                .position(|board| board.id == current_board_id);
                            if current_board_index_in_all_boards.is_none() {
                                debug!("Cannot move card up without a current board index");
                                return AppReturn::Continue;
                            }
                            let current_card_index_in_all = app.boards
                                [current_board_index_in_all_boards.unwrap()]
                            .cards
                            .iter()
                            .position(|card| card.id == current_card_id);
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
                                error!(
                                    "Cannot move card up, it is already at the top of the board"
                                );
                                return AppReturn::Continue;
                            }
                            // update visible boards and cards
                            // check if both the cards that are being swapped are in the visible cards
                            let current_card_index_in_visible = app.visible_boards_and_cards
                                [&current_board_id]
                                .iter()
                                .position(|card_id| *card_id == current_card_id);
                            if current_card_index_in_visible.is_none() {
                                debug!("Cannot move card up without a current card index in visible cards");
                                return AppReturn::Continue;
                            }
                            let current_card_index_in_visible =
                                current_card_index_in_visible.unwrap();
                            if current_card_index_in_visible == 0 {
                                let card_above_id = app.boards
                                    [current_board_index_in_all_boards.unwrap()]
                                .cards[current_card_index_in_all - 1]
                                    .id;
                                let mut visible_cards: Vec<u128> = vec![];
                                visible_cards.push(current_card_id);
                                visible_cards.push(card_above_id);

                                for card in app.visible_boards_and_cards[&current_board_id].iter() {
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
                            app.boards[current_board_index_in_all_boards.unwrap()]
                                .cards
                                .swap(current_card_index_in_all, current_card_index_in_all - 1);
                        }
                    }
                    _ => {}
                }
                AppReturn::Continue
            }
            Action::MoveCardDown => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        if app.state.current_card_id.is_none() {
                            return AppReturn::Continue;
                        } else {
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
                            let current_board_index_in_all_boards = app
                                .boards
                                .iter()
                                .position(|board| board.id == current_board_id);
                            if current_board_index_in_all_boards.is_none() {
                                debug!("Cannot move card down without a current board index");
                                return AppReturn::Continue;
                            }
                            let current_card_index_in_all = app.boards
                                [current_board_index_in_all_boards.unwrap()]
                            .cards
                            .iter()
                            .position(|card| card.id == current_card_id);
                            if current_card_index_in_all.is_none() {
                                debug!("Cannot move card down without a current card index");
                                return AppReturn::Continue;
                            }
                            let current_card_index_in_all = current_card_index_in_all.unwrap();
                            if current_card_index_in_all
                                == app.boards[current_board_index_in_all_boards.unwrap()]
                                    .cards
                                    .len()
                                    - 1
                            {
                                app.send_error_toast("Cannot move card down, it is already at the bottom of the board", None);
                                error!("Cannot move card down, it is already at the bottom of the board");
                                return AppReturn::Continue;
                            }
                            // update visible boards and cards
                            // check if both the cards that are being swapped are in the visible cards
                            let current_card_index_in_visible = app.visible_boards_and_cards
                                [&current_board_id]
                                .iter()
                                .position(|card_id| *card_id == current_card_id);
                            if current_card_index_in_visible.is_none() {
                                debug!("Cannot move card down without a current card index in visible cards");
                                return AppReturn::Continue;
                            }
                            let current_card_index_in_visible =
                                current_card_index_in_visible.unwrap();
                            if current_card_index_in_visible
                                == app.visible_boards_and_cards[&current_board_id].len() - 1
                            {
                                let card_below_id = app.boards
                                    [current_board_index_in_all_boards.unwrap()]
                                .cards[current_card_index_in_all + 1]
                                    .id;
                                let mut visible_cards: Vec<u128> = vec![];
                                visible_cards.push(card_below_id);
                                visible_cards.push(current_card_id);
                                // insert in reverse order till we reach the no of cards to show
                                for card in
                                    app.visible_boards_and_cards[&current_board_id].iter().rev()
                                {
                                    if *card != current_card_id
                                        && visible_cards.len()
                                            < app.config.no_of_cards_to_show as usize
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
                            app.boards[current_board_index_in_all_boards.unwrap()]
                                .cards
                                .swap(current_card_index_in_all, current_card_index_in_all + 1);
                        }
                    }
                    _ => {}
                }
                AppReturn::Continue
            }
            Action::MoveCardRight => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        if app.state.current_card_id.is_none() {
                            return AppReturn::Continue;
                        } else {
                            if let Some(current_board) = app.state.current_board_id {
                                let moved_from_board_index = app
                                    .boards
                                    .iter()
                                    .position(|board| board.id == current_board);
                                if moved_from_board_index.is_none() {
                                    app.send_error_toast(
                                        "Something went wrong, could not find the board",
                                        None,
                                    );
                                    debug!("Moved from board index is none");
                                    return AppReturn::Continue;
                                }
                                let moved_from_board_index = moved_from_board_index.unwrap();
                                // check if board is the last board
                                if moved_from_board_index < app.boards.len() - 1 {
                                    let moved_to_board_index = moved_from_board_index + 1;
                                    if let Some(current_card) = app.state.current_card_id {
                                        let card_index = app.boards[moved_from_board_index]
                                            .cards
                                            .iter()
                                            .position(|card| card.id == current_card);
                                        if let Some(card_index) = card_index {
                                            let card = app.boards[moved_from_board_index]
                                                .cards
                                                .remove(card_index);
                                            let card_id = card.id;
                                            let card_name = card.name.clone();
                                            app.boards[moved_to_board_index].cards.push(card);
                                            // if the next board has cards less than the app.config.no_of_cards_to_show, then add the card to the visible cards
                                            if app.boards[moved_to_board_index].cards.len()
                                                <= app.config.no_of_cards_to_show as usize
                                            {
                                                app.visible_boards_and_cards
                                                    .entry(app.boards[moved_to_board_index].id)
                                                    .and_modify(|cards| cards.push(card_id));
                                            }
                                            // remove the moved card from visible cards for the current board
                                            app.visible_boards_and_cards
                                                .entry(app.boards[moved_from_board_index].id)
                                                .and_modify(|cards| {
                                                    cards.retain(|card_id| *card_id != current_card)
                                                });
                                            // set the visible cards to the last no_of_cards_to_show cards
                                            let mut moved_to_board_visible_cards: Vec<u128> =
                                                vec![];
                                            let mut moved_from_board_visible_cards: Vec<u128> =
                                                vec![];
                                            for card in
                                                app.boards[moved_to_board_index].cards.iter().rev()
                                            {
                                                if moved_to_board_visible_cards.len()
                                                    < app.config.no_of_cards_to_show as usize
                                                {
                                                    moved_to_board_visible_cards.insert(0, card.id);
                                                }
                                            }
                                            for card in app.boards[moved_from_board_index]
                                                .cards
                                                .iter()
                                                .rev()
                                            {
                                                if moved_from_board_visible_cards.len()
                                                    < app.config.no_of_cards_to_show as usize
                                                    && !moved_to_board_visible_cards
                                                        .contains(&card.id)
                                                {
                                                    moved_from_board_visible_cards
                                                        .insert(0, card.id);
                                                }
                                            }
                                            app.visible_boards_and_cards
                                                .entry(app.boards[moved_to_board_index].id)
                                                .and_modify(|cards| {
                                                    *cards = moved_to_board_visible_cards
                                                });
                                            app.visible_boards_and_cards
                                                .entry(app.boards[moved_from_board_index].id)
                                                .and_modify(|cards| {
                                                    *cards = moved_from_board_visible_cards
                                                });
                                            app.state.current_board_id =
                                                Some(app.boards[moved_to_board_index].id);
                                            info!(
                                                "Moved card {} to board \"{}\"",
                                                card_name, app.boards[moved_to_board_index].name
                                            );
                                            app.send_info_toast(
                                                &format!(
                                                    "Moved card {} to board \"{}\"",
                                                    card_name,
                                                    app.boards[moved_to_board_index].name
                                                ),
                                                None,
                                            );
                                        }
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
                    }
                    _ => {}
                }
                AppReturn::Continue
            }
            Action::MoveCardLeft => {
                if !UiMode::view_modes().contains(&app.state.ui_mode) {
                    return AppReturn::Continue;
                }
                match app.state.focus {
                    Focus::Body => {
                        if app.state.current_card_id.is_none() {
                            return AppReturn::Continue;
                        } else {
                            if let Some(current_board) = app.state.current_board_id {
                                let moved_from_board_index = app
                                    .boards
                                    .iter()
                                    .position(|board| board.id == current_board);
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
                                // check if board is the first board
                                if moved_from_board_index > 0 {
                                    if let Some(current_card) = app.state.current_card_id {
                                        let card_index = app.boards[moved_from_board_index]
                                            .cards
                                            .iter()
                                            .position(|card| card.id == current_card);
                                        if let Some(card_index) = card_index {
                                            let card = app.boards[moved_from_board_index]
                                                .cards
                                                .remove(card_index);
                                            let card_id = card.id;
                                            let card_name = card.name.clone();
                                            app.boards[moved_to_board_index].cards.push(card);
                                            // if the next board has cards less than the app.config.no_of_cards_to_show, then add the card to the visible cards
                                            if app.boards[moved_to_board_index].cards.len()
                                                <= app.config.no_of_cards_to_show as usize
                                            {
                                                app.visible_boards_and_cards
                                                    .entry(app.boards[moved_to_board_index].id)
                                                    .and_modify(|cards| cards.push(card_id));
                                            }
                                            // remove the moved card from visible cards for the current board
                                            app.visible_boards_and_cards
                                                .entry(app.boards[moved_from_board_index].id)
                                                .and_modify(|cards| {
                                                    cards.retain(|card_id| *card_id != current_card)
                                                });
                                            // set the visible cards to the last no_of_cards_to_show cards
                                            let mut moved_to_board_visible_cards: Vec<u128> =
                                                vec![];
                                            let mut moved_from_board_visible_cards: Vec<u128> =
                                                vec![];
                                            for card in
                                                app.boards[moved_to_board_index].cards.iter().rev()
                                            {
                                                if moved_to_board_visible_cards.len()
                                                    < app.config.no_of_cards_to_show as usize
                                                {
                                                    moved_to_board_visible_cards.insert(0, card.id);
                                                }
                                            }
                                            for card in app.boards[moved_from_board_index]
                                                .cards
                                                .iter()
                                                .rev()
                                            {
                                                if moved_from_board_visible_cards.len()
                                                    < app.config.no_of_cards_to_show as usize
                                                    && !moved_to_board_visible_cards
                                                        .contains(&card.id)
                                                {
                                                    moved_from_board_visible_cards
                                                        .insert(0, card.id);
                                                }
                                            }
                                            app.visible_boards_and_cards
                                                .entry(app.boards[moved_to_board_index].id)
                                                .and_modify(|cards| {
                                                    *cards = moved_to_board_visible_cards
                                                });
                                            app.visible_boards_and_cards
                                                .entry(app.boards[moved_from_board_index].id)
                                                .and_modify(|cards| {
                                                    *cards = moved_from_board_visible_cards
                                                });
                                            app.state.current_board_id =
                                                Some(app.boards[moved_to_board_index].id);
                                            info!(
                                                "Moved card {} to board \"{}\"",
                                                card_name, app.boards[moved_to_board_index].name
                                            );
                                            app.send_info_toast(
                                                &format!(
                                                    "Moved card {} to board \"{}\"",
                                                    card_name,
                                                    app.boards[moved_to_board_index].name
                                                ),
                                                None,
                                            );
                                        }
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
                    }
                    _ => {}
                }
                AppReturn::Continue
            }
            Action::ToggleCommandPalette => {
                if app.state.popup_mode.is_none() {
                    app.state.popup_mode = Some(PopupMode::CommandPalette);
                    app.state.current_user_input = String::new();
                    app.state.current_cursor_position = None;
                    app.state.app_status = AppStatus::UserInput;
                } else if app.state.popup_mode == Some(PopupMode::CommandPalette) {
                    app.state.popup_mode = None;
                    app.state.current_user_input = String::new();
                    app.state.current_cursor_position = None;
                    app.state.app_status = AppStatus::Initialized;
                } else {
                    app.state.popup_mode = Some(PopupMode::CommandPalette);
                    app.state.current_user_input = String::new();
                    app.state.current_cursor_position = None;
                    app.state.app_status = AppStatus::UserInput;
                }
                AppReturn::Continue
            }
            Action::ClearAllToasts => {
                app.state.toasts.clear();
                info!("Cleared toast messages");
                AppReturn::Continue
            }
        }
    } else {
        warn!("No action accociated to {}", key);
        app.send_warning_toast(&format!("No action accociated to {}", key), None);
        AppReturn::Continue
    }
}

pub async fn handle_mouse_action(app: &mut App, mouse_action: Mouse) -> AppReturn {
    let mut left_button_pressed = false;
    let mut right_button_pressed = false;
    let mut middle_button_pressed = false;
    let mut mouse_scroll_up = false;
    let mut mouse_scroll_down = false;
    let mut mouse_scroll_left = false;
    let mut mouse_scroll_right = false;
    match mouse_action {
        Mouse::Move(x, y) => {
            app.state.current_mouse_coordinates = (x, y);
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
    if let Mouse::Move(x, y) = mouse_action {
        app.state.current_mouse_coordinates = (x, y);
    }
    if right_button_pressed {
        return handle_go_to_previous_ui_mode(app);
    }

    if middle_button_pressed {
        if app.state.popup_mode == Some(PopupMode::CommandPalette) {
            app.state.popup_mode = None;
            app.state.current_user_input = String::new();
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        } else {
            app.state.popup_mode = Some(PopupMode::CommandPalette);
            app.state.current_user_input = String::new();
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::UserInput;
        }
        return AppReturn::Continue;
    }

    if app.state.popup_mode.is_some() {
        let popup_mode = app.state.popup_mode.unwrap();
        match popup_mode {
            PopupMode::CommandPalette => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CommandPalette) {
                        app.state.popup_mode = None;
                        return CommandPalette::handle_command(app).await;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::SelectDefaultView => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SelectDefaultView) {
                        handle_default_view_selection(app);
                        app.state.popup_mode = None;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::ChangeUIMode => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ChangeUiModePopup) {
                        handle_change_ui_mode(app);
                        app.state.popup_mode = None;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::ChangeCurrentCardStatus => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ChangeCardStatusPopup) {
                        return handle_change_current_card_status(app);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::EditGeneralConfig => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::EditGeneralConfigPopup) {
                        app.state.app_status = AppStatus::UserInput;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.state.popup_mode = None;
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.state.popup_mode = None;
                        if app.state.ui_mode == UiMode::CreateTheme {
                            handle_create_theme_action(app);
                        } else {
                            handle_edit_general_config(app);
                        }
                    }
                }
            }
            PopupMode::EditSpecificKeyBinding => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::EditSpecificKeyBindingPopup) {
                        app.state.app_status = AppStatus::KeyBindMode;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.state.popup_mode = None;
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.state.popup_mode = None;
                        handle_edit_specific_keybinding(app);
                    }
                }
            }
            PopupMode::ChangeTheme => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::ThemeSelector) {
                        handle_change_theme(app, app.state.defualt_theme_mode);
                        app.state.popup_mode = None;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::ThemeEditor => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.app_status = AppStatus::Initialized;
                        app.state.popup_mode = None;
                    } else if app.state.mouse_focus == Some(Focus::StyleEditorFG)
                        || app.state.mouse_focus == Some(Focus::StyleEditorBG)
                        || app.state.mouse_focus == Some(Focus::StyleEditorModifier)
                        || app.state.mouse_focus == Some(Focus::SubmitButton)
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
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::CustomRGBPromptFG => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_custom_rgb_prompt(app, true);
                    } else if app.state.mouse_focus == Some(Focus::TextInput) {
                        app.state.app_status = AppStatus::UserInput;
                        app.state.current_user_input = String::new();
                        app.state.current_cursor_position = None;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                    }
                }
            }
            PopupMode::CustomRGBPromptBG => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_custom_rgb_prompt(app, false);
                    } else if app.state.mouse_focus == Some(Focus::TextInput) {
                        app.state.app_status = AppStatus::UserInput;
                        app.state.current_user_input = String::new();
                        app.state.current_cursor_position = None;
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                        app.state.app_status = AppStatus::Initialized;
                    }
                }
            }
            _ => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.popup_mode = None;
                        app.state.app_status = AppStatus::Initialized;
                    }
                }
            }
        }
    } else {
        match app.state.ui_mode {
            UiMode::Zen => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                }
            }
            UiMode::TitleBody => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Title) {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next()
                        }
                        app.state.prev_ui_mode = Some(UiMode::TitleBody);
                    } else if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::BodyHelp => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::Help) {
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.prev_ui_mode = Some(UiMode::BodyHelp);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::BodyLog => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::BodyLog);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::TitleBodyHelp => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Title) {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next()
                        }
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyHelp);
                    } else if app.state.mouse_focus == Some(Focus::Help) {
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyHelp);
                    } else if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::TitleBodyLog => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Title) {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next()
                        }
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyLog);
                    } else if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyLog);
                    } else if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::TitleBodyHelpLog => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Title) {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next()
                        }
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyHelpLog);
                    } else if app.state.mouse_focus == Some(Focus::Help) {
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyHelpLog);
                    } else if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::TitleBodyHelpLog);
                    } else if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::BodyHelpLog => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        app.state.popup_mode = Some(PopupMode::CardView);
                    } else if app.state.mouse_focus == Some(Focus::Help) {
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.prev_ui_mode = Some(UiMode::BodyHelpLog);
                    } else if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::BodyHelpLog);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_up(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_down(app);
                    }
                } else if mouse_scroll_right {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_right(app);
                    }
                } else if mouse_scroll_left {
                    if app.state.mouse_focus == Some(Focus::Body) {
                        scroll_left(app);
                    }
                }
            }
            UiMode::ConfigMenu => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Title) {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next()
                        }
                        app.state.prev_ui_mode = Some(UiMode::ConfigMenu);
                    } else if app.state.mouse_focus == Some(Focus::ConfigTable)
                        || app.state.mouse_focus == Some(Focus::SubmitButton)
                        || app.state.mouse_focus == Some(Focus::ExtraFocus)
                    {
                        return handle_config_menu_action(app);
                    } else if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::ConfigMenu);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_prv_ui_mode(app);
                    }
                }
            }
            UiMode::EditKeybindings => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Title) {
                        app.state.ui_mode = UiMode::MainMenu;
                        if app.state.main_menu_state.selected().is_none() {
                            app.main_menu_next()
                        }
                        app.state.prev_ui_mode = Some(UiMode::EditKeybindings);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.ui_mode = UiMode::ConfigMenu;
                    } else if app.state.mouse_focus == Some(Focus::EditKeybindingsTable) {
                        handle_edit_keybindings_action(app);
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        app.state.focus = Focus::SubmitButton;
                        handle_edit_keybindings_action(app);
                    }
                } else if mouse_scroll_down {
                    if app.state.mouse_focus == Some(Focus::EditKeybindingsTable) {
                        let current_selected = app.state.edit_keybindings_state.selected();
                        if current_selected.is_none() {
                            app.state.edit_keybindings_state.select(Some(0));
                        } else {
                            let current_selected = current_selected.unwrap();
                            if current_selected < app.config.keybindings.iter().count() - 1 {
                                app.state
                                    .edit_keybindings_state
                                    .select(Some(current_selected + 1));
                            } else {
                                app.state.edit_keybindings_state.select(Some(0));
                            }
                        }
                    }
                } else if mouse_scroll_up {
                    if app.state.mouse_focus == Some(Focus::EditKeybindingsTable) {
                        let current_selected = app.state.edit_keybindings_state.selected();
                        if current_selected.is_none() {
                            app.state.edit_keybindings_state.select(Some(0));
                        } else {
                            let current_selected = current_selected.unwrap();
                            if current_selected > 0 {
                                app.state
                                    .edit_keybindings_state
                                    .select(Some(current_selected - 1));
                            } else {
                                app.state
                                    .edit_keybindings_state
                                    .select(Some(app.config.keybindings.iter().count() - 1));
                            }
                        }
                    }
                }
            }
            UiMode::MainMenu => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Help) {
                        app.state.ui_mode = UiMode::HelpMenu;
                        app.state.prev_ui_mode = Some(UiMode::MainMenu);
                    } else if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::MainMenu);
                    } else if app.state.mouse_focus == Some(Focus::MainMenu) {
                        return handle_main_menu_action(app);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        return AppReturn::Exit;
                    }
                }
            }
            UiMode::HelpMenu => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::Log) {
                        app.state.ui_mode = UiMode::LogsOnly;
                        app.state.prev_ui_mode = Some(UiMode::HelpMenu);
                    } else if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_prv_ui_mode(app);
                    }
                }
            }
            UiMode::LogsOnly => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_prv_ui_mode(app);
                    }
                }
            }
            UiMode::NewBoard => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_prv_ui_mode(app);
                        app.state.new_board_form = NEW_BOARD_FORM_DEFAULT_STATE
                            .iter()
                            .map(|s| s.to_string())
                            .collect();
                    } else if app.state.mouse_focus == Some(Focus::NewBoardName) {
                        app.state.app_status = AppStatus::UserInput
                    } else if app.state.mouse_focus == Some(Focus::NewBoardDescription) {
                        app.state.app_status = AppStatus::UserInput
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_new_board_action(app);
                        app.state.app_status = AppStatus::Initialized;
                    }
                }
            }
            UiMode::NewCard => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_prv_ui_mode(app);
                        app.state.new_board_form = NEW_CARD_FORM_DEFAULT_STATE
                            .iter()
                            .map(|s| s.to_string())
                            .collect();
                    } else if app.state.mouse_focus == Some(Focus::NewCardName) {
                        app.state.app_status = AppStatus::UserInput
                    } else if app.state.mouse_focus == Some(Focus::NewCardDescription) {
                        app.state.app_status = AppStatus::UserInput
                    } else if app.state.mouse_focus == Some(Focus::NewCardDueDate) {
                        app.state.app_status = AppStatus::UserInput
                    } else if app.state.mouse_focus == Some(Focus::SubmitButton) {
                        handle_new_card_action(app);
                        app.state.app_status = AppStatus::Initialized;
                    }
                }
            }
            UiMode::LoadSave => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        handle_go_to_prv_ui_mode(app);
                    } else if app.state.mouse_focus == Some(Focus::LoadSave) {
                        if app.state.load_save_state.selected().is_some() {
                            app.dispatch(IoEvent::LoadPreview).await;
                        }
                    }
                }
            }
            UiMode::CreateTheme => {
                if left_button_pressed {
                    if app.state.mouse_focus == Some(Focus::CloseButton) {
                        app.state.theme_being_edited = Theme::default();
                        handle_go_to_prv_ui_mode(app);
                    } else {
                        handle_create_theme_action(app);
                    }
                }
            }
        }
    }
    AppReturn::Continue
}

fn handle_config_menu_action(app: &mut App) -> AppReturn {
    if app.state.focus == Focus::SubmitButton {
        app.config = AppConfig::default();
        app.state.focus = Focus::ConfigTable;
        app.state.config_state.select(Some(0));
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
            warn!("Reset Config and Keybinds to default");
            app.send_warning_toast("Reset Config and Keybinds to default", None);
        }
        app.keybind_list_maker();
        return AppReturn::Continue;
    } else if app.state.focus == Focus::ExtraFocus {
        // make a copy of the keybinds and reset only config and then write the config
        let keybinds = app.config.keybindings.clone();
        app.config = AppConfig::default();
        app.config.keybindings = keybinds;
        app.state.focus = Focus::ConfigTable;
        app.state.config_state.select(Some(0));
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
            warn!("Reset Config to default");
            app.send_warning_toast("Reset Config to default", None);
        }
        return AppReturn::Continue;
    }
    app.config_item_being_edited = Some(app.state.config_state.selected().unwrap_or(0));
    // check if the config_item_being_edited index is in the AppConfig list and the value in the list is Edit Keybindings
    let app_config_list = &app.config.to_list();
    if app.config_item_being_edited.unwrap_or(0) < app_config_list.len() {
        let default_config_item = String::from("");
        let config_item = &app_config_list[app.config_item_being_edited.unwrap_or(0)]
            .first()
            .unwrap_or_else(|| &default_config_item);
        if *config_item == "Edit Keybindings" {
            app.state.ui_mode = UiMode::EditKeybindings;
            if app.state.edit_keybindings_state.selected().is_none() {
                app.edit_keybindings_next();
            }
        } else if *config_item == "Select Default View" {
            if app.state.default_view_state.selected().is_none() {
                app.select_default_view_next();
            }
            app.state.popup_mode = Some(PopupMode::SelectDefaultView);
        } else if *config_item == "Auto Save on Exit" {
            let save_on_exit = app.config.save_on_exit;
            app.config.save_on_exit = !save_on_exit;
            let config_string = format!("{}: {}", "Auto Save on Exit", app.config.save_on_exit);
            let app_config = AppConfig::edit_with_string(&config_string, app);
            app.config = app_config.clone();
            let write_config_status = write_config(&app_config);
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
                app.send_info_toast("Config updated Successfully", None);
            }
        } else if *config_item == "Auto Load Last Save" {
            let always_load_last_save = app.config.always_load_last_save;
            app.config.always_load_last_save = !always_load_last_save;
            let config_string = format!(
                "{}: {}",
                "Auto Load Last Save", app.config.always_load_last_save
            );
            let app_config = AppConfig::edit_with_string(&config_string, app);
            app.config = app_config.clone();
            let write_config_status = write_config(&app_config);
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
                app.send_info_toast("Config updated Successfully", None);
            }
        } else if *config_item == "Disable Scrollbars" {
            let disable_scrollbars = app.config.disable_scrollbars;
            app.config.disable_scrollbars = !disable_scrollbars;
            let config_string = format!(
                "{}: {}",
                "Disable Scrollbars", app.config.disable_scrollbars
            );
            let app_config = AppConfig::edit_with_string(&config_string, app);
            app.config = app_config.clone();
            let write_config_status = write_config(&app_config);
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
                app.send_info_toast("Config updated Successfully", None);
            }
        } else if *config_item == "Enable Mouse Support" {
            let enable_mouse_support = app.config.enable_mouse_support;
            app.config.enable_mouse_support = !enable_mouse_support;
            let config_string = format!(
                "{}: {}",
                "Enable Mouse Support", app.config.enable_mouse_support
            );
            let app_config = AppConfig::edit_with_string(&config_string, app);
            app.config = app_config.clone();
            let write_config_status = write_config(&app_config);
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
                app.send_info_toast("Config updated Successfully", None);
                app.send_warning_toast("Please restart the app to apply the changes", None);
            }
        } else if *config_item == "Default Theme" {
            app.state.defualt_theme_mode = true;
            app.state.popup_mode = Some(PopupMode::ChangeTheme);
        } else {
            app.state.popup_mode = Some(PopupMode::EditGeneralConfig);
        }
    } else {
        debug!(
            "Config item being edited {} is not in the AppConfig list",
            app.config_item_being_edited.unwrap_or(0)
        );
    }
    AppReturn::Continue
}

fn handle_main_menu_action(app: &mut App) -> AppReturn {
    if app.state.main_menu_state.selected().is_some() {
        let selected_index = app.state.main_menu_state.selected().unwrap();
        let selected_item = MainMenu::from_index(selected_index);
        match selected_item {
            MainMenuItem::Quit => return AppReturn::Exit,
            MainMenuItem::Config => {
                app.state.prev_ui_mode = Some(UiMode::MainMenu);
                app.state.ui_mode = UiMode::ConfigMenu;
                if app.state.config_state.selected().is_none() {
                    app.config_next();
                }
            }
            MainMenuItem::View => {
                app.state.prev_ui_mode = Some(UiMode::MainMenu);
                app.state.ui_mode = app.config.default_view.clone();
            }
            MainMenuItem::Help => {
                app.state.prev_ui_mode = Some(UiMode::MainMenu);
                app.state.ui_mode = UiMode::HelpMenu;
            }
            MainMenuItem::LoadSave => {
                app.state.prev_ui_mode = Some(UiMode::MainMenu);
                app.state.ui_mode = UiMode::LoadSave;
            }
        }
    }
    AppReturn::Continue
}

fn handle_default_view_selection(app: &mut App) {
    let all_ui_modes = UiMode::all();
    let current_selected_mode = app.state.default_view_state.selected().unwrap_or(0);
    if current_selected_mode < all_ui_modes.len() {
        let selected_mode = &all_ui_modes[current_selected_mode];
        app.config.default_view = UiMode::from_string(&selected_mode).unwrap_or(UiMode::MainMenu);
        app.state.prev_ui_mode = Some(app.config.default_view.clone());
        let config_string = format!("{}: {}", "Select Default View", selected_mode);
        let app_config = AppConfig::edit_with_string(&config_string, app);
        app.config = app_config.clone();
        let write_config_status = write_config(&app_config);
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
            app.send_info_toast("Config updated Successfully", None);
        }

        // reset everything
        app.state.default_view_state.select(Some(0));
        app.state.ui_mode = UiMode::ConfigMenu;
        if app.state.config_state.selected().is_none() {
            app.config_next();
        }
        if app.state.popup_mode.is_some() {
            app.state.popup_mode = None;
        }
    } else {
        debug!(
            "Selected mode {} is not in the list of all UI modes",
            current_selected_mode
        );
    }
}

fn handle_change_ui_mode(app: &mut App) {
    let current_index = app.state.default_view_state.selected().unwrap_or(0);
    // UiMode::all() has strings map all of them to UiMode using UiMode::from_string which returns an option<UiMode>
    let all_ui_modes = UiMode::all()
        .iter()
        .map(|s| UiMode::from_string(s))
        .filter(|s| s.is_some())
        .map(|s| s.unwrap())
        .collect::<Vec<UiMode>>();

    // make sure index is in bounds
    let current_index = if current_index >= all_ui_modes.len() {
        all_ui_modes.len() - 1
    } else {
        current_index
    };
    let selected_ui_mode = all_ui_modes[current_index].clone();
    app.state.ui_mode = selected_ui_mode;
}

fn handle_edit_keybindings_action(app: &mut App) {
    if app.state.edit_keybindings_state.selected().is_some()
        && app.state.focus != Focus::SubmitButton
    {
        app.state.popup_mode = Some(PopupMode::EditSpecificKeyBinding);
    } else if app.state.focus == Focus::SubmitButton {
        app.config.keybindings = KeyBindings::default();
        warn!("Reset keybindings to default");
        app.send_warning_toast("Reset keybindings to default", None);
        app.state.focus = Focus::NoFocus;
        app.state.edit_keybindings_state.select(None);
        let write_config_status = write_config(&app.config);
        if write_config_status.is_err() {
            error!(
                "Error writing config: {}",
                write_config_status.clone().unwrap_err()
            );
            app.send_error_toast(&write_config_status.unwrap_err(), None);
        }
        app.keybind_list_maker();
    }
}

fn handle_go_to_previous_ui_mode(app: &mut App) -> AppReturn {
    if app.state.popup_mode.is_some() {
        if app.state.popup_mode.unwrap() == PopupMode::EditGeneralConfig {
            if app.state.ui_mode == UiMode::CreateTheme {
                app.state.popup_mode = None;
            } else {
                app.state.ui_mode = UiMode::ConfigMenu;
                if app.state.config_state.selected().is_none() {
                    app.config_next()
                }
            }
            app.state.current_user_input = String::new();
            app.state.current_cursor_position = None;
        } else if app.state.popup_mode.unwrap() == PopupMode::EditSpecificKeyBinding {
            app.state.ui_mode = UiMode::EditKeybindings;
            if app.state.edit_keybindings_state.selected().is_none() {
                app.edit_keybindings_next();
            }
        }
        app.state.popup_mode = None;
        if app.state.app_status == AppStatus::UserInput {
            app.state.app_status = AppStatus::Initialized;
        }
        return AppReturn::Continue;
    }
    match app.state.ui_mode {
        UiMode::ConfigMenu => {
            if app.state.prev_ui_mode == Some(UiMode::ConfigMenu) {
                app.state.prev_ui_mode = None;
                app.state.ui_mode = app.config.default_view.clone();
            } else {
                app.state.ui_mode = app
                    .state
                    .prev_ui_mode
                    .as_ref()
                    .unwrap_or_else(|| &app.config.default_view)
                    .clone();
                app.state.prev_ui_mode = Some(UiMode::ConfigMenu);
            }
            AppReturn::Continue
        }
        UiMode::MainMenu => AppReturn::Exit,
        UiMode::EditKeybindings => {
            app.state.ui_mode = UiMode::ConfigMenu;
            if app.state.config_state.selected().is_none() {
                app.config_next()
            }
            AppReturn::Continue
        }
        _ => {
            if app.state.ui_mode == UiMode::LoadSave {
                app.state.load_save_state = ListState::default();
            }
            // check if previous ui mode is the same as the current ui mode
            if app.state.prev_ui_mode == Some(app.state.ui_mode.clone()) {
                app.state.ui_mode = UiMode::MainMenu;
                if app.state.main_menu_state.selected().is_none() {
                    app.main_menu_next();
                }
            } else {
                app.state.ui_mode = app
                    .state
                    .prev_ui_mode
                    .as_ref()
                    .unwrap_or_else(|| &UiMode::MainMenu)
                    .clone();
                if app.state.main_menu_state.selected().is_none() {
                    app.main_menu_next();
                }
            }
            AppReturn::Continue
        }
    }
}

fn handle_change_current_card_status(app: &mut App) -> AppReturn {
    let current_index = app.state.card_status_selector_state.selected().unwrap_or(0);
    let all_statuses = CardStatus::all();

    let current_index = if current_index >= all_statuses.len() {
        all_statuses.len() - 1
    } else {
        current_index
    };
    let selected_status = all_statuses[current_index].clone();
    // find current board from app.boards
    if let Some(current_board_id) = app.state.current_board_id {
        if let Some(current_board) = app.boards.iter_mut().find(|b| b.id == current_board_id) {
            if let Some(current_card_id) = app.state.current_card_id {
                if let Some(current_card) = current_board
                    .cards
                    .iter_mut()
                    .find(|c| c.id == current_card_id)
                {
                    current_card.card_status = selected_status;
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
            }
        }
    }
    app.send_error_toast("Error Could not find current card", None);
    return AppReturn::Continue;
}

fn handle_edit_general_config(app: &mut App) {
    let config_item_index = app.state.config_state.selected().unwrap_or(0);
    let config_item_list = AppConfig::to_list(&app.config);
    let config_item = config_item_list[config_item_index].clone();
    // key is the second item in the list
    let default_key = String::from("");
    let config_item_key = config_item.get(0).unwrap_or_else(|| &default_key);
    let new_value = app.state.current_user_input.clone();
    // if new value is not empty update the config
    if !new_value.is_empty() {
        let config_string = format!("{}: {}", config_item_key, new_value);
        let app_config = AppConfig::edit_with_string(&config_string, app);
        app.config = app_config.clone();
        let write_config_status = write_config(&app_config);
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
            app.send_info_toast("Config updated Successfully", None);
        }

        // reset everything
        app.state.config_state.select(Some(0));
        app.config_item_being_edited = None;
        app.state.current_user_input = String::new();
        app.state.ui_mode = UiMode::ConfigMenu;
        if app.state.config_state.selected().is_none() {
            app.config_next();
        }
        refresh_visible_boards_and_cards(app);
    }
    app.state.config_state.select(Some(0));
}

fn handle_edit_specific_keybinding(app: &mut App) {
    if app.state.edited_keybinding.is_some() {
        let selected = app.state.edit_keybindings_state.selected().unwrap();
        if selected < app.config.keybindings.iter().count() {
            let result = app.config.edit_keybinding(
                selected,
                app.state
                    .edited_keybinding
                    .clone()
                    .unwrap_or_else(|| vec![]),
            );
            if result.is_err() {
                app.send_error_toast(&result.unwrap_err(), None);
            } else {
                let mut key_list = vec![];
                for (k, v) in app.config.keybindings.iter() {
                    key_list.push((k, v));
                }
                let (key, _) = key_list[selected];
                let key_string = key.to_string();
                let value = app
                    .state
                    .edited_keybinding
                    .clone()
                    .unwrap_or_else(|| vec![]);
                let value = value
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                app.send_info_toast(
                    &format!("Keybind for {} updated to {}", key_string, value),
                    None,
                );
            }
        } else {
            error!("Selected keybind with id {} not found", selected);
            app.send_error_toast("Selected keybind not found", None);
            app.state.edited_keybinding = None;
            app.state.edit_keybindings_state.select(None);
        }
        app.state.ui_mode = UiMode::EditKeybindings;
        if app.state.edit_keybindings_state.selected().is_none() {
            app.edit_keybindings_next()
        }
        app.state.edited_keybinding = None;
        let write_config_status = write_config(&app.config);
        if write_config_status.is_err() {
            error!(
                "Error writing config: {}",
                write_config_status.clone().unwrap_err()
            );
            app.send_error_toast(&write_config_status.unwrap_err(), None);
        }
    } else {
        app.state.ui_mode = UiMode::EditKeybindings;
        if app.state.edit_keybindings_state.selected().is_none() {
            app.edit_keybindings_next()
        }
    }
    app.keybind_list_maker();
}

fn handle_new_board_action(app: &mut App) {
    if app.state.focus == Focus::SubmitButton {
        // check if app.state.new_board_form[0] is not empty or is not the same as any of the existing boards
        let new_board_name = app.state.new_board_form[0].clone();
        let new_board_description = app.state.new_board_form[1].clone();
        let mut same_name_exists = false;
        for board in app.boards.iter() {
            if board.name == new_board_name {
                same_name_exists = true;
                break;
            }
        }
        if !new_board_name.is_empty() && !same_name_exists {
            let new_board = Board::new(new_board_name, new_board_description);
            app.boards.push(new_board.clone());
            app.state.current_board_id = Some(new_board.id);
            app.state.ui_mode = app
                .state
                .prev_ui_mode
                .as_ref()
                .unwrap_or_else(|| &app.config.default_view)
                .clone();
        } else {
            warn!("New board name is empty or already exists");
            app.send_warning_toast("New board name is empty or already exists", None);
        }
        app.state.ui_mode = app
            .state
            .prev_ui_mode
            .as_ref()
            .unwrap_or_else(|| &app.config.default_view)
            .clone();
        if let Some(previous_focus) = &app.state.previous_focus {
            app.state.focus = previous_focus.clone();
        }
        refresh_visible_boards_and_cards(app);
    }
    app.state.new_board_form = NEW_BOARD_FORM_DEFAULT_STATE
        .iter()
        .map(|s| s.to_string())
        .collect();
}

fn handle_new_card_action(app: &mut App) -> AppReturn {
    if app.state.focus == Focus::SubmitButton {
        // check if app.state.new_card_form[0] is not empty or is not the same as any of the existing cards
        let new_card_name = app.state.new_card_form[0].clone();
        let new_card_description = app.state.new_card_form[1].clone();
        let new_card_due_date = app.state.new_card_form[2].clone();
        let mut same_name_exists = false;
        let current_board_id = app.state.current_board_id.unwrap_or(0);
        let current_board = app.boards.iter().find(|board| board.id == current_board_id);
        if let Some(current_board) = current_board {
            for card in current_board.cards.iter() {
                if card.name == new_card_name {
                    same_name_exists = true;
                    break;
                }
            }
        } else {
            debug!("Current board not found");
            app.send_error_toast("Something went wrong", None);
            app.state.ui_mode = app
                .state
                .prev_ui_mode
                .as_ref()
                .unwrap_or_else(|| &app.config.default_view)
                .clone();
            return AppReturn::Continue;
        }
        // check if due date is empty or is a valid date
        let due_date = if new_card_due_date.is_empty() {
            Some(FIELD_NOT_SET.to_string())
        } else {
            match NaiveDateTime::parse_from_str(&new_card_due_date, "%d/%m/%Y-%H:%M:%S") {
                Ok(due_date) => {
                    debug!("Due date: {}", due_date);
                    let new_due = due_date.to_string();
                    // the date is in the format 2023-01-20 14:10:00 convert it to 20/01/2023-14:10:00
                    let new_due = new_due.replace("-", "/");
                    let new_due = new_due.replace(" ", "-");
                    debug!("New due date: {}", new_due);
                    Some(new_due)
                }
                Err(_) => {
                    // cehck if the user has not put the time if not put the default time
                    match NaiveDate::parse_from_str(&new_card_due_date, "%d/%m/%Y") {
                        Ok(due_date) => {
                            debug!("Due date: {}", due_date);
                            let new_due = due_date.to_string();
                            // the date is in the format 2023-01-20 14:10:00 convert it to 20/01/2023-14:10:00
                            let new_due = new_due.replace("-", "/");
                            let new_due = new_due.replace(" ", "-");
                            let new_due = format!("{}-{}", new_due, "12:00:00");
                            debug!("New due date: {}", new_due);
                            Some(new_due)
                        }
                        Err(e) => {
                            debug!("Invalid due date: {}", e);
                            debug!("Due date: {}", new_card_due_date);
                            None
                        }
                    }
                }
            }
        };
        if due_date.is_none() {
            warn!("Invalid due date");
            app.send_warning_toast("Invalid due date", None);
            app.state.ui_mode = app
                .state
                .prev_ui_mode
                .as_ref()
                .unwrap_or_else(|| &app.config.default_view)
                .clone();
            return AppReturn::Continue;
        }
        if !new_card_name.is_empty() && !same_name_exists {
            let new_card = Card::new(
                new_card_name,
                new_card_description,
                due_date.unwrap().to_string(),
                CardPriority::Low,
                vec![],
                vec![],
            );
            let current_board = app
                .boards
                .iter_mut()
                .find(|board| board.id == current_board_id);
            if let Some(current_board) = current_board {
                current_board.cards.push(new_card.clone());
                app.state.current_card_id = Some(new_card.id);
            } else {
                debug!("Current board not found");
                app.send_error_toast("Something went wrong", None);
                app.state.ui_mode = app
                    .state
                    .prev_ui_mode
                    .as_ref()
                    .unwrap_or_else(|| &app.config.default_view)
                    .clone();
                return AppReturn::Continue;
            }
            app.state.ui_mode = app
                .state
                .prev_ui_mode
                .as_ref()
                .unwrap_or_else(|| &app.config.default_view)
                .clone();
        } else {
            warn!("New card name is empty or already exists");
            app.send_warning_toast("New card name is empty or already exists", None);
        }

        if let Some(previous_focus) = &app.state.previous_focus {
            app.state.focus = previous_focus.clone();
        }
        refresh_visible_boards_and_cards(app);
    }
    app.state.new_card_form = NEW_CARD_FORM_DEFAULT_STATE
        .iter()
        .map(|s| s.to_string())
        .collect();
    AppReturn::Continue
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
    let current_board = app.boards.iter().find(|b| b.id == current_board_id);
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
    let all_card_ids = &current_board
        .cards
        .iter()
        .map(|c| c.id)
        .collect::<Vec<u128>>();
    // the current_visible_cards is a window of all_card_ids. check if another window can be created one card above the current window
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
    let current_board_id = app.state.current_board_id.unwrap();
    let current_board = app.boards.iter().find(|b| b.id == current_board_id);
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
    let all_card_ids = &current_board
        .cards
        .iter()
        .map(|c| c.id)
        .collect::<Vec<u128>>();
    // the current_visible_cards is a window of all_card_ids. check if another window can be created one card below the current window
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
    let last_board_in_visible = last_board_in_visible.unwrap();
    let last_board_index = app
        .boards
        .iter()
        .position(|b| b.id == *last_board_in_visible);
    if last_board_index.is_none() {
        debug!("No last board index found");
        return;
    }
    let last_board_index = last_board_index.unwrap();
    if last_board_index == app.boards.len() - 1 {
        return;
    }
    let next_board_index = last_board_index + 1;
    // get no_of_cards_to_show cards from the next board and add them to the visible boards
    let next_board = app
        .boards
        .iter()
        .find(|b| b.id == app.boards[next_board_index].id);
    if next_board.is_none() {
        debug!("No next board found");
        return;
    }
    let next_board = next_board.unwrap();
    let next_board_card_ids = next_board.cards.iter().map(|c| c.id).collect::<Vec<u128>>();
    let next_board_card_ids = if next_board_card_ids.len() > app.config.no_of_cards_to_show as usize
    {
        next_board_card_ids[0..app.config.no_of_cards_to_show as usize].to_vec()
    } else {
        next_board_card_ids
    };
    let mut new_visible_boards_and_cards = app.visible_boards_and_cards.clone();
    new_visible_boards_and_cards.insert(next_board.id, next_board_card_ids);
    // remove the first board from the visible boards
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
    let first_board_in_visible = first_board_in_visible.unwrap();
    let first_board_index = app
        .boards
        .iter()
        .position(|b| b.id == *first_board_in_visible);
    if first_board_index.is_none() {
        debug!("No first board index found");
        return;
    }
    let first_board_index = first_board_index.unwrap();
    if first_board_index == 0 {
        return;
    }
    let previous_board_index = first_board_index - 1;
    // get no_of_cards_to_show cards from the previous board and add them to the visible boards
    let previous_board = app
        .boards
        .iter()
        .find(|b| b.id == app.boards[previous_board_index].id);
    if previous_board.is_none() {
        debug!("No previous board found");
        return;
    }
    let previous_board = previous_board.unwrap();
    let previous_board_card_ids = previous_board
        .cards
        .iter()
        .map(|c| c.id)
        .collect::<Vec<u128>>();
    let previous_board_card_ids =
        if previous_board_card_ids.len() > app.config.no_of_cards_to_show as usize {
            previous_board_card_ids[0..app.config.no_of_cards_to_show as usize].to_vec()
        } else {
            previous_board_card_ids
        };
    let mut new_visible_boards_and_cards = LinkedHashMap::new();
    new_visible_boards_and_cards.insert(previous_board.id, previous_board_card_ids);
    // add the visible boards to the new visible boards except the last one
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
        app.state.defualt_theme_mode = false;
        let config_index = app.state.config_state.selected();
        if config_index.is_some() {
            let config_item_index = &app.config_item_being_edited;
            let list_items = get_config_items();
            let config_item_name = if config_item_index.is_some() {
                list_items[config_item_index.unwrap()].first().unwrap()
            } else {
                // TODO: This is temporary, as only the Theme editor uses this other than config
                "Theme Name"
            };
            if config_item_name == "Default Theme" {
                let theme_index = app.state.theme_selector_state.selected().unwrap_or(0);
                if theme_index < app.all_themes.len() {
                    let theme = app.all_themes[theme_index].clone();
                    app.config.default_theme = theme.name.clone();
                    let config_string = format!("{}: {}", "default_theme", theme.name);
                    let app_config = AppConfig::edit_with_string(&config_string, app);
                    app.config = app_config.clone();
                    let write_config_status = write_config(&app_config);
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
                        app.send_info_toast("Config updated Successfully", None);
                        app.theme = theme.clone();
                    }
                } else {
                    debug!("Theme index {} is not in the theme list", theme_index);
                }
            }
            app.state.popup_mode = None;
            return AppReturn::Continue;
        } else {
            debug!("No config index found");
            app.send_error_toast("Something went wrong", None);
            app.state.popup_mode = None;
            return AppReturn::Continue;
        }
    } else {
        let selected_item_index = app.state.theme_selector_state.selected();
        if selected_item_index.is_none() {
            debug!("No selected item index found");
            app.send_error_toast("Something went wrong", None);
            app.state.popup_mode = None;
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
        app.theme = selected_theme;
        app.state.popup_mode = None;
        AppReturn::Continue
    }
}

fn handle_create_theme_action(app: &mut App) -> AppReturn {
    if app.state.popup_mode.is_some() {
        let popup_mode = app.state.popup_mode.unwrap();
        if popup_mode == PopupMode::EditGeneralConfig {
            if app.state.current_user_input.is_empty() {
                app.send_error_toast("Theme name cannot be empty", None);
                app.state.popup_mode = None;
                return AppReturn::Continue;
            } else {
                let mut theme_name_duplicate = false;
                for theme in app.all_themes.iter() {
                    if theme.name == app.state.current_user_input {
                        theme_name_duplicate = true;
                        break;
                    }
                }
                if theme_name_duplicate {
                    app.send_error_toast("Theme name already exists", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                app.state.theme_being_edited.name = app.state.current_user_input.clone();
            }
            app.state.popup_mode = None;
        } else if popup_mode == PopupMode::ThemeEditor {
            if app.state.focus == Focus::StyleEditorFG {
                let all_color_options =
                    TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
                let selected_index = app.state.edit_specific_style_state.0.selected();
                if selected_index.is_none() {
                    debug!("No selected index found");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let selected_index = selected_index.unwrap();
                if selected_index >= all_color_options.len() {
                    debug!("Selected index is out of bounds");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let selected_color_option = all_color_options[selected_index].clone();
                let theme_style_bring_edited_index = app.state.theme_editor_state.selected();
                if theme_style_bring_edited_index.is_none() {
                    debug!("No theme style being edited index found");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let theme_style_bring_edited_index = theme_style_bring_edited_index.unwrap();
                if theme_style_bring_edited_index >= app.state.theme_being_edited.to_vec_str().len()
                {
                    debug!("Theme style being edited index is out of bounds");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
                    [theme_style_bring_edited_index]
                    .clone();
                // check if the selected_coloroption is of the type ColorOption::RGB(x,y,z)
                // if it is, then we need to open the color picker
                if let TextColorOptions::RGB(_, _, _) = selected_color_option {
                    app.state.popup_mode = Some(PopupMode::CustomRGBPromptFG);
                    app.state.focus = Focus::TextInput;
                    app.state.current_user_input = String::new();
                    app.state.current_cursor_position = None;
                    return AppReturn::Continue;
                }
                if selected_color_option.to_color().is_some() {
                    app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                        theme_style_being_edited,
                        Some(selected_color_option.to_color().unwrap()),
                        None,
                        None,
                    );
                } else {
                    app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                        theme_style_being_edited,
                        None,
                        None,
                        None,
                    );
                }
                handle_next_focus(app);
            } else if app.state.focus == Focus::StyleEditorBG {
                let all_color_options =
                    TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
                let selected_index = app.state.edit_specific_style_state.1.selected();
                if selected_index.is_none() {
                    debug!("No selected index found");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let selected_index = selected_index.unwrap();
                if selected_index >= all_color_options.len() {
                    debug!("Selected index is out of bounds");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let selected_color_option = all_color_options[selected_index].clone();
                let theme_style_bring_edited_index = app.state.theme_editor_state.selected();
                if theme_style_bring_edited_index.is_none() {
                    debug!("No theme style being edited index found");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let theme_style_bring_edited_index = theme_style_bring_edited_index.unwrap();
                if theme_style_bring_edited_index >= app.state.theme_being_edited.to_vec_str().len()
                {
                    debug!("Theme style being edited index is out of bounds");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
                    [theme_style_bring_edited_index]
                    .clone();
                // check if the selected_coloroption is of the type ColorOption::RGB(x,y,z)
                // if it is, then we need to open the color picker
                if let TextColorOptions::RGB(_, _, _) = selected_color_option {
                    app.state.popup_mode = Some(PopupMode::CustomRGBPromptBG);
                    app.state.focus = Focus::TextInput;
                    app.state.current_user_input = String::new();
                    app.state.current_cursor_position = None;
                    return AppReturn::Continue;
                }
                if selected_color_option.to_color().is_some() {
                    app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                        theme_style_being_edited,
                        None,
                        Some(selected_color_option.to_color().unwrap()),
                        None,
                    );
                } else {
                    app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                        theme_style_being_edited,
                        None,
                        None,
                        None,
                    );
                }
                handle_next_focus(app);
            } else if app.state.focus == Focus::StyleEditorModifier {
                let all_modifiers =
                    TextModifierOptions::to_iter().collect::<Vec<TextModifierOptions>>();
                let selected_index = app.state.edit_specific_style_state.2.selected();
                if selected_index.is_none() {
                    debug!("No selected index found");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let selected_index = selected_index.unwrap();
                if selected_index >= all_modifiers.len() {
                    debug!("Selected index is out of bounds");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let selected_modifier = all_modifiers[selected_index].clone();
                let theme_style_bring_edited_index = app.state.theme_editor_state.selected();
                if theme_style_bring_edited_index.is_none() {
                    debug!("No theme style being edited index found");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let theme_style_bring_edited_index = theme_style_bring_edited_index.unwrap();
                if theme_style_bring_edited_index >= app.state.theme_being_edited.to_vec_str().len()
                {
                    debug!("Theme style being edited index is out of bounds");
                    app.send_error_toast("Something went wrong", None);
                    app.state.popup_mode = None;
                    return AppReturn::Continue;
                }
                let theme_style_being_edited = app.state.theme_being_edited.to_vec_str()
                    [theme_style_bring_edited_index]
                    .clone();
                app.state.theme_being_edited = app.state.theme_being_edited.edit_style(
                    theme_style_being_edited,
                    None,
                    None,
                    Some(selected_modifier.to_modifier()),
                );
                handle_next_focus(app);
            } else if app.state.focus == Focus::SubmitButton {
                app.state.popup_mode = None;
                app.state.focus = Focus::ThemeEditor;
            }
            return AppReturn::Continue;
        };
    } else {
        if app.state.focus == Focus::SubmitButton {
            if app.state.theme_being_edited.name == Theme::default().name {
                app.send_error_toast("Theme name cannot be the Default theme name", None);
                return AppReturn::Continue;
            }
            app.state.popup_mode = Some(PopupMode::SaveThemePrompt);
        } else if app.state.focus == Focus::ThemeEditor
            && app.state.theme_editor_state.selected().is_some()
        {
            let selected_item_index = app.state.theme_editor_state.selected().unwrap();
            if selected_item_index == 0 {
                app.state.popup_mode = Some(PopupMode::EditGeneralConfig);
                app.state.current_user_input = String::new();
                app.state.current_cursor_position = None;
            } else {
                app.state.popup_mode = Some(PopupMode::ThemeEditor);
            }
        }
    }
    AppReturn::Continue
}

fn handle_go_to_prv_ui_mode(app: &mut App) {
    if app.state.prev_ui_mode.is_some() && app.state.prev_ui_mode.unwrap() != app.state.ui_mode {
        app.state.ui_mode = app.state.prev_ui_mode.unwrap();
    } else {
        app.state.ui_mode = UiMode::MainMenu;
    }
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
        app.state.focus = available_targets[0];
        return;
    }
    let next_focus = app.state.focus.next(&available_targets);
    if next_focus != app.state.focus && next_focus != Focus::NoFocus {
        app.state.focus = next_focus;
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
        app.state.focus = available_targets[available_targets.len() - 1];
        return;
    }
    let prv_focus = app.state.focus.prev(&available_targets);
    if prv_focus != app.state.focus && prv_focus != Focus::NoFocus {
        app.state.focus = prv_focus;
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
    app.state.popup_mode = None;
    handle_prv_focus(app);
}

fn handle_custom_rgb_prompt(app: &mut App, fg: bool) {
    if app.state.focus == Focus::TextInput {
        app.state.current_user_input = String::new();
        app.state.current_cursor_position = None;
        app.state.app_status = AppStatus::UserInput;
    } else if app.state.focus == Focus::SubmitButton {
        // check if the current_user_input is in the format x,y,z where x,y,z are three digit numbers from 0 to 255. use trim to remove whitespace if any
        let rgb_values: Vec<&str> = app
            .state
            .current_user_input
            .trim()
            .split(',')
            .collect::<Vec<&str>>();
        let rgb_values = rgb_values.iter().map(|x| x.trim()).collect::<Vec<&str>>();
        if rgb_values.len() != 3 {
            app.send_error_toast("Invalid RGB format. Please enter the RGB values in the format x,y,z where x,y,z are three digit numbers from 0 to 255", None);
            return;
        }
        let rgb_values = rgb_values
            .iter()
            .map(|x| x.parse::<u8>())
            .collect::<Result<Vec<u8>, _>>();
        if rgb_values.is_err() {
            app.send_error_toast("Invalid RGB format. Please enter the RGB values in the format x,y,z where x,y,z are three digit numbers from 0 to 255", None);
            return;
        }
        let rgb_values = rgb_values.unwrap();
        let all_color_options = TextColorOptions::to_iter().collect::<Vec<TextColorOptions>>();
        let selected_index = app.state.edit_specific_style_state.0.selected();
        if selected_index.is_none() {
            debug!("No selected index found");
            app.send_error_toast("Something went wrong", None);
        }
        let selected_index = selected_index.unwrap();
        if selected_index >= all_color_options.len() {
            debug!("Selected index is out of bounds");
            app.send_error_toast("Something went wrong", None);
        }
        let theme_style_bring_edited_index = app.state.theme_editor_state.selected();
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
            app.state.theme_being_edited.to_vec_str()[theme_style_bring_edited_index].clone();
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
        app.state.popup_mode = Some(PopupMode::ThemeEditor);
    }
}

fn handle_theme_maker_scroll_up(app: &mut App) {
    if app.state.focus == Focus::StyleEditorFG {
        let current_index = app.state.edit_specific_style_state.0.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state.edit_specific_style_state.0.select(Some(0));
        }
        let current_index = app.state.edit_specific_style_state.0.selected().unwrap();
        if current_index > 0 {
            app.state
                .edit_specific_style_state
                .0
                .select(Some(current_index - 1));
        } else {
            app.state
                .edit_specific_style_state
                .0
                .select(Some(total_length - 1));
        }
    } else if app.state.focus == Focus::StyleEditorBG {
        let current_index = app.state.edit_specific_style_state.1.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state.edit_specific_style_state.1.select(Some(0));
        }
        let current_index = app.state.edit_specific_style_state.1.selected().unwrap();
        if current_index > 0 {
            app.state
                .edit_specific_style_state
                .1
                .select(Some(current_index - 1));
        } else {
            app.state
                .edit_specific_style_state
                .1
                .select(Some(total_length - 1));
        }
    } else if app.state.focus == Focus::StyleEditorModifier {
        let current_index = app.state.edit_specific_style_state.2.selected();
        let total_length = TextModifierOptions::to_iter().count();
        if current_index.is_none() {
            app.state.edit_specific_style_state.2.select(Some(0));
        }
        let current_index = app.state.edit_specific_style_state.2.selected().unwrap();
        if current_index > 0 {
            app.state
                .edit_specific_style_state
                .2
                .select(Some(current_index - 1));
        } else {
            app.state
                .edit_specific_style_state
                .2
                .select(Some(total_length - 1));
        }
    }
}
fn handle_theme_maker_scroll_down(app: &mut App) {
    if app.state.focus == Focus::StyleEditorFG {
        let current_index = app.state.edit_specific_style_state.0.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state.edit_specific_style_state.0.select(Some(0));
        }
        let current_index = app.state.edit_specific_style_state.0.selected().unwrap();
        if current_index < total_length - 1 {
            app.state
                .edit_specific_style_state
                .0
                .select(Some(current_index + 1));
        } else {
            app.state.edit_specific_style_state.0.select(Some(0));
        }
    } else if app.state.focus == Focus::StyleEditorBG {
        let current_index = app.state.edit_specific_style_state.1.selected();
        let total_length = TextColorOptions::to_iter().count();
        if current_index.is_none() {
            app.state.edit_specific_style_state.1.select(Some(0));
        }
        let current_index = app.state.edit_specific_style_state.1.selected().unwrap();
        if current_index < total_length - 1 {
            app.state
                .edit_specific_style_state
                .1
                .select(Some(current_index + 1));
        } else {
            app.state.edit_specific_style_state.1.select(Some(0));
        }
    } else if app.state.focus == Focus::StyleEditorModifier {
        let current_index = app.state.edit_specific_style_state.2.selected();
        let total_length = TextModifierOptions::to_iter().count();
        if current_index.is_none() {
            app.state.edit_specific_style_state.2.select(Some(0));
        }
        let current_index = app.state.edit_specific_style_state.2.selected().unwrap();
        if current_index < total_length - 1 {
            app.state
                .edit_specific_style_state
                .2
                .select(Some(current_index + 1));
        } else {
            app.state.edit_specific_style_state.2.select(Some(0));
        }
    }
}
