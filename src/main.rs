use clap::Parser;
use crossterm::{event::DisableMouseCapture, execute, terminal};
use eyre::Result;
use log::LevelFilter;
use ratatui::{backend::CrosstermBackend, Terminal};
use rust_kanban::{
    app::App,
    constants::APP_TITLE,
    gen_new_key_main,
    io::{handler::IoAsyncHandler, logger, IoEvent},
    reset_app_main, start_ui,
};
use std::{io::stdout, sync::Arc};

// generate_new_encryption_key should not take an value

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    // optional argument to reset config
    #[arg(short, long, default_value = "false")]
    reset: bool,
    #[arg(short, long, default_value = "false")]
    generate_new_encryption_key: bool,
    #[arg(short, long)]
    email_id: Option<String>,
    #[arg(short, long)]
    password: Option<String>,
    #[arg(long)]
    encryption_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Handling Panic when terminal is in raw mode
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        _ = terminal::disable_raw_mode();
        let execute_result = execute!(stdout(), DisableMouseCapture);
        if let Err(e) = execute_result {
            println!("Error while disabling mouse capture: {}", e);
        }
        println!();
        let stdout = stdout();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend);
        if let Ok(mut terminal) = terminal {
            terminal.clear().unwrap();
        }
        if cfg!(debug_assertions) {
            default_panic(info);
        } else {
            println!(
                "An error occured 😢,\n{} has crashed please report this issue on github\n{}",
                APP_TITLE,
                env!("CARGO_PKG_REPOSITORY")
            );
        }
    }));

    // Configure log
    logger::init_logger(LevelFilter::Debug).unwrap();
    logger::set_default_level(log::LevelFilter::Debug);
    // parse cli args
    let args = CliArgs::parse();

    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // We need to share the App between thread
    let main_app_instance = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_widget_manager_instance = Arc::clone(&main_app_instance);
    let app_ui_instance = Arc::clone(&main_app_instance);

    // TODO: get term bg color
    // let term_bg = get_term_bg_color();

    // check if we need to reset config
    if args.reset {
        reset_app_main();
        return Ok(());
    }
    if args.generate_new_encryption_key {
        if args.email_id.is_none() || args.password.is_none() {
            println!();
            println!("[ERROR] - Please provide email id (-e) and password (-p) to reset you encryption key");
            println!();
            return Ok(());
        }
        gen_new_key_main(args.email_id.unwrap(), args.password.unwrap()).await?;
        return Ok(());
    } else if args.email_id.is_some() || args.password.is_some() {
        println!();
        println!("[ERROR] - Please provide the -g or --generate-new-encryption-key flag to generate a new encryption key");
        println!();
        return Ok(());
    }
    if args.encryption_key.is_some() {
        let encryption_key = args.encryption_key.unwrap();
        let mut app = main_app_instance.lock().await;
        app.state.encryption_key_from_arguments = Some(encryption_key);
    }

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(main_app_instance);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    tokio::spawn(async move {
        let mut widget_manager =
            rust_kanban::ui::widgets::WidgetManager::new(app_widget_manager_instance);
        loop {
            widget_manager.update().await;
        }
    });

    start_ui(&app_ui_instance).await?;

    Ok(())
}
