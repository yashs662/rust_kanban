use std::{sync::Arc, path::PathBuf};
use crate::constants::{
    CONFIG_DIR_NAME,
    CONFIG_FILE_NAME
};
use crate::app::AppConfig;
use eyre::Result;
use log::{error, info};

use super::IoEvent;
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
}

pub(crate) fn get_config_dir() -> PathBuf {
    let mut config_dir = home::home_dir().unwrap();
    config_dir.push(".config");
    config_dir.push(CONFIG_DIR_NAME);
    config_dir
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