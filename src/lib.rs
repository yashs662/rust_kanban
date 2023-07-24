use app::{App, AppReturn};
use constants::ENCRYPTION_KEY_FILE_NAME;
use crossterm::{event::EnableMouseCapture, execute};
use eyre::Result;
use inputs::{events::Events, InputEvent};
use io::{
    data_handler::reset_config,
    handler::{
        generate_new_encryption_key, get_all_save_ids_for_user, get_config_dir, login_for_user,
        save_user_encryption_key,
    },
    IoEvent,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use std::{borrow::Cow, io::stdout, sync::Arc, time::Duration};
use ui::ui_main;

pub mod app;
pub mod constants;
pub mod inputs;
pub mod io;
pub mod ui;

#[macro_use]
extern crate lazy_static;

pub async fn start_ui(app: &Arc<tokio::sync::Mutex<App>>) -> Result<()> {
    // Configure Crossterm backend for tui
    crossterm::terminal::enable_raw_mode()?;
    {
        let app = app.lock().await;
        if app.config.enable_mouse_support {
            execute!(stdout(), EnableMouseCapture)?;
        }
    }
    let my_stdout = stdout();
    let backend = CrosstermBackend::new(my_stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // User event handler
    let mut events = {
        let tick_rate = app.lock().await.config.tickrate;
        Events::new(Duration::from_millis(tick_rate))
    };

    // Trigger state change from Init to Initialized
    {
        let mut app = app.lock().await;
        // Here we assume the the first load is a long task
        app.dispatch(IoEvent::Initialize).await;
    }

    loop {
        let mut app = app.lock().await;
        // Render
        let render_start_time = std::time::Instant::now();
        terminal.draw(|rect| ui_main::draw(rect, &mut app))?;
        let render_end_time = std::time::Instant::now();
        app.state.ui_render_time = Some(
            render_end_time
                .duration_since(render_start_time)
                .as_micros(),
        );

        // Handle inputs
        let result = match events.next().await {
            InputEvent::KeyBoardInput(key) => app.do_action(key).await,
            InputEvent::MouseAction(mouse_action) => app.handle_mouse(mouse_action).await,
            InputEvent::Tick => AppReturn::Continue,
        };
        // Check if we should exit
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }

    // Restore the terminal and close application
    execute!(stdout(), crossterm::event::DisableMouseCapture)?;
    terminal.clear()?;
    terminal.set_cursor(0, 0)?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

/// Takes wrapped text and the current cursor position (1D) and the available space to return the x and y position of the cursor (2D)
fn calculate_cursor_position(
    text: Vec<Cow<str>>,
    current_cursor_position: usize,
    view_box: Rect,
) -> (u16, u16) {
    let wrapped_text_iter = text.iter();
    let mut cursor_pos = current_cursor_position;

    let mut x_pos = view_box.x + 1 + cursor_pos as u16;
    let mut y_pos = view_box.y + 1;
    for (i, line) in wrapped_text_iter.enumerate() {
        x_pos = view_box.x + 1 + cursor_pos as u16;
        y_pos = view_box.y + 1 + i as u16;
        if cursor_pos <= line.chars().count() {
            // if x_pos is > i subtract i
            let x_pos = if x_pos > i as u16 {
                x_pos - i as u16
            } else {
                x_pos
            };
            return (x_pos, y_pos);
        }
        cursor_pos -= line.chars().count();
    }
    (x_pos, y_pos)
}

/// function to lerp between rgb values of two colors
fn lerp_between(color_a: (u8, u8, u8), color_b: (u8, u8, u8), time_in_ms: f32) -> (u8, u8, u8) {
    let r = (color_a.0 as f32 * (1.0 - time_in_ms) + color_b.0 as f32 * time_in_ms) as u8;
    let g = (color_a.1 as f32 * (1.0 - time_in_ms) + color_b.1 as f32 * time_in_ms) as u8;
    let b = (color_a.2 as f32 * (1.0 - time_in_ms) + color_b.2 as f32 * time_in_ms) as u8;
    (r, g, b)
}

pub async fn gen_new_key_main(email_id: String, password: String) -> Result<()> {
    // only to be used as a cli argument function
    let mut previous_key_lost = false;
    // check if a key is already present
    let mut key_default_path = get_config_dir().unwrap();
    key_default_path.push(ENCRYPTION_KEY_FILE_NAME);
    if key_default_path.exists() {
        print_info(
            "An encryption key already exists, are you sure you want to generate a new one? (y/n)",
        );
        println!("> ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        if input == "y" || input == "yes" {
            print_info("Preparing to generate new encryption key...");
        } else {
            print_info("Aborting...");
            return Ok(());
        }
    } else {
        print_warn(
            "Previous encryption key not found, preparing to generate new encryption key...",
        );
        previous_key_lost = true;
    }
    print_info("Trying to login...");
    let (access_token, user_id) = login_for_user(&email_id, &password, true).await?;
    let save_ids = get_all_save_ids_for_user(user_id.to_owned(), &access_token).await?;
    if save_ids.is_empty() {
        print_warn("No Cloud save files found");
        print_info("Generating new encryption key...");
        let key = generate_new_encryption_key();
        let save_status = save_user_encryption_key(&key);
        if save_status.is_err() {
            print_error("Error saving encryption key");
            print_debug(&format!("Error: {:?}", save_status.err()));
            return Ok(());
        }
        let save_location = save_status.unwrap();
        print_info("Encryption key generated and saved");
        print_info("Please keep this key safe as it will be required to access your save files");
        print_info(&format!("New Key generated_at: {}", save_location));
    } else {
        print_info(&format!("{} save files found", save_ids.len()));
        if previous_key_lost {
            print_warn("It seems like the previous encryption key was lost as it could not be found");
        }
        print_warn("Please delete these files by logging in before generating a new encryption key as they will no longer be accessible");
        print_info("Aborting...");
    }
    Ok(())
}

pub fn reset_app_main() {
    print_info("🚀 Resetting config");
    reset_config();
    print_info("👍 Config reset");
}

pub fn print_error(error: &str) {
    bunt::println!("{$red}[ERROR]{/$} - {}", error);
}

pub fn print_info(info: &str) {
    bunt::println!("{$cyan}[INFO]{/$}  - {}", info);
}

pub fn print_debug(debug: &str) {
    if cfg!(debug_assertions) {
        bunt::println!("{$green}[DEBUG]{/$} - {}", debug);
    }
}

pub fn print_warn(warn: &str) {
    bunt::println!("{$yellow}[WARN]{/$}  - {}", warn);
}
