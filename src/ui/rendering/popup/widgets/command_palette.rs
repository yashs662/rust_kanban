use crate::{
    app::{
        state::{AppStatus, Focus, KeyBindingEnum},
        App,
    },
    constants::{
        LIST_SELECTED_SYMBOL, SCROLLBAR_BEGIN_SYMBOL, SCROLLBAR_END_SYMBOL, SCROLLBAR_TRACK_SYMBOL,
    },
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::widgets::CommandPalette,
            utils::{
                calculate_viewport_corrected_cursor_position, check_if_active_and_get_style,
                check_if_mouse_is_in_area, get_scrollable_widget_row_bounds,
            },
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

impl Renderable for CommandPalette {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        // Housekeeping
        match app.state.focus {
            Focus::CommandPaletteCommand => {
                if app
                    .state
                    .app_list_states
                    .command_palette_command_search
                    .selected()
                    .is_none()
                {
                    if let Some(results) = &app.widgets.command_palette.command_search_results {
                        if !results.is_empty() {
                            app.state
                                .app_list_states
                                .command_palette_command_search
                                .select(Some(0));
                        }
                    }
                }
            }
            Focus::CommandPaletteCard => {
                if app
                    .state
                    .app_list_states
                    .command_palette_card_search
                    .selected()
                    .is_none()
                {
                    if let Some(results) = &app.widgets.command_palette.card_search_results {
                        if !results.is_empty() {
                            app.state
                                .app_list_states
                                .command_palette_card_search
                                .select(Some(0));
                        }
                    }
                }
            }
            Focus::CommandPaletteBoard => {
                if app
                    .state
                    .app_list_states
                    .command_palette_board_search
                    .selected()
                    .is_none()
                {
                    if let Some(results) = &app.widgets.command_palette.board_search_results {
                        if !results.is_empty() {
                            app.state
                                .app_list_states
                                .command_palette_board_search
                                .select(Some(0));
                        }
                    }
                }
            }
            _ => {
                if app.state.app_status != AppStatus::UserInput {
                    app.state.app_status = AppStatus::UserInput;
                }
            }
        }

        let current_search_text_input = app.state.text_buffers.command_palette.get_joined_lines();
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(10),
                    Constraint::Percentage(80),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(rect.area());

        fn get_command_palette_style(app: &App, focus: Focus) -> (Style, Style, Style) {
            if app.state.focus == focus {
                (
                    app.current_theme.keyboard_focus_style,
                    app.current_theme.general_style,
                    app.current_theme.list_select_style,
                )
            } else {
                (
                    app.current_theme.inactive_text_style,
                    app.current_theme.inactive_text_style,
                    app.current_theme.inactive_text_style,
                )
            }
        }

        let (
            command_search_border_style,
            command_search_text_style,
            command_search_highlight_style,
        ) = get_command_palette_style(app, Focus::CommandPaletteCommand);
        let (card_search_border_style, card_search_text_style, card_search_highlight_style) =
            get_command_palette_style(app, Focus::CommandPaletteCard);
        let (board_search_border_style, board_search_text_style, board_search_highlight_style) =
            get_command_palette_style(app, Focus::CommandPaletteBoard);
        let keyboard_focus_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.keyboard_focus_style,
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
        let progress_bar_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.progress_bar_style,
        );
        let rapid_blink_general_style = if is_active {
            general_style.add_modifier(Modifier::RAPID_BLINK)
        } else {
            general_style
        };

        let command_search_results =
            if let Some(raw_search_results) = &app.widgets.command_palette.command_search_results {
                let mut list_items = vec![];
                for item in raw_search_results {
                    let mut spans = vec![];
                    for c in item.to_string().chars() {
                        if current_search_text_input
                            .to_lowercase()
                            .contains(c.to_string().to_lowercase().as_str())
                        {
                            spans.push(Span::styled(c.to_string(), keyboard_focus_style));
                        } else {
                            spans.push(Span::styled(c.to_string(), command_search_text_style));
                        }
                    }
                    list_items.push(ListItem::new(Line::from(spans)));
                }
                list_items
            } else {
                app.widgets
                    .command_palette
                    .available_commands
                    .iter()
                    .map(|c| ListItem::new(Line::from(format!("Command - {}", c))))
                    .collect::<Vec<ListItem>>()
            };

        let card_search_results = if app.widgets.command_palette.card_search_results.is_some()
            && !current_search_text_input.is_empty()
            && current_search_text_input.len() > 1
        {
            let raw_search_results = app
                .widgets
                .command_palette
                .card_search_results
                .as_ref()
                .unwrap();
            let mut list_items = vec![];
            for (item, _) in raw_search_results {
                let item = if item.len() > (horizontal_chunks[1].width - 2) as usize {
                    format!("{}...", &item[0..(horizontal_chunks[1].width - 5) as usize])
                } else {
                    item.to_string()
                };
                list_items.push(ListItem::new(Line::from(Span::styled(
                    item.to_string(),
                    card_search_text_style,
                ))));
            }
            list_items
        } else {
            vec![]
        };

        let board_search_results = if app.widgets.command_palette.board_search_results.is_some()
            && !current_search_text_input.is_empty()
            && current_search_text_input.len() > 1
        {
            let raw_search_results = app
                .widgets
                .command_palette
                .board_search_results
                .as_ref()
                .unwrap();
            let mut list_items = vec![];
            for (item, _) in raw_search_results {
                let item = if item.len() > (horizontal_chunks[1].width - 2) as usize {
                    format!("{}...", &item[0..(horizontal_chunks[1].width - 5) as usize])
                } else {
                    item.to_string()
                };
                list_items.push(ListItem::new(Line::from(Span::styled(
                    item.to_string(),
                    board_search_text_style,
                ))));
            }
            list_items
        } else {
            vec![]
        };

        let max_height = if app.state.user_login_data.auth_token.is_some() {
            (rect.area().height - 14) as usize
        } else {
            (rect.area().height - 12) as usize
        };
        let min_height = 2;
        let command_search_results_length = command_search_results.len() + 2;
        let card_search_results_length = card_search_results.len() + 2;
        let board_search_results_length = board_search_results.len() + 2;
        let command_search_results_length = if command_search_results_length >= min_height {
            if (command_search_results_length + (2 * min_height)) < max_height {
                command_search_results_length
            } else {
                let calc = max_height - (2 * min_height);
                if calc < min_height {
                    min_height
                } else {
                    calc
                }
            }
        } else {
            min_height
        };
        let card_search_results_length = if card_search_results_length >= min_height {
            if (command_search_results_length + card_search_results_length + min_height)
                < max_height
            {
                card_search_results_length
            } else {
                let calc = max_height - (command_search_results_length + min_height);
                if calc < min_height {
                    min_height
                } else {
                    calc
                }
            }
        } else {
            min_height
        };
        let board_search_results_length = if board_search_results_length >= min_height {
            if (command_search_results_length
                + card_search_results_length
                + board_search_results_length)
                < max_height
            {
                board_search_results_length
            } else {
                let calc = max_height
                    - (command_search_results_length + card_search_results_length + min_height);
                if calc < min_height {
                    min_height
                } else {
                    calc
                }
            }
        } else {
            min_height
        };

        let vertical_chunks = if app.state.user_login_data.auth_token.is_some() {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(
                            ((command_search_results_length
                                + card_search_results_length
                                + board_search_results_length)
                                + 2) as u16,
                        ),
                        Constraint::Fill(1),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(horizontal_chunks[1])
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(3),
                        Constraint::Length(
                            ((command_search_results_length
                                + card_search_results_length
                                + board_search_results_length)
                                + 2) as u16,
                        ),
                        Constraint::Fill(1),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(horizontal_chunks[1])
        };

        let search_box_chunk = if app.state.user_login_data.auth_token.is_some() {
            vertical_chunks[2]
        } else {
            vertical_chunks[1]
        };

        let search_results_chunk = if app.state.user_login_data.auth_token.is_some() {
            vertical_chunks[3]
        } else {
            vertical_chunks[2]
        };

        let help_chunk = if app.state.user_login_data.auth_token.is_some() {
            vertical_chunks[5]
        } else {
            vertical_chunks[4]
        };

        if app.state.user_login_data.auth_token.is_some() {
            let logged_in_indicator = Paragraph::new(format!(
                "Logged in as: {}",
                app.state.user_login_data.email_id.clone().unwrap()
            ))
            .style(rapid_blink_general_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
            rect.render_widget(Clear, vertical_chunks[0]);
            rect.render_widget(logged_in_indicator, vertical_chunks[0]);
        }

        app.state
            .text_buffers
            .command_palette
            .set_placeholder_text("Start typing to search for a command, card or board!");

        let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
            &app.state.text_buffers.command_palette,
            &app.config.show_line_numbers,
            &search_box_chunk,
        );
        rect.set_cursor_position((x_pos, y_pos));

        let search_box_block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL)
            .style(general_style)
            .border_type(BorderType::Rounded);
        app.state
            .text_buffers
            .command_palette
            .set_block(search_box_block);

        render_blank_styled_canvas(rect, &app.current_theme, search_box_chunk, is_active);
        rect.render_widget(
            app.state.text_buffers.command_palette.widget(),
            search_box_chunk,
        );

        let results_border = Block::default()
            .style(general_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let search_results_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(command_search_results_length as u16),
                    Constraint::Min(card_search_results_length as u16),
                    Constraint::Min(board_search_results_length as u16),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(search_results_chunk);

        let command_search_results_list = List::new(command_search_results.clone())
            .block(
                Block::default()
                    .title("Commands")
                    .border_style(command_search_border_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(command_search_highlight_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);

        let card_search_results_list = List::new(card_search_results.clone())
            .block(
                Block::default()
                    .title("Cards")
                    .border_style(card_search_border_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(card_search_highlight_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);

        let board_search_results_list = List::new(board_search_results.clone())
            .block(
                Block::default()
                    .title("Boards")
                    .border_style(board_search_border_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(board_search_highlight_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);

        let up_key = app
            .get_first_keybinding(KeyBindingEnum::Up)
            .unwrap_or("".to_string());
        let down_key = app
            .get_first_keybinding(KeyBindingEnum::Down)
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

        let help_spans = Line::from(vec![
            Span::styled("Use ", help_text_style),
            Span::styled(up_key, help_key_style),
            Span::styled(" and ", help_text_style),
            Span::styled(down_key, help_key_style),
            Span::styled(
                " or scroll with the mouse to highlight a Command/Card/Board. Press ",
                help_text_style,
            ),
            Span::styled(accept_key, help_key_style),
            Span::styled(" to select. Press ", help_text_style),
            Span::styled(next_focus_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(prv_focus_key, help_key_style),
            Span::styled(" to change focus", help_text_style),
        ]);

        let help_paragraph = Paragraph::new(help_spans)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(general_style),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: false });

        if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &search_results_chunks[0],
        ) {
            app.state.mouse_focus = Some(Focus::CommandPaletteCommand);
            app.state.set_focus(Focus::CommandPaletteCommand);
        }
        if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &search_results_chunks[1],
        ) {
            app.state.mouse_focus = Some(Focus::CommandPaletteCard);
            app.state.set_focus(Focus::CommandPaletteCard);
        }
        if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &search_results_chunks[2],
        ) {
            app.state.mouse_focus = Some(Focus::CommandPaletteBoard);
            app.state.set_focus(Focus::CommandPaletteBoard);
        }

        render_blank_styled_canvas(rect, &app.current_theme, search_results_chunk, is_active);
        rect.render_widget(results_border, search_results_chunk);
        if app.state.focus != Focus::CommandPaletteCommand {
            render_blank_styled_canvas(
                rect,
                &app.current_theme,
                search_results_chunks[0],
                is_active,
            );
        }
        rect.render_stateful_widget(
            command_search_results_list,
            search_results_chunks[0],
            &mut app.state.app_list_states.command_palette_command_search,
        );
        if app.state.focus != Focus::CommandPaletteCard {
            render_blank_styled_canvas(
                rect,
                &app.current_theme,
                search_results_chunks[1],
                is_active,
            );
        }
        rect.render_stateful_widget(
            card_search_results_list,
            search_results_chunks[1],
            &mut app.state.app_list_states.command_palette_card_search,
        );
        if app.state.focus != Focus::CommandPaletteBoard {
            render_blank_styled_canvas(
                rect,
                &app.current_theme,
                search_results_chunks[2],
                is_active,
            );
        }
        rect.render_stateful_widget(
            board_search_results_list,
            search_results_chunks[2],
            &mut app.state.app_list_states.command_palette_board_search,
        );

        if app.state.focus == Focus::CommandPaletteCommand {
            let current_index = app
                .state
                .app_list_states
                .command_palette_command_search
                .selected()
                .unwrap_or(0);
            let (row_start_index, _) = get_scrollable_widget_row_bounds(
                command_search_results_length.saturating_sub(2),
                current_index,
                app.state
                    .app_list_states
                    .command_palette_command_search
                    .offset(),
                (search_results_chunks[0].height - 2) as usize,
            );
            let current_mouse_y_position = app.state.current_mouse_coordinates.1;
            let hovered_index = if current_mouse_y_position > search_results_chunks[0].y
                && current_mouse_y_position
                    < (search_results_chunks[0].y + search_results_chunks[0].height - 1)
            {
                Some(
                    ((current_mouse_y_position - search_results_chunks[0].y - 1)
                        + row_start_index as u16) as usize,
                )
            } else {
                None
            };
            if hovered_index.is_some()
                && (app.state.previous_mouse_coordinates != app.state.current_mouse_coordinates)
            {
                app.state
                    .app_list_states
                    .command_palette_command_search
                    .select(hovered_index);
            }
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
                .style(progress_bar_style)
                .end_symbol(SCROLLBAR_END_SYMBOL)
                .track_symbol(SCROLLBAR_TRACK_SYMBOL)
                .track_style(app.current_theme.inactive_text_style);

            let mut scrollbar_state =
                ScrollbarState::new(command_search_results.len()).position(current_index);
            let scrollbar_area = search_results_chunks[0].inner(Margin {
                horizontal: 0,
                vertical: 1,
            });
            rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        } else if app.state.focus == Focus::CommandPaletteCard {
            let current_index = app
                .state
                .app_list_states
                .command_palette_card_search
                .selected()
                .unwrap_or(0);
            let (row_start_index, _) = get_scrollable_widget_row_bounds(
                card_search_results_length.saturating_sub(2),
                current_index,
                app.state
                    .app_list_states
                    .command_palette_card_search
                    .offset(),
                (search_results_chunks[1].height - 2) as usize,
            );
            let current_mouse_y_position = app.state.current_mouse_coordinates.1;
            let hovered_index = if current_mouse_y_position > search_results_chunks[1].y
                && current_mouse_y_position
                    < (search_results_chunks[1].y + search_results_chunks[1].height - 1)
            {
                Some(
                    ((current_mouse_y_position - search_results_chunks[1].y - 1)
                        + row_start_index as u16) as usize,
                )
            } else {
                None
            };
            if hovered_index.is_some()
                && (app.state.previous_mouse_coordinates != app.state.current_mouse_coordinates)
            {
                app.state
                    .app_list_states
                    .command_palette_card_search
                    .select(hovered_index);
            }
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
                .style(progress_bar_style)
                .end_symbol(SCROLLBAR_END_SYMBOL)
                .track_symbol(SCROLLBAR_TRACK_SYMBOL)
                .track_style(app.current_theme.inactive_text_style);

            let mut scrollbar_state =
                ScrollbarState::new(card_search_results.len()).position(current_index);
            let scrollbar_area = search_results_chunks[1].inner(Margin {
                horizontal: 0,
                vertical: 1,
            });
            rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        } else if app.state.focus == Focus::CommandPaletteBoard {
            let current_index = app
                .state
                .app_list_states
                .command_palette_board_search
                .selected()
                .unwrap_or(0);
            let (row_start_index, _) = get_scrollable_widget_row_bounds(
                board_search_results_length.saturating_sub(2),
                current_index,
                app.state
                    .app_list_states
                    .command_palette_board_search
                    .offset(),
                (search_results_chunks[2].height - 2) as usize,
            );
            let current_mouse_y_position = app.state.current_mouse_coordinates.1;
            let hovered_index = if current_mouse_y_position > search_results_chunks[2].y
                && current_mouse_y_position
                    < (search_results_chunks[2].y + search_results_chunks[2].height - 1)
            {
                Some(
                    ((current_mouse_y_position - search_results_chunks[2].y - 1)
                        + row_start_index as u16) as usize,
                )
            } else {
                None
            };
            if hovered_index.is_some()
                && (app.state.previous_mouse_coordinates != app.state.current_mouse_coordinates)
            {
                app.state
                    .app_list_states
                    .command_palette_board_search
                    .select(hovered_index);
            }
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
                .style(progress_bar_style)
                .end_symbol(SCROLLBAR_END_SYMBOL)
                .track_symbol(SCROLLBAR_TRACK_SYMBOL)
                .track_style(app.current_theme.inactive_text_style);

            let mut scrollbar_state =
                ScrollbarState::new(board_search_results.len()).position(current_index);
            let scrollbar_area = search_results_chunks[2].inner(Margin {
                horizontal: 0,
                vertical: 1,
            });
            rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        }

        render_blank_styled_canvas(rect, &app.current_theme, help_chunk, is_active);
        rect.render_widget(help_paragraph, help_chunk);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
