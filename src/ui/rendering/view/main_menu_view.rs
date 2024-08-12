use crate::{
    app::App,
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{draw_help, draw_title, render_close_button, render_logs},
            utils::{
                check_if_active_and_get_style,
                get_mouse_focusable_field_style_with_vertical_list_selection,
            },
            view::MainMenuView,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

impl Renderable for MainMenuView {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
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
            .split(rect.area());

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

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
        let rapid_blink_general_style = if is_active {
            general_style.add_modifier(Modifier::RAPID_BLINK)
        } else {
            general_style
        };

        let main_menu_help = draw_help(app, chunks[2], is_active);
        let help_separator = Block::default()
            .borders(Borders::LEFT)
            .border_style(general_style);

        rect.render_widget(draw_title(app, chunks[0], is_active), chunks[0]);

        if let Some(email_id) = &app.state.user_login_data.email_id {
            let email_id = email_id.to_string();
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
                .border_style(rapid_blink_general_style)
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
                .block(Block::default().style(rapid_blink_general_style))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });

            let email_id_text = Paragraph::new(email_id)
                .block(Block::default().style(rapid_blink_general_style))
                .alignment(Alignment::Center)
                .wrap(ratatui::widgets::Wrap { trim: true });

            draw_main_menu(app, sub_main_menu_chunks[0], rect, is_active);
            rect.render_widget(border_block, sub_main_menu_chunks[1]);
            rect.render_widget(heading_text, email_chunks[1]);
            rect.render_widget(email_id_text, email_chunks[3]);
        } else {
            draw_main_menu(app, chunks[1], rect, is_active);
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
        render_logs(app, true, chunks[3], rect, is_active);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}

fn draw_main_menu(app: &mut App, render_area: Rect, rect: &mut Frame, is_active: bool) {
    let main_menu_items = app.main_menu.all();
    let menu_style = get_mouse_focusable_field_style_with_vertical_list_selection(
        app,
        &main_menu_items,
        render_area,
        is_active,
    );
    let default_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.general_style,
    );
    let highlight_style = check_if_active_and_get_style(
        is_active,
        app.current_theme.inactive_text_style,
        app.current_theme.list_select_style,
    );
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
