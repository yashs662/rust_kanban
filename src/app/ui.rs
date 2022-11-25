use log::{error, debug};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::{Frame};
use tui_logger::TuiLoggerWidget;

use super::actions::{Actions, Action};
use super::kanban::Board;
use super::state::{UiMode};
use super::state::Focus;
use crate::app::App;
use crate::io::data_handler::get_config;

pub fn draw<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let size = rect.size();

    let msg = check_size(&size);
    // match to check if msg is size OK
    if &msg == "Size OK" {
        // pass
    } else {
        // draw error message
        draw_size_error(rect, &size, msg);
        return;
    }

    let current_ui_mode = &app.ui_mode;

    match current_ui_mode {
        UiMode::Zen => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(100),
                    ]
                    .as_ref(),
                )
                .split(size);

            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[0]);
        }

        UiMode::Title => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(80),
                    ]
                    .as_ref(),
                )
                .split(size);

            let title = draw_title(&app.focus);
            rect.render_widget(title, chunks[0]);
            
            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[1]);
        }
        
        UiMode::Help => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(90),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(size);
            
            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[0]);

            let help = draw_help(app.actions(), &app.focus);
            rect.render_widget(help, chunks[1]);
        }

        UiMode::Log => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Length(8),
                    ]
                    .as_ref(),
                )
                .split(size);

            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[0]);

            let log = draw_logs(&app.focus);
            rect.render_widget(log, chunks[1]);
        }

        UiMode::TitleHelp => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(80),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(size);

            let title = draw_title(&app.focus);
            rect.render_widget(title, chunks[0]);

            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[1]);

            let help = draw_help(app.actions(), &app.focus);
            rect.render_widget(help, chunks[2]);
        }

        UiMode::TitleLog => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(70),
                        Constraint::Length(8),
                    ]
                    .as_ref(),
                )
                .split(size);

            let title = draw_title(&app.focus);
            rect.render_widget(title, chunks[0]);

            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[1]);

            let log = draw_logs(&app.focus);
            rect.render_widget(log, chunks[2]);
        }

        UiMode::HelpLog => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(70),
                        Constraint::Length(4),
                        Constraint::Length(8),
                    ]
                    .as_ref(),
                )
                .split(size);

            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[0]);

            let help = draw_help(app.actions(), &app.focus);
            rect.render_widget(help, chunks[1]);

            let log = draw_logs(&app.focus);
            rect.render_widget(log, chunks[2]);
        }

        UiMode::TitleHelpLog => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(60),
                        Constraint::Length(4),
                        Constraint::Length(8),
                    ]
                    .as_ref(),
                )
                .split(size);

            let title = draw_title(&app.focus);
            rect.render_widget(title, chunks[0]);

            let body = draw_body(&app.focus, &app.boards);
            rect.render_widget(body, chunks[1]);

            let help = draw_help(app.actions(), &app.focus);
            rect.render_widget(help, chunks[2]);

            let log = draw_logs(&app.focus);
            rect.render_widget(log, chunks[3]);
        }

        UiMode::Config => {
            draw_config(rect, app);
        }
    }
}

fn draw_config<B>(rect: &mut Frame<B>, app: &mut App)
where
    B: Backend,
{
    let config = get_config();
    let config_list = config.to_list();
    // if len of config_list is 0, then set UiMode to Title and put log message
    if config_list.len() == 0 {
        error!("No config file found. Please create a config file at ~/.config/zenith/config.toml");
        return;
    }
    // check if current_option is in range of config_list
    if app.current_obj_index >= config_list.len() {
        // if not, set current_option to 0
        app.current_obj_index = 0;
        debug!("current_option is out of range. Setting to 0");
    }
    debug!("current_option: {}", app.current_obj_index);

    // if app.go_up is true, then decrement current_option
    if app.go_up {
        if app.current_obj_index > 0 {
            app.current_obj_index -= 1;
        }
        app.go_up = false;
    }
    // if app.go_down is true, then increment current_option
    if app.go_down {
        if app.current_obj_index < config_list.len() - 1 {
            app.current_obj_index += 1;
        }
        app.go_down = false;
    }
    let highlight_style = Style::default().bg(Color::LightGreen).fg(Color::Black);
    let normal_style = Style::default().bg(Color::Black).fg(Color::White);

    let mut config_spans = vec![];

    for (i, config) in config_list.iter().enumerate() {
        let style = if i == app.current_obj_index {
            highlight_style
        } else {
            normal_style
        };
        config_spans.push(Span::styled(config, style));
        // if not last element, add newline
        if i != config_list.len() - 1 {
            config_spans.push(Span::raw("\n"));
        } 
    }

    let config_text = Spans::from(config_spans);
    let config_paragraph = Paragraph::new(config_text)
        .block(Block::default().borders(Borders::ALL).title("Config"))
        .alignment(Alignment::Left)
        .wrap(tui::widgets::Wrap { trim: false });

    rect.render_widget(config_paragraph, rect.size());
}

fn draw_size_error<B>(rect: &mut Frame<B>, size: &Rect, msg: String)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(*size);

    let title = draw_title(&Focus::default());
    rect.render_widget(title, chunks[0]);

    let body = draw_body_error(msg);
    rect.render_widget(body, chunks[1]);
}

fn draw_body_error<'a>(msg: String) -> Paragraph<'a> {
    let mut text = vec![Spans::from(Span::styled(msg, Style::default().fg(Color::LightRed)))];
    text.append(&mut vec![Spans::from(Span::raw("Resize the window to continue, or press 'q' to quit."))]);
    Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center)
}

fn draw_title<'a>(focus: &Focus) -> Paragraph<'a> {
    // check if focus is on title
    let title_style = if matches!(focus, Focus::Title) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    Paragraph::new("Rust 🦀 Kanban")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(title_style)
                .border_type(BorderType::Plain),
        )
}

fn check_size(rect: &Rect) -> String {
    let mut msg = String::new();
    if rect.width < 80 {
        msg.push_str(&format!("Terminal width should be >= 80, (current {})", rect.width));
    }
    else if rect.height < 28 {
        msg.push_str(&format!("Terminal height should be >= 28, (current {})", rect.height));
    }
    else {
        msg.push_str("Size OK");
    }
    msg
}

fn draw_body<'a>(focus: &Focus, boards: &Vec<Board>) -> Paragraph<'a> {
    let body_style = if matches!(focus, Focus::Body) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    Paragraph::new("Temp")
        .style(body_style)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(body_style)
                .border_type(BorderType::Plain),
        )
}

fn draw_help<'a>(actions: &Actions, focus: &Focus) -> Paragraph<'a> {
    let helpbox_style = if matches!(focus, Focus::Help) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    // make a new string with the format key - action, or key1, key2 - action if there are multiple keys and join all pairs with ;

    let actions_iter = actions.actions().iter();
    let mut help_spans = vec![];
    for action in actions_iter {
        // check if action is SetUiMode if so then keys should be changed to string <1..8>
        let keys = action.keys();
        let mut keys_span = if keys.len() > 1 {
            let keys_str = keys
                .iter()
                .map(|k| k.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            Span::styled(keys_str, key_style)
        } else {
            Span::styled(keys[0].to_string(), key_style)
        };
        let action_span = Span::styled(action.to_string(), help_style);
        keys_span = match action {
            Action::SetUiMode => Span::styled("<1..8>", key_style),
            _ => keys_span,
        };
        help_spans.push(keys_span);
        help_spans.push(Span::raw(" - "));
        help_spans.push(action_span);
        // if action is not last
        if action != actions.actions().last().unwrap() {
            help_spans.push(Span::raw(" ; "));
        }
    }
    let help_span = Spans::from(help_spans);

    Paragraph::new(help_span)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL)
                .style(helpbox_style)
                .border_type(BorderType::Plain),
        )
        .wrap(tui::widgets::Wrap { trim: true })
}

fn draw_logs<'a>(focus: &Focus) -> TuiLoggerWidget<'a> {
    let logbox_style = if matches!(focus, Focus::Log) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::Blue))
        .block(
            Block::default()
                .title("Logs")
                .border_style(logbox_style)
                .borders(Borders::ALL),
        )
}
