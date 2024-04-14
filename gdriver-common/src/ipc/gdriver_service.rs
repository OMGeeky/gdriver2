use crate::drive_structure::drive_id::DriveId;
use crate::drive_structure::meta::FileKind;
use crate::ipc::gdriver_settings::GDriverSettings;
use crate::prelude::*;
use errors::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::path::PathBuf;

#[tarpc::service]
pub trait GDriverService {
    async fn get_file_by_name(
        name: OsString,
        parent: DriveId,
    ) -> StdResult<DriveId, GetFileByPathError>;
    async fn get_file_by_path(path: PathBuf) -> StdResult<DriveId, GetFileByPathError>;
    async fn write_local_change(id: DriveId) -> StdResult<(), WriteLocalChangeError>;
    async fn get_metadata_for_file(id: DriveId) -> StdResult<(), GetMetadataError>;
    async fn download_content_for_file(id: DriveId) -> StdResult<(), GetContentError>;
    async fn list_files_in_directory(
        id: DriveId,
    ) -> StdResult<Vec<ReadDirResult>, GetFileListError>;
    async fn list_files_in_directory_with_offset(
        id: DriveId,
        offset: u64,
    ) -> StdResult<Vec<ReadDirResult>, GetFileListError>;
    async fn mark_file_as_deleted(id: DriveId) -> StdResult<(), MarkFileAsDeletedError>;
    async fn mark_file_for_keeping_local(
        id: DriveId,
    ) -> StdResult<(), MarkFileForKeepingLocalError>;
    async fn unmark_file_for_keeping_local(
        id: DriveId,
    ) -> StdResult<(), UnmarkFileForKeepingLocalError>;
    /// Returns true if the file was had remote changes and was updated
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

lazy_static! {
    pub static ref SETTINGS: GDriverSettings = GDriverSettings::default();
}

pub mod errors {
    use super::*;
    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GDriverServiceError {
        #[error("Error getting the settings: {0}")]
        GetSettings(#[from] GetSettingsError),
        #[error("Backend Action had an Error: {0}")]
        BackendAction(#[from] BackendActionError),
        #[error("Could not get File by Path: {0}")]
        GetFileByPath(#[from] GetFileByPathError),
        #[error("Could not update changes: {0}")]
        UpdateChanges(#[from] UpdateChangesError),
        #[error("Could not write local change: {0}")]
        WriteLocalChange(#[from] WriteLocalChangeError),
        #[error("Could not get metadata: {0}")]
        GetMetadata(#[from] GetMetadataError),
        #[error("Could not get content: {0}")]
        GetContent(#[from] GetContentError),

        #[error("Could not get file list: {0}")]
        GetFileList(#[from] GetFileListError),
        #[error("Could not mark file as deleted: {0}")]
        MarkFileAsDeleted(#[from] MarkFileAsDeletedError),
        #[error("Could not mark file for keeping: {0}")]
        MarkFileForKeepingLocal(#[from] MarkFileForKeepingLocalError),
        #[error("Could not unmark file for keeping: {0}")]
        UnmarkFileForKeepingLocal(#[from] UnmarkFileForKeepingLocalError),
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetSettingsError {
        #[error("Unknown Error getting the settings")]
        Unknown,
    }
    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum BackendActionError {
        #[error("Unknown Error")]
        Unknown,
        #[error("Could not complete Error")]
        CouldNotComplete,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetFileByPathError {
        #[error("Other")]
        Other,
        #[error("The Specified name is invalid")]
        InvalidName,
        #[error("Could not find name specified")]
        NotFound,
        #[error("Could not update drive info")]
        Update(String),
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum UpdateChangesError {
        #[error("Other")]
        Other,
        #[error("Remote error")]
        Remote,
        #[error("Already running")]
        Running,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum WriteLocalChangeError {
        #[error("Remote has changed")]
        RemoteChanged,
        #[error("Unknown Id")]
        UnknownId,
        #[error("Not Allowed")]
        NotAllowed,
        #[error("Other")]
        Other,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetMetadataError {
        #[error("Other")]
        Other,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetContentError {
        #[error("Other")]
        Other,
    }

    //#[derive(Debug, Serialize, Deserialize)]
    //pub enum GetContentError {}
    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetFileListError {
        #[error("Other")]
        Other,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum MarkFileAsDeletedError {
        #[error("Other")]
        Other,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum MarkFileForKeepingLocalError {
        #[error("Other")]
        Other,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum UnmarkFileForKeepingLocalError {
        #[error("Other")]
        Other,
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct ReadDirResult {
    pub id: DriveId,
    pub kind: FileKind,
    pub name: String,
}
