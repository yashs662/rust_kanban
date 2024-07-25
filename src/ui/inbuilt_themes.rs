use crate::ui::theme::Theme;
use ratatui::style::{Color, Modifier, Style};

pub fn default_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_due_overdue_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_due_warning_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_priority_high_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_priority_low_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_priority_medium_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_status_active_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_status_completed_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        card_status_stale_style: Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        error_text_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        general_style: Style::default().fg(Color::White).bg(Color::Reset),
        help_key_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        help_text_style: Style::default().fg(Color::White).bg(Color::Reset),
        inactive_text_style: Style::default()
            .fg(Color::Rgb(40, 40, 40))
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        keyboard_focus_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        list_select_style: Style::default()
            .fg(Color::White)
            .bg(Color::LightMagenta)
            .add_modifier(Modifier::BOLD),
        log_debug_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        log_error_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        log_info_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        log_trace_style: Style::default()
            .fg(Color::Gray)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        log_warn_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        mouse_focus_style: Style::default()
            .fg(Color::Rgb(255, 165, 0))
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
        name: "Default Theme".to_string(),
        progress_bar_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Reset)
            .add_modifier(Modifier::BOLD),
    }
}
pub fn midnight_blue_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
        card_due_overdue_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(25, 25, 112)),
        card_due_warning_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Rgb(25, 25, 112)),
        card_priority_high_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(25, 25, 112)),
        card_priority_low_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(25, 25, 112)),
        card_priority_medium_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Rgb(25, 25, 112)),
        card_status_active_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(25, 25, 112)),
        card_status_completed_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
        card_status_stale_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(25, 25, 112)),
        error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
        general_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
        help_key_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 112)),
        help_text_style: Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(25, 25, 112)),
        inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
        keyboard_focus_style: Style::default()
            .fg(Color::LightBlue)
            .bg(Color::Rgb(25, 25, 112))
            .add_modifier(Modifier::BOLD),
        list_select_style: Style::default()
            .fg(Color::Gray)
            .bg(Color::Rgb(70, 130, 180)),
        log_debug_style: Style::default()
            .fg(Color::LightBlue)
            .bg(Color::Rgb(25, 25, 112)),
        log_error_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(25, 25, 112)),
        log_info_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(25, 25, 112)),
        log_trace_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Rgb(25, 25, 112)),
        log_warn_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(25, 25, 112)),
        mouse_focus_style: Style::default()
            .fg(Color::LightBlue)
            .bg(Color::Rgb(25, 25, 112))
            .add_modifier(Modifier::BOLD),
        name: "Midnight Blue".to_string(),
        progress_bar_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(25, 25, 112)),
    }
}
pub fn slate_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
        card_due_overdue_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(47, 79, 79)),
        card_due_warning_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Rgb(47, 79, 79)),
        card_priority_high_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(47, 79, 79)),
        card_priority_low_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(47, 79, 79)),
        card_priority_medium_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Rgb(47, 79, 79)),
        card_status_active_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(47, 79, 79)),
        card_status_completed_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
        card_status_stale_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(47, 79, 79)),
        error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
        general_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
        help_key_style: Style::default().fg(Color::Gray).bg(Color::Rgb(47, 79, 79)),
        help_text_style: Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(47, 79, 79)),
        inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
        keyboard_focus_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Rgb(47, 79, 79))
            .add_modifier(Modifier::BOLD),
        list_select_style: Style::default()
            .fg(Color::Gray)
            .bg(Color::Rgb(70, 130, 180)),
        log_debug_style: Style::default()
            .fg(Color::LightBlue)
            .bg(Color::Rgb(47, 79, 79)),
        log_error_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(47, 79, 79)),
        log_info_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(47, 79, 79)),
        log_trace_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Rgb(47, 79, 79)),
        log_warn_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(47, 79, 79)),
        mouse_focus_style: Style::default()
            .fg(Color::LightCyan)
            .bg(Color::Rgb(47, 79, 79))
            .add_modifier(Modifier::BOLD),
        name: "Slate".to_string(),
        progress_bar_style: Style::default()
            .fg(Color::LightGreen)
            .bg(Color::Rgb(47, 79, 79)),
    }
}
pub fn metro_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default().fg(Color::White).bg(Color::Rgb(25, 25, 25)),
        card_due_overdue_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(25, 25, 25)),
        card_due_warning_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(25, 25, 25)),
        card_priority_high_style: Style::default().fg(Color::Red).bg(Color::Rgb(25, 25, 25)),
        card_priority_low_style: Style::default().fg(Color::Green).bg(Color::Rgb(25, 25, 25)),
        card_priority_medium_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(25, 25, 25)),
        card_status_active_style: Style::default().fg(Color::Cyan).bg(Color::Rgb(25, 25, 25)),
        card_status_completed_style: Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(25, 25, 25)),
        card_status_stale_style: Style::default()
            .fg(Color::LightYellow)
            .bg(Color::Rgb(25, 25, 25)),
        error_text_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(25, 25, 25)),
        general_style: Style::default().fg(Color::Gray).bg(Color::Rgb(20, 20, 20)),
        help_key_style: Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(25, 25, 25)),
        help_text_style: Style::default().fg(Color::Gray).bg(Color::Rgb(25, 25, 25)),
        inactive_text_style: Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(25, 25, 25)),
        keyboard_focus_style: Style::default()
            .fg(Color::Green)
            .bg(Color::Rgb(25, 25, 25))
            .add_modifier(Modifier::BOLD),
        list_select_style: Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(124, 252, 0)),
        log_debug_style: Style::default().fg(Color::Cyan).bg(Color::Rgb(25, 25, 25)),
        log_error_style: Style::default()
            .fg(Color::LightRed)
            .bg(Color::Rgb(25, 25, 25)),
        log_info_style: Style::default().fg(Color::White).bg(Color::Rgb(25, 25, 25)),
        log_trace_style: Style::default().fg(Color::Green).bg(Color::Rgb(25, 25, 25)),
        log_warn_style: Style::default()
            .fg(Color::Yellow)
            .bg(Color::Rgb(25, 25, 25)),
        mouse_focus_style: Style::default()
            .fg(Color::Green)
            .bg(Color::Rgb(25, 25, 25))
            .add_modifier(Modifier::BOLD),
        name: "Metro".to_string(),
        progress_bar_style: Style::default().fg(Color::Green).bg(Color::Rgb(25, 25, 25)),
    }
}
pub fn matrix_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        card_due_overdue_style: Style::default().fg(Color::LightRed).bg(Color::Black),
        card_due_warning_style: Style::default().fg(Color::Yellow).bg(Color::Black),
        card_priority_high_style: Style::default().fg(Color::LightRed).bg(Color::Black),
        card_priority_low_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        card_priority_medium_style: Style::default().fg(Color::Yellow).bg(Color::Black),
        card_status_active_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        card_status_completed_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
        card_status_stale_style: Style::default().fg(Color::Yellow).bg(Color::Black),
        error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
        general_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        help_key_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        help_text_style: Style::default().fg(Color::Green).bg(Color::Black),
        inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
        keyboard_focus_style: Style::default()
            .fg(Color::Black)
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
        list_select_style: Style::default().fg(Color::Black).bg(Color::LightGreen),
        log_debug_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        log_error_style: Style::default().fg(Color::LightRed).bg(Color::Black),
        log_info_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
        log_trace_style: Style::default().fg(Color::LightCyan).bg(Color::Black),
        log_warn_style: Style::default().fg(Color::Yellow).bg(Color::Black),
        mouse_focus_style: Style::default()
            .fg(Color::Black)
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD),
        name: "Matrix".to_string(),
        progress_bar_style: Style::default().fg(Color::LightGreen).bg(Color::Black),
    }
}
pub fn cyberpunk_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
        card_due_overdue_style: Style::default()
            .fg(Color::Rgb(255, 28, 92))
            .bg(Color::Black),
        card_due_warning_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black),
        card_priority_high_style: Style::default()
            .fg(Color::Rgb(255, 28, 92))
            .bg(Color::Black),
        card_priority_low_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
        card_priority_medium_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black),
        card_status_active_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
        card_status_completed_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
        card_status_stale_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black),
        error_text_style: Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 28, 92)),
        general_style: Style::default()
            .fg(Color::Rgb(248, 12, 228))
            .bg(Color::Black),
        help_key_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
        help_text_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black),
        inactive_text_style: Style::default().fg(Color::DarkGray).bg(Color::Black),
        keyboard_focus_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD),
        list_select_style: Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(253, 248, 0)),
        log_debug_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
        log_error_style: Style::default()
            .fg(Color::Rgb(255, 28, 92))
            .bg(Color::Black),
        log_info_style: Style::default().fg(Color::Rgb(24, 252, 4)).bg(Color::Black),
        log_trace_style: Style::default().fg(Color::LightCyan).bg(Color::Black),
        log_warn_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black),
        mouse_focus_style: Style::default()
            .fg(Color::Rgb(253, 248, 0))
            .bg(Color::Black)
            .add_modifier(Modifier::BOLD),
        name: "Cyberpunk".to_string(),
        progress_bar_style: Style::default()
            .fg(Color::Rgb(248, 12, 228))
            .bg(Color::Black),
    }
}
pub fn light_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default().fg(Color::LightGreen).bg(Color::White),
        card_due_overdue_style: Style::default().fg(Color::LightRed).bg(Color::White),
        card_due_warning_style: Style::default()
            .fg(Color::Rgb(255, 165, 0))
            .bg(Color::White),
        card_priority_high_style: Style::default().fg(Color::LightRed).bg(Color::White),
        card_priority_low_style: Style::default().fg(Color::LightGreen).bg(Color::White),
        card_priority_medium_style: Style::default()
            .fg(Color::Rgb(255, 165, 0))
            .bg(Color::White),
        card_status_active_style: Style::default().fg(Color::Cyan).bg(Color::White),
        card_status_completed_style: Style::default().fg(Color::LightGreen).bg(Color::White),
        card_status_stale_style: Style::default().fg(Color::DarkGray).bg(Color::White),
        error_text_style: Style::default().fg(Color::Black).bg(Color::LightRed),
        general_style: Style::default().fg(Color::Black).bg(Color::White),
        help_key_style: Style::default().fg(Color::LightMagenta).bg(Color::White),
        help_text_style: Style::default().fg(Color::Black).bg(Color::White),
        inactive_text_style: Style::default().fg(Color::Gray).bg(Color::DarkGray),
        keyboard_focus_style: Style::default().fg(Color::Blue).bg(Color::White),
        list_select_style: Style::default().fg(Color::White).bg(Color::LightMagenta),
        log_debug_style: Style::default().fg(Color::LightGreen).bg(Color::White),
        log_error_style: Style::default().fg(Color::LightRed).bg(Color::White),
        log_info_style: Style::default().fg(Color::Blue).bg(Color::White),
        log_trace_style: Style::default().fg(Color::DarkGray).bg(Color::White),
        log_warn_style: Style::default()
            .fg(Color::Rgb(255, 165, 0))
            .bg(Color::White),
        mouse_focus_style: Style::default()
            .fg(Color::Rgb(255, 165, 0))
            .bg(Color::White),
        name: "Light".to_string(),
        progress_bar_style: Style::default().fg(Color::Green).bg(Color::White),
    }
}
pub fn dracula_theme() -> Theme {
    Theme {
        card_due_default_style: Style::default()
            .fg(Color::Rgb(80, 250, 123))
            .bg(Color::Rgb(40, 42, 54)),
        card_due_overdue_style: Style::default()
            .fg(Color::Rgb(255, 85, 85))
            .bg(Color::Rgb(40, 42, 54)),
        card_due_warning_style: Style::default()
            .fg(Color::Rgb(255, 184, 108))
            .bg(Color::Rgb(40, 42, 54)),
        card_priority_high_style: Style::default()
            .fg(Color::Rgb(255, 85, 85))
            .bg(Color::Rgb(40, 42, 54)),
        card_priority_low_style: Style::default()
            .fg(Color::Rgb(80, 250, 123))
            .bg(Color::Rgb(40, 42, 54)),
        card_priority_medium_style: Style::default()
            .fg(Color::Rgb(255, 184, 108))
            .bg(Color::Rgb(40, 42, 54)),
        card_status_active_style: Style::default()
            .fg(Color::Rgb(139, 233, 253))
            .bg(Color::Rgb(40, 42, 54)),
        card_status_completed_style: Style::default()
            .fg(Color::Rgb(80, 250, 123))
            .bg(Color::Rgb(40, 42, 54)),
        card_status_stale_style: Style::default()
            .fg(Color::Rgb(68, 71, 90))
            .bg(Color::Rgb(40, 42, 54)),
        error_text_style: Style::default()
            .fg(Color::Rgb(40, 42, 54))
            .bg(Color::Rgb(255, 85, 85)),
        general_style: Style::default()
            .fg(Color::Rgb(248, 248, 242))
            .bg(Color::Rgb(40, 42, 54)),
        help_key_style: Style::default()
            .fg(Color::Rgb(255, 121, 198))
            .bg(Color::Rgb(40, 42, 54)),
        help_text_style: Style::default()
            .fg(Color::Rgb(248, 248, 242))
            .bg(Color::Rgb(40, 42, 54)),
        inactive_text_style: Style::default()
            .fg(Color::Rgb(68, 71, 90))
            .bg(Color::Rgb(40, 42, 54)),
        keyboard_focus_style: Style::default()
            .fg(Color::Rgb(80, 250, 123))
            .bg(Color::Rgb(40, 42, 54)),
        list_select_style: Style::default()
            .fg(Color::Rgb(248, 248, 242))
            .bg(Color::Rgb(68, 71, 90)),
        log_debug_style: Style::default()
            .fg(Color::Rgb(80, 250, 123))
            .bg(Color::Rgb(40, 42, 54)),
        log_error_style: Style::default()
            .fg(Color::Rgb(255, 85, 85))
            .bg(Color::Rgb(40, 42, 54)),
        log_info_style: Style::default()
            .fg(Color::Rgb(139, 233, 253))
            .bg(Color::Rgb(40, 42, 54)),
        log_trace_style: Style::default()
            .fg(Color::Rgb(68, 71, 90))
            .bg(Color::Rgb(40, 42, 54)),
        log_warn_style: Style::default()
            .fg(Color::Rgb(255, 184, 108))
            .bg(Color::Rgb(40, 42, 54)),
        mouse_focus_style: Style::default()
            .fg(Color::Rgb(80, 250, 123))
            .bg(Color::Rgb(40, 42, 54)),
        name: "Dracula".to_string(),
        progress_bar_style: Style::default()
            .fg(Color::Rgb(189, 147, 249))
            .bg(Color::Rgb(68, 71, 90)),
    }
}
