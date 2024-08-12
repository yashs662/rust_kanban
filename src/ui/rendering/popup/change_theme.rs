use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App,
    },
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::ChangeTheme,
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
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

impl Renderable for ChangeTheme {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
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
        let render_area = centered_rect_with_percentage(70, 70, rect.area());
        let clear_area = centered_rect_with_percentage(80, 80, rect.area());
        let clear_area_border = Block::default()
            .title("Change Theme")
            .style(general_style)
            .borders(Borders::ALL)
            .border_style(keyboard_focus_style)
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
            app.state.set_focus(Focus::ThemeSelector);
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
            Span::styled(" To select a Theme. Press ", help_text_style),
            Span::styled(cancel_key, help_key_style),
            Span::styled(" to cancel", help_text_style),
        ]);

        let change_theme_help = Paragraph::new(help_spans)
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

        render_blank_styled_canvas(rect, &app.current_theme, clear_area, is_active);
        rect.render_widget(clear_area_border, clear_area);
        rect.render_stateful_widget(
            themes,
            chunks[0],
            &mut app.state.app_list_states.theme_selector,
        );
        rect.render_widget(change_theme_help, chunks[1]);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
    }
}
