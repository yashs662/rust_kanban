use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App,
    },
    constants::{HIDDEN_PASSWORD_SYMBOL, MIN_TIME_BETWEEN_SENDING_RESET_LINK},
    ui::{
        rendering::{
            common::{
                draw_crab_pattern, draw_title, render_blank_styled_canvas_with_margin,
                render_close_button,
            },
            utils::{
                calculate_viewport_corrected_cursor_position, centered_rect_with_length,
                check_if_active_and_get_style, check_if_mouse_is_in_area,
                get_mouse_focusable_field_style,
            },
            view::ResetPassword,
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
use std::time::Duration;

impl Renderable for ResetPassword {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        if is_active {
            if app.state.focus == Focus::EmailIDField
                || app.state.focus == Focus::ResetPasswordLinkField
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

        let info_box = centered_rect_with_length(54, 13, chunks[0]);

        let info_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Fill(1),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(info_box);

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((chunks[2].height - 24) / 2),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length((chunks[2].height - 24) / 2),
            ])
            .margin(1)
            .split(chunks[2]);

        let email_id_chunk = form_chunks[1];
        let send_reset_link_button_chunk = form_chunks[2];
        let reset_link_chunk = form_chunks[4];
        let new_password_chunk = form_chunks[5];
        let confirm_new_password_chunk = form_chunks[6];
        let show_password_main_chunk = form_chunks[7];
        let submit_button_chunk = form_chunks[8];

        let show_password_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(show_password_main_chunk.width - 7),
                Constraint::Length(5),
            ])
            .margin(1)
            .split(show_password_main_chunk);

        let submit_button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((submit_button_chunk.width - 12) / 2),
                Constraint::Length(12),
                Constraint::Length((submit_button_chunk.width - 12) / 2),
            ])
            .split(submit_button_chunk);

        let email_id_field_style = get_mouse_focusable_field_style(
            app,
            Focus::EmailIDField,
            &email_id_chunk,
            is_active,
            true,
        );

        let send_reset_link_button_style = if !is_active {
            app.current_theme.inactive_text_style
        } else if let Some(last_reset_password_link_sent_time) =
            app.state.last_reset_password_link_sent_time
        {
            if last_reset_password_link_sent_time.elapsed()
                < Duration::from_secs(MIN_TIME_BETWEEN_SENDING_RESET_LINK)
            {
                app.current_theme.inactive_text_style
            } else if check_if_mouse_is_in_area(
                &app.state.current_mouse_coordinates,
                &send_reset_link_button_chunk,
            ) {
                if app.state.mouse_focus != Some(Focus::SendResetPasswordLinkButton) {
                    app.state.app_status = AppStatus::Initialized;
                } else {
                    app.state.app_status = AppStatus::UserInput;
                }
                app.state.mouse_focus = Some(Focus::SendResetPasswordLinkButton);
                app.state.set_focus(Focus::SendResetPasswordLinkButton);
                app.current_theme.mouse_focus_style
            } else if app.state.focus == Focus::SendResetPasswordLinkButton {
                app.current_theme.keyboard_focus_style
            } else {
                app.current_theme.general_style
            }
        } else if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &send_reset_link_button_chunk,
        ) {
            if app.state.mouse_focus != Some(Focus::SendResetPasswordLinkButton) {
                app.state.app_status = AppStatus::Initialized;
            } else {
                app.state.app_status = AppStatus::UserInput;
            }
            app.state.mouse_focus = Some(Focus::SendResetPasswordLinkButton);
            app.state.set_focus(Focus::SendResetPasswordLinkButton);
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::SendResetPasswordLinkButton {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };

        let separator_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );

        let reset_link_field_style = get_mouse_focusable_field_style(
            app,
            Focus::ResetPasswordLinkField,
            &reset_link_chunk,
            is_active,
            true,
        );

        let new_password_field_style = get_mouse_focusable_field_style(
            app,
            Focus::PasswordField,
            &new_password_chunk,
            is_active,
            true,
        );

        let confirm_new_password_field_style = get_mouse_focusable_field_style(
            app,
            Focus::ConfirmPasswordField,
            &confirm_new_password_chunk,
            is_active,
            true,
        );

        let show_password_style = get_mouse_focusable_field_style(
            app,
            Focus::ExtraFocus,
            &show_password_main_chunk,
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
            .border_style(separator_style);

        let info_header = Paragraph::new("Reset Password")
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

        let help_lines = vec![
            Line::from(Span::styled(
                "1) Enter your email and send reset link first.",
                help_text_style,
            )),
            Line::from(Span::styled(
                "2) Copy the reset link from your email and then paste the reset link.",
                help_text_style,
            )),
            Line::from(Span::styled(
                "3) Enter new password and confirm the new password.",
                help_text_style,
            )),
            Line::from(""),
            Line::from(Span::styled(
                "### Check Spam folder if you don't see the email ###",
                help_text_style,
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", help_text_style),
                Span::styled(next_focus_key, help_key_style),
                Span::styled(" or ", help_text_style),
                Span::styled(prv_focus_key, help_key_style),
                Span::styled(" to change focus. Press ", help_text_style),
                Span::styled(accept_key, help_key_style),
                Span::styled(" to submit.", help_text_style),
            ]),
        ];

        let help_paragraph = Paragraph::new(help_lines)
            .style(general_style)
            .block(Block::default())
            .wrap(ratatui::widgets::Wrap { trim: true });

        let separator = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(separator_style);

        let send_reset_link_button_text = if let Some(last_reset_password_link_sent_time) =
            app.state.last_reset_password_link_sent_time
        {
            if last_reset_password_link_sent_time.elapsed()
                < Duration::from_secs(MIN_TIME_BETWEEN_SENDING_RESET_LINK)
            {
                let remaining_time = Duration::from_secs(MIN_TIME_BETWEEN_SENDING_RESET_LINK)
                    .checked_sub(last_reset_password_link_sent_time.elapsed())
                    .unwrap();
                format!("Please wait for {} seconds", remaining_time.as_secs())
            } else {
                "Send Reset Link".to_string()
            }
        } else {
            "Send Reset Link".to_string()
        };

        let email_id_block = Block::default()
            .style(email_id_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let send_reset_link_button = Paragraph::new(send_reset_link_button_text)
            .style(send_reset_link_button_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);

        let reset_link_block = Block::default()
            .style(reset_link_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let new_password_block = Block::default()
            .style(new_password_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let confirm_new_password_block = Block::default()
            .style(confirm_new_password_field_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        app.state
            .text_buffers
            .email_id
            .set_placeholder_text("Email ID");

        app.state.text_buffers.email_id.set_block(email_id_block);

        app.state
            .text_buffers
            .reset_password_link
            .set_placeholder_text("Reset Link");

        app.state
            .text_buffers
            .reset_password_link
            .set_block(reset_link_block);

        app.state
            .text_buffers
            .password
            .set_placeholder_text("New Password");

        app.state
            .text_buffers
            .password
            .set_block(new_password_block);

        app.state
            .text_buffers
            .confirm_password
            .set_placeholder_text("Confirm New Password");

        app.state
            .text_buffers
            .confirm_password
            .set_block(confirm_new_password_block);

        let show_password_paragraph = Paragraph::new("Show Password")
            .style(show_password_style)
            .block(Block::default())
            .alignment(Alignment::Right);

        let show_password_checkbox_value = if app.state.show_password {
            "[X]"
        } else {
            "[ ]"
        };

        let show_password_checkbox_paragraph = Paragraph::new(show_password_checkbox_value)
            .style(show_password_style)
            .block(Block::default())
            .alignment(Alignment::Center);

        let submit_button = Paragraph::new("Submit")
            .style(submit_button_style)
            .block(
                Block::default()
                    // TODO: Think about using the bordered shorthand everywhere
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);

        rect.render_widget(draw_title(app, main_chunks[0], is_active), main_chunks[0]);
        rect.render_widget(crab_paragraph, chunks[0]);
        rect.render_widget(Clear, info_box);
        render_blank_styled_canvas_with_margin(rect, app, info_box, is_active, -1);
        rect.render_widget(info_border, info_box);
        rect.render_widget(info_header, info_chunks[0]);
        rect.render_widget(help_paragraph, info_chunks[2]);
        rect.render_widget(separator, chunks[1]);
        rect.render_widget(app.state.text_buffers.email_id.widget(), email_id_chunk);
        rect.render_widget(send_reset_link_button, send_reset_link_button_chunk);
        rect.render_widget(
            app.state.text_buffers.reset_password_link.widget(),
            reset_link_chunk,
        );
        if app.state.show_password {
            rect.render_widget(app.state.text_buffers.password.widget(), new_password_chunk);
            rect.render_widget(
                app.state.text_buffers.confirm_password.widget(),
                confirm_new_password_chunk,
            );
        } else {
            if app.state.text_buffers.password.is_empty() {
                rect.render_widget(app.state.text_buffers.password.widget(), new_password_chunk);
            } else {
                let hidden_text = HIDDEN_PASSWORD_SYMBOL
                    .to_string()
                    .repeat(app.state.text_buffers.password.get_joined_lines().len());
                let hidden_paragraph = Paragraph::new(hidden_text)
                    .style(new_password_field_style)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    );
                rect.render_widget(hidden_paragraph, new_password_chunk);
            }
            if app.state.text_buffers.confirm_password.is_empty() {
                rect.render_widget(
                    app.state.text_buffers.confirm_password.widget(),
                    confirm_new_password_chunk,
                );
            } else {
                let hidden_text = HIDDEN_PASSWORD_SYMBOL.to_string().repeat(
                    app.state
                        .text_buffers
                        .confirm_password
                        .get_joined_lines()
                        .len(),
                );
                let hidden_paragraph = Paragraph::new(hidden_text)
                    .style(confirm_new_password_field_style)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    );
                rect.render_widget(hidden_paragraph, confirm_new_password_chunk);
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
                        &email_id_chunk,
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::ResetPasswordLinkField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.reset_password_link,
                        &app.config.show_line_numbers,
                        &reset_link_chunk,
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::PasswordField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.password,
                        &app.config.show_line_numbers,
                        &new_password_chunk,
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::ConfirmPasswordField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.confirm_password,
                        &app.config.show_line_numbers,
                        &confirm_new_password_chunk,
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                _ => {}
            }
        }
    }
}
