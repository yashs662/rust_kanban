use std::fs;
use log::{error, info};

use crate::app::{
    AppConfig,
    state::UiMode
};
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