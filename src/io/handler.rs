use std::path::Path;
use std::{sync::Arc, path::PathBuf};
use crate::app::kanban::Board;
use std::env;
use crate::constants::{
    CONFIG_DIR_NAME,
    CONFIG_FILE_NAME, SAVE_DIR_NAME
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
        app.set_visible_boards_and_cards();
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