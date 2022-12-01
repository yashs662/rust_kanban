use std::collections::BTreeMap;
use std::path::Path;
use std::{sync::Arc, path::PathBuf};
use crate::app::kanban::Board;
use std::env;
use crate::constants::{
    CONFIG_DIR_NAME,
    CONFIG_FILE_NAME,
    SAVE_DIR_NAME,
    NO_OF_BOARDS_PER_PAGE,
    NO_OF_CARDS_PER_BOARD
};
use crate::app::AppConfig;
use crate::io::data_handler::{reset_config, save_kanban_state_locally, get_config};
use eyre::Result;
use log::{error, info};

use super::IoEvent;
use super::data_handler::{get_available_local_savefiles, get_local_kanban_state};
use crate::app::App;

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
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        if !prepare_config_dir() {
            error!("Cannot create config directory");
        }
        if !prepare_save_dir() {
            error!("Cannot create save directory");
        }
        app.boards = prepare_boards();
        app.set_visible_boards_and_cards_on_load();
        app.initialized(); // we could update the app state
        info!("👍 Application initialized");
        Ok(())
    }

    async fn get_local_save(&mut self) -> Result<()> {
        info!("🚀 Getting local save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("👍 Local save loaded");
        Ok(())
    }

    async fn get_cloud_save(&mut self) -> Result<()> {
        info!("🚀 Getting cloud save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("👍 Cloud save loaded");
        Ok(())
    }

    async fn reset_config(&mut self) -> Result<()> {
        info!("🚀 Resetting config");
        reset_config();
        info!("👍 Config reset");
        Ok(())
    }

    async fn save_local_data(&mut self) -> Result<()> {
        info!("🚀 Saving local data");
        let app = self.app.lock().await;
        let board_data = &app.boards;
        let status = save_kanban_state_locally(board_data.to_vec());
        match status {
            Ok(_) => info!("👍 Local data saved"),
            Err(err) => error!("Cannot save local data: {:?}", err),
        }
        Ok(())
    }

    async fn load_save_file(&mut self) -> Result<()> {
        info!("🚀 Loading save file");
        let mut app = self.app.lock().await;
        let save_file_index = app.state.load_save_state.selected().unwrap_or(0);
        let local_files = get_available_local_savefiles();
        // check if the file exists
        if save_file_index >= local_files.len() {
            error!("Cannot load save file: index out of range");
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
        let version = version.unwrap();
        let board_data = get_local_kanban_state(save_file_name.clone(), version);
        match board_data {
            Ok(boards) => {
                app.set_boards(boards);
                info!("👍 Save file {:?} loaded", save_file_name);
            }
            Err(err) => error!("Cannot load save file: {:?}", err),
        }
        Ok(())
    }

    async fn delete_save_file(&mut self) -> Result<()> {
        info!("🚀 Deleting save file");
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
                info!("👍 Save file deleted");
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
        // check if the current board is the last one in visible_boards which is a btreemap of board_id and card_ids
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
                error!("Cannot go right: we are at the last board");
                return Ok(());
            }
            // we are not at the last board, we can go right
            // get the next NO_OF_BOARDS_PER_PAGE boards
            let next_board_index = current_board_index_in_all_boards + 1;
            let next_board_index = if next_board_index + NO_OF_BOARDS_PER_PAGE as usize > all_boards.len() {
                all_boards.len() - NO_OF_BOARDS_PER_PAGE as usize
            } else {
                next_board_index
            };
            let next_boards = all_boards[next_board_index..next_board_index + NO_OF_BOARDS_PER_PAGE as usize].to_vec();
            let mut visible_boards_and_cards = BTreeMap::new();
            for board in &next_boards {
                let card_ids = board.cards.iter().map(|card| card.id).collect::<Vec<u128>>();
                visible_boards_and_cards.insert(board.id, card_ids);
            }
            app.visible_boards_and_cards = visible_boards_and_cards;
            app.state.current_board_id = Some(next_boards[0].id);
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
            // reset the current card id
            app.state.current_card_id = None;
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
        // check if the current board is the first one in visible_boards which is a btreemap of board_id and card_ids
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
                error!("Cannot go left: we are at the first board");
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
            let mut visible_boards_and_cards = BTreeMap::new();
            for board in &previous_boards {
                let card_ids = board.cards.iter().map(|card| card.id).collect::<Vec<u128>>();
                visible_boards_and_cards.insert(board.id, card_ids);
            }
            app.visible_boards_and_cards = visible_boards_and_cards;
            app.state.current_board_id = Some(previous_boards[NO_OF_BOARDS_PER_PAGE as usize - 1].id);
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
            // reset the current card id
            app.state.current_card_id = None;
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
                error!("Cannot go up: we are at the first card");
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
            visible_boards_and_cards.insert(current_board_id, previous_cards.iter().map(|card| card.id).collect::<Vec<u128>>());
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
                .unwrap()
                .clone();
            app.state.current_card_id = Some(previous_card_id);
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
                error!("Cannot go down: we are at the last card");
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
            visible_boards_and_cards.insert(current_board_id, next_cards.iter().map(|card| card.id).collect::<Vec<u128>>());
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
                .unwrap()
                .clone();
            app.state.current_card_id = Some(next_card_id);
        }
        Ok(())
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

fn prepare_boards () -> Vec<Board> {
    let mut local_save_files = get_available_local_savefiles();
    // keep only the files which have _v 
    local_save_files = local_save_files
        .iter()
        .filter(|file| file.contains("_v"))
        .map(|file| file.to_string())
        .collect();
    let fall_back_version = "1".to_string();
    // parse file naming scheme to gett the latest file with the latest version
    // kanban_DD-MM-YYYY_V{version}
    // if local_save_files is empty, return empty vec
    if local_save_files.is_empty() {
        return vec![];
    }
    let mut latest_save_file = local_save_files[0].clone();
    let mut latest_version = fall_back_version.clone();
    for save_file in local_save_files {
        let save_file_name = &save_file;
        let version = save_file_name.split("_v").collect::<Vec<&str>>()[1].to_string();
        if version > latest_version {
            latest_version = version.to_string();
            latest_save_file = save_file;
        }
    }
    // get v1, v2 version number from latest_version
    let mut version_number = latest_version.split("v").collect::<Vec<&str>>();
    let last_version_number = version_number.pop().unwrap_or("1");
    let last_version_number = last_version_number.parse::<u32>().unwrap_or(1);
    let local_data = get_local_kanban_state(latest_save_file.clone(), last_version_number);
    match local_data {
        Ok(data) => {
            info!("👍 Local data loaded from {:?}", latest_save_file);
            data
        },
        Err(err) => {
            error!("Cannot get local data: {:?}", err);
            info!("👍 Local data loaded from default");
            vec![Board::default()]
        },
    }
}