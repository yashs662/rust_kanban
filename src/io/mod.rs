pub mod handler;
// For this dummy application we only need two IO event
#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize,      // Launch to initialize the application
    GetLocalData,  // Launch to get local data
    GetCloudData,  // Launch to get cloud data
}
