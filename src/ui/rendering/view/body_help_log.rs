use crate::{
    app::App,
    ui::{
        rendering::{
            common::{
                draw_help, render_body, render_card_being_dragged, render_close_button, render_logs,
            },
            utils::check_if_active_and_get_style,
            view::BodyHelpLog,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders},
    Frame,
};

impl Renderable for BodyHelpLog {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Length(5),
                ]
                .as_ref(),
            )
            .split(rect.size());

        let help_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(chunks[1]);

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );

        let help = draw_help(app, chunks[1], is_active);
        let help_separator = Block::default()
            .borders(Borders::LEFT)
            .border_style(general_style);

        render_body(rect, chunks[0], app, false, is_active);
        rect.render_widget(help.0, chunks[1]);
        rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.app_table_states.help);
        rect.render_widget(help_separator, help_chunks[1]);
        rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.app_table_states.help);
        render_logs(app, true, chunks[2], rect, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
        render_card_being_dragged(chunks[0], app, rect, is_active);
    }
}
