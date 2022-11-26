use tui::backend::Backend;
use tui::Frame;
use tui_logger::TuiLoggerWidget;
use tui::layout::{
    Alignment,
    Constraint,
    Direction,
    Layout,
    Rect
};
use tui::style::{
    Color,
    Style, Modifier,
};
use tui::text::{
    Span,
    Spans
};
use tui::widgets::{
    Block,
    BorderType,
    Borders,
    Paragraph,
    List,
    ListItem,
    ListState
};
use crate::constants::{
    APP_TITLE,
    MIN_TERM_WIDTH,
    MIN_TERM_HEIGHT,
};

use super::MainMenuItems;
use super::actions::{Actions, Action};
use super::kanban::Board;
use super::state::{UiMode};
use super::state::Focus;
use crate::app::App;
use crate::io::data_handler::get_config;

/// Main UI Drawing handler
pub fn draw<B>(rect: &mut Frame<B>, app: &App, config_state: &mut ListState, main_menu_state: &mut ListState)
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
    let current_board = &app.current_board;
    let current_card = &app.current_card;

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

        UiMode::TitleBody => {
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
        
        UiMode::BodyHelp => {
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

        UiMode::BodyLog => {
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

        UiMode::TitleBodyHelp => {
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

        UiMode::TitleBodyLog => {
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

        UiMode::BodyHelpLog => {
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

        UiMode::TitleBodyHelpLog => {
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
            let config = draw_config_list_selector(&app.focus);
            rect.render_stateful_widget(config, chunks[0], config_state);

            let config_help = draw_config_help(&app.focus);
            rect.render_widget(config_help, chunks[1]);

            let log = draw_logs(&app.focus);
            rect.render_widget(log, chunks[2]);
        }

        UiMode::EditConfig => {
            let area = centered_rect(70, 70, size);
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(40),
                        Constraint::Percentage(40),
                        Constraint::Length(4),
                    ]
                    .as_ref(),
                )
                .split(area);

            let config_item_index = &app.config_item_being_edited.unwrap_or(0);
            let list_items = get_config_items();
            let config_item_name = &list_items[*config_item_index];
            let config_item_value = list_items.iter().find(|&x| x == config_item_name).unwrap();
            let paragraph_text = format!("Current Value for {} \n\n{}",config_item_value,
                "Press 'i' to edit, or 'Esc' to cancel, Press 'Enter' to stop editing and press 'Enter' again to save");
            let paragraph_title = Spans::from(vec![Span::raw(config_item_name)]);
            let config_item = Paragraph::new(paragraph_text)
            .block(Block::default().borders(Borders::ALL).title(paragraph_title))
            .wrap(tui::widgets::Wrap { trim: false });
            let edit_item = Paragraph::new(&*app.current_user_input)
            .block(Block::default().borders(Borders::ALL).title("Edit"))
            .wrap(tui::widgets::Wrap { trim: false });

            let log = draw_logs(&app.focus);
            
            rect.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.current_user_input.len() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            );
            rect.render_widget(config_item, chunks[0]);
            rect.render_widget(edit_item, chunks[1]);
            rect.render_widget(log, chunks[2]);
        }
    
        UiMode::MainMenu => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Percentage(70),
                        Constraint::Length(4)
                    ]
                    .as_ref(),
                )
                .split(size);
            
            let title = draw_title(&app.focus);
            rect.render_widget(title, chunks[0]);
            
            let main_menu = draw_main_menu(&app.focus, app.main_menu.items.clone());
            rect.render_stateful_widget(main_menu, chunks[1], main_menu_state);

            let main_menu_help = draw_main_menu_help(&app.focus);
            rect.render_widget(main_menu_help, chunks[2]);
        }

        UiMode::HelpMenu => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(70),
                        Constraint::Length(4)
                    ]
                    .as_ref(),
                )
                .split(size);
            let help_menu = draw_help_menu(&app.focus);
            rect.render_widget(help_menu, chunks[0]);

            let log = draw_logs(&app.focus);
            rect.render_widget(log, chunks[1]);
        }

        UiMode::ViewCard => {
            todo!("ViewCard");
        }

    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ]
        .as_ref(),
    )
    .split(r);

Layout::default()
    .direction(Direction::Horizontal)
    .constraints(
        [
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ]
        .as_ref(),
    )
    .split(popup_layout[1])[1]
}

/// Draws size error screen if the terminal is too small
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

    let mut text = vec![Spans::from(Span::styled(msg, Style::default().fg(Color::LightRed)))];
    text.append(&mut vec![Spans::from(Span::raw("Resize the window to continue, or press 'q' to quit."))]);
    let body = Paragraph::new(text)
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Center);
    rect.render_widget(body, chunks[1]);
}

/// Draws the title bar
fn draw_title<'a>(focus: &Focus) -> Paragraph<'a> {
    // check if focus is on title
    let title_style = if matches!(focus, Focus::Title) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    Paragraph::new(APP_TITLE)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(title_style)
                .border_type(BorderType::Plain),
        )
}

/// Helper function to check terminal size
fn check_size(rect: &Rect) -> String {
    let mut msg = String::new();
    if rect.width < MIN_TERM_WIDTH {
        msg.push_str(&format!("For optimal viewing experience, Terminal width should be >= {}, (current {})",MIN_TERM_WIDTH, rect.width));
    }
    else if rect.height < MIN_TERM_HEIGHT {
        msg.push_str(&format!("For optimal viewing experience, Terminal height should be >= {}, (current {})",MIN_TERM_HEIGHT, rect.height));
    }
    else {
        msg.push_str("Size OK");
    }
    msg
}

/// Draws main screen with kanban boards
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

/// Draws Help section for normal mode
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

/// Draws help section for config mode
fn draw_config_help(focus: &Focus) -> Paragraph {
    let helpbox_style = if matches!(focus, Focus::ConfigHelp) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut help_spans = vec![];
    let keys_span = Span::styled("<Up>, <Down>", key_style);
    let action_span = Span::styled("Select config option", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);
    help_spans.push(Span::raw(" ; "));
    let keys_span = Span::styled("<Enter>", key_style);
    let action_span = Span::styled("Edit config option", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);
    let keys_span = Span::styled("<Esc>", key_style);
    let action_span = Span::styled("Exit config mode", help_style);
    help_spans.push(Span::raw(" ; "));
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);

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

/// Draws logs
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

/// Draws Main menu
fn draw_main_menu<'a>(focus: &Focus, main_menu_items: Vec<MainMenuItems>) -> List<'a> {
    let menu_style = if matches!(focus, Focus::MainMenu) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let list_items = main_menu_items
        .iter()
        .map(|i| ListItem::new(i.to_string()))
        .collect::<Vec<ListItem>>();
    List::new(list_items)
        .block(
            Block::default()
                .title("Main menu")
                .borders(Borders::ALL)
                .style(menu_style)
                .border_type(BorderType::Plain),
        )
        .highlight_style(Style::default().fg(Color::LightBlue))
        .highlight_symbol(">")
}

/// Draws Main menu help
fn draw_main_menu_help<'a>(focus: &Focus) -> Paragraph<'a> {
    let helpbox_style = if matches!(focus, Focus::MainMenuHelp) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut help_spans = vec![];
    let keys_span = Span::styled("<Up>, <Down>", key_style);
    let action_span = Span::styled("Select menu item", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);
    help_spans.push(Span::raw(" ; "));
    let keys_span = Span::styled("<Enter>", key_style);
    let action_span = Span::styled("Select menu item", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);
    let keys_span = Span::styled("<Esc>,<Ctrl+C>,<q>", key_style);
    let action_span = Span::styled("Quit", help_style);
    help_spans.push(Span::raw(" ; "));
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);

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

/// Draws Kanban boards
fn draw_board(current_board: String, current_card: String, focus: &Focus) {
    
}

/// Draws a card for a board
fn draw_card(current_card: String, focus: &Focus) {

}

/// Returns a list of ListItems for the config list selector
fn get_config_list_items<'action>() -> Vec<ListItem<'action>>
{
    let config_list = get_config_items();
    let mut config_spans = vec![];

    for (_i, config) in config_list.iter().enumerate() {
        config_spans.push(ListItem::new(Span::from(config.clone())));
    }
    return config_spans;
}

/// Draws config list selector
fn draw_config_list_selector(focus: &Focus) -> List<'static> {
    let config_style = if matches!(focus, Focus::Config) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let list_items = get_config_list_items();
    let config = List::new(list_items)
    .block(Block::default().borders(Borders::ALL).title("Config"))
    .highlight_style(
        Style::default()
        .bg(Color::LightGreen)
        .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol(">> ")
    .style(config_style);
    return config;
}

/// returns a list of all config items as a vector of strings
fn get_config_items() -> Vec<String>
{
    let config = get_config();
    let config_list = config.to_list();
    return config_list;
}

/// Draws Help Menu
fn draw_help_menu<'a>(focus: &Focus) -> Paragraph<'a> {
    let helpbox_style = if matches!(focus, Focus::Help) {
        Style::default().fg(Color::LightBlue)
    } else {
        Style::default().fg(Color::White)
    };
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);
    let general_style = Style::default().fg(Color::White);

    let general_help = "General help ; ";
    // TODO: Add help text

    let mut help_spans = vec![];
    help_spans.push(Span::styled(general_help, general_style));
    let keys_span = Span::styled("<Up>, <Down>", key_style);
    let action_span = Span::styled("Scroll up/down", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);
    help_spans.push(Span::raw(" ; "));
    let keys_span = Span::styled("<Left>, <Right>", key_style);
    let action_span = Span::styled("Scroll left/right", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);
    help_spans.push(Span::raw(" ; "));
    let keys_span = Span::styled("<Esc>", key_style);
    let action_span = Span::styled("Exit", help_style);
    help_spans.push(keys_span);
    help_spans.push(Span::raw(" - "));
    help_spans.push(action_span);

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