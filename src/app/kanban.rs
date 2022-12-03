use savefile_derive::Savefile;
use serde::{
    Serialize,
    Deserialize
};
use chrono::{
    DateTime,
    Utc
};
use uuid::Uuid;
use crate::constants::FIELD_NOT_SET;

#[derive(Serialize, Deserialize, Debug, Savefile, Clone)]
pub struct Board {
    pub id: u128,
    pub name: String,
    pub description: String,
    pub cards: Vec<Card>,
    pub action_history: Vec<String>,
}

impl Board {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: get_id(),
            name,
            description,
            cards: Vec::new(),
            action_history: Vec::new(),
        }
    }

    pub fn add_card(&mut self, card: Card) {
        let card_name = card.name.clone();
        self.cards.push(card);
        self.action_history.push(format!("Added card {}", card_name));
    }

    pub fn remove_card(&mut self, card: Card) {
        self.cards.retain(|c| c.id != card.id);
        self.action_history.push(format!("Removed card {}", card.name));
    }

    pub fn get_card(&self, id: u128) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
    }

    pub fn rename_board(&mut self, name: String) {
        let old_name = self.name.clone();
        let new_name = name.clone();
        self.name = name;
        self.action_history.push(format!("Renamed board {} to {}", old_name, new_name));
    }

    pub fn change_description(&mut self, description: String) {
        let old_description = self.description.clone();
        let new_description = description.clone();
        self.description = description;
        self.action_history.push(format!("Changed description from {} to {}", old_description, new_description));
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            id: get_id(),
            name: String::from("Default Board"),
            description: String::from("Default Board Description"),
            cards: vec![Card::default()],
            action_history: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Savefile)]
pub enum CardStatus {
    Active,
    Complete,
}

impl CardStatus {
    pub fn to_string(&self) -> String {
        match self {
            CardStatus::Active => "Active".to_string(),
            CardStatus::Complete => "Complete".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Savefile)]
pub enum CardPriority {
    Low,
    Medium,
    High,
}

impl CardPriority {
    fn to_string(&self) -> String {
        match self {
            CardPriority::Low => "Low".to_string(),
            CardPriority::Medium => "Medium".to_string(),
            CardPriority::High => "High".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Savefile, Clone)]
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
    pub action_history: Vec<String>,
}

impl Card {
    pub fn new(name: String, description: String, date_due: String, priority: CardPriority, tags: Vec<String>, comments: Vec<String>) -> Self {
        let name = if name.is_empty() { FIELD_NOT_SET } else { &name };
        let description = if description.is_empty() { FIELD_NOT_SET } else { &description };
        let date_due = if date_due.is_empty() { FIELD_NOT_SET } else { &date_due };
        let priority = if priority.to_string().is_empty() { CardPriority::Low } else { priority };
        let tags = if tags.is_empty() { Vec::new() } else { tags };
        let comments = if comments.is_empty() { Vec::new() } else { comments };
        
        Self {
            id: get_id(),
            name: name.to_string(),
            description: description.to_string(),
            date_created: Utc::now().to_string(),
            date_modified: Utc::now().to_string(),
            date_due: date_due.to_string(),
            date_completed: FIELD_NOT_SET.to_string(),
            priority,
            card_status: CardStatus::Active,
            tags,
            comments,
            action_history: Vec::new(),
        }
    }

    pub fn set_date_due(&mut self, date_due: DateTime<Utc>) {
        self.date_due = date_due.to_string();
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Date Due set to {}", date_due));
    }

    pub fn set_date_completed(&mut self, date_completed: DateTime<Utc>) {
        self.date_completed = date_completed.to_string();
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Date Completed set to {}", date_completed));
    }

    pub fn set_priority(&mut self, priority: CardPriority) {
        self.priority = priority;
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Priority set to {}", self.priority.to_string()));
    }

    pub fn set_card_status(&mut self, card_status: CardStatus) {
        let old_status = self.card_status.to_string();
        let new_status = card_status.to_string();
        self.card_status = card_status;
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Card Status changed from {} to {}", old_status, new_status));
    }

    pub fn add_tag(&mut self, tag: String) {
        let new_tag = tag.clone();
        self.tags.push(tag);
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Tag {} added", new_tag));
    }

    pub fn remove_tag(&mut self, tag: String) {
        let index = self.tags.iter().position(|t| t == &tag).unwrap_or(0);
        self.tags.remove(index);
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Tag {} removed", tag));
    }

    pub fn add_comment(&mut self, comment: String) {
        let new_comment = comment.clone();
        self.comments.push(comment);
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Comment {} added", new_comment));
    }

    pub fn remove_comment(&mut self, comment: String) {
        let index = self.comments.iter().position(|c| c == &comment).unwrap_or(0);
        self.comments.remove(index);
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Comment {} removed", comment));
    }

    pub fn clear_action_history(&mut self) {
        self.action_history.clear();
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
            action_history: Vec::new(),
        }
    }
}

fn get_id() -> u128 {
    Uuid::new_v4().as_u128()
}
