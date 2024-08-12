use crate::{
    app::App,
    constants::SAMPLE_TEXT,
    ui::inbuilt_themes::{
        cyberpunk_theme, default_theme, dracula_theme, light_theme, matrix_theme, metro_theme,
        midnight_blue_theme, slate_theme,
    },
};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Row},
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Theme {
    pub card_due_default_style: Style,
    pub card_due_overdue_style: Style,
    pub card_due_warning_style: Style,
    pub card_priority_high_style: Style,
    pub card_priority_low_style: Style,
    pub card_priority_medium_style: Style,
    pub card_status_active_style: Style,
    pub card_status_completed_style: Style,
    pub card_status_stale_style: Style,
    pub error_text_style: Style,
    pub general_style: Style,
    pub help_key_style: Style,
    pub help_text_style: Style,
    pub inactive_text_style: Style,
    pub keyboard_focus_style: Style,
    pub list_select_style: Style,
    pub log_debug_style: Style,
    pub log_error_style: Style,
    pub log_info_style: Style,
    pub log_trace_style: Style,
    pub log_warn_style: Style,
    pub mouse_focus_style: Style,
    pub name: String,
    pub progress_bar_style: Style,
}

impl Default for Theme {
    fn default() -> Self {
        default_theme()
    }
}

impl Theme {
    pub fn all_default_themes() -> Vec<Theme> {
        vec![
            cyberpunk_theme(),
            default_theme(),
            dracula_theme(),
            light_theme(),
            matrix_theme(),
            metro_theme(),
            midnight_blue_theme(),
            slate_theme(),
        ]
    }

    pub fn get_style(&self, theme_enum: ThemeEnum) -> Style {
        match theme_enum {
            ThemeEnum::Name => self.general_style,
            ThemeEnum::General => self.general_style,
            ThemeEnum::ListSelect => self.list_select_style,
            ThemeEnum::MouseFocus => self.mouse_focus_style,
            ThemeEnum::CardDueDefault => self.card_due_default_style,
            ThemeEnum::CardDueOverdue => self.card_due_overdue_style,
            ThemeEnum::CardDueWarning => self.card_due_warning_style,
            ThemeEnum::CardPriorityHigh => self.card_priority_high_style,
            ThemeEnum::CardPriorityLow => self.card_priority_low_style,
            ThemeEnum::CardPriorityMedium => self.card_priority_medium_style,
            ThemeEnum::CardStatusActive => self.card_status_active_style,
            ThemeEnum::CardStatusCompleted => self.card_status_completed_style,
            ThemeEnum::CardStatusStale => self.card_status_stale_style,
            ThemeEnum::ProgressBar => self.progress_bar_style,
            ThemeEnum::ErrorText => self.error_text_style,
            ThemeEnum::HelpKey => self.help_key_style,
            ThemeEnum::HelpText => self.help_text_style,
            ThemeEnum::InactiveText => self.inactive_text_style,
            ThemeEnum::KeyboardFocus => self.keyboard_focus_style,
            ThemeEnum::LogDebug => self.log_debug_style,
            ThemeEnum::LogError => self.log_error_style,
            ThemeEnum::LogInfo => self.log_info_style,
            ThemeEnum::LogTrace => self.log_trace_style,
            ThemeEnum::LogWarn => self.log_warn_style,
        }
    }

    pub fn get_mut_style(&mut self, theme_enum: ThemeEnum) -> &mut Style {
        match theme_enum {
            ThemeEnum::Name => &mut self.general_style,
            ThemeEnum::General => &mut self.general_style,
            ThemeEnum::ListSelect => &mut self.list_select_style,
            ThemeEnum::MouseFocus => &mut self.mouse_focus_style,
            ThemeEnum::CardDueDefault => &mut self.card_due_default_style,
            ThemeEnum::CardDueOverdue => &mut self.card_due_overdue_style,
            ThemeEnum::CardDueWarning => &mut self.card_due_warning_style,
            ThemeEnum::CardPriorityHigh => &mut self.card_priority_high_style,
            ThemeEnum::CardPriorityLow => &mut self.card_priority_low_style,
            ThemeEnum::CardPriorityMedium => &mut self.card_priority_medium_style,
            ThemeEnum::CardStatusActive => &mut self.card_status_active_style,
            ThemeEnum::CardStatusCompleted => &mut self.card_status_completed_style,
            ThemeEnum::CardStatusStale => &mut self.card_status_stale_style,
            ThemeEnum::ProgressBar => &mut self.progress_bar_style,
            ThemeEnum::ErrorText => &mut self.error_text_style,
            ThemeEnum::HelpKey => &mut self.help_key_style,
            ThemeEnum::HelpText => &mut self.help_text_style,
            ThemeEnum::InactiveText => &mut self.inactive_text_style,
            ThemeEnum::KeyboardFocus => &mut self.keyboard_focus_style,
            ThemeEnum::LogDebug => &mut self.log_debug_style,
            ThemeEnum::LogError => &mut self.log_error_style,
            ThemeEnum::LogInfo => &mut self.log_info_style,
            ThemeEnum::LogTrace => &mut self.log_trace_style,
            ThemeEnum::LogWarn => &mut self.log_warn_style,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn create_styled_spans(&self, app: &App, is_active: bool, sample_mode: bool) -> Vec<Span> {
        ThemeEnum::iter()
            .map(|theme_enum| {
                let text = if sample_mode {
                    if theme_enum == ThemeEnum::Name {
                        self.name.clone()
                    } else {
                        SAMPLE_TEXT.to_string()
                    }
                } else {
                    theme_enum.to_string()
                };
                let style = if !is_active {
                    app.current_theme.inactive_text_style
                } else if sample_mode {
                    self.get_style(theme_enum)
                } else {
                    app.current_theme.general_style
                };
                Span::styled(text, style)
            })
            .collect::<Vec<Span>>()
    }

    pub fn to_rows(&self, app: &App, is_active: bool) -> (Vec<Row>, Vec<Row>) {
        let theme_title_list = self.create_styled_spans(app, is_active, false);
        let theme_style_list = self.create_styled_spans(app, is_active, true);

        let rows_title = theme_title_list
            .iter()
            .map(|span| Row::new(vec![Cell::from(span.clone())]))
            .collect();
        let rows_elements = theme_style_list
            .iter()
            .map(|span| Row::new(vec![Cell::from(span.clone())]))
            .collect();

        (rows_title, rows_elements)
    }

    pub fn update_style(
        style: &mut Style,
        fg_color: Option<Color>,
        bg_color: Option<Color>,
        modifier: Option<Modifier>,
    ) {
        if let Some(fg) = fg_color {
            style.fg = Some(fg);
        }

        if let Some(bg) = bg_color {
            style.bg = Some(bg);
        }

        if let Some(modifier) = modifier {
            Self::add_modifier_to_style(style, modifier);
        }
    }

    pub fn add_modifier_to_style(style: &mut Style, modifier: Modifier) {
        style.sub_modifier = style.sub_modifier.difference(modifier);
        style.add_modifier = style.add_modifier.union(modifier);
    }
}

#[derive(EnumIter, Display, PartialEq, Clone, Copy)]
#[strum(serialize_all = "title_case")]
pub enum ThemeEnum {
    Name,
    General,
    ListSelect,
    MouseFocus,
    CardDueDefault,
    CardDueOverdue,
    CardDueWarning,
    CardPriorityHigh,
    CardPriorityLow,
    CardPriorityMedium,
    CardStatusActive,
    CardStatusCompleted,
    CardStatusStale,
    ProgressBar,
    ErrorText,
    HelpKey,
    HelpText,
    InactiveText,
    KeyboardFocus,
    LogDebug,
    LogError,
    LogInfo,
    LogTrace,
    LogWarn,
}
