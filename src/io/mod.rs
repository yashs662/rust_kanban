pub mod data_handler;
pub mod io_handler;
pub mod logger;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,
    SaveLocalData,
    LoadSaveLocal,
    LoadSaveCloud,
    DeleteLocalSave,
    ResetVisibleBoardsandCards,
    AutoSave,
    LoadLocalPreview,
    Login(String, String),
    SignUp(String, String, String),
    SendResetPasswordEmail(String),
    ResetPassword(String, String, String),
    SyncLocalData,
    DeleteCloudSave,
    GetCloudData,
    LoadCloudPreview,
    Logout,
}
