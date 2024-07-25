use crate::{
    app::App,
    constants::{TOAST_FADE_IN_TIME, TOAST_FADE_OUT_TIME},
    ui::{theme::Theme, widgets::Widget, TextColorOptions},
    util::lerp_between,
};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct ToastWidget {
    pub duration: Duration,
    pub message: String,
    pub start_time: Instant,
    pub title: String,
    pub toast_color: (u8, u8, u8),
    pub toast_type: ToastType,
}

impl ToastWidget {
    pub fn new(message: String, duration: Duration, toast_type: ToastType, theme: Theme) -> Self {
        Self {
            duration,
            message,
            start_time: Instant::now(),
            title: toast_type.as_string(),
            toast_color: toast_type.as_color(theme),
            toast_type: toast_type.clone(),
        }
    }

    pub fn new_with_title(
        title: String,
        message: String,
        duration: Duration,
        toast_type: ToastType,
        theme: Theme,
    ) -> Self {
        Self {
            duration,
            message,
            start_time: Instant::now(),
            title,
            toast_color: toast_type.as_color(theme),
            toast_type: toast_type.clone(),
        }
    }
}

impl Widget for ToastWidget {
    fn update(app: &mut App) {
        let theme = app.current_theme.clone();
        let term_background_color = if let Some(bg_color) = app.current_theme.general_style.bg {
            TextColorOptions::from(bg_color).to_rgb()
        } else {
            app.state.term_background_color
        };
        let disable_animations = app.config.disable_animations;
        let toasts = &mut app.widgets.toasts;
        for i in (0..toasts.len()).rev() {
            if toasts[i].start_time.elapsed() > toasts[i].duration {
                toasts.remove(i);
                continue;
            }
            if disable_animations {
                toasts[i].toast_color = toasts[i].toast_type.as_color(theme.clone());
                continue;
            }
            if toasts[i].start_time.elapsed() < Duration::from_millis(TOAST_FADE_IN_TIME) {
                let t =
                    toasts[i].start_time.elapsed().as_millis() as f32 / TOAST_FADE_IN_TIME as f32;
                toasts[i].toast_color = lerp_between(
                    term_background_color,
                    toasts[i].toast_type.as_color(theme.clone()),
                    t,
                );
            } else if toasts[i].start_time.elapsed()
                < toasts[i].duration - Duration::from_millis(TOAST_FADE_OUT_TIME)
            {
                toasts[i].toast_color = toasts[i].toast_type.as_color(theme.clone());
            } else {
                let t = (toasts[i].start_time.elapsed()
                    - (toasts[i].duration - Duration::from_millis(TOAST_FADE_OUT_TIME)))
                .as_millis() as f32
                    / TOAST_FADE_OUT_TIME as f32;
                toasts[i].toast_color = lerp_between(
                    toasts[i].toast_type.as_color(theme.clone()),
                    term_background_color,
                    t,
                );
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Error,
    Info,
    Loading,
    Warning,
}

impl ToastType {
    pub fn as_string(&self) -> String {
        match self {
            Self::Error => "Error".to_string(),
            Self::Info => "Info".to_string(),
            Self::Loading => "Loading".to_string(),
            Self::Warning => "Warning".to_string(),
        }
    }

    pub fn as_color(&self, theme: Theme) -> (u8, u8, u8) {
        match self {
            Self::Error => TextColorOptions::from(
                theme
                    .log_error_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightRed),
            )
            .to_rgb(),
            Self::Warning => TextColorOptions::from(
                theme
                    .log_warn_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightYellow),
            )
            .to_rgb(),
            Self::Info => TextColorOptions::from(
                theme
                    .log_info_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightCyan),
            )
            .to_rgb(),
            Self::Loading => TextColorOptions::from(
                theme
                    .log_debug_style
                    .fg
                    .unwrap_or(ratatui::style::Color::LightGreen),
            )
            .to_rgb(),
        }
    }
}
