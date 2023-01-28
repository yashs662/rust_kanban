use std::sync::Arc;
use clap::Parser;

use eyre::Result;
use log::LevelFilter;
use rust_kanban::{
    app::App,
    io::{
        handler::IoAsyncHandler,
        IoEvent
    }
};
use rust_kanban::start_ui;

extern crate savefile_derive;
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    
    // optional argument to reset config
    #[arg(short, long)]
    reset: Option<bool>
}

#[tokio::main]
async fn main() -> Result<()> {

    // parse cli args
    let args = CliArgs::parse();

    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_widget_manager = Arc::clone(&app);
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

    tokio::spawn(async move {
        let mut widget_manager = rust_kanban::ui::widgets::WidgetManager::new(app_widget_manager);
        loop {
            widget_manager.update().await;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    // check if we need to reset config
    if args.reset.is_some() {
        sync_io_tx.send(IoEvent::Reset).await.unwrap();
    }

    start_ui(&app_ui).await?;

    Ok(())
}
