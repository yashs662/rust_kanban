// TODO: Unify the style of all the views, with comments (styles, chunks, etc etc) in the same order with comments

pub mod body_help;
pub mod body_help_log;
pub mod body_log;
pub mod config_menu;
pub mod create_theme;
pub mod edit_keybindings;
pub mod help_menu;
pub mod load_a_save;
pub mod load_cloud_save;
pub mod log_view;
pub mod login;
pub mod main_menu_view;
pub mod new_board_form;
pub mod new_card_form;
pub mod reset_password;
pub mod signup;
pub mod title_body;
pub mod title_body_help;
pub mod title_body_help_log;
pub mod title_body_log;
pub mod zen;

pub struct Zen;
pub struct TitleBody;
pub struct BodyHelp;
pub struct BodyLog;
pub struct TitleBodyHelp;
pub struct TitleBodyLog;
pub struct BodyHelpLog;
pub struct TitleBodyHelpLog;
pub struct ConfigMenu;
pub struct EditKeybindings;
// TODO: see if this can be fixed; Another Struct with name MainMenu exists (reason for breaking the pattern)
pub struct MainMenuView;
pub struct HelpMenu;
pub struct LogView;
pub struct NewBoardForm;
pub struct NewCardForm;
pub struct LoadASave;
pub struct CreateTheme;
pub struct Login;
pub struct Signup;
pub struct ResetPassword;
pub struct LoadCloudSave;
