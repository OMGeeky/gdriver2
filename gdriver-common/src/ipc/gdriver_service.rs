use crate::prelude::*;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::drive_structure::drive_id::DriveId;

#[tarpc::service]
pub trait GDriverService {
    async fn get_file_by_path(path: PathBuf) -> StdResult<DriveId, GetFileByPathError>;
    async fn write_local_change(id: DriveId) -> StdResult<(), WriteLocalChangeError>;
    async fn get_metadata_for_file(id: DriveId) -> StdResult<(), GetMetadataError>;
    async fn download_content_for_file(id: DriveId) -> StdResult<(), GetContentError>;
    async fn list_files_in_directory(id: DriveId) -> StdResult<(), GetFileListError>;
    async fn mark_file_as_deleted(id: DriveId) -> StdResult<(), MarkFileAsDeletedError>;
    async fn mark_file_for_keeping_local(
        id: DriveId,
    ) -> StdResult<(), MarkFileForKeepingLocalError>;
    async fn unmark_file_for_keeping_local(
        id: DriveId,
    ) -> StdResult<(), UnmarkFileForKeepingLocalError>;
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
#[derive(Debug, Serialize, Deserialize)]
pub enum WriteLocalChangeError {
    RemoteChanged,
    UnknownId,
    NotAllowed,
    Other,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum GetMetadataError {}
#[derive(Debug, Serialize, Deserialize)]
pub enum GetContentError {}
//#[derive(Debug, Serialize, Deserialize)]
//pub enum GetContentError {}
#[derive(Debug, Serialize, Deserialize)]
pub enum GetFileListError {}
#[derive(Debug, Serialize, Deserialize)]
pub enum MarkFileAsDeletedError {}
#[derive(Debug, Serialize, Deserialize)]
pub enum MarkFileForKeepingLocalError {}
#[derive(Debug, Serialize, Deserialize)]
pub enum UnmarkFileForKeepingLocalError {}
