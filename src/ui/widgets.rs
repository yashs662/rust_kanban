use ngrammatic::{Corpus, CorpusBuilder, Pad};
use std::{sync::Arc, time::Duration};
use tokio::time::Instant;

use crate::{
    app::{App, PopupMode},
    constants::{TOAST_FADE_IN_TIME, TOAST_FADE_OUT_TIME},
    lerp_between,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ToastWidget {
    pub title: String,
    pub message: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub toast_type: ToastType,
    pub toast_color: (u8, u8, u8),
}

impl ToastWidget {
    pub fn new(message: String, duration: Duration, toast_type: ToastType) -> Self {
        Self {
            title: toast_type.as_str().to_string(),
            message,
            duration,
            start_time: Instant::now(),
            toast_type: toast_type.clone(),
            toast_color: toast_type.as_color(),
        }
    }
    pub fn new_with_title(
        title: String,
        message: String,
        duration: Duration,
        toast_type: ToastType,
    ) -> Self {
        Self {
            title,
            message,
            duration,
            start_time: Instant::now(),
            toast_type: toast_type.clone(),
            toast_color: toast_type.as_color(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Error,
    Warning,
    Info,
    Loading,
}

impl ToastType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Loading => "Loading",
        }
    }
    pub fn as_color(&self) -> (u8, u8, u8) {
        match self {
            Self::Error => (255, 0, 0),
            Self::Warning => (255, 255, 0),
            Self::Info => (0, 255, 255),
            Self::Loading => (0, 255, 0),
        }
    }
}

pub struct WidgetManager {
    pub app: Arc<tokio::sync::Mutex<App>>,
}

impl WidgetManager {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn update(&mut self) {
        let mut app = self.app.lock().await;
        let term_background_color = app.state.term_background_color;
        let toasts = &mut app.state.toasts;
        // remove all inactive toasts
        for i in (0..toasts.len()).rev() {
            // based on the toast_type lerp between the toast_type color and 0,0,0 within the TOAST_FADE_TIME which is in milliseconds
            if toasts[i].start_time.elapsed() < Duration::from_millis(TOAST_FADE_IN_TIME) {
                // make the toast fade in use fade in time lerp from 0,0,0 to toast_type color
                let t =
                    toasts[i].start_time.elapsed().as_millis() as f32 / TOAST_FADE_IN_TIME as f32;
                toasts[i].toast_color =
                    lerp_between(term_background_color, toasts[i].toast_type.as_color(), t);
            } else if toasts[i].start_time.elapsed()
                < toasts[i].duration - Duration::from_millis(TOAST_FADE_OUT_TIME)
            {
                // make the toast stay at the toast_type color
                toasts[i].toast_color = toasts[i].toast_type.as_color();
            } else {
                // make the toast fade out use fade out time lerp from toast_type color to 0,0,0
                let t = (toasts[i].start_time.elapsed()
                    - (toasts[i].duration - Duration::from_millis(TOAST_FADE_OUT_TIME)))
                .as_millis() as f32
                    / TOAST_FADE_OUT_TIME as f32;
                toasts[i].toast_color =
                    lerp_between(toasts[i].toast_type.as_color(), term_background_color, t);
            }
            if toasts[i].start_time.elapsed() > toasts[i].duration {
                toasts.remove(i);
            }
        }

        // update command palette
        if app.state.popup_mode.is_some()
            && app.state.popup_mode.unwrap() == PopupMode::CommandPalette
        {
            let mut search_result_changed = false;
            let current_search_results = app.command_palette.search_results.clone();
            let current_search_string = app.state.current_user_input.clone().to_lowercase();
            let result = app
                .command_palette
                .corpus
                .search(&current_search_string, 0.2);
            let mut search_results = vec![];
            for item in result {
                search_results.push(CommandPaletteActions::from_string(&item.text, true));
            }
            let search_results: Vec<CommandPaletteActions> =
                search_results.into_iter().filter_map(|x| x).collect();
            // if the search results are empty, then show all commands
            let search_results = if search_results.is_empty() {
                CommandPaletteActions::all()
            } else {
                search_results
            };
            app.command_palette.search_results = Some(search_results);
            if current_search_results.is_some() {
                if current_search_results.as_ref().unwrap().len()
                    != app.command_palette.search_results.clone().unwrap().len()
                {
                    search_result_changed = true;
                } else {
                    for i in 0..current_search_results.as_ref().unwrap().len() {
                        if current_search_results.as_ref().unwrap()[i]
                            != app.command_palette.search_results.clone().unwrap()[i]
                        {
                            search_result_changed = true;
                            break;
                        }
                    }
                }
            } else {
                search_result_changed = true;
            }
            if search_result_changed && app.command_palette.search_results.is_some() {
                // if lenght is > 1 select first item
                if app.command_palette.search_results.as_ref().unwrap().len() > 1 {
                    app.state.command_palette_list_state.select(Some(0));
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct CommandPalette {
    pub search_results: Option<Vec<CommandPaletteActions>>,
    pub available_commands: Vec<CommandPaletteActions>,
    pub corpus: Corpus,
}

impl CommandPalette {
    pub fn new() -> Self {
        let available_commands = CommandPaletteActions::all();
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();
        for command in available_commands {
            corpus.add_text(command.as_str().to_lowercase().as_str());
        }
        Self {
            search_results: None,
            available_commands: CommandPaletteActions::all(),
            corpus,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CommandPaletteActions {
    ExportToJSON,
    OpenConfigMenu,
    SaveKanbanState,
    LoadASave,
    NewBoard,
    NewCard,
    ResetUI,
    OpenMainMenu,
    OpenHelpMenu,
    ChangeUIMode,
    ChangeCurrentCardStatus,
    DebugMenu,
    Quit,
}

impl CommandPaletteActions {
    pub fn all() -> Vec<Self> {
        let all = vec![
            Self::ExportToJSON,
            Self::OpenConfigMenu,
            Self::SaveKanbanState,
            Self::LoadASave,
            Self::NewBoard,
            Self::NewCard,
            Self::ResetUI,
            Self::OpenMainMenu,
            Self::OpenHelpMenu,
            Self::ChangeUIMode,
            Self::ChangeCurrentCardStatus,
            Self::Quit,
        ];

        if cfg!(debug_assertions) {
            let mut all = all;
            all.push(Self::DebugMenu);
            all
        } else {
            all
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::ExportToJSON => "Export to JSON",
            Self::OpenConfigMenu => "Open Config Menu",
            Self::SaveKanbanState => "Save Kanban State",
            Self::LoadASave => "Load a Save",
            Self::NewBoard => "New Board",
            Self::NewCard => "New Card",
            Self::ResetUI => "Reset UI",
            Self::OpenMainMenu => "Open Main Menu",
            Self::OpenHelpMenu => "Open Help Menu",
            Self::ChangeUIMode => "Change UI Mode",
            Self::ChangeCurrentCardStatus => "Change Current Card Status",
            Self::DebugMenu => "Toggle Debug Panel",
            Self::Quit => "Quit",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn from_string(s: &str, lowercase_match: bool) -> Option<Self> {
        if lowercase_match {
            return match s.to_lowercase().as_str() {
                "export to json" => Some(Self::ExportToJSON),
                "open config menu" => Some(Self::OpenConfigMenu),
                "save kanban state" => Some(Self::SaveKanbanState),
                "load a save" => Some(Self::LoadASave),
                "new board" => Some(Self::NewBoard),
                "new card" => Some(Self::NewCard),
                "reset ui" => Some(Self::ResetUI),
                "open main menu" => Some(Self::OpenMainMenu),
                "open help menu" => Some(Self::OpenHelpMenu),
                "change ui mode" => Some(Self::ChangeUIMode),
                "change current card status" => Some(Self::ChangeCurrentCardStatus),
                "toggle debug panel" => Some(Self::DebugMenu),
                "quit" => Some(Self::Quit),
                _ => None,
            };
        } else {
            return match s {
                "Export to JSON" => Some(Self::ExportToJSON),
                "Open Config Menu" => Some(Self::OpenConfigMenu),
                "Save Kanban State" => Some(Self::SaveKanbanState),
                "Load a Save" => Some(Self::LoadASave),
                "New Board" => Some(Self::NewBoard),
                "New Card" => Some(Self::NewCard),
                "Reset UI" => Some(Self::ResetUI),
                "Open Main Menu" => Some(Self::OpenMainMenu),
                "Open Help Menu" => Some(Self::OpenHelpMenu),
                "Change UI Mode" => Some(Self::ChangeUIMode),
                "Change Current Card Status" => Some(Self::ChangeCurrentCardStatus),
                "Toggle Debug Panel" => Some(Self::DebugMenu),
                "Quit" => Some(Self::Quit),
                _ => None,
            };
        }
    }
}
