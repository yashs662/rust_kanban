use crate::{
    app::{state::Focus, App},
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::ChangeView,
            utils::{
                calculate_mouse_list_select_index, centered_rect_with_length,
                check_if_active_and_get_style, check_if_mouse_is_in_area,
            },
        },
        Renderable, View,
    },
};
use ratatui::{
    text::Line,
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

impl Renderable for ChangeView {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let all_views = View::all_views_as_string()
            .iter()
            .map(|s| ListItem::new(vec![Line::from(s.as_str().to_string())]))
            .collect::<Vec<ListItem>>();

        let popup_area = centered_rect_with_length(40, 10, rect.size());

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

        if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &popup_area) {
            app.state.mouse_focus = Some(Focus::ChangeViewPopup);
            app.state.set_focus(Focus::ChangeViewPopup);
            calculate_mouse_list_select_index(
                app.state.current_mouse_coordinates.1,
                &all_views,
                popup_area,
                &mut app.state.app_list_states.default_view,
            );
        }
        let views = List::new(all_views)
            .block(
                Block::default()
                    .title("Change View")
                    .style(general_style)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(list_select_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);

        render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);
        rect.render_stateful_widget(
            views,
            popup_area,
            &mut app.state.app_list_states.default_view,
        );
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active);
        }
    }
}
