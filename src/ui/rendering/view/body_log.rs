use crate::{
    app::App,
    ui::{
        rendering::{
            common::{render_body, render_card_being_dragged, render_close_button, render_logs},
            view::BodyLog,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

impl Renderable for BodyLog {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
            .split(rect.area());

        render_body(rect, chunks[0], app, false, is_active);
        render_logs(app, true, chunks[1], rect, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
        render_card_being_dragged(chunks[0], app, rect, is_active);
    }
}
