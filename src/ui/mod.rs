use log::debug;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Row},
};
use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    constants::{
        CARD_ACTIVE_STATUS_STYLE, CARD_COMPLETED_STATUS_STYLE, CARD_DUE_DATE_CRITICAL_STYLE,
        CARD_DUE_DATE_DEFAULT_STYLE, CARD_DUE_DATE_WARNING_STYLE, CARD_PRIORITY_HIGH_STYLE,
        CARD_PRIORITY_LOW_STYLE, CARD_PRIORITY_MEDIUM_STYLE, CARD_STALE_STATUS_STYLE,
        ERROR_TEXT_STYLE, GENERAL_STYLE, HELP_KEY_STYLE, INACTIVE_TEXT_STYLE, KEYBOARD_FOCUS_STYLE,
        LIST_SELECT_STYLE, LOG_DEBUG_STYLE, LOG_ERROR_STYLE, LOG_INFO_STYLE, LOG_TRACE_STYLE,
        LOG_WARN_STYLE, MOUSE_HIGHLIGHT_STYLE, PROGRESS_BAR_STYLE, SAMPLE_TEXT,
    },
};
pub mod ui_helper;
pub mod ui_main;
pub mod widgets;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub general_style: Style,
    pub list_select_style: Style,
    pub card_due_default_style: Style,
    pub card_due_warning_style: Style,
    pub card_due_overdue_style: Style,
    pub card_status_active_style: Style,
    pub card_status_completed_style: Style,
    pub card_status_stale_style: Style,
    pub keyboard_focus_style: Style,
    pub mouse_focus_style: Style,
    pub help_key_style: Style,
    pub help_text_style: Style,
    pub log_error_style: Style,
    pub log_debug_style: Style,
    pub log_warn_style: Style,
    pub log_trace_style: Style,
    pub log_info_style: Style,
    pub progress_bar_style: Style,
    pub error_text_style: Style,
    pub inactive_text_style: Style,
    pub card_priority_low_style: Style,
    pub card_priority_medium_style: Style,
    pub card_priority_high_style: Style,
}

// implement the themetype enum to return the theme struct
impl Theme {
    pub fn default() -> Theme {
        Theme {
            name: "Default Theme".to_string(),
            general_style: GENERAL_STYLE,
            list_select_style: LIST_SELECT_STYLE,
            card_due_default_style: CARD_DUE_DATE_DEFAULT_STYLE,
            card_due_warning_style: CARD_DUE_DATE_WARNING_STYLE,
            card_due_overdue_style: CARD_DUE_DATE_CRITICAL_STYLE,
            card_status_active_style: CARD_ACTIVE_STATUS_STYLE,
            card_status_completed_style: CARD_COMPLETED_STATUS_STYLE,
            card_status_stale_style: CARD_STALE_STATUS_STYLE,
            keyboard_focus_style: KEYBOARD_FOCUS_STYLE,
            mouse_focus_style: MOUSE_HIGHLIGHT_STYLE,
            help_key_style: HELP_KEY_STYLE,
            help_text_style: GENERAL_STYLE,
            log_error_style: LOG_ERROR_STYLE,
            log_debug_style: LOG_DEBUG_STYLE,
            log_warn_style: LOG_WARN_STYLE,
            log_trace_style: LOG_TRACE_STYLE,
            log_info_style: LOG_INFO_STYLE,
            progress_bar_style: PROGRESS_BAR_STYLE,
            error_text_style: ERROR_TEXT_STYLE,
            inactive_text_style: INACTIVE_TEXT_STYLE,
            card_priority_low_style: CARD_PRIORITY_LOW_STYLE,
            card_priority_medium_style: CARD_PRIORITY_MEDIUM_STYLE,
            card_priority_high_style: CARD_PRIORITY_HIGH_STYLE,
        }
    }
    fn midnight_blue() -> Theme {
        Theme {
            name: "Midnight Blue".to_string(),
            general_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
            list_select_style: Style::default()
                .fg(Color::Gray)
                .bg(Color::Rgb(70, 130, 180)),
            card_due_default_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
            card_due_warning_style: Style::default()
                .fg(Color::LightYellow)
                .bg(Color::Rgb(25, 25, 112)),
            card_due_overdue_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(25, 25, 112)),
            card_status_active_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(25, 25, 112)),
            card_status_completed_style: Style::default()
                .fg(Color::Gray)
                .bg(Color::Rgb(25, 25, 112)),
            card_status_stale_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(25, 25, 112)),
            keyboard_focus_style: Style::default()
                .fg(Color::LightBlue)
                .bg(Color::Rgb(25, 25, 112))
                .add_modifier(Modifier::BOLD),
            mouse_focus_style: Style::default()
                .fg(Color::LightBlue)
                .bg(Color::Rgb(25, 25, 112))
                .add_modifier(Modifier::BOLD),
            help_key_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
            help_text_style: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(25, 25, 112)),
            log_error_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(25, 25, 112)),
            log_debug_style: Style::default()
                .fg(Color::LightBlue)
                .bg(Color::Rgb(25, 25, 112)),
            log_warn_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(25, 25, 112)),
            log_trace_style: Style::default()
                .fg(Color::LightCyan)
                .bg(Color::Rgb(25, 25, 112)),
            log_info_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(25, 25, 112)),
            progress_bar_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(25, 25, 112)),
            error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
            inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
            card_priority_low_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(25, 25, 112)),
            card_priority_medium_style: Style::default()
                .fg(Color::LightYellow)
                .bg(Color::Rgb(25, 25, 112)),
            card_priority_high_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(25, 25, 112)),
        }
    }
    fn dark_slate() -> Theme {
        Theme {
            name: "Dark Slate".to_string(),
            general_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
            list_select_style: Style::default()
                .fg(Color::Gray)
                .bg(Color::Rgb(70, 130, 180)),
            card_due_default_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
            card_due_warning_style: Style::default()
                .fg(Color::LightYellow)
                .bg(Color::Rgb(47, 79, 79)),
            card_due_overdue_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(47, 79, 79)),
            card_status_active_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(47, 79, 79)),
            card_status_completed_style: Style::default()
                .fg(Color::Gray)
                .bg(Color::Rgb(47, 79, 79)),
            card_status_stale_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(47, 79, 79)),
            keyboard_focus_style: Style::default()
                .fg(Color::LightCyan)
                .bg(Color::Rgb(47, 79, 79))
                .add_modifier(Modifier::BOLD),
            mouse_focus_style: Style::default()
                .fg(Color::LightCyan)
                .bg(Color::Rgb(47, 79, 79))
                .add_modifier(Modifier::BOLD),
            help_key_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
            help_text_style: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(47, 79, 79)),
            log_error_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(47, 79, 79)),
            log_debug_style: Style::default()
                .fg(Color::LightBlue)
                .bg(Color::Rgb(47, 79, 79)),
            log_warn_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(47, 79, 79)),
            log_trace_style: Style::default()
                .fg(Color::LightCyan)
                .bg(Color::Rgb(47, 79, 79)),
            log_info_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(47, 79, 79)),
            progress_bar_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(47, 79, 79)),
            error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
            inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
            card_priority_low_style: Style::default()
                .fg(Color::LightGreen)
                .bg(Color::Rgb(47, 79, 79)),
            card_priority_medium_style: Style::default()
                .fg(Color::LightYellow)
                .bg(Color::Rgb(47, 79, 79)),
            card_priority_high_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(47, 79, 79)),
        }
    }
    fn metro() -> Theme {
        Theme {
            name: "Metro".to_string(),
            general_style: Style::default().fg(Color::Gray).bg(Color::Rgb(20, 20, 20)),
            list_select_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(124, 252, 0)),
            card_due_default_style: Style::default().fg(Color::White).bg(Color::Rgb(25, 25, 25)),
            card_due_warning_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(25, 25, 25)),
            card_due_overdue_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(25, 25, 25)),
            card_status_active_style: Style::default().fg(Color::Cyan).bg(Color::Rgb(25, 25, 25)),
            card_status_completed_style: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(25, 25, 25)),
            card_status_stale_style: Style::default()
                .fg(Color::LightYellow)
                .bg(Color::Rgb(25, 25, 25)),
            keyboard_focus_style: Style::default()
                .fg(Color::Green)
                .bg(Color::Rgb(25, 25, 25))
                .add_modifier(Modifier::BOLD),
            mouse_focus_style: Style::default()
                .fg(Color::Green)
                .bg(Color::Rgb(25, 25, 25))
                .add_modifier(Modifier::BOLD),
            help_key_style: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(25, 25, 25)),
            help_text_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 25)),
            log_error_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(25, 25, 25)),
            log_debug_style: Style::default().fg(Color::Cyan).bg(Color::Rgb(25, 25, 25)),
            log_warn_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(25, 25, 25)),
            log_trace_style: Style::default().fg(Color::Green).bg(Color::Rgb(25, 25, 25)),
            log_info_style: Style::default().fg(Color::White).bg(Color::Rgb(25, 25, 25)),
            progress_bar_style: Style::default().fg(Color::Green).bg(Color::Rgb(25, 25, 25)),
            error_text_style: Style::default()
                .fg(Color::LightRed)
                .bg(Color::Rgb(25, 25, 25)),
            inactive_text_style: Style::default()
                .fg(Color::DarkGray)
                .bg(Color::Rgb(25, 25, 25)),
            card_priority_low_style: Style::default().fg(Color::Green).bg(Color::Rgb(25, 25, 25)),
            card_priority_medium_style: Style::default()
                .fg(Color::Yellow)
                .bg(Color::Rgb(25, 25, 25)),
            card_priority_high_style: Style::default().fg(Color::Red).bg(Color::Rgb(25, 25, 25)),
        }
    }
    fn matrix() -> Theme {
        Theme {
            name: "Matrix".to_string(),
            general_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            list_select_style: Style::default().fg(Color::Black).bg(Color::LightGreen),
            card_due_default_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            card_due_warning_style: Style::default().fg(Color::Yellow).bg(Color::Black),
            card_due_overdue_style: Style::default().fg(Color::LightRed).bg(Color::Black),
            card_status_active_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            card_status_completed_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
            card_status_stale_style: Style::default().fg(Color::Yellow).bg(Color::Black),
            keyboard_focus_style: Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            mouse_focus_style: Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
            help_key_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            help_text_style: Style::default().fg(Color::Green).bg(Color::Black),
            log_error_style: Style::default().fg(Color::LightRed).bg(Color::Black),
            log_debug_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            log_warn_style: Style::default().fg(Color::Yellow).bg(Color::Black),
            log_trace_style: Style::default().fg(Color::LightCyan).bg(Color::Black),
            log_info_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            progress_bar_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
            inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
            card_priority_low_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
            card_priority_medium_style: Style::default().fg(Color::Yellow).bg(Color::Black),
            card_priority_high_style: Style::default().fg(Color::LightRed).bg(Color::Black),
        }
    }
    fn cyberpunk() -> Theme {
        Theme {
            name: "Cyberpunk".to_string(),
            general_style: Style::default()
                .fg(Color::Rgb(248, 12, 228))
                .bg(Color::Black),
            list_select_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(253, 248, 0)),
            card_due_default_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
            card_due_warning_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black),
            card_due_overdue_style: Style::default()
                .fg(Color::Rgb(255, 28, 92))
                .bg(Color::Black),
            card_status_active_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
            card_status_completed_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
            card_status_stale_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black),
            keyboard_focus_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            mouse_focus_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black)
                .add_modifier(Modifier::BOLD),
            help_key_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
            help_text_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black),
            log_error_style: Style::default()
                .fg(Color::Rgb(255, 28, 92))
                .bg(Color::Black),
            log_debug_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
            log_warn_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black),
            log_trace_style: Style::default().fg(Color::LightCyan).bg(Color::Black),
            log_info_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
            progress_bar_style: Style::default()
                .fg(Color::Rgb(248, 12, 228))
                .bg(Color::Black),
            error_text_style: Style::default()
                .fg(Color::Black)
                .bg(Color::Rgb(255, 28, 92)),
            inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
            card_priority_low_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
            card_priority_medium_style: Style::default()
                .fg(Color::Rgb(253, 248, 0))
                .bg(Color::Black),
            card_priority_high_style: Style::default()
                .fg(Color::Rgb(255, 28, 92))
                .bg(Color::Black),
        }
    }
    pub fn all_default_themes() -> Vec<Theme> {
        vec![
            Theme::default(),
            Theme::midnight_blue(),
            Theme::dark_slate(),
            Theme::metro(),
            Theme::matrix(),
            Theme::cyberpunk(),
        ]
    }

    pub fn to_rows(&self, app: &App) -> (Vec<Row>, Vec<Row>) {
        let popup_mode = app.state.popup_mode.is_some();
        let text_style = if popup_mode {
            self.inactive_text_style
        } else {
            self.general_style
        };
        let theme_title_list = vec![
            Span::styled("Name: ", text_style),
            Span::styled("General Style: ", text_style),
            Span::styled("List Select Style: ", text_style),
            Span::styled("Card Due Default Style: ", text_style),
            Span::styled("Card Due Warning Style: ", text_style),
            Span::styled("Card Due Overdue Style: ", text_style),
            Span::styled("Card Status Active Style: ", text_style),
            Span::styled("Card Status Completed Style: ", text_style),
            Span::styled("Card Status Stale Style: ", text_style),
            Span::styled("Keyboard Focus Style: ", text_style),
            Span::styled("Mouse Focus Style: ", text_style),
            Span::styled("Help Key Style: ", text_style),
            Span::styled("Help Text Style: ", text_style),
            Span::styled("Log Error Style: ", text_style),
            Span::styled("Log Debug Style: ", text_style),
            Span::styled("Log Warn Style: ", text_style),
            Span::styled("Log Trace Style: ", text_style),
            Span::styled("Log Info Style: ", text_style),
            Span::styled("Progress Bar Style: ", text_style),
            Span::styled("Error Text Style: ", text_style),
            Span::styled("Inactive Text Style: ", text_style),
            Span::styled("Card Priority Low Style: ", text_style),
            Span::styled("Card Priority Medium Style: ", text_style),
            Span::styled("Card Priority High Style: ", text_style),
        ];
        let theme_style_list = if popup_mode {
            vec![
                Span::styled(&self.name, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
            ]
        } else {
            vec![
                Span::styled(&self.name, self.general_style),
                Span::styled(SAMPLE_TEXT, self.general_style),
                Span::styled(SAMPLE_TEXT, self.list_select_style),
                Span::styled(SAMPLE_TEXT, self.card_due_default_style),
                Span::styled(SAMPLE_TEXT, self.card_due_warning_style),
                Span::styled(SAMPLE_TEXT, self.card_due_overdue_style),
                Span::styled(SAMPLE_TEXT, self.card_status_active_style),
                Span::styled(SAMPLE_TEXT, self.card_status_completed_style),
                Span::styled(SAMPLE_TEXT, self.card_status_stale_style),
                Span::styled(SAMPLE_TEXT, self.keyboard_focus_style),
                Span::styled(SAMPLE_TEXT, self.mouse_focus_style),
                Span::styled(SAMPLE_TEXT, self.help_key_style),
                Span::styled(SAMPLE_TEXT, self.help_text_style),
                Span::styled(SAMPLE_TEXT, self.log_error_style),
                Span::styled(SAMPLE_TEXT, self.log_debug_style),
                Span::styled(SAMPLE_TEXT, self.log_warn_style),
                Span::styled(SAMPLE_TEXT, self.log_trace_style),
                Span::styled(SAMPLE_TEXT, self.log_info_style),
                Span::styled(SAMPLE_TEXT, self.progress_bar_style),
                Span::styled(SAMPLE_TEXT, self.error_text_style),
                Span::styled(SAMPLE_TEXT, self.inactive_text_style),
                Span::styled(SAMPLE_TEXT, self.card_priority_low_style),
                Span::styled(SAMPLE_TEXT, self.card_priority_medium_style),
                Span::styled(SAMPLE_TEXT, self.card_priority_high_style),
            ]
        };
        let rows_title = theme_title_list
            .iter()
            .map(|row| Row::new(vec![Cell::from(row.clone())]))
            .collect::<Vec<Row>>();
        let rows_elements = theme_style_list
            .iter()
            .map(|row| Row::new(vec![Cell::from(row.clone())]))
            .collect::<Vec<Row>>();
        (rows_title, rows_elements)
    }

    pub fn to_vec_str(&self) -> Vec<&str> {
        vec![
            "name",
            "general_style",
            "list_select_style",
            "card_due_default_style",
            "card_due_warning_style",
            "card_due_overdue_style",
            "card_status_active_style",
            "card_status_completed_style",
            "card_status_stale_style",
            "keyboard_focus_style",
            "mouse_focus_style",
            "help_key_style",
            "help_text_style",
            "log_error_style",
            "log_debug_style",
            "log_warn_style",
            "log_trace_style",
            "log_info_style",
            "progress_bar_style",
            "error_text_style",
            "inactive_text_style",
            "card_priority_low_style",
            "card_priority_medium_style",
            "card_priority_high_style",
        ]
    }

    pub fn edit_style(
        &self,
        style_being_edited: &str,
        fg_color: Option<Color>,
        bg_color: Option<Color>,
        modifier: Option<Modifier>,
    ) -> Self {
        let mut theme = self.clone();
        match style_being_edited {
            "name" => debug!("Cannot edit name"),
            "general_style" => {
                if fg_color.is_some() {
                    theme.general_style = theme.general_style.fg(fg_color.unwrap());
                } else {
                    theme.general_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.general_style = theme.general_style.bg(bg_color.unwrap());
                } else {
                    theme.general_style.bg = None;
                }
                if modifier.is_some() {
                    theme.general_style = theme.general_style.add_modifier(modifier.unwrap());
                } else {
                    theme.general_style.sub_modifier = Modifier::empty();
                    theme.general_style.add_modifier = Modifier::empty();
                }
            }
            "list_select_style" => {
                if fg_color.is_some() {
                    theme.list_select_style = theme.list_select_style.fg(fg_color.unwrap());
                } else {
                    theme.list_select_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.list_select_style = theme.list_select_style.bg(bg_color.unwrap());
                } else {
                    theme.list_select_style.bg = None;
                }
                if modifier.is_some() {
                    theme.list_select_style =
                        theme.list_select_style.add_modifier(modifier.unwrap());
                } else {
                    theme.list_select_style.sub_modifier = Modifier::empty();
                    theme.list_select_style.add_modifier = Modifier::empty();
                }
            }
            "card_due_default_style" => {
                if fg_color.is_some() {
                    theme.card_due_default_style =
                        theme.card_due_default_style.fg(fg_color.unwrap());
                } else {
                    theme.card_due_default_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_due_default_style =
                        theme.card_due_default_style.bg(bg_color.unwrap());
                } else {
                    theme.card_due_default_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_due_default_style =
                        theme.card_due_default_style.add_modifier(modifier.unwrap());
                } else {
                    theme.card_due_default_style.sub_modifier = Modifier::empty();
                    theme.card_due_default_style.add_modifier = Modifier::empty();
                }
            }
            "card_due_warning_style" => {
                if fg_color.is_some() {
                    theme.card_due_warning_style =
                        theme.card_due_warning_style.fg(fg_color.unwrap());
                } else {
                    theme.card_due_warning_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_due_warning_style =
                        theme.card_due_warning_style.bg(bg_color.unwrap());
                } else {
                    theme.card_due_warning_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_due_warning_style =
                        theme.card_due_warning_style.add_modifier(modifier.unwrap());
                } else {
                    theme.card_due_warning_style.sub_modifier = Modifier::empty();
                    theme.card_due_warning_style.add_modifier = Modifier::empty();
                }
            }
            "card_due_overdue_style" => {
                if fg_color.is_some() {
                    theme.card_due_overdue_style =
                        theme.card_due_overdue_style.fg(fg_color.unwrap());
                } else {
                    theme.card_due_overdue_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_due_overdue_style =
                        theme.card_due_overdue_style.bg(bg_color.unwrap());
                } else {
                    theme.card_due_overdue_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_due_overdue_style =
                        theme.card_due_overdue_style.add_modifier(modifier.unwrap());
                } else {
                    theme.card_due_overdue_style.sub_modifier = Modifier::empty();
                    theme.card_due_overdue_style.add_modifier = Modifier::empty();
                }
            }
            "card_status_active_style" => {
                if fg_color.is_some() {
                    theme.card_status_active_style =
                        theme.card_status_active_style.fg(fg_color.unwrap());
                } else {
                    theme.card_status_active_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_status_active_style =
                        theme.card_status_active_style.bg(bg_color.unwrap());
                } else {
                    theme.card_status_active_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_status_active_style = theme
                        .card_status_active_style
                        .add_modifier(modifier.unwrap());
                } else {
                    theme.card_status_active_style.sub_modifier = Modifier::empty();
                    theme.card_status_active_style.add_modifier = Modifier::empty();
                }
            }
            "card_status_completed_style" => {
                if fg_color.is_some() {
                    theme.card_status_completed_style =
                        theme.card_status_completed_style.fg(fg_color.unwrap());
                } else {
                    theme.card_status_completed_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_status_completed_style =
                        theme.card_status_completed_style.bg(bg_color.unwrap());
                } else {
                    theme.card_status_completed_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_status_completed_style = theme
                        .card_status_completed_style
                        .add_modifier(modifier.unwrap());
                } else {
                    theme.card_status_completed_style.sub_modifier = Modifier::empty();
                    theme.card_status_completed_style.add_modifier = Modifier::empty();
                }
            }
            "card_status_stale_style" => {
                if fg_color.is_some() {
                    theme.card_status_stale_style =
                        theme.card_status_stale_style.fg(fg_color.unwrap());
                } else {
                    theme.card_status_stale_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_status_stale_style =
                        theme.card_status_stale_style.bg(bg_color.unwrap());
                } else {
                    theme.card_status_stale_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_status_stale_style = theme
                        .card_status_stale_style
                        .add_modifier(modifier.unwrap());
                } else {
                    theme.card_status_stale_style.sub_modifier = Modifier::empty();
                    theme.card_status_stale_style.add_modifier = Modifier::empty();
                }
            }
            "keyboard_focus_style" => {
                if fg_color.is_some() {
                    theme.keyboard_focus_style = theme.keyboard_focus_style.fg(fg_color.unwrap());
                } else {
                    theme.keyboard_focus_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.keyboard_focus_style = theme.keyboard_focus_style.bg(bg_color.unwrap());
                } else {
                    theme.keyboard_focus_style.bg = None;
                }
                if modifier.is_some() {
                    theme.keyboard_focus_style =
                        theme.keyboard_focus_style.add_modifier(modifier.unwrap());
                } else {
                    theme.keyboard_focus_style.sub_modifier = Modifier::empty();
                    theme.keyboard_focus_style.add_modifier = Modifier::empty();
                }
            }
            "mouse_focus_style" => {
                if fg_color.is_some() {
                    theme.mouse_focus_style = theme.mouse_focus_style.fg(fg_color.unwrap());
                } else {
                    theme.mouse_focus_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.mouse_focus_style = theme.mouse_focus_style.bg(bg_color.unwrap());
                } else {
                    theme.mouse_focus_style.bg = None;
                }
                if modifier.is_some() {
                    theme.mouse_focus_style =
                        theme.mouse_focus_style.add_modifier(modifier.unwrap());
                } else {
                    theme.mouse_focus_style.sub_modifier = Modifier::empty();
                    theme.mouse_focus_style.add_modifier = Modifier::empty();
                }
            }
            "help_key_style" => {
                if fg_color.is_some() {
                    theme.help_key_style = theme.help_key_style.fg(fg_color.unwrap());
                } else {
                    theme.help_key_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.help_key_style = theme.help_key_style.bg(bg_color.unwrap());
                } else {
                    theme.help_key_style.bg = None;
                }
                if modifier.is_some() {
                    theme.help_key_style = theme.help_key_style.add_modifier(modifier.unwrap());
                } else {
                    theme.help_key_style.sub_modifier = Modifier::empty();
                    theme.help_key_style.add_modifier = Modifier::empty();
                }
            }
            "help_text_style" => {
                if fg_color.is_some() {
                    theme.help_text_style = theme.help_text_style.fg(fg_color.unwrap());
                } else {
                    theme.help_text_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.help_text_style = theme.help_text_style.bg(bg_color.unwrap());
                } else {
                    theme.help_text_style.bg = None;
                }
                if modifier.is_some() {
                    theme.help_text_style = theme.help_text_style.add_modifier(modifier.unwrap());
                } else {
                    theme.help_text_style.sub_modifier = Modifier::empty();
                    theme.help_text_style.add_modifier = Modifier::empty();
                }
            }
            "log_error_style" => {
                if fg_color.is_some() {
                    theme.log_error_style = theme.log_error_style.fg(fg_color.unwrap());
                } else {
                    theme.log_error_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.log_error_style = theme.log_error_style.bg(bg_color.unwrap());
                } else {
                    theme.log_error_style.bg = None;
                }
                if modifier.is_some() {
                    theme.log_error_style = theme.log_error_style.add_modifier(modifier.unwrap());
                } else {
                    theme.log_error_style.sub_modifier = Modifier::empty();
                    theme.log_error_style.add_modifier = Modifier::empty();
                }
            }
            "log_debug_style" => {
                if fg_color.is_some() {
                    theme.log_debug_style = theme.log_debug_style.fg(fg_color.unwrap());
                } else {
                    theme.log_debug_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.log_debug_style = theme.log_debug_style.bg(bg_color.unwrap());
                } else {
                    theme.log_debug_style.bg = None;
                }
                if modifier.is_some() {
                    theme.log_debug_style = theme.log_debug_style.add_modifier(modifier.unwrap());
                } else {
                    theme.log_debug_style.sub_modifier = Modifier::empty();
                    theme.log_debug_style.add_modifier = Modifier::empty();
                }
            }
            "log_warn_style" => {
                if fg_color.is_some() {
                    theme.log_warn_style = theme.log_warn_style.fg(fg_color.unwrap());
                } else {
                    theme.log_warn_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.log_warn_style = theme.log_warn_style.bg(bg_color.unwrap());
                } else {
                    theme.log_warn_style.bg = None;
                }
                if modifier.is_some() {
                    theme.log_warn_style = theme.log_warn_style.add_modifier(modifier.unwrap());
                } else {
                    theme.log_warn_style.sub_modifier = Modifier::empty();
                    theme.log_warn_style.add_modifier = Modifier::empty();
                }
            }
            "log_trace_style" => {
                if fg_color.is_some() {
                    theme.log_trace_style = theme.log_trace_style.fg(fg_color.unwrap());
                } else {
                    theme.log_trace_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.log_trace_style = theme.log_trace_style.bg(bg_color.unwrap());
                } else {
                    theme.log_trace_style.bg = None;
                }
                if modifier.is_some() {
                    theme.log_trace_style = theme.log_trace_style.add_modifier(modifier.unwrap());
                } else {
                    theme.log_trace_style.sub_modifier = Modifier::empty();
                    theme.log_trace_style.add_modifier = Modifier::empty();
                }
            }
            "log_info_style" => {
                if fg_color.is_some() {
                    theme.log_info_style = theme.log_info_style.fg(fg_color.unwrap());
                } else {
                    theme.log_info_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.log_info_style = theme.log_info_style.bg(bg_color.unwrap());
                } else {
                    theme.log_info_style.bg = None;
                }
                if modifier.is_some() {
                    theme.log_info_style = theme.log_info_style.add_modifier(modifier.unwrap());
                } else {
                    theme.log_info_style.sub_modifier = Modifier::empty();
                    theme.log_info_style.add_modifier = Modifier::empty();
                }
            }
            "progress_bar_style" => {
                if fg_color.is_some() {
                    theme.progress_bar_style = theme.progress_bar_style.fg(fg_color.unwrap());
                } else {
                    theme.progress_bar_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.progress_bar_style = theme.progress_bar_style.bg(bg_color.unwrap());
                } else {
                    theme.progress_bar_style.bg = None;
                }
                if modifier.is_some() {
                    theme.progress_bar_style =
                        theme.progress_bar_style.add_modifier(modifier.unwrap());
                } else {
                    theme.progress_bar_style.sub_modifier = Modifier::empty();
                    theme.progress_bar_style.add_modifier = Modifier::empty();
                }
            }
            "error_text_style" => {
                if fg_color.is_some() {
                    theme.error_text_style = theme.error_text_style.fg(fg_color.unwrap());
                } else {
                    theme.error_text_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.error_text_style = theme.error_text_style.bg(bg_color.unwrap());
                } else {
                    theme.error_text_style.bg = None;
                }
                if modifier.is_some() {
                    theme.error_text_style = theme.error_text_style.add_modifier(modifier.unwrap());
                } else {
                    theme.error_text_style.sub_modifier = Modifier::empty();
                    theme.error_text_style.add_modifier = Modifier::empty();
                }
            }
            "inactive_text_style" => {
                if fg_color.is_some() {
                    theme.inactive_text_style = theme.inactive_text_style.fg(fg_color.unwrap());
                } else {
                    theme.inactive_text_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.inactive_text_style = theme.inactive_text_style.bg(bg_color.unwrap());
                } else {
                    theme.inactive_text_style.bg = None;
                }
                if modifier.is_some() {
                    theme.inactive_text_style =
                        theme.inactive_text_style.add_modifier(modifier.unwrap());
                } else {
                    theme.inactive_text_style.sub_modifier = Modifier::empty();
                    theme.inactive_text_style.add_modifier = Modifier::empty();
                }
            }
            "card_priority_low_style" => {
                if fg_color.is_some() {
                    theme.card_priority_low_style =
                        theme.card_priority_low_style.fg(fg_color.unwrap());
                } else {
                    theme.card_priority_low_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_priority_low_style =
                        theme.card_priority_low_style.bg(bg_color.unwrap());
                } else {
                    theme.card_priority_low_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_priority_low_style = theme
                        .card_priority_low_style
                        .add_modifier(modifier.unwrap());
                } else {
                    theme.card_priority_low_style.sub_modifier = Modifier::empty();
                    theme.card_priority_low_style.add_modifier = Modifier::empty();
                }
            }
            "card_priority_medium_style" => {
                if fg_color.is_some() {
                    theme.card_priority_medium_style =
                        theme.card_priority_medium_style.fg(fg_color.unwrap());
                } else {
                    theme.card_priority_medium_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_priority_medium_style =
                        theme.card_priority_medium_style.bg(bg_color.unwrap());
                } else {
                    theme.card_priority_medium_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_priority_medium_style = theme
                        .card_priority_medium_style
                        .add_modifier(modifier.unwrap());
                } else {
                    theme.card_priority_medium_style.sub_modifier = Modifier::empty();
                    theme.card_priority_medium_style.add_modifier = Modifier::empty();
                }
            }
            "card_priority_high_style" => {
                if fg_color.is_some() {
                    theme.card_priority_high_style =
                        theme.card_priority_high_style.fg(fg_color.unwrap());
                } else {
                    theme.card_priority_high_style.fg = None;
                }
                if bg_color.is_some() {
                    theme.card_priority_high_style =
                        theme.card_priority_high_style.bg(bg_color.unwrap());
                } else {
                    theme.card_priority_high_style.bg = None;
                }
                if modifier.is_some() {
                    theme.card_priority_high_style = theme
                        .card_priority_high_style
                        .add_modifier(modifier.unwrap());
                } else {
                    theme.card_priority_high_style.sub_modifier = Modifier::empty();
                    theme.card_priority_high_style.add_modifier = Modifier::empty();
                }
            }
            _ => {
                debug!("Style not found: {}", style_being_edited);
            }
        }
        theme
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextColorOptions {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    RGB(u8, u8, u8),
    None,
}

impl TextColorOptions {
    pub fn to_string(&self) -> String {
        match self {
            TextColorOptions::White => "White".to_string(),
            TextColorOptions::Black => "Black".to_string(),
            TextColorOptions::Red => "Red".to_string(),
            TextColorOptions::Green => "Green".to_string(),
            TextColorOptions::Yellow => "Yellow".to_string(),
            TextColorOptions::Blue => "Blue".to_string(),
            TextColorOptions::Magenta => "Magenta".to_string(),
            TextColorOptions::Cyan => "Cyan".to_string(),
            TextColorOptions::Gray => "Gray".to_string(),
            TextColorOptions::DarkGray => "DarkGray".to_string(),
            TextColorOptions::LightRed => "LightRed".to_string(),
            TextColorOptions::LightGreen => "LightGreen".to_string(),
            TextColorOptions::LightYellow => "LightYellow".to_string(),
            TextColorOptions::LightBlue => "LightBlue".to_string(),
            TextColorOptions::LightMagenta => "LightMagenta".to_string(),
            TextColorOptions::LightCyan => "LightCyan".to_string(),
            TextColorOptions::RGB(r, g, b) => format!("RGB({}, {}, {})", r, g, b),
            TextColorOptions::None => "None".to_string(),
        }
    }
    pub fn to_color(&self) -> Option<Color> {
        match self {
            TextColorOptions::White => Some(Color::White),
            TextColorOptions::Black => Some(Color::Black),
            TextColorOptions::Red => Some(Color::Red),
            TextColorOptions::Green => Some(Color::Green),
            TextColorOptions::Yellow => Some(Color::Yellow),
            TextColorOptions::Blue => Some(Color::Blue),
            TextColorOptions::Magenta => Some(Color::Magenta),
            TextColorOptions::Cyan => Some(Color::Cyan),
            TextColorOptions::Gray => Some(Color::Gray),
            TextColorOptions::DarkGray => Some(Color::DarkGray),
            TextColorOptions::LightRed => Some(Color::LightRed),
            TextColorOptions::LightGreen => Some(Color::LightGreen),
            TextColorOptions::LightYellow => Some(Color::LightYellow),
            TextColorOptions::LightBlue => Some(Color::LightBlue),
            TextColorOptions::LightMagenta => Some(Color::LightMagenta),
            TextColorOptions::LightCyan => Some(Color::LightCyan),
            TextColorOptions::RGB(r, g, b) => Some(Color::Rgb(*r, *g, *b)),
            TextColorOptions::None => None,
        }
    }
    pub fn to_iter() -> impl Iterator<Item = TextColorOptions> {
        vec![
            TextColorOptions::White,
            TextColorOptions::Black,
            TextColorOptions::Red,
            TextColorOptions::Green,
            TextColorOptions::Yellow,
            TextColorOptions::Blue,
            TextColorOptions::Magenta,
            TextColorOptions::Cyan,
            TextColorOptions::Gray,
            TextColorOptions::DarkGray,
            TextColorOptions::LightRed,
            TextColorOptions::LightGreen,
            TextColorOptions::LightYellow,
            TextColorOptions::LightBlue,
            TextColorOptions::LightMagenta,
            TextColorOptions::LightCyan,
            TextColorOptions::Black,
            TextColorOptions::RGB(255, 255, 255),
            TextColorOptions::None,
        ]
        .into_iter()
    }
    pub fn from(color: Color) -> TextColorOptions {
        match color {
            Color::White => TextColorOptions::White,
            Color::Black => TextColorOptions::Black,
            Color::Red => TextColorOptions::Red,
            Color::Green => TextColorOptions::Green,
            Color::Yellow => TextColorOptions::Yellow,
            Color::Blue => TextColorOptions::Blue,
            Color::Magenta => TextColorOptions::Magenta,
            Color::Cyan => TextColorOptions::Cyan,
            Color::Gray => TextColorOptions::Gray,
            Color::DarkGray => TextColorOptions::DarkGray,
            Color::LightRed => TextColorOptions::LightRed,
            Color::LightGreen => TextColorOptions::LightGreen,
            Color::LightYellow => TextColorOptions::LightYellow,
            Color::LightBlue => TextColorOptions::LightBlue,
            Color::LightMagenta => TextColorOptions::LightMagenta,
            Color::LightCyan => TextColorOptions::LightCyan,
            Color::Rgb(r, g, b) => TextColorOptions::RGB(r, g, b),
            Color::Reset => TextColorOptions::None,
            _ => TextColorOptions::None,
        }
    }
    // TODO: This is a hack to get around the fact that the Color struct doesn't have a way to get the RGB values
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            TextColorOptions::White => (255, 255, 255),
            TextColorOptions::Black => (0, 0, 0),
            TextColorOptions::Red => (128, 0, 0),
            TextColorOptions::Green => (0, 128, 0),
            TextColorOptions::Yellow => (128, 128, 0),
            TextColorOptions::Blue => (0, 0, 128),
            TextColorOptions::Magenta => (128, 0, 128),
            TextColorOptions::Cyan => (0, 128, 128),
            TextColorOptions::Gray => (192, 192, 192),
            TextColorOptions::DarkGray => (128, 128, 128),
            TextColorOptions::LightRed => (255, 0, 0),
            TextColorOptions::LightGreen => (255, 255, 0),
            TextColorOptions::LightYellow => (0, 255, 0),
            TextColorOptions::LightBlue => (0, 0, 255),
            TextColorOptions::LightMagenta => (255, 0, 255),
            TextColorOptions::LightCyan => (0, 255, 255),
            TextColorOptions::RGB(r, g, b) => (*r, *g, *b),
            TextColorOptions::None => (0, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextModifierOptions {
    Bold,
    Dim,
    Italic,
    Underlined,
    SlowBlink,
    RapidBlink,
    Reversed,
    Hidden,
    CrossedOut,
    None,
}

impl TextModifierOptions {
    pub fn to_string(&self) -> String {
        match self {
            TextModifierOptions::Bold => "Bold".to_string(),
            TextModifierOptions::Dim => "Dim".to_string(),
            TextModifierOptions::Italic => "Italic".to_string(),
            TextModifierOptions::Underlined => "Underlined".to_string(),
            TextModifierOptions::SlowBlink => "SlowBlink".to_string(),
            TextModifierOptions::RapidBlink => "RapidBlink".to_string(),
            TextModifierOptions::Reversed => "Reversed".to_string(),
            TextModifierOptions::Hidden => "Hidden".to_string(),
            TextModifierOptions::CrossedOut => "CrossedOut".to_string(),
            TextModifierOptions::None => "None".to_string(),
        }
    }
    pub fn to_modifier(&self) -> Modifier {
        match self {
            TextModifierOptions::Bold => Modifier::BOLD,
            TextModifierOptions::Dim => Modifier::DIM,
            TextModifierOptions::Italic => Modifier::ITALIC,
            TextModifierOptions::Underlined => Modifier::UNDERLINED,
            TextModifierOptions::SlowBlink => Modifier::SLOW_BLINK,
            TextModifierOptions::RapidBlink => Modifier::RAPID_BLINK,
            TextModifierOptions::Reversed => Modifier::REVERSED,
            TextModifierOptions::Hidden => Modifier::HIDDEN,
            TextModifierOptions::CrossedOut => Modifier::CROSSED_OUT,
            TextModifierOptions::None => Modifier::empty(),
        }
    }
    pub fn to_iter() -> impl Iterator<Item = TextModifierOptions> {
        vec![
            TextModifierOptions::Bold,
            TextModifierOptions::Dim,
            TextModifierOptions::Italic,
            TextModifierOptions::Underlined,
            TextModifierOptions::SlowBlink,
            TextModifierOptions::RapidBlink,
            TextModifierOptions::Reversed,
            TextModifierOptions::Hidden,
            TextModifierOptions::CrossedOut,
            TextModifierOptions::None,
        ]
        .into_iter()
    }
}
