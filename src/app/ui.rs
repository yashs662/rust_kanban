use tui::backend::Backend;
use tui::Frame;
use tui::widgets::{
    ListState
};

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
    render_help_menu
};
use crate::app::App;

/// Main UI Drawing handler
pub fn draw<B>(rect: &mut Frame<B>, app: &App, config_state: &mut ListState, main_menu_state: &mut ListState)
where
    B: Backend,
{
    let size = rect.size();

    let msg = check_size(&size);
    // match to check if msg is size OK
    if &msg == "Size OK" {
        // pass
    } else {
        // draw error message
        draw_size_error(rect, &size, msg);
        return;
    }

    let current_ui_mode = &app.ui_mode;
    let current_board = &app.current_board;
    let current_card = &app.current_card;

    match current_ui_mode {
        UiMode::Zen => {
            render_zen_mode(&app.focus, &app.boards, rect, current_board, current_card);
        }
        UiMode::TitleBody => {
            render_title_body(&app.focus, &app.boards, rect, current_board, current_card);
        }
        UiMode::BodyHelp => {
            render_body_help(&app.focus, &app.boards, rect, app.actions(), current_board, current_card)
        }
        UiMode::BodyLog => {
            render_body_log(&app.focus, &app.boards, rect, current_board, current_card)
        }
        UiMode::TitleBodyHelp => {
            render_title_body_help(&app.focus, &app.boards, rect, app.actions(), current_board, current_card)
        }
        UiMode::TitleBodyLog => {
            render_title_body_log(&app.focus, &app.boards, rect, current_board, current_card)
        }
        UiMode::BodyHelpLog => {
            render_body_help_log(&app.focus, &app.boards, rect, app.actions(), current_board, current_card)
        }
        UiMode::TitleBodyHelpLog => {
            render_title_body_help_log(&app.focus, &app.boards, rect, app.actions(), current_board, current_card)
        }
        UiMode::Config => {
            render_config(rect, config_state, &app.focus);
        }
        UiMode::EditConfig => {
            render_edit_config(rect, &app.focus, app.current_user_input.clone(), app.config_item_being_edited);
        }
        UiMode::MainMenu => {
            render_main_menu(rect, main_menu_state, app.main_menu.items.clone(), &app.focus);
        }
        UiMode::HelpMenu => {
            render_help_menu(rect, &app.focus);
        }
        UiMode::ViewCard => {
            todo!("ViewCard");
        }

    }
}