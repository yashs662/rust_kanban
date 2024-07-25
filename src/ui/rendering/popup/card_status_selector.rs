use crate::{
    app::{kanban::CardStatus, state::Focus, App},
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::CardStatusSelector,
            utils::{
                calculate_mouse_list_select_index, centered_rect_with_percentage,
                check_if_active_and_get_style, check_if_mouse_is_in_area,
            },
        },
        Renderable,
    },
};
use ratatui::{
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

impl Renderable for CardStatusSelector {
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
        let mut card_name = String::new();
        let mut board_name = String::new();
        let boards = if app.filtered_boards.is_empty() {
            app.boards.clone()
        } else {
            app.filtered_boards.clone()
        };
        if let Some(current_board_id) = app.state.current_board_id {
            if let Some(current_board) = boards.get_board_with_id(current_board_id) {
                if let Some(current_card_id) = app.state.current_card_id {
                    if let Some(current_card) =
                        current_board.cards.get_card_with_id(current_card_id)
                    {
                        card_name.clone_from(&current_card.name);
                        board_name.clone_from(&current_board.name);
                    }
                }
            }
        }
        let all_statuses = CardStatus::all()
            .iter()
            .map(|s| ListItem::new(vec![Line::from(s.to_string())]))
            .collect::<Vec<ListItem>>();
        let percent_height =
            (((all_statuses.len() + 3) as f32 / rect.size().height as f32) * 100.0) as u16;
        let popup_area = centered_rect_with_percentage(50, percent_height, rect.size());
        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &popup_area) {
            app.state.mouse_focus = Some(Focus::ChangeCardStatusPopup);
            app.state.set_focus(Focus::ChangeCardStatusPopup);
            calculate_mouse_list_select_index(
                app.state.current_mouse_coordinates.1,
                &all_statuses,
                popup_area,
                &mut app.state.app_list_states.card_status_selector,
            );
        }
        let statuses = List::new(all_statuses)
            .block(
                Block::default()
                    .title(format!(
                        "Changing Status of \"{}\" in \"{}\"",
                        card_name, board_name
                    ))
                    .style(general_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(list_select_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);

        render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);
        rect.render_stateful_widget(
            statuses,
            popup_area,
            &mut app.state.app_list_states.card_status_selector,
        );
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
