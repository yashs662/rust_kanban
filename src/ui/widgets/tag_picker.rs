use crate::{
    app::{state::Focus, App},
    constants::{TAG_SELECTOR_HEIGHT, TAG_SELECTOR_WIDTH},
    ui::{
        widgets::{SelfViewportCorrection, Widget},
        PopUp,
    },
};
use ratatui::layout::Rect;

#[derive(Debug, Default)]
pub struct TagPickerWidget {
    pub anchor: Option<(u16, u16)>,
    last_anchor: Option<(u16, u16)>,
    pub viewport_corrected_anchor: Option<(u16, u16)>,
    pub current_viewport: Option<Rect>,
    pub last_corrected_viewport: Option<Rect>,
    pub available_tags: Vec<String>,
    pub last_search_query: String,
}

impl Widget for TagPickerWidget {
    fn update(app: &mut App) {
        if !app.state.z_stack.contains(&PopUp::TagPicker) {
            return;
        }

        app.widgets.tag_picker.self_correct(
            TAG_SELECTOR_HEIGHT.min((app.widgets.tag_picker.available_tags.len() + 2) as u16),
            TAG_SELECTOR_WIDTH,
        );

        if app.state.focus != Focus::CardTags && app.state.z_stack.last() == Some(&PopUp::TagPicker)
        {
            app.close_popup();
        }

        if app.state.card_being_edited.is_some() {
            if let Some(selected_tag) = app.state.app_list_states.card_view_tag_list.selected() {
                if app.state.z_stack.last() != Some(&PopUp::TagPicker)
                    && !app.widgets.tag_picker.available_tags.is_empty()
                {
                    app.set_popup(PopUp::TagPicker);
                }

                let card = &app.state.card_being_edited.as_ref().unwrap().1;
                if let Some(search_query) = card.tags.get(selected_tag) {
                    let search_query = search_query.to_lowercase();
                    if app.widgets.tag_picker.last_search_query == search_query {
                        return;
                    }

                    app.widgets.tag_picker.last_search_query = search_query.clone();
                    let all_tags = app.calculate_tags();
                    log::debug!("All tags: {:?}", all_tags);
                    let mut filtered_tags = all_tags
                        .iter()
                        .map(|(tag, _)| (tag.clone(), tag.to_lowercase()))
                        .filter(|(_, tag_lower)| {
                            tag_lower.starts_with(&search_query.to_lowercase())
                        })
                        .collect::<std::collections::HashSet<(String, String)>>()
                        .into_iter()
                        .collect::<Vec<(String, String)>>();

                    filtered_tags.sort_by(|(_, a_lower), (_, b_lower)| {
                        let a_starts_with = a_lower.starts_with(&search_query.to_lowercase());
                        let b_starts_with = b_lower.starts_with(&search_query.to_lowercase());

                        if a_starts_with && !b_starts_with {
                            std::cmp::Ordering::Less
                        } else if !a_starts_with && b_starts_with {
                            std::cmp::Ordering::Greater
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });

                    // remove any tags that are already in the card
                    filtered_tags.retain(|(_, tag_lower)| {
                        !card
                            .tags
                            .iter()
                            .any(|card_tag| card_tag.to_lowercase() == *tag_lower)
                    });

                    // Keep only the first 6 tags and retain the original case
                    app.widgets.tag_picker.available_tags = filtered_tags
                        .into_iter()
                        .map(|(tag, _)| tag)
                        .take(6)
                        .collect();

                    log::debug!(
                        "Available tags: {:?}",
                        app.widgets.tag_picker.available_tags
                    );

                    if app.widgets.tag_picker.available_tags.is_empty() {
                        app.state.app_list_states.tag_picker.select(None);
                        if app.state.z_stack.last() == Some(&PopUp::TagPicker) {
                            app.close_popup();
                        }
                    } else if app.state.app_list_states.tag_picker.selected().is_none()
                        || app.state.app_list_states.tag_picker.selected().unwrap()
                            >= app.widgets.tag_picker.available_tags.len()
                    {
                        app.state.app_list_states.tag_picker.select(Some(0));
                    }
                }
            }
        }
    }
}

impl SelfViewportCorrection for TagPickerWidget {
    fn get_anchor(&self) -> Option<(u16, u16)> {
        self.anchor
    }
    fn get_last_anchor(&self) -> Option<(u16, u16)> {
        self.last_anchor
    }
    fn get_viewport_corrected_anchor(&self) -> Option<(u16, u16)> {
        self.viewport_corrected_anchor
    }
    fn get_current_viewport(&self) -> Option<Rect> {
        self.current_viewport
    }
    fn get_last_corrected_viewport(&self) -> Option<Rect> {
        self.last_corrected_viewport
    }
    fn set_anchor(&mut self, anchor: Option<(u16, u16)>) {
        self.set_last_anchor(self.anchor);
        self.anchor = anchor;
    }
    fn set_last_anchor(&mut self, anchor: Option<(u16, u16)>) {
        self.last_anchor = anchor;
    }
    fn set_viewport_corrected_anchor(&mut self, anchor: Option<(u16, u16)>) {
        self.viewport_corrected_anchor = anchor;
    }
    fn set_current_viewport(&mut self, viewport: Option<Rect>) {
        self.current_viewport = viewport;
    }
    fn set_last_corrected_viewport(&mut self, viewport: Option<Rect>) {
        self.last_corrected_viewport = viewport;
    }
}
