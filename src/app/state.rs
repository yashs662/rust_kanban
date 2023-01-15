use log::error;
use serde::{
    Serialize,
    Deserialize
};
use crate::inputs::key::Key;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UiMode {
    Zen,
    TitleBody,
    BodyHelp,
    BodyLog,
    TitleBodyHelp,
    TitleBodyLog,
    TitleBodyHelpLog,
    BodyHelpLog,
    Config,
    EditConfig,
    MainMenu,
    ViewCard,
    HelpMenu,
    LogsOnly,
    NewBoard,
    NewCard,
    LoadSave
}

#[derive(Clone, PartialEq, Debug)]
pub enum AppStatus {
    Init,
    Initialized,
    UserInput
}

#[derive(Clone, PartialEq, Debug)]
pub enum Focus {
    Title,
    Body,
    Help,
    Log,
    Config,
    ConfigHelp,
    MainMenu,
    MainMenuHelp,
    NewBoardName,
    NewBoardDescription,
    NewCardName,
    NewCardDescription,
    NewCardDueDate,
    SubmitButton,
    NoFocus
}

pub struct KeyBindings {
    pub quit: Vec<Key>,
    pub open_config_menu: Vec<Key>,
    pub up: Vec<Key>,
    pub down: Vec<Key>,
    pub right: Vec<Key>,
    pub left: Vec<Key>,
    pub next_focus: Vec<Key>,
    pub prev_focus: Vec<Key>,
    pub take_user_input: Vec<Key>,
    pub hide_ui_element: Vec<Key>,
    pub save_state: Vec<Key>,
    pub new_board: Vec<Key>,
    pub new_card: Vec<Key>,
    pub delete_board: Vec<Key>,
    pub delete_card: Vec<Key>,
    pub card_status_completed: Vec<Key>,
}

impl UiMode {
    pub fn default() -> UiMode {
        UiMode::Zen
    }

    pub fn to_string(&self) -> String {
        match self {
            UiMode::Zen => "Zen".to_string(),
            UiMode::TitleBody => "Title and Body".to_string(),
            UiMode::BodyHelp => "Body and Help".to_string(),
            UiMode::BodyLog => "Body and Log".to_string(),
            UiMode::TitleBodyHelp => "Title, Body and Help".to_string(),
            UiMode::TitleBodyLog => "Title, Body and Log".to_string(),
            UiMode::BodyHelpLog => "Body, Help and Log".to_string(),
            UiMode::TitleBodyHelpLog => "Title, Body, Help and Log".to_string(),
            UiMode::Config => "Config".to_string(),
            UiMode::EditConfig => "Edit Config".to_string(),
            UiMode::MainMenu => "Main Menu".to_string(),
            UiMode::ViewCard => "View Card".to_string(),
            UiMode::HelpMenu => "Help Menu".to_string(),
            UiMode::LogsOnly => "Logs Only".to_string(),
            UiMode::NewBoard => "New Board".to_string(),
            UiMode::NewCard => "New Card".to_string(),
            UiMode::LoadSave => "Load Save".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<UiMode> {
        match s {
            "Zen" => Some(UiMode::Zen),
            "Title and Body" => Some(UiMode::TitleBody),
            "Body and Help" => Some(UiMode::BodyHelp),
            "Body and Log" => Some(UiMode::BodyLog),
            "Title, Body and Help" => Some(UiMode::TitleBodyHelp),
            "Title, Body and Log" => Some(UiMode::TitleBodyLog),
            "Body, Help and Log" => Some(UiMode::BodyHelpLog),
            "Title, Body, Help and Log" => Some(UiMode::TitleBodyHelpLog),
            "Config" => Some(UiMode::Config),
            "Edit Config" => Some(UiMode::EditConfig),
            "Main Menu" => Some(UiMode::MainMenu),
            "View Card" => Some(UiMode::ViewCard),
            "Help Menu" => Some(UiMode::HelpMenu),
            "Logs Only" => Some(UiMode::LogsOnly),
            "New Board" => Some(UiMode::NewBoard),
            "New Card" => Some(UiMode::NewCard),
            "Load Save" => Some(UiMode::LoadSave),
            _ => None,
        }
    }

    pub fn from_number(n: u8) -> UiMode {
        match n {
            1 => UiMode::Zen,
            2 => UiMode::TitleBody,
            3 => UiMode::BodyHelp,
            4 => UiMode::BodyLog,
            5 => UiMode::TitleBodyHelp,
            6 => UiMode::TitleBodyLog,
            7 => UiMode::BodyHelpLog,
            8 => UiMode::TitleBodyHelpLog,
            9 => UiMode::LogsOnly,
            _ => {
                error!("Invalid UiMode: {}", n);
                UiMode::TitleBody
            }
        }
    }

    pub fn get_available_targets(&self) -> Vec<String> {
        match self {
            UiMode::Zen => vec!["Body".to_string()],
            UiMode::TitleBody => vec!["Title".to_string(), "Body".to_string()],
            UiMode::BodyHelp => vec!["Body".to_string(), "Help".to_string()],
            UiMode::BodyLog => vec!["Body".to_string(), "Log".to_string()],
            UiMode::TitleBodyHelp => vec!["Title".to_string(), "Body".to_string(), "Help".to_string()],
            UiMode::TitleBodyLog => vec!["Title".to_string(), "Body".to_string(), "Log".to_string()],
            UiMode::BodyHelpLog => vec!["Body".to_string(), "Help".to_string(), "Log".to_string()],
            UiMode::TitleBodyHelpLog => vec!["Title".to_string(), "Body".to_string(), "Help".to_string(), "Log".to_string()],
            UiMode::Config => vec![],
            UiMode::EditConfig => vec![],
            UiMode::MainMenu => vec![],
            UiMode::ViewCard => vec![],
            UiMode::HelpMenu => vec!["Help".to_string(), "Log".to_string()],
            UiMode::LogsOnly => vec!["Log".to_string()],
            UiMode::NewBoard => vec!["New Board Name".to_string(), "New Board Description".to_string(), "Submit Button".to_string()],
            UiMode::NewCard => vec!["New Card Name".to_string(), "New Card Description".to_string(), "New Card Due Date".to_string(), "Submit Button".to_string()],
            UiMode::LoadSave => vec![],
        }
    }

    pub fn all() -> String {
        let mut s = String::new();
        for i in 1..10 {
            s.push_str(&format!("{}: {} ||| ", i, UiMode::from_number(i).to_string()));
        }
        s
    }
}

impl AppStatus {
    pub fn default() -> Self {
        Self::Init
    }

    pub fn initialized() -> Self {
        Self::Initialized
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }
}

impl Focus {
    pub fn default() -> Self {
        Self::NoFocus
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::Title => "Title",
            Self::Body => "Body",
            Self::Help => "Help",
            Self::Log => "Log",
            Self::Config => "Config",
            Self::ConfigHelp => "Config Help",
            Self::MainMenu => "Main Menu",
            Self::MainMenuHelp => "Main Menu Help",
            Self::NewBoardName => "New Board Name",
            Self::NewBoardDescription => "New Board Description",
            Self::NewCardName => "New Card Name",
            Self::NewCardDescription => "New Card Description",
            Self::NewCardDueDate => "New Card Due Date",
            Self::SubmitButton => "Submit Button",
            Self::NoFocus => "No Focus",
        }
    }
    pub fn next(&self, available_tabs: &Vec<String>) -> Self {
        let current = self.to_str();
        let index = available_tabs.iter().position(|x| x == current);
        // check if index is None
        let index = match index {
            Some(i) => i,
            None => 0,
        };
        if available_tabs.len() <= 1 {
            return Self::NoFocus;
        }
        let next_index = (index + 1) % available_tabs.len();
        match available_tabs[next_index].as_str() {
            "Title" => Self::Title,
            "Body" => Self::Body,
            "Help" => Self::Help,
            "Log" => Self::Log,
            "Config" => Self::Config,
            "Config Help" => Self::ConfigHelp,
            "Main Menu" => Self::MainMenu,
            "Main Menu Help" => Self::MainMenuHelp,
            "No Focus" => Self::NoFocus,
            "New Board Name" => Self::NewBoardName,
            "New Board Description" => Self::NewBoardDescription,
            "New Card Name" => Self::NewCardName,
            "New Card Description" => Self::NewCardDescription,
            "New Card Due Date" => Self::NewCardDueDate,
            "Submit Button" => Self::SubmitButton,
            _ => Self::NoFocus,
        }
    }

    pub fn prev(&self, available_tabs: &Vec<String>) -> Self {
        let current = self.to_str();
        let index = available_tabs.iter().position(|x| x == current);
        // check if index is None
        let index = match index {
            Some(i) => i,
            None => 0,
        };
        let prev_index = if index == 0 {
            available_tabs.len() - 1
        } else {
            index - 1
        };
        match available_tabs[prev_index].as_str() {
            "Title" => Self::Title,
            "Body" => Self::Body,
            "Help" => Self::Help,
            "Log" => Self::Log,
            "Config" => Self::Config,
            "Config Help" => Self::ConfigHelp,
            "Main Menu" => Self::MainMenu,
            "Main Menu Help" => Self::MainMenuHelp,
            "No Focus" => Self::NoFocus,
            "New Board Name" => Self::NewBoardName,
            "New Board Description" => Self::NewBoardDescription,
            "New Card Name" => Self::NewCardName,
            "New Card Description" => Self::NewCardDescription,
            "New Card Due Date" => Self::NewCardDueDate,
            "Submit Button" => Self::SubmitButton,
            _ => Self::NoFocus,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "Title" => Self::Title,
            "Body" => Self::Body,
            "Help" => Self::Help,
            "Log" => Self::Log,
            "Config" => Self::Config,
            "Config Help" => Self::ConfigHelp,
            "Main Menu" => Self::MainMenu,
            "Main Menu Help" => Self::MainMenuHelp,
            "No Focus" => Self::NoFocus,
            "New Board Name" => Self::NewBoardName,
            "New Board Description" => Self::NewBoardDescription,
            "New Card Name" => Self::NewCardName,
            "New Card Description" => Self::NewCardDescription,
            "New Card Due Date" => Self::NewCardDueDate,
            "Submit Button" => Self::SubmitButton,
            _ => Self::NoFocus
        }
    }
}

impl KeyBindings {
    pub fn default() -> Self {
        Self {
            quit: vec![Key::Char('q'), Key::Ctrl('c')],
            open_config_menu: vec![Key::Char('p')],
            up: vec![Key::Up],
            down: vec![Key::Down],
            right: vec![Key::Right],
            left: vec![Key::Left],
            next_focus: vec![Key::Tab],
            prev_focus: vec![Key::ShiftTab],
            take_user_input: vec![Key::Char('i')],
            hide_ui_element: vec![Key::Char('h')],
            save_state: vec![Key::Ctrl('s')],
            new_board: vec![Key::Char('b')],
            new_card: vec![Key::Char('n')],
            delete_board: vec![Key::ShiftD],
            delete_card: vec![Key::Char('d')],
            card_status_completed: vec![Key::Char('1')],
        }
    }
}