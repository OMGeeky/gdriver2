use crate::prelude::*;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::drive_structure::drive_id::DriveId;

#[tarpc::service]
pub trait GDriverService {
    async fn get_file_by_path(path: PathBuf) -> StdResult<DriveId, GetFileByPathError>;
    /// Returns true if the file was had remote changes and was updadet
    async fn update_changes_for_file(id: DriveId) -> StdResult<bool, UpdateChangesError>;
    async fn update_changes() -> StdResult<(), UpdateChangesError>;
    async fn do_something2(req: BackendActionRequest) -> StdResult<String, BackendActionError>;
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BackendActionRequest {
    ShutdownGracefully,
    UpdateChanges,
    Ping,
    RunLong,
    StartLong,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum BackendActionError {
    Unknown,
    CouldNotComplete,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GetFileByPathError {}
#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateChangesError {}
