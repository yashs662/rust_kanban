use crate::{
    app::App,
    ui::{
        rendering::{
            common::{draw_title, render_body, render_card_being_dragged, render_close_button},
            view::TitleBody,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

impl Renderable for TitleBody {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
            .split(rect.size());

        rect.render_widget(draw_title(app, chunks[0], is_active), chunks[0]);
        render_body(rect, chunks[1], app, false, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
        render_card_being_dragged(chunks[1], app, rect, is_active);
    }
}
