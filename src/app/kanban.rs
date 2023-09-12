use crate::constants::{FIELD_NA, FIELD_NOT_SET};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Board {
    pub id: (u64, u64),
    pub name: String,
    pub description: String,
    pub cards: Vec<Card>,
}

impl Board {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            id: get_id(),
            name: name.to_owned(),
            description: description.to_owned(),
            cards: Vec::new(),
        }
    }

    pub fn get_card(&self, id: (u64, u64)) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
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
        let cards = match value["cards"].as_array() {
            Some(cards) => cards
                .iter()
                .map(Card::from_json)
                .collect::<Result<_, _>>()?,
            None => return Err("board cards is invalid for board".to_string()),
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
            id: get_id(),
            name: String::from("Default Board"),
            description: String::from("Default Board Description"),
            cards: vec![Card::default()],
        }
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
    Low,
    Medium,
    High,
}

impl fmt::Display for CardPriority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CardPriority::Low => write!(f, "Low"),
            CardPriority::Medium => write!(f, "Medium"),
            CardPriority::High => write!(f, "High"),
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
    pub id: (u64, u64),
    pub name: String,
    pub description: String,
    pub date_created: String,
    pub date_modified: String,
    pub due_date: String,
    pub date_completed: String,
    pub priority: CardPriority,
    pub card_status: CardStatus,
    pub tags: Vec<String>,
    pub comments: Vec<String>,
}

impl Card {
    pub fn new(
        name: &str,
        description: &str,
        due_date: &str,
        priority: CardPriority,
        tags: Vec<String>,
        comments: Vec<String>,
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

        Self {
            id: get_id(),
            name: name.to_string(),
            description: description.to_string(),
            date_created: Utc::now().to_string(),
            date_modified: Utc::now().to_string(),
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
            id: get_id(),
            name: String::from("Default Card"),
            description: String::from("Default Card Description"),
            date_created: Utc::now().to_string(),
            date_modified: Utc::now().to_string(),
            due_date: FIELD_NOT_SET.to_string(),
            date_completed: FIELD_NOT_SET.to_string(),
            priority: CardPriority::Low,
            card_status: CardStatus::Active,
            tags: Vec::new(),
            comments: Vec::new(),
        }
    }
}

fn get_id() -> (u64, u64) {
    Uuid::new_v4().as_u64_pair()
}
