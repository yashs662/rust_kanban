use super::io_handler::{get_config_dir, make_file_system_safe_name};
use crate::{
    app::{
        kanban::{Board, Boards},
        AppConfig,
    },
    constants::{
        CONFIG_DIR_NAME, CONFIG_FILE_NAME, SAVE_DIR_NAME, SAVE_FILE_NAME, SAVE_FILE_REGEX,
        THEME_DIR_NAME, THEME_FILE_NAME,
    },
    inputs::key::Key,
    io::io_handler::prepare_config_dir,
    ui::Theme,
};
use log::{debug, error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashMap, env, fs, path::PathBuf};

pub fn get_config(ignore_overlapped_keybindings: bool) -> Result<AppConfig, String> {
    let config_dir_status = get_config_dir();
    let config_dir = if let Ok(config_dir) = config_dir_status {
        config_dir
    } else {
        return Err(config_dir_status.unwrap_err());
    };
    let config_path = config_dir.join(CONFIG_FILE_NAME);
    let config = match fs::read_to_string(config_path) {
        Ok(config_json_string) => {
            let serde_value = serde_json::from_str(&config_json_string);
            if let Ok(app_config) = serde_value {
                app_config
            } else {
                let parsed_config = AppConfig::from_json_string(&config_json_string);
                if let Ok(parsed_config) = parsed_config {
                    match write_config(&parsed_config) {
                        Ok(_) => parsed_config,
                        Err(e) => {
                            error!("Error writing config file: {}", e);
                            AppConfig::default()
                        }
                    }
                } else {
                    debug!(
                        "Error parsing config from json: {}",
                        parsed_config.unwrap_err()
                    );
                    write_default_config();
                    AppConfig::default()
                }
            }
        }
        Err(_) => {
            write_default_config();
            AppConfig::default()
        }
    };
    let config_keybindings = config.keybindings.clone();
    if ignore_overlapped_keybindings {
        return Ok(config);
    }
    let mut key_count_map: HashMap<Key, u16> = HashMap::new();
    for (_, value) in config_keybindings.iter() {
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
    if !overlapped_keys.is_empty() {
        let mut overlapped_keys_str = String::new();
        for key in overlapped_keys.iter() {
            overlapped_keys_str.push_str(&format!("{:?}, ", key));
        }
        return Err(format!(
            "Overlapped keybindings found: {}",
            overlapped_keys_str
        ));
    }
    Ok(config)
}

pub fn write_config(config: &AppConfig) -> Result<(), String> {
    let config_str = serde_json::to_string_pretty(&config).unwrap();
    prepare_config_dir()?;
    let config_dir = get_config_dir()?;
    let write_result = fs::write(config_dir.join(CONFIG_FILE_NAME), config_str);
    match write_result {
        Ok(_) => Ok(()),
        Err(e) => {
            debug!("Error writing config file: {}", e);
            Err("Error writing config file".to_string())
        }
    }
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

pub fn save_kanban_state_locally(boards: Vec<Board>, config: &AppConfig) -> Result<(), String> {
    let files = fs::read_dir(&config.save_directory);
    if files.is_err() {
        return Err("Error reading save directory".to_string());
    }
    let files = files.unwrap();
    let mut version = 1;
    for file in files {
        if file.is_err() {
            continue;
        }
        let file = file.unwrap();
        let file_name = file.file_name().into_string().unwrap();
        if file_name.contains(SAVE_FILE_NAME)
            && file_name.contains(chrono::Local::now().format("%d-%m-%Y").to_string().as_str())
        {
            let file_version = file_name.split('_').last();
            if let Some(file_version) = file_version {
                let file_version = file_version.replace('v', "");
                let file_version = file_version.replace(".json", "");
                let file_version = file_version.parse::<u32>();
                if let Ok(file_version) = file_version {
                    match file_version.cmp(&version) {
                        Ordering::Greater => {
                            version = file_version;
                            version += 1;
                        }
                        Ordering::Equal => {
                            version += 1;
                        }
                        Ordering::Less => {}
                    }
                } else {
                    debug!(
                        "Error parsing version number: {}",
                        file_version.unwrap_err()
                    );
                    continue;
                }
            }
        }
    }
    let file_name = format!(
        "{}_{}_v{}.json",
        SAVE_FILE_NAME,
        chrono::Local::now().format("%d-%m-%Y"),
        version
    );
    match export_kanban_to_json(&boards, config, file_name) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn get_local_kanban_state(
    file_name: String,
    preview_mode: bool,
    config: &AppConfig,
) -> Result<Boards, String> {
    let file_path = config.save_directory.join(file_name);
    if !preview_mode {
        info!("Loading local save file: {:?}", file_path);
    }
    let file = fs::File::open(file_path);
    if file.is_err() {
        debug!("Error opening save file: {}", file.err().unwrap());
        return Err("Error opening save file".to_string());
    }
    let file = file.unwrap();
    let serde_object = serde_json::from_reader(file);
    if serde_object.is_err() {
        debug!("Error parsing save file: {}", serde_object.err().unwrap());
        return Err("Error parsing save file".to_string());
    }
    let serde_object: serde_json::Value = serde_object.unwrap();
    let boards = serde_object.get("boards");
    if boards.is_none() {
        debug!("Error parsing save file, no boards found");
        return Err("Error parsing save file".to_string());
    }
    let boards = boards.unwrap();
    let boards = boards.as_array();
    if boards.is_none() {
        debug!("Error parsing save file, boards is not an array");
        return Err("Error parsing save file".to_string());
    }
    let boards = boards.unwrap();
    let mut parsed_boards = Vec::new();
    for board in boards {
        let parsed_board = Board::from_json(board)?;
        parsed_boards.push(parsed_board);
    }
    Ok(Boards::from(parsed_boards))
}

pub fn get_available_local_save_files(config: &AppConfig) -> Option<Vec<String>> {
    let read_dir_status = fs::read_dir(&config.save_directory);
    match read_dir_status {
        Ok(files) => {
            let mut save_files = Vec::new();
            for file in files {
                let file = file.unwrap();
                let file_name = file.file_name().into_string().unwrap();
                save_files.push(file_name);
            }
            let re = Regex::new(SAVE_FILE_REGEX).unwrap();

            save_files.retain(|file| re.is_match(file));
            save_files.sort_by(|a, b| {
                let a_date = a.split('_').nth(1).unwrap();
                let b_date = b.split('_').nth(1).unwrap();
                let a_version = a.split('_').nth(2).unwrap();
                let b_version = b.split('_').nth(2).unwrap();
                let a_date = chrono::NaiveDate::parse_from_str(a_date, "%d-%m-%Y").unwrap();
                let b_date = chrono::NaiveDate::parse_from_str(b_date, "%d-%m-%Y").unwrap();
                let a_version = a_version
                    .split('v')
                    .nth(1)
                    .unwrap()
                    .replace(".json", "")
                    .parse::<u32>()
                    .unwrap();
                let b_version = b_version
                    .split('v')
                    .nth(1)
                    .unwrap()
                    .replace(".json", "")
                    .parse::<u32>()
                    .unwrap();
                if a_date > b_date {
                    std::cmp::Ordering::Greater
                } else if a_date < b_date {
                    std::cmp::Ordering::Less
                } else if a_version > b_version {
                    std::cmp::Ordering::Greater
                } else if a_version < b_version {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            });
            Some(save_files)
        }
        Err(_) => {
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

pub fn export_kanban_to_json(
    boards: &[Board],
    config: &AppConfig,
    file_name: String,
) -> Result<String, String> {
    let version = env!("CARGO_PKG_VERSION");
    let date = format!(
        "{} ({})",
        chrono::Local::now().format(config.date_time_format.to_parser_string()),
        config.date_time_format.to_human_readable_string()
    );
    let export_struct = ExportStruct {
        boards: boards.to_vec(),
        export_date: date,
        kanban_version: version.to_string(),
    };
    let file_path = config.save_directory.join(file_name);
    let write_status = fs::write(
        file_path.clone(),
        serde_json::to_string_pretty(&export_struct).unwrap(),
    );
    match write_status {
        Ok(_) => Ok(file_path.to_str().unwrap().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_default_save_directory() -> PathBuf {
    let mut default_save_path = env::temp_dir();
    default_save_path.push(SAVE_DIR_NAME);
    default_save_path
}

fn get_theme_dir() -> Result<PathBuf, String> {
    let home_dir = home::home_dir();
    if home_dir.is_none() {
        return Err(String::from("Error getting home directory"));
    }
    let mut theme_dir = home_dir.unwrap();
    if cfg!(windows) {
        theme_dir.push("AppData");
        theme_dir.push("Roaming");
    } else {
        theme_dir.push(".config");
    }
    theme_dir.push(CONFIG_DIR_NAME);
    theme_dir.push(THEME_DIR_NAME);
    Ok(theme_dir)
}

pub fn get_saved_themes() -> Option<Vec<Theme>> {
    let theme_dir = get_theme_dir();
    if theme_dir.is_err() {
        return None;
    }
    let theme_dir = theme_dir.unwrap();
    let read_dir_status = fs::read_dir(&theme_dir);
    let file_prefix = format!("{}_", THEME_FILE_NAME);
    let regex_str = format!("^{}.*\\.json$", file_prefix);
    let re = Regex::new(&regex_str).unwrap();
    match read_dir_status {
        Ok(files) => {
            let mut themes = Vec::new();
            for file in files {
                let file = file.unwrap();
                let file_name = file.file_name().into_string().unwrap();
                if re.is_match(&file_name) {
                    let file_path = theme_dir.join(file_name);
                    let read_status = fs::read_to_string(file_path);
                    if read_status.is_err() {
                        continue;
                    }
                    let read_status = read_status.unwrap();
                    let theme: Theme = serde_json::from_str(&read_status).unwrap();
                    themes.push(theme);
                }
            }
            Some(themes)
        }
        Err(_) => None,
    }
}

pub fn save_theme(theme: Theme) -> Result<String, String> {
    let theme_dir = get_theme_dir()?;
    let create_dir_status = fs::create_dir_all(&theme_dir);
    if let Err(e) = create_dir_status {
        return Err(e.to_string());
    }
    let theme_name = format!(
        "{}_{}.json",
        THEME_FILE_NAME,
        make_file_system_safe_name(&theme.name)
    );
    let theme_path = theme_dir.join(theme_name);
    let write_status = fs::write(
        theme_path.clone(),
        serde_json::to_string_pretty(&theme).unwrap(),
    );
    if let Err(write_status) = write_status {
        return Err(write_status.to_string());
    }
    Ok(theme_path.to_str().unwrap().to_string())
}

fn write_default_config() {
    let config = AppConfig::default();
    let write_config_status = write_config(&config);
    if write_config_status.is_err() {
        error!("{}", write_config_status.unwrap_err());
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportStruct {
    pub boards: Vec<Board>,
    pub export_date: String,
    pub kanban_version: String,
}
