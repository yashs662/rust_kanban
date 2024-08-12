use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App, DateTimeFormat,
    },
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::ChangeDateFormat,
            utils::{
                calculate_mouse_list_select_index, centered_rect_with_percentage,
                check_if_active_and_get_style, check_if_mouse_is_in_area,
            },
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

impl Renderable for ChangeDateFormat {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let render_area = centered_rect_with_percentage(70, 70, rect.area());

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(5)].as_ref())
            .split(render_area);

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
        let list_select_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.list_select_style,
        );
        let keyboard_focus_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.keyboard_focus_style,
        );

        let all_date_formats = DateTimeFormat::get_all_date_formats();
        let all_date_formats = all_date_formats
            .iter()
            .map(|s| ListItem::new(vec![Line::from(s.to_human_readable_string().to_string())]))
            .collect::<Vec<ListItem>>();

        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &render_area) {
            app.state.mouse_focus = Some(Focus::ChangeDateFormatPopup);
            app.state.set_focus(Focus::ChangeDateFormatPopup);
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
                    .style(general_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(list_select_style)
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
            .get_first_keybinding(KeyBindingEnum::GoToPreviousViewOrCancel)
            .unwrap_or("".to_string());

        let help_spans = Line::from(vec![
            Span::styled("Use ", help_text_style),
            Span::styled(up_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(down_key, help_key_style),
            Span::styled(
                " to navigate or use the mouse cursor. Press ",
                help_text_style,
            ),
            Span::styled(accept_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled("<Mouse Left Click>", help_key_style),
            Span::styled(" To select a Default Date Format. Press ", help_text_style),
            Span::styled(cancel_key, help_key_style),
            Span::styled(" to cancel", help_text_style),
        ]);

        let default_date_picker_help = Paragraph::new(help_spans)
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .title("Help")
                    .borders(Borders::ALL)
                    .style(general_style)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        let clear_area = centered_rect_with_percentage(80, 80, rect.area());
        let clear_area_border = Block::default()
            .title("Default Date Format Picker")
            .style(general_style)
            .borders(Borders::ALL)
            .border_style(keyboard_focus_style)
            .border_type(BorderType::Rounded);
        render_blank_styled_canvas(rect, &app.current_theme, clear_area, is_active);
        rect.render_widget(clear_area_border, clear_area);
        rect.render_stateful_widget(
            date_formats,
            chunks[0],
            &mut app.state.app_list_states.date_format_selector,
        );
        rect.render_widget(default_date_picker_help, chunks[1]);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
