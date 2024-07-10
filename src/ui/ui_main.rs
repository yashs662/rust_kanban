use super::{super::app::state::AppStatus, ui_helper};
use crate::app::App;
use ratatui::Frame;

/// Main UI Drawing handler
pub fn draw(rect: &mut Frame, app: &mut App) {
    let popup_mode = !app.state.z_stack.is_empty();
    // Background
    ui_helper::render_blank_styled_canvas(rect, &app.current_theme, rect.size(), popup_mode);

    if let Err(msg) = ui_helper::check_size(&rect.size()) {
        ui_helper::draw_size_error(rect, &rect.size(), msg, app);
        return;
    } else if *app.status() == AppStatus::Init {
        ui_helper::draw_loading_screen(rect, &rect.size(), app);
        return;
    }

    // Render the current ui mode
    app.state.ui_mode.render(rect, app, popup_mode);

    // Render Popups
    let z_stack_len = app.state.z_stack.len();
    for index in 0..z_stack_len {
        let is_last = index == z_stack_len - 1;
        if let Some(popup) = app.state.z_stack.get_mut(index) {
            popup.render(rect, app, !is_last);
        }
    }

    // Render Toasts
    ui_helper::render_toast(rect, app);

    // Render the debug menu if toggled
    if app.state.debug_menu_toggled {
        ui_helper::render_debug_panel(rect, app);
    }
}
