use super::{
    text_box::{CursorMove, TextBox},
    widgets::{ToastType, ToastWidget},
    TextColorOptions, TextModifierOptions,
};
use crate::{
    app::{
        app_helper::reset_card_drag_mode,
        date_format_converter, date_format_finder,
        kanban::{Card, CardPriority, CardStatus},
        state::{AppStatus, Focus, KeyBindingEnum, UiMode},
        App, DateFormat, PopupMode,
    },
    constants::{
        APP_TITLE, DEFAULT_BOARD_TITLE_LENGTH, DEFAULT_CARD_TITLE_LENGTH, FIELD_NOT_SET,
        HIDDEN_PASSWORD_SYMBOL, LIST_SELECTED_SYMBOL, MAX_TOASTS_TO_DISPLAY, MIN_TERM_HEIGHT,
        MIN_TERM_WIDTH, MIN_TIME_BETWEEN_SENDING_RESET_LINK, MOUSE_OUT_OF_BOUNDS_COORDINATES,
        PATTERN_CHANGE_INTERVAL, SCREEN_TO_TOAST_WIDTH_RATIO, SCROLLBAR_BEGIN_SYMBOL,
        SCROLLBAR_END_SYMBOL, SCROLLBAR_TRACK_SYMBOL, SPINNER_FRAMES,
    },
    io::{
        data_handler::get_available_local_save_files,
        logger::{get_logs, get_selected_index, RUST_KANBAN_LOGGER},
    },
    util::calculate_cursor_position,
};
use chrono::{Local, NaiveDate, NaiveDateTime};
use log::{debug, Level};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, List, ListItem, ListState, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    },
    Frame,
};
use std::{
    cmp::Ordering,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn render_zen_mode(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)].as_ref())
        .split(rect.size());

    render_body(rect, chunks[0], app, false);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[0], app, rect);
}

pub fn render_title_body(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
        .split(rect.size());

    render_title(app, &chunks[0], rect);
    render_body(rect, chunks[1], app, false);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[1], app, rect);
}

pub fn render_body_help(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[1]);

    let help = draw_help(app, chunks[1]);
    let help_separator =
        Block::default()
            .borders(Borders::LEFT)
            .border_style(check_for_popup_and_get_style(
                app,
                app.current_theme.general_style,
            ));

    render_body(rect, chunks[0], app, false);
    rect.render_widget(help.0, chunks[1]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.app_table_states.help);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.app_table_states.help);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[0], app, rect);
}

pub fn render_body_log(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
        .split(rect.size());

    render_body(rect, chunks[0], app, false);
    render_logs(app, true, chunks[1], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[0], app, rect);
}

pub fn render_title_body_help(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[2]);

    let help = draw_help(app, chunks[2]);
    let help_separator =
        Block::default()
            .borders(Borders::LEFT)
            .border_style(check_for_popup_and_get_style(
                app,
                app.current_theme.general_style,
            ));

    render_title(app, &chunks[0], rect);
    render_body(rect, chunks[1], app, false);
    rect.render_widget(help.0, chunks[2]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.app_table_states.help);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.app_table_states.help);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[1], app, rect);
}

pub fn render_title_body_log(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    render_title(app, &chunks[0], rect);
    render_body(rect, chunks[1], app, false);
    render_logs(app, true, chunks[2], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[1], app, rect);
}

fn render_card_being_dragged(parent_body_area: Rect, app: &mut App<'_>, rect: &mut Frame<'_>) {
    if app.state.card_drag_mode {
        if app.state.hovered_card.is_none() {
            debug!("Hovered card is none");
            return;
        }
        if app.state.hovered_card_dimensions.is_none() {
            debug!("Hovered card dimensions are none");
            return;
        }

        let current_mouse_coordinates = app.state.current_mouse_coordinates;
        if current_mouse_coordinates == MOUSE_OUT_OF_BOUNDS_COORDINATES
            || current_mouse_coordinates.0 < parent_body_area.x
            || current_mouse_coordinates.1 < parent_body_area.y
            || current_mouse_coordinates.0 > parent_body_area.x + parent_body_area.width
            || current_mouse_coordinates.1 > parent_body_area.y + parent_body_area.height
        {
            debug!("Mouse is out of bounds");
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

        let app_boards = app.boards.clone();
        let card = {
            let board = app_boards.iter().find(|x| x.id == board_id);
            if let Some(board) = board {
                board.cards.iter().find(|x| x.id == card_id)
            } else {
                None
            }
        };

        if card.is_none() {
            debug!("Card is none");
            return;
        }
        let card = card.unwrap();

        render_blank_styled_canvas(rect, app, render_area, app.state.popup_mode.is_some());
        render_a_single_card(
            app,
            render_area,
            app.current_theme.error_text_style,
            card,
            rect,
        )
    }
}

pub fn render_body_help_log(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[1]);

    let help = draw_help(app, chunks[1]);
    let help_separator =
        Block::default()
            .borders(Borders::LEFT)
            .border_style(check_for_popup_and_get_style(
                app,
                app.current_theme.general_style,
            ));

    render_body(rect, chunks[0], app, false);
    rect.render_widget(help.0, chunks[1]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.app_table_states.help);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.app_table_states.help);
    render_logs(app, true, chunks[2], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[0], app, rect);
}

pub fn render_title_body_help_log(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[2]);

    let help = draw_help(app, chunks[2]);
    let help_separator =
        Block::default()
            .borders(Borders::LEFT)
            .border_style(check_for_popup_and_get_style(
                app,
                app.current_theme.general_style,
            ));

    render_title(app, &chunks[0], rect);
    render_body(rect, chunks[1], app, false);
    rect.render_widget(help.0, chunks[2]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.app_table_states.help);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.app_table_states.help);
    render_logs(app, true, chunks[3], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
    render_card_being_dragged(chunks[1], app, rect);
}

pub fn render_config(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let reset_btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
        .split(chunks[2]);

    let reset_both_style = get_button_style_with_default_error_style(
        app,
        Focus::SubmitButton,
        &reset_btn_chunks[0],
        popup_mode,
    );
    let reset_config_style = get_button_style_with_default_error_style(
        app,
        Focus::ExtraFocus,
        &reset_btn_chunks[1],
        popup_mode,
    );
    let scrollbar_style = check_for_popup_and_get_style(app, app.current_theme.progress_bar_style);
    let config_text_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let default_style =
        get_mouse_focusable_field_style(app, Focus::ConfigTable, &chunks[1], popup_mode, false);

    let config_table = draw_config_table_selector(app, config_text_style, default_style);
    let all_rows = app.config.to_view_list();
    let total_rows = all_rows.len();
    let current_index = app
        .state
        .app_table_states
        .config
        .selected()
        .unwrap_or(0)
        .min(total_rows - 1);
    let available_height = (chunks[1].height - 2) as usize;
    let (row_start_index, _) = get_scrollable_widget_row_bounds(
        all_rows.len(),
        current_index,
        app.state.app_table_states.config.offset(),
        available_height,
    );
    let current_mouse_y_position = app.state.current_mouse_coordinates.1;
    let hovered_index = if current_mouse_y_position > chunks[1].y
        && current_mouse_y_position < (chunks[1].y + chunks[1].height - 1)
    {
        Some(((current_mouse_y_position - chunks[1].y - 1) + row_start_index as u16) as usize)
    } else {
        None
    };
    if hovered_index.is_some()
        && (app.state.previous_mouse_coordinates != app.state.current_mouse_coordinates)
    {
        app.state.app_table_states.config.select(hovered_index);
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
        .style(scrollbar_style)
        .end_symbol(SCROLLBAR_END_SYMBOL)
        .track_symbol(SCROLLBAR_TRACK_SYMBOL)
        .track_style(app.current_theme.inactive_text_style);

    let mut scrollbar_state = ScrollbarState::new(total_rows).position(current_index);
    let scrollbar_area = chunks[1].inner(&Margin {
        horizontal: 0,
        vertical: 1,
    });

    let reset_both_button = Paragraph::new("Reset Config and KeyBindings to Default")
        .block(
            Block::default()
                .title("Reset")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(reset_both_style)
        .alignment(Alignment::Center);

    let reset_config_button = Paragraph::new("Reset Only Config to Default")
        .block(
            Block::default()
                .title("Reset")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(reset_config_style)
        .alignment(Alignment::Center);

    let config_help = draw_config_help(app, popup_mode);

    render_title(app, &chunks[0], rect);
    rect.render_stateful_widget(
        config_table,
        chunks[1],
        &mut app.state.app_table_states.config,
    );
    rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    rect.render_widget(reset_both_button, reset_btn_chunks[0]);
    rect.render_widget(reset_config_button, reset_btn_chunks[1]);
    rect.render_widget(config_help, chunks[3]);
    render_logs(app, true, chunks[4], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

fn draw_config_table_selector(
    app: &mut App,
    config_text_style: Style,
    default_style: Style,
) -> Table<'static> {
    let config_list = app.config.to_view_list();
    let rows = config_list.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16)
    });

    let highlight_style = check_for_popup_and_get_style(app, app.current_theme.list_select_style);

    Table::new(
        rows,
        [Constraint::Percentage(40), Constraint::Percentage(60)],
    )
    .block(
        Block::default()
            .title("Config Editor")
            .borders(Borders::ALL)
            .style(config_text_style)
            .border_style(default_style)
            .border_type(BorderType::Rounded),
    )
    .highlight_style(highlight_style)
    .highlight_symbol(">> ")
}

pub fn render_edit_config(rect: &mut Frame, app: &mut App) {
    let area = centered_rect_with_percentage(70, 70, rect.size());

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
        false,
        true,
    );

    let config_item_index = &app.state.config_item_being_edited;
    let list_items = app.config.to_view_list();
    let config_item_name = if config_item_index.is_some() {
        list_items[config_item_index.unwrap()].first().unwrap()
    } else {
        // NOTE: This is temporary, as only the Theme editor uses this other than config
        "Theme Name"
    };
    let config_item_value = if config_item_index.is_some() {
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
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());
    let stop_editing_key = app
        .get_first_keybinding(KeyBindingEnum::StopUserInput)
        .unwrap_or("".to_string());

    let paragraph_text = vec![
        Line::from(vec![
            Span::styled("Current Value is '", app.current_theme.help_text_style),
            Span::styled(config_item_value, app.current_theme.help_key_style),
            Span::styled("'", app.current_theme.help_text_style),
        ]),
        Line::from(String::from("")),
        Line::from(vec![
            Span::styled("Press ", app.current_theme.help_text_style),
            Span::styled(start_editing_key, app.current_theme.help_key_style),
            Span::styled(" to edit, or ", app.current_theme.help_text_style),
            Span::styled(cancel_key, app.current_theme.help_key_style),
            Span::styled(" to cancel, Press ", app.current_theme.help_text_style),
            Span::styled(stop_editing_key, app.current_theme.help_key_style),
            Span::styled(
                " to stop editing and press ",
                app.current_theme.help_text_style,
            ),
            Span::styled(accept_key, app.current_theme.help_key_style),
            Span::styled(" on Submit to save", app.current_theme.help_text_style),
        ]),
    ];
    let paragraph_title = Line::from(vec![Span::raw(config_item_name)]);
    let config_item = Paragraph::new(paragraph_text)
        .block(
            Block::default()
                .title(paragraph_title)
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    let edit_item = Paragraph::new(app.state.current_user_input.clone())
        .block(
            Block::default()
                .title("Edit")
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_style(edit_box_style)
                .border_type(BorderType::Rounded),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    if app.state.app_status == AppStatus::UserInput {
        let current_cursor_position = if app.state.current_cursor_position.is_some() {
            app.state.current_cursor_position.unwrap() as u16
        } else {
            app.state.current_user_input.len() as u16
        };
        let x_offset = current_cursor_position % (chunks[1].width - 2);
        let y_offset = current_cursor_position / (chunks[1].width - 2);
        let x_cursor_position = chunks[1].x + x_offset + 1;
        let y_cursor_position = chunks[1].y + y_offset + 1;
        rect.set_cursor(x_cursor_position, y_cursor_position);
    }

    let clear_area = centered_rect_with_percentage(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Config Editor")
        .style(app.current_theme.general_style)
        .borders(Borders::ALL)
        .border_style(app.current_theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);

    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    rect.render_widget(config_item, chunks[0]);
    rect.render_widget(edit_item, chunks[1]);
    render_logs(app, false, chunks[2], rect, false);
    if app.config.enable_mouse_support {
        let submit_button_style =
            if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[3]) {
                app.state.mouse_focus = Some(Focus::SubmitButton);
                app.state.focus = Focus::SubmitButton;
                app.current_theme.mouse_focus_style
            } else if app.state.app_status == AppStatus::KeyBindMode {
                app.current_theme.keyboard_focus_style
            } else {
                app.current_theme.general_style
            };
        let submit_button = Paragraph::new("Submit")
            .block(
                Block::default()
                    .style(app.current_theme.general_style)
                    .borders(Borders::ALL)
                    .border_style(submit_button_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        rect.render_widget(submit_button, chunks[3]);
        render_close_button(rect, app)
    }
}

pub fn render_select_default_view(rect: &mut Frame, app: &mut App) {
    let render_area = centered_rect_with_percentage(70, 70, rect.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
        .split(render_area);

    let list_items = UiMode::view_modes_as_string();
    let list_items: Vec<ListItem> = list_items
        .iter()
        .map(|s| ListItem::new(s.to_string()))
        .collect();

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &render_area) {
        app.state.mouse_focus = Some(Focus::SelectDefaultView);
        app.state.focus = Focus::SelectDefaultView;
        calculate_mouse_list_select_index(
            app.state.current_mouse_coordinates.1,
            &list_items,
            render_area,
            &mut app.state.app_list_states.default_view,
        );
    }

    let default_view_list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.current_theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    let up_key = app
        .get_first_keybinding(KeyBindingEnum::Up)
        .unwrap_or("".to_string());
    let down_key = app
        .get_first_keybinding(KeyBindingEnum::Down)
        .unwrap_or("".to_string());
    let accept_key = app
        .get_first_keybinding(KeyBindingEnum::Accept)
        .unwrap_or("".to_string());
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());

    let help_spans = Line::from(vec![
        Span::styled("Use ", app.current_theme.help_text_style),
        Span::styled(up_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(down_key, app.current_theme.help_key_style),
        Span::styled(
            " to navigate or use the mouse cursor. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled("<Mouse Left Click>", app.current_theme.help_key_style),
        Span::styled(
            " To select a Default View. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(cancel_key, app.current_theme.help_key_style),
        Span::styled(" to cancel", app.current_theme.help_text_style),
    ]);

    let default_view_picker_help = Paragraph::new(help_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(app.current_theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let clear_area = centered_rect_with_percentage(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Default View Picker")
        .style(app.current_theme.general_style)
        .borders(Borders::ALL)
        .border_style(app.current_theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);
    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    rect.render_stateful_widget(
        default_view_list,
        chunks[0],
        &mut app.state.app_list_states.default_view,
    );
    rect.render_widget(default_view_picker_help, chunks[1]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_edit_keybindings(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Fill(1),
                Constraint::Length(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let default_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let scrollbar_style = check_for_popup_and_get_style(app, app.current_theme.progress_bar_style);
    let reset_style =
        get_button_style_with_default_error_style(app, Focus::SubmitButton, &chunks[3], popup_mode);
    let current_element_style =
        check_for_popup_and_get_style(app, app.current_theme.list_select_style);
    let table_border_style = get_mouse_focusable_field_style(
        app,
        Focus::EditKeybindingsTable,
        &chunks[1],
        popup_mode,
        false,
    );
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);

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
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());

    let edit_keybinding_help_spans = Line::from(vec![
        Span::styled("Use ", help_text_style),
        Span::styled(up_key, help_key_style),
        Span::styled(" and ", help_text_style),
        Span::styled(down_key, help_key_style),
        Span::styled(" or scroll with the mouse", help_text_style),
        Span::styled(" to select a keybinding, Press ", help_text_style),
        Span::styled(accept_key.clone(), help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled("<Mouse Left Click>", help_key_style),
        Span::styled(" to edit, ", help_text_style),
        Span::styled(cancel_key, help_key_style),
        Span::styled(
            " to cancel, To Reset Keybindings to Default Press ",
            help_text_style,
        ),
        Span::styled(next_focus_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(prv_focus_key, help_key_style),
        Span::styled(" to highlight Reset Button and Press ", help_text_style),
        Span::styled(accept_key, help_key_style),
        Span::styled(" on the Reset Keybindings Button", help_text_style),
    ]);

    let mut table_items: Vec<Vec<String>> = Vec::new();
    let keybindings = app.config.keybindings.clone();
    for (key, value) in keybindings.iter() {
        let mut row: Vec<String> = Vec::new();
        row.push(keybindings.keybinding_enum_to_action(key).to_string());
        let mut row_value = String::new();
        for v in value.iter() {
            row_value.push_str(&v.to_string());
            // check if it's the last element
            if value.iter().last().unwrap() != v {
                row_value.push_str(", ");
            }
        }
        row.push(row_value);
        table_items.push(row);
    }

    let rows = table_items.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16)
    });

    let current_index = app
        .state
        .app_table_states
        .edit_keybindings
        .selected()
        .unwrap_or(0);
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
        .style(scrollbar_style)
        .end_symbol(SCROLLBAR_END_SYMBOL)
        .track_symbol(SCROLLBAR_TRACK_SYMBOL)
        .track_style(app.current_theme.inactive_text_style);
    let mut scrollbar_state = ScrollbarState::new(table_items.len()).position(current_index);
    let scrollbar_area = chunks[1].inner(&Margin {
        vertical: 1,
        horizontal: 0,
    });

    let t = Table::new(rows, [Constraint::Fill(1), Constraint::Fill(1)])
        .block(
            Block::default()
                .title("Edit Keybindings")
                .style(default_style)
                .border_style(table_border_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(current_element_style)
        .highlight_symbol(">> ");

    let edit_keybinding_help = Paragraph::new(edit_keybinding_help_spans)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(default_style)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let reset_button = Paragraph::new("Reset Keybindings to Default")
        .block(
            Block::default()
                .title("Reset")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(reset_style)
        .alignment(Alignment::Center);

    render_title(app, &chunks[0], rect);
    rect.render_stateful_widget(
        t,
        chunks[1],
        &mut app.state.app_table_states.edit_keybindings,
    );
    rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    rect.render_widget(edit_keybinding_help, chunks[2]);
    rect.render_widget(reset_button, chunks[3]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_edit_specific_keybinding(rect: &mut Frame, app: &mut App) {
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

    let edit_box_style =
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1]) {
            app.state.mouse_focus = Some(Focus::EditSpecificKeyBindingPopup);
            app.state.focus = Focus::EditSpecificKeyBindingPopup;
            app.current_theme.mouse_focus_style
        } else if app.state.app_status == AppStatus::KeyBindMode {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
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
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());
    let stop_editing_key = app
        .get_first_keybinding(KeyBindingEnum::StopUserInput)
        .unwrap_or("".to_string());

    let paragraph_text = vec![
        Line::from(vec![
            Span::styled("Current Value is '", app.current_theme.help_text_style),
            Span::styled(key_value, app.current_theme.help_key_style),
            Span::styled("'", app.current_theme.help_text_style),
        ]),
        Line::from(String::from("")),
        Line::from(vec![
            Span::styled("Press ", app.current_theme.help_text_style),
            Span::styled(user_input_key, app.current_theme.help_key_style),
            Span::styled(" to edit, ", app.current_theme.help_text_style),
            Span::styled(cancel_key, app.current_theme.help_key_style),
            Span::styled(" to cancel, ", app.current_theme.help_text_style),
            Span::styled(stop_editing_key, app.current_theme.help_key_style),
            Span::styled(" to stop editing and ", app.current_theme.help_text_style),
            Span::styled(accept_key, app.current_theme.help_key_style),
            Span::styled(
                " to save when stopped editing",
                app.current_theme.help_text_style,
            ),
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

    if app.state.app_status == AppStatus::KeyBindMode {
        let current_cursor_position = if app.state.current_cursor_position.is_some() {
            app.state.current_cursor_position.unwrap() as u16
        } else {
            current_edited_keybinding_string.len() as u16
        };
        let x_offset = current_cursor_position % (chunks[1].width - 2);
        let y_offset = current_cursor_position / (chunks[1].width - 2);
        let x_cursor_position = chunks[1].x + x_offset + 1;
        let y_cursor_position = chunks[1].y + y_offset + 1;
        rect.set_cursor(x_cursor_position, y_cursor_position);
    }

    let clear_area = centered_rect_with_percentage(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Edit Keybindings")
        .borders(Borders::ALL)
        .border_style(app.current_theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);

    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    rect.render_widget(config_item, chunks[0]);
    rect.render_widget(edit_item, chunks[1]);
    render_logs(app, false, chunks[2], rect, false);
    if app.config.enable_mouse_support {
        let submit_button_style =
            if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[3]) {
                app.state.mouse_focus = Some(Focus::SubmitButton);
                app.state.focus = Focus::SubmitButton;
                app.current_theme.mouse_focus_style
            } else if app.state.app_status == AppStatus::KeyBindMode {
                app.current_theme.keyboard_focus_style
            } else {
                app.current_theme.general_style
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
        render_close_button(rect, app);
    }
}

pub fn render_main_menu(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(10),
                Constraint::Fill(1),
                Constraint::Fill(2),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[2]);

    let main_menu_help = draw_help(app, chunks[2]);
    let help_separator =
        Block::default()
            .borders(Borders::LEFT)
            .border_style(check_for_popup_and_get_style(
                app,
                app.current_theme.general_style,
            ));

    render_title(app, &chunks[0], rect);

    if app.state.user_login_data.email_id.is_some() {
        let email_id = app.state.user_login_data.email_id.clone().unwrap();
        let email_id_len = email_id.len() as u16 + 4;
        let sub_main_menu_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(chunks[1].width - email_id_len),
                    Constraint::Length(email_id_len),
                ]
                .as_ref(),
            )
            .split(chunks[1]);

        let border_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                app.current_theme
                    .general_style
                    .add_modifier(Modifier::RAPID_BLINK),
            )
            .border_type(BorderType::Rounded);

        let email_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length((sub_main_menu_chunks[1].height - 4) / 2),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length((sub_main_menu_chunks[1].height - 4) / 2),
                ]
                .as_ref(),
            )
            .split(sub_main_menu_chunks[1]);

        let heading_text = Paragraph::new("Logged in as:")
            .block(
                Block::default().style(
                    app.current_theme
                        .general_style
                        .add_modifier(Modifier::RAPID_BLINK),
                ),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let email_id_text = Paragraph::new(email_id)
            .block(
                Block::default().style(
                    app.current_theme
                        .general_style
                        .add_modifier(Modifier::RAPID_BLINK),
                ),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        draw_main_menu(app, sub_main_menu_chunks[0], rect);
        rect.render_widget(border_block, sub_main_menu_chunks[1]);
        rect.render_widget(heading_text, email_chunks[1]);
        rect.render_widget(email_id_text, email_chunks[3]);
    } else {
        draw_main_menu(app, chunks[1], rect);
    }

    rect.render_widget(main_menu_help.0, chunks[2]);
    rect.render_stateful_widget(
        main_menu_help.1,
        help_chunks[0],
        &mut app.state.app_table_states.help,
    );
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(
        main_menu_help.2,
        help_chunks[2],
        &mut app.state.app_table_states.help,
    );
    render_logs(app, true, chunks[3], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_help_menu(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(4)].as_ref())
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[0]);

    let help_menu = draw_help(app, chunks[0]);
    let help_separator =
        Block::default()
            .borders(Borders::LEFT)
            .border_style(check_for_popup_and_get_style(
                app,
                app.current_theme.general_style,
            ));

    rect.render_widget(help_menu.0, chunks[0]);
    rect.render_stateful_widget(
        help_menu.1,
        help_chunks[0],
        &mut app.state.app_table_states.help,
    );
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(
        help_menu.2,
        help_chunks[2],
        &mut app.state.app_table_states.help,
    );
    render_logs(app, true, chunks[1], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_logs_only(rect: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1)].as_ref())
        .split(rect.size());

    render_logs(app, true, chunks[0], rect, app.state.popup_mode.is_some());
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

fn draw_help<'a>(app: &mut App, render_area: Rect) -> (Block<'a>, Table<'a>, Table<'a>) {
    let border_style = get_mouse_focusable_field_style(
        app,
        Focus::Help,
        &render_area,
        app.state.popup_mode.is_some(),
        false,
    );
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);
    let current_element_style =
        check_for_popup_and_get_style(app, app.current_theme.list_select_style);

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

fn draw_config_help<'a>(app: &mut App, popup_mode: bool) -> Paragraph<'a> {
    let focus = app.state.focus;
    let help_box_style = if popup_mode {
        app.current_theme.inactive_text_style
    } else if matches!(focus, Focus::ConfigHelp) {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    };
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);

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
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());

    let help_spans = Line::from(vec![
        Span::styled("Use ", help_text_style),
        Span::styled(up_key, help_key_style),
        Span::styled(" and ", help_text_style),
        Span::styled(down_key, help_key_style),
        Span::styled(" or scroll with the mouse", help_text_style),
        Span::styled(" to navigate. To edit a value press ", help_text_style),
        Span::styled(accept_key.clone(), help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled("<Mouse Left Click>", help_key_style),
        Span::styled(". Press ", help_text_style),
        Span::styled(cancel_key, help_key_style),
        Span::styled(
            " to cancel. To Reset Keybindings or config to Default, press ",
            help_text_style,
        ),
        Span::styled(next_focus_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(prv_focus_key, help_key_style),
        Span::styled(
            " to highlight respective Reset Button then press ",
            help_text_style,
        ),
        Span::styled(accept_key, help_key_style),
        Span::styled(" to reset", help_text_style),
    ]);

    Paragraph::new(help_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(help_box_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
}

fn draw_main_menu(app: &mut App, render_area: Rect, rect: &mut Frame) {
    let main_menu_items = app.main_menu.all();
    let menu_style = get_mouse_focusable_field_style_with_vertical_list_selection(
        app,
        &main_menu_items,
        render_area,
        app.state.popup_mode.is_some(),
    );
    let default_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let highlight_style = check_for_popup_and_get_style(app, app.current_theme.list_select_style);
    let list_items = main_menu_items
        .iter()
        .map(|i| ListItem::new(i.to_string()))
        .collect::<Vec<ListItem>>();
    let main_menu = List::new(list_items)
        .block(
            Block::default()
                .title("Main menu")
                .style(default_style)
                .borders(Borders::ALL)
                .border_style(menu_style)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(highlight_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    rect.render_stateful_widget(
        main_menu,
        render_area,
        &mut app.state.app_list_states.main_menu,
    );
}

pub fn render_body(rect: &mut Frame, area: Rect, app: &mut App, preview_mode: bool) {
    let fallback_boards = vec![];
    let mut current_board_set = false;
    let mut current_card_set = false;
    let app_preview_boards_and_cards = app
        .state
        .preview_boards_and_cards
        .clone()
        .unwrap_or([].to_vec());
    let app_boards_and_cards = app.boards.clone();
    let app_filtered_boards_and_cards = app.filtered_boards.clone();
    let boards = if preview_mode {
        if app_preview_boards_and_cards.is_empty() {
            &fallback_boards
        } else {
            &app_preview_boards_and_cards
        }
    } else if !app.filtered_boards.is_empty() {
        &app_filtered_boards_and_cards
    } else {
        &app_boards_and_cards
    };
    let scrollbar_style = if app.state.card_drag_mode {
        app.current_theme.inactive_text_style
    } else {
        check_for_popup_and_get_style(app, app.current_theme.progress_bar_style)
    };
    let error_text_style = if app.state.card_drag_mode {
        app.current_theme.inactive_text_style
    } else {
        check_for_popup_and_get_style(app, app.current_theme.error_text_style)
    };
    let current_board_id = &app.state.current_board_id.unwrap_or((0, 0));

    let new_board_key = app
        .get_first_keybinding(KeyBindingEnum::NewBoard)
        .unwrap_or("".to_string());
    let new_card_key = app
        .get_first_keybinding(KeyBindingEnum::NewCard)
        .unwrap_or("".to_string());

    if preview_mode {
        if app.state.preview_boards_and_cards.is_none()
            || app
                .state
                .preview_boards_and_cards
                .as_ref()
                .unwrap()
                .is_empty()
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
        let board = boards.iter().find(|&b| b.id == *board_id);
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

        let board_style = if app.state.popup_mode.is_some() || app.state.card_drag_mode {
            app.current_theme.inactive_text_style
        } else {
            app.current_theme.general_style
        };
        let board_border_style = if app.state.popup_mode.is_some() {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &board_chunks[board_index],
        ) {
            app.state.mouse_focus = Some(Focus::Body);
            app.state.focus = Focus::Body;
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
                .iter()
                .position(|c| c.id == app.state.current_card_id.unwrap_or((0, 0)))
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
            let scrollbar_area = card_area_chunks[0].inner(&Margin {
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
            let card = board.get_card(*card_id);
            if card.is_none() {
                continue;
            }
            let card = card.unwrap();
            let card_style = if app.state.popup_mode.is_some() {
                app.current_theme.inactive_text_style
            } else if check_if_mouse_is_in_area(
                &app.state.current_mouse_coordinates,
                &card_chunks[card_index],
            ) {
                app.state.mouse_focus = Some(Focus::Body);
                app.state.focus = Focus::Body;
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
            render_a_single_card(app, card_chunks[card_index], card_style, card, rect);
        }

        if app.state.card_drag_mode {
            // TODO: add up and down hover zones to scroll while dragging a card
        }
    }

    if !app.config.disable_scroll_bar {
        let current_board_index = boards
            .iter()
            .position(|board| board.id == *current_board_id)
            .unwrap_or(0)
            + 1;
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
        let line_gauge = Gauge::default()
            .block(Block::default())
            .gauge_style(scrollbar_style)
            .percent(percentage);
        rect.render_widget(line_gauge, chunks[1]);
    }
}

fn render_a_single_card(
    app: &mut App,
    render_area: Rect,
    card_style: Style,
    card: &Card,
    frame_to_render_on: &mut Frame,
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

    let mut card_extra_info = vec![Line::from("")];
    if card.due_date == FIELD_NOT_SET {
        if app.state.popup_mode.is_some() {
            card_extra_info.push(Line::from(Span::styled(
                format!("Due: {}", FIELD_NOT_SET),
                app.current_theme.inactive_text_style,
            )))
        } else {
            card_extra_info.push(Line::from(Span::styled(
                format!("Due: {}", FIELD_NOT_SET),
                app.current_theme.card_due_default_style,
            )))
        }
    } else {
        let card_due_date = card.due_date.clone();
        let parsed_due_date = date_format_converter(card_due_date.trim(), app.config.date_format);
        let card_due_date_styled = if let Ok(parsed_due_date) = parsed_due_date {
            if app.state.popup_mode.is_some() {
                Line::from(Span::styled(
                    format!("Due: {}", parsed_due_date),
                    app.current_theme.inactive_text_style,
                ))
            } else if parsed_due_date == FIELD_NOT_SET || parsed_due_date.is_empty() {
                Line::from(Span::styled(
                    format!("Due: {}", parsed_due_date),
                    app.current_theme.card_due_default_style,
                ))
            } else {
                let formatted_date_format = date_format_finder(&parsed_due_date).unwrap();
                let (days_left, parsed_due_date) = match formatted_date_format {
                    DateFormat::DayMonthYear
                    | DateFormat::MonthDayYear
                    | DateFormat::YearMonthDay => {
                        let today = Local::now().date_naive();
                        let string_to_naive_date_format = NaiveDate::parse_from_str(
                            &parsed_due_date,
                            app.config.date_format.to_parser_string(),
                        )
                        .unwrap();
                        let days_left = string_to_naive_date_format
                            .signed_duration_since(today)
                            .num_days();
                        let parsed_due_date = string_to_naive_date_format
                            .format(app.config.date_format.to_parser_string())
                            .to_string();
                        (days_left, parsed_due_date)
                    }
                    DateFormat::DayMonthYearTime
                    | DateFormat::MonthDayYearTime
                    | DateFormat::YearMonthDayTime {} => {
                        let today = Local::now().naive_local();
                        let string_to_naive_date_format = NaiveDateTime::parse_from_str(
                            &parsed_due_date,
                            app.config.date_format.to_parser_string(),
                        )
                        .unwrap();
                        let days_left = string_to_naive_date_format
                            .signed_duration_since(today)
                            .num_days();
                        let parsed_due_date = string_to_naive_date_format
                            .format(app.config.date_format.to_parser_string())
                            .to_string();
                        (days_left, parsed_due_date)
                    }
                };
                if days_left >= 0 {
                    match days_left.cmp(&(app.config.warning_delta as i64)) {
                        Ordering::Less | Ordering::Equal => Line::from(Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.current_theme.card_due_warning_style,
                        )),
                        Ordering::Greater => Line::from(Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.current_theme.card_due_default_style,
                        )),
                    }
                } else {
                    Line::from(Span::styled(
                        format!("Due: {}", parsed_due_date),
                        app.current_theme.card_due_overdue_style,
                    ))
                }
            }
        } else if app.state.popup_mode.is_some() {
            Line::from(Span::styled(
                format!("Due: {}", card_due_date),
                app.current_theme.inactive_text_style,
            ))
        } else {
            Line::from(Span::styled(
                format!("Due: {}", card_due_date),
                app.current_theme.card_due_default_style,
            ))
        };
        card_extra_info.extend(vec![card_due_date_styled]);
    }

    let card_status = format!("Status: {}", card.card_status.clone());
    let card_status = if app.state.popup_mode.is_some() {
        Line::from(Span::styled(
            card_status,
            app.current_theme.inactive_text_style,
        ))
    } else {
        match card.card_status {
            CardStatus::Active => Line::from(Span::styled(
                card_status,
                app.current_theme.card_status_active_style,
            )),
            CardStatus::Complete => Line::from(Span::styled(
                card_status,
                app.current_theme.card_status_completed_style,
            )),
            CardStatus::Stale => Line::from(Span::styled(
                card_status,
                app.current_theme.card_status_stale_style,
            )),
        }
    };
    card_extra_info.extend(vec![card_status]);

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

fn centered_rect_with_percentage(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn centered_rect_with_length(length_x: u16, length_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - length_y) / 2),
                Constraint::Length(length_y),
                Constraint::Length((r.height - length_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - length_x) / 2),
                Constraint::Length(length_x),
                Constraint::Length((r.width - length_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn top_left_rect(length_x: u16, length_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(length_y),
                Constraint::Length((r.width - length_y) / 2),
                Constraint::Length((r.width - length_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(length_x),
                Constraint::Length((r.width - length_x) / 2),
                Constraint::Length((r.width - length_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[0])[0]
}

pub fn draw_size_error(rect: &mut Frame, size: &Rect, msg: String, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
        .split(*size);

    let error_text_spans = vec![
        Line::from(Span::styled(msg, app.current_theme.error_text_style)),
        Line::from(Span::styled(
            "Resize the window to continue, or press 'q' to quit.",
            app.current_theme.general_style,
        )),
    ];

    let body = Paragraph::new(error_text_spans)
        .block(Block::default().borders(Borders::ALL).borders(Borders::ALL))
        .alignment(Alignment::Center);

    rect.render_widget(draw_title(app, *size), chunks[0]);
    rect.render_widget(body, chunks[1]);
}

pub fn draw_loading_screen(rect: &mut Frame, size: &Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
        .split(*size);

    let mut text = vec![Line::from(vec![
        Span::styled("Loading", app.current_theme.keyboard_focus_style),
        Span::styled(
            "......`(*>﹏<*)′......",
            app.current_theme.keyboard_focus_style,
        ),
        Span::styled("Please wait", app.current_theme.keyboard_focus_style),
    ])];
    if app.config.auto_login {
        text.push(Line::from(Span::styled(
            "",
            app.current_theme.keyboard_focus_style,
        )));
        text.push(Line::from(Span::styled(
            "Auto login enabled, please wait",
            app.current_theme.keyboard_focus_style,
        )));
    }
    let body = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    rect.render_widget(draw_title(app, *size), chunks[0]);
    rect.render_widget(body, chunks[1]);
}

pub fn draw_title<'a>(app: &mut App, render_area: Rect) -> Paragraph<'a> {
    let title_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let border_style = get_mouse_focusable_field_style(
        app,
        Focus::Title,
        &render_area,
        app.state.popup_mode.is_some(),
        false,
    );
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

pub fn check_size(rect: &Rect) -> String {
    let mut msg = String::new();
    if rect.width < MIN_TERM_WIDTH {
        msg.push_str(&format!(
            "For optimal viewing experience, Terminal width should be >= {}, (current width {})",
            MIN_TERM_WIDTH, rect.width
        ));
    } else if rect.height < MIN_TERM_HEIGHT {
        msg.push_str(&format!(
            "For optimal viewing experience, Terminal height should be >= {}, (current height {})",
            MIN_TERM_HEIGHT, rect.height
        ));
    } else {
        msg.push_str("Size OK");
    }
    msg
}

pub fn render_new_board_form(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Fill(1),
                Constraint::Length(4),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let default_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);
    let name_style =
        get_mouse_focusable_field_style(app, Focus::NewBoardName, &chunks[1], popup_mode, false);
    let description_style = get_mouse_focusable_field_style(
        app,
        Focus::NewBoardDescription,
        &chunks[2],
        popup_mode,
        false,
    );
    let submit_style =
        get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[4], popup_mode, false);

    let title_paragraph = Paragraph::new("Create a new Board")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(default_style),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let wrapped_title_text = textwrap::wrap(
        &app.state.app_form_states.new_board[0],
        (chunks[1].width - 2) as usize,
    );
    let board_name_field = wrapped_title_text
        .iter()
        .map(|x| Line::from(Span::raw(&**x)))
        .collect::<Vec<Line>>();
    let wrapped_description_text = textwrap::wrap(
        &app.state.app_form_states.new_board[1],
        (chunks[2].width - 2) as usize,
    );
    let board_description_field = wrapped_description_text
        .iter()
        .map(|x| Line::from(Span::raw(&**x)))
        .collect::<Vec<Line>>();

    let board_name = Paragraph::new(board_name_field)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(name_style)
                .border_type(BorderType::Rounded)
                .title("Board Name (required)"),
        );
    rect.render_widget(board_name, chunks[1]);

    let board_description = Paragraph::new(board_description_field)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(description_style)
                .border_type(BorderType::Rounded)
                .title("Board Description"),
        );
    rect.render_widget(board_description, chunks[2]);

    let input_mode_key = app
        .get_first_keybinding(KeyBindingEnum::TakeUserInput)
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
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());
    let stop_user_input_key = app
        .get_first_keybinding(KeyBindingEnum::StopUserInput)
        .unwrap_or("".to_string());

    let help_text = Line::from(vec![
        Span::styled("Press ", help_text_style),
        Span::styled(input_mode_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(accept_key.clone(), help_key_style),
        Span::styled("to start typing. Press ", help_text_style),
        Span::styled(stop_user_input_key, help_key_style),
        Span::styled(" to stop typing. Press ", help_text_style),
        Span::styled(next_focus_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(prv_focus_key, help_key_style),
        Span::styled(" to switch focus. Press ", help_text_style),
        Span::styled(accept_key, help_key_style),
        Span::styled(" to submit. Press ", help_text_style),
        Span::styled(cancel_key, help_key_style),
        Span::styled(" to cancel", help_text_style),
    ]);
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(default_style),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    rect.render_widget(help_paragraph, chunks[3]);

    let submit_button = Paragraph::new("Submit").alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .style(submit_style)
            .border_type(BorderType::Rounded),
    );
    rect.render_widget(submit_button, chunks[4]);

    if app.state.focus == Focus::NewBoardName && app.state.app_status == AppStatus::UserInput {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_title_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.app_form_states.new_board[0].len()),
                chunks[1],
            );
            rect.set_cursor(x_pos, y_pos);
        } else {
            rect.set_cursor(chunks[1].x + 1, chunks[1].y + 1);
        }
    } else if app.state.focus == Focus::NewBoardDescription
        && app.state.app_status == AppStatus::UserInput
    {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_description_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.app_form_states.new_board[1].len()),
                chunks[2],
            );
            rect.set_cursor(x_pos, y_pos);
        } else {
            rect.set_cursor(chunks[2].x + 1, chunks[2].y + 1);
        }
    }

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_new_card_form(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let default_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let name_style =
        get_mouse_focusable_field_style(app, Focus::CardName, &chunks[1], popup_mode, false);
    let description_style =
        get_mouse_focusable_field_style(app, Focus::CardDescription, &chunks[2], popup_mode, false);
    let due_date_style =
        get_mouse_focusable_field_style(app, Focus::CardDueDate, &chunks[3], popup_mode, false);
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);
    let submit_style =
        get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[5], popup_mode, false);

    let title_paragraph = Paragraph::new("Create a new Card")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(default_style),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let wrapped_card_name_text = textwrap::wrap(
        &app.state.app_form_states.new_card[0],
        (chunks[1].width - 2) as usize,
    );
    let card_name_field = wrapped_card_name_text
        .iter()
        .map(|x| Line::from(Span::raw(&**x)))
        .collect::<Vec<Line>>();
    let wrapped_card_due_date_text = textwrap::wrap(
        &app.state.app_form_states.new_card[2],
        (chunks[3].width - 2) as usize,
    );
    let card_due_date_field = wrapped_card_due_date_text
        .iter()
        .map(|x| Line::from(Span::raw(&**x)))
        .collect::<Vec<Line>>();
    let card_name = Paragraph::new(card_name_field)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(name_style)
                .border_type(BorderType::Rounded)
                .title("Card Name (required)"),
        );
    rect.render_widget(card_name, chunks[1]);
    let description_block = Block::default()
        .borders(Borders::ALL)
        .style(description_style)
        .border_type(BorderType::Rounded)
        .title("Card Description");
    let card_description = if app.state.card_description_text_buffer.is_some() {
        let text_area = &mut app.state.card_description_text_buffer.as_mut().unwrap();
        if app.state.focus == Focus::CardDescription {
            if app.state.app_status != AppStatus::UserInput {
                text_area.disable_cursor();
            } else {
                text_area.enable_cursor(
                    app.current_theme
                        .keyboard_focus_style
                        .add_modifier(Modifier::REVERSED),
                );
            }
        }
        text_area.set_block(description_block.clone());
        text_area.clone()
    } else {
        let mut textarea = TextBox::default();
        textarea.set_block(description_block.clone());
        textarea.insert_str(&app.state.app_form_states.new_card[1]);
        textarea.move_cursor(CursorMove::Jump(0, 0));
        if app.config.show_line_numbers {
            textarea.set_line_number_style(app.current_theme.general_style)
        } else {
            textarea.remove_line_number()
        }
        if app.state.app_status != AppStatus::UserInput {
            textarea.set_cursor_style(Style::default());
        } else {
            textarea.set_cursor_style(
                app.current_theme
                    .keyboard_focus_style
                    .add_modifier(Modifier::REVERSED),
            );
        }
        textarea
    };
    rect.render_widget(card_description.widget(), chunks[2]);

    let parsed_due_date = date_format_converter(
        app.state.app_form_states.new_card[2].trim(),
        app.config.date_format,
    );
    let card_due_date = Paragraph::new(card_due_date_field)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(due_date_style)
                .border_type(BorderType::Rounded)
                .title("Card Due Date (DD/MM/YYYY-HH:MM:SS), (DD/MM/YYYY), (YYYY/MM/DD-HH:MM:SS), or (YYYY/MM/DD)"),
        );
    if parsed_due_date.is_err() && !app.state.app_form_states.new_card[2].is_empty() {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Length(21)].as_ref())
            .split(chunks[3]);
        rect.render_widget(card_due_date, new_chunks[0]);
        let error_text = Line::from(vec![Span::raw("Invalid date format")]);
        let error_paragraph = Paragraph::new(error_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.current_theme.error_text_style),
            );
        rect.render_widget(error_paragraph, new_chunks[1]);
    } else {
        rect.render_widget(card_due_date, chunks[3]);
    }

    let input_mode_key = app
        .get_first_keybinding(KeyBindingEnum::TakeUserInput)
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
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());
    let stop_user_input_key = app
        .get_first_keybinding(KeyBindingEnum::StopUserInput)
        .unwrap_or("".to_string());

    let help_spans = Line::from(vec![
        Span::styled("Press ", help_text_style),
        Span::styled(input_mode_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(accept_key.clone(), help_key_style),
        Span::styled("to start typing. Press ", help_text_style),
        Span::styled(stop_user_input_key, help_key_style),
        Span::styled(" to stop typing. Press ", help_text_style),
        Span::styled(next_focus_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(prv_focus_key, help_key_style),
        Span::styled(" to switch focus. Press ", help_text_style),
        Span::styled(accept_key, help_key_style),
        Span::styled(" to submit. Press ", help_text_style),
        Span::styled(cancel_key, help_key_style),
        Span::styled(" to cancel", help_text_style),
    ]);

    let help_paragraph = Paragraph::new(help_spans)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(default_style),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    rect.render_widget(help_paragraph, chunks[4]);

    let submit_button = Paragraph::new("Submit").alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .style(submit_style)
            .border_type(BorderType::Rounded),
    );
    rect.render_widget(submit_button, chunks[5]);

    if app.state.focus == Focus::CardName && app.state.app_status == AppStatus::UserInput {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_card_name_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.app_form_states.new_card[0].len()),
                chunks[1],
            );
            rect.set_cursor(x_pos, y_pos);
        } else {
            rect.set_cursor(chunks[1].x + 1, chunks[1].y + 1);
        }
    } else if app.state.focus == Focus::CardDueDate && app.state.app_status == AppStatus::UserInput
    {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_card_due_date_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.app_form_states.new_card[2].len()),
                chunks[3],
            );
            rect.set_cursor(x_pos, y_pos);
        } else {
            rect.set_cursor(chunks[3].x + 1, chunks[3].y + 1);
        }
    }

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_load_a_save(rect: &mut Frame, app: &mut App) {
    let default_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);
    let main_chunks = {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(35), Constraint::Fill(1)].as_ref())
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

    let title_paragraph = Paragraph::new("Load a Save (Local)")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(default_style);
    rect.render_widget(title_paragraph, chunks[0]);

    let item_list = get_available_local_save_files(&app.config);
    let item_list = if let Some(item_list) = item_list {
        item_list
    } else {
        Vec::new()
    };
    if item_list.is_empty() {
        let no_saves_paragraph = Paragraph::new("No saves found")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(app.current_theme.error_text_style);
        rect.render_widget(no_saves_paragraph, chunks[1]);
    } else {
        let items: Vec<ListItem> = item_list
            .iter()
            .map(|i| ListItem::new(i.to_string()))
            .collect();
        let choice_list = List::new(items)
            .block(
                Block::default()
                    .title("Available Saves")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(app.current_theme.list_select_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL)
            .style(default_style);

        if !(app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette)
            && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1])
        {
            app.state.mouse_focus = Some(Focus::LoadSave);
            app.state.focus = Focus::LoadSave;
            calculate_mouse_list_select_index(
                app.state.current_mouse_coordinates.1,
                &item_list,
                chunks[1],
                &mut app.state.app_list_states.load_save,
            );
        }
        rect.render_stateful_widget(
            choice_list,
            chunks[1],
            &mut app.state.app_list_states.load_save,
        );
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
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());

    let help_text = Line::from(vec![
        Span::styled("Use ", help_text_style),
        Span::styled(&up_key, help_key_style),
        Span::styled(" or ", help_text_style),
        Span::styled(&down_key, help_key_style),
        Span::styled(" to navigate. Press ", help_text_style),
        Span::styled(accept_key, help_key_style),
        Span::styled(" to Load the selected save file. Press ", help_text_style),
        Span::styled(cancel_key, help_key_style),
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
        // format!("Select a save file with {}or {}to preview or Click on a save file to preview if using a mouse", up_key, down_key)
        let help_text = Line::from(vec![
            Span::styled("Select a save file with ", help_text_style),
            Span::styled(&up_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(&down_key, help_key_style),
            Span::styled(
                " to preview. Click on a save file to preview if using a mouse",
                help_text_style,
            ),
        ]);
        let preview_paragraph = Paragraph::new(help_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(default_style)
            .wrap(ratatui::widgets::Wrap { trim: true });
        rect.render_widget(preview_paragraph, preview_chunks[1]);
    } else if app.state.preview_boards_and_cards.is_none() {
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
        render_body(rect, preview_chunks[1], app, true)
    }

    let preview_title_paragraph = if app.state.preview_file_name.is_some() {
        Paragraph::new("Previewing: ".to_string() + &app.state.preview_file_name.clone().unwrap())
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
        render_close_button(rect, app);
    } else {
        rect.render_widget(preview_title_paragraph, preview_chunks[0]);
    }
}

pub fn render_toast(rect: &mut Frame, app: &mut App) {
    let all_toasts = app.widgets.toasts.clone();
    let mut loading_toasts = all_toasts
        .iter()
        .filter(|x| x.toast_type == ToastType::Loading)
        .collect::<Vec<&ToastWidget>>();
    let app_toasts = app.widgets.toasts.clone();
    let toasts = if !loading_toasts.is_empty() {
        let sorted_loading_toasts = if loading_toasts.len() > MAX_TOASTS_TO_DISPLAY - 1 {
            loading_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            loading_toasts
                .iter()
                .copied()
                .take(MAX_TOASTS_TO_DISPLAY - 1)
                .rev()
                .collect::<Vec<&ToastWidget>>()
        } else {
            loading_toasts
        };
        let mut toasts = sorted_loading_toasts;
        let mut regular_toasts = all_toasts
            .iter()
            .filter(|x| x.toast_type != ToastType::Loading)
            .collect::<Vec<&ToastWidget>>();
        regular_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        while toasts.len() < MAX_TOASTS_TO_DISPLAY {
            if let Some(toast) = regular_toasts.pop() {
                toasts.push(toast);
            } else {
                break;
            }
        }
        if toasts.len() < MAX_TOASTS_TO_DISPLAY {
            let mut loading_toasts = all_toasts
                .iter()
                .filter(|x| x.toast_type == ToastType::Loading)
                .collect::<Vec<&ToastWidget>>();
            loading_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            while toasts.len() < MAX_TOASTS_TO_DISPLAY {
                if let Some(toast) = loading_toasts.pop() {
                    if !toasts.contains(&toast) {
                        toasts.push(toast);
                    }
                } else {
                    break;
                }
            }
        }
        toasts
    } else {
        app_toasts
            .iter()
            .rev()
            .take(MAX_TOASTS_TO_DISPLAY)
            .rev()
            .collect::<Vec<&ToastWidget>>()
    };

    if toasts.is_empty() {
        return;
    }
    let mut total_height_rendered = 1;
    for toast in toasts.iter() {
        let toast_style = app
            .current_theme
            .general_style
            .fg(ratatui::style::Color::Rgb(
                toast.toast_color.0,
                toast.toast_color.1,
                toast.toast_color.2,
            ));
        let mut toast_title = toast.title.to_owned();
        toast_title = match toast.toast_type {
            ToastType::Loading => {
                let spinner_frames = &SPINNER_FRAMES;
                let frame =
                    (toast.start_time.elapsed().as_millis() / 100) % spinner_frames.len() as u128;
                let frame = frame as usize;
                format!("{} {}", spinner_frames[frame], toast_title)
            }
            _ => toast_title,
        };
        let x_offset = rect.size().width - (rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO);
        let lines = textwrap::wrap(
            &toast.message,
            ((rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO) - 2) as usize,
        )
        .iter()
        .map(|x| Line::from(x.to_string()))
        .collect::<Vec<Line>>();
        let toast_height = lines.len() as u16 + 2;
        let toast_block = Block::default()
            .title(toast_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(toast_style);
        let toast_paragraph = Paragraph::new(lines)
            .block(toast_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(toast_style);
        if toast_height + total_height_rendered > rect.size().height {
            debug!("Toast height is greater than the height of the screen");
            break;
        }
        rect.render_widget(
            Clear,
            Rect::new(
                x_offset,
                total_height_rendered,
                rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO,
                toast_height,
            ),
        );
        rect.render_widget(
            toast_paragraph,
            Rect::new(
                x_offset,
                total_height_rendered,
                rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO,
                toast_height,
            ),
        );
        total_height_rendered += toast_height;
        if total_height_rendered >= rect.size().height {
            debug!("Toast height is greater than the height of the screen");
            break;
        }
    }

    let text_offset = 15;
    let toast_count = app.widgets.toasts.len();
    let toast_count_text = format!(" {} Message(s)", toast_count);
    let toast_count_paragraph = Paragraph::new(toast_count_text)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_type(BorderType::Rounded),
        )
        .style(app.current_theme.general_style);
    let message_area = Rect::new(rect.size().width - text_offset, 0, text_offset, 1);

    render_blank_styled_canvas(rect, app, message_area, false);
    rect.render_widget(toast_count_paragraph, message_area);
}

pub fn render_view_card(rect: &mut Frame, app: &mut App) {
    let popup_area = centered_rect_with_percentage(90, 90, rect.size());
    render_blank_styled_canvas(rect, app, popup_area, false);
    if app.state.current_board_id.is_none() || app.state.current_card_id.is_none() {
        let no_board_or_card_selected = Paragraph::new("No board or card selected.")
            .block(
                Block::default()
                    .title("Card Info")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.current_theme.error_text_style),
            )
            .alignment(Alignment::Center);
        rect.render_widget(no_board_or_card_selected, popup_area);
        return;
    }

    let board = app
        .boards
        .iter()
        .find(|b| b.id == app.state.current_board_id.unwrap());
    if board.is_none() {
        let could_not_find_board = Paragraph::new("Could not find board to view card.")
            .block(
                Block::default()
                    .title("Card Info")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.current_theme.error_text_style),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        rect.render_widget(could_not_find_board, popup_area);
        return;
    }

    let board = board.unwrap();
    let card = board
        .cards
        .iter()
        .find(|c| c.id == app.state.current_card_id.unwrap());
    if card.is_none() {
        let could_not_find_card = Paragraph::new("Could not find card to view.")
            .block(
                Block::default()
                    .title("Card Info")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.current_theme.error_text_style),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        rect.render_widget(could_not_find_card, popup_area);
        return;
    }

    let card = if app.state.card_being_edited.is_some() {
        &app.state.card_being_edited.as_ref().unwrap().1
    } else {
        card.unwrap()
    };

    let board_name = board.name.clone();
    let card_name = card.name.clone();

    // Prepare Main Block Widget
    let main_block_widget = {
        Block::default()
            .title(format!("{} >> Board({})", card_name, board_name))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(app.current_theme.general_style)
    };

    // Prepare Name Block Widget
    let name_paragraph_widget = {
        let name_style = if app.state.focus == Focus::CardName {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };

        Paragraph::new(card_name).block(
            Block::default()
                .title("Name")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(name_style),
        )
    };

    // Process Card Description
    let card_description = {
        let description_length = if app.state.card_description_text_buffer.is_some() {
            app.state
                .card_description_text_buffer
                .as_ref()
                .unwrap()
                .lines()
                .len()
        } else {
            let text_buffer =
                TextBox::from(card.description.clone().split('\n').collect::<Vec<&str>>());
            app.state.card_description_text_buffer = Some(text_buffer.clone());
            text_buffer.lines().len()
        };
        let description_style = if app.state.focus == Focus::CardDescription {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };

        let description_block = Block::default()
            .title(format!("Description ({} lines)", description_length))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(description_style);

        let text_area = &mut app.state.card_description_text_buffer.as_mut().unwrap();
        if app.config.show_line_numbers {
            text_area.set_line_number_style(app.current_theme.general_style)
        } else {
            text_area.remove_line_number()
        }
        if app.state.focus == Focus::CardDescription {
            if app.state.app_status != AppStatus::UserInput {
                text_area.disable_cursor();
            } else {
                text_area.enable_cursor(
                    app.current_theme
                        .keyboard_focus_style
                        .add_modifier(Modifier::REVERSED),
                );
            }
        }
        text_area.set_block(description_block.clone());
        text_area.clone()
    };
    let card_description_widget = card_description.widget();

    // Process Card Extra Info
    let (card_extra_info_widget, card_extra_info_items_len) = {
        let card_date_created = Span::styled(
            format!("Created: {}", card.date_created),
            app.current_theme.general_style,
        );
        let card_date_modified = Span::styled(
            format!("Modified: {}", card.date_modified),
            app.current_theme.general_style,
        );
        let card_date_completed = Span::styled(
            format!("Completed: {}", card.date_completed),
            app.current_theme.general_style,
        );
        let card_priority = format!("Priority: {}", card.priority);
        let card_status = format!("Status: {}", card.card_status);
        let card_due_date = card.due_date.clone();
        let parsed_due_date = date_format_converter(card_due_date.trim(), app.config.date_format);
        let card_due_date_styled = if let Ok(parsed_due_date) = parsed_due_date {
            if app.state.focus == Focus::CardDueDate {
                Span::styled(
                    format!("Due: {}", card.due_date),
                    app.current_theme.list_select_style,
                )
            } else if parsed_due_date == FIELD_NOT_SET || parsed_due_date.is_empty() {
                Span::styled(
                    format!("Due: {}", card.due_date),
                    app.current_theme.card_due_default_style,
                )
            } else {
                let formatted_date_format = date_format_finder(&parsed_due_date).unwrap();
                let days_left = match formatted_date_format {
                    DateFormat::DayMonthYear
                    | DateFormat::MonthDayYear
                    | DateFormat::YearMonthDay => {
                        let today = Local::now().date_naive();
                        let string_to_naive_date_format = NaiveDate::parse_from_str(
                            &parsed_due_date,
                            app.config.date_format.to_parser_string(),
                        )
                        .unwrap();
                        string_to_naive_date_format
                            .signed_duration_since(today)
                            .num_days()
                    }
                    DateFormat::DayMonthYearTime
                    | DateFormat::MonthDayYearTime
                    | DateFormat::YearMonthDayTime {} => {
                        let today = Local::now().naive_local();
                        let string_to_naive_date_format = NaiveDateTime::parse_from_str(
                            &parsed_due_date,
                            app.config.date_format.to_parser_string(),
                        )
                        .unwrap();
                        string_to_naive_date_format
                            .signed_duration_since(today)
                            .num_days()
                    }
                };
                if days_left <= app.config.warning_delta.into() && days_left >= 0 {
                    Span::styled(
                        format!("Due: {}", card.due_date),
                        app.current_theme.card_due_warning_style,
                    )
                } else if days_left < 0 {
                    Span::styled(
                        format!("Due: {}", card.due_date),
                        app.current_theme.card_due_overdue_style,
                    )
                } else {
                    Span::styled(
                        format!("Due: {}", card.due_date),
                        app.current_theme.card_due_default_style,
                    )
                }
            }
        } else if app.state.focus == Focus::CardDueDate {
            Span::styled(
                format!("Due: {}", card.due_date),
                app.current_theme.list_select_style,
            )
        } else {
            Span::styled(
                format!("Due: {}", card.due_date),
                app.current_theme.card_due_default_style,
            )
        };
        let card_priority_styled = if app.state.focus == Focus::CardPriority {
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
        let card_status_styled = if app.state.focus == Focus::CardStatus {
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
            ListItem::new(vec![Line::from(card_due_date_styled)]),
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
                .border_style(app.current_theme.general_style),
        );
        (card_extra_info, card_extra_info_items_len)
    };

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
                        app.current_theme.general_style,
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
                            app.current_theme.keyboard_focus_style,
                        ));
                    } else {
                        tags.push(Span::styled(
                            format!("{}) {} ", index + 1, tag),
                            app.current_theme.general_style,
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
                    app.current_theme.general_style,
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
                        app.current_theme.general_style,
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
                            app.current_theme.keyboard_focus_style,
                        ));
                    } else {
                        comments.push(Span::styled(
                            format!("{}) {} ", index + 1, comment),
                            app.current_theme.general_style,
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
                    app.current_theme.general_style,
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
        let max_box_width: u16 = popup_area.width - 2;
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

        let raw_card_description_height = if app.state.card_description_text_buffer.is_some() {
            app.state
                .card_description_text_buffer
                .as_ref()
                .unwrap()
                .lines()
                .len()
        } else {
            debug!("Text buffer not set for card description in render view card");
            card.description.len() / max_box_width as usize
        } as u16;

        let raw_tags_height = card_tag_lines.len() as u16;
        let raw_comments_height = card_comment_lines.len() as u16;

        let mut card_description_height = if app.state.focus == Focus::CardDescription {
            if available_height
                .saturating_sub(raw_tags_height + border_height)
                .saturating_sub(raw_comments_height + border_height)
                > 0
            {
                let calc =
                    available_height - raw_tags_height - raw_comments_height - (border_height * 2);
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

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[2]) {
        let top_of_list = card_chunks[2].y + 1;
        let mut bottom_of_list = card_chunks[2].y + card_extra_info_items_len as u16;
        if bottom_of_list > card_chunks[2].bottom() {
            bottom_of_list = card_chunks[2].bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            match mouse_y - top_of_list {
                2 => {
                    app.state.focus = Focus::CardDueDate;
                    app.state.mouse_focus = Some(Focus::CardDueDate);
                    app.state
                        .app_list_states
                        .card_view_comment_list
                        .select(None);
                    app.state.app_list_states.card_view_tag_list.select(None);
                    app.state.current_cursor_position = None;
                }
                4 => {
                    app.state.focus = Focus::CardPriority;
                    app.state.mouse_focus = Some(Focus::CardPriority);
                    app.state
                        .app_list_states
                        .card_view_comment_list
                        .select(None);
                    app.state.app_list_states.card_view_tag_list.select(None);
                    app.state.current_cursor_position = None;
                }
                5 => {
                    app.state.focus = Focus::CardStatus;
                    app.state.mouse_focus = Some(Focus::CardStatus);
                    app.state
                        .app_list_states
                        .card_view_comment_list
                        .select(None);
                    app.state.app_list_states.card_view_tag_list.select(None);
                    app.state.current_cursor_position = None;
                }
                _ => {
                    app.state.focus = Focus::NoFocus;
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
    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[0]) {
        app.state.focus = Focus::CardName;
        app.state.mouse_focus = Some(Focus::CardName);
        app.state
            .app_list_states
            .card_view_comment_list
            .select(None);
        app.state.app_list_states.card_view_tag_list.select(None);
        app.state.current_cursor_position = None;
    }
    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[1]) {
        app.state.focus = Focus::CardDescription;
        app.state.mouse_focus = Some(Focus::CardDescription);
        app.state
            .app_list_states
            .card_view_comment_list
            .select(None);
        app.state.app_list_states.card_view_tag_list.select(None);
        app.state.current_cursor_position = None;
    }
    let card_tags_style = if app.state.focus == Focus::CardTags {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    };
    let card_comments_style = if app.state.focus == Focus::CardComments {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    };

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

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[3]) {
        app.state.focus = Focus::CardTags;
        app.state.mouse_focus = Some(Focus::CardTags);
        app.state
            .app_list_states
            .card_view_comment_list
            .select(None);
    }

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[4]) {
        app.state.focus = Focus::CardComments;
        app.state.mouse_focus = Some(Focus::CardComments);
        app.state.app_list_states.card_view_tag_list.select(None);
    }

    if app.state.app_status == AppStatus::UserInput {
        match app.state.focus {
            Focus::CardName => {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(&card.name, card_chunks[0].width as usize),
                    app.state.current_cursor_position.unwrap_or(0),
                    card_chunks[0],
                );
                rect.set_cursor(x_pos, y_pos);
            }
            Focus::CardDueDate => {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(&card.due_date, card_chunks[2].width as usize),
                    app.state.current_cursor_position.unwrap_or(0),
                    card_chunks[2],
                );
                rect.set_cursor(x_pos + 5, y_pos + 2); // +5 and +2 are to account for the "Due: " text and extra info position offset
            }
            Focus::CardTags => {
                if app
                    .state
                    .app_list_states
                    .card_view_tag_list
                    .selected()
                    .is_some()
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
                    let x_pos = card_chunks[3].left()
                        + length_before_selected_tag as u16
                        + app.state.current_cursor_position.unwrap_or(0) as u16
                        + tag_offset
                        + digits_in_counter as u16;
                    let y_pos = card_chunks[3].top() + y_index as u16 + 1;
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
                    let x_pos = card_chunks[4].left()
                        + length_before_selected_comment as u16
                        + app.state.current_cursor_position.unwrap_or(0) as u16
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
    rect.render_widget(name_paragraph_widget, card_chunks[0]);
    rect.render_widget(card_description_widget, card_chunks[1]);
    rect.render_widget(card_extra_info_widget, card_chunks[2]);
    rect.render_widget(card_tags_widget, card_chunks[3]);
    rect.render_widget(card_comments_widget, card_chunks[4]);

    // Render Submit button if card is being edited
    if app.state.card_being_edited.is_some() {
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &card_chunks[5]) {
            app.state.focus = Focus::SubmitButton;
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state
                .app_list_states
                .card_view_comment_list
                .select(None);
            app.state.app_list_states.card_view_tag_list.select(None);
            app.state.current_cursor_position = None;
        }
        let save_changes_style = if app.state.focus == Focus::SubmitButton {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
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
        render_close_button(rect, app);
    }
}

pub fn render_command_palette(rect: &mut Frame, app: &mut App) {
    // Housekeeping
    match app.state.focus {
        Focus::CommandPaletteCommand => {
            if app
                .state
                .app_list_states
                .command_palette_command_search
                .selected()
                .is_none()
                && app.widgets.command_palette.command_search_results.is_some()
                && !app
                    .widgets
                    .command_palette
                    .command_search_results
                    .as_ref()
                    .unwrap()
                    .is_empty()
            {
                app.state
                    .app_list_states
                    .command_palette_command_search
                    .select(Some(0));
            }
        }
        Focus::CommandPaletteCard => {
            if app
                .state
                .app_list_states
                .command_palette_card_search
                .selected()
                .is_none()
                && app.widgets.command_palette.card_search_results.is_some()
                && !app
                    .widgets
                    .command_palette
                    .card_search_results
                    .as_ref()
                    .unwrap()
                    .is_empty()
            {
                app.state
                    .app_list_states
                    .command_palette_card_search
                    .select(Some(0));
            }
        }
        Focus::CommandPaletteBoard => {
            if app
                .state
                .app_list_states
                .command_palette_board_search
                .selected()
                .is_none()
                && app.widgets.command_palette.board_search_results.is_some()
                && !app
                    .widgets
                    .command_palette
                    .board_search_results
                    .as_ref()
                    .unwrap()
                    .is_empty()
            {
                app.state
                    .app_list_states
                    .command_palette_board_search
                    .select(Some(0));
            }
        }
        _ => {
            if app.state.app_status != AppStatus::UserInput {
                app.state.app_status = AppStatus::UserInput;
            }
            if app.state.focus != Focus::CommandPaletteCommand
                && app.state.focus != Focus::CommandPaletteBoard
                && app.state.focus != Focus::CommandPaletteCard
            {
                app.state.focus = Focus::CommandPaletteCommand;
            }
        }
    }

    let current_search_text_input = app.state.current_user_input.clone();
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
        .split(rect.size());

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

    let (command_search_border_style, command_search_text_style, command_search_highlight_style) =
        get_command_palette_style(app, Focus::CommandPaletteCommand);
    let (card_search_border_style, card_search_text_style, card_search_highlight_style) =
        get_command_palette_style(app, Focus::CommandPaletteCard);
    let (board_search_border_style, board_search_text_style, board_search_highlight_style) =
        get_command_palette_style(app, Focus::CommandPaletteBoard);

    let command_search_results = if app.widgets.command_palette.command_search_results.is_some() {
        let raw_search_results = app
            .widgets
            .command_palette
            .command_search_results
            .as_ref()
            .unwrap();
        let mut list_items = vec![];
        for item in raw_search_results {
            let mut spans = vec![];
            for c in item.to_string().chars() {
                if current_search_text_input
                    .to_lowercase()
                    .contains(c.to_string().to_lowercase().as_str())
                {
                    spans.push(Span::styled(
                        c.to_string(),
                        app.current_theme.keyboard_focus_style,
                    ));
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
        (rect.size().height - 14) as usize
    } else {
        (rect.size().height - 12) as usize
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
        if (command_search_results_length + card_search_results_length + min_height) < max_height {
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
        .style(
            app.current_theme
                .general_style
                .add_modifier(Modifier::RAPID_BLINK),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);
        rect.render_widget(Clear, vertical_chunks[0]);
        rect.render_widget(logged_in_indicator, vertical_chunks[0]);
    }

    let search_box_text = if app.state.current_user_input.is_empty() {
        vec![Line::from(
            "Start typing to search for a command, card or board!",
        )]
    } else {
        vec![Line::from(app.state.current_user_input.clone())]
    };

    let current_cursor_position = if app.state.current_cursor_position.is_some() {
        app.state.current_cursor_position.unwrap() as u16
    } else {
        app.state.current_user_input.len() as u16
    };
    let x_offset = current_cursor_position % (search_box_chunk.width - 2);
    let y_offset = current_cursor_position / (search_box_chunk.width - 2);
    let x_cursor_position = search_box_chunk.x + x_offset + 1;
    let y_cursor_position = search_box_chunk.y + y_offset + 1;
    rect.set_cursor(x_cursor_position, y_cursor_position);

    let search_box = Paragraph::new(search_box_text).block(
        Block::default()
            .title("Command Palette")
            .borders(Borders::ALL)
            .style(app.current_theme.general_style)
            .border_type(BorderType::Rounded),
    );
    render_blank_styled_canvas(rect, app, search_box_chunk, false);
    rect.render_widget(search_box, search_box_chunk);

    let results_border = Block::default()
        .style(app.current_theme.general_style)
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
        Span::styled("Use ", app.current_theme.help_text_style),
        Span::styled(up_key, app.current_theme.help_key_style),
        Span::styled(" and ", app.current_theme.help_text_style),
        Span::styled(down_key, app.current_theme.help_key_style),
        Span::styled(
            " or scroll with the mouse to highlight a Command/Card/Board. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" to select. Press ", app.current_theme.help_text_style),
        Span::styled(next_focus_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(prv_focus_key, app.current_theme.help_key_style),
        Span::styled(" to change focus", app.current_theme.help_text_style),
    ]);

    let help_paragraph = Paragraph::new(help_spans)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(app.current_theme.general_style),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: false });

    if check_if_mouse_is_in_area(
        &app.state.current_mouse_coordinates,
        &search_results_chunks[0],
    ) {
        app.state.mouse_focus = Some(Focus::CommandPaletteCommand);
        app.state.focus = Focus::CommandPaletteCommand;
    }
    if check_if_mouse_is_in_area(
        &app.state.current_mouse_coordinates,
        &search_results_chunks[1],
    ) {
        app.state.mouse_focus = Some(Focus::CommandPaletteCard);
        app.state.focus = Focus::CommandPaletteCard;
    }
    if check_if_mouse_is_in_area(
        &app.state.current_mouse_coordinates,
        &search_results_chunks[2],
    ) {
        app.state.mouse_focus = Some(Focus::CommandPaletteBoard);
        app.state.focus = Focus::CommandPaletteBoard;
    }

    render_blank_styled_canvas(rect, app, search_results_chunk, false);
    rect.render_widget(results_border, search_results_chunk);
    rect.render_stateful_widget(
        command_search_results_list,
        search_results_chunks[0],
        &mut app.state.app_list_states.command_palette_command_search,
    );
    rect.render_stateful_widget(
        card_search_results_list,
        search_results_chunks[1],
        &mut app.state.app_list_states.command_palette_card_search,
    );
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
            .style(app.current_theme.progress_bar_style)
            .end_symbol(SCROLLBAR_END_SYMBOL)
            .track_symbol(SCROLLBAR_TRACK_SYMBOL)
            .track_style(app.current_theme.inactive_text_style);

        let mut scrollbar_state =
            ScrollbarState::new(command_search_results.len()).position(current_index);
        let scrollbar_area = search_results_chunks[0].inner(&Margin {
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
            .style(app.current_theme.progress_bar_style)
            .end_symbol(SCROLLBAR_END_SYMBOL)
            .track_symbol(SCROLLBAR_TRACK_SYMBOL)
            .track_style(app.current_theme.inactive_text_style);

        let mut scrollbar_state =
            ScrollbarState::new(card_search_results.len()).position(current_index);
        let scrollbar_area = search_results_chunks[1].inner(&Margin {
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
            .style(app.current_theme.progress_bar_style)
            .end_symbol(SCROLLBAR_END_SYMBOL)
            .track_symbol(SCROLLBAR_TRACK_SYMBOL)
            .track_style(app.current_theme.inactive_text_style);

        let mut scrollbar_state =
            ScrollbarState::new(board_search_results.len()).position(current_index);
        let scrollbar_area = search_results_chunks[2].inner(&Margin {
            horizontal: 0,
            vertical: 1,
        });
        rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    render_blank_styled_canvas(rect, app, help_chunk, false);
    rect.render_widget(help_paragraph, help_chunk);
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_change_ui_mode_popup(rect: &mut Frame, app: &mut App) {
    let all_ui_modes = UiMode::view_modes_as_string()
        .iter()
        .map(|s| ListItem::new(vec![Line::from(s.as_str().to_string())]))
        .collect::<Vec<ListItem>>();

    let popup_area = centered_rect_with_length(40, 10, rect.size());

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &popup_area) {
        app.state.mouse_focus = Some(Focus::ChangeUiModePopup);
        app.state.focus = Focus::ChangeUiModePopup;
        calculate_mouse_list_select_index(
            app.state.current_mouse_coordinates.1,
            &all_ui_modes,
            popup_area,
            &mut app.state.app_list_states.default_view,
        );
    }
    let ui_modes = List::new(all_ui_modes)
        .block(
            Block::default()
                .title("Change UI Mode")
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        ui_modes,
        popup_area,
        &mut app.state.app_list_states.default_view,
    );
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_change_date_format_popup(rect: &mut Frame, app: &mut App) {
    let render_area = centered_rect_with_percentage(70, 70, rect.size());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
        .split(render_area);

    let all_date_formats = DateFormat::get_all_date_formats();
    let all_date_formats = all_date_formats
        .iter()
        .map(|s| ListItem::new(vec![Line::from(s.to_human_readable_string().to_string())]))
        .collect::<Vec<ListItem>>();

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &render_area) {
        app.state.mouse_focus = Some(Focus::ChangeDateFormatPopup);
        app.state.focus = Focus::ChangeDateFormatPopup;
        calculate_mouse_list_select_index(
            app.state.current_mouse_coordinates.1,
            &all_date_formats,
            render_area,
            &mut app.state.app_list_states.date_format_selector,
        );
    }
    let date_formats = List::new(all_date_formats)
        .block(
            Block::default()
                .title("Change Date Format")
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    let up_key = app
        .get_first_keybinding(KeyBindingEnum::Up)
        .unwrap_or("".to_string());
    let down_key = app
        .get_first_keybinding(KeyBindingEnum::Down)
        .unwrap_or("".to_string());
    let accept_key = app
        .get_first_keybinding(KeyBindingEnum::Accept)
        .unwrap_or("".to_string());
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());

    let help_spans = Line::from(vec![
        Span::styled("Use ", app.current_theme.help_text_style),
        Span::styled(up_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(down_key, app.current_theme.help_key_style),
        Span::styled(
            " to navigate or use the mouse cursor. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled("<Mouse Left Click>", app.current_theme.help_key_style),
        Span::styled(
            " To select a Default Date Format. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(cancel_key, app.current_theme.help_key_style),
        Span::styled(" to cancel", app.current_theme.help_text_style),
    ]);

    let default_date_picker_help = Paragraph::new(help_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(app.current_theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let clear_area = centered_rect_with_percentage(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Default Date Format Picker")
        .style(app.current_theme.general_style)
        .borders(Borders::ALL)
        .border_style(app.current_theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);
    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    rect.render_stateful_widget(
        date_formats,
        chunks[0],
        &mut app.state.app_list_states.date_format_selector,
    );
    rect.render_widget(default_date_picker_help, chunks[1]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_change_card_status_popup(rect: &mut Frame, app: &mut App) {
    let mut card_name = String::new();
    let mut board_name = String::new();
    let boards = if app.filtered_boards.is_empty() {
        app.boards.clone()
    } else {
        app.filtered_boards.clone()
    };
    if let Some(current_board_id) = app.state.current_board_id {
        if let Some(current_board) = boards.iter().find(|b| b.id == current_board_id) {
            if let Some(current_card_id) = app.state.current_card_id {
                if let Some(current_card) =
                    current_board.cards.iter().find(|c| c.id == current_card_id)
                {
                    card_name = current_card.name.clone();
                    board_name = current_board.name.clone();
                }
            }
        }
    }
    let all_statuses = CardStatus::all()
        .iter()
        .map(|s| ListItem::new(vec![Line::from(s.to_string())]))
        .collect::<Vec<ListItem>>();
    let percent_height =
        (((all_statuses.len() + 3) as f32 / rect.size().height as f32) * 100.0) as u16;
    let popup_area = centered_rect_with_percentage(50, percent_height, rect.size());
    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &popup_area) {
        app.state.mouse_focus = Some(Focus::ChangeCardStatusPopup);
        app.state.focus = Focus::ChangeCardStatusPopup;
        calculate_mouse_list_select_index(
            app.state.current_mouse_coordinates.1,
            &all_statuses,
            popup_area,
            &mut app.state.app_list_states.card_status_selector,
        );
    }
    let statuses = List::new(all_statuses)
        .block(
            Block::default()
                .title(format!(
                    "Changing Status of \"{}\" in {}",
                    card_name, board_name
                ))
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        statuses,
        popup_area,
        &mut app.state.app_list_states.card_status_selector,
    );
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_card_priority_selector(rect: &mut Frame, app: &mut App) {
    let mut card_name = String::new();
    let mut board_name = String::new();
    let boards = if app.filtered_boards.is_empty() {
        app.boards.clone()
    } else {
        app.filtered_boards.clone()
    };
    if let Some(current_board_id) = app.state.current_board_id {
        if let Some(current_board) = boards.iter().find(|b| b.id == current_board_id) {
            if let Some(current_card_id) = app.state.current_card_id {
                if let Some(current_card) =
                    current_board.cards.iter().find(|c| c.id == current_card_id)
                {
                    card_name = current_card.name.clone();
                    board_name = current_board.name.clone();
                }
            }
        }
    }
    let all_priorities = CardPriority::all()
        .iter()
        .map(|p| ListItem::new(vec![Line::from(p.to_string())]))
        .collect::<Vec<ListItem>>();
    let percent_height =
        (((all_priorities.len() + 3) as f32 / rect.size().height as f32) * 100.0) as u16;
    let popup_area = centered_rect_with_percentage(50, percent_height, rect.size());
    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &popup_area) {
        app.state.mouse_focus = Some(Focus::ChangeCardPriorityPopup);
        app.state.focus = Focus::ChangeCardPriorityPopup;
        calculate_mouse_list_select_index(
            app.state.current_mouse_coordinates.1,
            &all_priorities,
            popup_area,
            &mut app.state.app_list_states.card_priority_selector,
        );
    }
    let priorities = List::new(all_priorities)
        .block(
            Block::default()
                .title(format!(
                    "Changing Priority of \"{}\" in {}",
                    card_name, board_name
                ))
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        priorities,
        popup_area,
        &mut app.state.app_list_states.card_priority_selector,
    );
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_popup<T: ToString>(
    rect: &mut Frame,
    app: &mut App,
    focus: Focus,
    list_items: &[T],
    list_state: &mut ListState,
    title: &str,
    popup_area: Rect,
) {
    let items = list_items
        .iter()
        .map(|s| ListItem::new(vec![Line::from(s.to_string())]))
        .collect::<Vec<ListItem>>();

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &popup_area) {
        app.state.mouse_focus = Some(focus);
        app.state.focus = focus;
        calculate_mouse_list_select_index(
            app.state.current_mouse_coordinates.1,
            &items,
            popup_area,
            list_state,
        );
    }
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(list, popup_area, list_state);
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_filter_by_tag_popup(rect: &mut Frame, app: &mut App) {
    if app.state.all_available_tags.is_some() {
        let submit_style = if app.state.focus == Focus::SubmitButton {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };

        let tag_box_style = if app.state.focus == Focus::FilterByTagPopup {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
        let scrollbar_style = app.current_theme.progress_bar_style;

        let popup_area = centered_rect_with_percentage(80, 80, rect.size());

        let all_available_tags = app.state.all_available_tags.as_ref().unwrap();
        let empty_vec = vec![];
        let selected_tags = if app.state.filter_tags.is_some() {
            app.state.filter_tags.as_ref().unwrap()
        } else {
            &empty_vec
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(popup_area);

        let all_tags = all_available_tags
            .iter()
            .map(|tag| {
                if selected_tags.contains(&tag.0) {
                    ListItem::new(vec![Line::from(vec![Span::styled(
                        format!("(Selected) {} - {} occurrences", tag.0, tag.1),
                        app.current_theme.list_select_style,
                    )])])
                } else {
                    ListItem::new(vec![Line::from(vec![Span::styled(
                        format!("{} - {} occurrences", tag.0, tag.1),
                        app.current_theme.general_style,
                    )])])
                }
            })
            .collect::<Vec<ListItem>>();

        let tags = List::new(all_tags.clone())
            .block(
                Block::default()
                    .title("Filter by Tag")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.current_theme.general_style)
                    .border_style(tag_box_style),
            )
            .highlight_style(app.current_theme.list_select_style)
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
        let cancel_key = app
            .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
            .unwrap_or("".to_string());

        let help_spans = Line::from(vec![
            Span::styled("Use ", app.current_theme.help_text_style),
            Span::styled(up_key, app.current_theme.help_key_style),
            Span::styled(" and ", app.current_theme.help_text_style),
            Span::styled(down_key, app.current_theme.help_key_style),
            Span::styled(
                " or scroll with the mouse to navigate. Press ",
                app.current_theme.help_text_style,
            ),
            Span::styled(accept_key.clone(), app.current_theme.help_key_style),
            Span::styled(
                " To select a Tag (multiple tags can be selected). Press ",
                app.current_theme.help_text_style,
            ),
            Span::styled(accept_key, app.current_theme.help_key_style),
            Span::styled(
                " on an already selected tag to deselect it. Press ",
                app.current_theme.help_text_style,
            ),
            Span::styled(cancel_key, app.current_theme.help_key_style),
            Span::styled(" to cancel, Press ", app.current_theme.help_text_style),
            Span::styled(next_focus_key, app.current_theme.help_key_style),
            Span::styled(" or ", app.current_theme.help_text_style),
            Span::styled(prv_focus_key, app.current_theme.help_key_style),
            Span::styled(" to change focus", app.current_theme.help_text_style),
        ]);

        let help = Paragraph::new(help_spans)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .style(app.current_theme.general_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let submit_btn_text = if app.state.filter_tags.is_some() {
            if app.state.filter_tags.as_ref().unwrap().len() > 1 {
                "Confirm filters"
            } else {
                "Confirm filter"
            }
        } else {
            "Confirm filter"
        };

        let submit_button = Paragraph::new(submit_btn_text)
            .block(
                Block::default()
                    .title("Submit")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.current_theme.general_style)
                    .border_style(submit_style),
            )
            .alignment(Alignment::Center);

        let current_index = app
            .state
            .app_list_states
            .filter_by_tag_list
            .selected()
            .unwrap_or(0);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
            .style(scrollbar_style)
            .end_symbol(SCROLLBAR_END_SYMBOL)
            .track_symbol(SCROLLBAR_TRACK_SYMBOL)
            .track_style(app.current_theme.inactive_text_style);
        let mut scrollbar_state = ScrollbarState::new(all_tags.len()).position(current_index);
        let scrollbar_area = chunks[0].inner(&Margin {
            vertical: 1,
            horizontal: 0,
        });

        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[0]) {
            app.state.mouse_focus = Some(Focus::FilterByTagPopup);
            app.state.focus = Focus::FilterByTagPopup;
        }
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[2]) {
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state.focus = Focus::SubmitButton;
        }

        render_blank_styled_canvas(rect, app, popup_area, false);
        rect.render_stateful_widget(
            tags,
            chunks[0],
            &mut app.state.app_list_states.filter_by_tag_list,
        );
        rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        rect.render_widget(help, chunks[1]);
        rect.render_widget(submit_button, chunks[2]);
    }

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_debug_panel(rect: &mut Frame, app: &mut App) {
    let current_ui_mode = &app.state.ui_mode.to_string();
    let popup_mode = if app.state.popup_mode.is_some() {
        app.state.popup_mode.as_ref().unwrap().to_string()
    } else {
        "None".to_string()
    };
    let ui_render_time = if !app.state.ui_render_time.is_empty() {
        // let render_time =
        //     app.state.ui_render_time.iter().sum::<u128>() / app.state.ui_render_time.len() as u128;
        // minimum value of the array
        let render_time = *app.state.ui_render_time.iter().min().unwrap();
        if render_time > 1000 {
            format!("{}ms", render_time as f64 / 1000_f64)
        } else {
            format!("{}μs", render_time)
        }
    } else {
        "None".to_string()
    };
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;

    let menu_area = top_left_rect(38, 10, rect.size());
    let strings = vec![
        format!("UI Mode: {}", current_ui_mode),
        format!("Focus: {:?}", app.state.focus),
        format!("CMousePos: {:?}", app.state.current_mouse_coordinates),
        format!("Popup Mode: {}", popup_mode),
        format!("Render Time: {}", ui_render_time),
        format!("CB-ID: {:?}", current_board_id),
        format!("CC-ID: {:?}", current_card_id),
    ];
    let strings = strings
        .iter()
        .map(|s| {
            if s.len() > menu_area.width as usize - 2 {
                Line::from(format!("{}{}", &s[..menu_area.width as usize - 5], "..."))
            } else {
                Line::from(s.to_string())
            }
        })
        .collect::<Vec<Line>>();
    let debug_panel = Paragraph::new(strings).block(
        Block::default()
            .title("Debug Panel")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(app.current_theme.general_style)
            .border_style(app.current_theme.log_debug_style),
    );

    render_blank_styled_canvas(rect, app, menu_area, false);
    rect.render_widget(debug_panel, menu_area);
}

pub fn check_if_mouse_is_in_area(mouse_coordinates: &(u16, u16), rect_to_check: &Rect) -> bool {
    let (x, y) = mouse_coordinates;
    let (x1, y1, x2, y2) = (
        rect_to_check.x,
        rect_to_check.y,
        rect_to_check.x + rect_to_check.width,
        rect_to_check.y + rect_to_check.height,
    );
    if x >= &x1 && x <= &x2 && y >= &y1 && y <= &y2 {
        return true;
    }
    false
}

fn render_close_button(rect: &mut Frame, app: &mut App) {
    let close_btn_area = Rect::new(rect.size().width - 3, 0, 3, 3);
    let close_btn_style =
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &close_btn_area)
            || app.state.focus == Focus::CloseButton
        {
            app.state.mouse_focus = Some(Focus::CloseButton);
            app.state.focus = Focus::CloseButton;
            let close_button_color = app.widgets.close_button_widget.color;
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
        } else {
            app.current_theme.general_style
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

pub fn render_change_theme_popup(rect: &mut Frame, app: &mut App) {
    let render_area = centered_rect_with_percentage(70, 70, rect.size());
    let clear_area = centered_rect_with_percentage(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Change Theme")
        .style(app.current_theme.general_style)
        .borders(Borders::ALL)
        .border_style(app.current_theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
        .split(render_area);

    let theme_list = app
        .all_themes
        .iter()
        .map(|t| ListItem::new(vec![Line::from(t.name.clone())]))
        .collect::<Vec<ListItem>>();

    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[0]) {
        app.state.mouse_focus = Some(Focus::ThemeSelector);
        app.state.focus = Focus::ThemeSelector;
        let top_of_list = chunks[0].y + 1;
        let mut bottom_of_list = chunks[0].y + theme_list.len() as u16;
        if bottom_of_list > chunks[0].bottom() {
            bottom_of_list = chunks[0].bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .app_list_states
                .theme_selector
                .select(Some((mouse_y - top_of_list) as usize));
            let selected_theme = app
                .all_themes
                .get(app.state.app_list_states.theme_selector.selected().unwrap())
                .unwrap();
            app.current_theme = selected_theme.clone();
        } else {
            app.state.app_list_states.theme_selector.select(None);
        }
    };
    let themes = List::new(theme_list)
        .block(
            Block::default()
                .style(app.current_theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    let up_key = app
        .get_first_keybinding(KeyBindingEnum::Up)
        .unwrap_or("".to_string());
    let down_key = app
        .get_first_keybinding(KeyBindingEnum::Down)
        .unwrap_or("".to_string());
    let accept_key = app
        .get_first_keybinding(KeyBindingEnum::Accept)
        .unwrap_or("".to_string());
    let cancel_key = app
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
        .unwrap_or("".to_string());

    let help_spans = Line::from(vec![
        Span::styled("Use ", app.current_theme.help_text_style),
        Span::styled(up_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(down_key, app.current_theme.help_key_style),
        Span::styled(
            " to navigate or use the mouse cursor. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled("<Mouse Left Click>", app.current_theme.help_key_style),
        Span::styled(
            " To select a Theme. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(cancel_key, app.current_theme.help_key_style),
        Span::styled(" to cancel", app.current_theme.help_text_style),
    ]);

    let change_theme_help = Paragraph::new(help_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(app.current_theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    rect.render_stateful_widget(
        themes,
        chunks[0],
        &mut app.state.app_list_states.theme_selector,
    );
    rect.render_widget(change_theme_help, chunks[1]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_create_theme(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    let render_area = rect.size();
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(3)].as_ref())
        .split(render_area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
        .margin(1)
        .split(main_chunks[0]);
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
        .split(main_chunks[1]);

    let submit_button_style = get_mouse_focusable_field_style(
        app,
        Focus::SubmitButton,
        &button_chunks[0],
        popup_mode,
        false,
    );
    let reset_button_style = get_mouse_focusable_field_style(
        app,
        Focus::ExtraFocus,
        &button_chunks[1],
        popup_mode,
        false,
    );

    let theme_table_rows = app.state.theme_being_edited.to_rows(app);
    let list_highlight_style = if app.state.popup_mode.is_some() {
        app.current_theme.inactive_text_style
    } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &main_chunks[0]) {
        app.state.mouse_focus = Some(Focus::ThemeEditor);
        app.state.focus = Focus::ThemeEditor;
        let top_of_list = main_chunks[0].y + 1;
        let mut bottom_of_list = main_chunks[0].y + theme_table_rows.0.len() as u16;
        if bottom_of_list > main_chunks[0].bottom() {
            bottom_of_list = main_chunks[0].bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .app_table_states
                .theme_editor
                .select(Some((mouse_y - top_of_list) as usize));
        } else {
            app.state.app_table_states.theme_editor.select(None);
        }
        app.current_theme.list_select_style
    } else if app.state.app_table_states.theme_editor.selected().is_some() {
        app.current_theme.list_select_style
    } else {
        app.current_theme.general_style
    };
    let theme_block_style = if app.state.popup_mode.is_some() {
        app.current_theme.inactive_text_style
    } else if app.state.focus == Focus::ThemeEditor {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    };
    let theme_title_list = Table::new(theme_table_rows.0, [Constraint::Fill(1)])
        .block(Block::default().style(app.current_theme.general_style))
        .highlight_style(list_highlight_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let theme_element_list =
        Table::new(theme_table_rows.1, [Constraint::Fill(1)]).block(Block::default());
    let submit_button = Paragraph::new(vec![Line::from("Create Theme")])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(submit_button_style),
        )
        .alignment(Alignment::Center);

    let reset_button = Paragraph::new(vec![Line::from("Reset")])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(reset_button_style),
        )
        .alignment(Alignment::Center);

    let border_block = Block::default()
        .title("Create a new Theme")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme_block_style);

    rect.render_stateful_widget(
        theme_title_list,
        chunks[0],
        &mut app.state.app_table_states.theme_editor,
    );
    rect.render_stateful_widget(
        theme_element_list,
        chunks[1],
        &mut app.state.app_table_states.theme_editor,
    );
    rect.render_widget(submit_button, button_chunks[0]);
    rect.render_widget(reset_button, button_chunks[1]);
    rect.render_widget(border_block, main_chunks[0]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_edit_specific_style_popup(rect: &mut Frame, app: &mut App) {
    let popup_area = centered_rect_with_percentage(90, 80, rect.size());
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Length(4),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(popup_area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .split(main_chunks[0]);
    let fg_list_border_style =
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[0]) {
            if app
                .state
                .app_list_states
                .edit_specific_style
                .0
                .selected()
                .is_none()
            {
                app.state
                    .app_list_states
                    .edit_specific_style
                    .0
                    .select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorFG);
            app.state.focus = Focus::StyleEditorFG;
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorFG {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
    let bg_list_border_style =
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1]) {
            if app
                .state
                .app_list_states
                .edit_specific_style
                .1
                .selected()
                .is_none()
            {
                app.state
                    .app_list_states
                    .edit_specific_style
                    .1
                    .select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorBG);
            app.state.focus = Focus::StyleEditorBG;
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorBG {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
    let modifiers_list_border_style =
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[2]) {
            if app
                .state
                .app_list_states
                .edit_specific_style
                .2
                .selected()
                .is_none()
            {
                app.state
                    .app_list_states
                    .edit_specific_style
                    .2
                    .select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorModifier);
            app.state.focus = Focus::StyleEditorModifier;
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorModifier {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
    let submit_button_style =
        get_mouse_focusable_field_style(app, Focus::SubmitButton, &main_chunks[1], false, false);
    let fg_list_items: Vec<ListItem> = TextColorOptions::to_iter()
        .map(|color| {
            let mut fg_style = Style::default();
            let current_user_input = app.state.current_user_input.clone();
            let mut is_rgb = false;
            if let TextColorOptions::RGB(_, _, _) = color {
                is_rgb = true;
            }
            if !current_user_input.is_empty() && is_rgb {
                let split_input = current_user_input
                    .split(',')
                    .map(|s| s.to_string().trim().to_string());
                if split_input.clone().count() == 3 {
                    let mut input_is_valid = true;
                    for i in split_input.clone() {
                        if i.parse::<u8>().is_err() {
                            input_is_valid = false;
                        }
                    }
                    if input_is_valid {
                        let r = split_input.clone().next().unwrap().parse::<u8>().unwrap();
                        let g = split_input.clone().nth(1).unwrap().parse::<u8>().unwrap();
                        let b = split_input.clone().nth(2).unwrap().parse::<u8>().unwrap();
                        fg_style.fg = Some(ratatui::style::Color::Rgb(r, g, b));
                        return ListItem::new(vec![Line::from(vec![
                            Span::styled("Sample Text", fg_style),
                            Span::styled(
                                format!(" - RGB({},{},{})", r, g, b),
                                app.current_theme.general_style,
                            ),
                        ])]);
                    }
                }
            }
            return if color.to_color().is_some() {
                fg_style.fg = Some(color.to_color().unwrap());
                ListItem::new(vec![Line::from(vec![
                    Span::styled("Sample Text", fg_style),
                    Span::styled(format!(" - {}", color), app.current_theme.general_style),
                ])])
            } else {
                ListItem::new(vec![Line::from(vec![
                    Span::raw("Sample Text"),
                    Span::styled(format!(" - {}", color), app.current_theme.general_style),
                ])])
            };
        })
        .collect();
    let fg_list = List::new(fg_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Foreground")
                .border_style(fg_list_border_style),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let bg_list_items: Vec<ListItem> = TextColorOptions::to_iter()
        .map(|color| {
            let mut bg_style = Style::default();
            let current_user_input = app.state.current_user_input.clone();
            let mut is_rgb = false;
            if let TextColorOptions::RGB(_, _, _) = color {
                is_rgb = true;
            }
            if !current_user_input.is_empty() && is_rgb {
                let split_input = current_user_input
                    .split(',')
                    .map(|s| s.to_string().trim().to_string());
                if split_input.clone().count() == 3 {
                    let mut input_is_valid = true;
                    for i in split_input.clone() {
                        if i.parse::<u8>().is_err() {
                            input_is_valid = false;
                        }
                    }
                    if input_is_valid {
                        let r = split_input.clone().next().unwrap().parse::<u8>().unwrap();
                        let g = split_input.clone().nth(1).unwrap().parse::<u8>().unwrap();
                        let b = split_input.clone().nth(2).unwrap().parse::<u8>().unwrap();
                        bg_style.bg = Some(ratatui::style::Color::Rgb(r, g, b));
                        return ListItem::new(vec![Line::from(vec![
                            Span::styled("Sample Text", bg_style),
                            Span::styled(
                                format!(" - RGB({},{},{})", r, g, b),
                                app.current_theme.general_style,
                            ),
                        ])]);
                    }
                }
            }
            return if color.to_color().is_some() {
                bg_style.bg = Some(color.to_color().unwrap());
                ListItem::new(vec![Line::from(vec![
                    Span::styled("Sample Text", bg_style),
                    Span::styled(format!(" - {}", color), app.current_theme.general_style),
                ])])
            } else {
                ListItem::new(vec![Line::from(vec![
                    Span::raw("Sample Text"),
                    Span::styled(format!(" - {}", color), app.current_theme.general_style),
                ])])
            };
        })
        .collect();
    let bg_list = List::new(bg_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Background")
                .border_style(bg_list_border_style),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let modifier_list_items: Vec<ListItem> = TextModifierOptions::to_iter()
        .map(|modifier| {
            let modifier_style = Style {
                add_modifier: modifier.to_modifier(),
                ..Style::default()
            };
            ListItem::new(vec![Line::from(vec![
                Span::styled("Sample Text", modifier_style),
                Span::styled(format!(" - {}", modifier), app.current_theme.general_style),
            ])])
        })
        .collect();
    let modifier_list = List::new(modifier_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Modifier")
                .border_style(modifiers_list_border_style),
        )
        .highlight_style(app.current_theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let theme_style_being_edited_index = app.state.app_table_states.theme_editor.selected();
    let theme_style_being_edited = if let Some(index) = theme_style_being_edited_index {
        let theme_style_being_edited = app.state.theme_being_edited.to_vec_str();
        if index < theme_style_being_edited.len() {
            theme_style_being_edited[index]
        } else {
            debug!("Index is out of bounds for theme_style_being_edited");
            "None"
        }
    } else {
        "None"
    };
    let border_block = Block::default()
        .title(format!("Editing Style: {}", theme_style_being_edited))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.current_theme.general_style);

    let submit_button = Paragraph::new("Confirm Changes")
        .style(submit_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(submit_button_style),
        )
        .alignment(Alignment::Center);

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

    let help_spans = vec![
        Span::styled("Use ", app.current_theme.help_text_style),
        Span::styled(up_key, app.current_theme.help_key_style),
        Span::styled(" and ", app.current_theme.help_text_style),
        Span::styled(down_key, app.current_theme.help_key_style),
        Span::styled(
            " or scroll with the mouse",
            app.current_theme.help_text_style,
        ),
        Span::styled(
            " to select a Color/Modifier, Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled("<Mouse Left Click>", app.current_theme.help_key_style),
        Span::styled(
            " to edit (for custom RBG), Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(next_focus_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(prv_focus_key, app.current_theme.help_key_style),
        Span::styled(" to change focus.", app.current_theme.help_text_style),
    ];

    let help_text = Paragraph::new(Line::from(help_spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(app.current_theme.help_text_style),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        fg_list,
        chunks[0],
        &mut app.state.app_list_states.edit_specific_style.0,
    );
    rect.render_stateful_widget(
        bg_list,
        chunks[1],
        &mut app.state.app_list_states.edit_specific_style.1,
    );
    rect.render_stateful_widget(
        modifier_list,
        chunks[2],
        &mut app.state.app_list_states.edit_specific_style.2,
    );
    rect.render_widget(help_text, main_chunks[1]);
    rect.render_widget(submit_button, main_chunks[2]);
    rect.render_widget(border_block, popup_area);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_save_theme_prompt(rect: &mut Frame, app: &mut App) {
    let popup_area = centered_rect_with_length(40, 10, rect.size());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
        .margin(2)
        .split(popup_area);

    let save_theme_button_style =
        get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[0], false, false);
    let dont_save_theme_button_style =
        get_mouse_focusable_field_style(app, Focus::ExtraFocus, &chunks[1], false, false);
    let save_theme_button = Paragraph::new("Save Theme to File")
        .style(save_theme_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(save_theme_button_style),
        )
        .alignment(Alignment::Center);
    let dont_save_theme_button = Paragraph::new("Don't Save Theme to File")
        .style(dont_save_theme_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(dont_save_theme_button_style),
        )
        .alignment(Alignment::Center);
    let border_block = Block::default()
        .title("Save Theme?")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.current_theme.general_style);

    render_blank_styled_canvas(rect, app, popup_area, true);
    rect.render_widget(save_theme_button, chunks[0]);
    rect.render_widget(dont_save_theme_button, chunks[1]);
    rect.render_widget(border_block, popup_area);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_confirm_discard_card_changes(rect: &mut Frame, app: &mut App) {
    let popup_area = centered_rect_with_length(30, 7, rect.size());

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
        .margin(2)
        .split(popup_area);

    let save_card_button_style =
        get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[0], false, false);
    let dont_save_card_button_style =
        get_mouse_focusable_field_style(app, Focus::ExtraFocus, &chunks[1], false, false);
    let save_theme_button = Paragraph::new("Yes")
        .style(save_card_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(save_card_button_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    let dont_save_theme_button = Paragraph::new("No")
        .style(dont_save_card_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(dont_save_card_button_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);
    let border_block = Block::default()
        .title("Save Changes to Card?")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.current_theme.general_style);

    render_blank_styled_canvas(rect, app, popup_area, true);
    rect.render_widget(save_theme_button, chunks[0]);
    rect.render_widget(dont_save_theme_button, chunks[1]);
    rect.render_widget(border_block, popup_area);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_custom_rgb_color_prompt(rect: &mut Frame, app: &mut App) {
    let popup_area = centered_rect_with_length(60, 18, rect.size());
    let prompt_text = "Enter a custom RGB color in the format: r,g,b (0-254)";

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(popup_area);
    let border_block = Block::default()
        .title("Custom RGB Color")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.current_theme.general_style);

    let text_input_style =
        get_mouse_focusable_field_style(app, Focus::TextInput, &chunks[1], false, true);
    let submit_button_style =
        get_mouse_focusable_field_style(app, Focus::SubmitButton, &chunks[2], false, false);
    let prompt_text = Paragraph::new(prompt_text)
        .style(app.current_theme.general_style)
        .block(Block::default())
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
    let text_input = Paragraph::new(app.state.current_user_input.clone())
        .style(app.current_theme.general_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(text_input_style)
                .border_type(BorderType::Rounded),
        );

    let input_mode_key = app
        .get_first_keybinding(KeyBindingEnum::TakeUserInput)
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
    let stop_editing_key = app
        .get_first_keybinding(KeyBindingEnum::StopUserInput)
        .unwrap_or("".to_string());

    let help_spans = vec![
        Span::styled("Press ", app.current_theme.help_text_style),
        Span::styled(input_mode_key, app.current_theme.help_key_style),
        Span::styled(
            " to enter input mode. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(stop_editing_key, app.current_theme.help_key_style),
        Span::styled(" to stop editing. Use ", app.current_theme.help_text_style),
        Span::styled(next_focus_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(prv_focus_key, app.current_theme.help_key_style),
        Span::styled(
            " to change focus. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" to submit.", app.current_theme.help_text_style),
    ];
    let help_text = Paragraph::new(Line::from(help_spans))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.current_theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let submit_button = Paragraph::new("Submit")
        .style(app.current_theme.general_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(submit_button_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);

    render_blank_styled_canvas(rect, app, popup_area, true);
    rect.render_widget(prompt_text, chunks[0]);
    rect.render_widget(text_input, chunks[1]);
    rect.render_widget(submit_button, chunks[2]);
    rect.render_widget(help_text, chunks[3]);
    rect.render_widget(border_block, popup_area);

    if app.state.app_status == AppStatus::UserInput {
        let current_cursor_position = if app.state.current_cursor_position.is_some() {
            app.state.current_cursor_position.unwrap() as u16
        } else {
            app.state.current_user_input.len() as u16
        };
        let x_offset = current_cursor_position % (chunks[1].width - 2);
        let y_offset = current_cursor_position / (chunks[1].width - 2);
        let x_cursor_position = chunks[1].x + x_offset + 1;
        let y_cursor_position = chunks[1].y + y_offset + 1;
        rect.set_cursor(x_cursor_position, y_cursor_position);
    }
}

pub fn render_blank_styled_canvas(
    rect: &mut Frame,
    app: &mut App,
    render_area: Rect,
    popup_mode: bool,
) {
    let mut styled_text = vec![];
    for _ in 0..render_area.width + 1 {
        styled_text.push(" ".to_string());
    }
    let mut render_text = vec![];
    for _ in 0..render_area.height {
        render_text.push(format!("{}\n", styled_text.join("")));
    }
    let styled_text = if popup_mode {
        Paragraph::new(render_text.join(""))
            .style(app.current_theme.inactive_text_style)
            .block(Block::default())
    } else {
        Paragraph::new(render_text.join(""))
            .style(app.current_theme.general_style)
            .block(Block::default())
    };
    rect.render_widget(styled_text, render_area);
}

pub fn render_blank_styled_canvas_with_margin(
    rect: &mut Frame,
    app: &mut App,
    render_area: Rect,
    popup_mode: bool,
    margin: i16,
) {
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
    let styled_text = if popup_mode {
        Paragraph::new(render_text.join(""))
            .style(app.current_theme.inactive_text_style)
            .block(Block::default())
    } else {
        Paragraph::new(render_text.join(""))
            .style(app.current_theme.general_style)
            .block(Block::default())
    };
    rect.render_widget(styled_text, new_render_area);
}

fn render_title(app: &mut App<'_>, render_area: &Rect, rect: &mut Frame<'_>) {
    rect.render_widget(draw_title(app, *render_area), *render_area);
}

pub fn render_logs(
    app: &mut App,
    enable_focus_highlight: bool,
    render_area: Rect,
    rect: &mut Frame,
    popup_mode: bool,
) {
    let log_box_border_style = if enable_focus_highlight {
        get_mouse_focusable_field_style(app, Focus::Log, &render_area, popup_mode, false)
    } else {
        app.current_theme.general_style
    };
    let date_format = app.config.date_format.to_parser_string();
    let theme = &app.current_theme;
    let all_logs = get_logs();
    let mut highlight_style =
        check_for_popup_and_get_style(app, app.current_theme.list_select_style);
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
    // TODO: Optimise this by using the log state to avoid going through all the logs and only go throught the ones that can fit in the render area
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
        let style = if popup_mode {
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

    let log_box_style = check_for_popup_and_get_style(app, app.current_theme.general_style);

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

pub fn render_login(rect: &mut Frame, app: &mut App) {
    if app.state.popup_mode.is_none() {
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
        app.state.popup_mode.is_some(),
        true,
    );
    let password_field_style = get_mouse_focusable_field_style(
        app,
        Focus::PasswordField,
        &password_field_chunk,
        app.state.popup_mode.is_some(),
        true,
    );
    let show_password_style = get_mouse_focusable_field_style(
        app,
        Focus::ExtraFocus,
        &show_password_main_chunk,
        app.state.popup_mode.is_some(),
        true,
    );
    let submit_button_style = get_mouse_focusable_field_style(
        app,
        Focus::SubmitButton,
        &submit_button_chunks[1],
        app.state.popup_mode.is_some(),
        true,
    );

    let separator_style = check_for_popup_and_get_style(app, app.current_theme.general_style);

    let crab_paragraph = if app.state.popup_mode.is_some() {
        draw_crab_pattern(
            chunks[0],
            app.current_theme.inactive_text_style,
            true,
            app.config.disable_animations,
        )
    } else {
        draw_crab_pattern(
            chunks[0],
            app.current_theme.general_style,
            false,
            app.config.disable_animations,
        )
    };

    let info_border = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(separator_style);

    let info_paragraph = Paragraph::new("Log In")
        .style(app.current_theme.general_style)
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
        Span::styled("Press ", app.current_theme.help_text_style),
        Span::styled(next_focus_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(prv_focus_key, app.current_theme.help_key_style),
        Span::styled(
            " to change focus. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" to submit.", app.current_theme.help_text_style),
    ];

    let help_paragraph = Paragraph::new(Line::from(help_spans))
        .style(app.current_theme.general_style)
        .block(Block::default())
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let separator = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(separator_style);

    let form_email_id_text = app.state.app_form_states.login.0[0].clone();
    let email_id_text = if app.state.app_form_states.login.0[0].is_empty() {
        "Email ID"
    } else {
        &form_email_id_text
    };

    let form_password_text = app.state.app_form_states.login.0[1].clone();
    let mut hidden_password = String::new();
    for _ in 0..app.state.app_form_states.login.0[1].len() {
        hidden_password.push(HIDDEN_PASSWORD_SYMBOL);
    }
    let password_text = if app.state.app_form_states.login.0[1].is_empty() {
        "Password"
    } else if app.state.app_form_states.login.1 {
        &form_password_text
    } else {
        hidden_password.as_str()
    };

    let email_id_paragraph = Paragraph::new(email_id_text)
        .style(email_id_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let password_paragraph = Paragraph::new(password_text)
        .style(password_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let show_password_paragraph = Paragraph::new("Show Password")
        .style(show_password_style)
        .block(Block::default())
        .alignment(Alignment::Right);

    let show_password_checkbox_value = if app.state.app_form_states.login.1 {
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

    render_title(app, &main_chunks[0], rect);
    rect.render_widget(crab_paragraph, chunks[0]);
    rect.render_widget(Clear, info_box);
    render_blank_styled_canvas_with_margin(rect, app, info_box, app.state.popup_mode.is_some(), -1);
    rect.render_widget(info_border, info_box);
    rect.render_widget(info_paragraph, info_chunks[0]);
    rect.render_widget(help_paragraph, info_chunks[2]);
    rect.render_widget(separator, chunks[1]);
    rect.render_widget(show_password_paragraph, show_password_chunks[0]);
    rect.render_widget(show_password_checkbox_paragraph, show_password_chunks[1]);
    rect.render_widget(submit_button, submit_button_chunks[1]);
    rect.render_widget(email_id_paragraph, email_id_field_chunk);
    rect.render_widget(password_paragraph, password_field_chunk);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }

    if app.state.user_login_data.auth_token.is_some() {
        let email_id = if app.state.user_login_data.email_id.is_some() {
            app.state.user_login_data.email_id.clone().unwrap()
        } else {
            "Unknown".to_string()
        };
        let already_logged_in_indicator =
            Paragraph::new(format!("Already logged in! {}", email_id))
                .style(app.current_theme.general_style)
                .block(Block::default())
                .alignment(Alignment::Center);

        rect.render_widget(already_logged_in_indicator, form_chunks[0]);
    }

    if app.state.app_status == AppStatus::UserInput {
        if app.state.focus == Focus::EmailIDField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.login.0[0],
                        email_id_field_chunk.width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    email_id_field_chunk,
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    email_id_field_chunk.x + 1 + app.state.app_form_states.login.0[0].len() as u16,
                    email_id_field_chunk.y + 1,
                );
            }
        } else if app.state.focus == Focus::PasswordField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.login.0[1],
                        password_field_chunk.width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    password_field_chunk,
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    password_field_chunk.x + 1 + app.state.app_form_states.login.0[1].len() as u16,
                    password_field_chunk.y + 1,
                );
            }
        }
    }
}

pub fn render_signup(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    if app.state.popup_mode.is_none() {
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
        popup_mode,
        true,
    );

    let password_field_style = get_mouse_focusable_field_style(
        app,
        Focus::PasswordField,
        &form_chunks[2],
        popup_mode,
        true,
    );

    let confirm_password_field_style = get_mouse_focusable_field_style(
        app,
        Focus::ConfirmPasswordField,
        &form_chunks[3],
        popup_mode,
        true,
    );

    let show_password_checkbox_style = get_mouse_focusable_field_style(
        app,
        Focus::ExtraFocus,
        &show_password_chunks[1],
        popup_mode,
        false,
    );

    let submit_button_style = get_mouse_focusable_field_style(
        app,
        Focus::SubmitButton,
        &submit_button_chunks[1],
        popup_mode,
        false,
    );

    let separator_style = check_for_popup_and_get_style(app, app.current_theme.general_style);

    let crab_paragraph = if app.state.popup_mode.is_some() {
        draw_crab_pattern(
            chunks[0],
            app.current_theme.inactive_text_style,
            true,
            app.config.disable_animations,
        )
    } else {
        draw_crab_pattern(
            chunks[0],
            app.current_theme.general_style,
            false,
            app.config.disable_animations,
        )
    };

    let info_border = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(separator_style);

    let info_paragraph = Paragraph::new("Sign Up")
        .style(app.current_theme.general_style)
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
        Span::styled("Press ", app.current_theme.help_text_style),
        Span::styled(next_focus_key, app.current_theme.help_key_style),
        Span::styled(" or ", app.current_theme.help_text_style),
        Span::styled(prv_focus_key, app.current_theme.help_key_style),
        Span::styled(
            " to change focus. Press ",
            app.current_theme.help_text_style,
        ),
        Span::styled(accept_key, app.current_theme.help_key_style),
        Span::styled(" to submit.", app.current_theme.help_text_style),
    ];

    let help_paragraph = Paragraph::new(Line::from(help_spans))
        .style(app.current_theme.general_style)
        .block(Block::default())
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    let separator = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(separator_style);

    let form_email_id_text = app.state.app_form_states.signup.0[0].clone();
    let email_id_text = if app.state.app_form_states.signup.0[0].is_empty() {
        "Email ID"
    } else {
        &form_email_id_text
    };

    let form_password_text = app.state.app_form_states.signup.0[1].clone();
    let mut hidden_password = String::new();
    for _ in 0..app.state.app_form_states.signup.0[1].len() {
        hidden_password.push(HIDDEN_PASSWORD_SYMBOL);
    }
    let password_text = if app.state.app_form_states.signup.0[1].is_empty() {
        "Password"
    } else if app.state.app_form_states.signup.1 {
        &form_password_text
    } else {
        hidden_password.as_str()
    };

    let form_confirm_password_text = app.state.app_form_states.signup.0[2].clone();
    let mut hidden_confirm_password = String::new();
    for _ in 0..app.state.app_form_states.signup.0[2].len() {
        hidden_confirm_password.push(HIDDEN_PASSWORD_SYMBOL);
    }
    let confirm_password_text = if app.state.app_form_states.signup.0[2].is_empty() {
        "Confirm Password"
    } else if app.state.app_form_states.signup.1 {
        &form_confirm_password_text
    } else {
        hidden_confirm_password.as_str()
    };

    let email_id_paragraph = Paragraph::new(email_id_text)
        .style(email_id_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let password_paragraph = Paragraph::new(password_text)
        .style(password_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let confirm_password_paragraph = Paragraph::new(confirm_password_text)
        .style(confirm_password_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let show_password_paragraph = Paragraph::new("Show Password")
        .style(separator_style)
        .block(Block::default())
        .alignment(Alignment::Right);

    let show_password_checkbox_value = if app.state.app_form_states.signup.1 {
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

    render_title(app, &main_chunks[0], rect);
    rect.render_widget(crab_paragraph, chunks[0]);
    rect.render_widget(Clear, info_box);
    render_blank_styled_canvas_with_margin(rect, app, info_box, app.state.popup_mode.is_some(), -1);
    rect.render_widget(info_border, info_box);
    rect.render_widget(info_paragraph, info_chunks[0]);
    rect.render_widget(help_paragraph, info_chunks[2]);
    rect.render_widget(separator, chunks[1]);
    rect.render_widget(email_id_paragraph, form_chunks[1]);
    rect.render_widget(password_paragraph, form_chunks[2]);
    rect.render_widget(confirm_password_paragraph, form_chunks[3]);
    rect.render_widget(show_password_paragraph, show_password_chunks[0]);
    rect.render_widget(show_password_checkbox_paragraph, show_password_chunks[1]);
    rect.render_widget(submit_button, submit_button_chunks[1]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }

    if app.state.app_status == AppStatus::UserInput {
        if app.state.focus == Focus::EmailIDField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.signup.0[0],
                        form_chunks[1].width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    form_chunks[1],
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    form_chunks[1].x + 1 + app.state.app_form_states.signup.0[0].len() as u16,
                    form_chunks[1].y + 1,
                );
            }
        } else if app.state.focus == Focus::PasswordField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.signup.0[1],
                        form_chunks[2].width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    form_chunks[2],
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    form_chunks[2].x + 1 + app.state.app_form_states.signup.0[1].len() as u16,
                    form_chunks[2].y + 1,
                );
            }
        } else if app.state.focus == Focus::ConfirmPasswordField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.signup.0[2],
                        form_chunks[3].width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    form_chunks[3],
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    form_chunks[3].x + 1 + app.state.app_form_states.signup.0[2].len() as u16,
                    form_chunks[3].y + 1,
                );
            }
        }
    }
}

pub fn render_reset_password(rect: &mut Frame, app: &mut App) {
    let popup_mode = app.state.popup_mode.is_some();
    if app.state.popup_mode.is_none() {
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
        popup_mode,
        true,
    );

    let send_reset_link_button_style = if popup_mode {
        app.current_theme.inactive_text_style
    } else if app.state.last_reset_password_link_sent_time.is_some() {
        let last_reset_password_link_sent_time =
            app.state.last_reset_password_link_sent_time.unwrap();
        if last_reset_password_link_sent_time.elapsed()
            < Duration::from_secs(MIN_TIME_BETWEEN_SENDING_RESET_LINK)
        {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(
            &app.state.current_mouse_coordinates,
            &send_reset_link_button_chunk,
        ) {
            if app.state.mouse_focus != Some(Focus::SendResetPasswordLinkButton) {
                app.state.current_cursor_position = None;
                app.state.app_status = AppStatus::Initialized;
            } else {
                app.state.app_status = AppStatus::UserInput;
            }
            app.state.mouse_focus = Some(Focus::SendResetPasswordLinkButton);
            app.state.focus = Focus::SendResetPasswordLinkButton;
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
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        } else {
            app.state.app_status = AppStatus::UserInput;
        }
        app.state.mouse_focus = Some(Focus::SendResetPasswordLinkButton);
        app.state.focus = Focus::SendResetPasswordLinkButton;
        app.current_theme.mouse_focus_style
    } else if app.state.focus == Focus::SendResetPasswordLinkButton {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    };

    let separator_style = check_for_popup_and_get_style(app, app.current_theme.general_style);

    let reset_link_field_style = get_mouse_focusable_field_style(
        app,
        Focus::ResetPasswordLinkField,
        &reset_link_chunk,
        popup_mode,
        true,
    );

    let new_password_field_style = get_mouse_focusable_field_style(
        app,
        Focus::PasswordField,
        &new_password_chunk,
        popup_mode,
        true,
    );

    let confirm_new_password_field_style = get_mouse_focusable_field_style(
        app,
        Focus::ConfirmPasswordField,
        &confirm_new_password_chunk,
        popup_mode,
        true,
    );

    let show_password_style = get_mouse_focusable_field_style(
        app,
        Focus::ExtraFocus,
        &show_password_main_chunk,
        popup_mode,
        false,
    );

    let submit_button_style = get_mouse_focusable_field_style(
        app,
        Focus::SubmitButton,
        &submit_button_chunks[1],
        popup_mode,
        false,
    );

    let crab_paragraph = if app.state.popup_mode.is_some() {
        draw_crab_pattern(
            chunks[0],
            app.current_theme.inactive_text_style,
            true,
            app.config.disable_animations,
        )
    } else {
        draw_crab_pattern(
            chunks[0],
            app.current_theme.general_style,
            false,
            app.config.disable_animations,
        )
    };

    let info_border = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(separator_style);

    let info_header = Paragraph::new("Reset Password")
        .style(app.current_theme.general_style)
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
            app.current_theme.help_text_style,
        )),
        Line::from(Span::styled(
            "2) Copy the reset link from your email and then paste the reset link.",
            app.current_theme.help_text_style,
        )),
        Line::from(Span::styled(
            "3) Enter new password and confirm the new password.",
            app.current_theme.help_text_style,
        )),
        Line::from(""),
        Line::from(Span::styled(
            "### Check Spam folder if you don't see the email ###",
            app.current_theme.help_text_style,
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", app.current_theme.help_text_style),
            Span::styled(next_focus_key, app.current_theme.help_key_style),
            Span::styled(" or ", app.current_theme.help_text_style),
            Span::styled(prv_focus_key, app.current_theme.help_key_style),
            Span::styled(
                " to change focus. Press ",
                app.current_theme.help_text_style,
            ),
            Span::styled(accept_key, app.current_theme.help_key_style),
            Span::styled(" to submit.", app.current_theme.help_text_style),
        ]),
    ];

    let help_paragraph = Paragraph::new(help_lines)
        .style(app.current_theme.general_style)
        .block(Block::default())
        .wrap(ratatui::widgets::Wrap { trim: true });

    let separator = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(separator_style);

    let form_email_id_text = app.state.app_form_states.reset_password.0[0].clone();
    let email_id_text = if app.state.app_form_states.reset_password.0[0].is_empty() {
        "Email ID"
    } else {
        &form_email_id_text
    };

    let send_reset_link_button_text = if app.state.last_reset_password_link_sent_time.is_some() {
        let last_reset_password_link_sent_time =
            app.state.last_reset_password_link_sent_time.unwrap();
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

    let reset_link_field_text = if app.state.app_form_states.reset_password.0[1].is_empty() {
        "Reset Link".to_string()
    } else {
        app.state.app_form_states.reset_password.0[1].clone()
    };
    let (windowed_reset_link, reset_link_cursor_pos_x) = get_sliding_window_over_text(
        &reset_link_field_text,
        app.state
            .current_cursor_position
            .unwrap_or(reset_link_field_text.len()),
        reset_link_chunk.width - 2,
    );

    let new_password_field_text = if app.state.app_form_states.reset_password.0[2].is_empty() {
        "New Password".to_string()
    } else if app.state.app_form_states.reset_password.1 {
        app.state.app_form_states.reset_password.0[2].clone()
    } else {
        let mut hidden_password = String::new();
        for _ in 0..app.state.app_form_states.reset_password.0[2].len() {
            hidden_password.push(HIDDEN_PASSWORD_SYMBOL);
        }
        hidden_password
    };

    let confirm_new_password_field_text =
        if app.state.app_form_states.reset_password.0[3].is_empty() {
            "Confirm New Password".to_string()
        } else if app.state.app_form_states.reset_password.1 {
            app.state.app_form_states.reset_password.0[3].clone()
        } else {
            let mut hidden_password = String::new();
            for _ in 0..app.state.app_form_states.reset_password.0[3].len() {
                hidden_password.push(HIDDEN_PASSWORD_SYMBOL);
            }
            hidden_password
        };

    let show_password_paragraph = Paragraph::new("Show Password")
        .style(show_password_style)
        .block(Block::default())
        .alignment(Alignment::Right);

    let show_password_checkbox_value = if app.state.app_form_states.reset_password.1 {
        "[X]"
    } else {
        "[ ]"
    };

    let email_id_paragraph = Paragraph::new(email_id_text)
        .style(email_id_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let send_reset_link_button = Paragraph::new(send_reset_link_button_text)
        .style(send_reset_link_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center);

    let reset_link_paragraph = Paragraph::new(windowed_reset_link)
        .style(reset_link_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let new_password_paragraph = Paragraph::new(new_password_field_text)
        .style(new_password_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

    let confirm_new_password_paragraph = Paragraph::new(confirm_new_password_field_text)
        .style(confirm_new_password_field_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

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

    render_title(app, &main_chunks[0], rect);
    rect.render_widget(crab_paragraph, chunks[0]);
    rect.render_widget(Clear, info_box);
    render_blank_styled_canvas_with_margin(rect, app, info_box, app.state.popup_mode.is_some(), -1);
    rect.render_widget(info_border, info_box);
    rect.render_widget(info_header, info_chunks[0]);
    rect.render_widget(help_paragraph, info_chunks[2]);
    rect.render_widget(separator, chunks[1]);
    rect.render_widget(email_id_paragraph, email_id_chunk);
    rect.render_widget(send_reset_link_button, send_reset_link_button_chunk);
    rect.render_widget(reset_link_paragraph, reset_link_chunk);
    rect.render_widget(new_password_paragraph, new_password_chunk);
    rect.render_widget(confirm_new_password_paragraph, confirm_new_password_chunk);
    rect.render_widget(show_password_paragraph, show_password_chunks[0]);
    rect.render_widget(show_password_checkbox_paragraph, show_password_chunks[1]);
    rect.render_widget(submit_button, submit_button_chunks[1]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }

    if app.state.app_status == AppStatus::UserInput {
        if app.state.focus == Focus::EmailIDField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.reset_password.0[0],
                        email_id_chunk.width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    email_id_chunk,
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    email_id_chunk.x
                        + 1
                        + app.state.app_form_states.reset_password.0[0].len() as u16,
                    email_id_chunk.y + 1,
                );
            }
        } else if app.state.focus == Focus::ResetPasswordLinkField {
            rect.set_cursor(
                reset_link_chunk.x + 1 + reset_link_cursor_pos_x,
                reset_link_chunk.y + 1,
            );
        } else if app.state.focus == Focus::PasswordField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.reset_password.0[2],
                        new_password_chunk.width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    new_password_chunk,
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    new_password_chunk.x
                        + 1
                        + app.state.app_form_states.reset_password.0[2].len() as u16,
                    new_password_chunk.y + 1,
                );
            }
        } else if app.state.focus == Focus::ConfirmPasswordField {
            if app.state.current_cursor_position.is_some() {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(
                        &app.state.app_form_states.reset_password.0[3],
                        confirm_new_password_chunk.width as usize - 2,
                    ),
                    app.state.current_cursor_position.unwrap_or(0),
                    confirm_new_password_chunk,
                );
                rect.set_cursor(x_pos, y_pos);
            } else {
                rect.set_cursor(
                    confirm_new_password_chunk.x
                        + 1
                        + app.state.app_form_states.reset_password.0[3].len() as u16,
                    confirm_new_password_chunk.y + 1,
                );
            }
        }
    }
}

pub fn render_load_cloud_save(rect: &mut Frame, app: &mut App) {
    let default_style = check_for_popup_and_get_style(app, app.current_theme.general_style);
    let help_key_style = check_for_popup_and_get_style(app, app.current_theme.help_key_style);
    let help_text_style = check_for_popup_and_get_style(app, app.current_theme.help_text_style);
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

    let item_list = &app.state.cloud_data;
    if item_list.is_some() {
        let item_list = item_list.as_ref().unwrap();
        if item_list.is_empty() {
            let no_saves_paragraph = Paragraph::new("No saves Found")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(app.current_theme.error_text_style);
            rect.render_widget(no_saves_paragraph, chunks[1]);
        } else {
            let items: Vec<ListItem> = item_list
                .iter()
                .map(|i| ListItem::new(format!("cloud_save_{}", i.save_id)))
                .collect();
            let choice_list = List::new(items)
                .block(
                    Block::default()
                        .title("Available Saves")
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(app.current_theme.list_select_style)
                .highlight_symbol(LIST_SELECTED_SYMBOL)
                .style(default_style);

            if !(app.state.popup_mode.is_some()
                && app.state.popup_mode.unwrap() == PopupMode::CommandPalette)
                && check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1])
            {
                app.state.mouse_focus = Some(Focus::LoadSave);
                app.state.focus = Focus::LoadSave;
                calculate_mouse_list_select_index(
                    app.state.current_mouse_coordinates.1,
                    item_list,
                    chunks[1],
                    &mut app.state.app_list_states.load_save,
                );
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
            .style(app.current_theme.error_text_style);
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
        .get_first_keybinding(KeyBindingEnum::GoToPreviousUIModeorCancel)
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
    } else if app.state.preview_boards_and_cards.is_none() {
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
        render_body(rect, preview_chunks[1], app, true)
    }

    let preview_title_paragraph = if app.state.preview_file_name.is_some() {
        Paragraph::new("Previewing: ".to_string() + &app.state.preview_file_name.clone().unwrap())
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
        render_close_button(rect, app);
    } else {
        rect.render_widget(preview_title_paragraph, preview_chunks[0]);
    }
}

fn draw_crab_pattern(
    render_area: Rect,
    style: Style,
    popup_mode: bool,
    disable_animations: bool,
) -> Paragraph<'static> {
    let crab_pattern = if popup_mode || disable_animations {
        create_crab_pattern_1(render_area.width, render_area.height, popup_mode)
    } else {
        let patterns = vec![
            create_crab_pattern_1(render_area.width, render_area.height, popup_mode),
            create_crab_pattern_2(render_area.width, render_area.height, popup_mode),
            create_crab_pattern_3(render_area.width, render_area.height, popup_mode),
        ];
        // get_time_offset() gives offset from unix epoch use this to give different patterns every 1000ms
        let index = (get_time_offset() / PATTERN_CHANGE_INTERVAL) as usize % patterns.len();
        patterns[index].clone()
    };
    Paragraph::new(crab_pattern)
        .style(style)
        .block(Block::default())
}

fn create_crab_pattern_1(width: u16, height: u16, popup_mode: bool) -> String {
    let mut pattern = String::new();
    for row in 0..height {
        for col in 0..width {
            if (row + col) % 2 == 0 {
                if popup_mode {
                    pattern.push_str(HIDDEN_PASSWORD_SYMBOL.to_string().as_str());
                } else {
                    pattern.push('🦀');
                }
            } else {
                pattern.push_str("  ");
            }
        }
        pattern.push('\n');
    }
    pattern
}

fn create_crab_pattern_2(width: u16, height: u16, popup_mode: bool) -> String {
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
                if popup_mode {
                    pattern.push_str(HIDDEN_PASSWORD_SYMBOL.to_string().as_str());
                } else {
                    pattern.push_str(" 🦀 ");
                }
            } else {
                pattern.push_str("   ");
            }
        }
        pattern.push('\n');
    }
    pattern
}

fn create_crab_pattern_3(width: u16, height: u16, popup_mode: bool) -> String {
    let mut pattern = String::new();
    for row in 0..height {
        for col in 0..width {
            if (row % 2 == 0 && col % 2 == 0) || (row % 2 == 1 && col % 2 == 1) {
                if popup_mode {
                    pattern.push_str(HIDDEN_PASSWORD_SYMBOL.to_string().as_str());
                } else {
                    pattern.push_str(" 🦀 ");
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

fn get_sliding_window_over_text(
    text: &str,
    cursor_pos: usize,
    display_box_width: u16,
) -> (&str, u16) {
    let text_width = text.chars().count() as u16;
    if text_width <= display_box_width {
        return (text, cursor_pos as u16);
    }
    let mut start_pos = 0;
    let mut end_pos = text_width;
    if cursor_pos < display_box_width as usize / 2 {
        end_pos = display_box_width;
    } else if cursor_pos > text_width as usize - display_box_width as usize / 2 {
        start_pos = text_width - display_box_width;
    } else {
        start_pos = cursor_pos as u16 - display_box_width / 2;
        end_pos = cursor_pos as u16 + display_box_width / 2;
    }
    let text = &text[start_pos as usize..end_pos as usize];
    (text, cursor_pos as u16 - start_pos)
}

/// Returns the style for the field based on the current focus and mouse position and sets the focus if the mouse is in the field area
fn get_mouse_focusable_field_style(
    app: &mut App,
    focus: Focus,
    chunk: &Rect,
    popup_mode: bool,
    user_input_mode: bool,
) -> Style {
    if popup_mode {
        app.current_theme.inactive_text_style
    } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, chunk) {
        if app.state.mouse_focus != Some(focus) {
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        } else if user_input_mode {
            app.state.app_status = AppStatus::UserInput;
        } else {
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(focus);
        app.state.focus = focus;
        app.current_theme.mouse_focus_style
    } else if app.state.focus == focus {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    }
}

/// Checks for popup mode to return inactive style if not returns the style passed
fn check_for_popup_and_get_style(app: &App, style: Style) -> Style {
    if app.state.popup_mode.is_some() {
        app.current_theme.inactive_text_style
    } else {
        style
    }
}

fn get_button_style_with_default_error_style(
    app: &mut App,
    focus: Focus,
    chunk: &Rect,
    popup_mode: bool,
) -> Style {
    if popup_mode {
        app.current_theme.inactive_text_style
    } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, chunk) {
        app.state.mouse_focus = Some(focus);
        app.state.focus = focus;
        app.current_theme.mouse_focus_style
    } else if app.state.focus == focus {
        app.current_theme.error_text_style
    } else {
        app.current_theme.general_style
    }
}

fn get_mouse_focusable_field_style_with_vertical_list_selection<T>(
    app: &mut App<'_>,
    main_menu_items: &[T],
    render_area: Rect,
    popup_mode: bool,
) -> Style {
    let mouse_coordinates = app.state.current_mouse_coordinates;

    if popup_mode {
        app.current_theme.inactive_text_style
    } else if check_if_mouse_is_in_area(&mouse_coordinates, &render_area) {
        app.state.mouse_focus = Some(Focus::MainMenu);
        app.state.focus = Focus::MainMenu;
        calculate_mouse_list_select_index(
            mouse_coordinates.1,
            main_menu_items,
            render_area,
            &mut app.state.app_list_states.main_menu,
        );
        app.current_theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::MainMenu) {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    }
}

fn calculate_mouse_list_select_index<T>(
    mouse_y: u16,
    list_to_check_against: &[T],
    render_area: Rect,
    list_state: &mut ListState,
) {
    let top_of_list = render_area.top() + 1;
    let mut bottom_of_list = top_of_list + list_to_check_against.len() as u16;
    if bottom_of_list > render_area.bottom() {
        bottom_of_list = render_area.bottom();
    }
    if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
        list_state.select(Some((mouse_y - top_of_list) as usize));
    }
}

fn get_scrollable_widget_row_bounds(
    all_rows_len: usize,
    selected_index: usize,
    offset: usize,
    max_height: usize,
) -> (usize, usize) {
    let offset = offset.min(all_rows_len.saturating_sub(1));
    let mut start = offset;
    let mut end = offset;
    let mut height = 0;
    for _ in (0..all_rows_len)
        .collect::<std::vec::Vec<usize>>()
        .iter()
        .skip(offset)
    {
        if height + 1 > max_height {
            break;
        }
        height += 1;
        end += 1;
    }

    while selected_index >= end {
        height = height.saturating_add(1);
        end += 1;
        while height > max_height {
            height = height.saturating_sub(1);
            start += 1;
        }
    }
    while selected_index < start {
        start -= 1;
        height = height.saturating_add(1);
        while height > max_height {
            end -= 1;
            height = height.saturating_sub(1);
        }
    }
    (start, end - 1)
}
