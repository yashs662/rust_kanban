use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App,
    },
    constants::HIDDEN_PASSWORD_SYMBOL,
    ui::{
        rendering::{
            common::{
                draw_crab_pattern, draw_title, render_blank_styled_canvas_with_margin,
                render_close_button,
            },
            utils::{
                calculate_viewport_corrected_cursor_position, centered_rect_with_length,
                check_if_active_and_get_style, get_mouse_focusable_field_style,
            },
            view::Signup,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

impl Renderable for Signup {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        if is_active {
            if app.state.focus == Focus::EmailIDField
                || app.state.focus == Focus::PasswordField
                || app.state.focus == Focus::ConfirmPasswordField
            {
                if app.state.app_status != AppStatus::UserInput {
                    app.state.app_status = AppStatus::UserInput;
                }
            } else if app.state.app_status != AppStatus::Initialized {
                app.state.app_status = AppStatus::Initialized;
            }
        }

        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
            .split(rect.size());

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(50),
            ])
            .split(main_chunks[1]);

        let info_box = centered_rect_with_length(30, 10, chunks[0]);

        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(info_box);

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((chunks[2].height - 15) / 2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length((chunks[2].height - 15) / 2),
            ])
            .margin(1)
            .split(chunks[2]);

        let show_password_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(form_chunks[3].width - 7),
                Constraint::Length(5),
            ])
            .margin(1)
            .split(form_chunks[4]);

        let submit_button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((form_chunks[4].width - 12) / 2),
                Constraint::Length(12),
                Constraint::Length((form_chunks[4].width - 12) / 2),
            ])
            .split(form_chunks[5]);

        let email_id_field_style = get_mouse_focusable_field_style(
            app,
            Focus::EmailIDField,
            &form_chunks[1],
            is_active,
            true,
        );

        let password_field_style = get_mouse_focusable_field_style(
            app,
            Focus::PasswordField,
            &form_chunks[2],
            is_active,
            true,
        );

        let confirm_password_field_style = get_mouse_focusable_field_style(
            app,
            Focus::ConfirmPasswordField,
            &form_chunks[3],
            is_active,
            true,
        );

        let show_password_checkbox_style = get_mouse_focusable_field_style(
            app,
            Focus::ExtraFocus,
            &show_password_chunks[1],
            is_active,
            false,
        );

        let submit_button_style = get_mouse_focusable_field_style(
            app,
            Focus::SubmitButton,
            &submit_button_chunks[1],
            is_active,
            false,
        );

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
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

        let crab_paragraph = draw_crab_pattern(
            chunks[0],
            app.current_theme.inactive_text_style,
            is_active,
            app.config.disable_animations,
        );

        let info_border = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(general_style);

        let info_paragraph = Paragraph::new("Sign Up")
            .style(general_style)
            .block(Block::default())
            .alignment(Alignment::Center);

        let accept_key = app
            .get_first_keybinding(KeyBindingEnum::Accept)
            .unwrap_or("".to_string());
        let next_focus_key = app
            .get_first_keybinding(KeyBindingEnum::NextFocus)
            .unwrap_or("".to_string());
        let prv_focus_key = app
            .get_first_keybinding(KeyBindingEnum::PrvFocus)
            .unwrap_or("".to_string());

        let help_spans = vec![
            Span::styled("Press ", help_text_style),
            Span::styled(next_focus_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(prv_focus_key, help_key_style),
            Span::styled(" to change focus. Press ", help_text_style),
            Span::styled(accept_key, help_key_style),
            Span::styled(" to submit.", help_text_style),
        ];

        let help_paragraph = Paragraph::new(Line::from(help_spans))
            .style(general_style)
            .block(Block::default())
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let separator = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(general_style);

        let email_id_block = Block::default()
            .style(email_id_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let password_block = Block::default()
            .style(password_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let confirm_password_block = Block::default()
            .style(confirm_password_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        app.state
            .text_buffers
            .email_id
            .set_placeholder_text("Email ID");

        app.state.text_buffers.email_id.set_block(email_id_block);

        app.state
            .text_buffers
            .password
            .set_placeholder_text("Password");

        app.state.text_buffers.password.set_block(password_block);

        app.state
            .text_buffers
            .confirm_password
            .set_placeholder_text("Confirm Password");

        app.state
            .text_buffers
            .confirm_password
            .set_block(confirm_password_block);

        let show_password_paragraph = Paragraph::new("Show Password")
            .style(general_style)
            .block(Block::default())
            .alignment(Alignment::Right);

        let show_password_checkbox_value = if app.state.show_password {
            "[X]"
        } else {
            "[ ]"
        };

        let show_password_checkbox_paragraph = Paragraph::new(show_password_checkbox_value)
            .style(show_password_checkbox_style)
            .block(Block::default())
            .alignment(Alignment::Center);

        let submit_button = Paragraph::new("Submit")
            .style(submit_button_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);

        rect.render_widget(draw_title(app, main_chunks[0], is_active), main_chunks[0]);
        rect.render_widget(crab_paragraph, chunks[0]);
        rect.render_widget(Clear, info_box);
        render_blank_styled_canvas_with_margin(rect, app, info_box, is_active, -1);
        rect.render_widget(info_border, info_box);
        rect.render_widget(info_paragraph, info_chunks[0]);
        rect.render_widget(help_paragraph, info_chunks[2]);
        rect.render_widget(separator, chunks[1]);
        rect.render_widget(app.state.text_buffers.email_id.widget(), form_chunks[1]);

        if app.state.show_password {
            rect.render_widget(app.state.text_buffers.password.widget(), form_chunks[2]);
            rect.render_widget(
                app.state.text_buffers.confirm_password.widget(),
                form_chunks[3],
            );
        } else {
            if app.state.text_buffers.password.is_empty() {
                rect.render_widget(app.state.text_buffers.password.widget(), form_chunks[2]);
            } else {
                let hidden_text = HIDDEN_PASSWORD_SYMBOL
                    .to_string()
                    .repeat(app.state.text_buffers.password.get_joined_lines().len());
                let hidden_paragraph = Paragraph::new(hidden_text)
                    .style(password_field_style)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    );
                rect.render_widget(hidden_paragraph, form_chunks[2]);
            }
            if app.state.text_buffers.confirm_password.is_empty() {
                rect.render_widget(
                    app.state.text_buffers.confirm_password.widget(),
                    form_chunks[3],
                );
            } else {
                // TODO: Use the textbox function to hide it
                let hidden_text = HIDDEN_PASSWORD_SYMBOL.to_string().repeat(
                    app.state
                        .text_buffers
                        .confirm_password
                        .get_joined_lines()
                        .len(),
                );
                let hidden_paragraph = Paragraph::new(hidden_text)
                    .style(confirm_password_field_style)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    );
                rect.render_widget(hidden_paragraph, form_chunks[3]);
            }
        }

        rect.render_widget(show_password_paragraph, show_password_chunks[0]);
        rect.render_widget(show_password_checkbox_paragraph, show_password_chunks[1]);
        rect.render_widget(submit_button, submit_button_chunks[1]);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }

        if app.state.app_status == AppStatus::UserInput {
            match app.state.focus {
                Focus::EmailIDField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.email_id,
                        &app.config.show_line_numbers,
                        &form_chunks[1],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::PasswordField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.password,
                        &app.config.show_line_numbers,
                        &form_chunks[2],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::ConfirmPasswordField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.confirm_password,
                        &app.config.show_line_numbers,
                        &form_chunks[3],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                _ => {}
            }
        }
    }
}
