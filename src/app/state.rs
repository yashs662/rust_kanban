use super::{actions::Action, App};
use crate::{inputs::key::Key, ui::ui_helper};
use log::{debug, error};
use ratatui::Frame;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
    vec,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Default)]
pub enum UiMode {
    BodyHelp,
    BodyHelpLog,
    BodyLog,
    ConfigMenu,
    CreateTheme,
    EditKeybindings,
    HelpMenu,
    LoadCloudSave,
    LoadLocalSave,
    Login,
    LogsOnly,
    MainMenu,
    NewBoard,
    NewCard,
    ResetPassword,
    SignUp,
    TitleBody,
    TitleBodyHelp,
    TitleBodyHelpLog,
    TitleBodyLog,
    #[default]
    Zen,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum AppStatus {
    #[default]
    Init,
    Initialized,
    KeyBindMode,
    UserInput,
}

#[derive(Clone, PartialEq, Debug, Copy, Default)]
pub enum Focus {
    Body,
    CardComments,
    CardDescription,
    CardDueDate,
    CardName,
    CardPriority,
    CardStatus,
    CardTags,
    ChangeCardPriorityPopup,
    ChangeCardStatusPopup,
    ChangeDateFormatPopup,
    ChangeUiModePopup,
    CloseButton,
    CommandPaletteBoard,
    CommandPaletteCard,
    CommandPaletteCommand,
    ConfigHelp,
    ConfigTable,
    ConfirmPasswordField,
    EditGeneralConfigPopup,
    EditKeybindingsTable,
    EditSpecificKeyBindingPopup,
    EmailIDField,
    ExtraFocus, // Used in cases where defining a new focus is not necessary
    FilterByTagPopup,
    Help,
    LoadSave,
    Log,
    MainMenu,
    NewBoardDescription,
    NewBoardName,
    #[default]
    NoFocus,
    PasswordField,
    ResetPasswordLinkField,
    SelectDefaultView,
    SendResetPasswordLinkButton,
    StyleEditorBG,
    StyleEditorFG,
    StyleEditorModifier,
    SubmitButton,
    TextInput,
    ThemeEditor,
    ThemeSelector,
    Title,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyBindings {
    pub accept: Vec<Key>,
    pub change_card_status_to_active: Vec<Key>,
    pub change_card_status_to_completed: Vec<Key>,
    pub change_card_status_to_stale: Vec<Key>,
    pub clear_all_toasts: Vec<Key>,
    pub delete_board: Vec<Key>,
    pub delete_card: Vec<Key>,
    pub down: Vec<Key>,
    pub go_to_main_menu: Vec<Key>,
    pub go_to_previous_ui_mode_or_cancel: Vec<Key>,
    pub hide_ui_element: Vec<Key>,
    pub left: Vec<Key>,
    pub move_card_down: Vec<Key>,
    pub move_card_left: Vec<Key>,
    pub move_card_right: Vec<Key>,
    pub move_card_up: Vec<Key>,
    pub new_board: Vec<Key>,
    pub new_card: Vec<Key>,
    pub next_focus: Vec<Key>,
    pub open_config_menu: Vec<Key>,
    pub prv_focus: Vec<Key>,
    pub quit: Vec<Key>,
    pub redo: Vec<Key>,
    pub reset_ui: Vec<Key>,
    pub right: Vec<Key>,
    pub save_state: Vec<Key>,
    pub stop_user_input: Vec<Key>,
    pub take_user_input: Vec<Key>,
    pub toggle_command_palette: Vec<Key>,
    pub undo: Vec<Key>,
    pub up: Vec<Key>,
}

#[derive(Serialize, Deserialize, Debug, Clone, EnumIter, PartialEq)]
pub enum KeyBindingEnum {
    Accept,
    ChangeCardStatusToActive,
    ChangeCardStatusToCompleted,
    ChangeCardStatusToStale,
    ClearAllToasts,
    DeleteBoard,
    DeleteCard,
    Down,
    GoToMainMenu,
    GoToPreviousUIModeorCancel,
    HideUiElement,
    Left,
    MoveCardDown,
    MoveCardLeft,
    MoveCardRight,
    MoveCardUp,
    NewBoard,
    NewCard,
    NextFocus,
    OpenConfigMenu,
    PrvFocus,
    Quit,
    Redo,
    ResetUI,
    Right,
    SaveState,
    StopUserInput,
    TakeUserInput,
    ToggleCommandPalette,
    Undo,
    Up,
}

impl Display for KeyBindingEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Accept => "accept",
            Self::ChangeCardStatusToActive => "change_card_status_to_active",
            Self::ChangeCardStatusToCompleted => "change_card_status_to_completed",
            Self::ChangeCardStatusToStale => "change_card_status_to_stale",
            Self::ClearAllToasts => "clear_all_toasts",
            Self::DeleteBoard => "delete_board",
            Self::DeleteCard => "delete_card",
            Self::Down => "down",
            Self::GoToMainMenu => "go_to_main_menu",
            Self::GoToPreviousUIModeorCancel => "go_to_previous_ui_mode_or_cancel",
            Self::HideUiElement => "hide_ui_element",
            Self::Left => "left",
            Self::MoveCardDown => "move_card_down",
            Self::MoveCardLeft => "move_card_left",
            Self::MoveCardRight => "move_card_right",
            Self::MoveCardUp => "move_card_up",
            Self::NewBoard => "new_board",
            Self::NewCard => "new_card",
            Self::NextFocus => "next_focus",
            Self::OpenConfigMenu => "open_config_menu",
            Self::PrvFocus => "prev_focus",
            Self::Quit => "quit",
            Self::Redo => "redo",
            Self::ResetUI => "reset_ui",
            Self::Right => "right",
            Self::SaveState => "save_state",
            Self::StopUserInput => "stop_user_input",
            Self::TakeUserInput => "take_user_input",
            Self::ToggleCommandPalette => "toggle_command_palette",
            Self::Undo => "undo",
            Self::Up => "up",
        };
        write!(f, "{}", str)
    }
}

impl FromStr for KeyBindingEnum {
    type Err = KeyBindingEnum;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "change_card_status_to_active" => Ok(Self::ChangeCardStatusToActive),
            "change_card_status_to_completed" => Ok(Self::ChangeCardStatusToCompleted),
            "change_card_status_to_stale" => Ok(Self::ChangeCardStatusToStale),
            "clear_all_toasts" => Ok(Self::ClearAllToasts),
            "delete_board" => Ok(Self::DeleteBoard),
            "delete_card" => Ok(Self::DeleteCard),
            "down" => Ok(Self::Down),
            "go_to_main_menu" => Ok(Self::GoToMainMenu),
            "hide_ui_element" => Ok(Self::HideUiElement),
            "left" => Ok(Self::Left),
            "new_board" => Ok(Self::NewBoard),
            "new_card" => Ok(Self::NewCard),
            "next_focus" => Ok(Self::NextFocus),
            "open_config_menu" => Ok(Self::OpenConfigMenu),
            "prev_focus" => Ok(Self::PrvFocus),
            "quit" => Ok(Self::Quit),
            "redo" => Ok(Self::Redo),
            "reset_ui" => Ok(Self::ResetUI),
            "right" => Ok(Self::Right),
            "save_state" => Ok(Self::SaveState),
            "stop_user_input" => Ok(Self::StopUserInput),
            "take_user_input" => Ok(Self::TakeUserInput),
            "toggle_command_palette" => Ok(Self::ToggleCommandPalette),
            "undo" => Ok(Self::Undo),
            "up" => Ok(Self::Up),
            _ => Err(Self::ChangeCardStatusToActive),
        }
    }
}

impl UiMode {
    pub fn from_string(s: &str) -> Option<UiMode> {
        match s {
            "Body and Help" => Some(UiMode::BodyHelp),
            "Body, Help and Log" => Some(UiMode::BodyHelpLog),
            "Body and Log" => Some(UiMode::BodyLog),
            "Config" => Some(UiMode::ConfigMenu),
            "Create Theme" => Some(UiMode::CreateTheme),
            "Edit Keybindings" => Some(UiMode::EditKeybindings),
            "Help Menu" => Some(UiMode::HelpMenu),
            "Load a Save (Cloud)" => Some(UiMode::LoadCloudSave),
            "Load a Save (Local)" => Some(UiMode::LoadLocalSave),
            "Login" => Some(UiMode::Login),
            "Logs Only" => Some(UiMode::LogsOnly),
            "Main Menu" => Some(UiMode::MainMenu),
            "New Board" => Some(UiMode::NewBoard),
            "New Card" => Some(UiMode::NewCard),
            "Reset Password" => Some(UiMode::ResetPassword),
            "Sign Up" => Some(UiMode::SignUp),
            "Title and Body" => Some(UiMode::TitleBody),
            "Title, Body and Help" => Some(UiMode::TitleBodyHelp),
            "Title, Body, Help and Log" => Some(UiMode::TitleBodyHelpLog),
            "Title, Body and Log" => Some(UiMode::TitleBodyLog),
            "Zen" => Some(UiMode::Zen),
            _ => None,
        }
    }

    pub fn from_json_string(s: &str) -> Option<UiMode> {
        match s {
            "BodyHelp" => Some(UiMode::BodyHelp),
            "BodyHelpLog" => Some(UiMode::BodyHelpLog),
            "BodyLog" => Some(UiMode::BodyLog),
            "ConfigMenu" => Some(UiMode::ConfigMenu),
            "CreateTheme" => Some(UiMode::CreateTheme),
            "EditKeybindings" => Some(UiMode::EditKeybindings),
            "HelpMenu" => Some(UiMode::HelpMenu),
            "LoadCloudSave" => Some(UiMode::LoadCloudSave),
            "LoadLocalSave" => Some(UiMode::LoadLocalSave),
            "Login" => Some(UiMode::Login),
            "LogsOnly" => Some(UiMode::LogsOnly),
            "MainMenu" => Some(UiMode::MainMenu),
            "NewBoard" => Some(UiMode::NewBoard),
            "NewCard" => Some(UiMode::NewCard),
            "ResetPassword" => Some(UiMode::ResetPassword),
            "SignUp" => Some(UiMode::SignUp),
            "TitleBody" => Some(UiMode::TitleBody),
            "TitleBodyHelp" => Some(UiMode::TitleBodyHelp),
            "TitleBodyHelpLog" => Some(UiMode::TitleBodyHelpLog),
            "TitleBodyLog" => Some(UiMode::TitleBodyLog),
            "Zen" => Some(UiMode::Zen),
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

    pub fn get_available_targets(&self) -> Vec<Focus> {
        match self {
            UiMode::BodyHelp => vec![Focus::Body, Focus::Help],
            UiMode::BodyHelpLog => vec![Focus::Body, Focus::Help, Focus::Log],
            UiMode::BodyLog => vec![Focus::Body, Focus::Log],
            UiMode::ConfigMenu => vec![Focus::ConfigTable, Focus::SubmitButton, Focus::ExtraFocus],
            UiMode::CreateTheme => vec![Focus::ThemeEditor, Focus::SubmitButton, Focus::ExtraFocus],
            UiMode::EditKeybindings => vec![Focus::EditKeybindingsTable, Focus::SubmitButton],
            UiMode::HelpMenu => vec![Focus::Help, Focus::Log],
            UiMode::LoadCloudSave => vec![Focus::Body],
            UiMode::LoadLocalSave => vec![Focus::Body],
            UiMode::Login => vec![
                Focus::Title,
                Focus::EmailIDField,
                Focus::PasswordField,
                Focus::ExtraFocus,
                Focus::SubmitButton,
            ],
            UiMode::LogsOnly => vec![Focus::Log],
            UiMode::MainMenu => vec![Focus::MainMenu, Focus::Help, Focus::Log],
            UiMode::NewBoard => vec![
                Focus::NewBoardName,
                Focus::NewBoardDescription,
                Focus::SubmitButton,
            ],
            UiMode::NewCard => vec![
                Focus::CardName,
                Focus::CardDescription,
                Focus::CardDueDate,
                Focus::SubmitButton,
            ],
            UiMode::ResetPassword => vec![
                Focus::Title,
                Focus::EmailIDField,
                Focus::SendResetPasswordLinkButton,
                Focus::ResetPasswordLinkField,
                Focus::PasswordField,
                Focus::ConfirmPasswordField,
                Focus::ExtraFocus,
                Focus::SubmitButton,
            ],
            UiMode::SignUp => vec![
                Focus::Title,
                Focus::EmailIDField,
                Focus::PasswordField,
                Focus::ConfirmPasswordField,
                Focus::ExtraFocus,
                Focus::SubmitButton,
            ],
            UiMode::TitleBody => vec![Focus::Title, Focus::Body],
            UiMode::TitleBodyHelp => vec![Focus::Title, Focus::Body, Focus::Help],
            UiMode::TitleBodyHelpLog => vec![Focus::Title, Focus::Body, Focus::Help, Focus::Log],
            UiMode::TitleBodyLog => vec![Focus::Title, Focus::Body, Focus::Log],
            UiMode::Zen => vec![Focus::Body],
        }
    }

    pub fn view_modes_as_string() -> Vec<String> {
        UiMode::view_modes().iter().map(|x| x.to_string()).collect()
    }

    pub fn view_modes() -> Vec<UiMode> {
        vec![
            UiMode::Zen,
            UiMode::TitleBody,
            UiMode::BodyHelp,
            UiMode::BodyLog,
            UiMode::TitleBodyHelp,
            UiMode::TitleBodyLog,
            UiMode::BodyHelpLog,
            UiMode::TitleBodyHelpLog,
        ]
    }

    pub fn render(self, rect: &mut Frame, app: &mut App) {
        if app.state.popup_mode.is_none() {
            let current_focus = app.state.focus;
            if !self.get_available_targets().contains(&current_focus)
                && !self.get_available_targets().is_empty()
            {
                app.state.set_focus(self.get_available_targets()[0]);
            }
        }
        match self {
            UiMode::Zen => {
                ui_helper::render_zen_mode(rect, app);
            }
            UiMode::TitleBody => {
                ui_helper::render_title_body(rect, app);
            }
            UiMode::BodyHelp => {
                ui_helper::render_body_help(rect, app);
            }
            UiMode::BodyLog => {
                ui_helper::render_body_log(rect, app);
            }
            UiMode::TitleBodyHelp => {
                ui_helper::render_title_body_help(rect, app);
            }
            UiMode::TitleBodyLog => {
                ui_helper::render_title_body_log(rect, app);
            }
            UiMode::BodyHelpLog => {
                ui_helper::render_body_help_log(rect, app);
            }
            UiMode::TitleBodyHelpLog => {
                ui_helper::render_title_body_help_log(rect, app);
            }
            UiMode::ConfigMenu => {
                ui_helper::render_config(rect, app);
            }
            UiMode::EditKeybindings => {
                ui_helper::render_edit_keybindings(rect, app);
            }
            UiMode::MainMenu => {
                ui_helper::render_main_menu(rect, app);
            }
            UiMode::HelpMenu => {
                ui_helper::render_help_menu(rect, app);
            }
            UiMode::LogsOnly => {
                ui_helper::render_logs_only(rect, app);
            }
            UiMode::NewBoard => {
                ui_helper::render_new_board_form(rect, app);
            }
            UiMode::NewCard => ui_helper::render_new_card_form(rect, app),
            UiMode::LoadLocalSave => {
                ui_helper::render_load_a_save(rect, app);
            }
            UiMode::CreateTheme => ui_helper::render_create_theme(rect, app),
            UiMode::Login => ui_helper::render_login(rect, app),
            UiMode::SignUp => ui_helper::render_signup(rect, app),
            UiMode::ResetPassword => ui_helper::render_reset_password(rect, app),
            UiMode::LoadCloudSave => ui_helper::render_load_cloud_save(rect, app),
        }
    }
}

impl fmt::Display for UiMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UiMode::BodyHelp => write!(f, "Body and Help"),
            UiMode::BodyHelpLog => write!(f, "Body, Help and Log"),
            UiMode::BodyLog => write!(f, "Body and Log"),
            UiMode::ConfigMenu => write!(f, "Config"),
            UiMode::CreateTheme => write!(f, "Create Theme"),
            UiMode::EditKeybindings => write!(f, "Edit Keybindings"),
            UiMode::HelpMenu => write!(f, "Help Menu"),
            UiMode::LoadCloudSave => write!(f, "Load a Save (Cloud)"),
            UiMode::LoadLocalSave => write!(f, "Load a Save (Local)"),
            UiMode::Login => write!(f, "Login"),
            UiMode::LogsOnly => write!(f, "Logs Only"),
            UiMode::MainMenu => write!(f, "Main Menu"),
            UiMode::NewBoard => write!(f, "New Board"),
            UiMode::NewCard => write!(f, "New Card"),
            UiMode::ResetPassword => write!(f, "Reset Password"),
            UiMode::SignUp => write!(f, "Sign Up"),
            UiMode::TitleBody => write!(f, "Title and Body"),
            UiMode::TitleBodyHelp => write!(f, "Title, Body and Help"),
            UiMode::TitleBodyHelpLog => write!(f, "Title, Body, Help and Log"),
            UiMode::TitleBodyLog => write!(f, "Title, Body and Log"),
            UiMode::Zen => write!(f, "Zen"),
        }
    }
}

impl AppStatus {
    pub fn initialized() -> Self {
        Self::Initialized
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }
}

impl Display for Focus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Body => "Body",
            Self::CardComments => "Card Comments",
            Self::CardDescription => "Card Description",
            Self::CardDueDate => "Card Due Date",
            Self::CardName => "New Card Name",
            Self::CardPriority => "Card Priority",
            Self::CardStatus => "Card Status",
            Self::CardTags => "Card Tags",
            Self::ChangeCardPriorityPopup => "Change Card Priority Popup",
            Self::ChangeCardStatusPopup => "Change Card Status Popup",
            Self::ChangeDateFormatPopup => "Change Date Format Popup",
            Self::ChangeUiModePopup => "Change Ui Mode Popup",
            Self::CloseButton => "Close Button",
            Self::CommandPaletteBoard => "Command Palette Board",
            Self::CommandPaletteCard => "Command Palette Card",
            Self::CommandPaletteCommand => "Command Palette Command",
            Self::ConfigHelp => "Config Help",
            Self::ConfigTable => "Config",
            Self::ConfirmPasswordField => "Confirm Password Field",
            Self::EditGeneralConfigPopup => "Edit General Config Popup",
            Self::EditKeybindingsTable => "Edit Keybindings Table",
            Self::EditSpecificKeyBindingPopup => "Edit Specific Key Binding Popup",
            Self::EmailIDField => "Email ID Field",
            Self::ExtraFocus => "Extra Focus",
            Self::FilterByTagPopup => "Filter By Tag Popup",
            Self::Help => "Help",
            Self::LoadSave => "Load Save",
            Self::Log => "Log",
            Self::MainMenu => "Main Menu",
            Self::NewBoardDescription => "New Board Description",
            Self::NewBoardName => "New Board Name",
            Self::NoFocus => "No Focus",
            Self::PasswordField => "Password Field",
            Self::ResetPasswordLinkField => "Reset Password Link Field",
            Self::SelectDefaultView => "Select Default View",
            Self::SendResetPasswordLinkButton => "Send Reset Password Link Button",
            Self::StyleEditorBG => "Theme Editor BG",
            Self::StyleEditorFG => "Theme Editor FG",
            Self::StyleEditorModifier => "Theme Editor Modifier",
            Self::SubmitButton => "Submit Button",
            Self::TextInput => "Text Input",
            Self::ThemeEditor => "Theme Editor",
            Self::ThemeSelector => "Theme Selector",
            Self::Title => "Title",
        };
        write!(f, "{}", str)
    }
}

impl Focus {
    pub fn next(&self, available_tabs: &[Focus]) -> Self {
        if available_tabs.contains(self) {
            let index = available_tabs.iter().position(|x| x == self).unwrap();
            if index == available_tabs.len() - 1 {
                available_tabs[0]
            } else {
                available_tabs[index + 1]
            }
        } else {
            available_tabs[0]
        }
    }
    pub fn prev(&self, available_tabs: &[Focus]) -> Self {
        if available_tabs.contains(self) {
            let index = available_tabs.iter().position(|x| x == self).unwrap();
            if index == 0 {
                available_tabs[available_tabs.len() - 1]
            } else {
                available_tabs[index - 1]
            }
        } else {
            available_tabs[0]
        }
    }
}

impl FromStr for Focus {
    type Err = Focus;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Body" => Ok(Self::Body),
            "Card Comments" => Ok(Self::CardComments),
            "Card Description" => Ok(Self::CardDescription),
            "Card Due Date" => Ok(Self::CardDueDate),
            "Card Priority" => Ok(Self::CardPriority),
            "Card Status" => Ok(Self::CardStatus),
            "Card Tags" => Ok(Self::CardTags),
            "Change Card Priority Popup" => Ok(Self::ChangeCardPriorityPopup),
            "Change Card Status Popup" => Ok(Self::ChangeCardStatusPopup),
            "Change Date Format Popup" => Ok(Self::ChangeDateFormatPopup),
            "Change Ui Mode Popup" => Ok(Self::ChangeUiModePopup),
            "Close Button" => Ok(Self::CloseButton),
            "Command Palette Board" => Ok(Self::CommandPaletteBoard),
            "Command Palette Card" => Ok(Self::CommandPaletteCard),
            "Command Palette Command" => Ok(Self::CommandPaletteCommand),
            "Config Help" => Ok(Self::ConfigHelp),
            "Config" => Ok(Self::ConfigTable),
            "Confirm Password Field" => Ok(Self::ConfirmPasswordField),
            "Edit General Config Popup" => Ok(Self::EditGeneralConfigPopup),
            "Edit Keybindings Table" => Ok(Self::EditKeybindingsTable),
            "Edit Specific Key Binding Popup" => Ok(Self::EditSpecificKeyBindingPopup),
            "Email ID Field" => Ok(Self::EmailIDField),
            "Extra Focus" => Ok(Self::ExtraFocus),
            "Filter By Tag Popup" => Ok(Self::FilterByTagPopup),
            "Help" => Ok(Self::Help),
            "Load Save" => Ok(Self::LoadSave),
            "Log" => Ok(Self::Log),
            "Main Menu" => Ok(Self::MainMenu),
            "New Board Description" => Ok(Self::NewBoardDescription),
            "New Board Name" => Ok(Self::NewBoardName),
            "New Card Name" => Ok(Self::CardName),
            "No Focus" => Ok(Self::NoFocus),
            "Password Field" => Ok(Self::PasswordField),
            "Reset Password Link Field" => Ok(Self::ResetPasswordLinkField),
            "Select Default View" => Ok(Self::SelectDefaultView),
            "Send Reset Password Link Button" => Ok(Self::SendResetPasswordLinkButton),
            "Submit Button" => Ok(Self::SubmitButton),
            "Text Input" => Ok(Self::TextInput),
            "Theme Editor BG" => Ok(Self::StyleEditorBG),
            "Theme Editor FG" => Ok(Self::StyleEditorFG),
            "Theme Editor Modifier" => Ok(Self::StyleEditorModifier),
            "Theme Editor" => Ok(Self::ThemeEditor),
            "Theme Selector" => Ok(Self::ThemeSelector),
            "Title" => Ok(Self::Title),
            _ => Err(Self::NoFocus),
        }
    }
}

impl KeyBindings {
    pub fn iter(&self) -> impl Iterator<Item = (KeyBindingEnum, &Vec<Key>)> {
        KeyBindingEnum::iter().map(|enum_variant| {
            let value = match enum_variant {
                KeyBindingEnum::Accept => &self.accept,
                KeyBindingEnum::ChangeCardStatusToActive => &self.change_card_status_to_active,
                KeyBindingEnum::ChangeCardStatusToCompleted => {
                    &self.change_card_status_to_completed
                }
                KeyBindingEnum::ChangeCardStatusToStale => &self.change_card_status_to_stale,
                KeyBindingEnum::ClearAllToasts => &self.clear_all_toasts,
                KeyBindingEnum::DeleteBoard => &self.delete_board,
                KeyBindingEnum::DeleteCard => &self.delete_card,
                KeyBindingEnum::Down => &self.down,
                KeyBindingEnum::GoToMainMenu => &self.go_to_main_menu,
                KeyBindingEnum::GoToPreviousUIModeorCancel => {
                    &self.go_to_previous_ui_mode_or_cancel
                }
                KeyBindingEnum::HideUiElement => &self.hide_ui_element,
                KeyBindingEnum::Left => &self.left,
                KeyBindingEnum::MoveCardDown => &self.move_card_down,
                KeyBindingEnum::MoveCardLeft => &self.move_card_left,
                KeyBindingEnum::MoveCardRight => &self.move_card_right,
                KeyBindingEnum::MoveCardUp => &self.move_card_up,
                KeyBindingEnum::NewBoard => &self.new_board,
                KeyBindingEnum::NewCard => &self.new_card,
                KeyBindingEnum::NextFocus => &self.next_focus,
                KeyBindingEnum::OpenConfigMenu => &self.open_config_menu,
                KeyBindingEnum::PrvFocus => &self.prv_focus,
                KeyBindingEnum::Quit => &self.quit,
                KeyBindingEnum::Redo => &self.redo,
                KeyBindingEnum::ResetUI => &self.reset_ui,
                KeyBindingEnum::Right => &self.right,
                KeyBindingEnum::SaveState => &self.save_state,
                KeyBindingEnum::StopUserInput => &self.stop_user_input,
                KeyBindingEnum::TakeUserInput => &self.take_user_input,
                KeyBindingEnum::ToggleCommandPalette => &self.toggle_command_palette,
                KeyBindingEnum::Undo => &self.undo,
                KeyBindingEnum::Up => &self.up,
            };
            (enum_variant, value)
        })
    }

    pub fn key_to_action(&self, key: &Key) -> Option<Action> {
        let keybinding_enum = self
            .iter()
            .find(|(_, keybinding)| keybinding.contains(key))
            .map(|(keybinding_enum, _)| keybinding_enum);
        keybinding_enum.map(|keybinding_enum| self.keybinding_enum_to_action(keybinding_enum))
    }

    pub fn keybinding_enum_to_action(&self, keybinding_enum: KeyBindingEnum) -> Action {
        match keybinding_enum {
            KeyBindingEnum::Accept => Action::Accept,
            KeyBindingEnum::ChangeCardStatusToActive => Action::ChangeCardStatusToActive,
            KeyBindingEnum::ChangeCardStatusToCompleted => Action::ChangeCardStatusToCompleted,
            KeyBindingEnum::ChangeCardStatusToStale => Action::ChangeCardStatusToStale,
            KeyBindingEnum::ClearAllToasts => Action::ClearAllToasts,
            KeyBindingEnum::DeleteBoard => Action::DeleteBoard,
            KeyBindingEnum::DeleteCard => Action::Delete,
            KeyBindingEnum::Down => Action::Down,
            KeyBindingEnum::GoToMainMenu => Action::GoToMainMenu,
            KeyBindingEnum::GoToPreviousUIModeorCancel => Action::GoToPreviousUIModeorCancel,
            KeyBindingEnum::HideUiElement => Action::HideUiElement,
            KeyBindingEnum::Left => Action::Left,
            KeyBindingEnum::MoveCardDown => Action::MoveCardDown,
            KeyBindingEnum::MoveCardLeft => Action::MoveCardLeft,
            KeyBindingEnum::MoveCardRight => Action::MoveCardRight,
            KeyBindingEnum::MoveCardUp => Action::MoveCardUp,
            KeyBindingEnum::NewBoard => Action::NewBoard,
            KeyBindingEnum::NewCard => Action::NewCard,
            KeyBindingEnum::NextFocus => Action::NextFocus,
            KeyBindingEnum::OpenConfigMenu => Action::OpenConfigMenu,
            KeyBindingEnum::PrvFocus => Action::PrvFocus,
            KeyBindingEnum::Quit => Action::Quit,
            KeyBindingEnum::Redo => Action::Redo,
            KeyBindingEnum::ResetUI => Action::ResetUI,
            KeyBindingEnum::Right => Action::Right,
            KeyBindingEnum::SaveState => Action::SaveState,
            KeyBindingEnum::StopUserInput => Action::StopUserInput,
            KeyBindingEnum::TakeUserInput => Action::TakeUserInput,
            KeyBindingEnum::ToggleCommandPalette => Action::ToggleCommandPalette,
            KeyBindingEnum::Undo => Action::Undo,
            KeyBindingEnum::Up => Action::Up,
        }
    }

    pub fn edit_keybinding(&mut self, key: &str, keybinding: Vec<Key>) -> &mut Self {
        let mut keybinding = keybinding;
        keybinding.dedup();
        let keybinding_enum = KeyBindingEnum::from_str(key);
        if let Ok(keybinding_enum) = keybinding_enum {
            match keybinding_enum {
                KeyBindingEnum::Accept => self.accept = keybinding,
                KeyBindingEnum::ChangeCardStatusToActive => {
                    self.change_card_status_to_active = keybinding
                }
                KeyBindingEnum::ChangeCardStatusToCompleted => {
                    self.change_card_status_to_completed = keybinding
                }
                KeyBindingEnum::ChangeCardStatusToStale => {
                    self.change_card_status_to_stale = keybinding
                }
                KeyBindingEnum::ClearAllToasts => self.clear_all_toasts = keybinding,
                KeyBindingEnum::DeleteBoard => self.delete_board = keybinding,
                KeyBindingEnum::DeleteCard => self.delete_card = keybinding,
                KeyBindingEnum::Down => self.down = keybinding,
                KeyBindingEnum::GoToMainMenu => self.go_to_main_menu = keybinding,
                KeyBindingEnum::GoToPreviousUIModeorCancel => {
                    self.go_to_previous_ui_mode_or_cancel = keybinding
                }
                KeyBindingEnum::HideUiElement => self.hide_ui_element = keybinding,
                KeyBindingEnum::Left => self.left = keybinding,
                KeyBindingEnum::MoveCardDown => self.move_card_down = keybinding,
                KeyBindingEnum::MoveCardLeft => self.move_card_left = keybinding,
                KeyBindingEnum::MoveCardRight => self.move_card_right = keybinding,
                KeyBindingEnum::MoveCardUp => self.move_card_up = keybinding,
                KeyBindingEnum::NewBoard => self.new_board = keybinding,
                KeyBindingEnum::NewCard => self.new_card = keybinding,
                KeyBindingEnum::NextFocus => self.next_focus = keybinding,
                KeyBindingEnum::OpenConfigMenu => self.open_config_menu = keybinding,
                KeyBindingEnum::PrvFocus => self.prv_focus = keybinding,
                KeyBindingEnum::Quit => self.quit = keybinding,
                KeyBindingEnum::Redo => self.redo = keybinding,
                KeyBindingEnum::ResetUI => self.reset_ui = keybinding,
                KeyBindingEnum::Right => self.right = keybinding,
                KeyBindingEnum::SaveState => self.save_state = keybinding,
                KeyBindingEnum::StopUserInput => self.stop_user_input = keybinding,
                KeyBindingEnum::TakeUserInput => self.take_user_input = keybinding,
                KeyBindingEnum::ToggleCommandPalette => self.toggle_command_palette = keybinding,
                KeyBindingEnum::Undo => self.undo = keybinding,
                KeyBindingEnum::Up => self.up = keybinding,
            }
        } else {
            debug!("Invalid keybinding: {}", key);
        }
        self
    }

    pub fn get_keybindings(&self, keybinding_enum: KeyBindingEnum) -> Option<Vec<Key>> {
        match keybinding_enum {
            KeyBindingEnum::Accept => Some(self.accept.clone()),
            KeyBindingEnum::ChangeCardStatusToActive => {
                Some(self.change_card_status_to_active.clone())
            }
            KeyBindingEnum::ChangeCardStatusToCompleted => {
                Some(self.change_card_status_to_completed.clone())
            }
            KeyBindingEnum::ChangeCardStatusToStale => {
                Some(self.change_card_status_to_stale.clone())
            }
            KeyBindingEnum::ClearAllToasts => Some(self.clear_all_toasts.clone()),
            KeyBindingEnum::DeleteBoard => Some(self.delete_board.clone()),
            KeyBindingEnum::DeleteCard => Some(self.delete_card.clone()),
            KeyBindingEnum::Down => Some(self.down.clone()),
            KeyBindingEnum::GoToMainMenu => Some(self.go_to_main_menu.clone()),
            KeyBindingEnum::GoToPreviousUIModeorCancel => {
                Some(self.go_to_previous_ui_mode_or_cancel.clone())
            }
            KeyBindingEnum::HideUiElement => Some(self.hide_ui_element.clone()),
            KeyBindingEnum::Left => Some(self.left.clone()),
            KeyBindingEnum::MoveCardDown => Some(self.move_card_down.clone()),
            KeyBindingEnum::MoveCardLeft => Some(self.move_card_left.clone()),
            KeyBindingEnum::MoveCardRight => Some(self.move_card_right.clone()),
            KeyBindingEnum::MoveCardUp => Some(self.move_card_up.clone()),
            KeyBindingEnum::NewBoard => Some(self.new_board.clone()),
            KeyBindingEnum::NewCard => Some(self.new_card.clone()),
            KeyBindingEnum::NextFocus => Some(self.next_focus.clone()),
            KeyBindingEnum::OpenConfigMenu => Some(self.open_config_menu.clone()),
            KeyBindingEnum::PrvFocus => Some(self.prv_focus.clone()),
            KeyBindingEnum::Quit => Some(self.quit.clone()),
            KeyBindingEnum::Redo => Some(self.redo.clone()),
            KeyBindingEnum::ResetUI => Some(self.reset_ui.clone()),
            KeyBindingEnum::Right => Some(self.right.clone()),
            KeyBindingEnum::SaveState => Some(self.save_state.clone()),
            KeyBindingEnum::StopUserInput => Some(self.stop_user_input.clone()),
            KeyBindingEnum::TakeUserInput => Some(self.take_user_input.clone()),
            KeyBindingEnum::ToggleCommandPalette => Some(self.toggle_command_palette.clone()),
            KeyBindingEnum::Undo => Some(self.undo.clone()),
            KeyBindingEnum::Up => Some(self.up.clone()),
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            accept: vec![Key::Enter],
            change_card_status_to_active: vec![Key::Char('2')],
            change_card_status_to_completed: vec![Key::Char('1')],
            change_card_status_to_stale: vec![Key::Char('3')],
            clear_all_toasts: vec![Key::Char('t')],
            delete_board: vec![Key::Char('D')],
            delete_card: vec![Key::Char('d'), Key::Delete],
            down: vec![Key::Down],
            go_to_main_menu: vec![Key::Char('m')],
            go_to_previous_ui_mode_or_cancel: vec![Key::Esc],
            hide_ui_element: vec![Key::Char('h')],
            left: vec![Key::Left],
            move_card_down: vec![Key::ShiftDown],
            move_card_left: vec![Key::ShiftLeft],
            move_card_right: vec![Key::ShiftRight],
            move_card_up: vec![Key::ShiftUp],
            new_board: vec![Key::Char('b')],
            new_card: vec![Key::Char('n')],
            next_focus: vec![Key::Tab],
            open_config_menu: vec![Key::Char('c')],
            prv_focus: vec![Key::BackTab],
            quit: vec![Key::Ctrl('c'), Key::Char('q')],
            redo: vec![Key::Ctrl('y')],
            reset_ui: vec![Key::Char('r')],
            right: vec![Key::Right],
            save_state: vec![Key::Ctrl('s')],
            stop_user_input: vec![Key::Ins],
            take_user_input: vec![Key::Char('i')],
            toggle_command_palette: vec![Key::Ctrl('p')],
            undo: vec![Key::Ctrl('z')],
            up: vec![Key::Up],
        }
    }
}
