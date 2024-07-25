use crate::{
    app::{state::Focus, App},
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::ConfirmDiscardCardChanges,
            utils::{
                centered_rect_with_length, check_if_active_and_get_style,
                get_mouse_focusable_field_style,
            },
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

impl Renderable for ConfirmDiscardCardChanges {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let popup_area = centered_rect_with_length(30, 7, rect.size());

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
            .margin(2)
            .split(popup_area);

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );

        let save_card_button_style =
            get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[0], false, false);
        let dont_save_card_button_style =
            get_mouse_focusable_field_style(app, Focus::ExtraFocus, &chunks[1], false, false);
        let save_theme_button = Paragraph::new("Yes")
            .style(save_card_button_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(save_card_button_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        let dont_save_theme_button = Paragraph::new("No")
            .style(dont_save_card_button_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(dont_save_card_button_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        let border_block = Block::default()
            .title("Save Changes to Card?")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(general_style);

        render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);
        rect.render_widget(save_theme_button, chunks[0]);
        rect.render_widget(dont_save_theme_button, chunks[1]);
        rect.render_widget(border_block, popup_area);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
    }
}
