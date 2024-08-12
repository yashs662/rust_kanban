use crate::{
    app::DateTimeFormat,
    constants::{FIELD_NA, FIELD_NOT_SET},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Board {
    pub cards: Cards,
    pub description: String,
    pub id: (u64, u64),
    pub name: String,
}

impl Board {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            id: get_id(),
            name: name.to_owned(),
            description: description.to_owned(),
            cards: Cards::default(),
        }
    }

    pub fn from_json(value: &Value) -> Result<Self, String> {
        let id = match value["id"].as_array() {
            Some(id) => {
                let mut temp_id = (0, 0);
                let id_1 = match id[0].as_u64() {
                    Some(id_1) => id_1,
                    None => {
                        temp_id = get_id();
                        temp_id.0
                    }
                };
                let id_2 = match id[1].as_u64() {
                    Some(id_2) => id_2,
                    None => {
                        if temp_id == (0, 0) {
                            temp_id = get_id();
                        }
                        temp_id.1
                    }
                };
                (id_1, id_2)
            }
            None => {
                let temp_id = get_id();
                (temp_id.0, temp_id.1)
            }
        };
        let name = match value["name"].as_str() {
            Some(name) => name,
            None => return Err("board name is invalid for board".to_string()),
        };
        let description = match value["description"].as_str() {
            Some(description) => description,
            None => return Err("board description is invalid for board".to_string()),
        };
        // Mainly for backwards compatibility, recent versions use value["cards"]["cards"] due to Cards being a struct
        let cards = match value["cards"].as_array() {
            Some(cards) => cards
                .iter()
                .map(Card::from_json)
                .collect::<Result<Cards, String>>()?,
            None => match value["cards"]["cards"].as_array() {
                Some(cards) => cards
                    .iter()
                    .map(Card::from_json)
                    .collect::<Result<Cards, String>>()?,
                None => return Err("board cards is invalid for board".to_string()),
            },
        };

        Ok(Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            cards,
        })
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cards: Cards::default(),
            description: String::from("Default Board Description"),
            id: get_id(),
            name: String::from("Default Board"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Boards {
    boards: Vec<Board>,
}

impl Boards {
    pub fn add_board(&mut self, board: Board) {
        self.boards.push(board);
    }
    pub fn get_board_with_id(&self, board_id: (u64, u64)) -> Option<&Board> {
        self.boards.iter().find(|b| b.id == board_id)
    }
    pub fn get_mut_board_with_id(&mut self, board_id: (u64, u64)) -> Option<&mut Board> {
        self.boards.iter_mut().find(|b| b.id == board_id)
    }
    pub fn get_board_with_index(&self, index: usize) -> Option<&Board> {
        self.boards.get(index)
    }
    pub fn get_mut_board_with_index(&mut self, index: usize) -> Option<&mut Board> {
        self.boards.get_mut(index)
    }
    pub fn get_boards(&self) -> &Vec<Board> {
        &self.boards
    }
    pub fn get_mut_boards(&mut self) -> &mut Vec<Board> {
        &mut self.boards
    }
    pub fn set_boards(&mut self, boards: Boards) {
        self.boards = boards.boards;
    }
    pub fn is_empty(&self) -> bool {
        self.boards.is_empty()
    }
    pub fn get_first_board_id(&self) -> Option<(u64, u64)> {
        self.boards.first().map(|b| b.id)
    }
    pub fn get_board_index(&self, board_id: (u64, u64)) -> Option<usize> {
        self.boards.iter().position(|b| b.id == board_id)
    }
    pub fn len(&self) -> usize {
        self.boards.len()
    }
    pub fn remove_board_with_id(&mut self, board_id: (u64, u64)) {
        self.boards.retain(|b| b.id != board_id);
    }
    pub fn reset(&mut self) {
        self.boards.clear();
    }
    pub fn find_board_with_card_id(&self, card_id: (u64, u64)) -> Option<(usize, &Board)> {
        self.boards
            .iter()
            .enumerate()
            .find(|(_, b)| b.cards.get_card_with_id(card_id).is_some())
    }
}

impl From<Vec<Board>> for Boards {
    fn from(boards: Vec<Board>) -> Self {
        Self { boards }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardStatus {
    Active,
    Complete,
    Stale,
}

impl fmt::Display for CardStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CardStatus::Active => write!(f, "Active"),
            CardStatus::Complete => write!(f, "Complete"),
            CardStatus::Stale => write!(f, "Stale"),
        }
    }
}

impl CardStatus {
    pub fn all() -> Vec<CardStatus> {
        vec![CardStatus::Active, CardStatus::Complete, CardStatus::Stale]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CardPriority {
    High,
    Low,
    Medium,
}

impl fmt::Display for CardPriority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CardPriority::High => write!(f, "High"),
            CardPriority::Low => write!(f, "Low"),
            CardPriority::Medium => write!(f, "Medium"),
        }
    }
}

impl CardPriority {
    pub fn all() -> Vec<CardPriority> {
        vec![CardPriority::Low, CardPriority::Medium, CardPriority::High]
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Card {
    pub card_status: CardStatus,
    pub comments: Vec<String>,
    pub date_completed: String,
    pub date_created: String,
    pub date_modified: String,
    pub description: String,
    pub due_date: String,
    pub id: (u64, u64),
    pub name: String,
    pub priority: CardPriority,
    pub tags: Vec<String>,
}

impl Card {
    pub fn new(
        name: &str,
        description: &str,
        due_date: &str,
        priority: CardPriority,
        tags: Vec<String>,
        comments: Vec<String>,
        date_time_format: DateTimeFormat,
    ) -> Self {
        let name = if name.is_empty() { FIELD_NOT_SET } else { name };
        let description = if description.is_empty() {
            FIELD_NOT_SET
        } else {
            description
        };
        let due_date = if due_date.is_empty() {
            FIELD_NOT_SET
        } else {
            due_date
        };
        let priority = if priority.to_string().is_empty() {
            CardPriority::Low
        } else {
            priority
        };
        let tags = if tags.is_empty() { Vec::new() } else { tags };
        let comments = if comments.is_empty() {
            Vec::new()
        } else {
            comments
        };

        let corrected_date_time_format = DateTimeFormat::add_time_to_date_format(date_time_format);

        Self {
            id: get_id(),
            name: name.to_string(),
            description: description.to_string(),
            date_created: chrono::Local::now()
                .format(corrected_date_time_format.to_parser_string())
                .to_string(),
            date_modified: chrono::Local::now()
                .format(corrected_date_time_format.to_parser_string())
                .to_string(),
            due_date: due_date.to_string(),
            date_completed: FIELD_NA.to_string(),
            priority,
            card_status: CardStatus::Active,
            tags,
            comments,
        }
    }

    pub fn from_json(value: &Value) -> Result<Self, String> {
        let id = match value["id"].as_array() {
            Some(id) => {
                let mut temp_id = (0, 0);
                let id_1 = match id[0].as_u64() {
                    Some(id_1) => id_1,
                    None => {
                        temp_id = get_id();
                        temp_id.0
                    }
                };
                let id_2 = match id[1].as_u64() {
                    Some(id_2) => id_2,
                    None => {
                        if temp_id == (0, 0) {
                            temp_id = get_id();
                        }
                        temp_id.1
                    }
                };
                (id_1, id_2)
            }
            None => {
                let temp_id = get_id();
                (temp_id.0, temp_id.1)
            }
        };
        let name = match value["name"].as_str() {
            Some(name) => name,
            None => return Err("card name is invalid for card".to_string()),
        };
        let description = match value["description"].as_str() {
            Some(description) => description,
            None => return Err("card description is invalid for card".to_string()),
        };
        let date_created = match value["date_created"].as_str() {
            Some(date_created) => date_created,
            None => return Err("card date_created is invalid for card".to_string()),
        };
        let date_modified = match value["date_modified"].as_str() {
            Some(date_modified) => date_modified,
            None => return Err("card date_modified is invalid for card".to_string()),
        };
        let due_date = match value["due_date"].as_str() {
            Some(due_date) => due_date,
            None => return Err("card due_date is invalid for card".to_string()),
        };
        let date_completed = match value["date_completed"].as_str() {
            Some(date_completed) => date_completed,
            None => return Err("card date_completed is invalid for card".to_string()),
        };
        let priority = match value["priority"].as_str() {
            Some(priority) => match priority {
                "Low" => CardPriority::Low,
                "Medium" => CardPriority::Medium,
                "High" => CardPriority::High,
                _ => return Err("card priority is invalid for card".to_string()),
            },
            None => return Err("card priority is invalid for card".to_string()),
        };
        let card_status = match value["card_status"].as_str() {
            Some(card_status) => match card_status {
                "Active" => CardStatus::Active,
                "Complete" => CardStatus::Complete,
                "Stale" => CardStatus::Stale,
                _ => return Err("card card_status is invalid for card".to_string()),
            },
            None => return Err("card card_status is invalid for card".to_string()),
        };
        let tags = match value["tags"].as_array() {
            Some(tags) => tags
                .iter()
                .map(|t| t.as_str().unwrap().to_string())
                .collect(),
            None => return Err("card tags is invalid for card".to_string()),
        };
        let comments = match value["comments"].as_array() {
            Some(comments) => comments
                .iter()
                .map(|c| c.as_str().unwrap().to_string())
                .collect(),
            None => return Err("card comments is invalid for card".to_string()),
        };

        Ok(Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            date_created: date_created.to_string(),
            date_modified: date_modified.to_string(),
            due_date: due_date.to_string(),
            date_completed: date_completed.to_string(),
            priority,
            card_status,
            tags,
            comments,
        })
    }
}

impl Default for Card {
    fn default() -> Self {
        Self {
            card_status: CardStatus::Active,
            comments: Vec::new(),
            date_completed: FIELD_NOT_SET.to_string(),
            date_created: chrono::Local::now()
                .format(DateTimeFormat::default().to_parser_string())
                .to_string(),
            date_modified: chrono::Local::now()
                .format(DateTimeFormat::default().to_parser_string())
                .to_string(),
            description: String::from("Default Card Description"),
            due_date: FIELD_NOT_SET.to_string(),
            id: get_id(),
            name: String::from("Default Card"),
            priority: CardPriority::Low,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Cards {
    cards: Vec<Card>,
}

impl Cards {
    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
    }
    pub fn add_card_at_index(&mut self, index: usize, card: Card) {
        self.cards.insert(index, card);
    }
    pub fn get_card_with_id(&self, card_id: (u64, u64)) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == card_id)
    }
    pub fn get_mut_card_with_id(&mut self, card_id: (u64, u64)) -> Option<&mut Card> {
        self.cards.iter_mut().find(|c| c.id == card_id)
    }
    pub fn get_card_with_index(&self, index: usize) -> Option<&Card> {
        self.cards.get(index)
    }
    pub fn get_mut_card_with_index(&mut self, index: usize) -> Option<&mut Card> {
        self.cards.get_mut(index)
    }
    pub fn get_all_cards(&self) -> &Vec<Card> {
        &self.cards
    }
    pub fn get_all_card_ids(&self) -> Vec<(u64, u64)> {
        self.cards.iter().map(|c| c.id).collect()
    }
    pub fn get_cards_with_range(&self, start: usize, end: usize) -> Cards {
        self.cards[start..end].to_vec().into()
    }
    pub fn get_mut_all_cards(&mut self) -> &mut Vec<Card> {
        &mut self.cards
    }
    pub fn set_cards(&mut self, cards: Cards) {
        self.cards = cards.cards;
    }
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    pub fn get_first_card_id(&self) -> Option<(u64, u64)> {
        self.cards.first().map(|c| c.id)
    }
    pub fn get_card_index(&self, card_id: (u64, u64)) -> Option<usize> {
        self.cards.iter().position(|c| c.id == card_id)
    }
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    pub fn remove_card_with_id(&mut self, card_id: (u64, u64)) -> Option<Card> {
        let index = self.cards.iter().position(|c| c.id == card_id)?;
        Some(self.cards.remove(index))
    }
    pub fn reset(&mut self) {
        self.cards.clear();
    }
    pub fn swap(&mut self, index_1: usize, index_2: usize) {
        self.cards.swap(index_1, index_2);
    }
}

impl From<Vec<Card>> for Cards {
    fn from(cards: Vec<Card>) -> Self {
        Self { cards }
    }
}

impl FromIterator<Card> for Cards {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> Self {
        Self {
            cards: iter.into_iter().collect(),
        }
    }
}

fn get_id() -> (u64, u64) {
    Uuid::new_v4().as_u64_pair()
}
