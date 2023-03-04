use tui::{backend::Backend, Frame};

use super::super::app::state::{AppStatus, UiMode};

use super::ui_helper::{
    check_size, draw_loading_screen, draw_size_error, render_body_help, render_body_help_log,
    render_body_log, render_change_current_card_status_popup, render_change_ui_mode_popup,
    render_command_palette, render_config, render_debug_panel, render_edit_config,
    render_edit_keybindings, render_edit_specific_keybinding, render_help_menu, render_load_save,
    render_logs_only, render_main_menu, render_new_board_form, render_new_card_form,
    render_select_default_view, render_title_body, render_title_body_help,
    render_title_body_help_log, render_title_body_log, render_toast, render_view_card,
    render_zen_mode,
};
use crate::app::{App, PopupMode};

/// Main UI Drawing handler
pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
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
            render_zen_mode(rect, app);
        }
        UiMode::TitleBody => {
            render_title_body(rect, app);
        }
        UiMode::BodyHelp => {
            render_body_help(rect, app);
        }
        UiMode::BodyLog => {
            render_body_log(rect, app);
        }
        UiMode::TitleBodyHelp => {
            render_title_body_help(rect, app);
        }
        UiMode::TitleBodyLog => {
            render_title_body_log(rect, app);
        }
        UiMode::BodyHelpLog => {
            render_body_help_log(rect, app);
        }
        UiMode::TitleBodyHelpLog => {
            render_title_body_help_log(rect, app);
        }
        UiMode::ConfigMenu => {
            render_config(rect, app);
        }
        UiMode::EditKeybindings => {
            render_edit_keybindings(rect, app);
        }
        UiMode::MainMenu => {
            render_main_menu(rect, app);
        }
        UiMode::HelpMenu => {
            render_help_menu(rect, app);
        }
        UiMode::LogsOnly => {
            render_logs_only(rect, app);
        }
        UiMode::NewBoard => {
            render_new_board_form(rect, app);
        }
        UiMode::NewCard => render_new_card_form(rect, app),
        UiMode::LoadSave => {
            render_load_save(rect, app);
        }
    }

    // Popups are rendered above ui_mode
    if app.state.popup_mode.is_some() {
        match app.state.popup_mode.unwrap() {
            PopupMode::CardView => {
                render_view_card(rect, app);
            }
            PopupMode::ChangeCurrentCardStatus => {
                render_change_current_card_status_popup(rect, app);
            }
            PopupMode::ChangeUIMode => {
                render_change_ui_mode_popup(rect, app);
            }
            PopupMode::CommandPalette => {
                render_command_palette(rect, app);
            }
            PopupMode::EditGeneralConfig => {
                render_edit_config(rect, app);
            }
            PopupMode::EditSpecificKeyBinding => {
                render_edit_specific_keybinding(rect, app);
            }
            PopupMode::SelectDefaultView => {
                render_select_default_view(rect, app);
            }
        }
    }

    // Toasts are always rendered on top of everything else
    render_toast(rect, &app);
    if app.state.debug_menu_toggled {
        render_debug_panel(rect, &app);
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            render_command_palette(rect, app);
        }
    }
}
