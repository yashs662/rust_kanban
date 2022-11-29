use std::{fs, env};
use log::{error, info, debug};
extern crate savefile;
use savefile::prelude::*;


use crate::{app::{
    AppConfig,
    state::UiMode, kanban::Board
}, constants::{SAVE_DIR_NAME, SAVE_FILE_NAME}};
use super::handler::get_config_dir;
use crate::constants::{
    CONFIG_FILE_NAME
};

pub fn get_config() -> AppConfig {
    let config_path = get_config_dir().join(CONFIG_FILE_NAME);
    let config = match fs::read_to_string(config_path) {
        Ok(config) => AppConfig{
            // if config file has been found, parse it, if an error occurs, use default config and write it to file
            ..serde_json::from_str(&config).unwrap_or_else(|e| {
                error!("Error parsing config file: {}", e);
                write_config(&AppConfig::default());
                AppConfig::default()
            })
        },
        Err(_) => {
            // if config file has not been found, use default config and write it to file
            let config = AppConfig::default();
            write_config(&config);
            AppConfig::default()
        }
    };
    config
}

pub fn write_config(config: &AppConfig) {
    let config_str = serde_json::to_string(&config).unwrap();
    fs::write(get_config_dir().join(CONFIG_FILE_NAME), config_str).unwrap();
    info!("Config file written");
}

pub fn get_default_ui_mode() -> UiMode {
    let config = get_config();
    config.default_view
}

pub fn reset_config() {
    let config = AppConfig::default();
    write_config(&config);
}

pub fn save_kanban_state_locally(boards: Vec<Board>) -> Result<(), SavefileError> {
    let config = get_config();
    // check config.save_directory for previous versions of the boards
    // versioning style is: SAVE_FILE_NAME_27-12-2020_v1
    // if the file exists, increment the version number
    // if the file does not exist, version number is 1
    let files = fs::read_dir(&config.save_directory)?;
    let mut version = 1;
    for file in files {
        let file = file?;
        let file_name = file.file_name().into_string().unwrap();
        if file_name.contains(SAVE_FILE_NAME) {
            version += 1;
        }
    }
    let file_name = format!("{}_{}_v{}", SAVE_FILE_NAME, chrono::Local::now().format("%d-%m-%Y"), version);
    let file_path = config.save_directory.join(file_name);
    let save_status = save_file(file_path, version, &boards);
    match save_status {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

pub fn get_local_kanban_state(file_name: String, version: u32) -> Result<Vec<Board>, SavefileError> {
    let config = get_config();
    let file_path = config.save_directory.join(file_name);
    debug!("Loading local save file: {:?}", file_path);
    let boards = load_file(file_path, version)?;
    Ok(boards)
}

pub fn get_available_local_savefiles<'a>() -> Vec<String> {
    let config = get_config();
    let files = fs::read_dir(&config.save_directory).unwrap_or_else(|e| {
        error!("Error reading save directory: {}", e);
        info!("reverting to Default save directory");
        let default_save_dir = env::temp_dir();
        let status = default_save_dir.join(SAVE_DIR_NAME);
        match fs::create_dir(&status) {
            Ok(_) => info!("Default save directory created"),
            Err(e) => error!("Error creating default save directory: {}", e)
        }
        fs::read_dir(&status).unwrap()
    });
    let mut savefiles = Vec::new();
    for file in files {
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        if file_name.contains(SAVE_FILE_NAME) {
            savefiles.push(file_name);
        }
    }
    savefiles.retain(|file_name| {
        file_name.starts_with(SAVE_FILE_NAME) && file_name.contains("_v")
    });
    savefiles
}