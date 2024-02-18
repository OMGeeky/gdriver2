use crate::prelude::*;
use std::ffi::OsString;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::drive_structure::drive_id::DriveId;

#[tarpc::service]
pub trait GDriverService {
    async fn get_settings() -> StdResult<GDriverSettings, GetSettingsError>;
    async fn get_file_by_name(
        name: OsString,
        parent: DriveId,
    ) -> StdResult<DriveId, GetFileByPathError>;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GDriverSettings {
    metadata_path: PathBuf,
    cache_path: PathBuf,
    downloaded_path: PathBuf,
}

impl Default for GDriverSettings {
    fn default() -> Self {
        let p = directories::ProjectDirs::from("com", "OMGeeky", "gdriver2").expect(
            "Getting the Project dir needs to work (on all platforms) otherwise nothing will work as expected. \
            This is where all files will be stored, so there is not much use for this app without it.",
        );
        Self {
            metadata_path: p.data_dir().join("meta"),
            downloaded_path: p.data_dir().join("downloads"),
            cache_path: p.cache_dir().to_path_buf(),
        }
    }
}

use errors::*;
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
    pub enum GetFileByPathError {}

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum UpdateChangesError {}

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
    pub enum GetMetadataError {}

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetContentError {}

    //#[derive(Debug, Serialize, Deserialize)]
    //pub enum GetContentError {}
    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum GetFileListError {}

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum MarkFileAsDeletedError {}

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum MarkFileForKeepingLocalError {}

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum UnmarkFileForKeepingLocalError {}
}
