use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App,
    },
    constants::{
        LIST_SELECTED_SYMBOL, SCROLLBAR_BEGIN_SYMBOL, SCROLLBAR_END_SYMBOL, SCROLLBAR_TRACK_SYMBOL,
    },
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::FilterByTag,
            utils::{
                centered_rect_with_percentage, check_if_active_and_get_style,
                check_if_mouse_is_in_area, get_button_style,
            },
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState,
    },
    Frame,
};

impl Renderable for FilterByTag {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let submit_style = get_button_style(app, Focus::SubmitButton, None, is_active, false);
        let tag_box_style = get_button_style(app, Focus::FilterByTagPopup, None, is_active, false);
        let progress_bar_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.progress_bar_style,
        );
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

        if let Some(all_available_tags) = &app.state.all_available_tags {
            let popup_area = centered_rect_with_percentage(80, 80, rect.area());
            let empty_vec = vec![];
            let selected_tags = if let Some(filter_tags) = &app.state.filter_tags {
                filter_tags
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
                            list_select_style,
                        )])])
                    } else {
                        ListItem::new(vec![Line::from(vec![Span::styled(
                            format!("{} - {} occurrences", tag.0, tag.1),
                            general_style,
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
                        .style(general_style)
                        .border_style(tag_box_style),
                )
                .highlight_style(list_select_style)
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
                .get_first_keybinding(KeyBindingEnum::GoToPreviousViewOrCancel)
                .unwrap_or("".to_string());

            let help_spans = Line::from(vec![
                Span::styled("Use ", help_text_style),
                Span::styled(up_key, help_key_style),
                Span::styled(" and ", help_text_style),
                Span::styled(down_key, help_key_style),
                Span::styled(
                    " or scroll with the mouse to navigate. Press ",
                    help_text_style,
                ),
                Span::styled(accept_key.clone(), help_key_style),
                Span::styled(
                    " To select a Tag (multiple tags can be selected). Press ",
                    help_text_style,
                ),
                Span::styled(accept_key, help_key_style),
                Span::styled(
                    " on an already selected tag to deselect it. Press ",
                    help_text_style,
                ),
                Span::styled(cancel_key, help_key_style),
                Span::styled(" to cancel, Press ", help_text_style),
                Span::styled(next_focus_key, help_key_style),
                Span::styled(" or ", help_text_style),
                Span::styled(prv_focus_key, help_key_style),
                Span::styled(" to change focus", help_text_style),
            ]);

            let help = Paragraph::new(help_spans)
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

            let submit_btn_text = if let Some(filter_tags) = &app.state.filter_tags {
                if filter_tags.len() > 1 {
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
                        .style(general_style)
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
                .style(progress_bar_style)
                .end_symbol(SCROLLBAR_END_SYMBOL)
                .track_symbol(SCROLLBAR_TRACK_SYMBOL)
                .track_style(app.current_theme.inactive_text_style);
            let mut scrollbar_state = ScrollbarState::new(all_tags.len()).position(current_index);
            let scrollbar_area = chunks[0].inner(Margin {
                vertical: 1,
                horizontal: 0,
            });

            if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[0]) {
                app.state.mouse_focus = Some(Focus::FilterByTagPopup);
                app.state.set_focus(Focus::FilterByTagPopup);
            }
            if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[2]) {
                app.state.mouse_focus = Some(Focus::SubmitButton);
                app.state.set_focus(Focus::SubmitButton);
            }

            render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);
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
            render_close_button(rect, app, is_active);
        }
    }
}
