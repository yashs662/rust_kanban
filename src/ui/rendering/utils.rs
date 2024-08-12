use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    widgets::ListState,
};

use crate::{
    app::{
        state::{AppStatus, Focus},
        App,
    },
    ui::text_box::TextBox,
    util::num_digits,
};

/// Checks for popup to return inactive style if not returns the style passed
pub fn check_if_active_and_get_style(
    is_active: bool,
    inactive_style: Style,
    style: Style,
) -> Style {
    if !is_active {
        inactive_style
    } else {
        style
    }
}

pub fn check_for_card_drag_and_get_style(
    card_drag_mode: bool,
    is_active: bool,
    inactive_style: Style,
    style: Style,
) -> Style {
    if card_drag_mode {
        inactive_style
    } else {
        check_if_active_and_get_style(is_active, inactive_style, style)
    }
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

pub fn centered_rect_with_percentage(percent_width: u16, percent_height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_height) / 2),
                Constraint::Percentage(percent_height),
                Constraint::Percentage((100 - percent_height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_width) / 2),
                Constraint::Percentage(percent_width),
                Constraint::Percentage((100 - percent_width) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn centered_rect_with_length(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - height) / 2),
                Constraint::Length(height),
                Constraint::Length((r.height - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - width) / 2),
                Constraint::Length(width),
                Constraint::Length((r.width - width) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

pub fn top_left_rect(width: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(height),
                Constraint::Length((r.height - height) / 2),
                Constraint::Length((r.height - height) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(width),
                Constraint::Length((r.width - width) / 2),
                Constraint::Length((r.width - width) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[0])[0]
}

/// Returns the style for the field based on the current focus and mouse position and sets the focus if the mouse is in the field area
pub fn get_mouse_focusable_field_style(
    app: &mut App,
    focus: Focus,
    chunk: &Rect,
    is_active: bool,
    auto_user_input_mode: bool,
) -> Style {
    if !is_active {
        app.current_theme.inactive_text_style
    } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, chunk) {
        if app.state.mouse_focus != Some(focus) {
            app.state.app_status = AppStatus::Initialized;
        } else if auto_user_input_mode {
            app.state.app_status = AppStatus::UserInput;
        } else {
            app.state.app_status = AppStatus::Initialized;
        }
        app.state.mouse_focus = Some(focus);
        app.state.set_focus(focus);
        app.current_theme.mouse_focus_style
    } else if app.state.focus == focus {
        app.current_theme.keyboard_focus_style
    } else {
        app.current_theme.general_style
    }
}

// TODO: maybe merge with get_mouse_focusable_field_style
pub fn get_button_style(
    app: &mut App,
    focus: Focus,
    chunk_for_mouse_check: Option<&Rect>,
    is_active: bool,
    default_to_error_style: bool,
) -> Style {
    if !is_active {
        app.current_theme.inactive_text_style
    } else if let Some(chunk) = chunk_for_mouse_check {
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, chunk) {
            app.state.mouse_focus = Some(focus);
            app.state.set_focus(focus);
            app.current_theme.mouse_focus_style
        } else if app.state.focus == focus {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        }
    } else if app.state.focus == focus {
        if default_to_error_style {
            app.current_theme.error_text_style
        } else {
            app.current_theme.keyboard_focus_style
        }
    } else {
        app.current_theme.general_style
    }
}

pub fn get_scrollable_widget_row_bounds(
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
    (start, end.saturating_sub(1))
}

pub fn calculate_viewport_corrected_cursor_position(
    text_box: &TextBox,
    show_line_numbers: &bool,
    chunk: &Rect,
) -> (u16, u16) {
    let (y_pos, _) = text_box.cursor();
    let x_pos = text_box.get_non_ascii_aware_cursor_x_pos();
    let text_box_viewport = text_box.viewport.position();
    let adjusted_x_cursor: u16 = if x_pos as u16 > text_box_viewport.3 {
        x_pos as u16 - text_box_viewport.3
    } else {
        x_pos as u16
    };
    let x_pos = if *show_line_numbers && !text_box.single_line_mode {
        let mut line_number_padding = 3;
        let num_lines = text_box.get_num_lines();
        let num_digits_in_max_line_number = num_digits(num_lines) as u16;
        line_number_padding += num_digits_in_max_line_number;
        chunk.left()
            + 1
            + adjusted_x_cursor.saturating_sub(text_box_viewport.1)
            + line_number_padding
    } else {
        chunk.left() + 1 + adjusted_x_cursor.saturating_sub(text_box_viewport.1)
    };
    let adjusted_y_cursor = if y_pos as u16 > text_box_viewport.2 {
        y_pos as u16 - text_box_viewport.2
    } else {
        y_pos as u16
    };
    let y_pos = chunk.top() + 1 + adjusted_y_cursor - text_box_viewport.0;
    (x_pos, y_pos)
}

// TODO: maybe merge with get_mouse_focusable_field_style
// TODO: see if the name can be shortened
pub fn get_mouse_focusable_field_style_with_vertical_list_selection<T>(
    app: &mut App<'_>,
    main_menu_items: &[T],
    render_area: Rect,
    is_active: bool,
) -> Style {
    let mouse_coordinates = app.state.current_mouse_coordinates;

    if !is_active {
        app.current_theme.inactive_text_style
    } else if check_if_mouse_is_in_area(&mouse_coordinates, &render_area) {
        app.state.mouse_focus = Some(Focus::MainMenu);
        app.state.set_focus(Focus::MainMenu);
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

pub fn calculate_mouse_list_select_index<T>(
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
