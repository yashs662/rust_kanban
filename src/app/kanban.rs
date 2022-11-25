
use chrono::{DateTime, Utc};
use log::debug;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone)]
pub enum CardStatus {
    Active,
    Complete,
}

impl CardStatus {
    fn to_string(&self) -> String {
        match self {
            CardStatus::Active => "Active".to_string(),
            CardStatus::Complete => "Complete".to_string(),
        }
    }

    fn from_string(s: &str) -> Option<Self> {
        match s {
            "Active" => Some(CardStatus::Active),
            "Complete" => Some(CardStatus::Complete),
            _ => {
                debug!("Invalid card status: {}", s);
                None
            }
        }
    }
}

static FIELD_NOT_SET: &str = "Not Set";

pub struct Card {
    id: u32,
    name: String,
    short_description: String,
    long_description: String,
    date_created: String,
    date_modified: String,
    date_due: String,
    date_completed: String,
    priority: u8,
    card_status: CardStatus,
    tags: Vec<String>,
    comments: Vec<String>,
    action_history: Vec<String>,
}

pub struct Board {
    pub name: String,
    pub description: String,
    pub cards: Vec<Card>,
    pub action_history: Vec<String>,
}

impl Board {
    pub fn new(name: String, description: String) -> Self {
        Self {
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

    pub fn get_card(&self, id: u32) -> Option<&Card> {
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
            name: String::from("Default Board"),
            description: String::from("Default Board Description"),
            cards: Vec::new(),
            action_history: Vec::new(),
        }
    }
}

impl Card {
    pub fn new(name: String, short_description: String, long_description: String, date_due: String, priority: u8, tags: Vec<String>, comments: Vec<String>) -> Self {
        let name = if name.is_empty() { FIELD_NOT_SET } else { &name };
        let short_description = if short_description.is_empty() { FIELD_NOT_SET } else { &short_description };
        let long_description = if long_description.is_empty() { FIELD_NOT_SET } else { &long_description };
        let date_due = if date_due.is_empty() { FIELD_NOT_SET } else { &date_due };
        let priority = if priority == 0 { 1 } else { priority };
        let tags = if tags.is_empty() { Vec::new() } else { tags };
        let comments = if comments.is_empty() { Vec::new() } else { comments };
        
        Self {
            id: get_id(),
            name: name.to_string(),
            short_description: short_description.to_string(),
            long_description: long_description.to_string(),
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

    pub fn set_priority(&mut self, priority: u8) {
        self.priority = priority;
        self.date_modified = Utc::now().to_string();
        self.action_history.push(format!("Priority set to {}", priority));
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
        let index = self.tags.iter().position(|t| t == &tag).unwrap();
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
        let index = self.comments.iter().position(|c| c == &comment).unwrap();
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
            short_description: String::from("Default Card Short Description"),
            long_description: String::from("Default Card Long Description"),
            date_created: Utc::now().to_string(),
            date_modified: Utc::now().to_string(),
            date_due: FIELD_NOT_SET.to_string(),
            date_completed: FIELD_NOT_SET.to_string(),
            priority: 1,
            card_status: CardStatus::Active,
            tags: Vec::new(),
            comments: Vec::new(),
            action_history: Vec::new(),
        }
    }
}

fn get_id() -> u32 {
    static ID: AtomicUsize = AtomicUsize::new(0);
    ID.fetch_add(1, Ordering::SeqCst) as u32
}
