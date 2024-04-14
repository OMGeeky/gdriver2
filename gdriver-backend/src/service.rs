use super::*;
use crate::drive::Drive;
use chrono::Duration;
use gdriver_common::{
    drive_structure::drive_id::{DriveId, ROOT_ID},
    ipc::gdriver_service::{errors::*, *},
};
use std::ffi::OsString;
use std::{path::PathBuf, sync::Arc, thread};
use tarpc::context::Context;
use tokio::sync::Mutex;

#[derive(Clone)]
struct GdriverServer {
    socket_address: SocketAddr,
    drive: Arc<Mutex<Drive>>,
}
impl GDriverService for GdriverServer {
    async fn set_offline_mode(
        self,
        _context: Context,
        offline_mode: bool,
    ) -> StdResult<(), GDriverServiceError> {
        self.drive.lock().await.set_offline_mode(offline_mode);
        Ok(())
    }

    #[instrument(skip(self, _context))]
    async fn get_file_by_name(
        self,
        _context: Context,
        name: OsString,
        parent: DriveId,
    ) -> StdResult<DriveId, GetFileByPathError> {
        let drive = self.drive.lock().await;
        let name = name.to_str().ok_or(GetFileByPathError::InvalidName)?;
        info!("Getting file with name '{}' under parent {}", name, parent);
        let x = drive
            .path_resolver
            .get_id_from_parent_and_name(name, &parent);
        match x {
            None => {
                info!("Did not find {name}");
                Err(GetFileByPathError::NotFound)
            }
            Some(id) => Ok(id),
        }
    }

    async fn get_file_by_path(
        self,
        context: Context,
        path: PathBuf,
    ) -> StdResult<DriveId, GetFileByPathError> {
        let mut drive_lock = self.drive.lock().await;
        let x = drive_lock.path_resolver.get_id_from_path(&path).await?;
        match x {
            None => Err(GetFileByPathError::NotFound),
            Some(id) => Ok(id),
        }
    }

    #[instrument(skip(self, context))]
    async fn write_local_change(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), WriteLocalChangeError> {
        error!("Not implemented");
        Err(WriteLocalChangeError::Other)
    }

    async fn get_metadata_for_file(
        self,
        _context: Context,
        id: DriveId,
    ) -> StdResult<(), GetMetadataError> {
        info!("Getting metadata for {id}");
        let meta_path = SETTINGS.get_metadata_file_path(&id);
        // let meta_exists =
        if meta_path.exists() {
            return Ok(());
        }
        info!("Meta was not downloaded. Getting from api");
        let drive = self.drive.lock().await;
        drive
            .download_meta_for_file(&id)
            .await
            .map_err(|_| GetMetadataError::DownloadError)?;
        Ok(())
    }

    #[instrument(skip(self, context))]
    async fn download_content_for_file(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), GetContentError> {
        error!("Not implemented");
        Err(GetContentError::Other)
    }

    #[instrument(skip(self, context))]
    async fn list_files_in_directory(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<Vec<ReadDirResult>, GetFileListError> {
        self.list_files_in_directory_with_offset(context, id, 0)
            .await
    }
    #[instrument(skip(self, _context))]
    async fn list_files_in_directory_with_offset(
        self,
        _context: Context,
        id: DriveId,
        offset: usize,
    ) -> StdResult<Vec<ReadDirResult>, GetFileListError> {
        let drive = self.drive.lock().await;
        info!("Listing files in dir");
        let children = drive
            .path_resolver
            .get_children(&id)
            .map_err(|_| GetFileListError::NotFound)?
            .clone();
        Ok(children.into_iter().skip(offset).collect())
    }

    #[instrument(skip(self, context))]
    async fn mark_file_as_deleted(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), MarkFileAsDeletedError> {
        error!("Not implemented");
        Err(MarkFileAsDeletedError::Other)
    }

    #[instrument(skip(self, context))]
    async fn mark_file_for_keeping_local(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), MarkFileForKeepingLocalError> {
        error!("Not implemented");
        Err(MarkFileForKeepingLocalError::Other)
    }

    #[instrument(skip(self, context))]
    async fn unmark_file_for_keeping_local(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), UnmarkFileForKeepingLocalError> {
        error!("Not implemented");
        Err(UnmarkFileForKeepingLocalError::Other)
    }

    #[doc = " Returns true if the file was had remote changes and was updated"]
    #[instrument(skip(self, context))]
    async fn update_changes_for_file(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<bool, UpdateChangesError> {
        error!("Not implemented");
        Err(UpdateChangesError::Other)
    }

    async fn update_changes(self, _context: Context) -> StdResult<(), UpdateChangesError> {
        let drive = self.drive.try_lock();
        match drive {
            Ok(mut drive) => {
                drive.update().await.map_err(|e| {
                    info!("Error while updating: {e}");
                    dbg!(e);
                    UpdateChangesError::Remote
                })?;
                Ok(())
            }
            Err(_) => {
                info!("Drive is already updating");
                Err(UpdateChangesError::Running)
            }
        }
    }

    #[instrument(skip(self))]
    async fn do_something2(
        self,
        _: Context,
        req: BackendActionRequest,
    ) -> std::result::Result<String, BackendActionError> {
        info!("You are connected from {}", self.socket_address);

        match req {
            BackendActionRequest::ShutdownGracefully => {
                info!("Shutdown request received, but I dont want to.");
                Err(BackendActionError::CouldNotComplete)
                //Ok(String::from("OK. Shutting down"))
            }
            BackendActionRequest::UpdateChanges => {
                info!("UpdateChanges request received");
                let drive = &self.drive;
                print_sample_tracking_state(drive).await;

                Ok(String::from("OK"))
            }
            BackendActionRequest::Ping => {
                info!("Ping request received");
                Ok(String::from("Pong"))
            }
            BackendActionRequest::RunLong => {
                info!("RunLong request received");
                long_running_task(&self.drive).await;
                Ok(String::from("OK"))
            }
            BackendActionRequest::StartLong => {
                info!("StartLong request received");
                tokio::spawn(async move { long_running_task(&self.drive).await });
                Ok(String::from("OK"))
            }
        }
    }
}
async fn long_running_task(drive: &Arc<Mutex<Drive>>) {
    thread::sleep(Duration::seconds(10).to_std().unwrap());
    print_sample_tracking_state(drive).await;
}
async fn print_sample_tracking_state(drive: &Arc<Mutex<Drive>>) {
    let drive_lock = drive.lock().await;
    let state = drive_lock.get_file_tracking_state(&ROOT_ID);
    dbg!(state);
}
pub async fn start() -> Result<()> {
    info!("Hello, world!");
    let config = &CONFIGURATION;
    info!("Config: {:?}", **config);

    let mut drive = Drive::new().await?;
    match drive.ping().await {
        Ok(_) => {
            info!("Can reach google drive api.");
        }
        Err(e) => {
            error!("Cannot reach google drive api.");
            return Err(e);
        }
    }
    drive.get_all_file_metas().await?;
    let drive = Arc::new(Mutex::new(drive));

    let server_addr = (config.ip, config.port);
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);

    info!("Listening");
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // Limit channels to 1 per IP.
        .max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
        // serve is generated by the service attribute. It takes as input any type implementing
        // the generated World trait.
        .map(|channel| {
            let c = channel.transport().peer_addr().unwrap();
            let server = GdriverServer {
                socket_address: c,
                drive: drive.clone(),
            };
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
    Ok(())
}
