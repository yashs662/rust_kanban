use crate::{
    app::App,
    constants::{
        MAX_TOASTS_TO_DISPLAY, MIN_TERM_HEIGHT, MIN_TERM_WIDTH, SCREEN_TO_TOAST_WIDTH_RATIO,
        SPINNER_FRAMES,
    },
    ui::{
        rendering::{
            common::{draw_title, render_blank_styled_canvas, render_logs},
            utils::top_left_rect,
        },
        widgets::toast::{ToastType, ToastWidget},
    },
};
use log::debug;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_size_error(rect: &mut Frame, size: &Rect, msg: String, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
        .split(*size);

    let error_text_spans = vec![
        Line::from(Span::styled(msg, app.current_theme.error_text_style)),
        Line::from(Span::styled(
            "Resize the window to continue, or press 'q' to quit.",
            app.current_theme.general_style,
        )),
    ];

    let body = Paragraph::new(error_text_spans)
        .block(Block::default().borders(Borders::ALL).borders(Borders::ALL))
        .alignment(Alignment::Center);

    rect.render_widget(draw_title(app, *size, false), chunks[0]);
    rect.render_widget(body, chunks[1]);
}

pub fn draw_loading_screen(rect: &mut Frame, size: &Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Fill(1)].as_ref())
        .split(*size);

    let mut text = vec![Line::from(vec![
        Span::styled("Loading", app.current_theme.keyboard_focus_style),
        Span::styled(
            "......`(*>﹏<*)′......",
            app.current_theme.keyboard_focus_style,
        ),
        Span::styled("Please wait", app.current_theme.keyboard_focus_style),
    ])];
    if app.config.auto_login {
        text.push(Line::from(Span::styled(
            "",
            app.current_theme.keyboard_focus_style,
        )));
        text.push(Line::from(Span::styled(
            "Auto login enabled, please wait",
            app.current_theme.keyboard_focus_style,
        )));
    }
    let body = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);

    rect.render_widget(draw_title(app, *size, false), chunks[0]);
    rect.render_widget(body, chunks[1]);
}

pub fn check_size(rect: &Rect) -> Result<(), String> {
    if rect.width < MIN_TERM_WIDTH {
        Err(format!(
            "For optimal viewing experience, Terminal width should be >= {}, (current width {})",
            MIN_TERM_WIDTH, rect.width
        ))
    } else if rect.height < MIN_TERM_HEIGHT {
        Err(format!(
            "For optimal viewing experience, Terminal height should be >= {}, (current height {})",
            MIN_TERM_HEIGHT, rect.height
        ))
    } else {
        Ok(())
    }
}

pub fn render_toast(rect: &mut Frame, app: &mut App) {
    let all_toasts = app.widgets.toasts.clone();
    let mut loading_toasts = all_toasts
        .iter()
        .filter(|x| x.toast_type == ToastType::Loading)
        .collect::<Vec<&ToastWidget>>();
    let app_toasts = app.widgets.toasts.clone();
    let toasts = if !loading_toasts.is_empty() {
        let sorted_loading_toasts = if loading_toasts.len() > MAX_TOASTS_TO_DISPLAY - 1 {
            loading_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            loading_toasts
                .iter()
                .copied()
                .take(MAX_TOASTS_TO_DISPLAY - 1)
                .rev()
                .collect::<Vec<&ToastWidget>>()
        } else {
            loading_toasts
        };
        let mut toasts = sorted_loading_toasts;
        let mut regular_toasts = all_toasts
            .iter()
            .filter(|x| x.toast_type != ToastType::Loading)
            .collect::<Vec<&ToastWidget>>();
        regular_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        while toasts.len() < MAX_TOASTS_TO_DISPLAY {
            if let Some(toast) = regular_toasts.pop() {
                toasts.push(toast);
            } else {
                break;
            }
        }
        if toasts.len() < MAX_TOASTS_TO_DISPLAY {
            let mut loading_toasts = all_toasts
                .iter()
                .filter(|x| x.toast_type == ToastType::Loading)
                .collect::<Vec<&ToastWidget>>();
            loading_toasts.sort_by(|a, b| a.start_time.cmp(&b.start_time));
            while toasts.len() < MAX_TOASTS_TO_DISPLAY {
                if let Some(toast) = loading_toasts.pop() {
                    if !toasts.contains(&toast) {
                        toasts.push(toast);
                    }
                } else {
                    break;
                }
            }
        }
        toasts
    } else {
        app_toasts
            .iter()
            .rev()
            .take(MAX_TOASTS_TO_DISPLAY)
            .rev()
            .collect::<Vec<&ToastWidget>>()
    };

    if toasts.is_empty() {
        return;
    }
    let mut total_height_rendered = 1;
    for toast in toasts.iter() {
        let toast_style = app
            .current_theme
            .general_style
            .fg(ratatui::style::Color::Rgb(
                toast.toast_color.0,
                toast.toast_color.1,
                toast.toast_color.2,
            ));
        let mut toast_title = toast.title.to_owned();
        toast_title = match toast.toast_type {
            ToastType::Loading => {
                let spinner_frames = &SPINNER_FRAMES;
                let frame =
                    (toast.start_time.elapsed().as_millis() / 100) % spinner_frames.len() as u128;
                let frame = frame as usize;
                format!("{} {}", spinner_frames[frame], toast_title)
            }
            _ => toast_title,
        };
        let x_offset = rect.size().width - (rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO);
        let lines = textwrap::wrap(
            &toast.message,
            ((rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO) - 2) as usize,
        )
        .iter()
        .map(|x| Line::from(x.to_string()))
        .collect::<Vec<Line>>();
        let toast_height = lines.len() as u16 + 2;
        let toast_block = Block::default()
            .title(toast_title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(toast_style);
        let toast_paragraph = Paragraph::new(lines)
            .block(toast_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .style(toast_style);
        if toast_height + total_height_rendered > rect.size().height {
            debug!("Toast height is greater than the height of the screen");
            break;
        }
        rect.render_widget(
            Clear,
            Rect::new(
                x_offset,
                total_height_rendered,
                rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO,
                toast_height,
            ),
        );
        rect.render_widget(
            toast_paragraph,
            Rect::new(
                x_offset,
                total_height_rendered,
                rect.size().width / SCREEN_TO_TOAST_WIDTH_RATIO,
                toast_height,
            ),
        );
        total_height_rendered += toast_height;
        if total_height_rendered >= rect.size().height {
            debug!("Toast height is greater than the height of the screen");
            break;
        }
    }

    let text_offset = 15;
    let toast_count = app.widgets.toasts.len();
    let toast_count_text = format!(" {} Message(s)", toast_count);
    let toast_count_paragraph = Paragraph::new(toast_count_text)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::LEFT)
                .border_type(BorderType::Rounded),
        )
        .style(app.current_theme.general_style);
    let message_area = Rect::new(rect.size().width - text_offset, 0, text_offset, 1);

    render_blank_styled_canvas(rect, &app.current_theme, message_area, false);
    rect.render_widget(toast_count_paragraph, message_area);
}

pub fn render_debug_panel(rect: &mut Frame, app: &mut App) {
    let current_view = &app.state.current_view.to_string();
    let popup = if app.state.z_stack.is_empty() {
        "None".to_string()
    } else {
        app.state
            .z_stack
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(",\n")
    };
    let ui_render_time = if !app.state.ui_render_time.is_empty() {
        // Average render time
        let render_time =
            app.state.ui_render_time.iter().sum::<u128>() / app.state.ui_render_time.len() as u128;
        if render_time > 1000 {
            format!("{}ms", render_time as f64 / 1000_f64)
        } else {
            format!("{}μs", render_time)
        }
    } else {
        "None".to_string()
    };
    let current_board_id = app.state.current_board_id;
    let current_card_id = app.state.current_card_id;

    let debug_panel_area = top_left_rect(38, 10, rect.size());
    let strings = [
        format!("App status: {:?}", app.state.app_status),
        format!("View: {}", current_view),
        format!("Focus: {:?}", app.state.focus),
        format!("CMousePos: {:?}", app.state.current_mouse_coordinates),
        format!("Popup: {}", popup),
        format!("Avg Render Time: {}", ui_render_time),
        format!("CB-ID: {:?}", current_board_id),
        format!("CC-ID: {:?}", current_card_id),
    ];
    let strings = strings
        .iter()
        .flat_map(|s| {
            if s.len() > debug_panel_area.width as usize - 2 {
                // split on \n and get lines
                let mut lines = vec![];
                for line in s.split('\n') {
                    let mut line = line.to_string();
                    while line.len() > debug_panel_area.width as usize - 2 {
                        lines.push(format!(
                            "{}{}",
                            &line[..debug_panel_area.width as usize - 5],
                            "..."
                        ));
                        line = line[debug_panel_area.width as usize - 5..].to_string();
                    }
                    lines.push(line);
                }
                // Line::from(format!("{}{}", &s[..menu_area.width as usize - 5], "..."))
                lines
                    .iter()
                    .map(|l| Line::from(l.to_string()))
                    .collect::<Vec<Line>>()
            } else {
                vec![Line::from(s.to_string())]
            }
        })
        .collect::<Vec<Line>>();
    let debug_panel = Paragraph::new(strings).block(
        Block::default()
            .title("Debug Panel")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(app.current_theme.general_style)
            .border_style(app.current_theme.log_debug_style),
    );

    // get 5 lines at the bottom
    let logs_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(5)].as_ref())
        .split(rect.size());

    render_blank_styled_canvas(rect, &app.current_theme, debug_panel_area, true);
    rect.render_widget(debug_panel, debug_panel_area);

    // added logs for debugging
    render_blank_styled_canvas(rect, &app.current_theme, logs_chunks[1], true);
    render_logs(app, false, logs_chunks[1], rect, true);
}
