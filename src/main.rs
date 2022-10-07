use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, fs::{File}, path::Path};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal, style::{self, Color}, text::Text,
};
use std::env;

extern crate serde_json;
use serde_json::Value as JsonValue;

// starts from 0
static BOARD_LIMIT: usize = 2;
static CARD_LIMIT: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    env::set_var("RUST_BACKTRACE", "1");
    
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut playground = Playground::init();

    // create app and run it
    let res = run_app(&mut terminal, &mut playground);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// implement a card class that contains the title and the content
struct Card {
    title: String,
    content: String,
}

// implement a board class that holds a list of cards
struct Board {
    title: String,
    cards: Vec<Card>,
}

// implement a playground class that holds a list of boards
struct Playground {
    boards: Vec<Board>,
}

impl Playground {
    // initialize a playground with a default board and a default card
    fn init() -> Self {
        // get_cloud_save json string
        let cloud_save = Playground::get_cloud_save();
        // check if the json value cloud save is null
        if cloud_save.is_null() {
            // if it is null, create a new playground
            let mut playground = Playground { boards: Vec::new() };
            // create a new board
            let mut board = Board {
                title: String::from("Default Board"),
                cards: Vec::new()
            };
            // create a new card
            let card = Card {
                title: String::from("Title"),
                content: String::from("Content"),
            };
            // add the card to the board
            board.cards.push(card);
            // add the board to the playground
            playground.boards.push(board);
            // return the playground
            playground
        } else {
            // if it is not null, create a new playground
            let mut playground = Playground { boards: Vec::new() };
            // get the boards from the cloud save
            let boards = &cloud_save["Boards"];
            // loop through the boards
            for board in boards.as_array().unwrap() {
                // create a new board
                let mut new_board = Board {
                    title: String::from(board["Board_title"].as_str().unwrap()),
                    cards: Vec::new()
                };
                // get the cards from the board
                let cards = &board["Cards"];
                // check if the cards are null
                if !cards.is_null() {
                    // loop through the cards
                    for card in cards.as_array().unwrap() {
                        // create a new card
                        let new_card = Card {
                            title: card["Card_title"].to_string(),
                            content: card["Card_content"].to_string(),
                        };
                        // add the card to the board
                        new_board.cards.push(new_card);
                    }
                }
                // add the board to the playground
                playground.boards.push(new_board);
            }
            // return the playground
            playground
        }
        
    }


    // return error if the cloud save is not found
    fn get_cloud_save() -> JsonValue {
        // implement cloud save, return a playground with a default board and a default card for now in json format
        let json_file_path = Path::new("sample_data.json");
        let file = File::open(json_file_path);
        // check if the file is found, if found return the json string else return {}
        let default_data = if file.is_ok() {
            let file = File::open(json_file_path).unwrap();
            let json: JsonValue = serde_json::from_reader(file).unwrap();
            json
        } else {

            JsonValue::Null
        };
        default_data
    }

}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, playground: &mut Playground) -> io::Result<()> {
    let mut selection = (0,0);
    // make a dictionary of boards and the cards currently visible
    let mut visible_boards: Vec<(i32,Vec<i32>) > = Vec::new();
    // loop through the boards till at most 3 boards are visible
    // loop through the boards and determine visible cards and add them to the dictionary
    for (i, board) in playground.boards.iter().enumerate() {
        // if the board is not visible, break
        if i > BOARD_LIMIT {
            break;
        }
        // create a new vector of visible cards
        let mut visible_cards: Vec<i32> = Vec::new();
        // loop through the cards and determine visible cards and add them to the vector
        for (j, _card) in board.cards.iter().enumerate() {
            // if the card is not visible, break
            if j > CARD_LIMIT {
                break;
            }
            // add the card to the vector
            visible_cards.push(j as i32);
        }
        // add the board and the visible cards to the dictionary
        visible_boards.push((i as i32, visible_cards));
    }

    loop {
        terminal.draw(|f| ui(f, playground, selection, visible_boards.clone()))?; // draw the ui
        if let Event::Key(key) = event::read()? {

            // if right arrow key is pressed, move to the next board till length of boards
            // if left arrow key is pressed, move to the previous board till 0
            // if down arrow key is pressed, move to the next card till length of cards
            // if up arrow key is pressed, move to the previous card till 0

            if let KeyCode::Char('q') = key.code {
                return Ok(());
            }

            if let KeyCode::Right = key.code {
                selection.1 = 0;
                if selection.0 < (playground.boards.len() - 1) as i32 {
                    selection.0 += 1;
                }
                // if the board is not visible, make it visible. for example total 5 boards and first 3 are visible if selection.0 is 4, visible boards should be 2,3,4
                if selection.0 > visible_boards[2].0 {
                    // remove the first board from the visible boards
                    visible_boards.remove(0);
                    // add the next board to the visible boards
                    visible_boards.push((selection.0, Vec::new()));
                    // loop through the cards and determine visible cards and add them to the vector
                    for (j, _card) in playground.boards[selection.0 as usize].cards.iter().enumerate() {
                        // if the card is not visible, break
                        if j > CARD_LIMIT {
                            break;
                        }
                        // add the card to the vector
                        visible_boards[2].1.push(j as i32);
                    }
                }
            }

            if let KeyCode::Left = key.code {
                selection.1 = 0;
                if selection.0 > 0 {
                    selection.0 -= 1;
                }
                // if the board is not visible, make it visible. for example total 5 boards and last 3 are visible if selection.0 is 2, visible boards should be 0,1,2
                if selection.0 < visible_boards[0].0 {
                    // remove the last board from the visible boards
                    visible_boards.remove(2);
                    // add the previous board to the visible boards
                    visible_boards.insert(0, (selection.0, Vec::new()));
                    // loop through the cards and determine visible cards and add them to the vector
                    for (j, _card) in playground.boards[selection.0 as usize].cards.iter().enumerate() {
                        // if the card is not visible, break
                        if j > CARD_LIMIT {
                            break;
                        }
                        // add the card to the vector
                        visible_boards[0].1.push(j as i32);
                    }
                }
            }

            if let KeyCode::Down = key.code {
                if selection.1 < playground.boards[selection.0 as usize].cards.len() as i32 {
                    selection.1 += 1;
                }
            }

            if let KeyCode::Up = key.code {
                if selection.1 > 1 {
                    selection.1 -= 1;
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, playground: &mut Playground, selection: (i32,i32), visible_boards: Vec<(i32,Vec<i32>)>) {
    
    let selected_style = style::Style::default().fg(Color::White);
    let normal_style = style::Style::default().fg(Color::DarkGray);

    // initialise chunks based on the number of boards in the playground max 3 if more than 3 allow horizontal scrolling
    // if number of boards is 1 then take up the whole screen
    // if number of boards is 2 then take up half the screen

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .split(f.size());

    // draw the boards
    for i in 0..visible_boards.len() {
        let board = &playground.boards[visible_boards[i].0 as usize];
        let chunk = chunks[i];
        let block = if i == selection.0 as usize {
            Block::default().borders(Borders::ALL).title(board.title.as_str()).style(selected_style)
        } else {
            Block::default().borders(Borders::ALL).title(board.title.as_str()).style(normal_style)
        };

        // add the cards to the board with a margin of 1
        let card_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(chunk);

        for j in 0..visible_boards[i].1.len() {
            let card = &board.cards[visible_boards[i].1[j] as usize];
            let card_chunk = card_chunks[j];
            let card_block = if j == selection.1 as usize {
                Block::default().borders(Borders::ALL).title(card.title.as_str()).style(selected_style)
            } else {
                Block::default().borders(Borders::ALL).title(card.title.as_str()).style(normal_style)
            };
            
            let card_text = Text::raw(card.content.as_str());
            let card_paragraph = Paragraph::new(card_text).block(card_block);
            f.render_widget(card_paragraph, card_chunk);
        }
        f.render_widget(block, chunk);
    }
}