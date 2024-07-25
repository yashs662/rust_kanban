use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App,
    },
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button, render_logs},
            popup::EditSpecificKeybinding,
            utils::{
                centered_rect_with_percentage, check_if_active_and_get_style,
                check_if_mouse_is_in_area,
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

impl Renderable for EditSpecificKeybinding {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let area = centered_rect_with_percentage(70, 70, rect.size());

        let chunks = if app.config.enable_mouse_support {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(6),
                        Constraint::Fill(1),
                        Constraint::Length(4),
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

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
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

        let edit_box_style =
            if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1]) {
                app.state.mouse_focus = Some(Focus::EditSpecificKeyBindingPopup);
                app.state.set_focus(Focus::EditSpecificKeyBindingPopup);
                mouse_focus_style
            } else if app.state.app_status == AppStatus::KeyBindMode {
                keyboard_focus_style
            } else {
                general_style
            };

        let key_id = app
            .state
            .app_table_states
            .edit_keybindings
            .selected()
            .unwrap_or(0);
        let current_bindings = app.config.keybindings.clone();
        let mut key_list = vec![];

        for (k, v) in current_bindings.iter() {
            key_list.push((k, v));
        }

        if key_id > key_list.len() {
            return;
        }

        key_list.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));

        let paragraph_title = current_bindings
            .keybinding_enum_to_action(key_list[key_id].0.clone())
            .to_string();
        let value = key_list[key_id].1;
        let mut key_value = String::new();
        for v in value.iter() {
            key_value.push_str(v.to_string().as_str());
            if value.iter().last().unwrap() != v {
                key_value.push_str(", ");
            }
        }
        let user_input_key = app
            .get_first_keybinding(KeyBindingEnum::TakeUserInput)
            .unwrap_or("".to_string());
        let accept_key = app
            .get_first_keybinding(KeyBindingEnum::Accept)
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
                Span::styled(key_value, help_key_style),
                Span::styled("'", help_text_style),
            ]),
            Line::from(String::from("")),
            Line::from(vec![
                Span::styled("Press ", help_text_style),
                Span::styled(user_input_key, help_key_style),
                Span::styled(" to edit, ", help_text_style),
                Span::styled(cancel_key, help_key_style),
                Span::styled(" to cancel, ", help_text_style),
                Span::styled(stop_editing_key, help_key_style),
                Span::styled(" to stop editing and ", help_text_style),
                Span::styled(accept_key, help_key_style),
                Span::styled(" to save when stopped editing", help_text_style),
            ]),
        ];
        let config_item = Paragraph::new(paragraph_text)
            .block(
                Block::default()
                    .title(paragraph_title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });
        let current_edited_keybinding = app.state.edited_keybinding.clone();
        let mut current_edited_keybinding_string = String::new();
        if let Some(current_edited_keybinding) = current_edited_keybinding {
            for key in current_edited_keybinding {
                current_edited_keybinding_string.push_str(&key.to_string());
                current_edited_keybinding_string.push(' ');
            }
        }
        let edit_item = Paragraph::new(current_edited_keybinding_string.clone())
            .block(
                Block::default()
                    .title("Edit")
                    .borders(Borders::ALL)
                    .border_style(edit_box_style)
                    .border_type(BorderType::Rounded),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        let clear_area = centered_rect_with_percentage(80, 80, rect.size());
        let clear_area_border = Block::default()
            .title("Edit Keybindings")
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
                        .borders(Borders::ALL)
                        .border_style(submit_button_style)
                        .border_type(BorderType::Rounded),
                )
                .alignment(Alignment::Center);
            rect.render_widget(submit_button, chunks[3]);
            render_close_button(rect, app, is_active);
        }
    }
}
