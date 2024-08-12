use crate::{app::App, ui::theme::Theme};
use close_button::CloseButtonWidget;
use command_palette::CommandPaletteWidget;
use date_time_picker::{CalenderType, DateTimePickerWidget};
use ratatui::layout::Rect;
use std::sync::Arc;
use tag_picker::TagPickerWidget;
use toast::ToastWidget;

pub mod close_button;
pub mod command_palette;
pub mod date_time_picker;
pub mod tag_picker;
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
        TagPickerWidget::update(&mut app);
    }
}

pub struct Widgets<'a> {
    pub command_palette: CommandPaletteWidget,
    pub close_button: CloseButtonWidget,
    pub toast_widget: ToastWidget,
    pub date_time_picker: DateTimePickerWidget<'a>,
    pub tag_picker: TagPickerWidget,
}

impl<'a> Widgets<'a> {
    pub fn new(theme: Theme, debug_mode: bool, calender_type: CalenderType) -> Self {
        Self {
            command_palette: CommandPaletteWidget::new(debug_mode),
            close_button: CloseButtonWidget::new(theme.general_style),
            toast_widget: ToastWidget::default(),
            date_time_picker: DateTimePickerWidget::new(calender_type),
            tag_picker: TagPickerWidget::default(),
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

pub trait SelfViewportCorrection {
    fn get_anchor(&self) -> Option<(u16, u16)>;
    fn get_last_anchor(&self) -> Option<(u16, u16)>;
    fn get_viewport_corrected_anchor(&self) -> Option<(u16, u16)>;
    fn get_last_corrected_viewport(&self) -> Option<Rect>;
    fn get_current_viewport(&self) -> Option<Rect>;
    fn set_anchor(&mut self, anchor: Option<(u16, u16)>);
    fn set_last_anchor(&mut self, anchor: Option<(u16, u16)>);
    fn set_viewport_corrected_anchor(&mut self, anchor: Option<(u16, u16)>);
    fn set_last_corrected_viewport(&mut self, anchor: Option<Rect>);
    fn set_current_viewport(&mut self, anchor: Option<Rect>);
    fn self_correct(&mut self, target_height: u16, target_width: u16) {
        if self.get_current_viewport().is_some()
            && self.get_anchor().is_some()
            && (self.get_last_corrected_viewport() != self.get_current_viewport()
                || self.get_last_anchor() != self.get_anchor())
        {
            if let (Some(anchor), Some(viewport)) = (self.get_anchor(), self.get_current_viewport())
            {
                let mut viewport_corrected_anchor = anchor;
                if anchor.1 + target_height > viewport.height {
                    viewport_corrected_anchor.1 = viewport_corrected_anchor
                        .1
                        .saturating_sub((anchor.1 + target_height).saturating_sub(viewport.height));
                }
                if anchor.0 + target_width > viewport.width {
                    viewport_corrected_anchor.0 = viewport_corrected_anchor
                        .0
                        .saturating_sub((anchor.0 + target_width).saturating_sub(viewport.width));
                }
                self.set_viewport_corrected_anchor(Some(viewport_corrected_anchor));
                self.set_last_corrected_viewport(Some(viewport));
            }
        }
    }
}
