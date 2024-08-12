use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App, ConfigEnum,
    },
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button, render_logs},
            popup::EditGeneralConfig,
            utils::{
                calculate_viewport_corrected_cursor_position, centered_rect_with_percentage,
                check_if_active_and_get_style, check_if_mouse_is_in_area,
                get_mouse_focusable_field_style,
            },
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use std::str::FromStr;

impl Renderable for EditGeneralConfig {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let area = centered_rect_with_percentage(70, 70, rect.area());

        let chunks = if app.config.enable_mouse_support {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(6),
                        Constraint::Fill(1),
                        Constraint::Length(5),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(6),
                        Constraint::Fill(1),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(area)
        };

        let edit_box_style = get_mouse_focusable_field_style(
            app,
            Focus::EditGeneralConfigPopup,
            &chunks[1],
            is_active,
            true,
        );
        let help_text_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.help_text_style,
        );
        let help_key_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.help_key_style,
        );
        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
        let error_text_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.error_text_style,
        );
        let card_status_active_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.card_status_active_style,
        );
        let keyboard_focus_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.keyboard_focus_style,
        );
        let mouse_focus_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.mouse_focus_style,
        );

        let list_items = app.config.to_view_list();
        let config_item_name = if let Some(index) = app.state.app_table_states.config.selected() {
            list_items[index].first().unwrap()
        } else {
            // NOTE: This is temporary, as only the Theme editor uses this other than config
            "Theme Name"
        };
        if let Ok(config_enum) = ConfigEnum::from_str(config_item_name) {
            app.state.path_check_state.path_check_mode = config_enum == ConfigEnum::SaveDirectory;
        }
        let config_item_value = if app.state.app_table_states.config.selected().is_some() {
            list_items
                .iter()
                .find(|x| x.first().unwrap() == config_item_name)
                .unwrap()
                .get(1)
                .unwrap()
                .to_owned()
        } else {
            app.state.theme_being_edited.name.clone()
        };

        let accept_key = app
            .get_first_keybinding(KeyBindingEnum::Accept)
            .unwrap_or("".to_string());
        let start_editing_key = app
            .get_first_keybinding(KeyBindingEnum::TakeUserInput)
            .unwrap_or("".to_string());
        let cancel_key = app
            .get_first_keybinding(KeyBindingEnum::GoToPreviousViewOrCancel)
            .unwrap_or("".to_string());
        let stop_editing_key = app
            .get_first_keybinding(KeyBindingEnum::StopUserInput)
            .unwrap_or("".to_string());

        let paragraph_text = vec![
            Line::from(vec![
                Span::styled("Current Value is '", help_text_style),
                Span::styled(config_item_value, help_key_style),
                Span::styled("'", help_text_style),
            ]),
            Line::from(String::from("")),
            Line::from(vec![
                Span::styled("Press ", help_text_style),
                Span::styled(start_editing_key, help_key_style),
                Span::styled(" to edit, or ", help_text_style),
                Span::styled(cancel_key, help_key_style),
                Span::styled(" to cancel, Press ", help_text_style),
                Span::styled(stop_editing_key, help_key_style),
                Span::styled(" to stop editing and press ", help_text_style),
                Span::styled(accept_key, help_key_style),
                Span::styled(" on Submit to save", help_text_style),
            ]),
        ];
        let paragraph_title = Line::from(vec![Span::raw(config_item_name)]);
        let config_item = Paragraph::new(paragraph_text)
            .block(
                Block::default()
                    .title(paragraph_title)
                    .style(general_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        let current_user_input = app.state.text_buffers.general_config.get_joined_lines();
        let user_input = if app.state.path_check_state.path_check_mode {
            if (current_user_input != app.state.path_check_state.path_last_checked)
                || app.state.path_check_state.recheck_required
            {
                app.state.path_check_state.recheck_required = false;
                app.state.path_check_state.potential_completion = None;
                app.state
                    .path_check_state
                    .path_last_checked
                    .clone_from(&current_user_input);
                app.state.path_check_state.path_exists =
                    std::path::Path::new(&current_user_input).is_dir();
                if !app.state.path_check_state.path_exists {
                    let mut split_input = current_user_input
                        .split(std::path::MAIN_SEPARATOR)
                        .collect::<Vec<&str>>();
                    // remove any empty strings
                    split_input.retain(|&x| !x.is_empty());
                    if !split_input.is_empty() {
                        let last_input = split_input.pop().unwrap();
                        if split_input.is_empty() {
                            let to_check = last_input;
                            let dir = std::fs::read_dir(std::path::MAIN_SEPARATOR.to_string());
                            if dir.is_ok() {
                                let dir = dir.unwrap();
                                // only retain the ones that are directories
                                let dir = dir.flatten();
                                for entry in dir {
                                    let path = entry.path();
                                    if path.to_str().unwrap().starts_with(
                                        &(std::path::MAIN_SEPARATOR.to_string() + to_check),
                                    ) && path.is_dir()
                                    {
                                        app.state.path_check_state.potential_completion = Some(
                                            path.to_str()
                                                .unwrap()
                                                .to_string()
                                                .strip_prefix(
                                                    &(std::path::MAIN_SEPARATOR.to_string()
                                                        + to_check),
                                                )
                                                .unwrap()
                                                .to_string(),
                                        );
                                        break;
                                    }
                                }
                            }
                        } else {
                            let to_check = std::path::MAIN_SEPARATOR.to_string()
                                + &split_input.join(std::path::MAIN_SEPARATOR_STR);
                            if std::path::Path::new(&to_check).is_dir() {
                                let dir = std::fs::read_dir(&to_check);
                                let dir = dir.unwrap();
                                // only retain the ones that are directories
                                let dir = dir.flatten();
                                for entry in dir {
                                    let path = entry.path();
                                    if path
                                        .to_str()
                                        .unwrap()
                                        .starts_with(current_user_input.as_str())
                                        && path.is_dir()
                                    {
                                        app.state.path_check_state.potential_completion = Some(
                                            path.to_str()
                                                .unwrap()
                                                .strip_prefix(current_user_input.as_str())
                                                .unwrap()
                                                .to_string(),
                                        );
                                        break;
                                    }
                                }
                            }
                        };
                    }
                }
            }
            if !current_user_input.is_empty() {
                if let Some(potential_completion) = &app.state.path_check_state.potential_completion
                {
                    Line::from(vec![
                        Span::styled(current_user_input.clone(), general_style),
                        Span::styled(
                            potential_completion.clone(),
                            app.current_theme.inactive_text_style,
                        ),
                        Span::styled(
                            " (Press 'Tab' or 'Right Arrow' to autocomplete)",
                            help_text_style,
                        ),
                    ])
                } else if app.state.path_check_state.path_exists {
                    Line::from(Span::styled(
                        current_user_input.clone(),
                        card_status_active_style,
                    ))
                } else {
                    Line::from(vec![
                        Span::styled(
                            current_user_input.clone(),
                            error_text_style,
                        ),
                        Span::styled(
                            " (Path does not exist) - Press '%' to create a new directory at this location",
                            help_text_style,
                        ),
                    ])
                }
            } else {
                Line::from(Span::styled(
                    "No input",
                    app.current_theme.inactive_text_style,
                ))
            }
        } else {
            Line::from(Span::styled(current_user_input, general_style))
        };
        let edit_item = Paragraph::new(user_input)
            .block(
                Block::default()
                    .title("Edit")
                    .style(general_style)
                    .borders(Borders::ALL)
                    .border_style(edit_box_style)
                    .border_type(BorderType::Rounded),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        let clear_area = centered_rect_with_percentage(80, 80, rect.area());
        let clear_area_border = Block::default()
            .title("Config Editor")
            .style(general_style)
            .borders(Borders::ALL)
            .border_style(keyboard_focus_style)
            .border_type(BorderType::Rounded);

        render_blank_styled_canvas(rect, &app.current_theme, clear_area, is_active);
        rect.render_widget(clear_area_border, clear_area);
        rect.render_widget(config_item, chunks[0]);
        rect.render_widget(edit_item, chunks[1]);
        render_logs(app, false, chunks[2], rect, is_active);

        if app.config.enable_mouse_support {
            let submit_button_style =
                if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[3]) {
                    app.state.mouse_focus = Some(Focus::SubmitButton);
                    app.state.set_focus(Focus::SubmitButton);
                    mouse_focus_style
                } else if app.state.app_status == AppStatus::KeyBindMode {
                    keyboard_focus_style
                } else {
                    general_style
                };
            let submit_button = Paragraph::new("Submit")
                .block(
                    Block::default()
                        .style(general_style)
                        .borders(Borders::ALL)
                        .border_style(submit_button_style)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center);
            rect.render_widget(submit_button, chunks[3]);
            render_close_button(rect, app, is_active)
        }

        if app.state.app_status == AppStatus::UserInput {
            let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                &app.state.text_buffers.general_config,
                &app.config.show_line_numbers,
                &chunks[1],
            );
            rect.set_cursor_position((x_pos, y_pos));
        }
    }
}
