use std::{time::Duration, sync::Arc};
use tokio::time::Instant;

use crate::{app::App, constants::TOAST_FADE_TIME};

#[derive(Clone, Debug)]
pub struct ToastWidget {
    pub message: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub toast_type: ToastType,
    pub toast_color: (u8, u8, u8),
}

#[derive(Clone, Debug)]
pub enum ToastType {
    Error,
    Warning,
    Info,
}

pub struct WidgetManager {
    pub app: Arc<tokio::sync::Mutex<App>>,
}

impl ToastWidget {
    pub fn new(message: String, duration: Duration, toast_type: ToastType) -> Self {
        Self {
            message,
            duration,
            start_time: Instant::now(),
            toast_type: toast_type.clone(),
            toast_color: toast_type.as_color(),
        }
    }
}

impl ToastType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
        }
    }
    pub fn as_color(&self) -> (u8, u8, u8) {
        match self {
            Self::Error => (255, 0, 0),
            Self::Warning => (255, 255, 0),
            Self::Info => (0, 255, 255),
        }
    }
}

impl WidgetManager {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn update(&mut self) {
        let mut app = self.app.lock().await;
        let term_background_color = app.state.term_background_color;
        let toasts = &mut app.state.toasts;
        // remove all inactive toasts
        for i in (0..toasts.len()).rev() {
            // based on the toast_type lerp between the toast_type color and 0,0,0 within the TOAST_FADE_TIME which is in milliseconds
            if toasts[i].start_time.elapsed() < toasts[i].duration -  Duration::from_millis(TOAST_FADE_TIME) {
                toasts[i].toast_color = toasts[i].toast_type.as_color();
            } else {
                // lerp from toast_type color to term_background_color
                let t = (toasts[i].start_time.elapsed() - (toasts[i].duration - Duration::from_millis(TOAST_FADE_TIME))).as_millis() as f32 / TOAST_FADE_TIME as f32;
                toasts[i].toast_color = lerp_between(toasts[i].toast_type.as_color(), term_background_color, t);
            }
            if toasts[i].start_time.elapsed() > toasts[i].duration {
                toasts.remove(i);
            }
        }
    }
}

// make a function to lerp between rgb values of two colors
pub fn lerp_between(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let r = (a.0 as f32 * (1.0 - t) + b.0 as f32 * t) as u8;
    let g = (a.1 as f32 * (1.0 - t) + b.1 as f32 * t) as u8;
    let b = (a.2 as f32 * (1.0 - t) + b.2 as f32 * t) as u8;
    (r, g, b)
}
