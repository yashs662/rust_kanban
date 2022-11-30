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
    Style,
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
    ListItem, ListState, Gauge,
};
use crate::constants::{
    APP_TITLE,
    MIN_TERM_WIDTH,
    MIN_TERM_HEIGHT,
    NO_OF_BOARDS_PER_PAGE,
    DEFAULT_BOARD_TITLE_LENGTH,
    DEFAULT_CARD_TITLE_LENGTH,
    NO_OF_CARDS_PER_BOARD, LIST_SELECT_STYLE, LIST_SELECTED_SYMBOL
};

use super::{MainMenuItem, App, MainMenu};
use super::actions::{Actions, Action};
use super::state::{Focus, AppStatus};
use crate::io::data_handler::{get_config, get_available_local_savefiles};

/// Draws main screen with kanban boards
pub fn render_zen_mode<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(rect.size());

    render_body(rect, chunks[0], app,);
}

pub fn render_title_body<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(80),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let title = draw_title(&app.focus);
    rect.render_widget(title, chunks[0]);
    
    render_body(rect, chunks[1], app);
}

pub fn render_body_help<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(85),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let actions = app.actions();
    
    render_body(rect, chunks[0], app);

    let help = draw_help(actions, &app.focus);
    rect.render_widget(help, chunks[1]);
}

pub fn render_body_log<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(80),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    render_body(rect, chunks[0], app);

    let log = draw_logs(&app.focus, true);
    rect.render_widget(log, chunks[1]);
}

pub fn render_title_body_help<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(75),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let actions = app.actions();

    let title = draw_title(&app.focus);
    rect.render_widget(title, chunks[0]);

    render_body(rect, chunks[1], app);

    let help = draw_help(actions, &app.focus);
    rect.render_widget(help, chunks[2]);
}

pub fn render_title_body_log<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(75),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let title = draw_title(&app.focus);
    rect.render_widget(title, chunks[0]);

    render_body(rect, chunks[1], app);

    let log = draw_logs(&app.focus, true);
    rect.render_widget(log, chunks[2]);
}

pub fn render_body_help_log<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Length(5),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let actions = app.actions();

    render_body(rect, chunks[0], app);

    let help = draw_help(actions, &app.focus);
    rect.render_widget(help, chunks[1]);

    let log = draw_logs(&app.focus, true);
    rect.render_widget(log, chunks[2]);
}

pub fn render_title_body_help_log<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(5),
                Constraint::Length(8),
            ]
            .as_ref(),
        )
        .split(rect.size());

    let actions = app.actions();

    let title = draw_title(&app.focus);
    rect.render_widget(title, chunks[0]);

    render_body(rect, chunks[1], app);

    let help = draw_help(actions, &app.focus);
    rect.render_widget(help, chunks[2]);

    let log = draw_logs(&app.focus, true);
    rect.render_widget(log, chunks[3]);
}

pub fn render_config<'a,B>(rect: &mut Frame<B>, app: &App, config_state: &mut ListState)
where
    B: Backend,
{
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
        .split(rect.size());
    let config = draw_config_list_selector(&app.focus);
    rect.render_stateful_widget(config, chunks[0], config_state);

    let config_help = draw_config_help(&app.focus);
    rect.render_widget(config_help, chunks[1]);

    let log = draw_logs(&app.focus, true);
    rect.render_widget(log, chunks[2]);
}

pub fn render_edit_config<'a,B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let area = centered_rect(70, 70, rect.size());
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
    let binding = String::from("");
    let config_item_value = list_items.iter().find(|&x| x == config_item_name).unwrap_or(&binding);
    let paragraph_text = format!("Current Value for {} \n\n{}",config_item_value,
        "Press 'i' to edit, or 'Esc' to cancel, Press 'Enter' to stop editing and press 'Enter' again to save");
    let paragraph_title = Spans::from(vec![Span::raw(config_item_name)]);
    let config_item = Paragraph::new(paragraph_text)
    .block(Block::default().borders(Borders::ALL).title(paragraph_title))
    .wrap(tui::widgets::Wrap { trim: false });
    let edit_item = Paragraph::new(&*app.current_user_input)
    .block(Block::default().borders(Borders::ALL).title("Edit"))
    .wrap(tui::widgets::Wrap { trim: false });

    let log = draw_logs(&app.focus, true);
    
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

pub fn render_main_menu<'a,B>(rect: &mut Frame<B>, app: &App, main_menu_state: &mut ListState)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Percentage(50),
                Constraint::Length(4),
                Constraint::Length(8)
            ]
            .as_ref(),
        )
        .split(rect.size());
    
    let title = draw_title(&app.focus);
    rect.render_widget(title, chunks[0]);
    
    let main_menu = draw_main_menu(&app.focus, MainMenu::all());
    rect.render_stateful_widget(main_menu, chunks[1], main_menu_state);

    let main_menu_help = draw_main_menu_help(&app.focus, app.actions());
    rect.render_widget(main_menu_help, chunks[2]);

    let log = draw_logs(&app.focus, true);
    rect.render_widget(log, chunks[3]);
}

pub fn render_help_menu<'a,B>(rect: &mut Frame<B>, focus: &Focus)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(70),
                Constraint::Length(4)
            ]
            .as_ref(),
        )
        .split(rect.size());
    let help_menu = draw_help_menu(focus);
    rect.render_widget(help_menu, chunks[0]);

    let log = draw_logs(focus, true);
    rect.render_widget(log, chunks[1]);
}

pub fn render_logs_only<'a,B>(rect: &mut Frame<B>, focus: &Focus)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(rect.size());
    let log = draw_logs(focus, false);
    rect.render_widget(log, chunks[0]);
}

/// Draws Help section for normal mode
fn draw_help<'a>(actions: &Actions, focus: &Focus) -> Paragraph<'a> {
    let helpbox_style = if matches!(focus, Focus::Help) {
        Style::default().fg(Color::LightYellow)
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
        Style::default().fg(Color::LightYellow)
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
fn draw_logs<'a>(focus: &Focus, enable_focus_highlight: bool) -> TuiLoggerWidget<'a> {
    let logbox_style = if matches!(focus, Focus::Log) && enable_focus_highlight {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    TuiLoggerWidget::default()
        .style_error(Style::default().fg(Color::LightRed))
        .style_debug(Style::default().fg(Color::LightGreen))
        .style_warn(Style::default().fg(Color::LightYellow))
        .style_trace(Style::default().fg(Color::Gray))
        .style_info(Style::default().fg(Color::LightCyan))
        .block(
            Block::default()
                .title("Logs")
                .border_style(logbox_style)
                .borders(Borders::ALL),
        )
}

/// Draws Main menu
fn draw_main_menu<'a>(focus: &Focus, main_menu_items: Vec<MainMenuItem>) -> List<'a> {
    let menu_style = if matches!(focus, Focus::MainMenu) {
        Style::default().fg(Color::LightYellow)
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
        .highlight_style(LIST_SELECT_STYLE)
        .highlight_symbol(LIST_SELECTED_SYMBOL)
}

/// Draws Main menu help
fn draw_main_menu_help<'a>(focus: &Focus, actions: &Actions) -> Paragraph<'a> {
    let helpbox_style = if matches!(focus, Focus::MainMenuHelp) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut help_spans = vec![];
    let actions_iter = actions.actions().iter();
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
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let list_items = get_config_list_items();
    let config = List::new(list_items)
    .block(Block::default().borders(Borders::ALL).title("Config"))
    .highlight_style(LIST_SELECT_STYLE)
    .highlight_symbol(LIST_SELECTED_SYMBOL)
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
        Style::default().fg(Color::LightYellow)
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

/// Draws Kanban boards
pub fn render_body<'a,B>(rect: &mut Frame<B>, area: Rect, app: &App)
where
    B: Backend,
{
    let mut more_boards = false;
    let mut more_cards = false;
    let focus = &app.focus;
    let boards = &app.boards;
    let current_board = &app.state.current_board.unwrap_or(0);
    let current_card = &app.state.current_card.unwrap_or(0);
    let focused_board_style = Style::default().fg(Color::LightYellow);
    let focused_card_style = Style::default().fg(Color::LightYellow);

    // check if self.visible_boards_and_cards is empty
    if app.visible_boards_and_cards.is_empty() {
        let empty_paragraph = Paragraph::new("No boards found, press <b> to add a board")
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title("Boards")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .border_type(BorderType::Plain),
            )
            .wrap(tui::widgets::Wrap { trim: true });
        rect.render_widget(empty_paragraph, area);
        return;
    }
    
    // make a list of constraints depending on NO_OF_BOARDS_PER_PAGE constant
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(99),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(area);
    let mut constraints = vec![];
    // check if length of boards is more than NO_OF_BOARDS_PER_PAGE
    if boards.len() > NO_OF_BOARDS_PER_PAGE.into() {
        for _i in 0..NO_OF_BOARDS_PER_PAGE {
            constraints.push(Constraint::Percentage(100 / NO_OF_BOARDS_PER_PAGE as u16));
        }
        constraints.push(Constraint::Length(2));
        more_boards = true
    } else {
        for _i in 0..boards.len() {
            constraints.push(Constraint::Percentage(100 / boards.len() as u16));
        }
    }
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.as_ref())
        .split(chunks[0]);
    // visible_boards_and_cards: Vec<BTreeMap<String, Vec<String>>>
    let visible_boards_and_cards = app.visible_boards_and_cards.clone();
    for (board_index, board_and_card_tuple) in visible_boards_and_cards.iter().enumerate() {
        // render board with title in board chunks alongside with cards in card chunks of the board
        let board = &boards[board_index];
        let board_title = board.name.clone();
        let board_id = board.id.clone();
        let board_cards = board_and_card_tuple.1;
        // if board title is longer than DEFAULT_BOARD_TITLE_LENGTH, truncate it and add ... at the end
        let board_title = if board_title.len() > DEFAULT_BOARD_TITLE_LENGTH.into() {
            format!("{}...", &board_title[0..DEFAULT_BOARD_TITLE_LENGTH as usize])
        } else {
            board_title
        };
        let board_title = format!("{} ({})", board_title, board_cards.len());
        let board_title = if board_index as u128 == *current_board {
            format!("{} {}", ">>", board_title)
        } else {
            board_title
        };

        // check if length of cards is more than NO_OF_CARDS_PER_BOARD constant
        let mut card_constraints = vec![];
        if board_cards.len() > NO_OF_CARDS_PER_BOARD.into() {
            for _i in 0..NO_OF_CARDS_PER_BOARD {
                card_constraints.push(Constraint::Percentage(90 / NO_OF_CARDS_PER_BOARD as u16));
            }
            card_constraints.push(Constraint::Length(2));
            more_cards = true
        } else {
            for _i in 0..board_cards.len() {
                card_constraints.push(Constraint::Percentage(100 / board_cards.len() as u16));
            }
        }

        let card_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(card_constraints.as_ref())
            .split(board_chunks[board_index]);

        for (card_index, card_id) in board_cards.iter().enumerate() {
            // unwrap card if panic skip it and log it
            let mut card = board.get_card(*card_id);
            // check if card is None, if so skip it and log it
            if card.is_none() {
                continue;
            } else {
                card = Some(card.unwrap());
            }
            let card_title = card.unwrap().name.clone();
            let card_title = if card_title.len() > DEFAULT_CARD_TITLE_LENGTH.into() {
                format!("{}...", &card_title[0..DEFAULT_CARD_TITLE_LENGTH as usize])
            } else {
                card_title
            };

            let card_title = if card_index as u128 == *current_card {
                format!("{} {}", ">>", card_title)
            } else {
                card_title
            };

            let card_description = card.unwrap().description.clone();

            // if card id is same as current_card, highlight it
            let card_style = if card_index as u128 == *current_card && matches!(focus, Focus::Body){
                focused_card_style
            } else {
                Style::default()
            };

            let card_paragraph = Paragraph::new(card_description)
                .style(Style::default())
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .title(&*card_title)
                        .borders(Borders::ALL)
                        .style(card_style)
                        .border_type(BorderType::Plain),
                )
                .wrap(tui::widgets::Wrap { trim: true });

            rect.render_widget(card_paragraph, card_chunks[card_index]);

        }

        if more_cards {
            let more_cards_paragraph = Paragraph::new("...")
                .style(Style::default())
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .title("...")
                        .borders(Borders::ALL)
                        .style(Style::default())
                        .border_type(BorderType::Plain),
                )
                .wrap(tui::widgets::Wrap { trim: true });

            rect.render_widget(more_cards_paragraph, card_chunks[card_chunks.len() - 1]);
        }
        
        let board_style = if board_id == *current_board && matches!(focus, Focus::Body) {
            focused_board_style
        } else {
            Style::default()
        };
        
        let board_block = Block::default()
            .title(&*board_title)
            .borders(Borders::ALL)
            .style(board_style)
            .border_type(BorderType::Plain);
        rect.render_widget(board_block, board_chunks[board_index]);
    }

    if more_boards {
        let more_boards_paragraph = Paragraph::new("...")
            .style(Style::default())
            .alignment(Alignment::Left)
            .block(
                Block::default()
                    .title("...")
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(BorderType::Plain),
            )
            .wrap(tui::widgets::Wrap { trim: true });

        rect.render_widget(more_boards_paragraph, board_chunks[board_chunks.len() -1]);
    }

    // draw line_gauge in chunks[1]
    // get the index of the current board in boards and set percentage
    let current_board_id = app.state.current_board.unwrap_or(0);
    // get the index of the board with the id
    let current_board_index = boards
        .iter()
        .position(|board| board.id == current_board_id)
        .unwrap_or(0);
    let percentage = (current_board_index as f64 / boards.len() as f64) * 100.0;
    let line_gauge = Gauge::default()
        .block(Block::default())
        .gauge_style(Style::default().fg(Color::Green))
        .percent(percentage as u16);
    rect.render_widget(line_gauge, chunks[1]);
    
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
pub fn draw_size_error<B>(rect: &mut Frame<B>, size: &Rect, msg: String)
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
pub fn draw_title<'a>(focus: &Focus) -> Paragraph<'a> {
    // check if focus is on title
    let title_style = if matches!(focus, Focus::Title) {
        Style::default().fg(Color::LightYellow)
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
pub fn check_size(rect: &Rect) -> String {
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

pub fn render_new_board_form<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    // make a form for the Board struct
    // take name and description where description is optional
    // submit button

    let name_style = if matches!(app.focus, Focus::NewBoardName) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let description_style = if matches!(app.focus, Focus::NewBoardDescription) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let submit_style = if matches!(app.focus, Focus::SubmitButton) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Length(3),
            ].as_ref())
        .split(rect.size());

    let title_paragraph = Paragraph::new("Create a new Board")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let board_name_field = app.state.new_board_form[0].clone();
    let board_description_field = app.state.new_board_form[1].clone();
    let board_name = Paragraph::new(board_name_field)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(name_style)
                .border_type(BorderType::Plain)
                .title("Board Name")
        );
    rect.render_widget(board_name, chunks[1]);

    let board_description = Paragraph::new(board_description_field)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(description_style)
                .border_type(BorderType::Plain)
                .title("Board Description")
        );
    rect.render_widget(board_description, chunks[2]);

    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);
    let help_text = Spans::from(vec![
        Span::styled("<i>", key_style),
        Span::styled(" to start typing", help_style),
        Span::raw(" | "),
        Span::styled("<esc>", key_style),
        Span::styled(" to stop typing", help_style),
        Span::raw(" ; "),
        Span::styled("<Tab>", key_style),
        Span::styled(" to switch focus", help_style),
        Span::raw(" ; "),
        Span::styled("<Enter>", key_style),
        Span::styled(" to submit", help_style),
        Span::raw(" ; "),
        Span::styled("<Esc>", key_style),
        Span::styled(" to cancel", help_style),
    ]);
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    rect.render_widget(help_paragraph, chunks[3]);

    let submit_button = Paragraph::new("Submit")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(submit_style)
                .border_type(BorderType::Plain),
        );
    rect.render_widget(submit_button, chunks[4]);

    if app.focus == Focus::NewBoardName && app.state.status == AppStatus::UserInput{
        rect.set_cursor(
            // Put cursor past the end of the input text
            chunks[1].x + app.state.new_board_form[0].len() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[1].y + 1,
        );
    } else if app.focus == Focus::NewBoardDescription && app.state.status == AppStatus::UserInput{
        rect.set_cursor(
            // Put cursor past the end of the input text
            chunks[2].x + app.state.new_board_form[1].len() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[2].y + 1,
        );
    }
}

pub fn render_new_card_form<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let name_style = if matches!(app.focus, Focus::NewCardName) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let description_style = if matches!(app.focus, Focus::NewCardDescription) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let due_date_style = if matches!(app.focus, Focus::NewCardDueDate) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };
    let submit_style = if matches!(app.focus, Focus::SubmitButton) {
        Style::default().fg(Color::LightYellow)
    } else {
        Style::default().fg(Color::White)
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            ].as_ref())
        .split(rect.size());

    let title_paragraph = Paragraph::new("Create a new Card")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let card_name_field = app.state.new_card_form[0].clone();
    let card_description_field = app.state.new_card_form[1].clone();
    let card_due_date_field = app.state.new_card_form[2].clone();
    let card_name = Paragraph::new(card_name_field)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(name_style)
                .border_type(BorderType::Plain)
                .title("Card Name")
        );
    rect.render_widget(card_name, chunks[1]);

    let card_description = Paragraph::new(card_description_field)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(description_style)
                .border_type(BorderType::Plain)
                .title("Card Description")
        );
    rect.render_widget(card_description, chunks[2]);

    let card_due_date = Paragraph::new(card_due_date_field)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(due_date_style)
                .border_type(BorderType::Plain)
                .title("Card Due Date")
        );
    rect.render_widget(card_due_date, chunks[3]);

    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);
    let help_text = Spans::from(vec![
        Span::styled("<i>", key_style),
        Span::styled(" to start typing", help_style),
        Span::raw(" | "),
        Span::styled("<esc>", key_style),
        Span::styled(" to stop typing", help_style),
        Span::raw(" ; "),
        Span::styled("<Tab>", key_style),
        Span::styled(" to switch focus", help_style),
        Span::raw(" ; "),
        Span::styled("<Enter>", key_style),
        Span::styled(" to submit", help_style),
        Span::raw(" ; "),
        Span::styled("<Esc>", key_style),
        Span::styled(" to cancel", help_style),
    ]);

    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    rect.render_widget(help_paragraph, chunks[4]);

    let submit_button = Paragraph::new("Submit")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(submit_style)
                .border_type(BorderType::Plain),
        );
    rect.render_widget(submit_button, chunks[5]);

    if app.focus == Focus::NewCardName && app.state.status == AppStatus::UserInput{
        rect.set_cursor(
            // Put cursor past the end of the input text
            chunks[1].x + app.state.new_card_form[0].len() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[1].y + 1,
        );
    } else if app.focus == Focus::NewCardDescription && app.state.status == AppStatus::UserInput{
        rect.set_cursor(
            // Put cursor past the end of the input text
            chunks[2].x + app.state.new_card_form[1].len() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[2].y + 1,
        );
    } else if app.focus == Focus::NewCardDueDate && app.state.status == AppStatus::UserInput{
        rect.set_cursor(
            // Put cursor past the end of the input text
            chunks[3].x + app.state.new_card_form[2].len() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[3].y + 1,
        );
    }
}

pub fn render_load_save<B>(rect: &mut Frame<B>, load_save_state: &mut ListState)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(70),
            Constraint::Length(3),
            ].as_ref())
        .split(rect.size());

    let title_paragraph = Paragraph::new("Load a Save")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    rect.render_widget(title_paragraph, chunks[0]);

    let item_list = get_available_local_savefiles();
    // make a list from the Vec<string> of savefiles
    let items: Vec<ListItem> = item_list
        .iter()
        .map(|i| ListItem::new(i.to_string()))
        .collect();
    let choice_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Available Saves"))
        .highlight_style(LIST_SELECT_STYLE)
        .highlight_symbol(LIST_SELECTED_SYMBOL);
    rect.render_stateful_widget(choice_list, chunks[1], load_save_state);

    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);
    let help_text = Spans::from(vec![
        Span::styled("<Up>", key_style),
        Span::styled(" and ", help_style),
        Span::styled("<Down>", key_style),
        Span::styled(" to navigate", help_style),
        Span::raw(" ; "),
        Span::styled("<Enter>", key_style),
        Span::styled(" to submit", help_style),
        Span::raw(" ; "),
        Span::styled("<Esc>", key_style),
        Span::styled(" to cancel", help_style),
    ]);
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        );
    rect.render_widget(help_paragraph, chunks[2]);
}