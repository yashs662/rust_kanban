use crate::{
    app::{actions::Action, kanban::Card, VisibleBoardsAndCards},
    constants::{DEFAULT_VIEW, MOUSE_OUT_OF_BOUNDS_COORDINATES},
    inputs::{key::Key, mouse::Mouse},
    io::io_handler::CloudData,
    ui::{text_box::TextBox, theme::Theme, PopUp, View},
    util::get_term_bg_color,
};
use linked_hash_map::LinkedHashMap;
use log::debug;
use ratatui::widgets::{ListState, TableState};
use serde::{Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
    time::Instant,
    vec,
};
use strum::{Display, EnumString, IntoEnumIterator};
use strum_macros::EnumIter;

#[derive(Debug, Clone)]
pub struct AppState<'a> {
    pub all_available_tags: Option<Vec<(String, u32)>>,
    pub app_list_states: AppListStates,
    pub app_status: AppStatus,
    pub app_table_states: AppTableStates,
    pub card_being_edited: Option<((u64, u64), Card)>, // (board_id, card)
    pub card_drag_mode: bool,
    pub cloud_data: Option<Vec<CloudData>>,
    pub current_board_id: Option<(u64, u64)>,
    pub current_card_id: Option<(u64, u64)>,
    pub current_mouse_coordinates: (u16, u16),
    pub debug_menu_toggled: bool,
    pub default_theme_mode: bool,
    pub edited_keybinding: Option<Vec<Key>>,
    pub encryption_key_from_arguments: Option<String>,
    pub filter_tags: Option<Vec<String>>,
    pub focus: Focus,
    pub hovered_board: Option<(u64, u64)>,
    pub hovered_card_dimensions: Option<(u16, u16)>,
    pub hovered_card: Option<((u64, u64), (u64, u64))>,
    pub last_mouse_action: Option<Mouse>,
    pub last_reset_password_link_sent_time: Option<Instant>,
    pub mouse_focus: Option<Focus>,
    pub mouse_list_index: Option<u16>,
    pub z_stack: ZStack,
    pub prev_focus: Option<Focus>,
    pub prev_view: Option<View>,
    pub preview_file_name: Option<String>,
    pub preview_visible_boards_and_cards: VisibleBoardsAndCards,
    pub previous_mouse_coordinates: (u16, u16),
    pub term_background_color: (u8, u8, u8),
    pub theme_being_edited: Theme,
    pub current_view: View,
    pub ui_render_time: Vec<u128>,
    pub user_login_data: UserLoginData,
    pub path_check_state: PathCheckState,
    pub text_buffers: TextBuffers<'a>,
    pub show_password: bool,
    pub last_cursor_set_pos: (u16, u16),
}

impl AppState<'_> {
    pub fn set_focus(&mut self, focus: Focus) {
        self.focus = focus;
    }
    pub fn get_card_being_edited(&self) -> Option<((u64, u64), Card)> {
        self.card_being_edited.clone()
    }
    pub fn get_theme_being_edited(&self) -> Theme {
        self.theme_being_edited.clone()
    }
}

impl Default for AppState<'_> {
    fn default() -> AppState<'static> {
        AppState {
            all_available_tags: None,
            app_list_states: AppListStates::default(),
            app_status: AppStatus::default(),
            app_table_states: AppTableStates::default(),
            card_being_edited: None,
            card_drag_mode: false,
            cloud_data: None,
            current_board_id: None,
            current_card_id: None,
            current_mouse_coordinates: MOUSE_OUT_OF_BOUNDS_COORDINATES, // make sure it's out of bounds when mouse mode is disabled
            debug_menu_toggled: false,
            default_theme_mode: false,
            edited_keybinding: None,
            encryption_key_from_arguments: None,
            filter_tags: None,
            focus: Focus::NoFocus,
            hovered_board: None,
            hovered_card_dimensions: None,
            hovered_card: None,
            last_mouse_action: None,
            last_reset_password_link_sent_time: None,
            mouse_focus: None,
            mouse_list_index: None,
            z_stack: ZStack::default(),
            prev_focus: None,
            prev_view: None,
            preview_file_name: None,
            preview_visible_boards_and_cards: LinkedHashMap::new(),
            previous_mouse_coordinates: MOUSE_OUT_OF_BOUNDS_COORDINATES,
            term_background_color: get_term_bg_color(),
            theme_being_edited: Theme::default(),
            current_view: DEFAULT_VIEW,
            ui_render_time: Vec::new(),
            user_login_data: UserLoginData {
                email_id: None,
                auth_token: None,
                refresh_token: None,
                user_id: None,
            },
            path_check_state: PathCheckState::default(),
            text_buffers: TextBuffers::default(),
            show_password: false,
            last_cursor_set_pos: MOUSE_OUT_OF_BOUNDS_COORDINATES,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ZStack(Vec<PopUp>);

impl ZStack {
    pub fn checked_disabled_last(&self) -> Option<&PopUp> {
        if let Some(popup) = self.0.last() {
            if self.0.len() > 1 && popup.requires_previous_element_disabled() {
                self.0.get(self.0.len() - 2)
            } else {
                Some(popup)
            }
        } else {
            None
        }
    }

    pub fn checked_control_last(&self) -> Option<&PopUp> {
        if let Some(popup) = self.0.last() {
            if self.0.len() > 1 && popup.requires_previous_element_control() {
                self.0.get(self.0.len() - 2)
            } else {
                Some(popup)
            }
        } else {
            None
        }
    }
}

impl Deref for ZStack {
    type Target = Vec<PopUp>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZStack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppListStates {
    pub card_priority_selector: ListState,
    pub card_status_selector: ListState,
    pub card_view_comment_list: ListState,
    pub card_view_list: ListState,
    pub card_view_tag_list: ListState,
    pub tag_picker: ListState,
    pub command_palette_board_search: ListState,
    pub command_palette_card_search: ListState,
    pub command_palette_command_search: ListState,
    pub date_format_selector: ListState,
    pub default_view: ListState,
    pub edit_specific_style: [ListState; 3],
    pub filter_by_tag_list: ListState,
    pub load_save: ListState,
    pub logs: ListState,
    pub main_menu: ListState,
    pub theme_selector: ListState,
}

#[derive(Debug, Clone, Default)]
pub struct AppTableStates {
    pub config: TableState,
    pub edit_keybindings: TableState,
    pub help: TableState,
    pub theme_editor: TableState,
}

#[derive(Debug, Clone)]
pub struct TextBuffers<'a> {
    pub board_name: TextBox<'a>,
    pub board_description: TextBox<'a>,
    pub card_name: TextBox<'a>,
    pub card_description: TextBox<'a>,
    pub card_tags: Vec<TextBox<'a>>,
    pub card_comments: Vec<TextBox<'a>>,
    pub email_id: TextBox<'a>,
    pub password: TextBox<'a>,
    pub confirm_password: TextBox<'a>,
    pub reset_password_link: TextBox<'a>,
    pub general_config: TextBox<'a>,
    pub command_palette: TextBox<'a>,
    pub theme_editor_fg_hex: TextBox<'a>,
    pub theme_editor_bg_hex: TextBox<'a>,
}

impl Default for TextBuffers<'_> {
    fn default() -> Self {
        TextBuffers {
            board_name: TextBox::new(vec!["".to_string()], true),
            board_description: TextBox::new(vec!["".to_string()], false),
            card_name: TextBox::new(vec!["".to_string()], true),
            card_description: TextBox::new(vec!["".to_string()], false),
            card_tags: Vec::new(),
            card_comments: Vec::new(),
            email_id: TextBox::new(vec!["".to_string()], true),
            password: TextBox::new(vec!["".to_string()], true),
            confirm_password: TextBox::new(vec!["".to_string()], true),
            reset_password_link: TextBox::new(vec!["".to_string()], true),
            general_config: TextBox::new(vec!["".to_string()], true),
            command_palette: TextBox::new(vec!["".to_string()], true),
            theme_editor_fg_hex: TextBox::new(vec!["".to_string()], true),
            theme_editor_bg_hex: TextBox::new(vec!["".to_string()], true),
        }
    }
}

impl TextBuffers<'_> {
    pub fn prepare_tags_and_comments_for_card(&mut self, card: &Card) {
        self.card_tags = card
            .tags
            .iter()
            .map(|tag| TextBox::new(vec![tag.clone()], true))
            .collect();
        self.card_comments = card
            .comments
            .iter()
            .map(|comment| TextBox::new(vec![comment.clone()], true))
            .collect();
    }
}

#[derive(Debug, Clone, Default)]
pub struct PathCheckState {
    pub path_last_checked: String,
    pub path_exists: bool,
    pub potential_completion: Option<String>,
    pub recheck_required: bool,
    pub path_check_mode: bool,
}

#[derive(Debug, Clone, Default)]
pub struct UserLoginData {
    pub auth_token: Option<String>,
    pub email_id: Option<String>,
    pub refresh_token: Option<String>,
    pub user_id: Option<String>,
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
    ChangeViewPopup,
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
    DTPCalender,
    DTPMonth,
    DTPYear,
    DTPToggleTimePicker,
    DTPHour,
    DTPMinute,
    DTPSecond,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyBindings {
    pub accept: Vec<Key>,
    pub change_card_status_to_active: Vec<Key>,
    pub change_card_status_to_completed: Vec<Key>,
    pub change_card_status_to_stale: Vec<Key>,
    pub change_card_priority_to_high: Vec<Key>,
    pub change_card_priority_to_medium: Vec<Key>,
    pub change_card_priority_to_low: Vec<Key>,
    pub clear_all_toasts: Vec<Key>,
    pub delete_board: Vec<Key>,
    pub delete_card: Vec<Key>,
    pub down: Vec<Key>,
    pub go_to_main_menu: Vec<Key>,
    pub go_to_previous_view_or_cancel: Vec<Key>,
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, EnumIter, PartialEq, EnumString, Display)]
pub enum KeyBindingEnum {
    Accept,
    ChangeCardStatusToActive,
    ChangeCardStatusToCompleted,
    ChangeCardStatusToStale,
    ChangeCardPriorityToHigh,
    ChangeCardPriorityToMedium,
    ChangeCardPriorityToLow,
    ClearAllToasts,
    DeleteBoard,
    DeleteCard,
    Down,
    GoToMainMenu,
    GoToPreviousViewOrCancel,
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

impl AppStatus {
    pub fn initialized() -> Self {
        Self::Initialized
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }
}

impl Focus {
    pub fn next(&self, available_tabs: &[Focus]) -> Self {
        if let Some(index) = available_tabs.iter().position(|x| x == self) {
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
        if let Some(index) = available_tabs.iter().position(|x| x == self) {
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
                KeyBindingEnum::ChangeCardPriorityToHigh => &self.change_card_priority_to_high,
                KeyBindingEnum::ChangeCardPriorityToMedium => &self.change_card_priority_to_medium,
                KeyBindingEnum::ChangeCardPriorityToLow => &self.change_card_priority_to_low,
                KeyBindingEnum::ClearAllToasts => &self.clear_all_toasts,
                KeyBindingEnum::DeleteBoard => &self.delete_board,
                KeyBindingEnum::DeleteCard => &self.delete_card,
                KeyBindingEnum::Down => &self.down,
                KeyBindingEnum::GoToMainMenu => &self.go_to_main_menu,
                KeyBindingEnum::GoToPreviousViewOrCancel => &self.go_to_previous_view_or_cancel,
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
            KeyBindingEnum::ChangeCardPriorityToHigh => Action::ChangeCardPriorityToHigh,
            KeyBindingEnum::ChangeCardPriorityToMedium => Action::ChangeCardPriorityToMedium,
            KeyBindingEnum::ChangeCardPriorityToLow => Action::ChangeCardPriorityToLow,
            KeyBindingEnum::ClearAllToasts => Action::ClearAllToasts,
            KeyBindingEnum::DeleteBoard => Action::DeleteBoard,
            KeyBindingEnum::DeleteCard => Action::Delete,
            KeyBindingEnum::Down => Action::Down,
            KeyBindingEnum::GoToMainMenu => Action::GoToMainMenu,
            KeyBindingEnum::GoToPreviousViewOrCancel => Action::GoToPreviousViewOrCancel,
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
                KeyBindingEnum::ChangeCardPriorityToHigh => {
                    self.change_card_priority_to_high = keybinding
                }
                KeyBindingEnum::ChangeCardPriorityToMedium => {
                    self.change_card_priority_to_medium = keybinding
                }
                KeyBindingEnum::ChangeCardPriorityToLow => {
                    self.change_card_priority_to_low = keybinding
                }
                KeyBindingEnum::ClearAllToasts => self.clear_all_toasts = keybinding,
                KeyBindingEnum::DeleteBoard => self.delete_board = keybinding,
                KeyBindingEnum::DeleteCard => self.delete_card = keybinding,
                KeyBindingEnum::Down => self.down = keybinding,
                KeyBindingEnum::GoToMainMenu => self.go_to_main_menu = keybinding,
                KeyBindingEnum::GoToPreviousViewOrCancel => {
                    self.go_to_previous_view_or_cancel = keybinding
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
            KeyBindingEnum::ChangeCardPriorityToHigh => {
                Some(self.change_card_priority_to_high.clone())
            }
            KeyBindingEnum::ChangeCardPriorityToMedium => {
                Some(self.change_card_priority_to_medium.clone())
            }
            KeyBindingEnum::ChangeCardPriorityToLow => {
                Some(self.change_card_priority_to_low.clone())
            }
            KeyBindingEnum::ClearAllToasts => Some(self.clear_all_toasts.clone()),
            KeyBindingEnum::DeleteBoard => Some(self.delete_board.clone()),
            KeyBindingEnum::DeleteCard => Some(self.delete_card.clone()),
            KeyBindingEnum::Down => Some(self.down.clone()),
            KeyBindingEnum::GoToMainMenu => Some(self.go_to_main_menu.clone()),
            KeyBindingEnum::GoToPreviousViewOrCancel => {
                Some(self.go_to_previous_view_or_cancel.clone())
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
            change_card_status_to_completed: vec![Key::Char('1')],
            change_card_status_to_active: vec![Key::Char('2')],
            change_card_status_to_stale: vec![Key::Char('3')],
            change_card_priority_to_high: vec![Key::Char('4')],
            change_card_priority_to_medium: vec![Key::Char('5')],
            change_card_priority_to_low: vec![Key::Char('6')],
            clear_all_toasts: vec![Key::Char('t')],
            delete_board: vec![Key::Char('D')],
            delete_card: vec![Key::Char('d'), Key::Delete],
            down: vec![Key::Down],
            go_to_main_menu: vec![Key::Char('m')],
            go_to_previous_view_or_cancel: vec![Key::Esc],
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
