use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App,
    },
    constants::{SCROLLBAR_BEGIN_SYMBOL, SCROLLBAR_END_SYMBOL, SCROLLBAR_TRACK_SYMBOL},
    ui::{
        rendering::{
            common::{draw_title, render_close_button},
            utils::{
                check_if_active_and_get_style, get_button_style, get_mouse_focusable_field_style,
            },
            view::EditKeybindings,
        },
        Renderable,
    },
};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table,
    },
    Frame,
};

impl Renderable for EditKeybindings {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
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

        let default_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
        );
        let scrollbar_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.progress_bar_style,
        );
        let reset_style =
            get_button_style(app, Focus::SubmitButton, Some(&chunks[3]), is_active, true);
        let current_element_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.list_select_style,
        );
        let table_border_style = get_mouse_focusable_field_style(
            app,
            Focus::EditKeybindingsTable,
            &chunks[1],
            is_active,
            false,
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
        // sort according to the first string in the row
        table_items.sort_by(|a, b| a[0].cmp(&b[0]));

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
        let scrollbar_area = chunks[1].inner(Margin {
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

        rect.render_widget(draw_title(app, chunks[0], is_active), chunks[0]);
        rect.render_stateful_widget(
            t,
            chunks[1],
            &mut app.state.app_table_states.edit_keybindings,
        );
        rect.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        rect.render_widget(edit_keybinding_help, chunks[2]);
        rect.render_widget(reset_button, chunks[3]);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
    }
}
