use crate::{
    app::{state::Focus, App},
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            utils::{check_if_mouse_is_in_area, get_button_style, get_mouse_focusable_field_style},
            view::CreateTheme,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, BorderType, Borders, Paragraph, Table},
    Frame,
};

impl Renderable for CreateTheme {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        // TODO: add a help section
        let render_area = rect.area();
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
            is_active,
            false,
        );
        let reset_button_style = get_mouse_focusable_field_style(
            app,
            Focus::ExtraFocus,
            &button_chunks[1],
            is_active,
            false,
        );

        let theme_being_edited = app.state.get_theme_being_edited();
        let theme_table_rows = theme_being_edited.to_rows(app, is_active);
        // Exception to not using get_button_style as we have to manage other state
        let list_highlight_style = if !is_active {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &main_chunks[0]) {
            app.state.mouse_focus = Some(Focus::ThemeEditor);
            app.state.set_focus(Focus::ThemeEditor);
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
        let theme_block_style = get_button_style(app, Focus::ThemeEditor, None, is_active, false);
        let theme_title_list = Table::new(theme_table_rows.0, [Constraint::Fill(1)])
            .block(Block::default().style(theme_block_style))
            .row_highlight_style(list_highlight_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);
        let theme_element_list = Table::new(theme_table_rows.1, [Constraint::Fill(1)])
            .block(Block::default().style(theme_block_style));
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

        render_blank_styled_canvas(rect, &app.current_theme, render_area, is_active);
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
            render_close_button(rect, app, is_active)
        }
    }
}
