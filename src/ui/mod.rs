use crate::{
    app::App,
    constants::SAMPLE_TEXT,
    ui::themes::{
        cyberpunk_theme, default_theme, dracula_theme, light_theme, matrix_theme, metro_theme,
        midnight_blue_theme, slate_theme,
    },
};
use log::debug;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Cell, Row},
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

pub mod text_box;
pub mod themes;
pub mod ui_helper;
pub mod ui_main;
pub mod widgets;

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
            let mut return_vec = vec![Span::styled(&self.name, self.inactive_text_style)];
            let sample_text = Span::styled(&self.name, self.inactive_text_style);
            for _ in 0..23 {
                return_vec.push(sample_text.clone());
            }
            return_vec
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

    pub fn update_style(
        style: &mut Style,
        fg_color: Option<Color>,
        bg_color: Option<Color>,
        modifier: Option<Modifier>,
    ) {
        if let Some(fg) = fg_color {
            style.fg = Some(fg);
        } else {
            style.fg = None;
        }

        if let Some(bg) = bg_color {
            style.bg = Some(bg);
        } else {
            style.bg = None;
        }

        if let Some(modifier) = modifier {
            style.add_modifier(modifier);
        } else {
            style.sub_modifier = Modifier::empty();
            style.add_modifier = Modifier::empty();
        }
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
                Theme::update_style(&mut theme.general_style, fg_color, bg_color, modifier);
            }
            "list_select_style" => {
                Theme::update_style(&mut theme.list_select_style, fg_color, bg_color, modifier);
            }
            "card_due_default_style" => {
                Theme::update_style(
                    &mut theme.card_due_default_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_due_warning_style" => {
                Theme::update_style(
                    &mut theme.card_due_warning_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_due_overdue_style" => {
                Theme::update_style(
                    &mut theme.card_due_overdue_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_status_active_style" => {
                Theme::update_style(
                    &mut theme.card_status_active_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_status_completed_style" => {
                Theme::update_style(
                    &mut theme.card_status_completed_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_status_stale_style" => {
                Theme::update_style(
                    &mut theme.card_status_stale_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "keyboard_focus_style" => {
                Theme::update_style(
                    &mut theme.keyboard_focus_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "mouse_focus_style" => {
                Theme::update_style(&mut theme.mouse_focus_style, fg_color, bg_color, modifier);
            }
            "help_key_style" => {
                Theme::update_style(&mut theme.help_key_style, fg_color, bg_color, modifier);
            }
            "help_text_style" => {
                Theme::update_style(&mut theme.help_text_style, fg_color, bg_color, modifier);
            }
            "log_error_style" => {
                Theme::update_style(&mut theme.log_error_style, fg_color, bg_color, modifier);
            }
            "log_debug_style" => {
                Theme::update_style(&mut theme.log_debug_style, fg_color, bg_color, modifier);
            }
            "log_warn_style" => {
                Theme::update_style(&mut theme.log_warn_style, fg_color, bg_color, modifier);
            }
            "log_trace_style" => {
                Theme::update_style(&mut theme.log_trace_style, fg_color, bg_color, modifier);
            }
            "log_info_style" => {
                Theme::update_style(&mut theme.log_info_style, fg_color, bg_color, modifier);
            }
            "progress_bar_style" => {
                Theme::update_style(&mut theme.progress_bar_style, fg_color, bg_color, modifier);
            }
            "error_text_style" => {
                Theme::update_style(&mut theme.error_text_style, fg_color, bg_color, modifier);
            }
            "inactive_text_style" => {
                Theme::update_style(&mut theme.inactive_text_style, fg_color, bg_color, modifier);
            }
            "card_priority_low_style" => {
                Theme::update_style(
                    &mut theme.card_priority_low_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_priority_medium_style" => {
                Theme::update_style(
                    &mut theme.card_priority_medium_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
            }
            "card_priority_high_style" => {
                Theme::update_style(
                    &mut theme.card_priority_high_style,
                    fg_color,
                    bg_color,
                    modifier,
                );
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
    Blue,
    Cyan,
    DarkGray,
    Gray,
    Green,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightYellow,
    Magenta,
    None,
    RGB(u8, u8, u8),
    Red,
    White,
    Yellow,
}

impl Display for TextColorOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextColorOptions::Black => write!(f, "Black"),
            TextColorOptions::Blue => write!(f, "Blue"),
            TextColorOptions::Cyan => write!(f, "Cyan"),
            TextColorOptions::DarkGray => write!(f, "DarkGray"),
            TextColorOptions::Gray => write!(f, "Gray"),
            TextColorOptions::Green => write!(f, "Green"),
            TextColorOptions::LightBlue => write!(f, "LightBlue"),
            TextColorOptions::LightCyan => write!(f, "LightCyan"),
            TextColorOptions::LightGreen => write!(f, "LightGreen"),
            TextColorOptions::LightMagenta => write!(f, "LightMagenta"),
            TextColorOptions::LightRed => write!(f, "LightRed"),
            TextColorOptions::LightYellow => write!(f, "LightYellow"),
            TextColorOptions::Magenta => write!(f, "Magenta"),
            TextColorOptions::None => write!(f, "None"),
            TextColorOptions::Red => write!(f, "Red"),
            TextColorOptions::RGB(r, g, b) => write!(f, "RGB({}, {}, {})", r, g, b),
            TextColorOptions::White => write!(f, "White"),
            TextColorOptions::Yellow => write!(f, "Yellow"),
        }
    }
}

impl TextColorOptions {
    pub fn to_color(&self) -> Option<Color> {
        match self {
            TextColorOptions::Black => Some(Color::Black),
            TextColorOptions::Blue => Some(Color::Blue),
            TextColorOptions::Cyan => Some(Color::Cyan),
            TextColorOptions::DarkGray => Some(Color::DarkGray),
            TextColorOptions::Gray => Some(Color::Gray),
            TextColorOptions::Green => Some(Color::Green),
            TextColorOptions::LightBlue => Some(Color::LightBlue),
            TextColorOptions::LightCyan => Some(Color::LightCyan),
            TextColorOptions::LightGreen => Some(Color::LightGreen),
            TextColorOptions::LightMagenta => Some(Color::LightMagenta),
            TextColorOptions::LightRed => Some(Color::LightRed),
            TextColorOptions::LightYellow => Some(Color::LightYellow),
            TextColorOptions::Magenta => Some(Color::Magenta),
            TextColorOptions::None => None,
            TextColorOptions::Red => Some(Color::Red),
            TextColorOptions::RGB(r, g, b) => Some(Color::Rgb(*r, *g, *b)),
            TextColorOptions::White => Some(Color::White),
            TextColorOptions::Yellow => Some(Color::Yellow),
        }
    }
    pub fn to_iter() -> impl Iterator<Item = TextColorOptions> {
        vec![
            TextColorOptions::Black,
            TextColorOptions::Blue,
            TextColorOptions::Cyan,
            TextColorOptions::DarkGray,
            TextColorOptions::Gray,
            TextColorOptions::Green,
            TextColorOptions::LightBlue,
            TextColorOptions::LightCyan,
            TextColorOptions::LightGreen,
            TextColorOptions::LightMagenta,
            TextColorOptions::LightRed,
            TextColorOptions::LightYellow,
            TextColorOptions::Magenta,
            TextColorOptions::None,
            TextColorOptions::Red,
            TextColorOptions::RGB(128, 128, 128),
            TextColorOptions::White,
            TextColorOptions::Yellow,
        ]
        .into_iter()
    }
    pub fn from(color: Color) -> TextColorOptions {
        match color {
            Color::Black => TextColorOptions::Black,
            Color::Blue => TextColorOptions::Blue,
            Color::Cyan => TextColorOptions::Cyan,
            Color::DarkGray => TextColorOptions::DarkGray,
            Color::Gray => TextColorOptions::Gray,
            Color::Green => TextColorOptions::Green,
            Color::LightBlue => TextColorOptions::LightBlue,
            Color::LightCyan => TextColorOptions::LightCyan,
            Color::LightGreen => TextColorOptions::LightGreen,
            Color::LightMagenta => TextColorOptions::LightMagenta,
            Color::LightRed => TextColorOptions::LightRed,
            Color::LightYellow => TextColorOptions::LightYellow,
            Color::Magenta => TextColorOptions::Magenta,
            Color::Red => TextColorOptions::Red,
            Color::Reset => TextColorOptions::None,
            Color::Rgb(r, g, b) => TextColorOptions::RGB(r, g, b),
            Color::White => TextColorOptions::White,
            Color::Yellow => TextColorOptions::Yellow,
            _ => TextColorOptions::None,
        }
    }
    // TODO: This is a hack to get around the fact that the Color struct doesn't have a way to get the RGB values, find a better way to do this
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            TextColorOptions::Black => (0, 0, 0),
            TextColorOptions::Blue => (0, 0, 128),
            TextColorOptions::Cyan => (0, 128, 128),
            TextColorOptions::DarkGray => (128, 128, 128),
            TextColorOptions::Gray => (192, 192, 192),
            TextColorOptions::Green => (0, 128, 0),
            TextColorOptions::LightBlue => (0, 0, 255),
            TextColorOptions::LightCyan => (0, 255, 255),
            TextColorOptions::LightGreen => (255, 255, 0),
            TextColorOptions::LightMagenta => (255, 0, 255),
            TextColorOptions::LightRed => (255, 0, 0),
            TextColorOptions::LightYellow => (0, 255, 0),
            TextColorOptions::Magenta => (128, 0, 128),
            TextColorOptions::None => (0, 0, 0),
            TextColorOptions::Red => (128, 0, 0),
            TextColorOptions::RGB(r, g, b) => (*r, *g, *b),
            TextColorOptions::White => (255, 255, 255),
            TextColorOptions::Yellow => (128, 128, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextModifierOptions {
    Bold,
    CrossedOut,
    Dim,
    Hidden,
    Italic,
    None,
    RapidBlink,
    Reversed,
    SlowBlink,
    Underlined,
}

impl Display for TextModifierOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextModifierOptions::Bold => write!(f, "Bold"),
            TextModifierOptions::CrossedOut => write!(f, "CrossedOut"),
            TextModifierOptions::Dim => write!(f, "Dim"),
            TextModifierOptions::Hidden => write!(f, "Hidden"),
            TextModifierOptions::Italic => write!(f, "Italic"),
            TextModifierOptions::None => write!(f, "None"),
            TextModifierOptions::RapidBlink => write!(f, "RapidBlink"),
            TextModifierOptions::Reversed => write!(f, "Reversed"),
            TextModifierOptions::SlowBlink => write!(f, "SlowBlink"),
            TextModifierOptions::Underlined => write!(f, "Underlined"),
        }
    }
}

impl TextModifierOptions {
    pub fn to_modifier(&self) -> Modifier {
        match self {
            TextModifierOptions::Bold => Modifier::BOLD,
            TextModifierOptions::CrossedOut => Modifier::CROSSED_OUT,
            TextModifierOptions::Dim => Modifier::DIM,
            TextModifierOptions::Hidden => Modifier::HIDDEN,
            TextModifierOptions::Italic => Modifier::ITALIC,
            TextModifierOptions::None => Modifier::empty(),
            TextModifierOptions::RapidBlink => Modifier::RAPID_BLINK,
            TextModifierOptions::Reversed => Modifier::REVERSED,
            TextModifierOptions::SlowBlink => Modifier::SLOW_BLINK,
            TextModifierOptions::Underlined => Modifier::UNDERLINED,
        }
    }
    pub fn to_iter() -> impl Iterator<Item = TextModifierOptions> {
        vec![
            TextModifierOptions::Bold,
            TextModifierOptions::CrossedOut,
            TextModifierOptions::Dim,
            TextModifierOptions::Hidden,
            TextModifierOptions::Italic,
            TextModifierOptions::None,
            TextModifierOptions::RapidBlink,
            TextModifierOptions::Reversed,
            TextModifierOptions::SlowBlink,
            TextModifierOptions::Underlined,
        ]
        .into_iter()
    }
}
