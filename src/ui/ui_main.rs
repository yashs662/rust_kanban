use super::{
    super::app::state::{AppStatus, UiMode},
    ui_helper,
};
use crate::app::{App, PopupMode};
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

    match &app.state.ui_mode {
        UiMode::Zen => {
            ui_helper::render_zen_mode(rect, app);
        }
        UiMode::TitleBody => {
            ui_helper::render_title_body(rect, app);
        }
        UiMode::BodyHelp => {
            ui_helper::render_body_help(rect, app);
        }
        UiMode::BodyLog => {
            ui_helper::render_body_log(rect, app);
        }
        UiMode::TitleBodyHelp => {
            ui_helper::render_title_body_help(rect, app);
        }
        UiMode::TitleBodyLog => {
            ui_helper::render_title_body_log(rect, app);
        }
        UiMode::BodyHelpLog => {
            ui_helper::render_body_help_log(rect, app);
        }
        UiMode::TitleBodyHelpLog => {
            ui_helper::render_title_body_help_log(rect, app);
        }
        UiMode::ConfigMenu => {
            ui_helper::render_config(rect, app);
        }
        UiMode::EditKeybindings => {
            ui_helper::render_edit_keybindings(rect, app);
        }
        UiMode::MainMenu => {
            ui_helper::render_main_menu(rect, app);
        }
        UiMode::HelpMenu => {
            ui_helper::render_help_menu(rect, app);
        }
        UiMode::LogsOnly => {
            ui_helper::render_logs_only(rect, app);
        }
        UiMode::NewBoard => {
            ui_helper::render_new_board_form(rect, app);
        }
        UiMode::NewCard => ui_helper::render_new_card_form(rect, app),
        UiMode::LoadSave => {
            ui_helper::render_load_save(rect, app);
        }
        UiMode::CreateTheme => ui_helper::render_create_theme(rect, app),
    }

    // Popups are rendered above ui_mode
    if app.state.popup_mode.is_some() {
        match app.state.popup_mode.unwrap() {
            PopupMode::ViewCard => {
                ui_helper::render_view_card(rect, app);
            }
            PopupMode::CardStatusSelector => {
                ui_helper::render_change_card_status_popup(rect, app);
            }
            PopupMode::ChangeUIMode => {
                ui_helper::render_change_ui_mode_popup(rect, app);
            }
            PopupMode::CommandPalette => {
                ui_helper::render_command_palette(rect, app);
            }
            PopupMode::EditGeneralConfig => {
                ui_helper::render_edit_config(rect, app);
            }
            PopupMode::EditSpecificKeyBinding => {
                ui_helper::render_edit_specific_keybinding(rect, app);
            }
            PopupMode::SelectDefaultView => {
                ui_helper::render_select_default_view(rect, app);
            }
            PopupMode::ChangeTheme => {
                ui_helper::render_change_theme_popup(rect, app);
            }
            PopupMode::ThemeEditor => {
                ui_helper::render_edit_specific_style_popup(rect, app);
            }
            PopupMode::SaveThemePrompt => {
                ui_helper::render_save_theme_prompt(rect, app);
            }
            PopupMode::CustomRGBPromptFG | PopupMode::CustomRGBPromptBG => {
                ui_helper::render_custom_rgb_color_prompt(rect, app);
            }
            PopupMode::ConfirmDiscardCardChanges => {
                ui_helper::render_confirm_discard_card_changes(rect, app);
            }
            PopupMode::CardPrioritySelector => {
                ui_helper::render_card_priority_selector(rect, app);
            }
        }
    }

    // Toasts are always rendered on top of everything else
    ui_helper::render_toast(rect, app);
    if app.state.debug_menu_toggled {
        ui_helper::render_debug_panel(rect, app);
    }
}
