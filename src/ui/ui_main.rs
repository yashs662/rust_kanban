use tui::{
    backend::Backend,
    Frame
};

use super::{super::app::{
    AppState,
    state::{
        UiMode,
        AppStatus
    }
}, ui_helper::{render_change_ui_mode_popup, render_change_current_card_status_popup}};

use super::ui_helper::{
    check_size,
    draw_size_error,
    render_zen_mode,
    render_title_body,
    render_body_help,
    render_body_log,
    render_title_body_help,
    render_title_body_log,
    render_body_help_log,
    render_title_body_help_log,
    render_config,
    render_edit_config,
    render_edit_keybindings,
    render_edit_specific_keybinding,
    render_main_menu,
    render_help_menu,
    render_logs_only,
    render_new_board_form,
    render_load_save,
    render_new_card_form,
    draw_loading_screen,
    render_edit_default_homescreen,
    render_view_card,
    render_toast,
    render_command_palette
};
use crate::app::App;

/// Main UI Drawing handler
pub fn draw<B>(rect: &mut Frame<B>, app: &App, states: &mut AppState)
where
    B: Backend,
{   
    let msg = check_size(&rect.size());
    if &msg != "Size OK" {
        draw_size_error(rect, &rect.size(), msg);
        return;
    } else if *app.status() == AppStatus::Init {
        draw_loading_screen(rect, &rect.size());
        return;
    }

    match &app.state.ui_mode {
        UiMode::Zen => {
            render_zen_mode(rect, &app);
        }
        UiMode::TitleBody => {
            render_title_body(rect, &app);
        }
        UiMode::BodyHelp => {
            render_body_help(rect, &app, &mut states.help_state, states.keybind_store.clone());
        }
        UiMode::BodyLog => {
            render_body_log(rect, &app);
        }
        UiMode::TitleBodyHelp => {
            render_title_body_help(rect, &app, &mut states.help_state, states.keybind_store.clone());
        }
        UiMode::TitleBodyLog => {
            render_title_body_log(rect, &app);
        }
        UiMode::BodyHelpLog => {
            render_body_help_log(rect, &app, &mut states.help_state, states.keybind_store.clone());
        }
        UiMode::TitleBodyHelpLog => {
            render_title_body_help_log(rect, &app, &mut states.help_state, states.keybind_store.clone());
        }
        UiMode::ConfigMenu => {
            render_config(rect, &app, &mut states.config_state);
        }
        UiMode::EditConfig => {
            render_config(rect, &app, &mut states.config_state);
            render_edit_config(rect, &app);
        }
        UiMode::SelectDefaultView => {
            render_config(rect, &app, &mut states.config_state);
            render_edit_default_homescreen(rect, app, &mut states.default_view_state);
        }
        UiMode::EditKeybindings => {
            render_edit_keybindings(rect, &app, &mut states.edit_keybindings_state);
        }
        UiMode::EditSpecificKeybinding => {
            render_edit_keybindings(rect, &app, &mut states.edit_keybindings_state);
            render_edit_specific_keybinding(rect, &app);
        }
        UiMode::MainMenu => {
            render_main_menu(rect, &app, &mut states.main_menu_state, &mut states.help_state, states.keybind_store.clone());
        }
        UiMode::HelpMenu => {
            render_help_menu(rect, &app, &mut states.help_state, states.keybind_store.clone());
        }
        UiMode::LogsOnly => {
            render_logs_only(rect, &app);
        }
        UiMode::NewBoard => {
            render_new_board_form(rect, &app);
        }
        UiMode::NewCard => {
            render_new_card_form(rect, app)
        }
        UiMode::LoadSave => {
            render_load_save(rect, &mut states.load_save_state, app);
        }
    }

    // popups that do not have any UI modes / can be rendered on top of any UI mode,
    // order determines which is rendered on top, last function is always on top

    render_view_card(rect, &app);
    render_toast(rect, &app);
    render_change_ui_mode_popup(rect, &app, &mut states.default_view_state);
    render_change_current_card_status_popup(rect, &app, &mut states.card_status_selector_state);
    render_command_palette(rect, &app, &mut states.command_palette_list_state);
}