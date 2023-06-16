pub mod data_handler;
pub mod handler;
pub mod logger;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,   // Launch to initialize the application
    GetCloudData, // Launch to get cloud data (Not implemented yet)
    Reset,
    SaveLocalData,
    LoadSave,
    DeleteSave,
    ResetVisibleBoardsandCards,
    AutoSave,
    LoadPreview,
}
