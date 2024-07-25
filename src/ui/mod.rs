use crate::{
    app::{state::Focus, App},
    ui::rendering::view::{BodyHelp, TitleBody, Zen},
};
use ratatui::{
    style::{Color, Modifier},
    Frame,
};
use rendering::{
    popup::{
        CardPrioritySelector, CardStatusSelector, ChangeDateFormat, ChangeTheme, ChangeView,
        CommandPalette, ConfirmDiscardCardChanges, CustomHexColorPrompt, DateTimePicker,
        EditGeneralConfig, EditSpecificKeybinding, EditThemeStyle, FilterByTag, SaveThemePrompt,
        SelectDefaultView, ViewCard,
    },
    view::{
        BodyHelpLog, BodyLog, ConfigMenu, CreateTheme, EditKeybindings, HelpMenu, LoadASave,
        LoadCloudSave, LogView, Login, MainMenuView, NewBoardForm, NewCardForm, ResetPassword,
        Signup, TitleBodyHelp, TitleBodyHelpLog, TitleBodyLog,
    },
};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};
use strum::{Display, EnumIter, EnumString};

pub mod inbuilt_themes;
pub mod rendering;
pub mod text_box;
pub mod theme;
pub mod ui_helper;
pub mod ui_main;
pub mod widgets;

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, Copy)]
pub enum TextColorOptions {
    Black,
    Blue,
    Cyan,
    DarkGray,
    Gray,
    Green,
    LightBlue,
    LightCyan,
    LightGreen,
    LightMagenta,
    LightRed,
    LightYellow,
    Magenta,
    None,
    #[strum(to_string = "HEX #{0:02x}{1:02x}{2:02x}")]
    HEX(u8, u8, u8),
    Red,
    White,
    Yellow,
}

impl From<Color> for TextColorOptions {
    fn from(color: Color) -> Self {
        match color {
            Color::Black => TextColorOptions::Black,
            Color::Blue => TextColorOptions::Blue,
            Color::Cyan => TextColorOptions::Cyan,
            Color::DarkGray => TextColorOptions::DarkGray,
            Color::Gray => TextColorOptions::Gray,
            Color::Green => TextColorOptions::Green,
            Color::LightBlue => TextColorOptions::LightBlue,
            Color::LightCyan => TextColorOptions::LightCyan,
            Color::LightGreen => TextColorOptions::LightGreen,
            Color::LightMagenta => TextColorOptions::LightMagenta,
            Color::LightRed => TextColorOptions::LightRed,
            Color::LightYellow => TextColorOptions::LightYellow,
            Color::Magenta => TextColorOptions::Magenta,
            Color::Red => TextColorOptions::Red,
            Color::Reset => TextColorOptions::None,
            Color::Rgb(r, g, b) => TextColorOptions::HEX(r, g, b),
            Color::White => TextColorOptions::White,
            Color::Yellow => TextColorOptions::Yellow,
            _ => TextColorOptions::None,
        }
    }
}

impl From<TextColorOptions> for Color {
    fn from(color: TextColorOptions) -> Self {
        match color {
            TextColorOptions::Black => Color::Black,
            TextColorOptions::Blue => Color::Blue,
            TextColorOptions::Cyan => Color::Cyan,
            TextColorOptions::DarkGray => Color::DarkGray,
            TextColorOptions::Gray => Color::Gray,
            TextColorOptions::Green => Color::Green,
            TextColorOptions::LightBlue => Color::LightBlue,
            TextColorOptions::LightCyan => Color::LightCyan,
            TextColorOptions::LightGreen => Color::LightGreen,
            TextColorOptions::LightMagenta => Color::LightMagenta,
            TextColorOptions::LightRed => Color::LightRed,
            TextColorOptions::LightYellow => Color::LightYellow,
            TextColorOptions::Magenta => Color::Magenta,
            TextColorOptions::None => Color::Reset,
            TextColorOptions::Red => Color::Red,
            TextColorOptions::HEX(r, g, b) => Color::Rgb(r, g, b),
            TextColorOptions::White => Color::White,
            TextColorOptions::Yellow => Color::Yellow,
        }
    }
}

impl TextColorOptions {
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        match self {
            TextColorOptions::Black => (0, 0, 0),
            TextColorOptions::Blue => (0, 0, 128),
            TextColorOptions::Cyan => (0, 128, 128),
            TextColorOptions::DarkGray => (128, 128, 128),
            TextColorOptions::Gray => (192, 192, 192),
            TextColorOptions::Green => (0, 128, 0),
            TextColorOptions::LightBlue => (0, 0, 255),
            TextColorOptions::LightCyan => (0, 255, 255),
            TextColorOptions::LightGreen => (255, 255, 0),
            TextColorOptions::LightMagenta => (255, 0, 255),
            TextColorOptions::LightRed => (255, 0, 0),
            TextColorOptions::LightYellow => (0, 255, 0),
            TextColorOptions::Magenta => (128, 0, 128),
            TextColorOptions::None => (0, 0, 0),
            TextColorOptions::Red => (128, 0, 0),
            TextColorOptions::HEX(r, g, b) => (*r, *g, *b),
            TextColorOptions::White => (255, 255, 255),
            TextColorOptions::Yellow => (128, 128, 0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Display, EnumIter)]
pub enum TextModifierOptions {
    Bold,
    CrossedOut,
    Dim,
    Hidden,
    Italic,
    None,
    RapidBlink,
    Reversed,
    SlowBlink,
    Underlined,
}

impl From<TextModifierOptions> for Modifier {
    fn from(modifier: TextModifierOptions) -> Self {
        match modifier {
            TextModifierOptions::Bold => Modifier::BOLD,
            TextModifierOptions::CrossedOut => Modifier::CROSSED_OUT,
            TextModifierOptions::Dim => Modifier::DIM,
            TextModifierOptions::Hidden => Modifier::HIDDEN,
            TextModifierOptions::Italic => Modifier::ITALIC,
            TextModifierOptions::None => Modifier::empty(),
            TextModifierOptions::RapidBlink => Modifier::RAPID_BLINK,
            TextModifierOptions::Reversed => Modifier::REVERSED,
            TextModifierOptions::SlowBlink => Modifier::SLOW_BLINK,
            TextModifierOptions::Underlined => Modifier::UNDERLINED,
        }
    }
}

pub trait Renderable {
    fn render(rect: &mut Frame, app: &mut App, is_active: bool);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Copy, Default, EnumString)]
pub enum View {
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

impl View {
    pub fn from_string(s: &str) -> Option<View> {
        match s {
            "Body and Help" => Some(View::BodyHelp),
            "Body, Help and Log" => Some(View::BodyHelpLog),
            "Body and Log" => Some(View::BodyLog),
            "Config" => Some(View::ConfigMenu),
            "Create Theme" => Some(View::CreateTheme),
            "Edit Keybindings" => Some(View::EditKeybindings),
            "Help Menu" => Some(View::HelpMenu),
            "Load a Save (Cloud)" => Some(View::LoadCloudSave),
            "Load a Save (Local)" => Some(View::LoadLocalSave),
            "Login" => Some(View::Login),
            "Logs Only" => Some(View::LogsOnly),
            "Main Menu" => Some(View::MainMenu),
            "New Board" => Some(View::NewBoard),
            "New Card" => Some(View::NewCard),
            "Reset Password" => Some(View::ResetPassword),
            "Sign Up" => Some(View::SignUp),
            "Title and Body" => Some(View::TitleBody),
            "Title, Body and Help" => Some(View::TitleBodyHelp),
            "Title, Body, Help and Log" => Some(View::TitleBodyHelpLog),
            "Title, Body and Log" => Some(View::TitleBodyLog),
            "Zen" => Some(View::Zen),
            _ => None,
        }
    }

    pub fn from_number(n: u8) -> View {
        match n {
            1 => View::Zen,
            2 => View::TitleBody,
            3 => View::BodyHelp,
            4 => View::BodyLog,
            5 => View::TitleBodyHelp,
            6 => View::TitleBodyLog,
            7 => View::BodyHelpLog,
            8 => View::TitleBodyHelpLog,
            9 => View::LogsOnly,
            _ => {
                log::error!("Invalid View: {}", n);
                View::TitleBody
            }
        }
    }

    pub fn get_available_targets(&self) -> Vec<Focus> {
        match self {
            View::BodyHelp => vec![Focus::Body, Focus::Help],
            View::BodyHelpLog => vec![Focus::Body, Focus::Help, Focus::Log],
            View::BodyLog => vec![Focus::Body, Focus::Log],
            View::ConfigMenu => vec![Focus::ConfigTable, Focus::SubmitButton, Focus::ExtraFocus],
            View::CreateTheme => vec![Focus::ThemeEditor, Focus::SubmitButton, Focus::ExtraFocus],
            View::EditKeybindings => vec![Focus::EditKeybindingsTable, Focus::SubmitButton],
            View::HelpMenu => vec![Focus::Help, Focus::Log],
            View::LoadCloudSave => vec![Focus::Body],
            View::LoadLocalSave => vec![Focus::Body],
            View::Login => vec![
                Focus::Title,
                Focus::EmailIDField,
                Focus::PasswordField,
                Focus::ExtraFocus,
                Focus::SubmitButton,
            ],
            View::LogsOnly => vec![Focus::Log],
            View::MainMenu => vec![Focus::MainMenu, Focus::Help, Focus::Log],
            View::NewBoard => vec![
                Focus::NewBoardName,
                Focus::NewBoardDescription,
                Focus::SubmitButton,
            ],
            View::NewCard => vec![
                Focus::CardName,
                Focus::CardDescription,
                Focus::CardDueDate,
                Focus::SubmitButton,
            ],
            View::ResetPassword => vec![
                Focus::Title,
                Focus::EmailIDField,
                Focus::SendResetPasswordLinkButton,
                Focus::ResetPasswordLinkField,
                Focus::PasswordField,
                Focus::ConfirmPasswordField,
                Focus::ExtraFocus,
                Focus::SubmitButton,
            ],
            View::SignUp => vec![
                Focus::Title,
                Focus::EmailIDField,
                Focus::PasswordField,
                Focus::ConfirmPasswordField,
                Focus::ExtraFocus,
                Focus::SubmitButton,
            ],
            View::TitleBody => vec![Focus::Title, Focus::Body],
            View::TitleBodyHelp => vec![Focus::Title, Focus::Body, Focus::Help],
            View::TitleBodyHelpLog => vec![Focus::Title, Focus::Body, Focus::Help, Focus::Log],
            View::TitleBodyLog => vec![Focus::Title, Focus::Body, Focus::Log],
            View::Zen => vec![Focus::Body],
        }
    }

    pub fn all_views_as_string() -> Vec<String> {
        View::views_with_kanban_board()
            .iter()
            .map(|x| x.to_string())
            .collect()
    }

    pub fn views_with_kanban_board() -> Vec<View> {
        vec![
            View::Zen,
            View::TitleBody,
            View::BodyHelp,
            View::BodyLog,
            View::TitleBodyHelp,
            View::TitleBodyLog,
            View::BodyHelpLog,
            View::TitleBodyHelpLog,
        ]
    }

    pub fn render(self, rect: &mut Frame, app: &mut App, is_active: bool) {
        if is_active {
            let current_focus = app.state.focus;
            if !self.get_available_targets().contains(&current_focus)
                && !self.get_available_targets().is_empty()
            {
                app.state.set_focus(self.get_available_targets()[0]);
            }
        }
        match self {
            View::Zen => {
                Zen::render(rect, app, is_active);
            }
            View::TitleBody => {
                TitleBody::render(rect, app, is_active);
            }
            View::BodyHelp => {
                BodyHelp::render(rect, app, is_active);
            }
            View::BodyLog => {
                BodyLog::render(rect, app, is_active);
            }
            View::TitleBodyHelp => {
                TitleBodyHelp::render(rect, app, is_active);
            }
            View::TitleBodyLog => {
                TitleBodyLog::render(rect, app, is_active);
            }
            View::BodyHelpLog => {
                BodyHelpLog::render(rect, app, is_active);
            }
            View::TitleBodyHelpLog => {
                TitleBodyHelpLog::render(rect, app, is_active);
            }
            View::ConfigMenu => {
                ConfigMenu::render(rect, app, is_active);
            }
            View::EditKeybindings => {
                EditKeybindings::render(rect, app, is_active);
            }
            View::MainMenu => {
                MainMenuView::render(rect, app, is_active);
            }
            View::HelpMenu => {
                HelpMenu::render(rect, app, is_active);
            }
            View::LogsOnly => {
                LogView::render(rect, app, is_active);
            }
            View::NewBoard => {
                NewBoardForm::render(rect, app, is_active);
            }
            View::NewCard => NewCardForm::render(rect, app, is_active),
            View::LoadLocalSave => {
                LoadASave::render(rect, app, is_active);
            }
            View::CreateTheme => CreateTheme::render(rect, app, is_active),
            View::Login => Login::render(rect, app, is_active),
            View::SignUp => Signup::render(rect, app, is_active),
            View::ResetPassword => ResetPassword::render(rect, app, is_active),
            View::LoadCloudSave => LoadCloudSave::render(rect, app, is_active),
        }
    }
}

impl fmt::Display for View {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            View::BodyHelp => write!(f, "Body and Help"),
            View::BodyHelpLog => write!(f, "Body, Help and Log"),
            View::BodyLog => write!(f, "Body and Log"),
            View::ConfigMenu => write!(f, "Config"),
            View::CreateTheme => write!(f, "Create Theme"),
            View::EditKeybindings => write!(f, "Edit Keybindings"),
            View::HelpMenu => write!(f, "Help Menu"),
            View::LoadCloudSave => write!(f, "Load a Save (Cloud)"),
            View::LoadLocalSave => write!(f, "Load a Save (Local)"),
            View::Login => write!(f, "Login"),
            View::LogsOnly => write!(f, "Logs Only"),
            View::MainMenu => write!(f, "Main Menu"),
            View::NewBoard => write!(f, "New Board"),
            View::NewCard => write!(f, "New Card"),
            View::ResetPassword => write!(f, "Reset Password"),
            View::SignUp => write!(f, "Sign Up"),
            View::TitleBody => write!(f, "Title and Body"),
            View::TitleBodyHelp => write!(f, "Title, Body and Help"),
            View::TitleBodyHelpLog => write!(f, "Title, Body, Help and Log"),
            View::TitleBodyLog => write!(f, "Title, Body and Log"),
            View::Zen => write!(f, "Zen"),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum PopUp {
    ViewCard,
    CommandPalette,
    EditSpecificKeyBinding,
    ChangeView,
    CardStatusSelector,
    EditGeneralConfig,
    SelectDefaultView,
    ChangeDateFormatPopup,
    ChangeTheme,
    EditThemeStyle,
    SaveThemePrompt,
    CustomHexColorPromptFG,
    CustomHexColorPromptBG,
    ConfirmDiscardCardChanges,
    CardPrioritySelector,
    FilterByTag,
    DateTimePicker,
}

impl fmt::Display for PopUp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            PopUp::ViewCard => write!(f, "Card View"),
            PopUp::CommandPalette => write!(f, "Command Palette"),
            PopUp::EditSpecificKeyBinding => write!(f, "Edit Specific Key Binding"),
            PopUp::ChangeView => write!(f, "Change View"),
            PopUp::CardStatusSelector => write!(f, "Change Card Status"),
            PopUp::EditGeneralConfig => write!(f, "Edit General Config"),
            PopUp::SelectDefaultView => write!(f, "Select Default View"),
            PopUp::ChangeDateFormatPopup => write!(f, "Change Date Format"),
            PopUp::ChangeTheme => write!(f, "Change Theme"),
            PopUp::EditThemeStyle => write!(f, "Edit Theme Style"),
            PopUp::SaveThemePrompt => write!(f, "Save Theme Prompt"),
            PopUp::CustomHexColorPromptFG => write!(f, "Custom Hex Color Prompt FG"),
            PopUp::CustomHexColorPromptBG => write!(f, "Custom Hex Color Prompt BG"),
            PopUp::ConfirmDiscardCardChanges => write!(f, "Confirm Discard Card Changes"),
            PopUp::CardPrioritySelector => write!(f, "Change Card Priority"),
            PopUp::FilterByTag => write!(f, "Filter By Tag"),
            PopUp::DateTimePicker => write!(f, "Date Time Picker"),
        }
    }
}

impl PopUp {
    pub fn get_available_targets(&self) -> Vec<Focus> {
        match self {
            PopUp::ViewCard => vec![
                Focus::CardName,
                Focus::CardDescription,
                Focus::CardDueDate,
                Focus::CardPriority,
                Focus::CardStatus,
                Focus::CardTags,
                Focus::CardComments,
                Focus::SubmitButton,
            ],
            PopUp::CommandPalette => vec![
                Focus::CommandPaletteCommand,
                Focus::CommandPaletteCard,
                Focus::CommandPaletteBoard,
            ],
            PopUp::EditSpecificKeyBinding => vec![],
            PopUp::ChangeView => vec![],
            PopUp::CardStatusSelector => vec![],
            PopUp::EditGeneralConfig => vec![],
            PopUp::SelectDefaultView => vec![],
            PopUp::ChangeDateFormatPopup => vec![],
            PopUp::ChangeTheme => vec![],
            PopUp::EditThemeStyle => vec![
                Focus::StyleEditorFG,
                Focus::StyleEditorBG,
                Focus::StyleEditorModifier,
                Focus::SubmitButton,
            ],
            PopUp::SaveThemePrompt => vec![Focus::SubmitButton, Focus::ExtraFocus],
            PopUp::CustomHexColorPromptFG => vec![Focus::TextInput, Focus::SubmitButton],
            PopUp::CustomHexColorPromptBG => vec![Focus::TextInput, Focus::SubmitButton],
            PopUp::ConfirmDiscardCardChanges => vec![Focus::SubmitButton, Focus::ExtraFocus],
            PopUp::CardPrioritySelector => vec![],
            PopUp::FilterByTag => vec![Focus::FilterByTagPopup, Focus::SubmitButton],
            PopUp::DateTimePicker => vec![
                Focus::DTPCalender,
                Focus::DTPMonth,
                Focus::DTPYear,
                Focus::DTPToggleTimePicker,
                Focus::DTPHour,
                Focus::DTPMinute,
                Focus::DTPSecond,
            ],
        }
    }

    pub fn render(self, rect: &mut Frame, app: &mut App, is_active: bool) {
        if is_active {
            let current_focus = app.state.focus;
            if !self.get_available_targets().contains(&current_focus)
                && !self.get_available_targets().is_empty()
            {
                app.state.set_focus(self.get_available_targets()[0]);
            }
        }
        match self {
            PopUp::ViewCard => {
                ViewCard::render(rect, app, is_active);
            }
            PopUp::CardStatusSelector => {
                CardStatusSelector::render(rect, app, is_active);
            }
            PopUp::ChangeView => {
                ChangeView::render(rect, app, is_active);
            }
            PopUp::CommandPalette => {
                CommandPalette::render(rect, app, is_active);
            }
            PopUp::EditGeneralConfig => {
                EditGeneralConfig::render(rect, app, is_active);
            }
            PopUp::EditSpecificKeyBinding => {
                EditSpecificKeybinding::render(rect, app, is_active);
            }
            PopUp::SelectDefaultView => {
                SelectDefaultView::render(rect, app, is_active);
            }
            PopUp::ChangeTheme => {
                ChangeTheme::render(rect, app, is_active);
            }
            PopUp::EditThemeStyle => {
                EditThemeStyle::render(rect, app, is_active);
            }
            PopUp::SaveThemePrompt => {
                SaveThemePrompt::render(rect, app, is_active);
            }
            PopUp::CustomHexColorPromptFG | PopUp::CustomHexColorPromptBG => {
                CustomHexColorPrompt::render(rect, app, is_active);
            }
            PopUp::ConfirmDiscardCardChanges => {
                ConfirmDiscardCardChanges::render(rect, app, is_active);
            }
            PopUp::CardPrioritySelector => {
                CardPrioritySelector::render(rect, app, is_active);
            }
            PopUp::FilterByTag => {
                FilterByTag::render(rect, app, is_active);
            }
            PopUp::ChangeDateFormatPopup => {
                ChangeDateFormat::render(rect, app, is_active);
            }
            PopUp::DateTimePicker => {
                DateTimePicker::render(rect, app, is_active);
            }
        }
    }
}
