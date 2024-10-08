use crate::{
    app::App,
    ui::{
        rendering::{
            common::{
                draw_title, render_body, render_card_being_dragged, render_close_button,
                render_logs,
            },
            view::TitleBodyLog,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

impl Renderable for TitleBodyLog {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(5),
                ]
                .as_ref(),
            )
            .split(rect.area());

        rect.render_widget(draw_title(app, chunks[0], is_active), chunks[0]);
        render_body(rect, chunks[1], app, false, is_active);
        render_logs(app, true, chunks[2], rect, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
        render_card_being_dragged(chunks[1], app, rect, is_active);
    }
}
