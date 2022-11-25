use std::sync::Arc;

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
        app.initialized(); // we could update the app state
        info!("👍 Application initialized");
        Ok(())
    }

    async fn get_local_save(&mut self) -> Result<()> {
        info!("🚀 Get local save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("👍 Local save loaded");
        Ok(())
    }

    async fn get_cloud_save(&mut self) -> Result<()> {
        info!("🚀 Get cloud save");
        let mut app = self.app.lock().await;
        app.set_boards(vec![]);
        info!("👍 Cloud save loaded");
        Ok(())
    }
}
