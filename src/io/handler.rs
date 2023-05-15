use chrono::NaiveDate;
use eyre::{anyhow, Result};
use linked_hash_map::LinkedHashMap;
use log::{debug, error, info};
use ratatui::widgets::ListState;
use savefile::{load_file, save_file};
use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use super::data_handler::{get_available_local_savefiles, get_local_kanban_state};
use super::IoEvent;
use crate::{
    app::{kanban::Board, state::UiMode, App, AppConfig},
    constants::{CONFIG_DIR_NAME, CONFIG_FILE_NAME, SAVE_DIR_NAME, SAVE_FILE_NAME},
    io::data_handler::{
        get_default_save_directory, get_saved_themes, reset_config,
        save_kanban_state_locally,
    },
    ui::TextColorOptions,
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
            IoEvent::GetCloudData => self.get_cloud_save().await,
            IoEvent::Reset => self.reset_config().await,
            IoEvent::SaveLocalData => self.save_local_data().await,
            IoEvent::LoadSave => self.load_save_file().await,
            IoEvent::DeleteSave => self.delete_save_file().await,
            IoEvent::ResetVisibleBoardsandCards => self.refresh_visible_boards_and_cards().await,
            IoEvent::AutoSave => self.auto_save().await,
            IoEvent::LoadPreview => self.load_preview().await,
        };

        let mut app = self.app.lock().await;
        if let Err(err) = result {
            error!("Oops, something wrong happened 😢: {:?}", err);
            app.send_error_toast("Oops, something wrong happened 😢", None);
        }

        app.loaded();
    }

    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        let prepare_config_dir_status = prepare_config_dir();
        if prepare_config_dir_status.is_err() {
            error!("Cannot create config directory");
            app.send_error_toast("Cannot create config directory", None);
        }
        if !prepare_save_dir() {
            error!("Cannot create save directory");
            app.send_error_toast("Cannot create save directory", None);
        }
        app.boards = prepare_boards(&mut app);
        app.keybind_list_maker();
        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
        let saved_themes = get_saved_themes();
        if saved_themes.is_some() {
            app.all_themes.extend(saved_themes.unwrap());
        }
        let default_theme = app.config.default_theme.clone();
        for theme in &app.all_themes {
            if theme.name == default_theme {
                app.theme = theme.clone();
                break;
            }
        }
        let bg = app.theme.general_style.bg;
        if bg.is_some() {
            app.state.term_background_color = TextColorOptions::from(bg.unwrap()).to_rgb();
        } else {
            app.state.term_background_color = (0, 0, 0)
        }
        info!("👍 Application initialized");
        app.initialized(); // we could update the app state
        if app.config.save_directory == get_default_save_directory() {
            app.send_warning_toast(
                "Save directory is set to a temporary directory,
            your operating system may delete it at any time. Please change it in the settings.",
                Some(Duration::from_secs(10)),
            );
        }
        app.send_info_toast("Application initialized", None);
        Ok(())
    }

    async fn get_cloud_save(&mut self) -> Result<()> {
        info!("🚀 Getting cloud save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("👍 Cloud save loaded");
        app.send_info_toast("👍 Cloud save loaded", None);
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
        let mut app = self.app.lock().await;
        let board_data = &app.boards;
        let status = save_kanban_state_locally(board_data.to_vec());
        match status {
            Ok(_) => {
                info!("👍 Local data saved");
                app.send_info_toast("👍 Local data saved", None);
            }
            Err(err) => {
                debug!("Cannot save local data: {:?}", err);
                app.send_error_toast("Cannot save local data", None);
            }
        }
        Ok(())
    }

    async fn load_save_file(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let save_file_index = app.state.load_save_state.selected().unwrap_or(0);
        let local_files = get_available_local_savefiles();
        let local_files = if local_files.is_none() {
            error!("Could not get local save files");
            app.send_error_toast("Could not get local save files", None);
            vec![]
        } else {
            local_files.unwrap()
        };
        // check if the file exists
        if save_file_index >= local_files.len() {
            error!("Cannot load save file: No such file");
            app.send_error_toast("Cannot load save file: No such file", None);
            return Ok(());
        }
        let save_file_name = local_files[save_file_index].clone();
        let version = save_file_name.split("_v").collect::<Vec<&str>>();
        if version.len() < 2 {
            error!("Cannot load save file: invalid file name");
            app.send_error_toast("Cannot load save file: invalid file name", None);
            return Ok(());
        }
        // convert to u32
        let version = version[1].parse::<u32>();
        if version.is_err() {
            error!("Cannot load save file: invalid file name");
            app.send_error_toast("Cannot load save file: invalid file name", None);
            return Ok(());
        }
        info!("🚀 Loading save file: {}", save_file_name);
        let version = version.unwrap();
        let board_data = get_local_kanban_state(save_file_name.clone(), version, false);
        match board_data {
            Ok(boards) => {
                app.set_boards(boards);
                info!("👍 Save file {:?} loaded", save_file_name);
                app.send_info_toast(&format!("👍 Save file {:?} loaded", save_file_name), None);
            }
            Err(err) => {
                debug!("Cannot load save file: {:?}", err);
                app.send_error_toast("Cannot load save file", None);
            }
        }
        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
        app.state.ui_mode = app.config.default_view;
        Ok(())
    }

    async fn delete_save_file(&mut self) -> Result<()> {
        // get app.state.load_save_state.selected() and delete the file
        let mut app = self.app.lock().await;
        let file_list = get_available_local_savefiles();
        let file_list = if file_list.is_none() {
            error!("Cannot delete save file: no save files found");
            app.send_error_toast("Cannot delete save file: no save files found", None);
            return Ok(());
        } else {
            file_list.unwrap()
        };
        if app.state.load_save_state.selected().is_none() {
            error!("Cannot delete save file: no save file selected");
            app.send_error_toast("Cannot delete save file: no save file selected", None);
            return Ok(());
        }
        let selected = app.state.load_save_state.selected().unwrap_or(0);
        if selected >= file_list.len() {
            debug!("Cannot delete save file: index out of range");
            app.send_error_toast("Cannot delete save file: Something went wrong", None);
            return Ok(());
        }
        let file_name = file_list[selected].clone();
        info!("🚀 Deleting save file: {}", file_name);
        let path = app.config.save_directory.join(file_name);
        // check if the file exists
        if !Path::new(&path).exists() {
            error!("Cannot delete save file: file not found");
            app.send_error_toast("Cannot delete save file: file not found", None);
            return Ok(());
        } else {
            // delete the file
            if let Err(err) = std::fs::remove_file(&path) {
                debug!("Cannot delete save file: {:?}", err);
                app.send_error_toast("Cannot delete save file: Something went wrong", None);
                app.state.load_save_state = ListState::default();
                return Ok(());
            } else {
                info!("👍 Save file deleted");
                app.send_info_toast("👍 Save file deleted", None);
            }
        }
        // check if selected is still in range
        let file_list = get_available_local_savefiles();
        let file_list = if file_list.is_none() {
            app.state.load_save_state = ListState::default();
            return Ok(());
        } else {
            file_list.unwrap()
        };
        if selected >= file_list.len() {
            if file_list.is_empty() {
                app.state.load_save_state = ListState::default();
            } else {
                app.state.load_save_state.select(Some(file_list.len() - 1));
            }
        }
        Ok(())
    }

    async fn refresh_visible_boards_and_cards(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        refresh_visible_boards_and_cards(&mut app);
        Ok(())
    }

    async fn auto_save(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        auto_save(&mut app).await
    }

    async fn load_preview(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        if app.state.load_save_state.selected().is_none() {
            return Ok(());
        }
        app.state.preview_boards_and_cards = None;

        let save_file_index = app.state.load_save_state.selected().unwrap_or(0);
        let local_files = get_available_local_savefiles();
        let local_files = if local_files.is_none() {
            error!("Could not get local save files");
            app.send_error_toast("Could not get local save files", None);
            vec![]
        } else {
            local_files.unwrap()
        };
        // check if the file exists
        if save_file_index >= local_files.len() {
            error!("Cannot load preview: No such file");
            app.send_error_toast("Cannot load preview: No such file", None);
            return Ok(());
        }
        let save_file_name = local_files[save_file_index].clone();
        let version = save_file_name.split("_v").collect::<Vec<&str>>();
        if version.len() < 2 {
            error!("Cannot load preview: invalid file name");
            app.send_error_toast("Cannot load preview: invalid file name", None);
            return Ok(());
        }
        // convert to u32
        let version = version[1].parse::<u32>();
        if version.is_err() {
            error!("Cannot load preview: invalid file name");
            app.send_error_toast("Cannot load preview: invalid file name", None);
            return Ok(());
        }
        let version = version.unwrap();
        let board_data = get_local_kanban_state(save_file_name.clone(), version, true);
        match board_data {
            Ok(boards) => {
                app.state.preview_boards_and_cards = Some(boards);
                // get self.boards and make Vec<LinkedHashMap<u128, Vec<u128>>> of visible boards and cards
                let mut visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>> =
                    LinkedHashMap::new();
                for (counter, board) in app
                    .state
                    .preview_boards_and_cards
                    .as_ref()
                    .unwrap()
                    .iter()
                    .enumerate()
                {
                    if counter >= app.config.no_of_boards_to_show.into() {
                        break;
                    }
                    let mut visible_cards: Vec<u128> = Vec::new();
                    if board.cards.len() > app.config.no_of_cards_to_show.into() {
                        for card in board
                            .cards
                            .iter()
                            .take(app.config.no_of_cards_to_show.into())
                        {
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
                }
                app.state.preview_visible_boards_and_cards = visible_boards_and_cards;
                app.state.preview_file_name = Some(save_file_name);
            }
            Err(e) => {
                error!("Error loading preview: {}", e);
                app.send_error_toast("Error loading preview", None);
            }
        }
        Ok(())
    }
}

pub(crate) fn get_config_dir() -> Result<PathBuf, String> {
    let home_dir = home::home_dir();
    if home_dir.is_none() {
        return Err(String::from("Error getting home directory"));
    }
    let mut config_dir = home_dir.unwrap();
    // check if windows or unix
    if cfg!(windows) {
        config_dir.push("AppData");
        config_dir.push("Roaming");
    } else {
        config_dir.push(".config");
    }
    config_dir.push(CONFIG_DIR_NAME);
    Ok(config_dir)
}

pub(crate) fn get_save_dir() -> PathBuf {
    let mut save_dir = env::temp_dir();
    save_dir.push(SAVE_DIR_NAME);
    save_dir
}

pub fn prepare_config_dir() -> Result<(), String> {
    let config_dir = get_config_dir();
    if config_dir.is_err() {
        return Err(String::from("Error getting config directory"));
    }
    let config_dir = config_dir.unwrap();
    if !config_dir.exists() {
        let dir_creation_status = std::fs::create_dir_all(&config_dir);
        if dir_creation_status.is_err() {
            return Err(String::from("Error creating config directory"));
        }
    }
    // make config file if it doesn't exist and write default config to it
    let mut config_file = config_dir;
    config_file.push(CONFIG_FILE_NAME);
    if !config_file.exists() {
        let default_config = AppConfig::default();
        let config_json = serde_json::to_string_pretty(&default_config);
        if let Ok(config_json) = config_json {
            let file_creation_status = std::fs::write(&config_file, config_json);
            if file_creation_status.is_err() {
                return Err(String::from("Error creating config file"));
            }
        } else {
            return Err(String::from("Error creating config file"));
        }
    }
    Ok(())
}

fn prepare_save_dir() -> bool {
    let save_dir = get_save_dir();
    if !save_dir.exists() {
        std::fs::create_dir_all(&save_dir).unwrap();
    }
    true
}

fn prepare_boards(app: &mut App) -> Vec<Board> {
    if app.config.always_load_last_save {
        let latest_save_file_info = get_latest_save_file();
        if let Ok(latest_save_file_info) = latest_save_file_info {
            let latest_save_file = latest_save_file_info.0;
            let latest_version = latest_save_file_info.1;
            let local_data =
                get_local_kanban_state(latest_save_file.clone(), latest_version, false);
            match local_data {
                Ok(data) => {
                    info!("👍 Local data loaded from {:?}", latest_save_file);
                    app.send_info_toast(
                        &format!("👍 Local data loaded from {:?}", latest_save_file),
                        None,
                    );
                    data
                }
                Err(err) => {
                    debug!("Cannot get local data: {:?}", err);
                    error!("👎 Cannot get local data, Data might be corrupted or is not in the correct format");
                    app.send_error_toast("👎 Cannot get local data, Data might be corrupted or is not in the correct format", None);
                    vec![]
                }
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
    let local_save_files = if let Some(local_save_files) = local_save_files {
        local_save_files
    } else {
        return Err(anyhow!("No local save files found"));
    };
    let fall_back_version = -1;
    if local_save_files.is_empty() {
        return Err(anyhow!("No local save files found"));
    }
    let latest_date = local_save_files
        .iter()
        .map(|file| {
            let date = file.split('_').collect::<Vec<&str>>()[1];
            NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap()
        })
        .max()
        .unwrap();
    let latest_version = local_save_files
        .iter()
        .filter(|file| {
            let date = file.split('_').collect::<Vec<&str>>()[1];
            NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap() == latest_date
        })
        .map(|file| {
            let version = file.split("_v").collect::<Vec<&str>>()[1];
            version.parse::<i32>().unwrap_or(fall_back_version)
        })
        .max()
        .unwrap_or(fall_back_version);

    if latest_version == fall_back_version {
        return Err(anyhow!("No local save files found"));
    }
    let latest_version = latest_version as u32;

    let latest_save_file = format!(
        "kanban_{}_v{}",
        latest_date.format("%d-%m-%Y"),
        latest_version
    );
    Ok((latest_save_file, latest_version))
}

pub fn refresh_visible_boards_and_cards(app: &mut App) {
    let mut visible_boards_and_cards: LinkedHashMap<u128, Vec<u128>> = LinkedHashMap::new();
    let boards = if app.filtered_boards.is_empty() {
        app.boards.clone()
    } else {
        app.filtered_boards.clone()
    };
    for (i, board) in boards.iter().enumerate() {
        if (i) as u16 == app.config.no_of_boards_to_show {
            break;
        }
        let mut visible_cards: Vec<u128> = Vec::new();
        if board.cards.len() > app.config.no_of_cards_to_show.into() {
            for card in board
                .cards
                .iter()
                .take(app.config.no_of_cards_to_show.into())
            {
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
    }
    app.visible_boards_and_cards = visible_boards_and_cards;
    // if a board and card are there set it to current board and card
    if !app.visible_boards_and_cards.is_empty() {
        app.state.current_board_id = Some(*app.visible_boards_and_cards.keys().next().unwrap());
        if !app
            .visible_boards_and_cards
            .values()
            .next()
            .unwrap()
            .is_empty()
        {
            app.state.current_card_id =
                Some(app.visible_boards_and_cards.values().next().unwrap()[0]);
        }
    }
}

pub fn make_file_system_safe_name(name: &str) -> String {
    let mut safe_name = name.to_string();
    let unsafe_chars = vec!["/", "\\", ":", "*", "?", "\"", "<", ">", "|", " "];
    for unsafe_char in unsafe_chars {
        safe_name = safe_name.replace(unsafe_char, "");
    }
    safe_name
}

pub async fn auto_save(app: &mut App) -> Result<()> {
    let mut file_version = 0;
    let latest_save_file_info = get_latest_save_file();
    let save_required = if latest_save_file_info.is_ok() {
        let latest_save_file_info = latest_save_file_info.unwrap();
        let save_file_name = latest_save_file_info.0;
        file_version = latest_save_file_info.1;
        let file_path = app.config.save_directory.join(save_file_name);
        let boards: Vec<Board> = load_file(file_path, file_version)?;
        app.boards != boards
    } else {
        true
    };
    if save_required {
        let file_name = format!(
            "{}_{}_v{}",
            SAVE_FILE_NAME,
            chrono::Local::now().format("%d-%m-%Y"),
            file_version + 1
        );
        let file_path = app.config.save_directory.join(file_name);
        let save_status = save_file(file_path, file_version, &app.boards);
        match save_status {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Error saving file: {}", e)),
        }
    } else {
        Ok(())
    }
}
