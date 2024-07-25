use crate::{
    app::{state::AppStatus, App},
    ui::{rendering::common, ui_helper},
};
use ratatui::Frame;

/// Main UI Drawing handler
pub fn draw(rect: &mut Frame, app: &mut App) {
    let is_active = app.state.z_stack.is_empty();

    // Background
    common::render_blank_styled_canvas(rect, &app.current_theme, rect.size(), is_active);

    // Check if the terminal size is too small or the app is still initializing
    if let Err(msg) = ui_helper::check_size(&rect.size()) {
        ui_helper::draw_size_error(rect, &rect.size(), msg, app);
        return;
    } else if *app.status() == AppStatus::Init {
        ui_helper::draw_loading_screen(rect, &rect.size(), app);
        return;
    }

    // Render the current View
    app.state.current_view.render(rect, app, is_active);

    // Render Popups
    let z_stack_len = app.state.z_stack.len();
    for index in 0..z_stack_len {
        let is_last = index == z_stack_len - 1;
        if let Some(popup) = app.state.z_stack.get_mut(index) {
            popup.render(rect, app, is_last);
        }
    }

    // Render Toasts
    ui_helper::render_toast(rect, app);

    // Render the debug menu if toggled
    if app.state.debug_menu_toggled {
        ui_helper::render_debug_panel(rect, app);
    }
}
