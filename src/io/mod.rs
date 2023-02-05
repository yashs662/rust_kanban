pub mod handler;
pub mod data_handler;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,      // Launch to initialize the application
    GetCloudData,  // Launch to get cloud data
    Reset,
    SaveLocalData,
    LoadSave,
    DeleteSave,
    GoRight,
    GoLeft,
    GoUp,
    GoDown,
    RefreshVisibleBoardsandCards,
    AutoSave
}
