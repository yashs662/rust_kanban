use crate::{
    app::{
        kanban::{CardPriority, CardStatus},
        state::{AppStatus, Focus},
        App, DateTimeFormat,
    },
    constants::FIELD_NOT_SET,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::ViewCard,
            utils::{
                calculate_viewport_corrected_cursor_position, centered_rect_with_percentage,
                check_if_active_and_get_style, check_if_mouse_is_in_area, get_button_style,
            },
        },
        PopUp, Renderable,
    },
    util::{date_format_converter, date_format_finder},
};
use chrono::{Local, NaiveDate, NaiveDateTime};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

impl Renderable for ViewCard {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let popup_area = centered_rect_with_percentage(90, 90, rect.size());
        // This is done early as board and card are not guaranteed to be selected
        render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);
        let error_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.error_text_style,
        );
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
        let list_select_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.list_select_style,
        );
        let card_tags_style = get_button_style(app, Focus::CardTags, None, is_active, false);
        let card_comments_style =
            get_button_style(app, Focus::CardComments, None, is_active, false);
        let save_changes_style = get_button_style(app, Focus::SubmitButton, None, is_active, false);
        let name_style = get_button_style(app, Focus::CardName, None, is_active, false);
        let description_style =
            get_button_style(app, Focus::CardDescription, None, is_active, false);
        let card_due_default_style =
            get_button_style(app, Focus::CardDueDate, None, is_active, false);
        if app.state.current_board_id.is_none() || app.state.current_card_id.is_none() {
            let no_board_or_card_selected = Paragraph::new("No board or card selected.")
                .block(
                    Block::default()
                        .title("Card Info")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(error_style),
                )
                .alignment(Alignment::Center);
            rect.render_widget(no_board_or_card_selected, popup_area);
            return;
        }

        let board = app
            .boards
            .get_board_with_id(app.state.current_board_id.unwrap());
        if board.is_none() {
            let could_not_find_board = Paragraph::new("Could not find board to view card.")
                .block(
                    Block::default()
                        .title("Card Info")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(error_style),
                )
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            rect.render_widget(could_not_find_board, popup_area);
            return;
        }

        let board = board.unwrap();
        let card = board
            .cards
            .get_card_with_id(app.state.current_card_id.unwrap());
        if card.is_none() {
            let could_not_find_card = Paragraph::new("Could not find card to view.")
                .block(
                    Block::default()
                        .title("Card Info")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .style(error_style),
                )
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });
            rect.render_widget(could_not_find_card, popup_area);
            return;
        }

        let card_being_edited = app.state.get_card_being_edited();
        let card = if let Some(card_being_edited) = card_being_edited {
            card_being_edited.1
        } else {
            card.unwrap().to_owned()
        };
        if app.widgets.date_time_picker.selected_date_time.is_none()
            && !card.due_date.is_empty()
            && card.due_date != FIELD_NOT_SET
        {
            if let Ok(current_format) = date_format_finder(card.due_date.trim()) {
                app.widgets.date_time_picker.selected_date_time = Some(
                    NaiveDateTime::parse_from_str(
                        card.due_date.trim(),
                        current_format.to_parser_string(),
                    )
                    .unwrap(),
                );
            }
        }
        let board_name = board.name.clone();
        let card_name = card.name.clone();

        // Prepare Main Block Widget
        let main_block_widget = {
            Block::default()
                .title(format!("{} >> Board({})", card_name, board_name))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(general_style)
        };

        // Prepare Name Block Widget
        let name_paragraph_block = Block::default()
            .title("Name")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(name_style);

        app.state
            .text_buffers
            .card_name
            .set_block(name_paragraph_block);

        // Process Card Description
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
            .set_block(description_block);

        // Process Card Extra Info
        let (card_extra_info_widget, card_extra_info_items_len, card_due_date_width) = {
            let card_date_created = if date_format_finder(&card.date_created).is_ok() {
                if let Ok(parsed_date) =
                    date_format_converter(&card.date_created, app.config.date_time_format)
                {
                    Span::styled(format!("Created: {}", parsed_date), general_style)
                } else {
                    Span::styled(format!("Created: {}", card.date_created), general_style)
                }
            } else {
                Span::styled(format!("Created: {}", card.date_created), general_style)
            };
            let card_date_modified = if date_format_finder(&card.date_modified).is_ok() {
                if let Ok(parsed_date) =
                    date_format_converter(&card.date_modified, app.config.date_time_format)
                {
                    Span::styled(format!("Modified: {}", parsed_date), general_style)
                } else {
                    Span::styled(format!("Modified: {}", card.date_modified), general_style)
                }
            } else {
                Span::styled(format!("Modified: {}", card.date_modified), general_style)
            };
            let card_date_completed = if date_format_finder(&card.date_completed).is_ok() {
                if let Ok(parsed_date) =
                    date_format_converter(&card.date_completed, app.config.date_time_format)
                {
                    Span::styled(format!("Completed: {}", parsed_date), general_style)
                } else {
                    Span::styled(format!("Completed: {}", card.date_completed), general_style)
                }
            } else {
                Span::styled(format!("Completed: {}", card.date_completed), general_style)
            };
            let card_priority = format!("Priority: {}", card.priority);
            let card_status = format!("Status: {}", card.card_status);
            let parsed_due_date = if date_format_finder(&card.due_date).is_ok() {
                date_format_converter(&card.due_date, app.config.date_time_format)
            } else {
                Ok(FIELD_NOT_SET.to_string())
            };
            let card_due_date_styled = if let Ok(parsed_due_date) = parsed_due_date {
                if app.state.focus == Focus::CardDueDate {
                    Span::styled(format!("Due: {}", parsed_due_date), list_select_style)
                } else if parsed_due_date == FIELD_NOT_SET || parsed_due_date.is_empty() {
                    Span::styled(format!("Due: {}", parsed_due_date), card_due_default_style)
                } else {
                    let formatted_date_format = date_format_finder(&parsed_due_date).unwrap();
                    let days_left = match formatted_date_format {
                        DateTimeFormat::DayMonthYear
                        | DateTimeFormat::MonthDayYear
                        | DateTimeFormat::YearMonthDay => {
                            let today = Local::now().date_naive();
                            let string_to_naive_date_format = NaiveDate::parse_from_str(
                                &parsed_due_date,
                                app.config.date_time_format.to_parser_string(),
                            )
                            .unwrap();
                            string_to_naive_date_format
                                .signed_duration_since(today)
                                .num_days()
                        }
                        DateTimeFormat::DayMonthYearTime
                        | DateTimeFormat::MonthDayYearTime
                        | DateTimeFormat::YearMonthDayTime {} => {
                            let today = Local::now().naive_local();
                            let string_to_naive_date_format = NaiveDateTime::parse_from_str(
                                &parsed_due_date,
                                app.config.date_time_format.to_parser_string(),
                            )
                            .unwrap();
                            string_to_naive_date_format
                                .signed_duration_since(today)
                                .num_days()
                        }
                    };
                    if !is_active {
                        Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.current_theme.inactive_text_style,
                        )
                    } else if days_left <= app.config.warning_delta.into() && days_left >= 0 {
                        Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.current_theme.card_due_warning_style,
                        )
                    } else if days_left < 0 {
                        Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.current_theme.card_due_overdue_style,
                        )
                    } else {
                        Span::styled(format!("Due: {}", parsed_due_date), card_due_default_style)
                    }
                }
            } else if app.state.focus == Focus::CardDueDate {
                Span::styled(format!("Due: {}", FIELD_NOT_SET), list_select_style)
            } else {
                Span::styled(format!("Due: {}", FIELD_NOT_SET), general_style)
            };
            let card_priority_styled = if !is_active {
                Span::styled(card_priority, app.current_theme.inactive_text_style)
            } else if app.state.focus == Focus::CardPriority {
                Span::styled(card_priority, app.current_theme.list_select_style)
            } else if card.priority == CardPriority::High {
                Span::styled(card_priority, app.current_theme.card_priority_high_style)
            } else if card.priority == CardPriority::Medium {
                Span::styled(card_priority, app.current_theme.card_priority_medium_style)
            } else if card.priority == CardPriority::Low {
                Span::styled(card_priority, app.current_theme.card_priority_low_style)
            } else {
                Span::styled(card_priority, app.current_theme.general_style)
            };
            let card_status_styled = if !is_active {
                Span::styled(card_status, app.current_theme.inactive_text_style)
            } else if app.state.focus == Focus::CardStatus {
                Span::styled(card_status, app.current_theme.list_select_style)
            } else if card.card_status == CardStatus::Complete {
                Span::styled(card_status, app.current_theme.card_status_completed_style)
            } else if card.card_status == CardStatus::Active {
                Span::styled(card_status, app.current_theme.card_status_active_style)
            } else if card.card_status == CardStatus::Stale {
                Span::styled(card_status, app.current_theme.card_status_stale_style)
            } else {
                Span::styled(card_status, app.current_theme.general_style)
            };
            let card_extra_info_items = vec![
                ListItem::new(vec![Line::from(card_date_created)]),
                ListItem::new(vec![Line::from(card_date_modified)]),
                ListItem::new(vec![Line::from(card_due_date_styled.clone())]),
                ListItem::new(vec![Line::from(card_date_completed)]),
                ListItem::new(vec![Line::from(card_priority_styled)]),
                ListItem::new(vec![Line::from(card_status_styled)]),
            ];
            let card_extra_info_items_len = card_extra_info_items.len();
            let card_extra_info = List::new(card_extra_info_items).block(
                Block::default()
                    .title("Card Info")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(general_style),
            );
            (
                card_extra_info,
                card_extra_info_items_len,
                card_due_date_styled.width(),
            )
        };

        // TODO: Refactor card tag and comments processing
        // Process Card Tags
        let card_tag_lines = {
            let card_tags = if app.state.focus == Focus::CardTags {
                let mut tags = vec![];
                if app
                    .state
                    .app_list_states
                    .card_view_tag_list
                    .selected()
                    .is_none()
                {
                    for (index, tag) in card.tags.iter().enumerate() {
                        tags.push(Span::styled(
                            format!("{}) {} ", index + 1, tag),
                            general_style,
                        ));
                    }
                } else {
                    let selected_tag = app
                        .state
                        .app_list_states
                        .card_view_tag_list
                        .selected()
                        .unwrap();
                    for (index, tag) in card.tags.iter().enumerate() {
                        if index == selected_tag {
                            tags.push(Span::styled(
                                format!("{}) {} ", index + 1, tag),
                                keyboard_focus_style,
                            ));
                        } else {
                            tags.push(Span::styled(
                                format!("{}) {} ", index + 1, tag),
                                general_style,
                            ));
                        }
                    }
                }
                tags
            } else {
                let mut tags = vec![];
                for (index, tag) in card.tags.iter().enumerate() {
                    tags.push(Span::styled(
                        format!("{}) {} ", index + 1, tag),
                        general_style,
                    ));
                }
                tags
            };
            let mut card_tag_lines = vec![];
            let mut card_tags_per_line = vec![];
            let mut collector = String::new();
            let mut collector_start = 0;
            let mut collector_end = 0;
            for (i, tag) in card.tags.iter().enumerate() {
                let tag_string = format!("{}) {} ", i + 1, tag);
                if (collector.len() + tag_string.len()) < (popup_area.width - 2) as usize {
                    collector.push_str(&tag_string);
                    collector_end = i + 1;
                } else {
                    card_tag_lines.push(Line::from(
                        card_tags[collector_start..collector_end].to_vec(),
                    ));
                    card_tags_per_line.push(collector_end - collector_start);
                    collector = String::new();
                    collector.push_str(&tag_string);
                    collector_start = i;
                    collector_end = i + 1;
                }
            }
            if !collector.is_empty() {
                card_tag_lines.push(Line::from(
                    card_tags[collector_start..collector_end].to_vec(),
                ));
            }
            card_tag_lines
        };

        // Process Card Comments
        let card_comment_lines = {
            let card_comments = if app.state.focus == Focus::CardComments {
                let mut comments = vec![];
                if app
                    .state
                    .app_list_states
                    .card_view_comment_list
                    .selected()
                    .is_none()
                {
                    for (index, comment) in card.comments.iter().enumerate() {
                        comments.push(Span::styled(
                            format!("{}) {} ", index + 1, comment),
                            general_style,
                        ));
                    }
                } else {
                    let selected_comment = app
                        .state
                        .app_list_states
                        .card_view_comment_list
                        .selected()
                        .unwrap();
                    for (index, comment) in card.comments.iter().enumerate() {
                        if index == selected_comment {
                            comments.push(Span::styled(
                                format!("{}) {} ", index + 1, comment),
                                keyboard_focus_style,
                            ));
                        } else {
                            comments.push(Span::styled(
                                format!("{}) {} ", index + 1, comment),
                                general_style,
                            ));
                        }
                    }
                }
                comments
            } else {
                let mut comments = vec![];
                for (index, comment) in card.comments.iter().enumerate() {
                    comments.push(Span::styled(
                        format!("{}) {} ", index + 1, comment),
                        general_style,
                    ));
                }
                comments
            };
            let mut card_comment_lines = vec![];
            let mut collector = String::new();
            let mut collector_start = 0;
            let mut collector_end = 0;
            for (i, comment) in card.comments.iter().enumerate() {
                let comment_string = format!("{}) {} ", i + 1, comment);
                if (collector.len() + comment_string.len()) < (popup_area.width - 2) as usize {
                    collector.push_str(&comment_string);
                    collector_end = i + 1;
                } else {
                    card_comment_lines.push(Line::from(
                        card_comments[collector_start..collector_end].to_vec(),
                    ));
                    collector = String::new();
                    collector.push_str(&comment_string);
                    collector_start = i;
                    collector_end = i + 1;
                }
            }
            if !collector.is_empty() {
                card_comment_lines.push(Line::from(
                    card_comments[collector_start..collector_end].to_vec(),
                ));
            }
            card_comment_lines
        };

        // Determine chunk sizes
        let card_chunks = {
            let min_box_height: u16 = 2;
            let border_height: u16 = 2;
            let max_height: u16 = popup_area.height - border_height;
            let submit_button_height: u16 = 3;
            let card_name_box_height: u16 = 3;
            let card_extra_info_height: u16 = 8;
            let mut available_height: u16 = if app.state.card_being_edited.is_some() {
                max_height - card_name_box_height - card_extra_info_height - submit_button_height
            } else {
                max_height - card_name_box_height - card_extra_info_height
            };

            let raw_card_description_height =
                app.state.text_buffers.card_description.get_num_lines() as u16;

            let raw_tags_height = card_tag_lines.len() as u16;
            let raw_comments_height = card_comment_lines.len() as u16;

            let mut card_description_height = if app.state.focus == Focus::CardDescription {
                if available_height
                    .saturating_sub(raw_tags_height + border_height)
                    .saturating_sub(raw_comments_height + border_height)
                    > 0
                {
                    let calc = available_height
                        - raw_tags_height
                        - raw_comments_height
                        - (border_height * 2);
                    if calc < (raw_card_description_height + border_height) {
                        let diff = (raw_card_description_height + border_height) - calc;
                        if diff < min_box_height {
                            raw_card_description_height + border_height
                        } else {
                            calc
                        }
                    } else {
                        calc
                    }
                } else if (raw_card_description_height + border_height) <= available_height {
                    raw_card_description_height + border_height
                } else {
                    available_height
                }
            } else if ((raw_card_description_height + border_height) <= available_height)
                && app.state.focus != Focus::CardTags
                && app.state.focus != Focus::CardComments
            {
                raw_card_description_height.saturating_sub(border_height)
            } else {
                min_box_height
            };

            available_height = available_height.saturating_sub(card_description_height);

            let card_tags_height = if available_height > 0 {
                if app.state.focus == Focus::CardTags {
                    raw_tags_height + border_height
                } else {
                    min_box_height
                }
            } else {
                min_box_height
            };

            available_height = available_height.saturating_sub(card_tags_height);

            let card_comments_height = if available_height > 0 {
                if app.state.focus == Focus::CardComments {
                    raw_comments_height + border_height
                } else {
                    min_box_height
                }
            } else {
                min_box_height
            };

            available_height = available_height.saturating_sub(card_comments_height);

            if available_height > 0 {
                card_description_height += available_height;
            }

            if app.state.card_being_edited.is_some() {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(card_name_box_height),
                        Constraint::Length(card_description_height),
                        Constraint::Length(card_extra_info_height),
                        Constraint::Length(card_tags_height),
                        Constraint::Length(card_comments_height),
                        Constraint::Length(submit_button_height),
                    ])
                    .margin(1)
                    .split(popup_area)
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(card_name_box_height),
                        Constraint::Length(card_description_height),
                        Constraint::Length(card_extra_info_height),
                        Constraint::Length(card_tags_height),
                        Constraint::Length(card_comments_height),
                    ])
                    .margin(1)
                    .split(popup_area)
            }
        };

        if app.state.z_stack.last() == Some(&PopUp::DateTimePicker) {
            if app.widgets.date_time_picker.anchor.is_none() {
                app.widgets.date_time_picker.anchor = Some((
                    card_chunks[2].x + card_due_date_width as u16 + 2,
                    card_chunks[2].y + 3,
                )); // offsets to make sure date is visible
                log::debug!(
                    "Setting anchor for date time picker to: {:?}",
                    app.widgets.date_time_picker.anchor
                );
            }
            app.widgets.date_time_picker.current_viewport = Some(rect.size());
        }

        if is_active
            && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[2])
        {
            let top_of_list = card_chunks[2].y + 1;
            let mut bottom_of_list = card_chunks[2].y + card_extra_info_items_len as u16;
            if bottom_of_list > card_chunks[2].bottom() {
                bottom_of_list = card_chunks[2].bottom();
            }
            let mouse_y = app.state.current_mouse_coordinates.1;
            if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
                match mouse_y - top_of_list {
                    2 => {
                        app.state.set_focus(Focus::CardDueDate);
                        app.state.mouse_focus = Some(Focus::CardDueDate);
                        app.state
                            .app_list_states
                            .card_view_comment_list
                            .select(None);
                        app.state.app_list_states.card_view_tag_list.select(None);
                    }
                    4 => {
                        app.state.set_focus(Focus::CardPriority);
                        app.state.mouse_focus = Some(Focus::CardPriority);
                        app.state
                            .app_list_states
                            .card_view_comment_list
                            .select(None);
                        app.state.app_list_states.card_view_tag_list.select(None);
                    }
                    5 => {
                        app.state.set_focus(Focus::CardStatus);
                        app.state.mouse_focus = Some(Focus::CardStatus);
                        app.state
                            .app_list_states
                            .card_view_comment_list
                            .select(None);
                        app.state.app_list_states.card_view_tag_list.select(None);
                    }
                    _ => {
                        app.state.set_focus(Focus::NoFocus);
                        app.state.mouse_focus = None;
                    }
                }
                app.state
                    .app_list_states
                    .card_view_list
                    .select(Some((mouse_y - top_of_list) as usize));
            } else {
                app.state.app_list_states.card_view_list.select(None);
            }
        };
        if is_active
            && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[0])
        {
            app.state.set_focus(Focus::CardName);
            app.state.mouse_focus = Some(Focus::CardName);
            app.state
                .app_list_states
                .card_view_comment_list
                .select(None);
            app.state.app_list_states.card_view_tag_list.select(None);
        }
        if is_active
            && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[1])
        {
            app.state.set_focus(Focus::CardDescription);
            app.state.mouse_focus = Some(Focus::CardDescription);
            app.state
                .app_list_states
                .card_view_comment_list
                .select(None);
            app.state.app_list_states.card_view_tag_list.select(None);
        }

        let card_tags_widget = Paragraph::new(card_tag_lines.clone())
            .block(
                Block::default()
                    .title(format!("Tags ({})", card.tags.len()))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(card_tags_style),
            )
            .alignment(Alignment::Left);

        let card_comments_widget = Paragraph::new(card_comment_lines.clone())
            .block(
                Block::default()
                    .title(format!("Comments ({})", card.comments.len()))
                    .border_type(BorderType::Rounded)
                    .borders(Borders::ALL)
                    .border_style(card_comments_style),
            )
            .alignment(Alignment::Left);

        if is_active
            && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[3])
        {
            app.state.set_focus(Focus::CardTags);
            app.state.mouse_focus = Some(Focus::CardTags);
            app.state
                .app_list_states
                .card_view_comment_list
                .select(None);
        }

        if is_active
            && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[4])
        {
            app.state.set_focus(Focus::CardComments);
            app.state.mouse_focus = Some(Focus::CardComments);
            app.state.app_list_states.card_view_tag_list.select(None);
        }

        if app.state.app_status == AppStatus::UserInput {
            match app.state.focus {
                Focus::CardName => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.card_name,
                        &app.config.show_line_numbers,
                        &card_chunks[0],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::CardDescription => {
                    let (x_pos, y_pos) = calculate_viewport_corrected_cursor_position(
                        &app.state.text_buffers.card_description,
                        &app.config.show_line_numbers,
                        &card_chunks[1],
                    );
                    rect.set_cursor(x_pos, y_pos);
                }
                Focus::CardTags => {
                    if app
                        .state
                        .app_list_states
                        .card_view_tag_list
                        .selected()
                        .is_some()
                        && !app.state.text_buffers.card_tags.is_empty()
                    {
                        let selected_index = app
                            .state
                            .app_list_states
                            .card_view_tag_list
                            .selected()
                            .unwrap();
                        let mut counter = 0;
                        let mut y_index = 0;
                        let mut length_before_selected_tag = 0;
                        let mut prv_spans_length = 0;
                        let tag_offset = 3;
                        for line in card_tag_lines.iter() {
                            for _ in line.spans.iter() {
                                if counter == selected_index {
                                    break;
                                } else {
                                    let element = line.spans.get(counter - prv_spans_length);
                                    if let Some(element) = element {
                                        length_before_selected_tag += element.content.len();
                                    }
                                    counter += 1;
                                }
                            }
                            if counter == selected_index {
                                break;
                            }
                            y_index += 1;
                            prv_spans_length += line.spans.iter().len();
                            length_before_selected_tag = 0;
                        }
                        let digits_in_counter = (counter + 1).to_string().len();
                        let text_box_cursor = app
                            .state
                            .text_buffers
                            .card_tags
                            .get(selected_index)
                            .unwrap()
                            .cursor();
                        let x_pos = card_chunks[3].left()
                            + length_before_selected_tag as u16
                            + text_box_cursor.1 as u16
                            + tag_offset
                            + digits_in_counter as u16;
                        let y_pos = card_chunks[3].top() + y_index as u16 + 1;
                        // TODO: Card tags and comments cursor is incorrect as the view does not change when the comment or tag is longer than the screen
                        rect.set_cursor(x_pos, y_pos);
                    }
                }
                Focus::CardComments => {
                    if app
                        .state
                        .app_list_states
                        .card_view_comment_list
                        .selected()
                        .is_some()
                        && !app.state.text_buffers.card_comments.is_empty()
                    {
                        let selected_index = app
                            .state
                            .app_list_states
                            .card_view_comment_list
                            .selected()
                            .unwrap();
                        let mut counter = 0;
                        let mut y_index = 0;
                        let mut length_before_selected_comment = 0;
                        let mut prv_spans_length = 0;
                        let comment_offset = 3;
                        for line in card_comment_lines.iter() {
                            for _ in line.spans.iter() {
                                if counter == selected_index {
                                    break;
                                } else {
                                    let element = line.spans.get(counter - prv_spans_length);
                                    if let Some(element) = element {
                                        length_before_selected_comment += element.content.len();
                                    }
                                    counter += 1;
                                }
                            }
                            if counter == selected_index {
                                break;
                            }
                            y_index += 1;
                            prv_spans_length += line.spans.iter().len();
                            length_before_selected_comment = 0;
                        }
                        let digits_in_counter = (counter + 1).to_string().len();
                        let text_box_cursor = app
                            .state
                            .text_buffers
                            .card_comments
                            .get(selected_index)
                            .unwrap()
                            .cursor();
                        let x_pos = card_chunks[4].left()
                            + length_before_selected_comment as u16
                            + text_box_cursor.1 as u16
                            + comment_offset
                            + digits_in_counter as u16;
                        let y_pos = card_chunks[4].top() + y_index as u16 + 1;
                        rect.set_cursor(x_pos, y_pos);
                    }
                }
                _ => {}
            }
        }

        // Render everything
        rect.render_widget(main_block_widget, popup_area);
        rect.render_widget(app.state.text_buffers.card_name.widget(), card_chunks[0]);
        rect.render_widget(
            app.state.text_buffers.card_description.widget(),
            card_chunks[1],
        );
        rect.render_widget(card_extra_info_widget, card_chunks[2]);
        rect.render_widget(card_tags_widget, card_chunks[3]);
        rect.render_widget(card_comments_widget, card_chunks[4]);

        // Render Submit button if card is being edited
        if app.state.card_being_edited.is_some() {
            if is_active
                && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[5])
            {
                app.state.set_focus(Focus::SubmitButton);
                app.state.mouse_focus = Some(Focus::SubmitButton);
                app.state
                    .app_list_states
                    .card_view_comment_list
                    .select(None);
                app.state.app_list_states.card_view_tag_list.select(None);
            }
            let save_changes_button = Paragraph::new("Save Changes")
                .block(
                    Block::default()
                        .title("Save Changes")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(save_changes_style),
                )
                .alignment(Alignment::Center);
            rect.render_widget(save_changes_button, card_chunks[5]);
        }

        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
