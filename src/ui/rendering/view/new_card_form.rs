use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App,
    },
    ui::{
        rendering::{
            common::render_close_button,
            utils::{
                calculate_viewport_corrected_cursor_position, check_if_active_and_get_style,
                get_mouse_focusable_field_style,
            },
            view::NewCardForm,
        },
        PopUp, Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

impl Renderable for NewCardForm {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Length(4),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(rect.size());

        let card_due_date = app
            .widgets
            .date_time_picker
            .get_date_time_as_string(app.config.date_time_format);

        if app.state.z_stack.last() == Some(&PopUp::DateTimePicker) {
            if app.widgets.date_time_picker.anchor.is_none() {
                app.widgets.date_time_picker.anchor = Some((
                    chunks[3].x + card_due_date.len() as u16 + 2,
                    chunks[3].y + 3,
                )); // offsets to make sure date is visible
                log::debug!(
                    "Setting anchor for date time picker to: {:?}",
                    app.widgets.date_time_picker.anchor
                );
            }
            app.widgets.date_time_picker.current_viewport = Some(rect.size());
        }

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
        let name_style =
            get_mouse_focusable_field_style(app, Focus::CardName, &chunks[1], is_active, false);
        let description_style = get_mouse_focusable_field_style(
            app,
            Focus::CardDescription,
            &chunks[2],
            is_active,
            false,
        );
        let due_date_style =
            get_mouse_focusable_field_style(app, Focus::CardDueDate, &chunks[3], is_active, false);
        let help_key_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.help_key_style,
        );
        let help_text_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.help_text_style,
        );
        let submit_style =
            get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[5], is_active, false);

        let title_paragraph = Paragraph::new("Create a new Card")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(general_style),
            );
        rect.render_widget(title_paragraph, chunks[0]);

        let card_name_block = Block::default()
            .borders(Borders::ALL)
            .style(name_style)
            .border_type(BorderType::Rounded)
            .title("Card Name (required)");
        app.state.text_buffers.card_name.set_block(card_name_block);
        rect.render_widget(app.state.text_buffers.card_name.widget(), chunks[1]);
        let description_length = app.state.text_buffers.card_description.get_num_lines();
        let description_block = Block::default()
            .title(format!("Description ({} line(s))", description_length))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(description_style);

        if app.config.show_line_numbers {
            app.state
                .text_buffers
                .card_description
                .set_line_number_style(general_style)
        } else {
            app.state.text_buffers.card_description.remove_line_number()
        }
        app.state
            .text_buffers
            .card_description
            .set_block(description_block.clone());
        rect.render_widget(app.state.text_buffers.card_description.widget(), chunks[2]);

        let card_due_date = app
            .widgets
            .date_time_picker
            .get_date_time_as_string(app.config.date_time_format);
        let card_due_date_paragraph = Paragraph::new(card_due_date).block(
            Block::default()
                .borders(Borders::ALL)
                .style(due_date_style)
                .border_type(BorderType::Rounded),
        );
        rect.render_widget(card_due_date_paragraph, chunks[3]);

        let input_mode_key = app
            .get_first_keybinding(KeyBindingEnum::TakeUserInput)
            .unwrap_or("".to_string());
        let next_focus_key = app
            .get_first_keybinding(KeyBindingEnum::NextFocus)
            .unwrap_or("".to_string());
        let prv_focus_key = app
            .get_first_keybinding(KeyBindingEnum::PrvFocus)
            .unwrap_or("".to_string());
        let accept_key = app
            .get_first_keybinding(KeyBindingEnum::Accept)
            .unwrap_or("".to_string());
        let cancel_key = app
            .get_first_keybinding(KeyBindingEnum::GoToPreviousViewOrCancel)
            .unwrap_or("".to_string());
        let stop_user_input_key = app
            .get_first_keybinding(KeyBindingEnum::StopUserInput)
            .unwrap_or("".to_string());

        let help_spans = Line::from(vec![
            Span::styled("Press ", help_text_style),
            Span::styled(input_mode_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(accept_key.clone(), help_key_style),
            Span::styled(" to start typing. Press ", help_text_style),
            Span::styled(stop_user_input_key, help_key_style),
            Span::styled(" to stop typing. Press ", help_text_style),
            Span::styled(next_focus_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(prv_focus_key, help_key_style),
            Span::styled(" to switch focus. Press ", help_text_style),
            Span::styled(accept_key, help_key_style),
            Span::styled(" to submit. Press ", help_text_style),
            Span::styled(cancel_key, help_key_style),
            Span::styled(" to cancel", help_text_style),
        ]);

        let help_paragraph = Paragraph::new(help_spans)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(general_style),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        rect.render_widget(help_paragraph, chunks[4]);

        let submit_button = Paragraph::new("Submit").alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::ALL)
                .style(submit_style)
                .border_type(BorderType::Rounded),
        );
        rect.render_widget(submit_button, chunks[5]);

        if app.state.app_status == AppStatus::UserInput {
            match app.state.focus {
                Focus::CardName => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.card_name,
                        &app.config.show_line_numbers,
                        &chunks[1],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::CardDescription => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.card_description,
                        &app.config.show_line_numbers,
                        &chunks[2],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                _ => {}
            }
        }

        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
