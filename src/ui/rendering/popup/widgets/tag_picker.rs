use crate::{
    app::App,
    constants::{LIST_SELECTED_SYMBOL, TAG_SELECTOR_HEIGHT, TAG_SELECTOR_WIDTH},
    ui::{
        rendering::{
            common::render_blank_styled_canvas, popup::widgets::TagPicker,
            utils::check_if_active_and_get_style,
        },
        widgets::SelfViewportCorrection,
        Renderable,
    },
};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

impl Renderable for TagPicker {
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

        let available_tags = app
            .widgets
            .tag_picker
            .available_tags
            .iter()
            .map(|tag| ListItem::new(tag.clone()))
            .collect::<Vec<ListItem>>();

        let anchor = app
            .widgets
            .tag_picker
            .viewport_corrected_anchor
            .unwrap_or_default();
        let render_area = Rect {
            x: anchor.0,
            y: anchor.1,
            width: TAG_SELECTOR_WIDTH,
            height: TAG_SELECTOR_HEIGHT.min((app.widgets.tag_picker.available_tags.len() + 2) as u16),
        };
        app.widgets
            .tag_picker
            .set_current_viewport(Some(rect.area()));

        let tag_picker = List::new(available_tags)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Tags")
                    .title_style(general_style),
            )
            .style(general_style)
            .highlight_style(list_select_style)
            .highlight_symbol(LIST_SELECTED_SYMBOL);

        render_blank_styled_canvas(rect, &app.current_theme, render_area, is_active);
        rect.render_stateful_widget(
            tag_picker,
            render_area,
            &mut app.state.app_list_states.tag_picker,
        );
    }
}
