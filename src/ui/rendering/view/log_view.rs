use crate::{
    app::App,
    ui::{
        rendering::{
            common::{render_close_button, render_logs},
            view::LogView,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

impl Renderable for LogView {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1)].as_ref())
            .split(rect.size());

        render_logs(app, true, chunks[0], rect, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
