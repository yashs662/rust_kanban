use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App,
    },
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_body, render_close_button},
            utils::{
                calculate_mouse_list_select_index, check_if_active_and_get_style,
                check_if_mouse_is_in_area,
            },
            view::LoadCloudSave,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

impl Renderable for LoadCloudSave {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let default_style = check_if_active_and_get_style(
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
        let list_select_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.list_select_style,
        );
        let main_chunks = {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(rect.size())
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Fill(1),
                    Constraint::Length(9),
                ]
                .as_ref(),
            )
            .split(main_chunks[0]);

        let preview_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
            .split(main_chunks[1]);

        let title_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
            .split(preview_chunks[0]);

        let title_paragraph = Paragraph::new("Load a Save (Cloud)")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(default_style);
        rect.render_widget(title_paragraph, chunks[0]);

        if let Some(item_list) = &app.state.cloud_data {
            if item_list.is_empty() {
                let no_saves_paragraph = Paragraph::new("No saves Found")
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    )
                    .style(error_text_style);
                rect.render_widget(no_saves_paragraph, chunks[1]);
            } else {
                let items: Vec<ListItem> = item_list
                    .iter()
                    .map(|i| ListItem::new(format!("cloud_save_{}", i.save_id)))
                    .collect();
                let choice_list = List::new(items)
                    .block(
                        Block::default()
                            .title(format!("Available Saves ({})", item_list.len()))
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    )
                    .highlight_style(list_select_style)
                    .highlight_symbol(LIST_SELECTED_SYMBOL)
                    .style(default_style);

                if is_active
                    && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1])
                {
                    app.state.mouse_focus = Some(Focus::LoadSave);
                    calculate_mouse_list_select_index(
                        app.state.current_mouse_coordinates.1,
                        item_list,
                        chunks[1],
                        &mut app.state.app_list_states.load_save,
                    );
                    app.state.set_focus(Focus::LoadSave);
                }
                rect.render_stateful_widget(
                    choice_list,
                    chunks[1],
                    &mut app.state.app_list_states.load_save,
                );
            }
        } else {
            let no_saves_paragraph = Paragraph::new("Waiting for data from the cloud...")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(error_text_style);
            rect.render_widget(no_saves_paragraph, chunks[1]);
        }

        let up_key = app
            .get_first_keybinding(KeyBindingEnum::Up)
            .unwrap_or("".to_string());
        let down_key = app
            .get_first_keybinding(KeyBindingEnum::Down)
            .unwrap_or("".to_string());
        let delete_key = app
            .get_first_keybinding(KeyBindingEnum::DeleteCard)
            .unwrap_or("".to_string());
        let accept_key = app
            .get_first_keybinding(KeyBindingEnum::Accept)
            .unwrap_or("".to_string());
        let cancel_key = app
            .get_first_keybinding(KeyBindingEnum::GoToPreviousViewOrCancel)
            .unwrap_or("".to_string());

        let help_text = Line::from(vec![
            Span::styled("Use ", help_text_style),
            Span::styled(&up_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(&down_key, help_key_style),
            Span::styled(" to navigate. Press ", help_text_style),
            Span::styled(&accept_key, help_key_style),
            Span::styled(" to Load the selected save file. Press ", help_text_style),
            Span::styled(&cancel_key, help_key_style),
            Span::styled(" to cancel. Press ", help_text_style),
            Span::styled(delete_key, help_key_style),
            Span::styled(
                " to delete a save file. If using a mouse click on a save file to preview",
                help_text_style,
            ),
        ]);
        let help_paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(default_style)
            .wrap(ratatui::widgets::Wrap { trim: true });
        rect.render_widget(help_paragraph, chunks[2]);

        if app.state.app_list_states.load_save.selected().is_none() {
            let preview_paragraph =
                Paragraph::new(format!("Select a save file with {}or {}to preview or Click on a save file to preview if using a mouse", up_key, down_key))
                    .alignment(Alignment::Center)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    )
                    .style(default_style)
                    .wrap(ratatui::widgets::Wrap { trim: true });
            rect.render_widget(preview_paragraph, preview_chunks[1]);
        } else if app.preview_boards_and_cards.is_none() {
            let loading_text = if app.config.enable_mouse_support {
                "Click on a save file to preview"
            } else {
                "Loading preview..."
            };
            let preview_paragraph = Paragraph::new(loading_text)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(default_style)
                .wrap(ratatui::widgets::Wrap { trim: true });
            rect.render_widget(preview_paragraph, preview_chunks[1]);
        } else {
            render_body(rect, preview_chunks[1], app, true, is_active)
        }

        let preview_title_paragraph = if let Some(file_name) = &app.state.preview_file_name {
            Paragraph::new("Previewing: ".to_string() + file_name)
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(default_style)
                .wrap(ratatui::widgets::Wrap { trim: true })
        } else {
            Paragraph::new("Select a file to preview")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(default_style)
                .wrap(ratatui::widgets::Wrap { trim: true })
        };

        if app.config.enable_mouse_support {
            rect.render_widget(preview_title_paragraph, title_bar_chunks[0]);
            render_close_button(rect, app, is_active);
        } else {
            rect.render_widget(preview_title_paragraph, preview_chunks[0]);
        }
    }
}
