use super::{
    data_handler::{get_available_local_save_files, get_local_kanban_state},
    IoEvent,
};
use crate::{
    app::{
        app_helper::handle_go_to_previous_ui_mode, kanban::Boards, state::UiMode, App, AppConfig,
        UserLoginData,
    },
    constants::{
        CONFIG_DIR_NAME, CONFIG_FILE_NAME, ENCRYPTION_KEY_FILE_NAME, MAX_PASSWORD_LENGTH,
        MIN_PASSWORD_LENGTH, MIN_TIME_BETWEEN_SENDING_RESET_LINK, REFRESH_TOKEN_FILE_NAME,
        REFRESH_TOKEN_SEPARATOR, SAVE_DIR_NAME, SUPABASE_ANON_KEY, SUPABASE_URL,
    },
    io::data_handler::{get_default_save_directory, get_saved_themes, save_kanban_state_locally},
    ui::TextColorOptions,
    util::{print_debug, print_error, print_info},
};
use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use base64::Engine;
use chrono::{NaiveDate, NaiveDateTime};
use eyre::{anyhow, Result};
use linked_hash_map::LinkedHashMap;
use log::{debug, error, info, warn};
use ratatui::widgets::ListState;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub struct IoAsyncHandler<'a> {
    app: Arc<tokio::sync::Mutex<App<'a>>>,
}

impl IoAsyncHandler<'_> {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> IoAsyncHandler {
        IoAsyncHandler { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::SaveLocalData => self.save_local_data().await,
            IoEvent::LoadSaveLocal => self.load_save_file_local().await,
            IoEvent::DeleteLocalSave => self.delete_local_save_file().await,
            IoEvent::ResetVisibleBoardsandCards => self.refresh_visible_boards_and_cards().await,
            IoEvent::AutoSave => self.auto_save().await,
            IoEvent::LoadLocalPreview => self.load_local_preview().await,
            IoEvent::Login(email_id, password) => self.cloud_login(email_id, password).await,
            IoEvent::Logout => self.cloud_logout().await,
            IoEvent::SignUp(email_id, password, confirm_password) => {
                self.cloud_signup(email_id, password, confirm_password)
                    .await
            }
            IoEvent::SendResetPasswordEmail(email_id) => {
                self.send_reset_password_email(email_id).await
            }
            IoEvent::ResetPassword(reset_link, new_password, confirm_password) => {
                self.reset_password(reset_link, new_password, confirm_password)
                    .await
            }
            IoEvent::SyncLocalData => self.sync_local_data().await,
            IoEvent::GetCloudData => self.get_cloud_data().await,
            IoEvent::LoadSaveCloud => self.load_save_file_cloud().await,
            IoEvent::LoadCloudPreview => self.preview_cloud_save().await,
            IoEvent::DeleteCloudSave => self.delete_cloud_save().await,
        };

        let mut app = self.app.lock().await;
        if let Err(err) = result {
            error!("Oops, something wrong happened 😢: {:?}", err);
            app.send_error_toast("Oops, something wrong happened 😢", None);
        }

        app.loaded();
    }

    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        let prepare_config_dir_status = prepare_config_dir();
        if prepare_config_dir_status.is_err() {
            error!("Cannot create config directory");
            app.send_error_toast("Cannot create config directory", None);
        }
        if !prepare_save_dir() {
            error!("Cannot create save directory");
            app.send_error_toast("Cannot create save directory", None);
        }
        prepare_boards(&mut app);
        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
        let saved_themes = get_saved_themes();
        if let Some(saved_themes) = saved_themes {
            app.all_themes.extend(saved_themes);
        }
        let default_theme = app.config.default_theme.clone();
        for theme in &app.all_themes {
            if theme.name == default_theme {
                app.current_theme = theme.clone();
                break;
            }
        }
        let bg = app.current_theme.general_style.bg;
        if bg.is_some() {
            app.state.term_background_color = TextColorOptions::from(bg.unwrap()).to_rgb();
        } else {
            app.state.term_background_color = (0, 0, 0)
        }
        app.state.ui_mode = app.config.default_view;
        info!("👍 Application initialized");
        app.initialized();
        if app.config.save_directory == get_default_save_directory() {
            app.send_warning_toast(
                "Save directory is set to a temporary directory,
            your operating system may delete it at any time. Please change it in the settings.",
                Some(Duration::from_secs(10)),
            );
        }
        app.send_info_toast("Application initialized", None);
        if app.config.auto_login {
            app.send_info_toast("Attempting to auto login", None);
            let user_login_data =
                test_refresh_token_on_disk(app.state.encryption_key_from_arguments.clone()).await;
            if user_login_data.is_err() {
                let refresh_token_file_path = get_config_dir();
                if refresh_token_file_path.is_err() {
                    error!("Cannot get config directory");
                    app.send_error_toast("Cannot get config directory", None);
                    return Ok(());
                }
                let mut refresh_token_file_path = refresh_token_file_path.unwrap();
                refresh_token_file_path.push(REFRESH_TOKEN_FILE_NAME);
                if refresh_token_file_path.exists() {
                    if let Err(err) = std::fs::remove_file(refresh_token_file_path) {
                        error!("Cannot delete refresh token file: {:?}", err);
                        app.send_error_toast("Cannot delete refresh token file", None);
                        return Ok(());
                    } else {
                        warn!("Previous access token has expired or does not exist. Please login again");
                        app.send_warning_toast("Previous access token has expired or does not exist. Please login again", None)
                    }
                } else {
                    warn!(
                        "Previous access token has expired or does not exist. Please login again"
                    );
                    app.send_warning_toast(
                        "Previous access token has expired or does not exist. Please login again",
                        None,
                    )
                }
            } else {
                let user_login_data = user_login_data.unwrap();
                app.state.user_login_data = user_login_data;
                app.main_menu.logged_in = true;
                app.send_info_toast("👍 Auto login successful", None);
            }
        }
        Ok(())
    }

    async fn save_local_data(&mut self) -> Result<()> {
        info!("🚀 Saving local data");
        let mut app = self.app.lock().await;
        if save_required(&mut app) {
            let board_data = app.boards.get_boards();
            let status = save_kanban_state_locally(board_data.to_vec(), &app.config);
            match status {
                Ok(_) => {
                    info!("👍 Local data saved");
                    app.send_info_toast("👍 Local data saved", None);
                }
                Err(err) => {
                    debug!("Cannot save local data: {:?}", err);
                    app.send_error_toast("Cannot save local data", None);
                }
            }
            Ok(())
        } else {
            warn!("No changes to save");
            app.send_warning_toast("No changes to save", None);
            Ok(())
        }
    }

    async fn load_save_file_local(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let save_file_index = app.state.app_list_states.load_save.selected().unwrap_or(0);
        let local_files = get_available_local_save_files(&app.config);
        let local_files = if let Some(local_files) = local_files {
            local_files
        } else {
            error!("Could not get local save files");
            app.send_error_toast("Could not get local save files", None);
            vec![]
        };
        if save_file_index >= local_files.len() {
            error!("Cannot load save file: No such file");
            app.send_error_toast("Cannot load save file: No such file", None);
            return Ok(());
        }
        let save_file_name = local_files[save_file_index].clone();
        info!("🚀 Loading save file: {}", save_file_name);
        let board_data = get_local_kanban_state(save_file_name.clone(), false, &app.config);
        match board_data {
            Ok(boards) => {
                app.boards.set_boards(boards);
                app.action_history_manager.reset();
                info!("👍 Save file {:?} loaded", save_file_name);
                app.send_info_toast(&format!("👍 Save file {:?} loaded", save_file_name), None);
            }
            Err(err) => {
                debug!("Cannot load save file: {:?}", err);
                app.send_error_toast("Cannot load save file", None);
            }
        }
        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
        app.state.ui_mode = app.config.default_view;
        Ok(())
    }

    async fn delete_local_save_file(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let file_list = get_available_local_save_files(&app.config);
        let file_list = if let Some(file_list) = file_list {
            file_list
        } else {
            error!("Cannot delete save file: no save files found");
            app.send_error_toast("Cannot delete save file: no save files found", None);
            return Ok(());
        };
        if app.state.app_list_states.load_save.selected().is_none() {
            error!("Cannot delete save file: no save file selected");
            app.send_error_toast("Cannot delete save file: no save file selected", None);
            return Ok(());
        }
        let selected = app.state.app_list_states.load_save.selected().unwrap_or(0);
        if selected >= file_list.len() {
            debug!("Cannot delete save file: index out of range");
            app.send_error_toast("Cannot delete save file: Something went wrong", None);
            return Ok(());
        }
        let file_name = file_list[selected].clone();
        info!("🚀 Deleting save file: {}", file_name);
        let path = app.config.save_directory.join(file_name);
        if !Path::new(&path).exists() {
            error!("Cannot delete save file: file not found");
            app.send_error_toast("Cannot delete save file: file not found", None);
            return Ok(());
        } else if let Err(err) = std::fs::remove_file(&path) {
            debug!("Cannot delete save file: {:?}", err);
            app.send_error_toast("Cannot delete save file: Something went wrong", None);
            app.state.app_list_states.load_save = ListState::default();
            return Ok(());
        } else {
            info!("👍 Save file deleted");
            app.send_info_toast("👍 Save file deleted", None);
        }
        let file_list = get_available_local_save_files(&app.config);
        let file_list = if let Some(file_list) = file_list {
            file_list
        } else {
            app.state.app_list_states.load_save = ListState::default();
            return Ok(());
        };
        if selected >= file_list.len() {
            if file_list.is_empty() {
                app.state.app_list_states.load_save = ListState::default();
            } else {
                app.state
                    .app_list_states
                    .load_save
                    .select(Some(file_list.len() - 1));
            }
        }
        Ok(())
    }

    async fn refresh_visible_boards_and_cards(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        refresh_visible_boards_and_cards(&mut app);
        Ok(())
    }

    async fn auto_save(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        match auto_save(&mut app).await {
            Ok(_) => Ok(()),
            Err(err) => Err(anyhow!(err)),
        }
    }

    async fn load_local_preview(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        if app.state.app_list_states.load_save.selected().is_none() {
            return Ok(());
        }
        app.preview_boards_and_cards = None;

        let save_file_index = app.state.app_list_states.load_save.selected().unwrap_or(0);
        let local_files = get_available_local_save_files(&app.config);
        let local_files = if let Some(local_files) = local_files {
            local_files
        } else {
            error!("Could not get local save files");
            app.send_error_toast("Could not get local save files", None);
            vec![]
        };
        if save_file_index >= local_files.len() {
            error!("Cannot load preview: No such file");
            app.send_error_toast("Cannot load preview: No such file", None);
            return Ok(());
        }
        let save_file_name = local_files[save_file_index].clone();
        let board_data = get_local_kanban_state(save_file_name.clone(), true, &app.config);
        match board_data {
            Ok(boards) => {
                app.preview_boards_and_cards = Some(boards);
                let mut visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
                    LinkedHashMap::new();
                for (counter, board) in app
                    .preview_boards_and_cards
                    .as_ref()
                    .unwrap()
                    .get_boards()
                    .iter()
                    .enumerate()
                {
                    if counter >= app.config.no_of_boards_to_show.into() {
                        break;
                    }
                    let mut visible_cards: Vec<(u64, u64)> = Vec::new();
                    if board.cards.len() > app.config.no_of_cards_to_show.into() {
                        for card in board
                            .cards
                            .get_all_cards()
                            .iter()
                            .take(app.config.no_of_cards_to_show.into())
                        {
                            visible_cards.push(card.id);
                        }
                    } else {
                        for card in board.cards.get_all_cards() {
                            visible_cards.push(card.id);
                        }
                    }

                    let mut visible_board: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
                        LinkedHashMap::new();
                    visible_board.insert(board.id, visible_cards);
                    visible_boards_and_cards.extend(visible_board);
                }
                app.state.preview_visible_boards_and_cards = visible_boards_and_cards;
                app.state.preview_file_name = Some(save_file_name);
            }
            Err(e) => {
                error!("Error loading preview: {}", e);
                app.send_error_toast("Error loading preview", None);
            }
        }
        Ok(())
    }

    async fn cloud_login(&mut self, email_id: String, password: String) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_some() {
                error!("Already logged in, Please logout first");
                app.send_error_toast("Already logged in, Please logout first", None);
                return Ok(());
            } else {
                info!("Logging in, please wait...");
                app.send_info_toast("Logging in, please wait...", None);
            }
            if email_id.is_empty() {
                error!("Email cannot be empty");
                app.send_error_toast("Email cannot be empty", None);
                return Ok(());
            } else if password.is_empty() {
                error!("Password cannot be empty");
                app.send_error_toast("Password cannot be empty", None);
                return Ok(());
            }
        }

        let login_for_user_status = login_for_user(&email_id, &password, false).await;
        if let Err(err) = login_for_user_status {
            debug!("Error logging in: {:?}", err);
            error!("Error logging in");
            let mut app = self.app.lock().await;
            app.send_error_toast("Error logging in", None);
            return Ok(());
        }
        let (access_token, user_id, refresh_token) = login_for_user_status.unwrap();
        let mut app = self.app.lock().await;
        app.state.user_login_data.auth_token = Some(access_token.to_string());
        app.state.user_login_data.refresh_token = Some(refresh_token.to_string());
        app.state.user_login_data.email_id = Some(email_id.to_string());
        app.state.user_login_data.user_id = Some(user_id.to_string());
        app.main_menu.logged_in = true;

        if app.config.auto_login {
            save_refresh_token_to_disk(
                &refresh_token,
                &email_id,
                app.state.encryption_key_from_arguments.clone(),
            )
            .await?;
        }

        if app.state.ui_mode == UiMode::Login {
            handle_go_to_previous_ui_mode(&mut app).await;
        }

        info!("👍 Logged in");
        app.send_info_toast("👍 Logged in", None);

        Ok(())
    }

    async fn cloud_logout(&mut self) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_none() {
                error!("Not logged in");
                app.send_error_toast("Not logged in", None);
                return Ok(());
            } else {
                info!("Logging out, please wait...");
                app.send_info_toast("Logging out, please wait...", None);
            }
        }
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/auth/v1/logout", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                format!(
                    "Bearer {}",
                    self.app
                        .lock()
                        .await
                        .state
                        .user_login_data
                        .auth_token
                        .as_ref()
                        .unwrap()
                ),
            )
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::NO_CONTENT {
            let mut app = self.app.lock().await;
            app.state.user_login_data = UserLoginData::default();
            app.main_menu.logged_in = false;
            info!("👍 Logged out");
            app.send_info_toast("👍 Logged out", None);
        } else {
            error!("Error logging out");
            let mut app = self.app.lock().await;
            app.send_error_toast("Error logging out", None);
        }
        delete_refresh_token_from_disk().await?;
        Ok(())
    }

    async fn cloud_signup(
        &mut self,
        email_id: String,
        password: String,
        confirm_password: String,
    ) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_some() {
                error!("Already logged in");
                app.send_error_toast("Already logged in", None);
                return Ok(());
            }
            if email_id.is_empty() {
                error!("Email cannot be empty");
                app.send_error_toast("Email cannot be empty", None);
                return Ok(());
            }
            if password.is_empty() || confirm_password.is_empty() {
                error!("Password cannot be empty");
                app.send_error_toast("Password cannot be empty", None);
                return Ok(());
            }
            if password != confirm_password {
                error!("Passwords do not match");
                app.send_error_toast("Passwords do not match", None);
                return Ok(());
            }

            let password_status = check_for_safe_password(&password);
            match password_status {
                PasswordStatus::Strong => {}
                PasswordStatus::MissingLowercase => {
                    error!("Password must contain at least one lowercase character");
                    app.send_error_toast(
                        "Password must contain at least one lowercase character",
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::MissingUppercase => {
                    error!("Password must contain at least one uppercase character");
                    app.send_error_toast(
                        "Password must contain at least one uppercase character",
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::MissingNumber => {
                    error!("Password must contain at least one number");
                    app.send_error_toast("Password must contain at least one number", None);
                    return Ok(());
                }
                PasswordStatus::MissingSpecialChar => {
                    error!("Password must contain at least one special character");
                    app.send_error_toast(
                        "Password must contain at least one special character",
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::TooShort => {
                    error!(
                        "Password must be at least {} characters long",
                        MIN_PASSWORD_LENGTH
                    );
                    app.send_error_toast(
                        &format!(
                            "Password must be at least {} characters long",
                            MIN_PASSWORD_LENGTH
                        ),
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::TooLong => {
                    error!(
                        "Password must be atmost {} characters long",
                        MAX_PASSWORD_LENGTH
                    );
                    app.send_error_toast(
                        &format!(
                            "Password must be atmost {} characters long",
                            MAX_PASSWORD_LENGTH
                        ),
                        None,
                    );
                }
            }

            info!("Signing up, please wait...");
            app.send_info_toast("Signing up, please wait...", None);
        }

        let request_body = json!(
            {
                "email": email_id,
                "password": password
            }
        );
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/auth/v1/signup", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .body(request_body.to_string())
            .send()
            .await?;
        let status = response.status();
        let body = response.json::<serde_json::Value>().await;
        if status == StatusCode::OK {
            match body {
                Ok(body) => {
                    let confirmation_sent = body.get("confirmation_sent_at");
                    match confirmation_sent {
                        Some(confirmation_sent) => {
                            let confirmation_sent = confirmation_sent.as_str();
                            if confirmation_sent.is_none() {
                                error!("Error signing up");
                                let mut app = self.app.lock().await;
                                app.send_error_toast("Error signing up", None);
                                return Ok(());
                            }
                            info!("👍 Confirmation email sent");
                            let mut app = self.app.lock().await;
                            app.send_info_toast("👍 Confirmation email sent", None);
                            let key = generate_new_encryption_key();
                            let save_result = save_user_encryption_key(&key);
                            if save_result.is_err() {
                                error!("Error saving encryption key");
                                debug!("Error saving encryption key: {:?}", save_result);
                                app.send_error_toast("Error saving encryption key", None);
                                return Ok(());
                            } else {
                                let save_path = save_result.unwrap();
                                info!("👍 Encryption key saved at {}", save_path);
                                app.send_info_toast(
                                    &format!("👍 Encryption key saved at {}", save_path),
                                    None,
                                );
                                warn!("Please keep this key safe, you will need it to decrypt your data, you will not be able to recover your data without it");
                                app.send_warning_toast(
                                    "Please keep this key safe, you will need it to decrypt your data, you will not be able to recover your data without it",
                                    None,
                                );
                            }
                        }
                        None => {
                            error!("Error signing up");
                            let mut app = self.app.lock().await;
                            app.send_error_toast("Error signing up", None);
                        }
                    }
                }
                Err(e) => {
                    error!("Error signing up: {}", e);
                    let mut app = self.app.lock().await;
                    app.send_error_toast("Error signing up", None);
                }
            }
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            error!("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢");
            debug!("status code {}, response body: {:?}", status, body);
            let mut app = self.app.lock().await;
            app.send_error_toast("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢", None);
        } else {
            error!("Error signing up");
            debug!("status code {}, response body: {:?}", status, body);
            let mut app = self.app.lock().await;
            app.send_error_toast("Error signing up", None);
        }
        Ok(())
    }

    async fn send_reset_password_email(&mut self, email_id: String) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if let Some(reset_time) = app.state.last_reset_password_link_sent_time {
                if reset_time.elapsed() < Duration::from_secs(MIN_TIME_BETWEEN_SENDING_RESET_LINK) {
                    let remaining_time = Duration::from_secs(MIN_TIME_BETWEEN_SENDING_RESET_LINK)
                        .checked_sub(reset_time.elapsed())
                        .unwrap();
                    error!(
                        "Please wait for {} seconds before sending another reset password email",
                        remaining_time.as_secs()
                    );
                    app.send_error_toast(
                        &format!(
                        "Please wait for {} seconds before sending another reset password email",
                        remaining_time.as_secs()
                    ),
                        None,
                    );
                    return Ok(());
                }
            }
            if email_id.is_empty() {
                error!("Email cannot be empty");
                app.send_error_toast("Email cannot be empty", None);
                return Ok(());
            } else {
                info!("Sending reset password email, please wait...");
                app.send_info_toast("Sending reset password email, please wait...", None);
            }
        }

        let request_body = json!({ "email": email_id });

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/auth/v1/recover", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .body(request_body.to_string())
            .send()
            .await?;

        let status = response.status();
        if status == StatusCode::OK {
            info!("👍 Reset password email sent");
            let mut app = self.app.lock().await;
            app.state.last_reset_password_link_sent_time = Some(Instant::now());
            app.send_info_toast("👍 Reset password email sent", None);
        } else if status == StatusCode::TOO_MANY_REQUESTS {
            let body = response.json::<serde_json::Value>().await;
            error!("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢");
            debug!("status code {}, response body: {:?}", status, body);
            let mut app = self.app.lock().await;
            app.send_error_toast("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢", None);
        } else {
            error!("Error sending reset password email");
            let mut app = self.app.lock().await;
            app.send_error_toast("Error sending reset password email", None);
        }
        Ok(())
    }

    async fn reset_password(
        &mut self,
        reset_link: String,
        new_password: String,
        confirm_password: String,
    ) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if reset_link.is_empty() {
                error!("Reset link cannot be empty");
                app.send_error_toast("Reset link cannot be empty", None);
                return Ok(());
            }
            if new_password.is_empty() || confirm_password.is_empty() {
                error!("Password cannot be empty");
                app.send_error_toast("Password cannot be empty", None);
                return Ok(());
            }
            if new_password != confirm_password {
                error!("Passwords do not match");
                app.send_error_toast("Passwords do not match", None);
                return Ok(());
            }
            let password_status = check_for_safe_password(&new_password);
            match password_status {
                PasswordStatus::Strong => {}
                PasswordStatus::MissingLowercase => {
                    error!("Password must contain at least one lowercase character");
                    app.send_error_toast(
                        "Password must contain at least one lowercase character",
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::MissingUppercase => {
                    error!("Password must contain at least one uppercase character");
                    app.send_error_toast(
                        "Password must contain at least one uppercase character",
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::MissingNumber => {
                    error!("Password must contain at least one number");
                    app.send_error_toast("Password must contain at least one number", None);
                    return Ok(());
                }
                PasswordStatus::MissingSpecialChar => {
                    error!("Password must contain at least one special character");
                    app.send_error_toast(
                        "Password must contain at least one special character",
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::TooShort => {
                    error!(
                        "Password must be at least {} characters long",
                        MIN_PASSWORD_LENGTH
                    );
                    app.send_error_toast(
                        &format!(
                            "Password must be at least {} characters long",
                            MIN_PASSWORD_LENGTH
                        ),
                        None,
                    );
                    return Ok(());
                }
                PasswordStatus::TooLong => {
                    error!(
                        "Password must be atmost {} characters long",
                        MAX_PASSWORD_LENGTH
                    );
                    app.send_error_toast(
                        &format!(
                            "Password must be atmost {} characters long",
                            MAX_PASSWORD_LENGTH
                        ),
                        None,
                    );
                }
            }

            info!("Resetting password, please wait...");
            app.send_info_toast("Resetting password, please wait...", None);
        }

        let client = reqwest::Client::new();
        let response = client.get(reset_link).send().await;
        match response {
            Ok(_) => {
                error!("Error verifying reset password link");
                let mut app = self.app.lock().await;
                app.send_error_toast("Error verifying reset password link", None);
            }
            Err(e) => {
                let mut app = self.app.lock().await;
                let error_url = e.url();
                if error_url.is_none() {
                    error!("Error verifying reset password link");
                    app.send_error_toast("Error verifying reset password link", None);
                    return Ok(());
                }
                let error_url = error_url.unwrap();
                let error_url = error_url.to_string();
                debug!("Error verifying reset password link: {}", error_url);
                let access_token = error_url.split("access_token=");
                let access_token = access_token.last();
                if access_token.is_none() {
                    error!("Error verifying reset password link");
                    app.send_error_toast("Error verifying reset password link", None);
                    return Ok(());
                }
                let mut access_token = access_token.unwrap().split("&expires_in");
                let access_token = access_token.next();
                if access_token.is_none() {
                    error!("Error verifying reset password link");
                    app.send_error_toast("Error verifying reset password link", None);
                    return Ok(());
                }
                drop(app);
                let access_token = access_token.unwrap();
                let request_body = json!({ "password": new_password });
                let reset_client = reqwest::Client::new();
                let reset_response = reset_client
                    .put(format!("{}/auth/v1/user", SUPABASE_URL))
                    .header("apikey", SUPABASE_ANON_KEY)
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", access_token))
                    .body(request_body.to_string())
                    .send()
                    .await?;

                let status = reset_response.status();
                let mut app = self.app.lock().await;
                match status {
                    StatusCode::OK => {
                        info!("👍 Password reset successful");
                        if app.state.ui_mode == UiMode::ResetPassword {
                            handle_go_to_previous_ui_mode(&mut app).await;
                        }
                        app.send_info_toast("👍 Password reset successful", None);
                    }
                    StatusCode::UNPROCESSABLE_ENTITY => {
                        error!(
                            "Error resetting password, new password cannot be same as old password"
                        );
                        debug!(
                            "Error resetting password: {:?}",
                            reset_response.text().await
                        );
                        app.send_error_toast(
                            "Error resetting password, new password cannot be same as old password",
                            None,
                        );
                    }
                    _ => {
                        error!("Error resetting password");
                        debug!(
                            "Error resetting password: {:?}",
                            reset_response.text().await
                        );
                        app.send_error_toast("Error resetting password", None);
                    }
                }
            }
        }
        Ok(())
    }

    async fn sync_local_data(&mut self) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_none() {
                error!("Not logged in");
                app.send_error_toast("Not logged in", None);
                return Ok(());
            } else {
                info!("Syncing local data, please wait...");
                app.send_info_toast("Syncing local data, please wait...", None);
            }
        }

        let save_ids = self.get_save_ids_for_user().await?;
        let max_save_id = save_ids.iter().max();
        let max_save_id = if let Some(max_save_id) = max_save_id {
            max_save_id + 1
        } else {
            0
        };

        let mut app = self.app.lock().await;
        let key = get_user_encryption_key(app.state.encryption_key_from_arguments.clone());
        if key.is_err() {
            error!("Error syncing local data, Could not get encryption key, If you have lost it please generate a new one using the -g flag");
            debug!(
                "Error syncing local data: {:?}, Could not get encryption key",
                key.err()
            );
            app.send_error_toast("Error syncing local data, Could not get encryption key, If you have lost it please generate a new one using the -g flag", None);
            return Ok(());
        }
        let key = key.unwrap();
        let encrypt_result = encrypt_save(&app.boards, &key);
        if encrypt_result.is_err() {
            error!("Error syncing local data");
            debug!(
                "Error syncing local data: {:?}, could not encrypt",
                encrypt_result.err()
            );
            app.send_error_toast("Error syncing local data", None);
            return Ok(());
        }
        let (encrypted_board_data, nonce) = encrypt_result.unwrap();
        let auth_token = app.state.user_login_data.auth_token.clone().unwrap();
        let user_id = app.state.user_login_data.user_id.clone().unwrap();
        drop(app);
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/rest/v1/user_data", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", auth_token))
            .body(
                json!(
                    {
                        "user_id": user_id,
                        "board_data": encrypted_board_data,
                        "save_id": max_save_id,
                        "nonce": nonce
                    }
                )
                .to_string(),
            )
            .send()
            .await?;

        let status = response.status();
        let mut app = self.app.lock().await;
        if status == StatusCode::CREATED {
            info!("👍 Local data synced to the cloud");
            app.send_info_toast("👍 Local data synced to the cloud", None);
            if app.state.cloud_data.is_some() {
                app.dispatch(IoEvent::GetCloudData).await;
            }
        } else {
            error!("Error syncing local data");
            debug!("Error syncing local data: {:?}", response.text().await);
            app.send_error_toast("Error syncing local data", None);
        }
        Ok(())
    }

    async fn get_save_ids_for_user(&mut self) -> Result<Vec<usize>> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_none() {
                error!("Not logged in");
                app.send_error_toast("Not logged in", None);
                return Ok(vec![]);
            }
        }

        let (user_id, access_token) = {
            let app = self.app.lock().await;
            let user_id = app.state.user_login_data.user_id.as_ref().unwrap().clone();
            let access_token = app
                .state
                .user_login_data
                .auth_token
                .as_ref()
                .unwrap()
                .clone();
            (user_id, access_token)
        };

        let result = get_all_save_ids_for_user(user_id, &access_token).await;
        if result.is_err() {
            let error_string = format!("{:?}", result.err());
            error!("{}", error_string);
            let mut app = self.app.lock().await;
            app.send_error_toast(&error_string, None);
            Ok(vec![])
        } else {
            let save_ids = result.unwrap();
            Ok(save_ids)
        }
    }

    async fn get_cloud_data(&mut self) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_none() {
                error!("Not logged in");
                app.send_error_toast("Not logged in", None);
                return Ok(());
            } else {
                info!("Refreshing cloud data, please wait...");
                app.send_info_toast("Refreshing cloud data, please wait...", None);
                app.state.cloud_data = None;
            }
        }

        let app = self.app.lock().await;
        let auth_token = app.state.user_login_data.auth_token.clone().unwrap();
        drop(app);
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/rest/v1/user_data", SUPABASE_URL))
            .header("apikey", SUPABASE_ANON_KEY)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", auth_token))
            .send()
            .await?;

        let mut app = self.app.lock().await;

        let status = response.status();
        if status == StatusCode::OK {
            let body = response.json::<Vec<CloudData>>().await;
            match body {
                Ok(cloud_data) => {
                    app.state.cloud_data = Some(cloud_data);
                    info!("👍 Cloud data loaded");
                    app.send_info_toast("👍 Cloud data loaded", None);
                }
                Err(e) => {
                    error!("Error Refreshing cloud data: {}", e);
                    app.send_error_toast("Error Refreshing cloud data", None);
                }
            }
        } else {
            error!("Error Refreshing cloud data");
            app.send_error_toast("Error Refreshing cloud data", None);
        }
        Ok(())
    }

    async fn preview_cloud_save(&mut self) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.app_list_states.load_save.selected().is_none() {
                error!("No save selected to preview");
                app.send_error_toast("No save selected to preview", None);
                return Ok(());
            }
        }

        let mut app = self.app.lock().await;
        let selected_index = app.state.app_list_states.load_save.selected().unwrap();
        let cloud_data = app.state.cloud_data.clone();
        if cloud_data.is_none() {
            debug!("No cloud data preview found to select");
            return Ok(());
        }
        let cloud_data = cloud_data.unwrap();
        if selected_index >= cloud_data.len() {
            debug!("Selected index is out of bounds");
            return Ok(());
        }
        let save = cloud_data[selected_index].clone();
        let key = get_user_encryption_key(app.state.encryption_key_from_arguments.clone());
        if key.is_err() {
            error!("Error loading save file, Could not get user Encryption key .If lost please generate a new one by using the -g flag");
            debug!("Error loading save file: {:?}", key.err());
            app.send_error_toast(
                "Error loading save file, Could not get user Encryption key .If lost please generate a new one by using the -g flag",
                None,
            );
            return Ok(());
        }
        let key = key.unwrap();
        let decrypt_result = decrypt_save(save.board_data, key.as_slice(), &save.nonce);
        if decrypt_result.is_err() {
            error!("Error loading save file, Could not decrypt save file. The save file must have been created with a different encryption key, either generate a new one with the -g flag or replace the current encryption key with the one used to create the save file");
            debug!("Error loading save file: {:?}", decrypt_result.err());
            app.send_error_toast("Error loading save file, Could not decrypt save file. The save file must have been created with a different encryption key, either generate a new one with the -g flag or replace the current encryption key with the one used to create the save file", None);
            return Ok(());
        }
        app.preview_boards_and_cards = Some(decrypt_result.unwrap());
        let mut visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
            LinkedHashMap::new();
        for (counter, board) in app
            .preview_boards_and_cards
            .as_ref()
            .unwrap()
            .get_boards()
            .iter()
            .enumerate()
        {
            if counter >= app.config.no_of_boards_to_show.into() {
                break;
            }
            let mut visible_cards: Vec<(u64, u64)> = Vec::new();
            if board.cards.len() > app.config.no_of_cards_to_show.into() {
                for card in board
                    .cards
                    .get_all_cards()
                    .iter()
                    .take(app.config.no_of_cards_to_show.into())
                {
                    visible_cards.push(card.id);
                }
            } else {
                for card in board.cards.get_all_cards() {
                    visible_cards.push(card.id);
                }
            }

            let mut visible_board: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
                LinkedHashMap::new();
            visible_board.insert(board.id, visible_cards);
            visible_boards_and_cards.extend(visible_board);
        }
        let save_timestamp = save.created_at.split('.').next();
        if save_timestamp.is_none() {
            debug!("Error splitting {}", save.created_at);
            app.state.preview_visible_boards_and_cards = visible_boards_and_cards;
            app.state.preview_file_name = Some(format!("Cloud_save_{}", save.save_id,));
            return Ok(());
        }
        let save_timestamp = save_timestamp.unwrap();
        let save_date = NaiveDateTime::parse_from_str(save_timestamp, "%Y-%m-%dT%H:%M:%S");
        if save_date.is_ok() {
            let save_date = save_date.unwrap();
            let save_date = save_date.format(app.config.date_format.to_parser_string());
            app.state.preview_file_name =
                Some(format!("Cloud_save_{} - {}", save.save_id, save_date));
        } else {
            debug!("Error parsing save date {}", save.created_at);
        }
        app.state.preview_visible_boards_and_cards = visible_boards_and_cards;
        Ok(())
    }

    async fn load_save_file_cloud(&mut self) -> Result<()> {
        let mut app = self.app.lock().await;
        let save_file_index = app.state.app_list_states.load_save.selected().unwrap_or(0);
        let cloud_saves = app.state.cloud_data.clone();
        let local_files = if let Some(cloud_saves) = cloud_saves {
            cloud_saves
        } else {
            error!("Could not get local save files");
            app.send_error_toast("Could not get local save files", None);
            vec![]
        };
        if save_file_index >= local_files.len() {
            error!("Cannot load save file: No such file");
            app.send_error_toast("Cannot load save file: No such file", None);
            return Ok(());
        }
        let save_file_number = local_files[save_file_index].save_id;
        info!("🚀 Loading save file: cloud_save_{}", save_file_number);
        let encrypted_board_data = &local_files[save_file_index].board_data;
        let key = get_user_encryption_key(app.state.encryption_key_from_arguments.clone());
        if key.is_err() {
            error!("Error loading save file, Could not get user Encryption key. If lost please generate a new one by using the -g flag");
            debug!("Error loading save file: {:?}", key.err());
            app.send_error_toast(
                "Error loading save file, Could not get user Encryption key. If lost please generate a new one by using the -g flag",
                None,
            );
            return Ok(());
        }
        let key = key.unwrap();
        let decrypt_result = decrypt_save(
            encrypted_board_data.to_string(),
            key.as_slice(),
            &local_files[save_file_index].nonce,
        );
        if decrypt_result.is_err() {
            error!("Error loading save file, Could not decrypt save file. The save file must have been created with a different encryption key, either generate a new one with the -g flag or replace the current encryption key with the one used to create the save file");
            debug!("Error loading save file: {:?}", decrypt_result.err());
            app.send_error_toast("Error loading save file, Could not decrypt save file. The save file must have been created with a different encryption key, either generate a new one with the -g flag or replace the current encryption key with the one used to create the save file", None);
            return Ok(());
        }
        let decrypt_result = decrypt_result.unwrap();
        app.boards.set_boards(decrypt_result);
        info!("👍 Save file cloud_save_{} loaded", save_file_number);
        app.send_info_toast(
            &format!("👍 Save file cloud_save_{} loaded", save_file_number),
            None,
        );
        app.dispatch(IoEvent::ResetVisibleBoardsandCards).await;
        app.state.ui_mode = app.config.default_view;
        Ok(())
    }

    async fn delete_cloud_save(&mut self) -> Result<()> {
        {
            let mut app = self.app.lock().await;
            if app.state.user_login_data.auth_token.is_none() {
                error!("Not logged in");
                app.send_error_toast("Not logged in", None);
                return Ok(());
            } else {
                info!("Deleting cloud save, please wait...");
                app.send_info_toast("Deleting cloud save, please wait...", None);
            }
        }

        let mut app = self.app.lock().await;
        let save_file_index = app.state.app_list_states.load_save.selected().unwrap_or(0);
        let user_access_token = app.state.user_login_data.auth_token.clone().unwrap();
        let cloud_saves = app.state.cloud_data.clone();
        let cloud_saves = if let Some(cloud_saves) = cloud_saves {
            cloud_saves
        } else {
            error!("Could not get local save files");
            app.send_error_toast("Could not get local save files", None);
            return Ok(());
        };
        if save_file_index >= cloud_saves.len() {
            error!("Cannot delete save file: No such file");
            app.send_error_toast("Cannot delete save file: No such file", None);
            return Ok(());
        }
        drop(app);
        let save_file_id = cloud_saves[save_file_index].id;
        let save_number = cloud_saves[save_file_index].save_id;
        let delete_status =
            delete_a_save_from_database(&user_access_token, false, save_file_id, Some(save_number))
                .await;
        let mut app = self.app.lock().await;
        if delete_status.is_err() {
            app.send_error_toast("Error deleting cloud save", None);
        } else {
            app.send_info_toast(
                &format!("👍 Cloud save cloud_save_{} deleted", save_number),
                None,
            );
        }

        Ok(())
    }
}

pub(crate) fn get_config_dir() -> Result<PathBuf, String> {
    let home_dir = home::home_dir();
    if home_dir.is_none() {
        return Err(String::from("Error getting home directory"));
    }
    let mut config_dir = home_dir.unwrap();
    if cfg!(windows) {
        config_dir.push("AppData");
        config_dir.push("Roaming");
    } else {
        config_dir.push(".config");
    }
    config_dir.push(CONFIG_DIR_NAME);
    Ok(config_dir)
}

pub(crate) fn get_save_dir() -> PathBuf {
    let mut save_dir = env::temp_dir();
    save_dir.push(SAVE_DIR_NAME);
    save_dir
}

pub fn prepare_config_dir() -> Result<(), String> {
    let config_dir = get_config_dir();
    if config_dir.is_err() {
        return Err(String::from("Error getting config directory"));
    }
    let config_dir = config_dir.unwrap();
    if !config_dir.exists() {
        let dir_creation_status = std::fs::create_dir_all(&config_dir);
        if dir_creation_status.is_err() {
            return Err(String::from("Error creating config directory"));
        }
    }
    let mut config_file = config_dir;
    config_file.push(CONFIG_FILE_NAME);
    if !config_file.exists() {
        let default_config = AppConfig::default();
        let config_json = serde_json::to_string_pretty(&default_config);
        if let Ok(config_json) = config_json {
            let file_creation_status = std::fs::write(&config_file, config_json);
            if file_creation_status.is_err() {
                return Err(String::from("Error creating config file"));
            }
        } else {
            return Err(String::from("Error creating config file"));
        }
    }
    Ok(())
}

fn prepare_save_dir() -> bool {
    let save_dir = get_save_dir();
    if !save_dir.exists() {
        std::fs::create_dir_all(&save_dir).unwrap();
    }
    true
}

fn prepare_boards(app: &mut App) {
    let boards = if app.config.always_load_last_save {
        let latest_save_file_info = get_latest_save_file(&app.config);
        if let Ok(latest_save_file) = latest_save_file_info {
            let local_data = get_local_kanban_state(latest_save_file.clone(), false, &app.config);
            match local_data {
                Ok(data) => {
                    info!("👍 Local data loaded from {:?}", latest_save_file);
                    app.send_info_toast(
                        &format!("👍 Local data loaded from {:?}", latest_save_file),
                        None,
                    );
                    data
                }
                Err(err) => {
                    debug!("Cannot get local data: {:?}", err);
                    error!("👎 Cannot get local data, Data might be corrupted or is not in the correct format");
                    app.send_error_toast("👎 Cannot get local data, Data might be corrupted or is not in the correct format", None);
                    Boards::default()
                }
            }
        } else {
            Boards::default()
        }
    } else {
        app.set_ui_mode(UiMode::LoadLocalSave);
        Boards::default()
    };
    app.boards.set_boards(boards);
}

fn get_latest_save_file(config: &AppConfig) -> Result<String, String> {
    let local_save_files = get_available_local_save_files(config);
    let local_save_files = if let Some(local_save_files) = local_save_files {
        local_save_files
    } else {
        return Err("No local save files found".to_string());
    };
    let fall_back_version = -1;
    if local_save_files.is_empty() {
        return Err("No local save files found".to_string());
    }

    let latest_date = local_save_files
        .iter()
        .map(|file| {
            let date = file.split('_').collect::<Vec<&str>>()[1];
            NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap()
        })
        .max()
        .unwrap();

    let latest_version = local_save_files
        .iter()
        .filter(|file| {
            let date = file.split('_').collect::<Vec<&str>>()[1];
            NaiveDate::parse_from_str(date, "%d-%m-%Y").unwrap() == latest_date
        })
        .map(|file| {
            let version = file.split("_v").collect::<Vec<&str>>()[1];
            let version = version.split('.').collect::<Vec<&str>>()[0];
            version.parse::<i32>().unwrap_or(fall_back_version)
        })
        .max()
        .unwrap_or(fall_back_version);

    if latest_version == fall_back_version {
        return Err("No local save files found".to_string());
    }
    let latest_version = latest_version as u32;

    let latest_save_file = format!(
        "kanban_{}_v{}.json",
        latest_date.format("%d-%m-%Y"),
        latest_version
    );
    Ok(latest_save_file)
}

pub fn refresh_visible_boards_and_cards(app: &mut App) {
    let mut visible_boards_and_cards: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> =
        LinkedHashMap::new();
    let boards = if app.filtered_boards.is_empty() {
        app.boards.get_boards()
    } else {
        app.filtered_boards.get_boards()
    };
    for (i, board) in boards.iter().enumerate() {
        if (i) as u16 == app.config.no_of_boards_to_show {
            break;
        }
        let mut visible_cards: Vec<(u64, u64)> = Vec::new();
        if board.cards.len() > app.config.no_of_cards_to_show.into() {
            for card in board
                .cards
                .get_all_cards()
                .iter()
                .take(app.config.no_of_cards_to_show.into())
            {
                visible_cards.push(card.id);
            }
        } else {
            for card in board.cards.get_all_cards() {
                visible_cards.push(card.id);
            }
        }

        let mut visible_board: LinkedHashMap<(u64, u64), Vec<(u64, u64)>> = LinkedHashMap::new();
        visible_board.insert(board.id, visible_cards);
        visible_boards_and_cards.extend(visible_board);
    }
    app.visible_boards_and_cards = visible_boards_and_cards;
    if !app.visible_boards_and_cards.is_empty() {
        app.state.current_board_id = Some(*app.visible_boards_and_cards.keys().next().unwrap());
        if !app
            .visible_boards_and_cards
            .values()
            .next()
            .unwrap()
            .is_empty()
        {
            app.state.current_card_id =
                Some(app.visible_boards_and_cards.values().next().unwrap()[0]);
        }
    }
}

pub fn make_file_system_safe_name(name: &str) -> String {
    let mut safe_name = name.to_string();
    let unsafe_chars = vec!["/", "\\", ":", "*", "?", "\"", "<", ">", "|", " "];
    for unsafe_char in unsafe_chars {
        safe_name = safe_name.replace(unsafe_char, "");
    }
    safe_name
}

pub async fn auto_save(app: &mut App<'_>) -> Result<(), String> {
    if save_required(app) {
        save_kanban_state_locally(app.boards.get_boards().to_vec(), &app.config)
    } else {
        Ok(())
    }
}

fn save_required(app: &mut App) -> bool {
    let latest_save_file_info = get_latest_save_file(&app.config);
    if let Ok(save_file_name) = latest_save_file_info {
        let board_data = get_local_kanban_state(save_file_name, false, &app.config);
        match board_data {
            Ok(boards) => app.boards != boards,
            Err(_) => true,
        }
    } else {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudData {
    pub id: u64,
    pub created_at: String,
    pub user_id: String,
    pub board_data: String,
    pub nonce: String,
    pub save_id: usize,
}

enum PasswordStatus {
    Strong,
    MissingUppercase,
    MissingLowercase,
    MissingNumber,
    MissingSpecialChar,
    TooShort,
    TooLong,
}

fn check_for_safe_password(check_password: &str) -> PasswordStatus {
    let mut password_status = PasswordStatus::Strong;
    if check_password.len() < MIN_PASSWORD_LENGTH {
        password_status = PasswordStatus::TooShort;
    }
    if !check_password.chars().any(|c| c.is_uppercase()) {
        password_status = PasswordStatus::MissingUppercase;
    }
    if !check_password.chars().any(|c| c.is_lowercase()) {
        password_status = PasswordStatus::MissingLowercase;
    }
    if !check_password.chars().any(|c| c.is_numeric()) {
        password_status = PasswordStatus::MissingNumber;
    }
    if !check_password.chars().any(|c| c.is_ascii_punctuation()) {
        password_status = PasswordStatus::MissingSpecialChar;
    }
    if check_password.len() > MAX_PASSWORD_LENGTH {
        password_status = PasswordStatus::TooLong;
    }
    password_status
}

fn encrypt_save(boards: &Boards, key: &[u8]) -> Result<(String, String), String> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let boards_json = serde_json::to_string(&boards);
    if boards_json.is_err() {
        return Err("Error serializing boards".to_string());
    }
    let boards_json = boards_json.unwrap();
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let nonce_vec = nonce.to_vec();
    let nonce_encoded = base64_engine.encode(nonce_vec);
    let encrypted_boards = cipher.encrypt(&nonce, boards_json.as_bytes());
    if encrypted_boards.is_err() {
        return Err("Error encrypting boards".to_string());
    }
    let encrypted_boards = encrypted_boards.unwrap();
    let encoded_boards = base64_engine.encode(encrypted_boards);
    Ok((encoded_boards, nonce_encoded))
}

fn decrypt_save(
    encrypted_boards: String,
    key: &[u8],
    encoded_nonce: &str,
) -> Result<Boards, String> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let encrypted_boards = base64_engine.decode(encrypted_boards);
    if encrypted_boards.is_err() {
        return Err("Error decoding boards".to_string());
    }
    let encrypted_boards = encrypted_boards.unwrap();
    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = base64_engine.decode(encoded_nonce);
    if nonce.is_err() {
        return Err("Error decoding nonce".to_string());
    }
    let nonce = nonce.unwrap();
    let nonce = GenericArray::from_slice(&nonce);
    let decrypted_board_data = cipher.decrypt(nonce, encrypted_boards.as_slice());
    if decrypted_board_data.is_err() {
        return Err("Error decrypting boards".to_string());
    }
    let decrypted_board_data = decrypted_board_data.unwrap();
    let decrypted_board_data = String::from_utf8(decrypted_board_data);
    if decrypted_board_data.is_err() {
        return Err("Error converting decrypted boards to string".to_string());
    }
    let decrypted_board_data = decrypted_board_data.unwrap();
    let boards = serde_json::from_str(&decrypted_board_data);
    if boards.is_err() {
        return Err("Error deserializing boards".to_string());
    }
    Ok(boards.unwrap())
}

pub fn save_user_encryption_key(key: &[u8]) -> Result<String> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let key = base64_engine.encode(key);
    let mut config_dir = get_config_dir().unwrap();
    config_dir.push(ENCRYPTION_KEY_FILE_NAME);
    let file_creation_status = std::fs::write(&config_dir, key);
    if let Err(e) = file_creation_status {
        Err(anyhow!(e))
    } else {
        Ok(config_dir.to_str().unwrap().to_string())
    }
}

fn get_user_encryption_key(encryption_key_from_arguments: Option<String>) -> Result<Vec<u8>> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    if let Some(encryption_key_from_arguments) = encryption_key_from_arguments {
        let decoded_key = base64_engine.decode(encryption_key_from_arguments);
        if let Err(e) = decoded_key {
            Err(anyhow!(e))
        } else {
            Ok(decoded_key.unwrap())
        }
    } else {
        let mut encryption_key_path = get_config_dir().unwrap();
        encryption_key_path.push(ENCRYPTION_KEY_FILE_NAME);
        if !encryption_key_path.exists() {
            return Err(anyhow!("Encryption key file not found!! Please generate a new one by using the -g flag or move it to the path: {:?}", encryption_key_path));
        }
        let encoded_key = std::fs::read_to_string(&encryption_key_path);
        if let Err(e) = encoded_key {
            Err(anyhow!(e))
        } else {
            let key = encoded_key.unwrap();
            let decoded_key = base64_engine.decode(key);
            if let Err(e) = decoded_key {
                Err(anyhow!(e))
            } else {
                Ok(decoded_key.unwrap())
            }
        }
    }
}

pub fn generate_new_encryption_key() -> Vec<u8> {
    Aes256Gcm::generate_key(&mut OsRng).to_vec()
}

pub async fn get_all_save_ids_for_user(user_id: String, access_token: &str) -> Result<Vec<usize>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/rest/v1/user_data?user_id=eq.{}&select=save_id",
            SUPABASE_URL, user_id
        ))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Range", "0-9")
        .send()
        .await;
    if response.is_err() {
        debug!("Error getting save ids: {:?}", response.err());
        return Err(anyhow!("Error getting save ids".to_string()));
    }
    let response = response.unwrap();
    let status = response.status();
    if status == StatusCode::OK {
        let body = response.json::<serde_json::Value>().await;
        match body {
            Ok(save_instances) => {
                let mut save_ids: Vec<usize> = Vec::new();
                let save_instances_as_array = save_instances.as_array();
                if save_instances_as_array.is_none() {
                    return Err(anyhow!("Error getting save ids".to_string()));
                }
                let save_instances_as_array = save_instances_as_array.unwrap();
                for save_instance in save_instances_as_array {
                    let save_id = save_instance.get("save_id");
                    if save_id.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let save_id = save_id.unwrap().as_u64();
                    if save_id.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    debug!("Found save_id: {:?}", save_id.unwrap() as usize);
                    save_ids.push(save_id.unwrap() as usize);
                }
                debug!("Save instances: {:?}", save_instances);
                Ok(save_ids)
            }
            Err(e) => {
                debug!("Error getting save ids: {:?}", e);
                Err(anyhow!("Error getting save ids".to_string()))
            }
        }
    } else {
        debug!("Status: {:?}", status);
        debug!("Error getting save ids: {:?}", response.text().await);
        Err(anyhow!("Error getting save ids".to_string()))
    }
}

pub async fn get_all_save_ids_and_creation_dates_for_user(
    user_id: String,
    access_token: &str,
    cli_mode: bool,
) -> Result<Vec<(usize, String, usize)>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/rest/v1/user_data?user_id=eq.{}&select=save_id,created_at,id",
            SUPABASE_URL, user_id
        ))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;
    if let Err(e) = response {
        debug!("Error getting save ids and created_at: {:?}", e);
        return Err(anyhow!("Error getting save ids".to_string()));
    }
    let response = response.unwrap();
    let status = response.status();
    if status == StatusCode::OK {
        let body = response.json::<serde_json::Value>().await;
        match body {
            Ok(save_instances) => {
                let mut save_ids_and_creation_dates: Vec<(usize, String, usize)> = Vec::new();
                let save_instances_as_array = save_instances.as_array();
                if save_instances_as_array.is_none() {
                    return Err(anyhow!("Error getting save ids".to_string()));
                }
                let save_instances_as_array = save_instances_as_array.unwrap().to_owned();
                for save_instance in save_instances_as_array {
                    let save_id = save_instance.get("save_id");
                    if save_id.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let save_id = save_id.unwrap().as_u64();
                    if save_id.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let save_id = save_id.unwrap() as usize;
                    let created_at = save_instance.get("created_at");
                    if created_at.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let created_at = created_at.unwrap().as_str();
                    if created_at.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let created_at = created_at.unwrap().to_owned();
                    let id = save_instance.get("id");
                    if id.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let id = id.unwrap().as_u64();
                    if id.is_none() {
                        return Err(anyhow!("Error getting save ids".to_string()));
                    }
                    let id = id.unwrap() as usize;
                    if cli_mode {
                        print_debug(&format!("save_id: {:?}", save_id));
                        print_debug(&format!("created_at: {:?}", created_at));
                        print_debug(&format!("id: {:?}", id));
                    } else {
                        debug!("save_id: {:?}", save_id);
                        debug!("created_at: {:?}", created_at);
                        debug!("id: {:?}", id);
                    }
                    save_ids_and_creation_dates.push((save_id, created_at, id));
                }
                if cli_mode {
                    print_debug(&format!("save_instances: {:?}", save_instances));
                } else {
                    debug!("save_instances: {:?}", save_instances);
                }
                Ok(save_ids_and_creation_dates)
            }
            Err(e) => {
                if cli_mode {
                    print_debug(&format!("Error getting save ids: {:?}", e));
                } else {
                    debug!("Error getting save ids: {:?}", e);
                }
                Err(anyhow!("Error getting save ids".to_string()))
            }
        }
    } else {
        if cli_mode {
            print_debug(&format!("Status: {:?}", status));
            print_debug(&format!(
                "Error getting save ids: {:?}",
                response.text().await
            ));
        } else {
            debug!("Status: {:?}", status);
            debug!("Error getting save ids: {:?}", response.text().await);
        }
        Err(anyhow!("Error getting save ids".to_string()))
    }
}

pub async fn delete_a_save_from_database(
    access_token: &str,
    cli_mode: bool,
    save_id: u64,
    save_number: Option<usize>,
) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .delete(format!(
            "{}/rest/v1/user_data?id=eq.{}",
            SUPABASE_URL, save_id
        ))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let status = response.status();
    if status == StatusCode::NO_CONTENT {
        if cli_mode {
            if save_number.is_some() {
                print_info(&format!("👍 Cloud save {} deleted", save_number.unwrap()));
            } else {
                print_info("👍 Cloud save deleted");
            }
            Ok("👍 Cloud save deleted".to_string())
        } else if save_number.is_some() {
            info!("👍 Cloud save {} deleted", save_number.unwrap());
            Ok(format!("👍 Cloud save {} deleted", save_number.unwrap()).to_string())
        } else {
            info!("👍 Cloud save deleted");
            Ok("👍 Cloud save deleted".to_string())
        }
    } else {
        let body = response.json::<serde_json::Value>().await;
        if cli_mode {
            print_error("Error deleting cloud save");
            print_debug(&format!(
                "status code {}, response body: {:?}",
                status, body
            ));
        } else {
            error!("Error deleting cloud save");
            debug!("status code {}, response body: {:?}", status, body);
        }
        Err(anyhow!("Error deleting cloud save"))
    }
}

pub async fn get_user_id_from_database(access_token: &str, cli_mode: bool) -> Result<String> {
    let user_data_client = reqwest::Client::new();
    let user_data_response = user_data_client
        .get(format!("{}/auth/v1/user", SUPABASE_URL))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    let user_data_status = user_data_response.status();
    let user_data_body = user_data_response.json::<serde_json::Value>().await;
    if user_data_status != StatusCode::OK {
        if cli_mode {
            print_error("Error retrieving user data");
            print_debug(&format!(
                "status code {}, response body: {:?}",
                user_data_status, user_data_body
            ));
        } else {
            error!("Error retrieving user data");
            debug!(
                "status code {}, response body: {:?}",
                user_data_status, user_data_body
            );
        }
        return Err(anyhow!("Error retrieving user data"));
    }
    let user_data_body = user_data_body.unwrap();
    let user_id = user_data_body.get("id");
    if user_id.is_none() {
        if cli_mode {
            print_error("Error retrieving user data");
            print_debug(&format!(
                "status code {}, response body: {:?}, could not find id",
                user_data_status, user_data_body
            ));
        } else {
            error!("Error retrieving user data");
            debug!(
                "status code {}, response body: {:?}, could not find id",
                user_data_status, user_data_body
            );
        }
        return Err(anyhow!("Error retrieving user data"));
    }
    let user_id = user_id.unwrap().as_str();
    if cli_mode {
        print_debug(&format!("user_id: {:?}", user_id));
    } else {
        debug!("user_id: {:?}", user_id);
    }
    Ok(user_id.unwrap().to_string())
}

pub async fn login_for_user(
    email_id: &str,
    password: &str,
    cli_mode: bool,
) -> Result<(String, String, String), String> {
    let request_body = json!(
        {
            "email": email_id,
            "password": password
        }
    );
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/auth/v1/token?grant_type=password",
            SUPABASE_URL
        ))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .send()
        .await;
    if let Err(e) = response {
        if cli_mode {
            print_debug(&format!("Error logging in: {}", e));
            print_error("Error logging in, Something went wrong, please try again later");
        } else {
            debug!("Error logging in: {}", e);
            error!("Error logging in, Something went wrong, please try again later");
        }
        return Err("Error logging in, Something went wrong, please try again later".to_string());
    }
    let response = response.unwrap();
    let status = response.status();
    let body = response.json::<serde_json::Value>().await;
    if status == StatusCode::OK {
        match body {
            Ok(body) => {
                let access_token = body.get("access_token");
                let refresh_token = body.get("refresh_token");
                let access_token_result = match access_token {
                    Some(access_token) => {
                        let access_token = access_token.as_str().unwrap();
                        if cli_mode {
                            print_info("🚀 Login successful");
                            print_debug(&format!("Access token: {}", access_token));
                        } else {
                            info!("🚀 Login successful");
                            debug!("Access token: {}", access_token);
                        }
                        let user_id = get_user_id_from_database(access_token, cli_mode)
                            .await
                            .unwrap_or_else(|_| "Error getting user id".to_string());
                        Ok((access_token.to_string(), user_id))
                    }
                    None => {
                        if cli_mode {
                            print_error("Error logging in");
                            print_debug(&format!(
                                "status code {}, response body: {:?}, could not find access token",
                                status, body
                            ));
                        } else {
                            error!("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer");
                            debug!(
                                "status code {}, response body: {:?}, could not find access token",
                                status, body
                            );
                        }
                        Err("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer".to_string())
                    }
                };
                let refresh_token_result = match refresh_token {
                    Some(refresh_token) => {
                        let refresh_token = refresh_token.as_str().unwrap();
                        if cli_mode {
                            print_debug(&format!("Refresh token: {}", refresh_token));
                        } else {
                            debug!("Refresh token: {}", refresh_token);
                        }
                        Ok(refresh_token.to_string())
                    }
                    None => {
                        if cli_mode {
                            print_error("Error logging in");
                            print_debug(&format!(
                                "status code {}, response body: {:?}, could not find refresh token",
                                status, body
                            ));
                        } else {
                            error!("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer");
                            debug!(
                                "status code {}, response body: {:?}, could not find refresh token",
                                status, body
                            );
                        }
                        Err("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer".to_string())
                    }
                };

                if access_token_result.is_err() || refresh_token_result.is_err() {
                    Err("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer".to_string())
                } else {
                    let access_token_result = access_token_result.unwrap();
                    let refresh_token_result = refresh_token_result.unwrap();
                    Ok((
                        access_token_result.0,
                        access_token_result.1,
                        refresh_token_result,
                    ))
                }
            }
            Err(e) => Err(format!("Error logging in: {}", e)),
        }
    } else if status == StatusCode::TOO_MANY_REQUESTS {
        if cli_mode {
            print_error("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢");
            print_debug(&format!(
                "status code {}, response body: {:?}",
                status, body
            ));
        } else {
            error!("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢");
            debug!("status code {}, response body: {:?}", status, body);
        }
        Err("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢".to_string())
    } else {
        match body {
            Ok(body) => {
                let error_description = body.get("error_description");
                match error_description {
                    Some(error_description) => {
                        let error_description = error_description.to_string();
                        if cli_mode {
                            print_error(&error_description);
                            print_debug(&format!(
                                "status code {}, response body: {:?}",
                                status, body
                            ));
                        } else {
                            error!("{}", error_description);
                            debug!("status code {}, response body: {:?}", status, body);
                        }
                        Err(format!("Error logging in: {}", error_description))
                    }
                    None => {
                        if cli_mode {
                            print_error("Error logging in");
                            print_debug(&format!(
                                "status code {}, response body: {:?}",
                                status, body
                            ));
                        } else {
                            error!("Error logging in");
                            debug!("status code {}, response body: {:?}", status, body);
                        }
                        Err("Error logging in".to_string())
                    }
                }
            }
            Err(e) => {
                if cli_mode {
                    print_error(&format!("Error logging in: {}", e));
                } else {
                    error!("Error logging in: {}", e);
                }
                Err(format!("Error logging in: {}", e))
            }
        }
    }
}

async fn save_refresh_token_to_disk(
    refresh_token: &str,
    email_id: &str,
    encryption_key_from_arguments: Option<String>,
) -> Result<()> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let refresh_token_path = get_config_dir();
    if refresh_token_path.is_err() {
        return Err(anyhow!("Error getting config directory"));
    }
    let mut refresh_token_path = refresh_token_path.unwrap();
    refresh_token_path.push(REFRESH_TOKEN_FILE_NAME);
    if refresh_token_path.exists() {
        let delete_file_status = std::fs::remove_file(&refresh_token_path);
        if delete_file_status.is_err() {
            return Err(anyhow!("Error deleting refresh token file"));
        }
    }
    let encryption_key = get_user_encryption_key(encryption_key_from_arguments);
    if let Err(e) = encryption_key {
        return Err(anyhow!(e));
    }
    let encryption_key = encryption_key.unwrap();
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let encrypted_refresh_token = cipher.encrypt(&nonce, refresh_token.as_bytes());
    if encrypted_refresh_token.is_err() {
        return Err(anyhow!("Error encrypting refresh token"));
    }
    let encrypted_refresh_token = encrypted_refresh_token.unwrap();
    let nonce = nonce.to_vec();
    let nonce = base64_engine.encode(nonce);
    let encrypted_refresh_token = base64_engine.encode(encrypted_refresh_token);
    let encoded_email_id = base64_engine.encode(email_id.as_bytes());
    let refresh_token_data = format!(
        "{}{}{}{}{}",
        nonce,
        REFRESH_TOKEN_SEPARATOR,
        encrypted_refresh_token,
        REFRESH_TOKEN_SEPARATOR,
        encoded_email_id
    );
    let file_creation_status = std::fs::write(&refresh_token_path, refresh_token_data);
    if file_creation_status.is_err() {
        return Err(anyhow!("Error creating refresh token file"));
    }
    Ok(())
}

fn get_refresh_token_from_disk(
    encryption_key_from_arguments: Option<String>,
) -> Result<(String, String)> {
    let base64_engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let refresh_token_path = get_config_dir();
    if refresh_token_path.is_err() {
        return Err(anyhow!("Error getting config directory"));
    }
    let mut refresh_token_path = refresh_token_path.unwrap();
    refresh_token_path.push(REFRESH_TOKEN_FILE_NAME);
    let refresh_token_data = std::fs::read_to_string(&refresh_token_path);
    if refresh_token_data.is_err() {
        return Err(anyhow!("Error reading refresh token file"));
    }
    let refresh_token_data = refresh_token_data.unwrap();
    let refresh_token_data = refresh_token_data
        .split(REFRESH_TOKEN_SEPARATOR)
        .collect::<Vec<&str>>();
    if refresh_token_data.len() != 3 {
        return Err(anyhow!("Error reading refresh token file"));
    }
    let nonce = refresh_token_data[0];
    let nonce = base64_engine.decode(nonce);
    if nonce.is_err() {
        return Err(anyhow!("Error reading refresh token file"));
    }
    let nonce = nonce.unwrap();
    let nonce = GenericArray::from_slice(&nonce);
    let encrypted_refresh_token = refresh_token_data[1];
    let encrypted_refresh_token = base64_engine.decode(encrypted_refresh_token);
    if encrypted_refresh_token.is_err() {
        return Err(anyhow!("Error reading refresh token file"));
    }
    let encrypted_refresh_token = encrypted_refresh_token.unwrap();
    let email_id = refresh_token_data[2];
    let email_id = base64_engine.decode(email_id);
    if email_id.is_err() {
        return Err(anyhow!("Error reading refresh token file"));
    }
    let email_id = email_id.unwrap();
    let email_id = String::from_utf8(email_id);
    if email_id.is_err() {
        return Err(anyhow!("Error reading refresh token file"));
    }
    let email_id = email_id.unwrap();
    let encryption_key = get_user_encryption_key(encryption_key_from_arguments);
    if let Err(e) = encryption_key {
        return Err(anyhow!(e));
    }
    let encryption_key = encryption_key.unwrap();
    let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
    let cipher = Aes256Gcm::new(key);
    let decrypted_refresh_token = cipher.decrypt(nonce, encrypted_refresh_token.as_slice());
    if decrypted_refresh_token.is_err() {
        return Err(anyhow!("Error decrypting refresh token"));
    }
    let decrypted_refresh_token = decrypted_refresh_token.unwrap();
    let decrypted_refresh_token = String::from_utf8(decrypted_refresh_token);
    if decrypted_refresh_token.is_err() {
        return Err(anyhow!(
            "Error converting decrypted refresh token to string"
        ));
    }
    let decrypted_refresh_token = decrypted_refresh_token.unwrap();
    Ok((decrypted_refresh_token, email_id))
}

async fn delete_refresh_token_from_disk() -> Result<()> {
    let refresh_token_path = get_config_dir();
    if refresh_token_path.is_err() {
        return Err(anyhow!("Error getting config directory"));
    }
    let mut refresh_token_path = refresh_token_path.unwrap();
    refresh_token_path.push(REFRESH_TOKEN_FILE_NAME);
    if !refresh_token_path.exists() {
        return Ok(());
    }
    let delete_file_status = std::fs::remove_file(&refresh_token_path);
    if delete_file_status.is_err() {
        return Err(anyhow!("Error deleting refresh token file"));
    }
    Ok(())
}

async fn refresh_access_token(refresh_token: &str) -> Result<(String, String, String), String> {
    let request_body = json!(
        {
            "grant_type": "refresh_token",
            "refresh_token": refresh_token
        }
    );
    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/auth/v1/token?grant_type=refresh_token",
            SUPABASE_URL
        ))
        .header("apikey", SUPABASE_ANON_KEY)
        .header("Content-Type", "application/json")
        .body(request_body.to_string())
        .send()
        .await;
    if let Err(e) = response {
        debug!("Error logging in: {}", e);
        error!("Error logging in, Something went wrong, please try again later");
        return Err("Error logging in, Something went wrong, please try again later".to_string());
    }
    let response = response.unwrap();
    let status = response.status();
    let body = response.json::<serde_json::Value>().await;
    if status == StatusCode::OK {
        match body {
            Ok(body) => {
                let access_token = body.get("access_token");
                let refresh_token = body.get("refresh_token");
                let access_token_result = match access_token {
                    Some(access_token) => {
                        let access_token = access_token.as_str().unwrap();
                        info!("🚀 Login successful");
                        debug!("Access token: {}", access_token);
                        let user_id = get_user_id_from_database(access_token, false)
                            .await
                            .unwrap_or_else(|_| "Error getting user id".to_string());
                        Ok((access_token.to_string(), user_id))
                    }
                    None => {
                        error!("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer");
                        debug!(
                            "status code {}, response body: {:?}, could not find access token",
                            status, body
                        );
                        Err("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer".to_string())
                    }
                };
                let refresh_token_result = match refresh_token {
                    Some(refresh_token) => {
                        let refresh_token = refresh_token.as_str().unwrap();
                        debug!("Refresh token: {}", refresh_token);
                        Ok(refresh_token.to_string())
                    }
                    None => {
                        error!("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer");
                        debug!(
                            "status code {}, response body: {:?}, could not find refresh token",
                            status, body
                        );
                        Err("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer".to_string())
                    }
                };

                if access_token_result.is_err() || refresh_token_result.is_err() {
                    Err("Error logging in, If this is your first login attempt after signup please login again, if it is not please contact the developer".to_string())
                } else {
                    let access_token_result = access_token_result.unwrap();
                    let refresh_token_result = refresh_token_result.unwrap();
                    Ok((
                        access_token_result.0,
                        access_token_result.1,
                        refresh_token_result,
                    ))
                }
            }
            Err(e) => Err(format!("Error logging in: {}", e)),
        }
    } else if status == StatusCode::TOO_MANY_REQUESTS {
        error!("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢");
        debug!("status code {}, response body: {:?}", status, body);
        Err("Too many requests, please try again later. Due to the free nature of supabase i am limited to only 4 signup requests per hour. Sorry! 😢".to_string())
    } else {
        match body {
            Ok(body) => {
                let error_description = body.get("error_description");
                match error_description {
                    Some(error_description) => {
                        let error_description = error_description.to_string();
                        error!("{}", error_description);
                        debug!("status code {}, response body: {:?}", status, body);
                        Err(format!("Error logging in: {}", error_description))
                    }
                    None => {
                        error!("Error logging in");
                        debug!("status code {}, response body: {:?}", status, body);
                        Err("Error logging in".to_string())
                    }
                }
            }
            Err(e) => {
                error!("Error logging in: {}", e);
                Err(format!("Error logging in: {}", e))
            }
        }
    }
}

async fn test_refresh_token_on_disk(
    encryption_key_from_arguments: Option<String>,
) -> Result<UserLoginData> {
    let (refresh_token, email_id) =
        get_refresh_token_from_disk(encryption_key_from_arguments.clone())?;
    debug!("refresh_token: {:?}", refresh_token);
    let status = refresh_access_token(&refresh_token).await;
    if status.is_err() {
        return Err(anyhow!(status.err().unwrap()));
    }
    let status = status.unwrap();
    let access_token = status.0;
    let user_id = status.1;
    let refresh_token = status.2;
    let save_status =
        save_refresh_token_to_disk(&refresh_token, &email_id, encryption_key_from_arguments).await;
    if save_status.is_err() {
        error!("Error saving refresh token to disk");
    }
    let user_data = UserLoginData {
        auth_token: Some(access_token),
        email_id: Some(email_id),
        refresh_token: Some(refresh_token),
        user_id: Some(user_id),
    };
    Ok(user_data)
}
