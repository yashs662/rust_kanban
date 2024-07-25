use crate::{
    app::{state::Focus, App},
    ui::{widgets::Widget, TextColorOptions},
    util::lerp_between,
};
use ratatui::style::{Color, Style};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct CloseButtonWidget {
    start_time: Instant,
    fade_time: f32,
    pub color: (u8, u8, u8),
    offset: f32,
}

impl CloseButtonWidget {
    pub fn new(style: Style) -> Self {
        let color = style.fg.unwrap_or(Color::White);
        let text_color = TextColorOptions::from(color).to_rgb();
        Self {
            start_time: Instant::now(),
            fade_time: 1.0,
            color: text_color,
            offset: 0.8,
        }
    }
}

impl Widget for CloseButtonWidget {
    fn update(app: &mut App) {
        if app.state.focus == Focus::CloseButton {
            let theme = app.current_theme.clone();
            let disable_animations = app.config.disable_animations;
            let widget = &mut app.widgets.close_button;
            if disable_animations {
                widget.color = TextColorOptions::from(theme.error_text_style.bg.unwrap()).to_rgb();
                return;
            }

            let normal_color = TextColorOptions::from(theme.general_style.bg.unwrap()).to_rgb();
            let hover_color = TextColorOptions::from(theme.error_text_style.bg.unwrap()).to_rgb();
            let total_duration = Duration::from_millis((widget.fade_time * 1000.0) as u64);
            let half_duration = Duration::from_millis((widget.fade_time * 500.0) as u64);

            if widget.start_time.elapsed() > total_duration {
                widget.start_time = Instant::now();
            }

            let mut t = (widget.start_time.elapsed().as_millis() as f32
                / total_duration.as_millis() as f32)
                + widget.offset; // offset to make it overall brighter

            if widget.start_time.elapsed() < half_duration {
                widget.color = lerp_between(normal_color, hover_color, t);
            } else {
                t = t - widget.fade_time - (widget.offset / 4.0); // offset to make it overall brighter
                widget.color = lerp_between(hover_color, normal_color, t);
            }
        }
    }
}
