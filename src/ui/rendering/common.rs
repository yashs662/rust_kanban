use crate::{
    app::{
        app_helper::reset_card_drag_mode,
        kanban::{Boards, Card, CardPriority, CardStatus},
        state::{Focus, KeyBindingEnum},
        App, DateTimeFormat,
    },
    constants::{
        APP_TITLE, DEFAULT_BOARD_TITLE_LENGTH, DEFAULT_CARD_TITLE_LENGTH, FIELD_NOT_SET,
        HIDDEN_PASSWORD_SYMBOL, LIST_SELECTED_SYMBOL, MOUSE_OUT_OF_BOUNDS_COORDINATES,
        PATTERN_CHANGE_INTERVAL, SCROLLBAR_BEGIN_SYMBOL, SCROLLBAR_END_SYMBOL,
        SCROLLBAR_TRACK_SYMBOL,
    },
    io::logger::{get_logs, get_selected_index, RUST_KANBAN_LOGGER},
    ui::{
        rendering::utils::{
            centered_rect_with_length, check_for_card_drag_and_get_style,
            check_if_active_and_get_style, check_if_mouse_is_in_area,
            get_mouse_focusable_field_style,
        },
        theme::Theme,
    },
    util::{date_format_converter, date_format_finder},
};
use chrono::{Local, NaiveDate, NaiveDateTime};
use log::Level;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
    Frame,
};
use std::{
    cmp::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn render_body(
    rect: &mut Frame,
    area: Rect,
    app: &mut App,
    preview_mode: bool,
    is_active: bool,
) {
    let mut current_board_set = false;
    let mut current_card_set = false;
    let app_preview_boards_and_cards = app.preview_boards_and_cards.clone().unwrap_or_default();
    let boards = if preview_mode {
        if app_preview_boards_and_cards.is_empty() {
            Boards::default()
        } else {
            app_preview_boards_and_cards
        }
    } else if !app.filtered_boards.is_empty() {
        app.filtered_boards.clone()
    } else {
        app.boards.clone()
    };
    let scrollbar_style = check_for_card_drag_and_get_style(
        app.state.card_drag_mode,
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.progress_bar_style,
    );
    let error_text_style = check_for_card_drag_and_get_style(
        app.state.card_drag_mode,
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.error_text_style,
    );
    let current_board_id = &app.state.current_board_id.unwrap_or((0, 0));

    let new_board_key = app
        .get_first_keybinding(KeyBindingEnum::NewBoard)
        .unwrap_or("".to_string());
    let new_card_key = app
        .get_first_keybinding(KeyBindingEnum::NewCard)
        .unwrap_or("".to_string());

    if preview_mode {
        if app.preview_boards_and_cards.is_none()
            || app
                .preview_boards_and_cards
                .as_ref()
                .map_or(false, |v| v.is_empty())
        {
            let empty_paragraph = Paragraph::new("No boards found".to_string())
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .title("Boards")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(error_text_style);
            rect.render_widget(empty_paragraph, area);
            return;
        }
    } else if app.visible_boards_and_cards.is_empty() {
        let empty_paragraph = Paragraph::new(
            [
                "No boards found, press ".to_string(),
                new_board_key,
                " to add a new board".to_string(),
            ]
            .concat(),
        )
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Boards")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(error_text_style);
        rect.render_widget(empty_paragraph, area);
        return;
    }

    let filter_chunks = if app.filtered_boards.is_empty() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(0), Constraint::Fill(1)].as_ref())
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Fill(1)].as_ref())
            .split(area)
    };

    let chunks = if app.config.disable_scroll_bar {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1)].as_ref())
            .split(filter_chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(1)].as_ref())
            .split(filter_chunks[1])
    };

    if !app.filtered_boards.is_empty() {
        let filtered_text = "This is a filtered view, Clear filter to see all boards and cards";
        let filtered_paragraph = Paragraph::new(filtered_text.to_string())
            .alignment(Alignment::Center)
            .block(Block::default())
            .style(error_text_style);
        rect.render_widget(filtered_paragraph, filter_chunks[0]);
    }

    let mut constraints = vec![];
    if boards.len() > app.config.no_of_boards_to_show.into() {
        for _i in 0..app.config.no_of_boards_to_show {
            constraints.push(Constraint::Fill(1));
        }
    } else {
        for _i in 0..boards.len() {
            constraints.push(Constraint::Fill(1));
        }
    }
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(AsRef::<[Constraint]>::as_ref(&constraints))
        .split(chunks[0]);
    let visible_boards_and_cards = if preview_mode {
        app.state.preview_visible_boards_and_cards.clone()
    } else {
        app.visible_boards_and_cards.clone()
    };
    for (board_index, board_and_card_tuple) in visible_boards_and_cards.iter().enumerate() {
        let board_id = board_and_card_tuple.0;
        let board = boards.get_board_with_id(*board_id);
        if board.is_none() {
            continue;
        }
        let board = board.unwrap();
        let board_title = board.name.clone();
        let board_cards = board_and_card_tuple.1;
        let board_title = if board_title.len() > DEFAULT_BOARD_TITLE_LENGTH.into() {
            format!(
                "{}...",
                &board_title[0..DEFAULT_BOARD_TITLE_LENGTH as usize]
            )
        } else {
            board_title
        };
        let board_title = format!("{} ({})", board_title, board.cards.len());
        let board_title = if board_id == current_board_id {
            format!("{} {}", ">>", board_title)
        } else {
            board_title
        };

        let mut card_constraints = vec![];
        if board_cards.len() > app.config.no_of_cards_to_show.into() {
            for _i in 0..app.config.no_of_cards_to_show {
                card_constraints.push(Constraint::Fill(1));
            }
        } else if board_cards.is_empty() {
            card_constraints.push(Constraint::Fill(1));
        } else {
            for _i in 0..board_cards.len() {
                card_constraints.push(Constraint::Fill(1));
            }
        }

        if board_index >= board_chunks.len() {
            continue;
        }

        let board_style = check_for_card_drag_and_get_style(
            app.state.card_drag_mode,
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
        // Exception to not using check_for_card_drag_and_get_style as we have to manage other state
        let board_border_style = if !is_active {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &board_chunks[board_index],
        ) {
            app.state.mouse_focus = Some(Focus::Body);
            app.state.set_focus(Focus::Body);
            if !current_board_set {
                app.state.current_board_id = Some(*board_id);
                current_board_set = true;
            }
            app.state.hovered_board = Some(*board_id);
            app.current_theme.mouse_focus_style
        } else if (app.state.current_board_id.unwrap_or((0, 0)) == *board_id)
            && app.state.current_card_id.is_none()
            && matches!(app.state.focus, Focus::Body)
        {
            app.current_theme.keyboard_focus_style
        } else if app.state.card_drag_mode {
            app.current_theme.inactive_text_style
        } else {
            app.current_theme.general_style
        };

        let board_block = Block::default()
            .title(&*board_title)
            .borders(Borders::ALL)
            .style(board_style)
            .border_style(board_border_style)
            .border_type(BorderType::Rounded);
        rect.render_widget(board_block, board_chunks[board_index]);

        let card_area_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1)].as_ref())
            .split(board_chunks[board_index]);

        let card_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(AsRef::<[Constraint]>::as_ref(&card_constraints))
            .split(card_area_chunks[0]);
        if board_cards.is_empty() {
            let available_width = card_chunks[0].width - 2;
            let empty_card_text = if preview_mode {
                "No cards found".to_string()
            } else {
                "No cards found, press ".to_string() + &new_card_key + " to add a new card"
            };
            let mut usable_length = empty_card_text.len() as u16;
            let mut usable_height = 1.0;
            if empty_card_text.len() > available_width.into() {
                usable_length = available_width;
                usable_height = empty_card_text.len() as f32 / available_width as f32;
                usable_height = usable_height.ceil();
            }
            let message_centered_rect =
                centered_rect_with_length(usable_length, usable_height as u16, card_chunks[0]);
            let empty_card_paragraph = Paragraph::new(empty_card_text)
                .alignment(Alignment::Center)
                .block(Block::default())
                .style(board_style)
                .wrap(ratatui::widgets::Wrap { trim: true });
            rect.render_widget(empty_card_paragraph, message_centered_rect);
            continue;
        }
        if !app.config.disable_scroll_bar && !board_cards.is_empty() && board_cards.len() > 1 {
            let current_card_index = board
                .cards
                .get_card_index(app.state.current_card_id.unwrap_or((0, 0)))
                .unwrap_or(0);
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
                .style(scrollbar_style)
                .end_symbol(SCROLLBAR_END_SYMBOL)
                .track_symbol(SCROLLBAR_TRACK_SYMBOL)
                .track_style(app.current_theme.inactive_text_style);
            let mut scrollbar_state = ScrollbarState::new(board.cards.len())
                .position(current_card_index)
                .viewport_content_length((card_chunks[0].height) as usize);
            let scrollbar_area = card_area_chunks[0].inner(Margin {
                vertical: 1,
                horizontal: 0,
            });
            rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        };
        for (card_index, card_id) in board_cards.iter().enumerate() {
            if app.state.hovered_card.is_some()
                && app.state.card_drag_mode
                && app.state.hovered_card.unwrap().1 == *card_id
            {
                continue;
            }
            let card = board.cards.get_card_with_id(*card_id);
            if card.is_none() {
                continue;
            }
            let card = card.unwrap();
            // Exception to not using get_button_style as we have to manage other state
            let card_style = if !is_active {
                app.current_theme.inactive_text_style
            } else if check_if_mouse_is_in_area(
                &app.state.current_mouse_coordinates,
                &card_chunks[card_index],
            ) {
                app.state.mouse_focus = Some(Focus::Body);
                app.state.set_focus(Focus::Body);
                if !current_card_set {
                    app.state.current_card_id = Some(card.id);
                    current_card_set = true;
                }
                if !app.state.card_drag_mode {
                    app.state.hovered_card = Some((*board_id, card.id));
                    app.state.hovered_card_dimensions = Some((
                        card_chunks[card_index].width,
                        card_chunks[card_index].height,
                    ));
                }
                app.current_theme.mouse_focus_style
            } else if app.state.current_card_id.unwrap_or((0, 0)) == card.id
                && matches!(app.state.focus, Focus::Body)
                && *board_id == *current_board_id
            {
                app.current_theme.keyboard_focus_style
            } else if app.state.card_drag_mode {
                app.current_theme.inactive_text_style
            } else {
                app.current_theme.general_style
            };
            render_a_single_card(
                app,
                card_chunks[card_index],
                card_style,
                card,
                rect,
                is_active,
            );
        }

        if app.state.card_drag_mode {
            // TODO: add up and down hover zones to scroll while dragging a card
        }
    }

    if !app.config.disable_scroll_bar {
        let current_board_index = boards.get_board_index(*current_board_id).unwrap_or(0) + 1;
        let percentage = {
            let temp_percent = (current_board_index as f64 / boards.len() as f64) * 100.0;
            if temp_percent.is_nan() {
                0
            } else if temp_percent > 100.0 {
                100
            } else {
                temp_percent as u16
            }
        };
        // TODO: Consider using a Scrollbar from ratatui
        let line_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(scrollbar_style)
            .percent(percentage);
        rect.render_widget(line_gauge, chunks[1]);
    }
}

pub fn render_card_being_dragged(
    parent_body_area: Rect,
    app: &mut App<'_>,
    rect: &mut Frame<'_>,
    is_active: bool,
) {
    if app.state.card_drag_mode {
        if app.state.hovered_card.is_none() {
            log::debug!("Hovered card is none");
            return;
        }
        if app.state.hovered_card_dimensions.is_none() {
            log::debug!("Hovered card dimensions are none");
            return;
        }

        let current_mouse_coordinates = app.state.current_mouse_coordinates;
        if current_mouse_coordinates == MOUSE_OUT_OF_BOUNDS_COORDINATES
            || current_mouse_coordinates.0 < parent_body_area.x
            || current_mouse_coordinates.1 < parent_body_area.y
            || current_mouse_coordinates.0 > parent_body_area.x + parent_body_area.width
            || current_mouse_coordinates.1 > parent_body_area.y + parent_body_area.height
        {
            log::debug!("Mouse is out of bounds");
            reset_card_drag_mode(app);
            return;
        }
        let card_dimensions = app.state.hovered_card_dimensions.unwrap();
        let card_width = card_dimensions.0;
        let card_height = card_dimensions.1;
        let mut card_x = current_mouse_coordinates.0.saturating_sub(card_width / 2);
        let mut card_y = current_mouse_coordinates.1.saturating_sub(card_height / 2);

        if card_x < parent_body_area.x {
            card_x = parent_body_area.x;
        }
        if card_y < parent_body_area.y {
            card_y = parent_body_area.y;
        }
        if card_x + card_width > parent_body_area.x + parent_body_area.width {
            card_x = parent_body_area.x + parent_body_area.width - card_width;
        }
        if card_y + card_height > parent_body_area.y + parent_body_area.height {
            card_y = parent_body_area.y + parent_body_area.height - card_height;
        }

        let render_area = Rect::new(card_x, card_y, card_width, card_height);

        let board_id = app.state.hovered_card.unwrap().0;
        let card_id = app.state.hovered_card.unwrap().1;

        let card = {
            let board = app.boards.get_board_with_id(board_id);
            if let Some(board) = board {
                board.cards.get_card_with_id(card_id)
            } else {
                None
            }
        }
        .cloned();

        if card.is_none() {
            log::debug!("Card is none");
            return;
        }
        let card = card.unwrap();

        render_blank_styled_canvas(rect, &app.current_theme, render_area, is_active);
        render_a_single_card(
            app,
            render_area,
            app.current_theme.error_text_style,
            &card,
            rect,
            is_active,
        )
    }
}

pub fn render_close_button(rect: &mut Frame, app: &mut App, is_active: bool) {
    let close_btn_area = Rect::new(rect.size().width - 3, 0, 3, 3);
    // Exception to not using get_button_style as we have to manage other state
    let close_btn_style = if is_active
        && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &close_btn_area)
    {
        app.state.mouse_focus = Some(Focus::CloseButton);
        app.state.set_focus(Focus::CloseButton);
        let close_button_color = app.widgets.close_button.color;
        let fg_color = app
            .current_theme
            .error_text_style
            .fg
            .unwrap_or(Color::White);
        Style::default().fg(fg_color).bg(Color::Rgb(
            close_button_color.0,
            close_button_color.1,
            close_button_color.2,
        ))
    } else if is_active {
        app.current_theme.general_style
    } else {
        app.current_theme.inactive_text_style
    };
    let close_btn = Paragraph::new(vec![Line::from("X")])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(close_btn_style),
        )
        .alignment(Alignment::Right);

    rect.render_widget(close_btn, close_btn_area);
}

pub fn render_blank_styled_canvas(
    rect: &mut Frame,
    current_theme: &Theme,
    render_area: Rect,
    is_active: bool,
) {
    let mut styled_text = vec![];
    for _ in 0..render_area.width + 1 {
        styled_text.push(" ".to_string());
    }
    let mut render_text = vec![];
    for _ in 0..render_area.height {
        render_text.push(format!("{}\n", styled_text.join("")));
    }
    let styled_text = if is_active {
        let mut style = current_theme.general_style;
        style.add_modifier = Modifier::empty();
        style.sub_modifier = Modifier::all();
        Paragraph::new(render_text.join(""))
            .style(style)
            .block(Block::default())
    } else {
        let mut style = current_theme.inactive_text_style;
        style.add_modifier = Modifier::empty();
        style.sub_modifier = Modifier::all();
        Paragraph::new(render_text.join(""))
            .style(style)
            .block(Block::default())
    };
    rect.render_widget(styled_text, render_area);
}

pub fn render_logs(
    app: &mut App,
    enable_focus_highlight: bool,
    render_area: Rect,
    rect: &mut Frame,
    is_active: bool,
) {
    let log_box_border_style = if enable_focus_highlight {
        get_mouse_focusable_field_style(app, Focus::Log, &render_area, is_active, false)
    } else {
        check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        )
    };
    let date_format = app.config.date_time_format.to_parser_string();
    let theme = &app.current_theme;
    let all_logs = get_logs();
    let mut highlight_style = check_if_active_and_get_style(
        is_active,
        theme.inactive_text_style,
        theme.list_select_style,
    );
    let mut items = vec![];
    let date_length = date_format.len() + 5;
    let wrap_length = render_area.width as usize - date_length - 6; // Border + arrow + padding
    for log_record in all_logs.buffer {
        let mut push_vec = vec![format!("[{}] - ", log_record.timestamp.format(date_format))];
        let wrapped_text = textwrap::fill(&log_record.msg, wrap_length);
        push_vec.push(wrapped_text);
        push_vec.push(log_record.level.to_string());
        items.push(push_vec);
    }
    // TODO: Optimize this by using the log state to avoid going through all the logs and only go through the ones that can fit in the render area
    let rows = items.iter().enumerate().map(|(index, item_og)| {
        let mut item = item_og.clone();
        let mut height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        if height > (render_area.height as usize - 2) {
            height = render_area.height as usize - 2;
        }
        let style = if !is_active {
            theme.inactive_text_style
        } else {
            let style = match item[2].parse::<Level>().unwrap() {
                Level::Error => theme.log_error_style,
                Level::Warn => theme.log_warn_style,
                Level::Info => theme.log_info_style,
                Level::Debug => theme.log_debug_style,
                Level::Trace => theme.log_trace_style,
            };
            if index == get_selected_index() {
                highlight_style = style.add_modifier(Modifier::REVERSED);
            };
            style
        };
        item.remove(2);
        let cells = item.iter().map(|c| Cell::from(c.to_string()).style(style));
        Row::new(cells).height(height as u16)
    });

    let log_box_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.general_style,
    );

    let log_list = Table::new(
        rows,
        [
            Constraint::Length(date_length as u16),
            Constraint::Length(wrap_length as u16),
        ],
    )
    .block(
        Block::default()
            .title("Logs")
            .style(log_box_style)
            .border_style(log_box_border_style)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    )
    .highlight_style(highlight_style)
    .highlight_symbol(LIST_SELECTED_SYMBOL);

    rect.render_stateful_widget(
        log_list,
        render_area,
        &mut RUST_KANBAN_LOGGER.hot_log.lock().state,
    );
}

fn render_a_single_card(
    app: &mut App,
    render_area: Rect,
    card_style: Style,
    card: &Card,
    frame_to_render_on: &mut Frame,
    is_active: bool,
) {
    let inner_card_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
        .margin(1)
        .split(render_area);

    let card_title = if card.name.len() > DEFAULT_CARD_TITLE_LENGTH.into() {
        format!("{}...", &card.name[0..DEFAULT_CARD_TITLE_LENGTH as usize])
    } else {
        card.name.clone()
    };
    let card_title = if app.state.current_card_id.unwrap_or((0, 0)) == card.id {
        format!("{} {}", ">>", card_title)
    } else {
        card_title
    };

    let card_description = if card.description == FIELD_NOT_SET {
        format!("Description: {}", FIELD_NOT_SET)
    } else {
        card.description.clone()
    };

    let card_due_default_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.card_due_default_style,
    );
    let card_due_warning_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.card_due_warning_style,
    );
    let card_due_overdue_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.card_due_overdue_style,
    );
    let general_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.general_style,
    );

    let mut card_extra_info = vec![Line::from("")];
    if card.due_date == FIELD_NOT_SET {
        card_extra_info.push(Line::from(Span::styled(
            format!("Due: {}", FIELD_NOT_SET),
            card_due_default_style,
        )))
    } else {
        let card_due_date = card.due_date.clone();
        let parsed_due_date =
            date_format_converter(card_due_date.trim(), app.config.date_time_format);
        let card_due_date_styled = if let Ok(parsed_due_date) = parsed_due_date {
            if parsed_due_date == FIELD_NOT_SET || parsed_due_date.is_empty() {
                Line::from(Span::styled(
                    format!("Due: {}", parsed_due_date),
                    card_due_default_style,
                ))
            } else {
                let formatted_date_format = date_format_finder(&parsed_due_date).unwrap();
                let (days_left, parsed_due_date) = match formatted_date_format {
                    DateTimeFormat::DayMonthYear
                    | DateTimeFormat::MonthDayYear
                    | DateTimeFormat::YearMonthDay => {
                        let today = Local::now().date_naive();
                        let string_to_naive_date_format = NaiveDate::parse_from_str(
                            &parsed_due_date,
                            app.config.date_time_format.to_parser_string(),
                        )
                        .unwrap();
                        let days_left = string_to_naive_date_format
                            .signed_duration_since(today)
                            .num_days();
                        let parsed_due_date = string_to_naive_date_format
                            .format(app.config.date_time_format.to_parser_string())
                            .to_string();
                        (days_left, parsed_due_date)
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
                        let days_left = string_to_naive_date_format
                            .signed_duration_since(today)
                            .num_days();
                        let parsed_due_date = string_to_naive_date_format
                            .format(app.config.date_time_format.to_parser_string())
                            .to_string();
                        (days_left, parsed_due_date)
                    }
                };
                if days_left >= 0 {
                    match days_left.cmp(&(app.config.warning_delta as i64)) {
                        Ordering::Less | Ordering::Equal => Line::from(Span::styled(
                            format!("Due: {}", parsed_due_date),
                            card_due_warning_style,
                        )),
                        Ordering::Greater => Line::from(Span::styled(
                            format!("Due: {}", parsed_due_date),
                            card_due_default_style,
                        )),
                    }
                } else {
                    Line::from(Span::styled(
                        format!("Due: {}", parsed_due_date),
                        card_due_overdue_style,
                    ))
                }
            }
        } else {
            Line::from(Span::styled(
                format!("Due: {}", card_due_date),
                card_due_default_style,
            ))
        };
        card_extra_info.extend(vec![card_due_date_styled]);
    }

    let mut card_status = format!("Status: {}", card.card_status.clone());
    let mut card_priority = format!("Priority: {}", card.priority.clone());
    let required_space = card_status.len() + 3 + card_priority.len(); // 3 is for the " | " separator

    // if required space is not available abbreviate the card status and priority
    if required_space > (render_area.width - 2) as usize {
        // accounting for border
        card_status = format!("S: {}", card.card_status.clone());
        card_priority = format!("P: {}", card.priority.clone());
    }
    let spacer_span = Span::styled(" | ", general_style);
    let card_status = if !is_active {
        Span::styled(card_status, app.current_theme.inactive_text_style)
    } else {
        match card.card_status {
            CardStatus::Active => {
                Span::styled(card_status, app.current_theme.card_status_active_style)
            }
            CardStatus::Complete => {
                Span::styled(card_status, app.current_theme.card_status_completed_style)
            }
            CardStatus::Stale => {
                Span::styled(card_status, app.current_theme.card_status_stale_style)
            }
        }
    };
    let card_priority = if !is_active {
        Span::styled(card_priority, app.current_theme.inactive_text_style)
    } else {
        match card.priority {
            CardPriority::High => {
                Span::styled(card_priority, app.current_theme.card_priority_high_style)
            }
            CardPriority::Medium => {
                Span::styled(card_priority, app.current_theme.card_priority_medium_style)
            }
            CardPriority::Low => {
                Span::styled(card_priority, app.current_theme.card_priority_low_style)
            }
        }
    };
    let status_line = Line::from(vec![card_priority, spacer_span, card_status]);
    card_extra_info.extend(vec![status_line]);

    let card_block = Block::default()
        .title(&*card_title)
        .borders(Borders::ALL)
        .border_style(card_style)
        .border_type(BorderType::Rounded);
    let card_paragraph = Paragraph::new(card_description)
        .alignment(Alignment::Left)
        .block(Block::default())
        .wrap(ratatui::widgets::Wrap { trim: false });
    let card_extra_info = Paragraph::new(card_extra_info)
        .alignment(Alignment::Left)
        .block(Block::default())
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame_to_render_on.render_widget(card_block, render_area);
    frame_to_render_on.render_widget(card_paragraph, inner_card_chunks[0]);
    frame_to_render_on.render_widget(card_extra_info, inner_card_chunks[1]);
}

pub fn draw_title<'a>(app: &mut App, render_area: Rect, is_active: bool) -> Paragraph<'a> {
    let title_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.general_style,
    );
    let border_style =
        get_mouse_focusable_field_style(app, Focus::Title, &render_area, is_active, false);
    Paragraph::new(APP_TITLE)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .style(title_style)
                .borders(Borders::ALL)
                .border_style(border_style)
                .border_type(BorderType::Rounded),
        )
}

pub fn draw_help<'a>(
    app: &mut App,
    render_area: Rect,
    is_active: bool,
) -> (Block<'a>, Table<'a>, Table<'a>) {
    let border_style =
        get_mouse_focusable_field_style(app, Focus::Help, &render_area, is_active, false);
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
    let current_element_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.list_select_style,
    );

    let rows: Vec<Row> = app
        .config
        .keybindings
        .iter()
        .map(|item| {
            let keys = item
                .1
                .iter()
                .map(|key| key.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            let cells = vec![
                Cell::from(item.0.to_string()).style(help_text_style),
                Cell::from(keys).style(help_key_style),
            ];
            Row::new(cells)
        })
        .collect();

    let mid_point = rows.len() / 2;
    let left_rows = rows[..mid_point].to_vec();
    let right_rows = rows[mid_point..].to_vec();

    let left_table = Table::new(
        left_rows,
        [Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .block(Block::default().style(help_text_style))
    .highlight_style(current_element_style)
    .highlight_symbol(">> ")
    .style(border_style);

    let right_table = Table::new(
        right_rows,
        [Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .block(Block::default().style(help_text_style))
    .highlight_style(current_element_style)
    .highlight_symbol(">> ")
    .style(border_style);

    let border_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(help_text_style)
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    (border_block, left_table, right_table)
}

pub fn draw_crab_pattern(
    render_area: Rect,
    style: Style,
    is_active: bool,
    disable_animations: bool,
) -> Paragraph<'static> {
    let crab_pattern = if !is_active || disable_animations {
        create_crab_pattern_1(render_area.width, render_area.height, is_active)
    } else {
        let patterns = [
            create_crab_pattern_1(render_area.width, render_area.height, is_active),
            create_crab_pattern_2(render_area.width, render_area.height, is_active),
            create_crab_pattern_3(render_area.width, render_area.height, is_active),
        ];
        // get_time_offset() gives offset from unix epoch use this to give different patterns every 1000ms
        let index = (get_time_offset() / PATTERN_CHANGE_INTERVAL) as usize % patterns.len();
        patterns[index].clone()
    };
    Paragraph::new(crab_pattern)
        .style(style)
        .block(Block::default())
}

fn create_crab_pattern_1(width: u16, height: u16, is_active: bool) -> String {
    let mut pattern = String::new();
    for row in 0..height {
        for col in 0..width {
            if (row + col) % 2 == 0 {
                if is_active {
                    pattern.push('🦀');
                } else {
                    pattern.push_str(HIDDEN_PASSWORD_SYMBOL.to_string().as_str());
                }
            } else {
                pattern.push_str("  ");
            }
        }
        pattern.push('\n');
    }
    pattern
}

fn create_crab_pattern_2(width: u16, height: u16, is_active: bool) -> String {
    let mut pattern = String::new();
    let block_size = 4;

    for row in 0..height {
        let block_row = row % block_size;

        for col in 0..width {
            let block_col = col % block_size;

            if (block_row == 0 && block_col <= 1)
                || (block_row == 1 && block_col >= 2)
                || (block_row == 2 && block_col <= 1)
            {
                if is_active {
                    pattern.push_str(" 🦀 ");
                } else {
                    pattern.push_str(HIDDEN_PASSWORD_SYMBOL.to_string().as_str());
                }
            } else {
                pattern.push_str("   ");
            }
        }
        pattern.push('\n');
    }
    pattern
}

fn create_crab_pattern_3(width: u16, height: u16, is_active: bool) -> String {
    let mut pattern = String::new();
    for row in 0..height {
        for col in 0..width {
            if (row % 2 == 0 && col % 2 == 0) || (row % 2 == 1 && col % 2 == 1) {
                if is_active {
                    pattern.push_str(" 🦀 ");
                } else {
                    pattern.push_str(HIDDEN_PASSWORD_SYMBOL.to_string().as_str());
                }
            } else {
                pattern.push_str("   ");
            }
        }
        pattern.push('\n');
    }
    pattern
}

fn get_time_offset() -> u64 {
    let start_time = SystemTime::now();
    let since_epoch = start_time.duration_since(UNIX_EPOCH).unwrap();
    since_epoch.as_millis() as u64
}

pub fn render_blank_styled_canvas_with_margin(
    rect: &mut Frame,
    app: &mut App,
    render_area: Rect,
    is_active: bool,
    margin: i16,
) {
    let general_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.general_style,
    );

    let x = render_area.x as i16 + margin;
    let x = if x < 0 { 0 } else { x };
    let y = render_area.y as i16 + margin;
    let y = if y < 0 { 0 } else { y };
    let width = render_area.width as i16 - margin * 2;
    let width = if width < 0 { 0 } else { width };
    let height = render_area.height as i16 - margin * 2;
    let height = if height < 0 { 0 } else { height };

    let new_render_area = Rect::new(x as u16, y as u16, width as u16, height as u16);

    let mut styled_text = vec![];
    for _ in 0..new_render_area.width + 1 {
        styled_text.push(" ".to_string());
    }
    let mut render_text = vec![];
    for _ in 0..new_render_area.height {
        render_text.push(format!("{}\n", styled_text.join("")));
    }
    let styled_text = Paragraph::new(render_text.join(""))
        .style(general_style)
        .block(Block::default());
    rect.render_widget(styled_text, new_render_area);
}
