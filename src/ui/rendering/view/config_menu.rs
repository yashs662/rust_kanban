use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App,
    },
    constants::{SCROLLBAR_BEGIN_SYMBOL, SCROLLBAR_END_SYMBOL, SCROLLBAR_TRACK_SYMBOL},
    ui::{
        rendering::{
            common::{draw_title, render_close_button, render_logs},
            utils::{
                check_if_active_and_get_style, get_button_style, get_mouse_focusable_field_style,
                get_scrollable_widget_row_bounds,
            },
            view::ConfigMenu,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::Style,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
    Frame,
};

impl Renderable for ConfigMenu {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
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
            .split(rect.area());

        let reset_btn_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Fill(1), Constraint::Fill(1)].as_ref())
            .split(chunks[2]);

        let reset_both_style = get_button_style(
            app,
            Focus::SubmitButton,
            Some(&reset_btn_chunks[0]),
            is_active,
            true,
        );
        let reset_config_style = get_button_style(
            app,
            Focus::ExtraFocus,
            Some(&reset_btn_chunks[1]),
            is_active,
            true,
        );
        let scrollbar_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.progress_bar_style,
        );
        let config_text_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
        let default_style =
            get_mouse_focusable_field_style(app, Focus::ConfigTable, &chunks[1], is_active, false);

        let config_table =
            draw_config_table_selector(app, config_text_style, default_style, is_active);

        let all_rows = app.config.to_view_list();
        let total_rows = all_rows.len();
        let current_index = app
            .state
            .app_table_states
            .config
            .selected()
            .unwrap_or(0)
            .min(total_rows - 1);

        // mouse selection, TODO: make this a helper function
        if is_active {
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
                Some(
                    ((current_mouse_y_position - chunks[1].y - 1) + row_start_index as u16)
                        as usize,
                )
            } else {
                None
            };
            if hovered_index.is_some()
                && (app.state.previous_mouse_coordinates != app.state.current_mouse_coordinates)
            {
                app.state.app_table_states.config.select(hovered_index);
            }
        }

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(SCROLLBAR_BEGIN_SYMBOL)
            .style(scrollbar_style)
            .end_symbol(SCROLLBAR_END_SYMBOL)
            .track_symbol(SCROLLBAR_TRACK_SYMBOL)
            .track_style(app.current_theme.inactive_text_style);

        let mut scrollbar_state = ScrollbarState::new(total_rows).position(current_index);
        let scrollbar_area = chunks[1].inner(Margin {
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

        let config_help = draw_config_help(app, is_active);

        rect.render_widget(draw_title(app, chunks[0], is_active), chunks[0]);
        rect.render_stateful_widget(
            config_table,
            chunks[1],
            &mut app.state.app_table_states.config,
        );
        rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        rect.render_widget(reset_both_button, reset_btn_chunks[0]);
        rect.render_widget(reset_config_button, reset_btn_chunks[1]);
        rect.render_widget(config_help, chunks[3]);
        render_logs(app, true, chunks[4], rect, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
    }
}

fn draw_config_table_selector(
    app: &mut App,
    config_text_style: Style,
    default_style: Style,
    is_active: bool,
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

    let highlight_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.list_select_style,
    );

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

fn draw_config_help<'a>(app: &mut App, is_active: bool) -> Paragraph<'a> {
    let help_box_style = get_button_style(app, Focus::ConfigHelp, None, is_active, false);
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
