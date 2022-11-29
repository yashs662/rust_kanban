use tui::backend::Backend;
use tui::Frame;

use super::AppState;
use super::state::UiMode;
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
    render_main_menu,
    render_help_menu, render_logs_only, render_new_board_form, render_load_save
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
    }

    match &app.ui_mode {
        UiMode::Zen => {
            render_zen_mode(rect, &app);
        }
        UiMode::TitleBody => {
            render_title_body(rect, &app);
        }
        UiMode::BodyHelp => {
            render_body_help(rect, &app);
        }
        UiMode::BodyLog => {
            render_body_log(rect, &app);
        }
        UiMode::TitleBodyHelp => {
            render_title_body_help(rect, &app);
        }
        UiMode::TitleBodyLog => {
            render_title_body_log(rect, &app);
        }
        UiMode::BodyHelpLog => {
            render_body_help_log(rect, &app);
        }
        UiMode::TitleBodyHelpLog => {
            render_title_body_help_log(rect, &app);
        }
        UiMode::Config => {
            render_config(rect, &app, &mut states.config_state);
        }
        UiMode::EditConfig => {
            render_edit_config(rect, &app);
        }
        UiMode::MainMenu => {
            render_main_menu(rect, &app, &mut states.main_menu_state);
        }
        UiMode::HelpMenu => {
            render_help_menu(rect, &app.focus);
        }
        UiMode::LogsOnly => {
            render_logs_only(rect, &app.focus);
        }
        UiMode::ViewCard => {
            todo!("ViewCard");
        }
        UiMode::NewBoard => {
            render_new_board_form(rect, &app);
        }
        UiMode::LoadSave => {
            render_load_save(rect, &mut states.load_save_state);
        }
    }
}