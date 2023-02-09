use std::borrow::Cow;
use std::io::stdout;
use std::sync::Arc;
use std::time::Duration;

use app::{
    App,
    AppReturn
};
use eyre::Result;
use inputs::events::Events;
use inputs::InputEvent;
use io::IoEvent;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use tui::layout::Rect;
use ui::ui_main;

pub mod app;
pub mod inputs;
pub mod io;
pub mod constants;
pub mod ui;

pub async fn start_ui(app: &Arc<tokio::sync::Mutex<App>>) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    // User event handler
    let tick_rate = app.lock().await.config.tickrate;
    let mut events = Events::new(Duration::from_millis(tick_rate));

    // Trigger state change from Init to Initialized
    {
        let mut app = app.lock().await;
        // Here we assume the the first load is a long task
        app.dispatch(IoEvent::Initialize).await;
    }

    loop {
        let mut app = app.lock().await;
        let mut states = app.state.clone();
        // Render
        terminal.draw(|rect| ui_main::draw(rect, &mut app, &mut states))?;

        // Handle inputs
        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key).await,
            InputEvent::Tick => {
                AppReturn::Continue
            }
        };
        // Check if we should exit
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.set_cursor(0, 0)?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

/// Takes wrapped text and the current cursor position (1D) and the avaiable space to return the x and y position of the cursor (2D)
fn calculate_cursor_position(text: Vec<Cow<str>>, current_cursor_position: usize, view_box: Rect) -> (u16, u16) {
    let wrapped_text_iter = text.iter();
    let mut cursor_pos = current_cursor_position;

    for (i, line) in wrapped_text_iter.enumerate() {
        if cursor_pos <= line.len() || i == text.len() - 1 {
            let x_pos = view_box.x + 1 + cursor_pos as u16;
            let y_pos = view_box.y + 1 + i as u16;
            // if x_pos is > i subtract i
            let x_pos = if x_pos > i as u16 {
                x_pos - i as u16
            } else {
                x_pos
            };
            return (x_pos, y_pos);
        }
        cursor_pos -= line.len();
    }
    (view_box.x + 1, view_box.y + 1)
}

