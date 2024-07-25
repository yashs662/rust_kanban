use crate::{app::App, ui::theme::Theme};
use close_button::CloseButtonWidget;
use command_palette::CommandPaletteWidget;
use date_time_picker::{CalenderType, DateTimePickerWidget};
use std::sync::Arc;
use toast::ToastWidget;

pub mod close_button;
pub mod command_palette;
pub mod date_time_picker;
pub mod toast;

trait Widget {
    fn update(app: &mut App);
}

pub struct WidgetManager<'a> {
    pub app: Arc<tokio::sync::Mutex<App<'a>>>,
}

impl WidgetManager<'_> {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> WidgetManager {
        WidgetManager { app }
    }

    pub async fn update(&mut self) {
        let mut app = self.app.lock().await;
        ToastWidget::update(&mut app);
        CommandPaletteWidget::update(&mut app);
        CloseButtonWidget::update(&mut app);
        DateTimePickerWidget::update(&mut app);
    }
}

pub struct Widgets<'a> {
    pub command_palette: CommandPaletteWidget,
    pub close_button: CloseButtonWidget,
    pub toasts: Vec<ToastWidget>,
    pub date_time_picker: DateTimePickerWidget<'a>,
}

impl<'a> Widgets<'a> {
    pub fn new(theme: Theme, debug_mode: bool, calender_type: CalenderType) -> Self {
        Self {
            command_palette: CommandPaletteWidget::new(debug_mode),
            close_button: CloseButtonWidget::new(theme.general_style),
            toasts: vec![],
            date_time_picker: DateTimePickerWidget::new(calender_type),
        }
    }
}

#[derive(Debug)]
enum WidgetAnimState {
    Closed,
    Closing,
    Open,
    Opening,
}

impl WidgetAnimState {
    fn complete_current_stage(&self) -> Self {
        match self {
            Self::Closed => Self::Closed,
            Self::Closing => Self::Closed,
            Self::Open => Self::Open,
            Self::Opening => Self::Open,
        }
    }
}

// enum AnimType {
//     Slide,
//     // Implement Pulse for close button (get rid of old janky solution used
//     // in close button widget, refer DateTimePickerWidget for example)
//     // Pulse(Duration),
// }
