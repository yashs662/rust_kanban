use log::{debug, error, info};
use std::{collections::HashMap, env, fs};
extern crate savefile;
use regex::Regex;
use savefile::prelude::*;
use serde::Serialize;

use super::handler::get_config_dir;
use crate::constants::CONFIG_FILE_NAME;
use crate::{
    app::{kanban::Board, state::UiMode, AppConfig},
    constants::{SAVE_DIR_NAME, SAVE_FILE_NAME},
    inputs::key::Key,
    io::handler::prepare_config_dir,
};

pub fn get_config(ignore_overlapped_keybinds: bool) -> Result<AppConfig, String> {
    let config_dir_status = get_config_dir();
    if config_dir_status.is_err() {
        return Err(config_dir_status.unwrap_err());
    }
    let config_dir = config_dir_status.unwrap();
    let config_path = config_dir.join(CONFIG_FILE_NAME);
    let config = match fs::read_to_string(config_path) {
        Ok(config) => AppConfig {
            // if config file has been found, parse it, if an error occurs, use default config and write it to file
            ..serde_json::from_str(&config).unwrap_or_else(|e| {
                error!("Error parsing config file: {}", e);
                let write_config_status = write_config(&AppConfig::default());
                if write_config_status.is_err() {
                    error!("{}", write_config_status.unwrap_err());
                }
                AppConfig::default()
            })
        },
        Err(_) => {
            // if config file has not been found, use default config and write it to file
            let config = AppConfig::default();
            let write_config_status = write_config(&config);
            if write_config_status.is_err() {
                error!("{}", write_config_status.unwrap_err());
            }
            AppConfig::default()
        }
    };
    let config_keybinds = config.keybindings.clone();
    // make sure there is no overlap between keybinds
    if ignore_overlapped_keybinds {
        return Ok(config);
    }
    let mut key_count_map: HashMap<Key, u16> = HashMap::new();
    for (_, value) in config_keybinds.iter() {
        for key in value.iter() {
            let key_count = key_count_map.entry(*key).or_insert(0);
            *key_count += 1;
        }
    }
    let mut overlapped_keys: Vec<Key> = Vec::new();
    for (key, count) in key_count_map.iter() {
        if *count > 1 {
            overlapped_keys.push(*key);
        }
    }
    if overlapped_keys.len() > 0 {
        let mut overlapped_keys_str = String::new();
        for key in overlapped_keys.iter() {
            overlapped_keys_str.push_str(&format!("{:?}, ", key));
        }
        return Err(format!(
            "Overlapped keybinds found: {}",
            overlapped_keys_str
        ));
    }
    Ok(config)
}

pub fn write_config(config: &AppConfig) -> Result<(), String> {
    let config_str = serde_json::to_string(&config).unwrap();
    let prepare_config_dir_status = prepare_config_dir();
    if prepare_config_dir_status.is_err() {
        return Err(prepare_config_dir_status.unwrap_err());
    }
    let config_dir_status = get_config_dir();
    if config_dir_status.is_err() {
        return Err(config_dir_status.unwrap_err());
    }
    let config_dir = config_dir_status.unwrap();
    let write_result = fs::write(config_dir.join(CONFIG_FILE_NAME), config_str);
    match write_result {
        Ok(_) => Ok(()),
        Err(e) => {
            debug!("Error writing config file: {}", e);
            Err("Error writing config file".to_string())
        }
    }
}

pub fn get_default_ui_mode() -> UiMode {
    let get_config_status = get_config(false);
    let config = if get_config_status.is_err() {
        debug!("Error getting config: {}", get_config_status.unwrap_err());
        AppConfig::default()
    } else {
        get_config_status.unwrap()
    };
    config.default_view
}

pub fn reset_config() {
    let config = AppConfig::default();
    let write_config_status = write_config(&config);
    if write_config_status.is_err() {
        error!(
            "Error writing config file: {}",
            write_config_status.unwrap_err()
        );
    }
}

pub fn save_kanban_state_locally(boards: Vec<Board>) -> Result<(), SavefileError> {
    let get_config_status = get_config(false);
    let config = if get_config_status.is_err() {
        debug!("Error getting config: {}", get_config_status.unwrap_err());
        AppConfig::default()
    } else {
        get_config_status.unwrap()
    };
    // check config.save_directory for previous versions of the boards
    // versioning style is: SAVE_FILE_NAME_27-12-2020_v1
    // if the file exists, increment the version number
    // if the file does not exist, version number is 1
    let files = fs::read_dir(&config.save_directory)?;
    let mut version = 1;
    for file in files {
        let file = file?;
        let file_name = file.file_name().into_string().unwrap();
        if file_name.contains(SAVE_FILE_NAME)
            && file_name.contains(chrono::Local::now().format("%d-%m-%Y").to_string().as_str())
        {
            let file_version = file_name.split("_").last();
            if file_version.is_none() {
                debug!("File version not found");
                continue;
            } else {
                // remove v from version number and find max of version numbers
                let file_version = file_version.unwrap().replace("v", "");
                let file_version = file_version.parse::<u32>();
                if file_version.is_err() {
                    debug!(
                        "Error parsing version number: {}",
                        file_version.unwrap_err()
                    );
                    continue;
                } else {
                    let file_version = file_version.unwrap();
                    if file_version > version {
                        version = file_version;
                        version += 1;
                    } else if file_version == version {
                        version += 1;
                    }
                }
            }
        }
    }
    let file_name = format!(
        "{}_{}_v{}",
        SAVE_FILE_NAME,
        chrono::Local::now().format("%d-%m-%Y"),
        version
    );
    let file_path = config.save_directory.join(file_name);
    let save_status = save_file(file_path, version, &boards);
    match save_status {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn get_local_kanban_state(
    file_name: String,
    version: u32,
    preview_mode: bool,
) -> Result<Vec<Board>, SavefileError> {
    let get_config_status = get_config(false);
    let config = if get_config_status.is_err() {
        debug!("Error getting config: {}", get_config_status.unwrap_err());
        AppConfig::default()
    } else {
        get_config_status.unwrap()
    };
    let file_path = config.save_directory.join(file_name);
    if !preview_mode {
        info!("Loading local save file: {:?}", file_path);
    }
    let boards = load_file(file_path, version)?;
    Ok(boards)
}

pub fn get_available_local_savefiles() -> Option<Vec<String>> {
    let get_config_status = get_config(false);
    let config = if get_config_status.is_err() {
        debug!("Error getting config: {}", get_config_status.unwrap_err());
        AppConfig::default()
    } else {
        get_config_status.unwrap()
    };
    let read_dir_status = fs::read_dir(&config.save_directory);
    match read_dir_status {
        Ok(files) => {
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
            Some(savefiles)
        }
        Err(_) => {
            // try to create the save directory
            let default_save_path = env::temp_dir().join(SAVE_DIR_NAME);
            let dir_creation_status = fs::create_dir_all(&default_save_path);
            match dir_creation_status {
                Ok(_) => {
                    info!(
                        "Could not find save directory, created default save directory at: {:?}",
                        default_save_path
                    );
                }
                Err(e) => {
                    error!("Could not find save directory and could not create default save directory at: {:?}, error: {}", default_save_path, e);
                }
            }
            None
        }
    }
}

pub fn export_kanban_to_json(boards: &Vec<Board>) -> Result<String, String> {
    #[derive(Serialize)]
    struct ExportStruct {
        kanban_version: String,
        export_date: String,
        boards: Vec<Board>,
    }
    // use serde serialization
    let get_config_status = get_config(false);
    let config = if get_config_status.is_err() {
        debug!("Error getting config: {}", get_config_status.unwrap_err());
        AppConfig::default()
    } else {
        get_config_status.unwrap()
    };
    // make json with the keys Version, Date, Boards
    // get version from cargo.toml
    let version = env!("CARGO_PKG_VERSION");
    let date = chrono::Local::now().format("%d-%m-%Y");
    // make sure boards list is not converted to string but is a list in json
    let export_struct = ExportStruct {
        kanban_version: version.to_string(),
        export_date: date.to_string(),
        boards: boards.clone(),
    };
    let file_path = config.save_directory.join("kanban_export.json");
    // write to file
    let write_status = fs::write(
        file_path.clone(),
        serde_json::to_string_pretty(&export_struct).unwrap(),
    );
    match write_status {
        Ok(_) => Ok(file_path.to_str().unwrap().to_string()),
        Err(e) => Err(e.to_string()),
    }
}
