use linked_hash_map::LinkedHashMap;
use savefile::{load_file, save_file};
use std::path::Path;
use std::{
    sync::Arc,
    path::PathBuf
};
use crate::app::kanban::Board;
use crate::app::state::UiMode;
use std::env;
use crate::constants::{
    CONFIG_DIR_NAME,
    CONFIG_FILE_NAME,
    SAVE_DIR_NAME,
    NO_OF_BOARDS_PER_PAGE,
    NO_OF_CARDS_PER_BOARD,
    SAVE_FILE_NAME
};
use crate::app::{
    AppConfig,
    App
};
use crate::io::data_handler::{
    reset_config,
    save_kanban_state_locally,
    get_config
};
use chrono::NaiveDate;
use eyre::{Result, anyhow};
use log::{
    error,
    info,
};

use super::IoEvent;
use super::data_handler::{
    get_available_local_savefiles,
    get_local_kanban_state
};

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::GetLocalData => self.get_local_save().await,
            IoEvent::GetCloudData => self.get_cloud_save().await,
            IoEvent::Reset => self.reset_config().await,
            IoEvent::SaveLocalData => self.save_local_data().await,
            IoEvent::LoadSave => self.load_save_file().await,
            IoEvent::DeleteSave => self.delete_save_file().await,
            IoEvent::GoRight => self.go_right().await,
            IoEvent::GoLeft => self.go_left().await,
            IoEvent::GoUp => self.go_up().await,
            IoEvent::GoDown => self.go_down().await,
            IoEvent::RefreshVisibleBoardsandCards => self.refresh_visible_boards_and_cards().await,
            IoEvent::AutoSave => self.auto_save().await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<()> {
        info!("???? Initialize the application");
        let mut app = self.app.lock().await;
        if !prepare_config_dir() {
            error!("Cannot create config directory");
        }
        if !prepare_save_dir() {
            error!("Cannot create save directory");
        }
        app.boards = prepare_boards(&mut app);
        app.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
        app.initialized(); // we could update the app state
        info!("???? Application initialized");
        Ok(())
    }

    async fn get_local_save(&mut self) -> Result<()> {
        info!("???? Getting local save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("???? Local save loaded");
        Ok(())
    }

    async fn get_cloud_save(&mut self) -> Result<()> {
        info!("???? Getting cloud save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("???? Cloud save loaded");
        Ok(())
    }

    async fn reset_config(&mut self) -> Result<()> {
        info!("???? Resetting config");
        reset_config();
        info!("???? Config reset");
        Ok(())
    }

    async fn save_local_data(&mut self) -> Result<()> {
        info!("???? Saving local data");
        let app = self.app.lock().await;
        let board_data = &app.boards;
        let status = save_kanban_state_locally(board_data.to_vec());
        match status {
            Ok(_) => info!("???? Local data saved"),
            Err(err) => error!("Cannot save local data: {:?}", err),
        }
        Ok(())
    }

    async fn load_save_file(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let save_file_index = app.state.load_save_state.selected().unwrap_or(0);
        let local_files = get_available_local_savefiles();
        // check if the file exists
        if save_file_index >= local_files.len() {
            error!("Cannot load save file: No such file");
            return Ok(());
        }
        let save_file_name = local_files[save_file_index].clone();
        let version = save_file_name.split("_v").collect::<Vec<&str>>();
        if version.len() < 2 {
            error!("Cannot load save file: invalid file name");
            return Ok(());
        }
        // convert to u32
        let version = version[1].parse::<u32>();
        if version.is_err() {
            error!("Cannot load save file: invalid file name");
            return Ok(());
        }
        info!("???? Loading save file: {}", save_file_name);
        let version = version.unwrap();
        let board_data = get_local_kanban_state(save_file_name.clone(), version);
        match board_data {
            Ok(boards) => {
                app.set_boards(boards);
                info!("???? Save file {:?} loaded", save_file_name);
            }
            Err(err) => error!("Cannot load save file: {:?}", err),
        }
        app.dispatch(IoEvent::RefreshVisibleBoardsandCards).await;
        Ok(())
    }

    async fn delete_save_file(&mut self) -> Result<()> {
        info!("???? Deleting save file");
        // get app.state.load_save_state.selected() and delete the file
        let app = self.app.lock().await;
        let file_list = get_available_local_savefiles();
        let selected = app.state.load_save_state.selected().unwrap_or(0);
        if selected >= file_list.len() {
            error!("Cannot delete save file: index out of range");
            return Ok(());
        }
        let file_name = file_list[selected].clone();
        let config = get_config();
        let path = config.save_directory.join(file_name);
        // check if the file exists
        if !Path::new(&path).exists() {
            error!("Cannot delete save file: file not found");
            return Ok(());
        } else {
            // delete the file
            if let Err(err) = std::fs::remove_file(&path) {
                error!("Cannot delete save file: {:?}", err);
                return Ok(());
            } else {
                info!("???? Save file deleted");
            }
        }
        Ok(())
    }

    async fn go_right(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let current_visible_boards = app.visible_boards_and_cards.clone();
        let all_boards = app.boards.clone();
        let current_board_id = app.state.current_board_id;
        // check if current_board_id is set, if not assign to the first board
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
            error!("Cannot go right: current board not found");
            return Ok(());
        }
        let current_board_index = current_board_index.unwrap();
        if current_board_index == current_visible_boards.len() - 1 {
            // we are at the last board, check the index for the current board in all boards, if it is the last one, we cannot go right
            let current_board_index_in_all_boards = all_boards
                .iter()
                .position(|board| board.id == current_board_id);
            if current_board_index_in_all_boards.is_none() {
                error!("Cannot go right: current board not found");
                return Ok(());
            }
            let current_board_index_in_all_boards = current_board_index_in_all_boards.unwrap();
            if current_board_index_in_all_boards == all_boards.len() - 1 {
                // we are at the last board, we cannot go right
                return Ok(());
            }
            // we are not at the last board, we can go right
            // get the next NO_OF_BOARDS_PER_PAGE boards
            let next_board_index = current_board_index_in_all_boards + 1;
            let next_board_index = if (next_board_index + NO_OF_BOARDS_PER_PAGE as usize) > all_boards.len() {
                all_boards.len() - NO_OF_BOARDS_PER_PAGE as usize
            } else {
                next_board_index
            };
            let next_boards = all_boards[next_board_index..next_board_index + NO_OF_BOARDS_PER_PAGE as usize].to_vec();
            let mut visible_boards_and_cards = LinkedHashMap::new();
            for board in &next_boards {
                let card_ids = board.cards.iter().map(|card| card.id).collect::<Vec<u128>>();
                visible_boards_and_cards.insert(board.id, card_ids);
            }
            app.visible_boards_and_cards = visible_boards_and_cards;
            // check if the current board is in the next boards, if not, set the current board to the first board in the next boards
            if !next_boards.iter().any(|board| board.id == current_board_id) {
                app.state.current_board_id = Some(next_boards[0].id);
            }
            // reset the current card id
            app.state.current_card_id = None;
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
        Ok(())
    }

    async fn go_left(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let current_visible_boards = app.visible_boards_and_cards.clone();
        let all_boards = app.boards.clone();
        let current_board_id = app.state.current_board_id;
        // check if current_board_id is set, if not assign to the first board
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
            error!("Cannot go left: current board not found");
            return Ok(());
        }
        let current_board_index = current_board_index.unwrap();
        if current_board_index == 0 {
            // we are at the first board, check the index for the current board in all boards, if it is the first one, we cannot go left
            let current_board_index_in_all_boards = all_boards
                .iter()
                .position(|board| board.id == current_board_id);
            if current_board_index_in_all_boards.is_none() {
                error!("Cannot go left: current board not found");
                return Ok(());
            }
            let current_board_index_in_all_boards = current_board_index_in_all_boards.unwrap();
            if current_board_index_in_all_boards == 0 {
                // we are at the first board, we cannot go left
                return Ok(());
            }
            // we are not at the first board, we can go left
            // get the previous NO_OF_BOARDS_PER_PAGE boards
            let previous_board_index = current_board_index_in_all_boards - 1;
            let previous_board_index = if previous_board_index < NO_OF_BOARDS_PER_PAGE as usize {
                0
            } else {
                previous_board_index - NO_OF_BOARDS_PER_PAGE as usize
            };
            let previous_boards = all_boards[previous_board_index..previous_board_index + NO_OF_BOARDS_PER_PAGE as usize].to_vec();
            let mut visible_boards_and_cards = LinkedHashMap::new();
            for board in &previous_boards {
                let card_ids = board.cards.iter().map(|card| card.id).collect::<Vec<u128>>();
                visible_boards_and_cards.insert(board.id, card_ids);
            }
            app.visible_boards_and_cards = visible_boards_and_cards;
            // check if the current board is in the previous boards, if not, set the current board to the last board in the previous boards
            if !previous_boards.iter().any(|board| board.id == current_board_id) {
                app.state.current_board_id = Some(previous_boards[previous_boards.len() - 1].id);
            }
            // reset the current card id
            app.state.current_card_id = None;
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
        Ok(())
    }

    async fn go_up(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        // up and down is for cards
        let current_visible_boards = app.visible_boards_and_cards.clone();
        let current_board_id = app.state.current_board_id;
        let current_card_id = app.state.current_card_id;
        let current_board_id = if current_board_id.is_none() {
            app.boards[0].id
        } else {
            current_board_id.unwrap()
        };
        let current_card_id = if current_card_id.is_none() {
            // get the first card of the current board
            let current_board = app.boards.iter().find(|board| board.id == current_board_id);
            if current_board.is_none() {
                error!("Cannot go up: current board not found");
                return Ok(());
            }
            let current_board = current_board.unwrap();
            if current_board.cards.is_empty() {
                error!("Cannot go up: current board has no cards");
                return Ok(());
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
            error!("Cannot go up: current card not found");
            return Ok(());
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
                error!("Cannot go up: current card not found");
                return Ok(());
            }
            let current_card_index_in_all_cards = current_card_index_in_all_cards.unwrap();
            if current_card_index_in_all_cards == 0 {
                // we are at the first card, we cannot go up
                return Ok(());
            }
            // we are not at the first card, we can go up
            // get the previous NO_OF_CARDS_PER_PAGE cards
            let previous_card_index = current_card_index_in_all_cards - 1;
            let previous_card_index = if previous_card_index < NO_OF_CARDS_PER_BOARD as usize {
                0
            } else {
                previous_card_index - NO_OF_CARDS_PER_BOARD as usize
            };
            let previous_cards = app
                .boards
                .iter()
                .find(|board| board.id == current_board_id)
                .unwrap()
                .cards[previous_card_index..previous_card_index + NO_OF_CARDS_PER_BOARD as usize]
                .to_vec();
            let mut visible_boards_and_cards = app.visible_boards_and_cards.clone();
            // replace the cards of the current board
            visible_boards_and_cards.entry(current_board_id).and_modify(|cards| {
                *cards = previous_cards.iter().map(|card| card.id).collect::<Vec<u128>>()
            });
            app.visible_boards_and_cards = visible_boards_and_cards;
            app.state.current_card_id = Some(previous_cards[NO_OF_CARDS_PER_BOARD as usize - 1].id);
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
                error!("Cannot go up: previous card not found");
                return Ok(());
            } else {
                app.state.current_card_id = Some(previous_card_id);
            }
        }
        Ok(())
    }

    async fn go_down(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        // up and down is for cards
        let current_visible_boards = app.visible_boards_and_cards.clone();
        let current_board_id = app.state.current_board_id;
        let current_card_id = app.state.current_card_id;
        let current_board_id = if current_board_id.is_none() {
            app.boards[0].id
        } else {
            current_board_id.unwrap()
        };
        let current_card_id = if current_card_id.is_none() {
            // get the first card of the current board
            let current_board = app.boards.iter().find(|board| board.id == current_board_id);
            if current_board.is_none() {
                error!("Cannot go down: current board not found");
                return Ok(());
            }
            let current_board = current_board.unwrap();
            if current_board.cards.is_empty() {
                error!("Cannot go down: current board has no cards");
                return Ok(());
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
            error!("Cannot go down: current card not found");
            return Ok(());
        }
        let current_card_index = current_card_index.unwrap();
        if current_card_index == NO_OF_CARDS_PER_BOARD as usize - 1 {
            let current_card_index_in_all_cards = app
                .boards
                .iter()
                .find(|board| board.id == current_board_id)
                .unwrap()
                .cards
                .iter()
                .position(|card| card.id == current_card_id);
            if current_card_index_in_all_cards.is_none() {
                error!("Cannot go down: current card not found");
                return Ok(());
            }
            let current_card_index_in_all_cards = current_card_index_in_all_cards.unwrap();
            if current_card_index_in_all_cards == app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards.len() - 1 {
                // we are at the last card, we cannot go down
                return Ok(());
            }
            // we are not at the last card, we can go down
            // get the next NO_OF_CARDS_PER_PAGE cards
            let next_card_index = current_card_index_in_all_cards + 1;
            let next_card_index = if next_card_index + NO_OF_CARDS_PER_BOARD as usize > app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards.len() {
                app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards.len() - NO_OF_CARDS_PER_BOARD as usize
            } else {
                next_card_index
            };
            let next_cards = app
                .boards
                .iter()
                .find(|board| board.id == current_board_id)
                .unwrap()
                .cards[next_card_index..next_card_index + NO_OF_CARDS_PER_BOARD as usize]
                .to_vec();
            let mut visible_boards_and_cards = app.visible_boards_and_cards.clone();
            // replace the cards of the current board
            visible_boards_and_cards.entry(current_board_id).and_modify(|cards| {
                *cards = next_cards.iter().map(|card| card.id).collect::<Vec<u128>>()
            });
            app.visible_boards_and_cards = visible_boards_and_cards;
            app.state.current_card_id = Some(next_cards[0].id);
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
                return Ok(());
            } else {
                app.state.current_card_id = Some(next_card_id);
            }
        }
        Ok(())
    }

    async fn refresh_visible_boards_and_cards(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let mut counter = 0;
        // get self.boards and make Vec<LinkedHashMap<u128, Vec<u128>>> of visible boards and cards
        let mut visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>> = LinkedHashMap::new();
        for board in &app.boards {
            if counter == NO_OF_BOARDS_PER_PAGE {
                break;
            }
            let mut visible_cards: Vec<u128> = Vec::new();
            if board.cards.len() > NO_OF_CARDS_PER_BOARD.into() {
                for card in board.cards.iter().take(NO_OF_CARDS_PER_BOARD.into()) {
                    visible_cards.push(card.id);
                }
            } else {
                for card in &board.cards {
                    visible_cards.push(card.id);
                }
            }

            let mut visible_board: LinkedHashMap<u128, Vec<u128>> = LinkedHashMap::new();
            visible_board.insert(board.id, visible_cards);
            visible_boards_and_cards.extend(visible_board);
            counter += 1;
        }
        app.visible_boards_and_cards = visible_boards_and_cards;
        // check if current_board_id and current_card_id are still valid if not chack if current_board_id is still valid and
        // set current_card_id to the first card of the current board, else set current_board_id to the first board and
        // current_card_id to the first card of the current board if there are any boards and cards
        let current_board_id = app.state.current_board_id;
        let current_card_id = app.state.current_card_id;
        if current_board_id.is_none() {
            // set current_board_id to the first board
            if app.boards.is_empty() {
                // there are no boards
                app.state.current_board_id = None;
                app.state.current_card_id = None;
            } else {
                // there are boards
                app.state.current_board_id = Some(app.boards[0].id);
                // set current_card_id to the first card of the current board
                if app.boards[0].cards.is_empty() {
                    // there are no cards
                    app.state.current_card_id = None;
                } else {
                    // there are cards
                    app.state.current_card_id = Some(app.boards[0].cards[0].id);
                }
            }
        } else {
            // current_board_id is not None
            let current_board_id = current_board_id.unwrap();
            if app.visible_boards_and_cards.iter().find(|board_card_tuple| *board_card_tuple.0 == current_board_id).is_none() {
                // current_board_id is not valid
                // set current_board_id to the first board
                if app.boards.is_empty() {
                    // there are no boards
                    app.state.current_board_id = None;
                    app.state.current_card_id = None;
                } else {
                    // there are boards
                    app.state.current_board_id = Some(app.boards[0].id);
                    // set current_card_id to the first card of the current board
                    if app.boards[0].cards.is_empty() {
                        // there are no cards
                        app.state.current_card_id = None;
                    } else {
                        // there are cards
                        app.state.current_card_id = Some(app.boards[0].cards[0].id);
                    }
                }
            } else {
                // current_board_id is valid
                if current_card_id.is_none() {
                    // set current_card_id to the first card of the current board
                    if app.visible_boards_and_cards.iter().find(|board_card_tuple| *board_card_tuple.0 == current_board_id).unwrap().1.is_empty() {
                        // there are no cards
                        app.state.current_card_id = None;
                    } else {
                        // there are cards
                        app.state.current_card_id = Some(app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards[0].id);
                    }
                } else {
                    // current_card_id is not None
                    let current_card_id = current_card_id.unwrap();
                    if app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards.iter().find(|card| card.id == current_card_id).is_none() {
                        // current_card_id is not valid
                        // set current_card_id to the first card of the current board
                        if app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards.is_empty() {
                            // there are no cards
                            app.state.current_card_id = None;
                        } else {
                            // there are cards
                            app.state.current_card_id = Some(app.boards.iter().find(|board| board.id == current_board_id).unwrap().cards[0].id);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    async fn auto_save(&mut self) -> Result<()> {
        let app = self.app.lock().await;
        let latest_save_file_info = get_latest_save_file();
        let config = get_config();
        if latest_save_file_info.is_ok() {
            let latest_save_file_info = latest_save_file_info.unwrap();
            let default_board = Board::new(String::from("Board not found"), String::from("Board not found"));
            let save_file_name = latest_save_file_info.0;
            let version = latest_save_file_info.1;
            let file_path = config.save_directory.join(save_file_name);
            let boards: Vec<Board> = load_file(&file_path, version)?;

            // check if boards are the same compare the length of the boards and the length of the cards of each board
            if boards.len() == app.boards.len() {
                let mut boards_are_the_same = true;
                for board in &boards {
                    let board_id = board.id;
                    let board_cards = &board.cards;
                    let app_board = app.boards.iter().find(|board| board.id == board_id)
                        .unwrap_or_else(|| {
                            info!("board with id {} not found", board_id);
                            &default_board
                        });
                    // check if Board not found is returned
                    if app_board.id == default_board.id {
                        boards_are_the_same = false;
                        break;
                    }
                    let app_board_cards = &app_board.cards;
                    // compare the boards to check if the cards are the same by checking the id of the cards
                    if board_cards.len() != app_board_cards.len() {
                        boards_are_the_same = false;
                        break;
                    }
                    for card in board_cards {
                        let card_id = card.id;
                        if app_board_cards.iter().find(|card| card.id == card_id).is_none() {
                            boards_are_the_same = false;
                            break;
                        }
                    }
                }
                if boards_are_the_same {
                    return Ok(());
                } else {
                    let file_name = format!("{}_{}_v{}", SAVE_FILE_NAME, chrono::Local::now().format("%d-%m-%Y"), version + 1);
                    let file_path = config.save_directory.join(file_name);
                    let save_status = save_file(file_path, version, &app.boards);
                    match save_status {
                        Ok(_) => Ok(()),
                        Err(e) => Err(anyhow!("Error saving file: {}", e)),
                    }
                }
            } else {
                // boards are not the same
                let file_name = format!("{}_{}_v{}", SAVE_FILE_NAME, chrono::Local::now().format("%d-%m-%Y"), version + 1);
                let file_path = config.save_directory.join(file_name);
                let save_status = save_file(file_path, version, &app.boards);
                match save_status {
                    Ok(_) => Ok(()),
                    Err(e) => Err(anyhow!("Error saving file: {}", e)),
                }
            }
        } else {
            // there is no save file
            let file_name = format!("{}_{}_v{}", SAVE_FILE_NAME, chrono::Local::now().format("%d-%m-%Y"), 1);
            let file_path = config.save_directory.join(file_name);
            let save_status = save_file(file_path, 1, &app.boards);
            match save_status {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Error saving file: {}", e)),
            }
        }
    }

}

pub(crate) fn get_config_dir() -> PathBuf {
    let mut config_dir = home::home_dir().unwrap();
    config_dir.push(".config");
    config_dir.push(CONFIG_DIR_NAME);
    config_dir
}

pub(crate) fn get_save_dir() -> PathBuf {
    let mut save_dir = env::temp_dir();
    save_dir.push(SAVE_DIR_NAME);
    save_dir
}

fn prepare_config_dir() -> bool {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).unwrap();
    }
    // make config file if it doesn't exist and write default config to it
    let mut config_file = config_dir.clone();
    config_file.push(CONFIG_FILE_NAME);
    if !config_file.exists() {
        let default_config = AppConfig::default();
        let config_json = serde_json::to_string_pretty(&default_config).unwrap();
        std::fs::write(&config_file, config_json).unwrap();
    }
    true
}

fn prepare_save_dir() -> bool {
    let save_dir = get_save_dir();
    if !save_dir.exists() {
        std::fs::create_dir_all(&save_dir).unwrap();
    }
    true
}

fn prepare_boards (app: &mut App) -> Vec<Board> {
    let config = get_config();
    if config.always_load_latest_save {
        let latest_save_file_info = get_latest_save_file();
        if latest_save_file_info.is_ok() {
            let latest_save_file_info = latest_save_file_info.unwrap();
            let latest_save_file = latest_save_file_info.0;
            let latest_version = latest_save_file_info.1;
            let local_data = get_local_kanban_state(latest_save_file.clone(), latest_version);
            match local_data {
                Ok(data) => {
                    info!("???? Local data loaded from {:?}", latest_save_file);
                    data
                },
                Err(err) => {
                    error!("Cannot get local data: {:?}", err);
                    vec![]
                },
            }
        } else {
            vec![]
        }
    } else {
        app.set_ui_mode(UiMode::LoadSave);
        vec![]
    }
}

// return save file name and the latest verison
fn get_latest_save_file() -> Result<(String, u32)> {
    let local_save_files = get_available_local_savefiles();
    let fall_back_version = "1".to_string();
    // if local_save_files is empty, return empty vec
    if local_save_files.is_empty() {
        return Err(anyhow!("No local save files found"));
    }
    let latest_date = local_save_files
        .iter()
        .map(|file| {
            let date = file.split("_").collect::<Vec<&str>>()[1];
            let date = NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap();
            date
        })
        .max()
        .unwrap();
    let latest_version = local_save_files
        .iter()
        .filter(|file| {
            let date = file.split("_").collect::<Vec<&str>>()[1];
            let date = NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap();
            date == latest_date
        })
        .map(|file| {
            let version = file.split("_v").collect::<Vec<&str>>()[1];
            version.to_string()
        })
        .max()
        .unwrap_or(fall_back_version);
    let latest_version = latest_version.parse::<u32>().unwrap_or(1);
    let latest_save_file = format!("kanban_{}_v{}", latest_date.format("%d-%m-%Y"), latest_version);
    Ok((latest_save_file, latest_version))
}