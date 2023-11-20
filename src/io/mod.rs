pub mod data_handler;
pub mod io_handler;
pub mod logger;

#[derive(Debug, Clone)]
pub enum IoEvent {
    AutoSave,
    DeleteCloudSave,
    DeleteLocalSave,
    GetCloudData,
    Initialize,
    LoadCloudPreview,
    LoadLocalPreview,
    LoadSaveCloud,
    LoadSaveLocal,
    Login(String, String),
    Logout,
    ResetPassword(String, String, String),
    ResetVisibleBoardsandCards,
    SaveLocalData,
    SendResetPasswordEmail(String),
    SignUp(String, String, String),
    SyncLocalData,
}
