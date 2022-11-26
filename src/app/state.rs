use log::debug;
use serde::{Serialize, Deserialize};

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
    HelpMenu
}

impl UiMode {
    pub fn to_string(&self) -> String {
        match self {
            UiMode::Zen => "Zen".to_string(),
            UiMode::TitleBody => "Title".to_string(),
            UiMode::BodyHelp => "Help".to_string(),
            UiMode::BodyLog => "Log".to_string(),
            UiMode::TitleBodyHelp => "Title and Help".to_string(),
            UiMode::TitleBodyLog => "Title and Log".to_string(),
            UiMode::BodyHelpLog => "Help and Log".to_string(),
            UiMode::TitleBodyHelpLog => "Title, Help and Log".to_string(),
            UiMode::Config => "Config".to_string(),
            UiMode::EditConfig => "Edit Config".to_string(),
            UiMode::MainMenu => "Main Menu".to_string(),
            UiMode::ViewCard => "View Card".to_string(),
            UiMode::HelpMenu => "Help Menu".to_string(),
        }
    }

    pub fn from_string(s: &str) -> Option<UiMode> {
        match s {
            "Zen" => Some(UiMode::Zen),
            "Title" => Some(UiMode::TitleBody),
            "Help" => Some(UiMode::BodyHelp),
            "Log" => Some(UiMode::BodyLog),
            "Title and Help" => Some(UiMode::TitleBodyHelp),
            "Title and Log" => Some(UiMode::TitleBodyLog),
            "Help and Log" => Some(UiMode::BodyHelpLog),
            "Title, Help and Log" => Some(UiMode::TitleBodyHelpLog),
            "Config" => Some(UiMode::Config),
            "Edit Config" => Some(UiMode::EditConfig),
            "Main Menu" => Some(UiMode::MainMenu),
            "View Card" => Some(UiMode::ViewCard),
            "Help Menu" => Some(UiMode::HelpMenu),
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
            _ => {
                debug!("Invalid UiMode: {}", n);
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
            UiMode::Config => vec!["Config".to_string(), "Config Help".to_string(), "Log".to_string()],
            UiMode::EditConfig => vec!["Edit Config".to_string()],
            UiMode::MainMenu => vec![],
            UiMode::ViewCard => vec!["View Card".to_string()],
            UiMode::HelpMenu => vec![],
        }
    }

    pub fn all() -> String {
        let mut s = String::new();
        for i in 1..9 {
            s.push_str(&format!("{}: {} ||| ", i, UiMode::from_number(i).to_string()));
        }
        s
    }
}

#[derive(Clone, PartialEq)]
pub enum AppState {
    Init,
    Initialized,
    UserInput
}

#[derive(Clone, PartialEq)]
pub enum Focus {
    Title,
    Body,
    Help,
    Log,
    Config,
    ConfigHelp,
    MainMenu,
    MainMenuHelp,
    NoFocus
}

impl AppState {
    pub fn initialized() -> Self {
        Self::Initialized
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}

impl Focus {
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
            _ => Self::Title,
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
            _ => Self::Title,
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
            _ => Self::NoFocus
        }
    }
}

impl Default for Focus {
    fn default() -> Self {
        Self::NoFocus
    }
}
