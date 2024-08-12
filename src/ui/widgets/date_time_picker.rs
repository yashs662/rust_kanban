use crate::{
    app::{state::Focus, App, DateTimeFormat},
    constants::{
        DATE_TIME_PICKER_ANIM_DURATION, FIELD_NOT_SET, MIN_DATE_PICKER_HEIGHT, MIN_DATE_PICKER_WIDTH, TIME_PICKER_WIDTH
    },
    ui::{
        theme::Theme,
        widgets::{SelfViewportCorrection, Widget, WidgetAnimState},
        PopUp, View,
    },
};
use chrono::{Datelike, NaiveDate, Timelike};
use log::debug;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use strum::{Display, EnumString};

#[derive(Serialize, Deserialize, Debug, Clone, Default, EnumString, Display)]
pub enum CalenderType {
    #[default]
    SundayFirst,
    MondayFirst,
}

type CalculatedMouseCoordsCache = Option<(Vec<(Rect, u8)>, chrono::NaiveDateTime, Rect)>;

pub struct DateTimePickerWidget<'a> {
    pub time_picker_active: bool,
    pub anchor: Option<(u16, u16)>,
    last_anchor: Option<(u16, u16)>,
    pub viewport_corrected_anchor: Option<(u16, u16)>,
    date_picker_anim_state: WidgetAnimState,
    time_picker_anim_state: WidgetAnimState,
    calender_type: CalenderType,
    pub widget_height: u16,
    pub widget_width: u16,
    pub date_target_height: u16,
    pub date_target_width: u16,
    pub time_target_width: u16,
    pub selected_date_time: Option<chrono::NaiveDateTime>,
    last_anim_tick: Instant,
    pub styled_date_lines: (Vec<Line<'a>>, Option<chrono::NaiveDateTime>),
    pub styled_time_lines: (Vec<Line<'a>>, Option<chrono::NaiveDateTime>),
    pub calculated_mouse_coords: CalculatedMouseCoordsCache,
    pub current_viewport: Option<Rect>,
    pub last_corrected_viewport: Option<Rect>,
    pub current_render_area: Option<Rect>,
}

impl<'a> DateTimePickerWidget<'a> {
    pub fn new(calender_type: CalenderType) -> Self {
        Self {
            time_picker_active: false,
            anchor: None,
            last_anchor: None,
            viewport_corrected_anchor: None,
            date_picker_anim_state: WidgetAnimState::Closed,
            time_picker_anim_state: WidgetAnimState::Closed,
            calender_type,
            widget_height: MIN_DATE_PICKER_HEIGHT,
            widget_width: MIN_DATE_PICKER_WIDTH,
            date_target_height: MIN_DATE_PICKER_HEIGHT,
            date_target_width: MIN_DATE_PICKER_WIDTH,
            time_target_width: TIME_PICKER_WIDTH,
            selected_date_time: None,
            last_anim_tick: Instant::now(),
            styled_date_lines: (vec![], None),
            styled_time_lines: (vec![], None),
            calculated_mouse_coords: None,
            current_viewport: None,
            last_corrected_viewport: None,
            current_render_area: None,
        }
    }

    pub fn set_calender_type(&mut self, calender_type: CalenderType) {
        self.calender_type = calender_type;
    }

    pub fn open_date_picker(&mut self) {
        if matches!(self.date_picker_anim_state, WidgetAnimState::Closed)
            || matches!(self.date_picker_anim_state, WidgetAnimState::Closing)
        {
            self.time_picker_active = false;
            self.time_picker_anim_state = WidgetAnimState::Closed;
            self.date_picker_anim_state = WidgetAnimState::Opening;
            self.last_anim_tick = Instant::now();
        }
    }

    pub fn close_date_picker(&mut self) {
        if matches!(self.date_picker_anim_state, WidgetAnimState::Open)
            || matches!(self.date_picker_anim_state, WidgetAnimState::Opening)
        {
            self.time_picker_active = false;
            self.time_picker_anim_state = WidgetAnimState::Closed;
            self.date_picker_anim_state = WidgetAnimState::Closing;
            self.last_anim_tick = Instant::now();
        }
    }

    pub fn reset(&mut self) {
        self.time_picker_active = false;
        self.anchor = None;
        self.viewport_corrected_anchor = None;
        self.date_picker_anim_state = WidgetAnimState::Closed;
        self.time_picker_anim_state = WidgetAnimState::Closed;
        self.selected_date_time = None;
        self.widget_height = MIN_DATE_PICKER_HEIGHT;
        self.widget_width = MIN_DATE_PICKER_WIDTH;
        self.date_target_height = MIN_DATE_PICKER_HEIGHT;
        self.date_target_width = MIN_DATE_PICKER_WIDTH;
        self.styled_date_lines = (vec![], None);
        self.styled_time_lines = (vec![], None);
        self.current_viewport = None;
        debug!("DateTimePickerWidget reset");
    }

    pub fn open_time_picker(&mut self) {
        if !self.time_picker_active {
            self.time_picker_anim_state = WidgetAnimState::Opening;
            self.last_anim_tick = Instant::now();
            self.time_picker_active = true;
        }
    }

    pub fn close_time_picker(&mut self) {
        if self.time_picker_active {
            self.time_picker_anim_state = WidgetAnimState::Closing;
            self.last_anim_tick = Instant::now();
            self.time_picker_active = false;
        }
    }

    fn num_days_in_month(year: i32, month: u32) -> Option<u32> {
        // the first day of the next month...
        let (y, m) = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let d = match NaiveDate::from_ymd_opt(y, m, 1) {
            Some(d) => d,
            None => return None,
        };
        d.pred_opt().map(|d| d.day())
    }

    fn adjust_selected_date_with_days(&mut self, days: i64) {
        if let Some(current_date) = self.selected_date_time {
            self.selected_date_time = current_date.checked_add_signed(chrono::Duration::days(days));
        } else {
            debug!("No selected date time found, defaulting to current date time");
            self.selected_date_time = chrono::Local::now()
                .naive_local()
                .checked_add_signed(chrono::Duration::days(days));
        }
    }

    fn adjust_selected_date_with_months(&mut self, months: i64) {
        self.selected_date_time = if let Some(selected_date_time) = self.selected_date_time {
            if months.is_negative() {
                selected_date_time
                    .checked_sub_months(chrono::Months::new(months.unsigned_abs() as u32))
            } else {
                selected_date_time
                    .checked_add_months(chrono::Months::new(months.unsigned_abs() as u32))
            }
        } else {
            debug!("No selected date time found, defaulting to current date time");
            let current_date_time = chrono::Local::now().naive_local();
            if months.is_negative() {
                current_date_time
                    .checked_sub_months(chrono::Months::new(months.unsigned_abs() as u32))
            } else {
                current_date_time
                    .checked_add_months(chrono::Months::new(months.unsigned_abs() as u32))
            }
        };
    }

    fn adjust_selected_date_with_years(&mut self, years: i64) {
        let current_date_time = if let Some(selected_date_time) = self.selected_date_time {
            selected_date_time
        } else {
            debug!("No selected date time found, defaulting to current date time");
            chrono::Local::now().naive_local()
        };
        let current_time = current_date_time.time();
        let modified_years = current_date_time.year() as i64 + years;
        let modified_date = NaiveDate::from_ymd_opt(
            modified_years as i32,
            current_date_time.month(),
            current_date_time.day(),
        );
        if let Some(modified_date) = modified_date {
            self.selected_date_time = Some(modified_date.and_time(current_time));
        } else {
            debug!("Could not adjust the selected date with years");
        }
    }

    fn adjust_selected_date_with_seconds(&mut self, seconds: i64) {
        if let Some(current_date) = self.selected_date_time {
            self.selected_date_time =
                current_date.checked_add_signed(chrono::Duration::seconds(seconds));
        } else {
            debug!("No selected date time found, defaulting to current date time");
            self.selected_date_time = chrono::Local::now()
                .naive_local()
                .checked_add_signed(chrono::Duration::seconds(seconds));
        }
    }

    pub fn calender_move_up(&mut self) {
        self.adjust_selected_date_with_days(-7);
    }

    pub fn move_hours_next(&mut self) {
        self.adjust_selected_date_with_seconds(3600);
    }

    pub fn move_minutes_next(&mut self) {
        self.adjust_selected_date_with_seconds(60);
    }

    pub fn move_seconds_next(&mut self) {
        self.adjust_selected_date_with_seconds(1);
    }

    pub fn calender_move_down(&mut self) {
        self.adjust_selected_date_with_days(7);
    }

    pub fn move_hours_prv(&mut self) {
        self.adjust_selected_date_with_seconds(-3600);
    }

    pub fn move_minutes_prv(&mut self) {
        self.adjust_selected_date_with_seconds(-60);
    }

    pub fn move_seconds_prv(&mut self) {
        self.adjust_selected_date_with_seconds(-1);
    }

    pub fn move_left(&mut self) {
        self.adjust_selected_date_with_days(-1);
    }

    pub fn move_right(&mut self) {
        self.adjust_selected_date_with_days(1);
    }

    pub fn month_prv(&mut self) {
        self.adjust_selected_date_with_months(-1);
    }

    pub fn month_next(&mut self) {
        self.adjust_selected_date_with_months(1);
    }

    pub fn year_prv(&mut self) {
        self.adjust_selected_date_with_years(-1);
    }

    pub fn year_next(&mut self) {
        self.adjust_selected_date_with_years(1);
    }

    /// Calculates and caches styled line and returns selected month and year
    pub fn calculate_styled_lines_of_dates(
        &mut self,
        is_active: bool,
        current_theme: &Theme,
    ) -> (String, String) {
        let date = self
            .selected_date_time
            .unwrap_or_else(|| chrono::Local::now().naive_local());
        let current_month = chrono::Month::try_from(date.month() as u8).unwrap();
        let current_year = date.year();
        if self.styled_date_lines.1 == Some(date) {
            return (current_month.name().to_string(), current_year.to_string());
        }
        let first_day_of_month = match self.calender_type {
            CalenderType::MondayFirst => date.with_day(1).unwrap().weekday().number_from_monday(),
            CalenderType::SundayFirst => date.with_day(1).unwrap().weekday().number_from_sunday(),
        };
        let previous_month = current_month.pred();
        let number_of_days_in_previous_month =
            Self::num_days_in_month(current_year, previous_month.number_from_month()).unwrap();
        let number_of_days_in_current_month =
            Self::num_days_in_month(current_year, current_month.number_from_month()).unwrap();
        let num_lines_required = if first_day_of_month == 1 {
            (number_of_days_in_current_month as f32 / 7.0).ceil() as u32
        } else {
            ((number_of_days_in_current_month + first_day_of_month - 1) as f32 / 7.0).ceil() as u32
        };
        let previous_month_padding_required = first_day_of_month != 1;
        let next_month_padding_required = num_lines_required * 7 > number_of_days_in_current_month;

        let inactive_style = current_theme.inactive_text_style;
        let general_style = if is_active {
            current_theme.general_style
        } else {
            inactive_style
        };
        let highlight_style = if is_active {
            current_theme.keyboard_focus_style
        } else {
            inactive_style
        };

        let mut lines: Vec<Line> = Vec::new();
        let days_line = match self.calender_type {
            CalenderType::MondayFirst => Line::from(vec![
                Span::styled("Mo ", general_style),
                Span::styled("Tu ", general_style),
                Span::styled("We ", general_style),
                Span::styled("Th ", general_style),
                Span::styled("Fr ", general_style),
                Span::styled("Sa ", general_style),
                Span::styled("Su ", general_style),
            ]),
            CalenderType::SundayFirst => Line::from(vec![
                Span::styled("Su ", general_style),
                Span::styled("Mo ", general_style),
                Span::styled("Tu ", general_style),
                Span::styled("We ", general_style),
                Span::styled("Th ", general_style),
                Span::styled("Fr ", general_style),
                Span::styled("Sa ", general_style),
            ]),
        };
        lines.push(days_line);
        let mut current_date = 1;
        for line_num in 0..num_lines_required {
            let mut current_line_spans: Vec<Span> = Vec::new();
            if line_num == 0 {
                if previous_month_padding_required {
                    for pre_month_days in 0..first_day_of_month - 1 {
                        let calc = number_of_days_in_previous_month - first_day_of_month
                            + pre_month_days
                            + 2;
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc), inactive_style));
                    }
                }
                for current_month_days in first_day_of_month..8 {
                    let calc_date = current_month_days - first_day_of_month + 1;
                    current_date = calc_date;
                    if calc_date == date.day() {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), highlight_style));
                    } else {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), general_style));
                    }
                }
            } else if line_num == num_lines_required - 1 {
                for calc_date in current_date + 1..number_of_days_in_current_month + 1 {
                    if calc_date == date.day() {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), highlight_style));
                    } else {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), general_style));
                    }
                }
                if next_month_padding_required {
                    let mut next_month_days = 1;
                    for _ in 0..7 - current_line_spans.len() {
                        current_line_spans.push(Span::styled(
                            format!("{:3}", next_month_days),
                            inactive_style,
                        ));
                        next_month_days += 1;
                    }
                }
            } else {
                for current_month_days in 1..8 {
                    let calc_date = (line_num * 7) + current_month_days - first_day_of_month + 1;
                    current_date = calc_date;
                    if calc_date == date.day() {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), highlight_style));
                    } else {
                        current_line_spans
                            .push(Span::styled(format!("{:3}", calc_date), general_style));
                    }
                }
            }
            lines.push(Line::from(current_line_spans));
        }
        self.date_target_height = MIN_DATE_PICKER_HEIGHT + lines.len() as u16 + 2; // Extra 2 for header and space below it
        self.date_target_width =
            ((current_month.name().to_string().len() + 3 + current_year.to_string().len() + 3)
                as u16)
                .max(MIN_DATE_PICKER_WIDTH);
        self.styled_date_lines = (lines, Some(date));
        (current_month.name().to_string(), current_year.to_string())
    }

    fn adjust_hour(hour: i64) -> i64 {
        if hour < 0 {
            24 + hour
        } else if hour > 23 {
            hour - 24
        } else {
            hour
        }
    }

    fn adjust_minute_or_second(value: i64) -> i64 {
        if value < 0 {
            60 + value
        } else if value > 59 {
            value - 60
        } else {
            value
        }
    }

    fn create_time_line(
        lines: &mut Vec<Line>,
        hms: (i64, i64, i64),
        styles: (Style, Style, Style),
        current_line: bool,
    ) {
        let hour = if hms.0 < 10 {
            format!("{}  ", hms.0)
        } else {
            format!("{} ", hms.0)
        };
        let minute = if hms.1 < 10 {
            format!(" {}", hms.1)
        } else {
            format!("{}", hms.1)
        };
        let second = if hms.2 < 10 {
            format!("  {}", hms.2)
        } else {
            format!(" {}", hms.2)
        };
        if current_line {
            lines.push(Line::from(vec![
                Span::styled("-- ", styles.0),
                Span::styled("--", styles.1),
                Span::styled(" --", styles.2),
            ]));
            lines.push(Line::from(vec![
                Span::styled(hour.to_string(), styles.0),
                Span::styled(minute.to_string(), styles.1),
                Span::styled(second.to_string(), styles.2),
            ]));
            lines.push(Line::from(vec![
                Span::styled("-- ", styles.0),
                Span::styled("--", styles.1),
                Span::styled(" --", styles.2),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(hour.to_string(), styles.0),
                Span::styled(minute.to_string(), styles.1),
                Span::styled(second.to_string(), styles.2),
            ]));
        }
    }

    pub fn get_styled_lines_of_time(
        &mut self,
        is_active: bool,
        current_theme: &Theme,
        current_focus: &Focus,
    ) -> Vec<Line> {
        let date = self
            .selected_date_time
            .unwrap_or_else(|| chrono::Local::now().naive_local());

        if self.styled_time_lines.1 == Some(date) {
            return self.styled_time_lines.0.clone();
        }

        let general_style = if is_active {
            current_theme.general_style
        } else {
            current_theme.inactive_text_style
        };
        let highlight_style = if is_active {
            current_theme.keyboard_focus_style
        } else {
            current_theme.inactive_text_style
        };
        let hour_style = if current_focus == &Focus::DTPHour {
            highlight_style
        } else {
            general_style
        };
        let minute_style = if current_focus == &Focus::DTPMinute {
            highlight_style
        } else {
            general_style
        };
        let second_style = if current_focus == &Focus::DTPSecond {
            highlight_style
        } else {
            general_style
        };

        let current_time = date.time();
        let current_hours = current_time.hour();
        let current_minutes = current_time.minute();
        let current_seconds = current_time.second();
        let available_height = self.date_target_height.saturating_sub(6); // 2 for border, 2 for extra padding, 2 for current time line
        let num_previous_lines = available_height / 2;
        let num_after_lines = if available_height % 2 == 0 {
            num_previous_lines
        } else {
            num_previous_lines + 1
        };
        let mut lines: Vec<Line> = Vec::new();

        for offset in (1..(num_previous_lines + 1)).rev() {
            let current_hour = Self::adjust_hour(current_hours as i64 - offset as i64);
            let current_minute =
                Self::adjust_minute_or_second(current_minutes as i64 - offset as i64);
            let current_second =
                Self::adjust_minute_or_second(current_seconds as i64 - offset as i64);
            Self::create_time_line(
                &mut lines,
                (current_hour, current_minute, current_second),
                (
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                ),
                false,
            );
        }

        Self::create_time_line(
            &mut lines,
            (
                current_hours as i64,
                current_minutes as i64,
                current_seconds as i64,
            ),
            (hour_style, minute_style, second_style),
            true,
        );

        for offset in 1..=num_after_lines {
            let current_hour = Self::adjust_hour(current_hours as i64 + offset as i64);
            let current_minute =
                Self::adjust_minute_or_second(current_minutes as i64 + offset as i64);
            let current_second =
                Self::adjust_minute_or_second(current_seconds as i64 + offset as i64);
            Self::create_time_line(
                &mut lines,
                (current_hour, current_minute, current_second),
                (
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                    current_theme.inactive_text_style,
                ),
                false,
            );
        }
        lines
    }

    fn calculate_mouse_coords_for_dates(&mut self) {
        if self.current_render_area.is_none() {
            debug!("No render area found for calculating mouse coords");
            return;
        }
        let render_area = self.current_render_area.unwrap();
        let top_padding = 4; // border, header, extra space, day line
        let left_padding = 2; // border, margin
        let date = self
            .selected_date_time
            .unwrap_or_else(|| chrono::Local::now().naive_local());
        let current_month = chrono::Month::try_from(date.month() as u8).unwrap();
        let current_year = date.year();
        let first_day_of_month = match self.calender_type {
            CalenderType::MondayFirst => {
                date.with_day(1).unwrap().weekday().number_from_monday() - 1
            } // Starts from 0
            CalenderType::SundayFirst => {
                date.with_day(1).unwrap().weekday().number_from_sunday() - 1
            } // Starts from 0
        };
        let number_of_days_in_current_month =
            Self::num_days_in_month(current_year, current_month.number_from_month()).unwrap();
        let mut record = Vec::new();
        for iter_date in 0..number_of_days_in_current_month as u16 {
            // Calculate the correct row and column taking into account the first day of the month
            let adjusted_iter_date = iter_date + first_day_of_month as u16; // Adjust the iter_date based on the first day of the month
            let row = adjusted_iter_date / 7;
            let col = adjusted_iter_date % 7; // Use adjusted_iter_date for column calculation
            let x = render_area.x + left_padding + (col * 3) - 1; // Column position
            let y = render_area.y + top_padding + row - 1; // Row position
            let rect = Rect::new(x, y, 3, 1);
            record.push((rect, (iter_date + 1) as u8));
        }
        self.calculated_mouse_coords = Some((record, date, render_area));
    }

    pub fn get_date_time_as_string(&self, date_time_format: DateTimeFormat) -> String {
        if let Some(selected_date) = self.selected_date_time {
            selected_date
                .format(date_time_format.to_parser_string())
                .to_string()
        } else {
            FIELD_NOT_SET.to_string()
        }
    }

    fn calculate_animation_percentage(&self) -> f32 {
        let milliseconds_passed = self.last_anim_tick.elapsed().as_millis() as f32;
        milliseconds_passed / (DATE_TIME_PICKER_ANIM_DURATION as f32)
    }

    // Update the height for the date picker animation
    fn update_date_picker_height(&mut self, current_percentage: f32, opening: bool) {
        self.widget_height = if opening {
            (MIN_DATE_PICKER_HEIGHT as f32
                + (self.date_target_height as f32 - MIN_DATE_PICKER_HEIGHT as f32)
                    * current_percentage) as u16
        } else {
            (self.date_target_height as f32
                - (self.date_target_height as f32 - MIN_DATE_PICKER_HEIGHT as f32)
                    * current_percentage) as u16
        };
    }

    // Update the width for the time picker animation
    fn update_time_picker_width(&mut self, current_percentage: f32, opening: bool) {
        self.widget_width = if opening {
            self.date_target_width + (self.time_target_width as f32 * current_percentage) as u16
        } else {
            self.date_target_width + self.time_target_width
                - (self.time_target_width as f32 * current_percentage) as u16
        };
    }

    pub fn select_date_in_current_month(&mut self, date_to_select: u8) {
        if let Some(selected_date) = self.selected_date_time {
            self.selected_date_time = selected_date.with_day(date_to_select as u32);
        } else {
            debug!("No selected date time found, defaulting to current date time");
            self.selected_date_time = chrono::Local::now()
                .naive_local()
                .with_day(date_to_select as u32);
        }
    }
}

impl<'a> Widget for DateTimePickerWidget<'a> {
    fn update(app: &mut App) {
        if app.state.z_stack.last() != Some(&PopUp::DateTimePicker) {
            return;
        }
        let disable_animations = app.config.disable_animations;
        let date_time_picker = &mut app.widgets.date_time_picker;
        match date_time_picker.date_picker_anim_state {
            WidgetAnimState::Opening | WidgetAnimState::Closing => {
                if disable_animations {
                    date_time_picker.date_picker_anim_state = date_time_picker
                        .date_picker_anim_state
                        .complete_current_stage();
                    return;
                }
                let current_percentage = date_time_picker.calculate_animation_percentage();
                let opening = matches!(
                    date_time_picker.date_picker_anim_state,
                    WidgetAnimState::Opening
                );
                if current_percentage < 1.0 {
                    date_time_picker.date_picker_anim_state = if opening {
                        WidgetAnimState::Opening
                    } else {
                        WidgetAnimState::Closing
                    };
                    date_time_picker.update_date_picker_height(current_percentage, opening);
                } else {
                    date_time_picker.date_picker_anim_state = if opening {
                        WidgetAnimState::Open
                    } else {
                        WidgetAnimState::Closed
                    };
                }
            }
            WidgetAnimState::Open => {
                if date_time_picker.date_target_height != date_time_picker.widget_height {
                    date_time_picker.widget_height = date_time_picker.date_target_height;
                }
            }
            WidgetAnimState::Closed => {
                app.state.z_stack.pop();
                if app.state.current_view != View::NewCard {
                    date_time_picker.reset();
                }
            }
        }

        match date_time_picker.time_picker_anim_state {
            WidgetAnimState::Opening | WidgetAnimState::Closing => {
                if disable_animations {
                    date_time_picker.time_picker_anim_state = date_time_picker
                        .time_picker_anim_state
                        .complete_current_stage();
                    return;
                }
                let current_percentage = date_time_picker.calculate_animation_percentage();
                let opening = matches!(
                    date_time_picker.time_picker_anim_state,
                    WidgetAnimState::Opening
                );
                if current_percentage < 1.0 {
                    date_time_picker.time_picker_anim_state = if opening {
                        WidgetAnimState::Opening
                    } else {
                        WidgetAnimState::Closing
                    };
                    date_time_picker.update_time_picker_width(current_percentage, opening);
                } else {
                    date_time_picker.time_picker_anim_state = if opening {
                        WidgetAnimState::Open
                    } else {
                        WidgetAnimState::Closed
                    };
                }
            }
            WidgetAnimState::Open => {
                if (date_time_picker.date_target_width + date_time_picker.time_target_width)
                    != date_time_picker.widget_width
                {
                    date_time_picker.widget_width =
                        date_time_picker.date_target_width + date_time_picker.time_target_width;
                }
            }
            WidgetAnimState::Closed => {
                if date_time_picker.widget_width != date_time_picker.date_target_width {
                    date_time_picker.widget_width = date_time_picker.date_target_width;
                }
            }
        }

        date_time_picker.self_correct(
            date_time_picker.date_target_height,
            date_time_picker.date_target_width,
        );

        let mut re_calculate = false;
        if let Some((_, calc_date, calc_render_area)) = &date_time_picker.calculated_mouse_coords {
            if let Some(selected_date) = date_time_picker.selected_date_time {
                // check if same month
                if selected_date.month() != calc_date.month() {
                    re_calculate = true;
                }
            } else {
                re_calculate = true;
            }
            if let Some(render_area) = date_time_picker.current_render_area {
                if render_area != *calc_render_area {
                    re_calculate = true;
                }
            }
        } else if date_time_picker.current_render_area.is_some() {
            re_calculate = true;
        }
        if re_calculate {
            date_time_picker.calculate_mouse_coords_for_dates();
        }
    }
}

impl<'a> SelfViewportCorrection for DateTimePickerWidget<'a> {
    fn get_anchor(&self) -> Option<(u16, u16)> {
        self.anchor
    }
    fn get_last_anchor(&self) -> Option<(u16, u16)> {
        self.last_anchor
    }
    fn get_viewport_corrected_anchor(&self) -> Option<(u16, u16)> {
        self.viewport_corrected_anchor
    }
    fn get_current_viewport(&self) -> Option<Rect> {
        self.current_viewport
    }
    fn get_last_corrected_viewport(&self) -> Option<Rect> {
        self.last_corrected_viewport
    }
    fn set_anchor(&mut self, anchor: Option<(u16, u16)>) {
        self.set_last_anchor(self.anchor);
        self.anchor = anchor;        
    }
    fn set_last_anchor(&mut self, anchor: Option<(u16, u16)>) {
        self.last_anchor = anchor;
    }
    fn set_viewport_corrected_anchor(&mut self, anchor: Option<(u16, u16)>) {
        self.viewport_corrected_anchor = anchor;
    }
    fn set_current_viewport(&mut self, viewport: Option<Rect>) {
        self.current_viewport = viewport;
    }
    fn set_last_corrected_viewport(&mut self, viewport: Option<Rect>) {
        self.last_corrected_viewport = viewport;
    }
}
