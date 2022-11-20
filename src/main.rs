use std::sync::Arc;

use eyre::Result;
use log::LevelFilter;
use rust_kanban::app::App;
use rust_kanban::io::handler::IoAsyncHandler;
use rust_kanban::io::IoEvent;
use rust_kanban::start_ui;

#[tokio::main]
async fn main() -> Result<()> {
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    // Configure log
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui).await?;

    Ok(())
}
