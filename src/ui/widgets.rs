use std::{time::Duration, sync::Arc};
use tokio::time::Instant;

use crate::app::App;

#[derive(Clone, Debug)]
pub struct ToastWidget {
    pub message: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub toast_type: ToastType,
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
            toast_type,
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
}

impl WidgetManager {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn update(&mut self) {
        let mut app = self.app.lock().await;
        let toast_list = &mut app.state.toast_list;
        // remove all inactive toasts
        toast_list.retain(|toast| toast.start_time.elapsed() < toast.duration);
    }
}
