use chrono::{Local, NaiveDateTime};
use log::debug;
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Table, Wrap,
    },
    Frame,
};
use std::cmp::Ordering;
use tui_logger::TuiLoggerWidget;

use crate::{
    app::{
        kanban::{CardPriority, CardStatus},
        state::{AppStatus, Focus, UiMode},
        App, AppConfig, MainMenu, PopupMode,
    },
    calculate_cursor_position,
    constants::{
        APP_TITLE, DEFAULT_BOARD_TITLE_LENGTH, DEFAULT_CARD_TITLE_LENGTH, DEFAULT_DATE_FORMAT,
        FIELD_NOT_SET, LIST_SELECTED_SYMBOL, MAX_TOASTS_TO_DISPLAY, MIN_TERM_HEIGHT,
        MIN_TERM_WIDTH, SCREEN_TO_TOAST_WIDTH_RATIO, SPINNER_FRAMES, VERTICAL_SCROLL_BAR_SYMBOL,
    },
    io::data_handler::{get_available_local_savefiles, get_config},
};

use super::{
    widgets::{ToastType, ToastWidget},
    TextColorOptions, TextModifierOptions,
};

/// Draws main screen with kanban boards
pub fn render_zen_mode<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(rect.size());

    render_body(rect, chunks[0], app, false);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_title_body<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Percentage(80)].as_ref())
        .split(rect.size());

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    render_body(rect, chunks[1], app, false);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_body_help<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(85), Constraint::Length(5)].as_ref())
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[1]);

    render_body(rect, chunks[0], app, false);

    let help = draw_help(app, chunks[1]);
    let help_separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(default_style);
    rect.render_widget(help.0, chunks[1]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.help_state);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.help_state);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_body_log<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Length(8)].as_ref())
        .split(rect.size());

    render_body(rect, chunks[0], app, false);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[1]);
    rect.render_widget(log, chunks[1]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_title_body_help<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(75),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[2]);

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    render_body(rect, chunks[1], app, false);

    let help = draw_help(app, chunks[2]);
    let help_separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(default_style);
    rect.render_widget(help.0, chunks[2]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.help_state);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.help_state);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_title_body_log<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(75),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    render_body(rect, chunks[1], app, false);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[2]);
    rect.render_widget(log, chunks[2]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_body_help_log<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Length(5),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[1]);

    render_body(rect, chunks[0], app, false);

    let help = draw_help(app, chunks[1]);
    let help_separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(default_style);
    rect.render_widget(help.0, chunks[1]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.help_state);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.help_state);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[2]);
    rect.render_widget(log, chunks[2]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_title_body_help_log<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(5),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[2]);

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    render_body(rect, chunks[1], app, false);

    let help = draw_help(app, chunks[2]);
    let help_separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(default_style);
    rect.render_widget(help.0, chunks[2]);
    rect.render_stateful_widget(help.1, help_chunks[0], &mut app.state.help_state);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help.2, help_chunks[2], &mut app.state.help_state);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[3]);
    rect.render_widget(log, chunks[3]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_config<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_mode = app.state.popup_mode.is_some();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let reset_btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[2]);

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    let config_table = draw_config_table_selector(app, chunks[1]);
    rect.render_stateful_widget(config_table, chunks[1], &mut app.state.config_state);

    let reset_both_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, reset_btn_chunks[0]) {
        app.state.mouse_focus = Some(Focus::SubmitButton);
        app.state.focus = Focus::SubmitButton;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::SubmitButton) {
        app.theme.error_text_style
    } else {
        app.theme.general_style
    };
    let reset_config_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, reset_btn_chunks[1]) {
        app.state.mouse_focus = Some(Focus::ExtraFocus);
        app.state.focus = Focus::ExtraFocus;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::ExtraFocus) {
        app.theme.error_text_style
    } else {
        app.theme.general_style
    };

    let reset_both_button = Paragraph::new("Reset Config and Keybinds to Default")
        .block(
            Block::default()
                .title("Reset")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(reset_both_style)
        .alignment(Alignment::Center);
    rect.render_widget(reset_both_button, reset_btn_chunks[0]);

    let reset_config_button = Paragraph::new("Reset Only Config to Default")
        .block(
            Block::default()
                .title("Reset")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(reset_config_style)
        .alignment(Alignment::Center);
    rect.render_widget(reset_config_button, reset_btn_chunks[1]);

    let config_help = draw_config_help(&app.state.focus, popup_mode, app);
    rect.render_widget(config_help, chunks[3]);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[4]);
    rect.render_widget(log, chunks[4]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

/// Draws config list selector
fn draw_config_table_selector(app: &mut App, render_area: Rect) -> Table<'static> {
    let popup_mode = app.state.popup_mode.is_some();
    let mouse_coordinates = app.state.current_mouse_coordinates;
    let focus = app.state.focus;
    let config_list = get_config_items();
    let default_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(mouse_coordinates, render_area) {
        app.state.mouse_focus = Some(Focus::ConfigTable);
        app.state.focus = Focus::ConfigTable;
        let top_of_list = render_area.top() + 1;
        let mut bottom_of_list = top_of_list + config_list.len() as u16;
        if bottom_of_list > render_area.bottom() {
            bottom_of_list = render_area.bottom() - 1;
        }
        let mouse_y = mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .config_state
                .select(Some((mouse_y - top_of_list) as usize));
        }
        app.theme.mouse_focus_style
    } else if focus == Focus::ConfigTable {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };

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

    let config_text_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };

    let current_element_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        let mouse_row = mouse_coordinates.1 as usize;
        let current_selected_row = app.state.config_state.selected().unwrap_or(0);
        let current_selected_row_in_terminal_area =
            current_selected_row + render_area.y as usize + 1; // +1 for border
        if mouse_row == current_selected_row_in_terminal_area || focus == Focus::ConfigTable {
            app.theme.list_select_style
        } else {
            app.theme.general_style
        }
    };

    Table::new(rows)
        .block(
            Block::default()
                .title("Config Editor")
                .borders(Borders::ALL)
                .style(config_text_style)
                .border_style(default_style)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(current_element_style)
        .highlight_symbol(">> ")
        .widths(&[Constraint::Percentage(40), Constraint::Percentage(60)])
}

/// returns a list of all config items as a vector of strings
pub fn get_config_items() -> Vec<Vec<String>> {
    let get_config_status = get_config(false);
    let config = if let Ok(config) = get_config_status {
        config
    } else {
        debug!("Error getting config: {}", get_config_status.unwrap_err());
        AppConfig::default()
    };
    config.to_list()
}

pub fn render_edit_config<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let area = centered_rect(70, 70, rect.size());
    let clear_area = centered_rect(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Config Editor")
        .style(app.theme.general_style)
        .borders(Borders::ALL)
        .border_style(app.theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);
    rect.render_widget(Clear, clear_area);
    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);

    let chunks = if app.config.enable_mouse_support {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(6),
                    Constraint::Min(6),
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
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                    Constraint::Length(4),
                ]
                .as_ref(),
            )
            .split(area)
    };

    let edit_box_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
            app.state.mouse_focus = Some(Focus::EditGeneralConfigPopup);
            app.state.focus = Focus::EditGeneralConfigPopup;
            app.theme.mouse_focus_style
        } else if app.state.app_status == AppStatus::UserInput {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };

    let config_item_index = &app.config_item_being_edited;
    let list_items = get_config_items();
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
    } else {
        &app.state.theme_being_edited.name
    };
    let paragraph_text = format!("Current Value is {}\n\n{}",config_item_value,
        "Press 'i' to edit, or 'Esc' to cancel, Press 'Ins' to stop editing and press 'Enter' on Submit to save");
    let paragraph_title = Spans::from(vec![Span::raw(config_item_name)]);
    let config_item = Paragraph::new(paragraph_text)
        .block(
            Block::default()
                .title(paragraph_title)
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });
    let edit_item = Paragraph::new(app.state.current_user_input.clone())
        .block(
            Block::default()
                .title("Edit")
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_style(edit_box_style)
                .border_type(BorderType::Rounded),
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    let log = draw_logs(app, true, false, chunks[2]);

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
    rect.render_widget(config_item, chunks[0]);
    rect.render_widget(edit_item, chunks[1]);
    rect.render_widget(log, chunks[2]);

    if app.config.enable_mouse_support {
        let submit_button_style =
            if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[3]) {
                app.state.mouse_focus = Some(Focus::SubmitButton);
                app.state.focus = Focus::SubmitButton;
                app.theme.mouse_focus_style
            } else if app.state.app_status == AppStatus::KeyBindMode {
                app.theme.keyboard_focus_style
            } else {
                app.theme.general_style
            };
        let submit_button = Paragraph::new("Submit")
            .block(
                Block::default()
                    .style(app.theme.general_style)
                    .borders(Borders::ALL)
                    .border_style(submit_button_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        rect.render_widget(submit_button, chunks[3]);
        render_close_button(rect, app)
    }
}

pub fn render_select_default_view<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let render_area = centered_rect(70, 70, rect.size());
    let mouse_coordinates = app.state.current_mouse_coordinates;
    let clear_area = centered_rect(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Default HomeScreen Editor")
        .style(app.theme.general_style)
        .borders(Borders::ALL)
        .border_style(app.theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);
    rect.render_widget(Clear, clear_area);
    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(5)].as_ref())
        .split(render_area);

    let list_items = UiMode::all();
    let list_items: Vec<ListItem> = list_items
        .iter()
        .map(|s| ListItem::new(s.to_string()))
        .collect();

    if check_if_mouse_is_in_area(mouse_coordinates, render_area) {
        app.state.mouse_focus = Some(Focus::SelectDefaultView);
        app.state.focus = Focus::SelectDefaultView;
        let top_of_list = render_area.top() + 1;
        let mut bottom_of_list = top_of_list + list_items.len() as u16;
        if bottom_of_list > render_area.bottom() {
            bottom_of_list = render_area.bottom();
        }
        let mouse_y = mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .default_view_state
                .select(Some((mouse_y - top_of_list) as usize));
        }
    }

    let default_view_list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    let up_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go up")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let down_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go down")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let help_spans = Spans::from(vec![
        Span::styled("Use ", app.theme.general_style),
        Span::styled(up_key, app.theme.help_key_style),
        Span::styled(" and ", app.theme.general_style),
        Span::styled(down_key, app.theme.help_key_style),
        Span::styled("to navigate", app.theme.general_style),
        Span::raw("; "),
        Span::raw("Press "),
        Span::styled("<Enter>", app.theme.help_key_style),
        Span::raw(" To select a Default View; Press "),
        Span::styled("<Esc>", app.theme.help_key_style),
        Span::raw(" to cancel"),
    ]);

    let config_help = Paragraph::new(help_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(app.theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });

    rect.render_stateful_widget(
        default_view_list,
        chunks[0],
        &mut app.state.default_view_state,
    );
    rect.render_widget(config_help, chunks[1]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_edit_keybindings<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_mode = app.state.popup_mode.is_some();
    let chunks = Layout::default()
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect.size());
    let table_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(95), Constraint::Length(1)].as_ref())
        .split(chunks[1]);
    let default_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let progress_bar_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.progress_bar_style
    };
    let reset_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[3])
        && app.state.popup_mode.is_none()
    {
        app.state.mouse_focus = Some(Focus::SubmitButton);
        app.state.focus = Focus::SubmitButton;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::SubmitButton) {
        app.theme.error_text_style
    } else {
        app.theme.general_style
    };
    let current_element_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.list_select_style
    };

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    let mut table_items: Vec<Vec<String>> = Vec::new();
    // app.config.keybindings
    let keybindings = app.config.keybindings.clone();
    for (key, value) in keybindings.iter() {
        let mut row: Vec<String> = Vec::new();
        row.push(key.to_string());
        let mut row_value = String::new();
        for v in value.iter() {
            row_value.push_str(&v.to_string());
            row_value.push(' ');
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

    // draw a progress bar based on the number of items being displayed as not all rows will fit on the screen, calculate the percentage of rows that are visible
    let current_index = app.state.edit_keybindings_state.selected().unwrap_or(0);
    let total_rows = table_items.len();
    let visible_rows = (table_chunks[1].height - 1) as usize;
    let percentage = ((current_index + 1) as f32 / total_rows as f32) * 100.0;
    let blocks_to_render = (percentage / 100.0 * visible_rows as f32) as usize;

    // render blocks VERTICAL_SCROLL_BAR_SYMBOL
    for i in 0..blocks_to_render {
        let block_x = table_chunks[1].right() - 2;
        let block_y = table_chunks[1].top() + i as u16;
        let block = Paragraph::new(VERTICAL_SCROLL_BAR_SYMBOL)
            .style(progress_bar_style)
            .block(Block::default().borders(Borders::NONE));
        rect.render_widget(block, Rect::new(block_x, block_y, 1, 1));
    }

    let table_border_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1])
        && app.state.popup_mode.is_none()
    {
        app.state.mouse_focus = Some(Focus::EditKeybindingsTable);
        app.state.focus = Focus::EditKeybindingsTable;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::EditKeybindingsTable) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let help_key_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.help_key_style
    };

    let t = Table::new(rows)
        .block(
            Block::default()
                .title("Edit Keybindings")
                .style(default_style)
                .border_style(table_border_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(current_element_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Length(30),
            Constraint::Min(10),
        ]);

    let next_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus next")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let prev_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus previous")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let up_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go up")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let down_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go down")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let edit_keybind_help_spans = Spans::from(vec![
        Span::styled("Use ", app.theme.help_text_style),
        Span::styled(up_key, help_key_style),
        Span::styled("and ", app.theme.help_text_style),
        Span::styled(down_key, help_key_style),
        Span::styled("or scroll with the mouse", app.theme.help_text_style),
        Span::styled(" to select a keybinding, ", app.theme.help_text_style),
        Span::styled("<Enter>", help_key_style),
        Span::styled(" or ", app.theme.help_text_style),
        Span::styled("<Mouse Left Click>", help_key_style),
        Span::styled(" to edit, ", app.theme.help_text_style),
        Span::styled("<Esc>", help_key_style),
        Span::styled(
            " to cancel, To Reset Keybindings to Default, Press ",
            app.theme.help_text_style,
        ),
        Span::styled([next_focus_key, prev_focus_key].join("or "), help_key_style),
        Span::styled(
            "to highlight Reset Button and Press ",
            app.theme.help_text_style,
        ),
        Span::styled("<Enter>", help_key_style),
        Span::styled(
            " on the Reset Keybindings Button",
            app.theme.help_text_style,
        ),
    ]);

    let edit_keybind_help = Paragraph::new(edit_keybind_help_spans)
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

    rect.render_stateful_widget(t, chunks[1], &mut app.state.edit_keybindings_state);
    rect.render_widget(edit_keybind_help, chunks[2]);
    rect.render_widget(reset_button, chunks[3]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_edit_specific_keybinding<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let area = centered_rect(70, 70, rect.size());
    let clear_area = centered_rect(80, 80, rect.size());
    let clear_area_border = Block::default()
        .title("Edit Keybindings")
        .borders(Borders::ALL)
        .border_style(app.theme.keyboard_focus_style)
        .border_type(BorderType::Rounded);

    rect.render_widget(Clear, clear_area);
    render_blank_styled_canvas(rect, app, clear_area, false);
    rect.render_widget(clear_area_border, clear_area);
    let chunks = if app.config.enable_mouse_support {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(7),
                    Constraint::Min(6),
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
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                    Constraint::Length(4),
                ]
                .as_ref(),
            )
            .split(area)
    };

    let edit_box_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
            app.state.mouse_focus = Some(Focus::EditSpecificKeyBindingPopup);
            app.state.focus = Focus::EditSpecificKeyBindingPopup;
            app.theme.mouse_focus_style
        } else if app.state.app_status == AppStatus::KeyBindMode {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };

    let key_id = app.state.edit_keybindings_state.selected().unwrap_or(0);
    let current_bindings = app.config.keybindings.clone();
    let mut key_list = vec![];

    for (k, v) in current_bindings.iter() {
        key_list.push((k, v));
    }

    if key_id > key_list.len() {
        return;
    } else {
        let key = key_list[key_id].0;
        let value = key_list[key_id].1;
        let mut key_value = String::new();
        for v in value.iter() {
            key_value.push_str(&v.to_string());
            key_value.push(' ');
        }
        let paragraph_text = format!("Current Value is {}\n\n{}",key_value,
            "Press 'i' to edit, or 'Esc' to cancel, Press 'Ins' to stop editing and press 'Enter' on Submit to save");
        let paragraph_title = key.to_uppercase();
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

        let log = draw_logs(app, true, false, chunks[2]);

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
        rect.render_widget(config_item, chunks[0]);
        rect.render_widget(edit_item, chunks[1]);
        rect.render_widget(log, chunks[2]);
    }

    if app.config.enable_mouse_support {
        let submit_button_style =
            if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[3]) {
                app.state.mouse_focus = Some(Focus::SubmitButton);
                app.state.focus = Focus::SubmitButton;
                app.theme.mouse_focus_style
            } else if app.state.app_status == AppStatus::KeyBindMode {
                app.theme.keyboard_focus_style
            } else {
                app.theme.general_style
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

pub fn render_main_menu<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(16),
                Constraint::Min(8),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[2]);

    if app.config.enable_mouse_support {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(10), Constraint::Max(3)].as_ref())
            .split(chunks[0]);
        rect.render_widget(draw_title(app, new_chunks[0]), new_chunks[0]);
    } else {
        rect.render_widget(draw_title(app, chunks[0]), chunks[0]);
    };

    draw_main_menu(app, chunks[1], rect);

    let main_menu_help = draw_help(app, chunks[2]);
    let help_separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(default_style);
    rect.render_widget(main_menu_help.0, chunks[2]);
    rect.render_stateful_widget(main_menu_help.1, help_chunks[0], &mut app.state.help_state);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(main_menu_help.2, help_chunks[2], &mut app.state.help_state);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[3]);
    rect.render_widget(log, chunks[3]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_help_menu<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Length(4)].as_ref())
        .split(rect.size());

    let help_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(1),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunks[0]);

    let help_menu = draw_help(app, chunks[0]);
    let help_separator = Block::default()
        .borders(Borders::LEFT)
        .border_style(default_style);
    rect.render_widget(help_menu.0, chunks[0]);
    rect.render_stateful_widget(help_menu.1, help_chunks[0], &mut app.state.help_state);
    rect.render_widget(help_separator, help_chunks[1]);
    rect.render_stateful_widget(help_menu.2, help_chunks[2], &mut app.state.help_state);

    let log = draw_logs(app, true, app.state.popup_mode.is_some(), chunks[1]);
    rect.render_widget(log, chunks[1]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_logs_only<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(rect.size());
    let log = draw_logs(app, false, app.state.popup_mode.is_some(), chunks[0]);
    rect.render_widget(log, chunks[0]);
    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

/// Draws Help section for normal mode
fn draw_help<'a>(app: &mut App, render_area: Rect) -> (Block<'a>, Table<'a>, Table<'a>) {
    let keybind_store = &app.state.keybind_store;
    let popup_mode = app.state.popup_mode.is_some();
    let mouse_coordinates = app.state.current_mouse_coordinates;
    let focus = &mut app.state.focus;
    let default_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let border_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(mouse_coordinates, render_area) {
        app.state.mouse_focus = Some(Focus::Help);
        app.state.focus = Focus::Help;
        app.theme.mouse_focus_style
    } else if *focus == Focus::Help || *focus == Focus::MainMenuHelp {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };

    let current_element_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.list_select_style
    };

    let rows = keybind_store.iter().map(|item| {
        let height = item
            .iter()
            .map(|content| content.chars().filter(|c| *c == '\n').count())
            .max()
            .unwrap_or(0)
            + 1;
        let cells = item.iter().map(|c| Cell::from(c.to_string()));
        Row::new(cells).height(height as u16)
    });

    // split the rows into two tables
    let left_rows = rows.clone().take(rows.clone().count() / 2);
    let right_rows = rows.clone().skip(rows.clone().count() / 2);

    let left_table = Table::new(left_rows)
        .block(Block::default().style(default_style))
        .highlight_style(current_element_style)
        .highlight_symbol(">> ")
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
        .style(border_style);

    let right_table = Table::new(right_rows)
        .block(Block::default().style(default_style))
        .highlight_style(current_element_style)
        .highlight_symbol(">> ")
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
        .style(border_style);

    let border_block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(default_style)
        .border_style(border_style)
        .border_type(BorderType::Rounded);

    (border_block, left_table, right_table)
}

/// Draws help section for config mode
fn draw_config_help<'a>(focus: &'a Focus, popup_mode: bool, app: &'a App) -> Paragraph<'a> {
    let helpbox_style = if popup_mode {
        app.theme.inactive_text_style
    } else if matches!(focus, Focus::ConfigHelp) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let text_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let helpkey_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.help_key_style
    };

    let up_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go up")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let down_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go down")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let next_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus next")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let prev_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus previous")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let help_spans = Spans::from(vec![
        Span::styled("Use ", text_style),
        Span::styled(up_key, helpkey_style),
        Span::styled(" and ", text_style),
        Span::styled(down_key, helpkey_style),
        Span::styled("to navigate", text_style),
        Span::styled("; ", text_style),
        Span::styled("To edit a value, press ", text_style),
        Span::styled("<Enter>", helpkey_style),
        Span::styled("; Press ", text_style),
        Span::styled("<Esc>", helpkey_style),
        Span::styled(
            " to cancel, To Reset Keybindings to Default, Press ",
            text_style,
        ),
        Span::styled([next_focus_key, prev_focus_key].join(" or "), helpkey_style),
        Span::styled("to highlight Reset Button and Press ", text_style),
        Span::styled("<Enter>", helpkey_style),
        Span::styled(" on the Reset Keybindings Button", text_style),
    ]);

    Paragraph::new(help_spans)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(helpbox_style)
                .border_type(BorderType::Rounded),
        )
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true })
}

/// Draws logs
fn draw_logs<'a>(
    app: &mut App,
    enable_focus_highlight: bool,
    popup_mode: bool,
    render_area: Rect,
) -> TuiLoggerWidget<'a> {
    let focus = app.state.focus;
    let mouse_coordinates = app.state.current_mouse_coordinates;
    let logbox_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let logbox_border_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(mouse_coordinates, render_area) {
        app.state.mouse_focus = Some(Focus::Log);
        app.state.focus = Focus::Log;
        app.theme.mouse_focus_style
    } else if matches!(focus, Focus::Log) && enable_focus_highlight {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    if popup_mode {
        TuiLoggerWidget::default()
            .style_error(app.theme.inactive_text_style)
            .style_debug(app.theme.inactive_text_style)
            .style_warn(app.theme.inactive_text_style)
            .style_trace(app.theme.inactive_text_style)
            .style_info(app.theme.inactive_text_style)
            .output_file(false)
            .output_line(false)
            .output_target(false)
            .block(
                Block::default()
                    .title("Logs")
                    .style(logbox_style)
                    .border_style(logbox_border_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
    } else {
        TuiLoggerWidget::default()
            .style_error(app.theme.log_error_style)
            .style_debug(app.theme.log_debug_style)
            .style_warn(app.theme.log_warn_style)
            .style_trace(app.theme.log_trace_style)
            .style_info(app.theme.log_info_style)
            .output_file(false)
            .output_line(false)
            .output_target(false)
            .block(
                Block::default()
                    .title("Logs")
                    .style(logbox_style)
                    .border_style(logbox_border_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
    }
}

/// Draws Main menu
fn draw_main_menu<B>(app: &mut App, render_area: Rect, rect: &mut Frame<B>)
where
    B: Backend,
{
    let main_menu_items = MainMenu::all();
    let popup_mode = app.state.popup_mode.is_some();
    let focus = app.state.focus;
    let mouse_coordinates = app.state.current_mouse_coordinates;
    let menu_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(mouse_coordinates, render_area) {
        if !(app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette)
        {
            app.state.mouse_focus = Some(Focus::MainMenu);
            app.state.focus = Focus::MainMenu;
            // calculate the mouse_list_index based on the mouse coordinates and the length of the list
            let top_of_list = render_area.top() + 1;
            let mut bottom_of_list = top_of_list + main_menu_items.len() as u16;
            if bottom_of_list > render_area.bottom() {
                bottom_of_list = render_area.bottom();
            }
            let mouse_y = mouse_coordinates.1;
            if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
                app.state
                    .main_menu_state
                    .select(Some((mouse_y - top_of_list) as usize));
            }
        }
        app.theme.mouse_focus_style
    } else if matches!(focus, Focus::MainMenu) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let default_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let highlight_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.list_select_style
    };
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
    rect.render_stateful_widget(main_menu, render_area, &mut app.state.main_menu_state);
}

/// Draws Kanban boards
pub fn render_body<B>(rect: &mut Frame<B>, area: Rect, app: &mut App, preview_mode: bool)
where
    B: Backend,
{
    let fallback_boards = vec![];
    let focus = app.state.focus;
    let boards = if preview_mode {
        if app.state.preview_boards_and_cards.is_some() {
            app.state.preview_boards_and_cards.as_ref().unwrap()
        } else {
            &fallback_boards
        }
    } else {
        &app.boards
    };
    let progress_bar_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.progress_bar_style
    };
    let error_text_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.error_text_style
    };
    let current_board = &app.state.current_board_id.unwrap_or(0);

    let add_board_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Create new board")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    // check if any boards are present
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
                add_board_key,
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

    // make a list of constraints depending on NO_OF_BOARDS_PER_PAGE constant
    let chunks = if app.config.disable_scrollbars {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(99), Constraint::Length(1)].as_ref())
            .split(area)
    };
    let mut constraints = vec![];
    // check if length of boards is more than NO_OF_BOARDS_PER_PAGE
    if boards.len() > app.config.no_of_boards_to_show.into() {
        for _i in 0..app.config.no_of_boards_to_show {
            constraints.push(Constraint::Percentage(
                100 / app.config.no_of_boards_to_show,
            ));
        }
    } else {
        for _i in 0..boards.len() {
            constraints.push(Constraint::Percentage(100 / boards.len() as u16));
        }
    }
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.as_ref())
        .split(chunks[0]);
    // visible_boards_and_cards: Vec<LinkedHashMap<String, Vec<String>>>
    let visible_boards_and_cards = if preview_mode {
        app.state.preview_visible_boards_and_cards.clone()
    } else {
        app.visible_boards_and_cards.clone()
    };
    for (board_index, board_and_card_tuple) in visible_boards_and_cards.iter().enumerate() {
        // render board with title in board chunks alongside with cards in card chunks of the board
        // break if board_index is more than NO_OF_BOARDS_PER_PAGE
        if board_index >= app.config.no_of_boards_to_show.into() {
            break;
        }
        let board_id = board_and_card_tuple.0;
        // find index of board with board_id in boards
        let board = if preview_mode {
            app.state
                .preview_boards_and_cards
                .as_ref()
                .unwrap()
                .iter()
                .find(|&b| b.id == *board_id)
        } else {
            app.boards.iter().find(|&b| b.id == *board_id)
        };
        // check if board is found if not continue
        if board.is_none() {
            continue;
        }
        let board = board.unwrap();
        let board_title = board.name.clone();
        let board_cards = board_and_card_tuple.1;
        // if board title is longer than DEFAULT_BOARD_TITLE_LENGTH, truncate it and add ... at the end
        let board_title = if board_title.len() > DEFAULT_BOARD_TITLE_LENGTH.into() {
            format!(
                "{}...",
                &board_title[0..DEFAULT_BOARD_TITLE_LENGTH as usize]
            )
        } else {
            board_title
        };
        let board_title = format!("{} ({})", board_title, board.cards.len());
        let board_title = if board_id == current_board {
            format!("{} {}", ">>", board_title)
        } else {
            board_title
        };

        // check if length of cards is more than NO_OF_CARDS_PER_BOARD constant
        let mut card_constraints = vec![];
        if board_cards.len() > app.config.no_of_cards_to_show.into() {
            for _i in 0..app.config.no_of_cards_to_show {
                card_constraints.push(Constraint::Percentage(90 / app.config.no_of_cards_to_show));
            }
        } else {
            for _i in 0..board_cards.len() {
                card_constraints.push(Constraint::Percentage(100 / board_cards.len() as u16));
            }
        }

        // check if board_index is >= board_chunks.len() if yes continue
        if board_index >= board_chunks.len() {
            continue;
        }

        let board_style = if app.state.popup_mode.is_some() {
            app.theme.inactive_text_style
        } else {
            app.theme.general_style
        };
        let board_border_style = if app.state.popup_mode.is_some() {
            app.theme.inactive_text_style
        } else if check_if_mouse_is_in_area(
            app.state.current_mouse_coordinates,
            board_chunks[board_index],
        ) {
            app.state.mouse_focus = Some(Focus::Body);
            app.state.focus = Focus::Body;
            app.state.current_board_id = Some(*board_id);
            app.theme.mouse_focus_style
        } else if *board_id == *current_board
            && matches!(focus, Focus::Body)
            && app.state.current_card_id.is_none()
        {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };

        let board_block = Block::default()
            .title(&*board_title)
            .borders(Borders::ALL)
            .style(board_style)
            .border_style(board_border_style)
            .border_type(BorderType::Rounded);
        rect.render_widget(board_block, board_chunks[board_index]);

        let card_area_chunks = if app.config.disable_scrollbars {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(board_chunks[board_index])
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(1), Constraint::Percentage(99)].as_ref())
                .split(board_chunks[board_index])
        };

        let card_chunks = if app.config.disable_scrollbars {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(card_constraints.as_ref())
                .split(card_area_chunks[0])
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(card_constraints.as_ref())
                .split(card_area_chunks[1])
        };

        if !app.config.disable_scrollbars {
            // calculate the current card scroll percentage
            // get the index of current card in board_cards
            let all_board_cards = boards
                .iter()
                .find(|&b| b.id == *board_id)
                .unwrap()
                .cards
                .clone();
            let current_card_index = all_board_cards
                .iter()
                .position(|c| c.id == app.state.current_card_id.unwrap_or(0));
            let cards_scroll_percentage =
                (current_card_index.unwrap_or(0) + 1) as f64 / all_board_cards.len() as f64;
            let cards_scroll_percentage = cards_scroll_percentage.clamp(0.0, 1.0);
            let available_height = if card_area_chunks[0].height >= 2 {
                (card_area_chunks[0].height - 2) as f64
            } else {
                0.0
            };
            // calculate number of blocks to render
            let blocks_to_render = (available_height * cards_scroll_percentage) as u16;
            // render blocks VERTICAL_SCROLL_BAR_SYMBOL
            if !all_board_cards.is_empty() {
                for i in 0..blocks_to_render {
                    let block = Paragraph::new(VERTICAL_SCROLL_BAR_SYMBOL)
                        .style(progress_bar_style)
                        .block(Block::default().borders(Borders::NONE));
                    rect.render_widget(
                        block,
                        Rect::new(
                            card_area_chunks[0].x,
                            card_area_chunks[0].y + i + 1,
                            card_area_chunks[0].width,
                            1,
                        ),
                    );
                }
            }
        };
        for (card_index, card_id) in board_cards.iter().enumerate() {
            if card_index >= app.config.no_of_cards_to_show.into() {
                break;
            }
            let inner_card_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
                .margin(1)
                .split(card_chunks[card_index]);
            // unwrap card if panic skip it and log it
            let card = board.get_card(*card_id);
            // check if card is None, if so skip it and log it
            if card.is_none() {
                continue;
            }
            let card = card.unwrap();

            let card_title = if card.name.len() > DEFAULT_CARD_TITLE_LENGTH.into() {
                format!("{}...", &card.name[0..DEFAULT_CARD_TITLE_LENGTH as usize])
            } else {
                card.name.clone()
            };
            let card_title = if app.state.current_card_id.unwrap_or(0) == *card_id {
                format!("{} {}", ">>", card_title)
            } else {
                card_title
            };

            let card_description = if card.description == FIELD_NOT_SET {
                "Description: Not Set".to_string()
            } else {
                card.description.clone()
            };

            let mut card_extra_info = vec![Spans::from("")];
            if card.date_due == FIELD_NOT_SET {
                if app.state.popup_mode.is_some() {
                    card_extra_info.push(Spans::from(Span::styled(
                        "Due: Not Set",
                        app.theme.inactive_text_style,
                    )))
                } else {
                    card_extra_info.push(Spans::from(Span::styled(
                        "Due: Not Set",
                        app.theme.card_due_default_style,
                    )))
                }
            } else {
                let card_due_date = card.date_due.clone();
                let parsed_due_date =
                    NaiveDateTime::parse_from_str(&card_due_date, "%Y/%m/%d-%H:%M:%S");
                // card due date is in the format dd/mm/yyyy check if the due date is within WARNING_DUE_DATE_DAYS if so highlight it
                let card_due_date_styled = if let Ok(parsed_due_date) = parsed_due_date {
                    let today = Local::now().naive_local();
                    let days_left = parsed_due_date.signed_duration_since(today).num_days();
                    let parsed_due_date = parsed_due_date.format(DEFAULT_DATE_FORMAT).to_string();
                    if app.state.popup_mode.is_some() {
                        Spans::from(Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.theme.inactive_text_style,
                        ))
                    } else if days_left >= 0 {
                        match days_left.cmp(&(app.config.warning_delta as i64)) {
                            Ordering::Less | Ordering::Equal => Spans::from(Span::styled(
                                format!("Due: {}", parsed_due_date),
                                app.theme.card_due_warning_style,
                            )),
                            Ordering::Greater => Spans::from(Span::styled(
                                format!("Due: {}", parsed_due_date),
                                app.theme.card_due_default_style,
                            )),
                        }
                    } else {
                        Spans::from(Span::styled(
                            format!("Due: {}", parsed_due_date),
                            app.theme.card_due_overdue_style,
                        ))
                    }
                } else if app.state.popup_mode.is_some() {
                    Spans::from(Span::styled(
                        format!("Due: {}", card_due_date),
                        app.theme.inactive_text_style,
                    ))
                } else {
                    Spans::from(Span::styled(
                        format!("Due: {}", card_due_date),
                        app.theme.card_due_default_style,
                    ))
                };
                card_extra_info.extend(vec![card_due_date_styled]);
            }

            let card_status = format!("Status: {}", card.card_status.clone());
            let card_status = if app.state.popup_mode.is_some() {
                Spans::from(Span::styled(card_status, app.theme.inactive_text_style))
            } else {
                match card.card_status {
                    CardStatus::Active => Spans::from(Span::styled(
                        card_status,
                        app.theme.card_status_active_style,
                    )),
                    CardStatus::Complete => Spans::from(Span::styled(
                        card_status,
                        app.theme.card_status_completed_style,
                    )),
                    CardStatus::Stale => {
                        Spans::from(Span::styled(card_status, app.theme.card_status_stale_style))
                    }
                }
            };
            card_extra_info.extend(vec![card_status]);

            // if card id is same as current_card, highlight it
            let card_style = if app.state.popup_mode.is_some() {
                app.theme.inactive_text_style
            } else if check_if_mouse_is_in_area(
                app.state.current_mouse_coordinates,
                card_chunks[card_index],
            ) {
                app.state.mouse_focus = Some(Focus::Body);
                app.state.focus = Focus::Body;
                app.state.current_card_id = Some(*card_id);
                app.theme.mouse_focus_style
            } else if app.state.current_card_id.unwrap_or(0) == *card_id
                && matches!(focus, Focus::Body)
                && *board_id == *current_board
            {
                app.theme.keyboard_focus_style
            } else {
                app.theme.general_style
            };
            let card_block = Block::default()
                .title(&*card_title)
                .borders(Borders::ALL)
                .border_style(card_style)
                .border_type(BorderType::Rounded);
            rect.render_widget(card_block, card_chunks[card_index]);
            let card_paragraph = Paragraph::new(card_description)
                .alignment(Alignment::Left)
                .block(Block::default())
                .wrap(ratatui::widgets::Wrap { trim: false });
            rect.render_widget(card_paragraph, inner_card_chunks[0]);
            let card_extra_info = Paragraph::new(card_extra_info)
                .alignment(Alignment::Left)
                .block(Block::default())
                .wrap(ratatui::widgets::Wrap { trim: false });
            rect.render_widget(card_extra_info, inner_card_chunks[1]);
        }
    }

    if !app.config.disable_scrollbars {
        // draw line_gauge in chunks[1]
        // get the index of the current board in boards and set percentage
        let current_board_id = app.state.current_board_id.unwrap_or(0);
        // get the index of the board with the id
        let current_board_index = boards
            .iter()
            .position(|board| board.id == current_board_id)
            .unwrap_or(0)
            + 1;
        let percentage = {
            // make sure percentage is not nan and is between 0 and 100
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
            .gauge_style(progress_bar_style)
            .percent(percentage);
        rect.render_widget(line_gauge, chunks[1]);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn top_left_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[0])[0]
}

/// Draws size error screen if the terminal is too small
pub fn draw_size_error<B>(rect: &mut Frame<B>, size: &Rect, msg: String, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(*size);

    let title = draw_title(app, *size);
    rect.render_widget(title, chunks[0]);

    let mut text = vec![Spans::from(Span::styled(msg, app.theme.error_text_style))];
    text.append(&mut vec![Spans::from(Span::raw(
        "Resize the window to continue, or press 'q' to quit.",
    ))]);
    let body = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    rect.render_widget(body, chunks[1]);
}

pub fn draw_loading_screen<B>(rect: &mut Frame<B>, size: &Rect, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(*size);

    let title = draw_title(app, *size);
    rect.render_widget(title, chunks[0]);

    let text = Spans::from(vec![
        Span::styled("Loading......", app.theme.keyboard_focus_style),
        Span::styled("`(*>﹏<*)′", app.theme.keyboard_focus_style),
        Span::styled("Please wait", app.theme.keyboard_focus_style),
    ]);
    let body = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    rect.render_widget(body, chunks[1]);
}

/// Draws the title bar
pub fn draw_title<'a>(app: &mut App, render_area: Rect) -> Paragraph<'a> {
    let popup_mode = app.state.popup_mode.is_some();
    let mouse_coordinates = app.state.current_mouse_coordinates;
    let focus = app.state.focus;
    let title_style = if popup_mode {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let border_style = if popup_mode {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(mouse_coordinates, render_area) {
        app.state.mouse_focus = Some(Focus::Title);
        app.state.focus = Focus::Title;
        app.theme.mouse_focus_style
    } else if matches!(focus, Focus::Title) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    // check if focus is on title
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

/// Helper function to check terminal size
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

pub fn render_new_board_form<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    // make a form for the Board struct
    // take name and description where description is optional
    // submit button

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(8),
                Constraint::Length(4),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let name_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
        if app.state.mouse_focus != Some(Focus::NewBoardName) {
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(Focus::NewBoardName);
        app.state.focus = Focus::NewBoardName;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::NewBoardName) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let description_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[2]) {
        if app.state.mouse_focus != Some(Focus::NewBoardDescription) {
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(Focus::NewBoardDescription);
        app.state.focus = Focus::NewBoardDescription;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::NewBoardDescription) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let help_key_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.help_key_style
    };
    let submit_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[4]) {
        app.state.mouse_focus = Some(Focus::SubmitButton);
        app.state.focus = Focus::SubmitButton;
        app.state.current_cursor_position = None;
        app.state.app_status = AppStatus::Initialized;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::SubmitButton) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };

    let title_paragraph = Paragraph::new("Create a new Board")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(default_style),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let wrapped_title_text =
        textwrap::wrap(&app.state.new_board_form[0], (chunks[1].width - 2) as usize);
    let board_name_field = wrapped_title_text
        .iter()
        .map(|x| Spans::from(Span::raw(&**x)))
        .collect::<Vec<Spans>>();
    let wrapped_description_text =
        textwrap::wrap(&app.state.new_board_form[1], (chunks[2].width - 2) as usize);
    let board_description_field = wrapped_description_text
        .iter()
        .map(|x| Spans::from(Span::raw(&**x)))
        .collect::<Vec<Spans>>();

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
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Enter input mode")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let next_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus next")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let prev_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus previous")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let help_text = Spans::from(vec![
        Span::styled("Press ", default_style),
        Span::styled(input_mode_key, help_key_style),
        Span::styled("to start typing", default_style),
        Span::styled("; ", default_style),
        Span::styled("<Ins>", help_key_style),
        Span::styled(" to stop typing", default_style),
        Span::styled("; ", default_style),
        Span::styled("Press ", default_style),
        Span::styled(
            [next_focus_key, prev_focus_key].join(" or "),
            help_key_style,
        ),
        Span::styled("to switch focus", default_style),
        Span::styled("; ", default_style),
        Span::styled("<Enter>", help_key_style),
        Span::styled(" to submit", default_style),
        Span::styled("; ", default_style),
        Span::styled("<Esc>", help_key_style),
        Span::styled(" to cancel", default_style),
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
                    .unwrap_or_else(|| app.state.new_board_form[0].len()),
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
                    .unwrap_or_else(|| app.state.new_board_form[1].len()),
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

pub fn render_new_card_form<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(8),
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let name_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
        if app.state.mouse_focus != Some(Focus::NewCardName) {
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(Focus::NewCardName);
        app.state.focus = Focus::NewCardName;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::NewCardName) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let description_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[2]) {
        if app.state.mouse_focus != Some(Focus::CardDescription) {
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(Focus::CardDescription);
        app.state.focus = Focus::CardDescription;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::CardDescription) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let due_date_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[3]) {
        if app.state.mouse_focus != Some(Focus::CardDueDate) {
            app.state.current_cursor_position = None;
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(Focus::CardDueDate);
        app.state.focus = Focus::CardDueDate;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::CardDueDate) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let help_key_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.help_key_style
    };
    let submit_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[5]) {
        app.state.mouse_focus = Some(Focus::SubmitButton);
        app.state.focus = Focus::SubmitButton;
        app.state.current_cursor_position = None;
        app.state.app_status = AppStatus::Initialized;
        app.theme.mouse_focus_style
    } else if matches!(app.state.focus, Focus::SubmitButton) {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };

    let title_paragraph = Paragraph::new("Create a new Card")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(default_style),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let wrapped_card_name_text =
        textwrap::wrap(&app.state.new_card_form[0], (chunks[1].width - 2) as usize);
    let card_name_field = wrapped_card_name_text
        .iter()
        .map(|x| Spans::from(Span::raw(&**x)))
        .collect::<Vec<Spans>>();
    let wrapped_card_description_text =
        textwrap::wrap(&app.state.new_card_form[1], (chunks[2].width - 2) as usize);
    let card_description_field = wrapped_card_description_text
        .iter()
        .map(|x| Spans::from(Span::raw(&**x)))
        .collect::<Vec<Spans>>();
    let wrapped_card_due_date_text =
        textwrap::wrap(&app.state.new_card_form[2], (chunks[3].width - 2) as usize);
    let card_due_date_field = wrapped_card_due_date_text
        .iter()
        .map(|x| Spans::from(Span::raw(&**x)))
        .collect::<Vec<Spans>>();
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

    let card_description = Paragraph::new(card_description_field)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(description_style)
                .border_type(BorderType::Rounded)
                .title("Card Description"),
        );
    rect.render_widget(card_description, chunks[2]);

    // check if &app.state.new_card_form[2] is in the format (DD/MM/YYYY-HH:MM:SS) or (DD/MM/YYYY)
    let parsed_date =
        match NaiveDateTime::parse_from_str(&app.state.new_card_form[2], DEFAULT_DATE_FORMAT) {
            Ok(date) => Some(date),
            Err(_) => {
                match NaiveDateTime::parse_from_str(&app.state.new_card_form[2], "%d/%m/%Y") {
                    Ok(date) => Some(date),
                    Err(_) => None,
                }
            }
        };
    let card_due_date = Paragraph::new(card_due_date_field)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(due_date_style)
                .border_type(BorderType::Rounded)
                .title("Card Due Date (DD/MM/YYYY-HH:MM:SS) or (DD/MM/YYYY)"),
        );
    if parsed_date.is_some() {
        rect.render_widget(card_due_date, chunks[3]);
    } else if !app.state.new_card_form[2].is_empty() {
        let new_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Length(20)].as_ref())
            .split(chunks[3]);
        rect.render_widget(card_due_date, new_chunks[0]);
        let error_text = Spans::from(vec![Span::raw("Invalid date format")]);
        let error_paragraph = Paragraph::new(error_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.theme.error_text_style),
            );
        rect.render_widget(error_paragraph, new_chunks[1]);
    } else {
        rect.render_widget(card_due_date, chunks[3]);
    }

    let input_mode_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Enter input mode")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let next_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus next")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let prev_focus_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Focus previous")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let help_text = Spans::from(vec![
        Span::styled("Press ", default_style),
        Span::styled(input_mode_key, help_key_style),
        Span::styled("to start typing", default_style),
        Span::styled("; ", default_style),
        Span::styled("<Ins>", help_key_style),
        Span::styled(" to stop typing", default_style),
        Span::styled("; ", default_style),
        Span::styled("Press ", default_style),
        Span::styled(
            [next_focus_key, prev_focus_key].join(" or "),
            help_key_style,
        ),
        Span::styled("to switch focus", default_style),
        Span::styled("; ", default_style),
        Span::styled("<Enter>", help_key_style),
        Span::styled(" to submit", default_style),
        Span::styled("; ", default_style),
        Span::styled("<Esc>", help_key_style),
        Span::styled(" to cancel", default_style),
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
    rect.render_widget(help_paragraph, chunks[4]);

    let submit_button = Paragraph::new("Submit").alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .style(submit_style)
            .border_type(BorderType::Rounded),
    );
    rect.render_widget(submit_button, chunks[5]);

    if app.state.focus == Focus::NewCardName && app.state.app_status == AppStatus::UserInput {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_card_name_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.new_card_form[0].len()),
                chunks[1],
            );
            rect.set_cursor(x_pos, y_pos);
        } else {
            rect.set_cursor(chunks[1].x + 1, chunks[1].y + 1);
        }
    } else if app.state.focus == Focus::CardDescription
        && app.state.app_status == AppStatus::UserInput
    {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_card_description_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.new_card_form[1].len()),
                chunks[2],
            );
            rect.set_cursor(x_pos, y_pos);
        } else {
            rect.set_cursor(chunks[2].x + 1, chunks[2].y + 1);
        }
    } else if app.state.focus == Focus::CardDueDate && app.state.app_status == AppStatus::UserInput
    {
        if app.state.current_cursor_position.is_some() {
            let (x_pos, y_pos) = calculate_cursor_position(
                wrapped_card_due_date_text,
                app.state
                    .current_cursor_position
                    .unwrap_or_else(|| app.state.new_card_form[2].len()),
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

pub fn render_load_save<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let default_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.general_style
    };
    let help_key_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else {
        app.theme.help_key_style
    };
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
                Constraint::Min(10),
                Constraint::Length(9),
            ]
            .as_ref(),
        )
        .split(main_chunks[0]);

    let preview_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(main_chunks[1]);

    let title_bar_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(preview_chunks[0]);

    let title_paragraph = Paragraph::new("Load a Save")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .style(default_style);
    rect.render_widget(title_paragraph, chunks[0]);

    let item_list = get_available_local_savefiles();
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
            .style(app.theme.error_text_style);
        rect.render_widget(no_saves_paragraph, chunks[1]);
    } else {
        // make a list from the Vec<string> of savefiles
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
            .highlight_style(app.theme.list_select_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL)
            .style(default_style);

        if !(app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette)
            && check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1])
        {
            app.state.mouse_focus = Some(Focus::LoadSave);
            app.state.focus = Focus::LoadSave;
            let top_of_list = chunks[1].y + 1;
            let mut bottom_of_list = chunks[1].y + item_list.len() as u16;
            if bottom_of_list > chunks[1].bottom() {
                bottom_of_list = chunks[1].bottom();
            }
            let mouse_y = app.state.current_mouse_coordinates.1;
            if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
                app.state
                    .load_save_state
                    .select(Some((mouse_y - top_of_list) as usize));
            }
        }
        rect.render_stateful_widget(choice_list, chunks[1], &mut app.state.load_save_state);
    }

    let delete_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Delete focused element")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let up_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go up")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();
    let down_key = app
        .state
        .keybind_store
        .iter()
        .find(|x| x[1] == "Go down")
        .unwrap_or(&vec!["".to_string(), "".to_string()])[0]
        .clone();

    let help_text = Spans::from(vec![
        Span::styled("Use ", default_style),
        Span::styled(&up_key, help_key_style),
        Span::styled(" and ", default_style),
        Span::styled(&down_key, help_key_style),
        Span::styled("to navigate", default_style),
        Span::raw("; "),
        Span::styled("<Enter>", help_key_style),
        Span::styled(" to Load the save file", default_style),
        Span::raw("; "),
        Span::styled("<Esc>", help_key_style),
        Span::styled(" to cancel", default_style),
        Span::raw("; "),
        Span::styled(delete_key, help_key_style),
        Span::styled("to delete a save file", default_style),
        Span::styled(
            ". If using a mouse click on a save file to preview",
            default_style,
        ),
    ]);
    let help_paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: true })
        .style(default_style);
    rect.render_widget(help_paragraph, chunks[2]);

    // preview pane
    if app.state.load_save_state.selected().is_none() {
        let preview_paragraph =
            Paragraph::new(format!("Select a save file with {} or {} to preview or Click on a save file to preview if using a mouse", up_key, down_key))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .style(default_style)
                .wrap(Wrap { trim: true });
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
            .wrap(Wrap { trim: true });
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
            .wrap(Wrap { trim: true })
    } else {
        Paragraph::new("Select a file to preview")
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .style(default_style)
            .wrap(Wrap { trim: true })
    };

    if app.config.enable_mouse_support {
        rect.render_widget(preview_title_paragraph, title_bar_chunks[0]);
        render_close_button(rect, app);
    } else {
        rect.render_widget(preview_title_paragraph, preview_chunks[0]);
    }
}

pub fn render_toast<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    // get the latest MAX_TOASTS_TO_DISPLAY number of toasts from app.state.toasts
    let all_toasts = app.state.toasts.clone();
    let mut loading_toasts = all_toasts
        .iter()
        .filter(|x| x.toast_type == ToastType::Loading)
        .collect::<Vec<&ToastWidget>>();
    let app_toasts = app.state.toasts.clone();
    let toasts = if !loading_toasts.is_empty() {
        // if loading_toasts are > MAX_TOASTS_TO_DISPLAY then put the loading toasts in the order of start time where the oldest is at the top only put MAX_TOASTS_TO_DISPLAY - 1 loading toasts and put the latest regular toast at the bottom
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
        // append the latest regular toast to the loading toasts till length is MAX_TOASTS_TO_DISPLAY
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
        // check if any more loading toasts are there and if so then append them to the toasts if there is space
        if toasts.len() < MAX_TOASTS_TO_DISPLAY {
            let mut loading_toasts = all_toasts
                .iter()
                .filter(|x| x.toast_type == ToastType::Loading)
                .collect::<Vec<&ToastWidget>>();
            loading_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            while toasts.len() < MAX_TOASTS_TO_DISPLAY {
                if let Some(toast) = loading_toasts.pop() {
                    // check if the toast is already present in toasts
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
    let mut total_height_rendered = 1; // for messages indicator

    // loop through the toasts and draw them
    for toast in toasts.iter() {
        let toast_style = app.theme.general_style.fg(ratatui::style::Color::Rgb(
            toast.toast_color.0,
            toast.toast_color.1,
            toast.toast_color.2,
        ));
        let toast_title = match toast.toast_type {
            ToastType::Error => "Error",
            ToastType::Info => "Info",
            ToastType::Warning => "Warning",
            ToastType::Loading => "Loading",
        };
        // if the toast type is loading display a spinner next to the title and use the duration.elapsed() to determine the current frame of the spinner
        let toast_title = match toast.toast_type {
            ToastType::Loading => {
                let spinner_frames = &SPINNER_FRAMES;
                let frame =
                    (toast.start_time.elapsed().as_millis() / 100) % spinner_frames.len() as u128;
                let frame = frame as usize;
                format!("{} {}", spinner_frames[frame], toast_title)
            }
            _ => toast_title.to_string(),
        };
        let x_offset = rect.size().width - (rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO);
        let lines = textwrap::wrap(
            &toast.message,
            ((rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO) - 2) as usize,
        )
        .iter()
        .map(|x| Spans::from(x.to_string()))
        .collect::<Vec<Spans>>();
        let toast_height = lines.len() as u16 + 2;
        let toast_block = Block::default()
            .title(toast_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(toast_style);
        let toast_paragraph = Paragraph::new(lines)
            .block(toast_block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .style(toast_style);
        rect.render_widget(
            Clear,
            Rect::new(
                x_offset,
                total_height_rendered,
                rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO,
                toast_height,
            ),
        );
        render_blank_styled_canvas(
            rect,
            app,
            Rect::new(
                x_offset,
                total_height_rendered,
                rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO,
                toast_height,
            ),
            false,
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
    }

    // display a total count of toasts on the top right corner
    let text_offset = 15;
    let toast_count = app.state.toasts.len();
    let toast_count_text = format!(" {} Message(s)", toast_count);
    let toast_count_paragraph = Paragraph::new(toast_count_text)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_type(BorderType::Rounded),
        )
        .style(app.theme.general_style);
    let message_area = Rect::new(rect.size().width - text_offset, 0, text_offset, 1);
    rect.render_widget(Clear, message_area);
    render_blank_styled_canvas(rect, app, message_area, false);
    rect.render_widget(toast_count_paragraph, message_area);
}

pub fn render_view_card<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_area = centered_rect(90, 90, rect.size());
    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, false);
    let card_chunks = if app.card_being_edited.is_some() {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),
                Constraint::Length(16),
                Constraint::Length(3),
            ])
            .margin(1)
            .split(popup_area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(16)])
            .margin(1)
            .split(popup_area)
    };

    if app.state.focus == Focus::SubmitButton && app.card_being_edited.is_none() {
        app.state.focus = Focus::CardDescription;
    }

    if app.state.current_board_id.is_none() || app.state.current_card_id.is_none() {
        let no_board_or_card_selected = Paragraph::new("No board or card selected.")
            .block(
                Block::default()
                    .title("Card Info")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .style(app.theme.error_text_style),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
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
                    .style(app.theme.error_text_style),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
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
                    .style(app.theme.error_text_style),
            )
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        rect.render_widget(could_not_find_card, popup_area);
        return;
    }
    let card = if app.card_being_edited.is_some() {
        &app.card_being_edited.as_ref().unwrap().1
    } else {
        card.unwrap()
    };

    let board_name = board.name.clone();
    let card_name = card.name.clone();
    let card_description = card.description.clone();
    let wrapped_description =
        textwrap::wrap(&card_description, (card_chunks[0].width - 2) as usize);
    let wrapped_description_spans = wrapped_description
        .iter()
        .map(|x| Spans::from(Span::styled(&**x, app.theme.general_style)))
        .collect::<Vec<Spans>>();
    let main_block = Block::default()
        .title(format!("{} >> Board({})", card_name, board_name))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.theme.general_style);
    rect.render_widget(main_block, popup_area);

    let description_style = if app.state.focus == Focus::CardDescription {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };

    let description_paragraph = Paragraph::new(wrapped_description_spans).block(
        Block::default()
            .title("Description")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(description_style),
    );
    rect.render_widget(description_paragraph, card_chunks[0]);

    let card_date_created = Span::styled(
        format!("Created: {}", card.date_created),
        app.theme.general_style,
    );
    let card_date_modified = Span::styled(
        format!("Modified: {}", card.date_modified),
        app.theme.general_style,
    );
    let card_date_completed = Span::styled(
        format!("Completed: {}", card.date_completed),
        app.theme.general_style,
    );
    let card_priority = format!("Priority: {}", card.priority);
    let card_status = format!("Status: {}", card.card_status);
    let parsed_due_date = NaiveDateTime::parse_from_str(&card.date_due, DEFAULT_DATE_FORMAT);
    // card due date is in the format dd/mm/yyyy check if the due date is within WARNING_DUE_DATE_DAYS if so highlight it
    let card_due_date_styled = if let Ok(parsed_due_date) = parsed_due_date {
        let today = Local::now().naive_local();
        let days_left = parsed_due_date.signed_duration_since(today).num_days();
        if app.state.focus == Focus::CardDueDate {
            Span::styled(
                format!("Due: {}", card.date_due),
                app.theme.list_select_style,
            )
        } else if days_left <= app.config.warning_delta.into() && days_left >= 0 {
            Span::styled(
                format!("Due: {}", card.date_due),
                app.theme.card_due_warning_style,
            )
        } else if days_left < 0 {
            Span::styled(
                format!("Due: {}", card.date_due),
                app.theme.card_due_overdue_style,
            )
        } else {
            Span::styled(
                format!("Due: {}", card.date_due),
                app.theme.card_due_default_style,
            )
        }
    } else if app.state.focus == Focus::CardDueDate {
        Span::styled(
            format!("Due: {}", card.date_due),
            app.theme.list_select_style,
        )
    } else {
        Span::styled(
            format!("Due: {}", card.date_due),
            app.theme.card_due_default_style,
        )
    };
    let card_priority_styled = if app.state.focus == Focus::CardPriority {
        Span::styled(card_priority, app.theme.list_select_style)
    } else if card.priority == CardPriority::High {
        Span::styled(card_priority, app.theme.card_priority_high_style)
    } else if card.priority == CardPriority::Medium {
        Span::styled(card_priority, app.theme.card_priority_medium_style)
    } else if card.priority == CardPriority::Low {
        Span::styled(card_priority, app.theme.card_priority_low_style)
    } else {
        Span::styled(card_priority, app.theme.general_style)
    };
    let card_status_styled = if app.state.focus == Focus::CardStatus {
        Span::styled(card_status, app.theme.list_select_style)
    } else if card.card_status == CardStatus::Complete {
        Span::styled(card_status, app.theme.card_status_completed_style)
    } else if card.card_status == CardStatus::Active {
        Span::styled(card_status, app.theme.card_status_active_style)
    } else if card.card_status == CardStatus::Stale {
        Span::styled(card_status, app.theme.card_status_stale_style)
    } else {
        Span::styled(card_status, app.theme.general_style)
    };
    let card_extra_info_items = vec![
        ListItem::new(vec![Spans::from(card_date_created)]),
        ListItem::new(vec![Spans::from(card_date_modified)]),
        ListItem::new(vec![Spans::from(card_due_date_styled)]),
        ListItem::new(vec![Spans::from(card_date_completed)]),
        ListItem::new(vec![Spans::from(card_priority_styled)]),
        ListItem::new(vec![Spans::from(card_status_styled)]),
    ];
    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, card_chunks[1]) {
        let top_of_list = card_chunks[1].y + 1;
        let mut bottom_of_list = card_chunks[1].y + card_extra_info_items.len() as u16;
        if bottom_of_list > card_chunks[1].bottom() {
            bottom_of_list = card_chunks[1].bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            match mouse_y - top_of_list {
                2 => {
                    app.state.focus = Focus::CardDueDate;
                    app.state.mouse_focus = Some(Focus::CardDueDate);
                    app.state.card_view_comment_list_state.select(None);
                    app.state.card_view_tag_list_state.select(None);
                    app.state.current_cursor_position = None;
                }
                4 => {
                    app.state.focus = Focus::CardPriority;
                    app.state.mouse_focus = Some(Focus::CardPriority);
                    app.state.card_view_comment_list_state.select(None);
                    app.state.card_view_tag_list_state.select(None);
                    app.state.current_cursor_position = None;
                }
                5 => {
                    app.state.focus = Focus::CardStatus;
                    app.state.mouse_focus = Some(Focus::CardStatus);
                    app.state.card_view_comment_list_state.select(None);
                    app.state.card_view_tag_list_state.select(None);
                    app.state.current_cursor_position = None;
                }
                _ => {
                    app.state.focus = Focus::NoFocus;
                    app.state.mouse_focus = None;
                }
            }
            app.state
                .card_view_list_state
                .select(Some((mouse_y - top_of_list) as usize));
        } else {
            app.state.card_view_list_state.select(None);
        }
    };
    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, card_chunks[0]) {
        app.state.focus = Focus::CardDescription;
        app.state.mouse_focus = Some(Focus::CardDescription);
        app.state.card_view_comment_list_state.select(None);
        app.state.card_view_tag_list_state.select(None);
        app.state.current_cursor_position = None;
    }
    let card_tags_style = if app.state.focus == Focus::CardTags {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let card_comments_style = if app.state.focus == Focus::CardComments {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let card_extra_info = List::new(card_extra_info_items).block(
        Block::default()
            .title("Card Info")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(app.theme.general_style),
    );

    let card_tags = if app.state.focus == Focus::CardTags {
        let mut tags = vec![];
        if app.state.card_view_tag_list_state.selected().is_none() {
            for (index, tag) in card.tags.iter().enumerate() {
                tags.push(Span::styled(
                    format!("{}) {} ", index + 1, tag),
                    app.theme.general_style,
                ));
            }
        } else {
            let selected_tag = app.state.card_view_tag_list_state.selected().unwrap();
            for (index, tag) in card.tags.iter().enumerate() {
                if index == selected_tag {
                    tags.push(Span::styled(
                        format!("{}) {} ", index + 1, tag),
                        app.theme.keyboard_focus_style,
                    ));
                } else {
                    tags.push(Span::styled(
                        format!("{}) {} ", index + 1, tag),
                        app.theme.general_style,
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
                app.theme.general_style,
            ));
        }
        tags
    };

    // do the same for card.comments
    let card_comments = if app.state.focus == Focus::CardComments {
        let mut comments = vec![];
        if app.state.card_view_comment_list_state.selected().is_none() {
            for (index, comment) in card.comments.iter().enumerate() {
                comments.push(Span::styled(
                    format!("{}) {} ", index + 1, comment),
                    app.theme.general_style,
                ));
            }
        } else {
            let selected_comment = app.state.card_view_comment_list_state.selected().unwrap();
            for (index, comment) in card.comments.iter().enumerate() {
                if index == selected_comment {
                    comments.push(Span::styled(
                        format!("{}) {} ", index + 1, comment),
                        app.theme.keyboard_focus_style,
                    ));
                } else {
                    comments.push(Span::styled(
                        format!("{}) {} ", index + 1, comment),
                        app.theme.general_style,
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
                app.theme.general_style,
            ));
        }
        comments
    };

    let mut card_tag_spans = vec![];
    let mut collector = String::new();
    let mut collector_start = 0;
    let mut collector_end = 0;
    for (i, tag) in card.tags.iter().enumerate() {
        let tag_string = format!("{}) {} ", i + 1, tag);
        if (collector.len() + tag_string.len()) < card_chunks[1].width as usize {
            collector.push_str(&tag_string);
            collector_end = i + 1;
        } else {
            card_tag_spans.push(Spans::from(
                card_tags[collector_start..collector_end].to_vec(),
            ));
            collector = String::new();
            collector.push_str(&tag_string);
            collector_start = i;
            collector_end = i + 1;
        }
    }
    if !collector.is_empty() {
        card_tag_spans.push(Spans::from(
            card_tags[collector_start..collector_end].to_vec(),
        ));
    }

    let mut card_comment_spans = vec![];
    let mut collector = String::new();
    let mut collector_start = 0;
    let mut collector_end = 0;
    for (i, comment) in card.comments.iter().enumerate() {
        let comment_string = format!("{}) {} ", i + 1, comment);
        if (collector.len() + comment_string.len()) < card_chunks[1].width as usize {
            collector.push_str(&comment_string);
            collector_end = i + 1;
        } else {
            card_comment_spans.push(Spans::from(
                card_comments[collector_start..collector_end].to_vec(),
            ));
            collector = String::new();
            collector.push_str(&comment_string);
            collector_start = i;
            collector_end = i + 1;
        }
    }
    if !collector.is_empty() {
        card_comment_spans.push(Spans::from(
            card_comments[collector_start..collector_end].to_vec(),
        ));
    }

    let card_tags_paragraph = Paragraph::new(card_tag_spans.clone())
        .block(
            Block::default()
                .title("Tags")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(card_tags_style),
        )
        .alignment(Alignment::Left);

    let card_comments_paragraph = Paragraph::new(card_comment_spans.clone())
        .block(
            Block::default()
                .title("Comments")
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(card_comments_style),
        )
        .alignment(Alignment::Left);

    let extra_info_chunks = {
        let card_tags = card_tags
            .clone()
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();
        let card_comments = card_comments
            .clone()
            .iter()
            .map(|span| span.content.to_string())
            .collect::<String>();

        let available_height = card_chunks[1].height - 8;
        let tags_height = if card_tags.is_empty() {
            0
        } else {
            textwrap::wrap(&card_tags, card_chunks[1].width as usize).len() as u16
        };
        let comments_height = if card_comments.is_empty() {
            0
        } else {
            textwrap::wrap(&card_comments, card_chunks[1].width as usize).len() as u16
        };

        let mut tags_height = tags_height + 2;
        let mut comments_height = comments_height + 2;

        if tags_height + comments_height > available_height {
            if tags_height > comments_height {
                tags_height = available_height - comments_height;
            } else {
                comments_height = available_height - tags_height;
            }
        } else {
            comments_height = available_height - tags_height;
        }

        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(tags_height),
                Constraint::Length(comments_height),
            ])
            .split(card_chunks[1])
    };

    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, extra_info_chunks[1]) {
        app.state.focus = Focus::CardTags;
        app.state.mouse_focus = Some(Focus::CardTags);
        app.state.card_view_comment_list_state.select(None);
    }

    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, extra_info_chunks[2]) {
        app.state.focus = Focus::CardComments;
        app.state.mouse_focus = Some(Focus::CardComments);
        app.state.card_view_tag_list_state.select(None);
    }

    rect.render_widget(card_extra_info, extra_info_chunks[0]);
    rect.render_widget(card_tags_paragraph, extra_info_chunks[1]);
    rect.render_widget(card_comments_paragraph, extra_info_chunks[2]);

    if app.state.app_status == AppStatus::UserInput {
        match app.state.focus {
            Focus::CardDescription => {
                let (x_pos, y_pos) = calculate_cursor_position(
                    wrapped_description,
                    app.state.current_cursor_position.unwrap_or(0),
                    card_chunks[0],
                );
                rect.set_cursor(x_pos, y_pos);
            }
            Focus::CardDueDate => {
                let (x_pos, y_pos) = calculate_cursor_position(
                    textwrap::wrap(&card.date_due, card_chunks[1].width as usize),
                    app.state.current_cursor_position.unwrap_or(0),
                    card_chunks[1],
                );
                rect.set_cursor(x_pos + 5, y_pos + 2); // +5 and +2 are to account for the "Due: " text and extra info position offset
            }
            Focus::CardTags => {
                // TODO: Fix cursor position
                // card_tag_spans.0 is a vector of spans check app.state.card_view_tag_list_state.selected() check i which span is selected and then calculate the cursor position
                if app.state.card_view_tag_list_state.selected().is_some() {
                    let selected_index = app.state.card_view_tag_list_state.selected().unwrap();
                    let mut counter = 0;
                    let mut y_index = 0;
                    let mut length_before_selected_tag = 0;
                    let mut prv_spans_length = 0;
                    let tag_offset = 3;
                    for spans in card_tag_spans.iter() {
                        for _ in spans.0.iter() {
                            if counter == selected_index {
                                break;
                            } else {
                                let element = spans.0.get(counter - prv_spans_length);
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
                        prv_spans_length += spans.0.iter().len();
                        length_before_selected_tag = 0;
                    }
                    let digits_in_counter = (counter + 1).to_string().len();
                    let x_pos = extra_info_chunks[1].left()
                        + length_before_selected_tag as u16
                        + app.state.current_cursor_position.unwrap_or(0) as u16
                        + tag_offset
                        + digits_in_counter as u16;
                    let y_pos = extra_info_chunks[1].top() + y_index as u16 + 1;
                    rect.set_cursor(x_pos, y_pos);
                }
            }
            Focus::CardComments => {
                // do the same as tags
                if app.state.card_view_comment_list_state.selected().is_some() {
                    let selected_index = app.state.card_view_comment_list_state.selected().unwrap();
                    let mut counter = 0;
                    let mut y_index = 0;
                    let mut length_before_selected_comment = 0;
                    let mut prv_spans_length = 0;
                    let comment_offset = 3;
                    for spans in card_comment_spans.iter() {
                        for _ in spans.0.iter() {
                            if counter == selected_index {
                                break;
                            } else {
                                let element = spans.0.get(counter - prv_spans_length);
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
                        prv_spans_length += spans.0.iter().len();
                        length_before_selected_comment = 0;
                    }
                    let digits_in_counter = (counter + 1).to_string().len();
                    let x_pos = extra_info_chunks[2].left()
                        + length_before_selected_comment as u16
                        + app.state.current_cursor_position.unwrap_or(0) as u16
                        + comment_offset
                        + digits_in_counter as u16;
                    let y_pos = extra_info_chunks[2].top() + y_index as u16 + 1;
                    rect.set_cursor(x_pos, y_pos);
                }
            }
            _ => {}
        }
    }

    if app.card_being_edited.is_some() {
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, card_chunks[2]) {
            app.state.focus = Focus::SubmitButton;
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state.card_view_comment_list_state.select(None);
            app.state.card_view_tag_list_state.select(None);
            app.state.current_cursor_position = None;
        }
        let save_changes_style = if app.state.focus == Focus::SubmitButton {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
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
        rect.render_widget(save_changes_button, card_chunks[2]);
    }

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_command_palette<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
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

    let search_results = if app.command_palette.search_results.is_some() {
        // convert the vec of strings to a vec of list items
        let raw_search_results = app.command_palette.search_results.as_ref().unwrap();

        let mut list_items = vec![];
        // make a for loop and go through the raw search results and check if the current item has a character that is in the charaters of the search string highlight it with selected style using Span::Styled
        for item in raw_search_results {
            let mut spans = vec![];
            for (_, c) in item.to_string().chars().enumerate() {
                if current_search_text_input
                    .to_lowercase()
                    .contains(c.to_string().to_lowercase().as_str())
                {
                    spans.push(Span::styled(c.to_string(), app.theme.keyboard_focus_style));
                } else {
                    spans.push(Span::styled(c.to_string(), app.theme.general_style));
                }
            }
            list_items.push(ListItem::new(vec![Spans::from(spans)]));
        }
        list_items
    } else {
        app.command_palette
            .available_commands
            .iter()
            .map(|s| ListItem::new(vec![Spans::from(s.to_string())]))
            .collect::<Vec<ListItem>>()
    };

    let search_results_length = if (search_results.len() + 2) > 3 {
        if (search_results.len() + 2) > (rect.size().height - 7) as usize {
            rect.size().height - 7
        } else {
            (search_results.len() + 2) as u16
        }
    } else {
        3
    };

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(2),
                Constraint::Length(3),
                Constraint::Length(search_results_length),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(horizontal_chunks[1]);

    let search_box_text = if app.state.current_user_input.is_empty() {
        vec![Spans::from("Start typing to search")]
    } else {
        vec![Spans::from(app.state.current_user_input.clone())]
    };

    let current_cursor_position = if app.state.current_cursor_position.is_some() {
        app.state.current_cursor_position.unwrap() as u16
    } else {
        app.state.current_user_input.len() as u16
    };
    let x_offset = current_cursor_position % (vertical_chunks[1].width - 2);
    let y_offset = current_cursor_position / (vertical_chunks[1].width - 2);
    let x_cursor_position = vertical_chunks[1].x + x_offset + 1;
    let y_cursor_position = vertical_chunks[1].y + y_offset + 1;
    rect.set_cursor(x_cursor_position, y_cursor_position);

    // make a search bar and display all the commands that match the search below it in a list
    let search_bar = Paragraph::new(search_box_text)
        .block(
            Block::default()
                .title("Command Palette")
                .borders(Borders::ALL)
                .style(app.theme.general_style)
                .border_type(BorderType::Rounded),
        )
        .wrap(Wrap { trim: false });
    rect.render_widget(Clear, vertical_chunks[1]);
    render_blank_styled_canvas(rect, app, vertical_chunks[1], false);
    rect.render_widget(search_bar, vertical_chunks[1]);

    let search_results = List::new(search_results)
        .block(
            Block::default()
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    rect.render_widget(Clear, vertical_chunks[2]);
    render_blank_styled_canvas(rect, app, vertical_chunks[2], false);
    rect.render_stateful_widget(
        search_results,
        vertical_chunks[2],
        &mut app.state.command_palette_list_state,
    );

    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, vertical_chunks[2]) {
        app.state.mouse_focus = Some(Focus::CommandPalette);
        app.state.focus = Focus::CommandPalette;
        let top_of_list = vertical_chunks[2].y + 1;
        let mut bottom_of_list = vertical_chunks[2].y + search_results_length;
        if bottom_of_list > vertical_chunks[2].bottom() {
            bottom_of_list = vertical_chunks[2].bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .command_palette_list_state
                .select(Some((mouse_y - top_of_list) as usize));
        }
    }

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_change_ui_mode_popup<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let all_ui_modes = UiMode::all()
        .iter()
        .map(|s| ListItem::new(vec![Spans::from(s.as_str().to_string())]))
        .collect::<Vec<ListItem>>();

    let percent_height =
        (((all_ui_modes.len() + 3) as f32 / rect.size().height as f32) * 100.0) as u16;

    let popup_area = centered_rect(50, percent_height, rect.size());

    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, popup_area) {
        app.state.mouse_focus = Some(Focus::ChangeUiModePopup);
        app.state.focus = Focus::ChangeUiModePopup;
        let top_of_list = popup_area.y + 1;
        let mut bottom_of_list = popup_area.y + all_ui_modes.len() as u16;
        if bottom_of_list > popup_area.bottom() {
            bottom_of_list = popup_area.bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .default_view_state
                .select(Some((mouse_y - top_of_list) as usize));
        }
    }
    let ui_modes = List::new(all_ui_modes)
        .block(
            Block::default()
                .title("Change UI Mode")
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(ui_modes, popup_area, &mut app.state.default_view_state);

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_change_card_status_popup<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let mut card_name = String::new();
    let mut board_name = String::new();
    if let Some(current_board_id) = app.state.current_board_id {
        if let Some(current_board) = app.boards.iter().find(|b| b.id == current_board_id) {
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
        .map(|s| ListItem::new(vec![Spans::from(s.to_string())]))
        .collect::<Vec<ListItem>>();
    let percent_height =
        (((all_statuses.len() + 3) as f32 / rect.size().height as f32) * 100.0) as u16;
    let popup_area = centered_rect(50, percent_height, rect.size());
    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, popup_area) {
        app.state.mouse_focus = Some(Focus::ChangeCardStatusPopup);
        app.state.focus = Focus::ChangeCardStatusPopup;
        let top_of_list = popup_area.y + 1;
        let mut bottom_of_list = popup_area.y + all_statuses.len() as u16;
        if bottom_of_list > popup_area.bottom() {
            bottom_of_list = popup_area.bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .card_status_selector_state
                .select(Some((mouse_y - top_of_list) as usize));
        }
    }
    let statuses = List::new(all_statuses)
        .block(
            Block::default()
                .title(format!(
                    "Changing Status of \"{}\" in {}",
                    card_name, board_name
                ))
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        statuses,
        popup_area,
        &mut app.state.card_status_selector_state,
    );

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_card_priority_selector<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let mut card_name = String::new();
    let mut board_name = String::new();
    if let Some(current_board_id) = app.state.current_board_id {
        if let Some(current_board) = app.boards.iter().find(|b| b.id == current_board_id) {
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
        .map(|p| ListItem::new(vec![Spans::from(p.to_string())]))
        .collect::<Vec<ListItem>>();
    let percent_height =
        (((all_priorities.len() + 3) as f32 / rect.size().height as f32) * 100.0) as u16;
    let popup_area = centered_rect(50, percent_height, rect.size());
    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, popup_area) {
        app.state.mouse_focus = Some(Focus::ChangeCardPriorityPopup);
        app.state.focus = Focus::ChangeCardPriorityPopup;
        let top_of_list = popup_area.y + 1;
        let mut bottom_of_list = popup_area.y + all_priorities.len() as u16;
        if bottom_of_list > popup_area.bottom() {
            bottom_of_list = popup_area.bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .card_priority_selector_state
                .select(Some((mouse_y - top_of_list) as usize));
        }
    }
    let priorities = List::new(all_priorities)
        .block(
            Block::default()
                .title(format!(
                    "Changing Priority of \"{}\" in {}",
                    card_name, board_name
                ))
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);

    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        priorities,
        popup_area,
        &mut app.state.card_priority_selector_state,
    );

    if app.config.enable_mouse_support {
        render_close_button(rect, app);
    }
}

pub fn render_debug_panel<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let current_ui_mode = &app.state.ui_mode.to_string();
    let popup_mode = if app.state.popup_mode.is_some() {
        app.state.popup_mode.as_ref().unwrap().to_string()
    } else {
        "None".to_string()
    };
    let ui_render_time = if app.state.ui_render_time.is_some() {
        let render_time = app.state.ui_render_time.unwrap();
        // render time is in microseconds, so we convert it to milliseconds if render time is greater than 1 millisecond
        if render_time > 1000 {
            format!("{}ms", render_time / 1000)
        } else {
            format!("{}μs", render_time)
        }
    } else {
        "None".to_string()
    };
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;

    let menu_area = top_left_rect(30, 30, rect.size());
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
                Spans::from(format!("{}{}", &s[..menu_area.width as usize - 5], "..."))
            } else {
                Spans::from(s.to_string())
            }
        })
        .collect::<Vec<Spans>>();
    let debug_panel = Paragraph::new(strings)
        .block(
            Block::default()
                .title("Debug Panel")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(app.theme.general_style)
                .border_style(app.theme.log_debug_style),
        )
        .wrap(Wrap { trim: false });
    rect.render_widget(Clear, menu_area);
    render_blank_styled_canvas(rect, app, menu_area, false);
    rect.render_widget(debug_panel, menu_area);
}

fn check_if_mouse_is_in_area(mouse_coordinates: (u16, u16), rect_to_check: Rect) -> bool {
    let (x, y) = mouse_coordinates;
    let (x1, y1, x2, y2) = (
        rect_to_check.x,
        rect_to_check.y,
        rect_to_check.x + rect_to_check.width,
        rect_to_check.y + rect_to_check.height,
    );
    if x >= x1 && x <= x2 && y >= y1 && y <= y2 {
        return true;
    }
    false
}

fn render_close_button<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let close_btn_area = Rect::new(rect.size().width - 3, 0, 3, 3);
    let close_btn_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, close_btn_area) {
            app.state.mouse_focus = Some(Focus::CloseButton);
            app.state.focus = Focus::CloseButton;
            app.theme.mouse_focus_style
        } else {
            app.theme.error_text_style
        };
    // render a X in the top right corner of the rect
    let close_btn = Paragraph::new(vec![Spans::from("X")])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(close_btn_style),
        )
        .alignment(Alignment::Right);
    rect.render_widget(close_btn, close_btn_area);
}

pub fn render_change_theme_popup<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_area = centered_rect(50, 50, rect.size());
    let theme_list = app
        .all_themes
        .iter()
        .map(|t| ListItem::new(vec![Spans::from(t.name.clone())]))
        .collect::<Vec<ListItem>>();
    if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, popup_area) {
        app.state.mouse_focus = Some(Focus::ThemeSelector);
        app.state.focus = Focus::ThemeSelector;
        let top_of_list = popup_area.y + 1;
        let mut bottom_of_list = popup_area.y + theme_list.len() as u16;
        if bottom_of_list > popup_area.bottom() {
            bottom_of_list = popup_area.bottom();
        }
        let mouse_y = app.state.current_mouse_coordinates.1;
        if mouse_y >= top_of_list && mouse_y <= bottom_of_list {
            app.state
                .theme_selector_state
                .select(Some((mouse_y - top_of_list) as usize));
        } else {
            app.state.theme_selector_state.select(None);
        }
    };
    let themes = List::new(theme_list)
        .block(
            Block::default()
                .title("Change Theme")
                .style(app.theme.general_style)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(themes, popup_area, &mut app.state.theme_selector_state);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_create_theme<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    // use serde to iterate over the theme struct and create a list of items to render
    let render_area = rect.size();
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
        .split(render_area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .margin(1)
        .split(main_chunks[0]);
    let theme_table_rows = app.state.theme_being_edited.to_rows(app);
    let list_highlight_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, main_chunks[0]) {
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
                .theme_editor_state
                .select(Some((mouse_y - top_of_list) as usize));
        } else {
            app.state.theme_editor_state.select(None);
        }
        app.theme.list_select_style
    } else if app.state.theme_editor_state.selected().is_some() {
        app.theme.list_select_style
    } else {
        app.theme.general_style
    };
    let theme_block_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if app.state.focus == Focus::ThemeEditor {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let submit_button_style = if app.state.popup_mode.is_some() {
        app.theme.inactive_text_style
    } else if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, main_chunks[1]) {
        app.state.mouse_focus = Some(Focus::SubmitButton);
        app.state.focus = Focus::SubmitButton;
        app.theme.mouse_focus_style
    } else if app.state.focus == Focus::SubmitButton {
        app.theme.keyboard_focus_style
    } else {
        app.theme.general_style
    };
    let theme_title_list = Table::new(theme_table_rows.0)
        .block(Block::default().style(app.theme.general_style))
        .widths(&[Constraint::Percentage(100)])
        .highlight_style(list_highlight_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    rect.render_stateful_widget(
        theme_title_list,
        chunks[0],
        &mut app.state.theme_editor_state,
    );
    let theme_element_list = Table::new(theme_table_rows.1)
        .block(Block::default())
        .widths(&[Constraint::Percentage(100)]);
    rect.render_stateful_widget(
        theme_element_list,
        chunks[1],
        &mut app.state.theme_editor_state,
    );
    let submit_button = Paragraph::new(vec![Spans::from("Create Theme")])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(submit_button_style),
        )
        .alignment(Alignment::Center);
    rect.render_widget(submit_button, main_chunks[1]);

    let border_block = Block::default()
        .title("Create a new Theme")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme_block_style);
    rect.render_widget(border_block, main_chunks[0]);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_edit_specific_style_popup<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_area = centered_rect(90, 80, rect.size());
    // show the fg , bg, and modifiers in a table to be selected by the user
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)].as_ref())
        .margin(2)
        .split(popup_area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(main_chunks[0]);
    let fg_list_border_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[0]) {
            if app.state.edit_specific_style_state.0.selected().is_none() {
                app.state.edit_specific_style_state.0.select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorFG);
            app.state.focus = Focus::StyleEditorFG;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorFG {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let bg_list_border_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
            if app.state.edit_specific_style_state.1.selected().is_none() {
                app.state.edit_specific_style_state.1.select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorBG);
            app.state.focus = Focus::StyleEditorBG;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorBG {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let modifiers_list_border_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[2]) {
            if app.state.edit_specific_style_state.2.selected().is_none() {
                app.state.edit_specific_style_state.2.select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorModifier);
            app.state.focus = Focus::StyleEditorModifier;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorModifier {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let submit_button_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, main_chunks[1]) {
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state.focus = Focus::SubmitButton;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::SubmitButton {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let fg_list_items: Vec<ListItem> = TextColorOptions::to_iter()
        .map(|color| {
            let mut fg_style = Style::default();
            if color.to_color().is_some() {
                fg_style.fg = Some(color.to_color().unwrap());
                ListItem::new(vec![Spans::from(vec![
                    Span::styled("Sample Text", fg_style),
                    Span::styled(format!(" - {}", color), app.theme.general_style),
                ])])
            } else {
                ListItem::new(vec![Spans::from(vec![
                    Span::raw("Sample Text"),
                    Span::styled(format!(" - {}", color), app.theme.general_style),
                ])])
            }
        })
        .collect();
    let fg_list = List::new(fg_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Foreground")
                .border_style(fg_list_border_style),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let bg_list_items: Vec<ListItem> = TextColorOptions::to_iter()
        .map(|color| {
            let mut bg_style = Style::default();
            if color.to_color().is_some() {
                bg_style.bg = Some(color.to_color().unwrap());
                ListItem::new(vec![Spans::from(vec![
                    Span::styled("Sample Text", bg_style),
                    Span::styled(format!(" - {}", color), app.theme.general_style),
                ])])
            } else {
                ListItem::new(vec![Spans::from(vec![
                    Span::raw("Sample Text"),
                    Span::styled(format!(" - {}", color), app.theme.general_style),
                ])])
            }
        })
        .collect();
    let bg_list = List::new(bg_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Background")
                .border_style(bg_list_border_style),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let modifier_list_items: Vec<ListItem> = TextModifierOptions::to_iter()
        .map(|modifier| {
            let modifier_style = Style {
                add_modifier: modifier.to_modifier(),
                ..Style::default()
            };
            ListItem::new(vec![Spans::from(vec![
                Span::styled("Sample Text", modifier_style),
                Span::styled(format!(" - {}", modifier), app.theme.general_style),
            ])])
        })
        .collect();
    let modifier_list = List::new(modifier_list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Modifier")
                .border_style(modifiers_list_border_style),
        )
        .highlight_style(app.theme.list_select_style)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    let theme_style_being_edited_index = app.state.theme_editor_state.selected();
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
        .border_style(app.theme.general_style);

    let submit_button = Paragraph::new("Confirm Changes")
        .style(submit_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(submit_button_style),
        )
        .alignment(Alignment::Center);
    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, false);
    rect.render_stateful_widget(
        fg_list,
        chunks[0],
        &mut app.state.edit_specific_style_state.0,
    );
    rect.render_stateful_widget(
        bg_list,
        chunks[1],
        &mut app.state.edit_specific_style_state.1,
    );
    rect.render_stateful_widget(
        modifier_list,
        chunks[2],
        &mut app.state.edit_specific_style_state.2,
    );
    rect.render_widget(submit_button, main_chunks[1]);
    rect.render_widget(border_block, popup_area);
    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_save_theme_prompt<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_area = centered_rect(50, 50, rect.size());
    // make two buttons save theme to file and only save for current session
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .margin(2)
        .split(popup_area);
    let save_theme_button_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[0]) {
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state.focus = Focus::SubmitButton;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::SubmitButton {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let dont_save_theme_button_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
            app.state.mouse_focus = Some(Focus::ExtraFocus);
            app.state.focus = Focus::ExtraFocus;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::ExtraFocus {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let save_theme_button = Paragraph::new("Save Theme to File")
        .style(save_theme_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(save_theme_button_style),
        )
        .alignment(Alignment::Center);
    let dont_save_theme_button = Paragraph::new("Don't Save Theme to File")
        .style(dont_save_theme_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(dont_save_theme_button_style),
        )
        .alignment(Alignment::Center);
    let border_block = Block::default()
        .title("Save Theme?")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.theme.general_style);
    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, true);
    rect.render_widget(save_theme_button, chunks[0]);
    rect.render_widget(dont_save_theme_button, chunks[1]);
    rect.render_widget(border_block, popup_area);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_confirm_discard_card_changes<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let popup_area = centered_rect(30, 25, rect.size());
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .margin(2)
        .split(popup_area);
    let save_card_button_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[0]) {
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state.focus = Focus::SubmitButton;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::SubmitButton {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let dont_save_card_button_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
            app.state.mouse_focus = Some(Focus::ExtraFocus);
            app.state.focus = Focus::ExtraFocus;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::ExtraFocus {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let save_theme_button = Paragraph::new("Yes")
        .style(save_card_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(save_card_button_style),
        )
        .alignment(Alignment::Center);
    let dont_save_theme_button = Paragraph::new("No")
        .style(dont_save_card_button_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(dont_save_card_button_style),
        )
        .alignment(Alignment::Center);
    let border_block = Block::default()
        .title("Save Changes to Card?")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.theme.general_style);
    rect.render_widget(Clear, popup_area);
    rect.render_widget(save_theme_button, chunks[0]);
    rect.render_widget(dont_save_theme_button, chunks[1]);
    rect.render_widget(border_block, popup_area);

    if app.config.enable_mouse_support {
        render_close_button(rect, app)
    }
}

pub fn render_custom_rgb_color_prompt<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    // make a small popup with a text input field and a submit button
    let popup_area = centered_rect(50, 50, rect.size());
    let prompt_text = "Enter a custom RGB color in the format: r,g,b (0-254)";

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .margin(2)
        .split(popup_area);
    let border_block = Block::default()
        .title("Custom RGB Color")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(app.theme.general_style);

    let text_input_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[1]) {
            app.state.mouse_focus = Some(Focus::TextInput);
            app.state.focus = Focus::TextInput;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::TextInput {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let submit_button_style =
        if check_if_mouse_is_in_area(app.state.current_mouse_coordinates, chunks[2]) {
            app.state.mouse_focus = Some(Focus::SubmitButton);
            app.state.focus = Focus::SubmitButton;
            app.state.app_status = AppStatus::Initialized;
            app.theme.mouse_focus_style
        } else if app.state.focus == Focus::SubmitButton {
            app.theme.keyboard_focus_style
        } else {
            app.theme.general_style
        };
    let prompt_text = Paragraph::new(prompt_text)
        .style(app.theme.general_style)
        .block(Block::default())
        .alignment(Alignment::Center);
    let text_input = Paragraph::new(app.state.current_user_input.clone())
        .style(app.theme.general_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(text_input_style),
        );
    let submit_button = Paragraph::new("Submit")
        .style(app.theme.general_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(submit_button_style),
        )
        .alignment(Alignment::Center);

    rect.render_widget(Clear, popup_area);
    render_blank_styled_canvas(rect, app, popup_area, true);
    rect.render_widget(prompt_text, chunks[0]);
    rect.render_widget(text_input, chunks[1]);
    rect.render_widget(submit_button, chunks[2]);
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

pub fn render_blank_styled_canvas<B>(
    rect: &mut Frame<B>,
    app: &mut App,
    render_area: Rect,
    popup_mode: bool,
) where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100), Constraint::Percentage(100)].as_ref())
        .split(render_area);
    let mut styled_text = vec![];
    for _ in 0..chunks[0].width + 1 {
        styled_text.push(" ".to_string());
    }
    let mut render_text = vec![];
    for _ in 0..chunks[0].height {
        render_text.push(format!("{}\n", styled_text.join("")));
    }
    let styled_text = if popup_mode {
        Paragraph::new(render_text.join(""))
            .style(app.theme.inactive_text_style)
            .block(Block::default())
    } else {
        Paragraph::new(render_text.join(""))
            .style(app.theme.general_style)
            .block(Block::default())
    };
    rect.render_widget(styled_text, render_area);
}
