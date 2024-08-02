use crate::{
    app::{
        state::{Focus, KeyBindingEnum},
        App,
    },
    constants::LIST_SELECTED_SYMBOL,
    ui::{
        rendering::{
            common::{render_blank_styled_canvas, render_close_button},
            popup::EditThemeStyle,
            utils::{
                centered_rect_with_percentage, check_if_active_and_get_style,
                check_if_mouse_is_in_area, get_mouse_focusable_field_style,
            },
        },
        theme::{Theme, ThemeEnum},
        Renderable, TextColorOptions, TextModifierOptions,
    },
    util::parse_hex_to_rgb,
};
use log::debug;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};
use strum::IntoEnumIterator;

impl Renderable for EditThemeStyle {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool) {
        let popup_area = centered_rect_with_percentage(90, 80, rect.size());
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Length(4),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .margin(1)
            .split(popup_area);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                    Constraint::Fill(1),
                ]
                .as_ref(),
            )
            .split(main_chunks[0]);

        fn set_foreground_color(app: &App, style: &mut Style) {
            if let Some(fg_selected) = app.state.app_list_states.edit_specific_style[0].selected() {
                if let Some(fg_color) = TextColorOptions::iter().nth(fg_selected).map(Color::from) {
                    if let Color::Rgb(_, _, _) = fg_color {
                        let user_input = app
                            .state
                            .text_buffers
                            .theme_editor_fg_hex
                            .get_joined_lines();
                        let parsed_hex = parse_hex_to_rgb(&user_input);
                        if let Some((r, g, b)) = parsed_hex {
                            style.fg = Some(Color::Rgb(r, g, b));
                            return;
                        }
                    }
                    style.fg = Some(fg_color);
                }
            }
        }

        fn set_background_color(app: &mut App, style: &mut Style) {
            if let Some(bg_selected) = app.state.app_list_states.edit_specific_style[1].selected() {
                if let Some(bg_color) = TextColorOptions::iter().nth(bg_selected).map(Color::from) {
                    if let Color::Rgb(_, _, _) = bg_color {
                        let user_input = app
                            .state
                            .text_buffers
                            .theme_editor_bg_hex
                            .get_joined_lines();
                        let parsed_hex = parse_hex_to_rgb(&user_input);
                        if let Some((r, g, b)) = parsed_hex {
                            style.bg = Some(Color::Rgb(r, g, b));
                            return;
                        }
                    }
                    style.bg = Some(bg_color);
                }
            }
        }

        fn set_text_modifier(app: &mut App, style: &mut Style) {
            if let Some(modifier) = app.state.app_list_states.edit_specific_style[2].selected() {
                if let Some(modifier) = TextModifierOptions::iter()
                    .nth(modifier)
                    .map(ratatui::style::Modifier::from)
                {
                    Theme::add_modifier_to_style(style, modifier);
                }
            }
        }

        fn create_list_item_from_color<'a>(
            color: TextColorOptions,
            style: Style,
            app: &mut App,
            is_active: bool,
        ) -> ListItem<'a> {
            let text_style = check_if_active_and_get_style(
                is_active,
                app.current_theme.inactive_text_style,
                style,
            );
            let general_style = check_if_active_and_get_style(
                is_active,
                app.current_theme.inactive_text_style,
                app.current_theme.general_style,
            );
            ListItem::new(vec![Line::from(vec![
                Span::styled("Sample Text", text_style),
                Span::styled(format!(" - {}", color), general_style),
            ])])
        }

        fn handle_custom_hex_input<'a>(
            hex_value: String,
            mut style: Style,
            app: &mut App,
            is_active: bool,
        ) -> Option<ListItem<'a>> {
            if let Some((red_channel, green_channel, blue_channel)) = parse_hex_to_rgb(&hex_value) {
                let color = TextColorOptions::HEX(red_channel, green_channel, blue_channel);
                style.fg = Some(Color::from(color));
                Some(create_list_item_from_color(color, style, app, is_active))
            } else {
                None
            }
        }

        let general_style = check_if_active_and_get_style(
            is_active,
            app.current_theme.inactive_text_style,
            app.current_theme.general_style,
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
        // TODO: Generalize this;
        // Exception to not using get_button_style as we have to manage other state
        let fg_list_border_style = if !is_active {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[0]) {
            if app.state.app_list_states.edit_specific_style[0]
                .selected()
                .is_none()
            {
                app.state.app_list_states.edit_specific_style[0].select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorFG);
            app.state.set_focus(Focus::StyleEditorFG);
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorFG {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
        // Exception to not using get_button_style as we have to manage other state
        let bg_list_border_style = if !is_active {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[1]) {
            if app.state.app_list_states.edit_specific_style[1]
                .selected()
                .is_none()
            {
                app.state.app_list_states.edit_specific_style[1].select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorBG);
            app.state.set_focus(Focus::StyleEditorBG);
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorBG {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
        // Exception to not using get_button_style as we have to manage other state
        let modifiers_list_border_style = if !is_active {
            app.current_theme.inactive_text_style
        } else if check_if_mouse_is_in_area(&app.state.current_mouse_coordinates, &chunks[2]) {
            if app.state.app_list_states.edit_specific_style[2]
                .selected()
                .is_none()
            {
                app.state.app_list_states.edit_specific_style[2].select(Some(0));
            }
            app.state.mouse_focus = Some(Focus::StyleEditorModifier);
            app.state.set_focus(Focus::StyleEditorModifier);
            app.current_theme.mouse_focus_style
        } else if app.state.focus == Focus::StyleEditorModifier {
            app.current_theme.keyboard_focus_style
        } else {
            app.current_theme.general_style
        };
        let submit_button_style = get_mouse_focusable_field_style(
            app,
            Focus::SubmitButton,
            &main_chunks[1],
            is_active,
            false,
        );
        let fg_list_items: Vec<ListItem> = TextColorOptions::iter()
            .map(|color| {
                let mut fg_style = Style::default();
                let current_user_input = app
                    .state
                    .text_buffers
                    .theme_editor_fg_hex
                    .get_joined_lines();

                set_background_color(app, &mut fg_style);
                set_text_modifier(app, &mut fg_style);

                if let TextColorOptions::HEX(_, _, _) = color {
                    if current_user_input.is_empty() {
                        fg_style.fg = Some(Color::Rgb(0, 0, 0));
                        return create_list_item_from_color(
                            TextColorOptions::HEX(0, 0, 0),
                            fg_style,
                            app,
                            is_active,
                        );
                    } else if let Some(list_item) =
                        handle_custom_hex_input(current_user_input, fg_style, app, is_active)
                    {
                        return list_item;
                    }
                }
                fg_style.fg = Some(Color::from(color));
                create_list_item_from_color(color, fg_style, app, is_active)
            })
            .collect();

        let bg_list_items: Vec<ListItem> = TextColorOptions::iter()
            .map(|color| {
                let mut bg_style = Style::default();
                let current_user_input = app
                    .state
                    .text_buffers
                    .theme_editor_bg_hex
                    .get_joined_lines();

                set_foreground_color(app, &mut bg_style);
                set_text_modifier(app, &mut bg_style);

                if let TextColorOptions::HEX(_, _, _) = color {
                    if current_user_input.is_empty() {
                        bg_style.bg = Some(Color::Rgb(0, 0, 0));
                        return create_list_item_from_color(
                            TextColorOptions::HEX(0, 0, 0),
                            bg_style,
                            app,
                            is_active,
                        );
                    } else if let Some(list_item) =
                        handle_custom_hex_input(current_user_input, bg_style, app, is_active)
                    {
                        return list_item;
                    }
                }
                bg_style.bg = Some(Color::from(color));
                create_list_item_from_color(color, bg_style, app, is_active)
            })
            .collect();

        let modifier_list_items: Vec<ListItem> = TextModifierOptions::iter()
            .map(|modifier| {
                let mut modifier_style = general_style;
                set_foreground_color(app, &mut modifier_style);
                set_background_color(app, &mut modifier_style);

                Theme::add_modifier_to_style(
                    &mut modifier_style,
                    ratatui::style::Modifier::from(modifier.clone()),
                );
                ListItem::new(vec![Line::from(vec![
                    Span::styled("Sample Text", modifier_style),
                    Span::styled(format!(" - {}", modifier), general_style),
                ])])
            })
            .collect();

        let fg_list = if is_active {
            List::new(fg_list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Foreground")
                        .border_style(fg_list_border_style),
                )
                .highlight_symbol(LIST_SELECTED_SYMBOL)
        } else {
            List::new(fg_list_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Foreground")
                    .border_style(fg_list_border_style),
            )
        };

        let bg_list = if is_active {
            List::new(bg_list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Background")
                        .border_style(bg_list_border_style),
                )
                .highlight_symbol(LIST_SELECTED_SYMBOL)
        } else {
            List::new(bg_list_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Background")
                    .border_style(bg_list_border_style),
            )
        };

        let modifier_list = if is_active {
            List::new(modifier_list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title("Modifiers")
                        .border_style(modifiers_list_border_style),
                )
                .highlight_symbol(LIST_SELECTED_SYMBOL)
        } else {
            List::new(modifier_list_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("Modifiers")
                    .border_style(modifiers_list_border_style),
            )
        };

        let theme_style_being_edited =
            if let Some(index) = app.state.app_table_states.theme_editor.selected() {
                if let Some(theme_enum) = ThemeEnum::iter().nth(index) {
                    theme_enum.to_string()
                } else {
                    debug!("Index is out of bounds for theme_style_being_edited");
                    "None".to_string()
                }
            } else {
                "None".to_string()
            };
        let border_block = Block::default()
            .title(format!("Editing Style: {}", theme_style_being_edited))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(general_style);

        let submit_button = Paragraph::new("Confirm Changes")
            .style(submit_button_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(submit_button_style),
            )
            .alignment(Alignment::Center);

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

        let help_spans = vec![
            Span::styled("Use ", help_text_style),
            Span::styled(up_key, help_key_style),
            Span::styled(" and ", help_text_style),
            Span::styled(down_key, help_key_style),
            Span::styled(" or scroll with the mouse", help_text_style),
            Span::styled(" to select a Color/Modifier, Press ", help_text_style),
            Span::styled(accept_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled("<Mouse Left Click>", help_key_style),
            Span::styled(" to edit (for custom RBG), Press ", help_text_style),
            Span::styled(next_focus_key, help_key_style),
            Span::styled(" or ", help_text_style),
            Span::styled(prv_focus_key, help_key_style),
            Span::styled(" to change focus.", help_text_style),
        ];

        let help_text = Paragraph::new(Line::from(help_spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(help_text_style),
            )
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        render_blank_styled_canvas(rect, &app.current_theme, popup_area, is_active);
        rect.render_stateful_widget(
            fg_list,
            chunks[0],
            &mut app.state.app_list_states.edit_specific_style[0],
        );
        rect.render_stateful_widget(
            bg_list,
            chunks[1],
            &mut app.state.app_list_states.edit_specific_style[1],
        );
        rect.render_stateful_widget(
            modifier_list,
            chunks[2],
            &mut app.state.app_list_states.edit_specific_style[2],
        );
        rect.render_widget(help_text, main_chunks[1]);
        rect.render_widget(submit_button, main_chunks[2]);
        rect.render_widget(border_block, popup_area);
        if app.config.enable_mouse_support {
            render_close_button(rect, app, is_active)
        }
    }
}
