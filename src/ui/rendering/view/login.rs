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
            view::Login,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

impl Renderable for Login {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        if is_active {
            if app.state.focus == Focus::EmailIDField || app.state.focus == Focus::PasswordField {
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

        let form_chunks = if app.state.user_login_data.auth_token.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length((chunks[2].height - 15) / 2),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length((chunks[2].height - 15) / 2),
                ])
                .margin(1)
                .split(chunks[2])
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length((chunks[2].height - 12) / 2),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length((chunks[2].height - 12) / 2),
                ])
                .margin(1)
                .split(chunks[2])
        };

        fn get_form_chunk(
            app: &App,
            form_chunks: &[Rect],
            index_if_auth: usize,
            index_if_not_auth: usize,
        ) -> Rect {
            if app.state.user_login_data.auth_token.is_some() {
                form_chunks[index_if_auth]
            } else {
                form_chunks[index_if_not_auth]
            }
        }

        let email_id_field_chunk = get_form_chunk(app, &form_chunks, 2, 1);
        let password_field_chunk = get_form_chunk(app, &form_chunks, 3, 2);
        let show_password_main_chunk = get_form_chunk(app, &form_chunks, 4, 3);
        let submit_button_chunk = get_form_chunk(app, &form_chunks, 5, 4);

        let show_password_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(form_chunks[3].width - 7),
                Constraint::Length(5),
            ])
            .margin(1)
            .split(show_password_main_chunk);

        let submit_button_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((form_chunks[4].width - 12) / 2),
                Constraint::Length(12),
                Constraint::Length((form_chunks[4].width - 12) / 2),
            ])
            .split(submit_button_chunk);

        let email_id_field_style = get_mouse_focusable_field_style(
            app,
            Focus::EmailIDField,
            &email_id_field_chunk,
            is_active,
            true,
        );
        let password_field_style = get_mouse_focusable_field_style(
            app,
            Focus::PasswordField,
            &password_field_chunk,
            is_active,
            true,
        );
        let show_password_style = get_mouse_focusable_field_style(
            app,
            Focus::ExtraFocus,
            &show_password_main_chunk,
            is_active,
            true,
        );
        let submit_button_style = get_mouse_focusable_field_style(
            app,
            Focus::SubmitButton,
            &submit_button_chunks[1],
            is_active,
            true,
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

        let info_paragraph = Paragraph::new("Log In")
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
        rect.render_widget(show_password_paragraph, show_password_chunks[0]);
        rect.render_widget(show_password_checkbox_paragraph, show_password_chunks[1]);
        rect.render_widget(submit_button, submit_button_chunks[1]);
        rect.render_widget(
            app.state.text_buffers.email_id.widget(),
            email_id_field_chunk,
        );
        if app.state.show_password || app.state.text_buffers.password.is_empty() {
            rect.render_widget(
                app.state.text_buffers.password.widget(),
                password_field_chunk,
            );
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
            rect.render_widget(hidden_paragraph, password_field_chunk);
        }

        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }

        if app.state.user_login_data.auth_token.is_some() {
            let email_id = if let Some(email_id) = &app.state.user_login_data.email_id {
                email_id
            } else {
                "Unknown"
            };
            let already_logged_in_indicator =
                Paragraph::new(format!("Already logged in! {}", email_id))
                    .style(general_style)
                    .block(Block::default())
                    .alignment(Alignment::Center);

            rect.render_widget(already_logged_in_indicator, form_chunks[0]);
        }

        if app.state.app_status == AppStatus::UserInput {
            match app.state.focus {
                Focus::EmailIDField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.email_id,
                        &app.config.show_line_numbers,
                        &email_id_field_chunk,
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::PasswordField => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.password,
                        &app.config.show_line_numbers,
                        &password_field_chunk,
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                _ => {}
            }
        }
    }
}
