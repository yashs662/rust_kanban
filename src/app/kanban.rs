use chrono::Utc;
use savefile_derive::Savefile;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::constants::FIELD_NOT_SET;

#[derive(Serialize, Deserialize, Debug, Savefile, Clone, Eq, PartialEq)]
pub struct Board {
    pub id: u128,
    pub name: String,
    pub description: String,
    pub cards: Vec<Card>,
}

impl Board {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: get_id(),
            name,
            description,
            cards: Vec::new(),
        }
    }

    pub fn get_card(&self, id: u128) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
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

#[derive(Debug, Clone, Serialize, Deserialize, Savefile, PartialEq, Eq)]
pub enum CardStatus {
    Active,
    Complete,
    Stale,
}

impl CardStatus {
    pub fn to_string(&self) -> String {
        match self {
            CardStatus::Active => "Active".to_string(),
            CardStatus::Complete => "Complete".to_string(),
            CardStatus::Stale => "Stale".to_string(),
        }
    }
    pub fn all() -> Vec<CardStatus> {
        vec![CardStatus::Active, CardStatus::Complete, CardStatus::Stale]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Savefile, PartialEq, Eq)]
pub enum CardPriority {
    Low,
    Medium,
    High,
}

impl CardPriority {
    pub fn to_string(&self) -> String {
        match self {
            CardPriority::Low => "Low".to_string(),
            CardPriority::Medium => "Medium".to_string(),
            CardPriority::High => "High".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Savefile, Clone, PartialEq, Eq)]
pub struct Card {
    pub id: u128,
    pub name: String,
    pub description: String,
    pub date_created: String,
    pub date_modified: String,
    pub date_due: String,
    pub date_completed: String,
    pub priority: CardPriority,
    pub card_status: CardStatus,
    pub tags: Vec<String>,
    pub comments: Vec<String>,
}

impl Card {
    pub fn new(
        name: String,
        description: String,
        date_due: String,
        priority: CardPriority,
        tags: Vec<String>,
        comments: Vec<String>,
    ) -> Self {
        let name = if name.is_empty() {
            FIELD_NOT_SET
        } else {
            &name
        };
        let description = if description.is_empty() {
            FIELD_NOT_SET
        } else {
            &description
        };
        let date_due = if date_due.is_empty() {
            FIELD_NOT_SET
        } else {
            &date_due
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
            date_due: date_due.to_string(),
            date_completed: "N/A".to_string(),
            priority,
            card_status: CardStatus::Active,
            tags,
            comments,
        }
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
            date_due: FIELD_NOT_SET.to_string(),
            date_completed: FIELD_NOT_SET.to_string(),
            priority: CardPriority::Low,
            card_status: CardStatus::Active,
            tags: Vec::new(),
            comments: Vec::new(),
        }
    }
}

fn get_id() -> u128 {
    Uuid::new_v4().as_u128()
}
