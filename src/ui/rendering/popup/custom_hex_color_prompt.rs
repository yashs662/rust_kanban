use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App,
    },
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::CustomHexColorPrompt,
            utils::{
                calculate_viewport_corrected_cursor_position, centered_rect_with_length,
                check_if_active_and_get_style, get_mouse_focusable_field_style,
            },
        },
        PopUp, Renderable,
    },
    util::parse_hex_to_rgb,
};
use log::debug;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Color,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

impl Renderable for CustomHexColorPrompt {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let popup_area = centered_rect_with_length(72, 12, rect.size());
        let prompt_text = "Enter a custom Hex color in the format: #RRGGBB (e.g. #FF0000)";

        let chunks = if app.config.enable_mouse_support {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .margin(1)
                .split(popup_area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .margin(1)
                .split(popup_area)
        };

        let custom_hex_color_input = match app.state.z_stack.last() {
            Some(PopUp::CustomHexColorPromptFG) => app
                .state
                .text_buffers
                .theme_editor_fg_hex
                .get_joined_lines(),
            Some(PopUp::CustomHexColorPromptBG) => app
                .state
                .text_buffers
                .theme_editor_bg_hex
                .get_joined_lines(),
            _ => {
                debug!("Invalid PopupView for custom Hex color prompt");
                "".to_string()
            }
        };

        let parsed_hex = parse_hex_to_rgb(&custom_hex_color_input);

        let input_field_chunks = if parsed_hex.is_some() {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(chunks[1])
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Fill(1), Constraint::Length(20)].as_ref())
                .split(chunks[1])
        };

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
        let error_text_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.error_text_style,
        );
        let text_input_style = get_mouse_focusable_field_style(
            app,
            Focus::TextInput,
            &input_field_chunks[0],
            is_active,
            true,
        );

        let prompt_text = Paragraph::new(prompt_text)
            .style(general_style)
            .block(Block::default())
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let text_input = if let Some(hex) = parsed_hex {
            let styled_text = Line::from(vec![Span::styled(
                custom_hex_color_input,
                general_style.fg(Color::Rgb(hex.0, hex.1, hex.2)),
            )]);
            Paragraph::new(styled_text).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(text_input_style)
                    .border_type(BorderType::Rounded),
            )
        } else {
            Paragraph::new(custom_hex_color_input)
                .style(general_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(text_input_style)
                        .border_type(BorderType::Rounded),
                )
        };

        let accept_key = app
            .get_first_keybinding(KeyBindingEnum::Accept)
            .unwrap_or("".to_string());

        let help_spans = vec![
            Span::styled("Press ", help_text_style),
            Span::styled(accept_key, help_key_style),
            Span::styled(" to submit.", help_text_style),
        ];

        let border_block = Block::default()
            .title("Custom Hex Color")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(general_style);

        let help_text = Paragraph::new(Line::from(help_spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(general_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        if app.state.app_status == AppStatus::UserInput {
            match app.state.z_stack.last() {
                Some(PopUp::CustomHexColorPromptFG) => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.theme_editor_fg_hex,
                        &app.config.show_line_numbers,
                        &input_field_chunks[0],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Some(PopUp::CustomHexColorPromptBG) => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.theme_editor_bg_hex,
                        &app.config.show_line_numbers,
                        &input_field_chunks[0],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                _ => {}
            }
        }

        render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);

        if app.config.enable_mouse_support {
            let submit_button_style = get_mouse_focusable_field_style(
                app,
                Focus::SubmitButton,
                &chunks[2],
                is_active,
                false,
            );
            let submit_button = Paragraph::new("Submit")
                .style(general_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(submit_button_style)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center);
            rect.render_widget(submit_button, chunks[2]);
            rect.render_widget(help_text, chunks[3]);
            render_close_button(rect, app, is_active);
        } else {
            rect.render_widget(help_text, chunks[2]);
        }

        if parsed_hex.is_none() {
            let invalid_text = Paragraph::new("Invalid Hex Color")
                .style(error_text_style)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(error_text_style)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center);
            rect.render_widget(invalid_text, input_field_chunks[1]);
        }

        rect.render_widget(prompt_text, chunks[0]);
        rect.render_widget(text_input, input_field_chunks[0]);
        rect.render_widget(border_block, popup_area);
    }
}
