use crate::{
    app::{state::Focus, App},
    constants::MOUSE_OUT_OF_BOUNDS_COORDINATES,
    ui::{
        rendering::{
            common::render_blank_styled_canvas,
            popup::DateTimePicker,
            utils::{check_if_active_and_get_style, check_if_mouse_is_in_area, get_button_style},
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

impl Renderable for DateTimePicker {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let anchor = app
            .widgets
            .date_time_picker
            .viewport_corrected_anchor
            .unwrap_or_default();
        let (current_month, current_year) = app
            .widgets
            .date_time_picker
            .calculate_styled_lines_of_dates(is_active, &app.current_theme);
        let render_area = Rect {
            x: anchor.0,
            y: anchor.1,
            width: app.widgets.date_time_picker.widget_width,
            height: app.widgets.date_time_picker.widget_height,
        };

        app.widgets.date_time_picker.current_render_area = Some(render_area);

        // 3 is for the " - ", additional 4 is to compensate for the borders that show when focus is on month or year
        let title_length = (current_month.len() + 3 + current_year.len() + 4) as u16;
        let padding = (render_area
            .width
            .min(app.widgets.date_time_picker.date_target_width)
            - 3
            - 2)
        .saturating_sub(title_length); // 3 is for the Time section expand button, 2 is for margin
        let month_length = current_month.len() as u16 + (padding / 2) + 2;
        let year_length = current_year.len() as u16 + (padding / 2) + 2;

        let (date_picker_render_area, time_picker_render_area) =
            if app.widgets.date_time_picker.widget_width
                > app.widgets.date_time_picker.date_target_width
            {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Length(app.widgets.date_time_picker.date_target_width),
                            Constraint::Fill(1),
                        ]
                        .as_ref(),
                    )
                    .split(render_area);
                // add margin to the time picker
                let time_picker_render_area = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Fill(1)].as_ref())
                    .margin(1)
                    .split(chunks[1]);
                (chunks[0], Some(time_picker_render_area[1]))
            } else {
                (render_area, None)
            };

        let chunks = Layout::default()
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
            .split(date_picker_render_area);
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(month_length),
                    Constraint::Length(1),
                    Constraint::Length(year_length),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(chunks[0]);

        let time_picker_toggle_style = get_button_style(
            app,
            Focus::DTPToggleTimePicker,
            Some(&header_chunks[3]),
            is_active,
            false,
        );
        let month_style = get_button_style(
            app,
            Focus::DTPMonth,
            Some(&header_chunks[0]),
            is_active,
            false,
        );
        let year_style = get_button_style(
            app,
            Focus::DTPYear,
            Some(&header_chunks[2]),
            is_active,
            false,
        );
        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );

        let month_block = if app.state.focus == Focus::DTPMonth {
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(month_style)
        } else {
            Block::default()
        };

        let year_block = if app.state.focus == Focus::DTPYear {
            Block::default()
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_type(BorderType::Rounded)
                .border_style(year_style)
        } else {
            Block::default()
        };

        let border_block = if app.widgets.date_time_picker.time_picker_active {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(general_style)
                .title("Date Time Picker")
        } else {
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(general_style)
                .title("Date Picker")
        };

        let time_picker_toggle_button = if app.widgets.date_time_picker.time_picker_active {
            "<"
        } else {
            ">"
        };

        let main_paragraph =
            Paragraph::new(app.widgets.date_time_picker.styled_date_lines.0.clone())
                .block(Block::default())
                .wrap(ratatui::widgets::Wrap { trim: true })
                .alignment(Alignment::Center);
        let month_paragraph = Paragraph::new(current_month)
            .style(month_style)
            .block(month_block)
            .alignment(Alignment::Center);
        let separator_paragraph = Paragraph::new("-")
            .style(general_style)
            .block(Block::default())
            .alignment(Alignment::Center);
        let year_paragraph = Paragraph::new(current_year)
            .style(year_style)
            .block(year_block)
            .alignment(Alignment::Center);
        let toggle_time_panel_paragraph = Paragraph::new(time_picker_toggle_button)
            .style(time_picker_toggle_style)
            .block(Block::default())
            .alignment(Alignment::Right);

        if !check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &render_area)
            && (app.state.current_mouse_coordinates != MOUSE_OUT_OF_BOUNDS_COORDINATES)
        {
            app.state.focus = Focus::NoFocus;
        }

        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[2]) {
            app.state.focus = Focus::DTPCalender;
            let maybe_date_to_select = if let Some((calculated_pos, _, _)) =
                &app.widgets.date_time_picker.calculated_mouse_coords
            {
                calculated_pos.iter().find_map(|(rect, date)| {
                    if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, rect) {
                        Some(*date)
                    } else {
                        None
                    }
                })
            } else {
                None
            };

            if let Some(date_to_select) = maybe_date_to_select {
                app.widgets
                    .date_time_picker
                    .select_date_in_current_month(date_to_select);
            }
        }

        render_blank_styled_canvas(rect, &app.current_theme, render_area, is_active);
        rect.render_widget(border_block, render_area);
        rect.render_widget(month_paragraph, header_chunks[0]);
        rect.render_widget(separator_paragraph, header_chunks[1]);
        rect.render_widget(year_paragraph, header_chunks[2]);
        rect.render_widget(toggle_time_panel_paragraph, header_chunks[3]);
        rect.render_widget(main_paragraph, chunks[2]);

        if app.widgets.date_time_picker.time_picker_active && time_picker_render_area.is_some() {
            let render_area = time_picker_render_area.unwrap();
            // only used for mouse detection, it looks like it would be incorrect but it is not
            let time_picker_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(3),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .margin(1)
                .split(render_area);
            if check_if_mouse_is_in_area(
                &app.state.current_mouse_coordinates,
                &time_picker_chunks[0],
            ) {
                app.state.focus = Focus::DTPHour;
            }
            if check_if_mouse_is_in_area(
                &app.state.current_mouse_coordinates,
                &time_picker_chunks[1],
            ) {
                app.state.focus = Focus::DTPMinute;
            }
            if check_if_mouse_is_in_area(
                &app.state.current_mouse_coordinates,
                &time_picker_chunks[2],
            ) {
                app.state.focus = Focus::DTPSecond;
            }
            let time_picker_lines = app.widgets.date_time_picker.get_styled_lines_of_time(
                is_active,
                &app.current_theme,
                &app.state.focus,
            );
            let time_picker_paragraph = Paragraph::new(time_picker_lines)
                .block(Block::default())
                .wrap(ratatui::widgets::Wrap { trim: true });
            rect.render_widget(time_picker_paragraph, render_area);
        }
    }
}
