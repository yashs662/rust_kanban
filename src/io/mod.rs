pub mod data_handler;
pub mod handler;
pub mod logger;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize, // Launch to initialize the application
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
