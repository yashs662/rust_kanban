use super::{super::app::state::AppStatus, ui_helper};
use crate::app::App;
use ratatui::{backend::Backend, Frame};

/// Main UI Drawing handler
pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    ui_helper::render_blank_styled_canvas(rect, app, rect.size(), app.state.popup_mode.is_some());
    let msg = ui_helper::check_size(&rect.size());
    if &msg != "Size OK" {
        ui_helper::draw_size_error(rect, &rect.size(), msg, app);
        return;
    } else if *app.status() == AppStatus::Init {
        ui_helper::draw_loading_screen(rect, &rect.size(), app);
        return;
    }

    // Render the current ui mode
    app.state.ui_mode.render(rect, app);

    // Render the popup if it exists
    if app.state.popup_mode.is_some() {
        app.state.popup_mode.unwrap().render(rect, app);
    }

    // Render Toasts
    ui_helper::render_toast(rect, app);

    // Render the debug menu if toggled
    if app.state.debug_menu_toggled {
        ui_helper::render_debug_panel(rect, app);
    }
}
