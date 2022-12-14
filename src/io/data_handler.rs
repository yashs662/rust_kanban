use std::{
    fs,
    env
};
use log::{
    error,
    info
};
extern crate savefile;
use regex::Regex;
use savefile::prelude::*;


use crate::{
    app::{
        AppConfig,
        state::UiMode,
        kanban::Board
    },
    constants::{
        SAVE_DIR_NAME,
        SAVE_FILE_NAME
    }
};
use super::handler::get_config_dir;
use crate::constants::CONFIG_FILE_NAME;

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
    info!("Loading local save file: {:?}", file_path);
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
        savefiles.push(file_name);
    }
    // keep only the files which have follow the pattern SAVEFILE_NAME_<NaiveDate in format DD-MM-YYYY>_v<version number>
    // example kanban_02-12-2022_v7
    // use regex to match the pattern
    let re = Regex::new(r"^kanban_\d{2}-\d{2}-\d{4}_v\d+$").unwrap();
    savefiles.retain(|file| re.is_match(file));
    // order the files by date and version
    savefiles.sort_by(|a, b| {
        let a_date = a.split("_").nth(1).unwrap();
        let b_date = b.split("_").nth(1).unwrap();
        let a_version = a.split("_").nth(2).unwrap();
        let b_version = b.split("_").nth(2).unwrap();
        let a_date = chrono::NaiveDate::parse_from_str(a_date, "%d-%m-%Y").unwrap();
        let b_date = chrono::NaiveDate::parse_from_str(b_date, "%d-%m-%Y").unwrap();
        let a_version = a_version.split("v").nth(1).unwrap().parse::<u32>().unwrap();
        let b_version = b_version.split("v").nth(1).unwrap().parse::<u32>().unwrap();
        if a_date > b_date {
            std::cmp::Ordering::Greater
        } else if a_date < b_date {
            std::cmp::Ordering::Less
        } else {
            if a_version > b_version {
                std::cmp::Ordering::Greater
            } else if a_version < b_version {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        }
    });
    // reverse the order so that the most recent save file is first
    savefiles.reverse();
    savefiles
}