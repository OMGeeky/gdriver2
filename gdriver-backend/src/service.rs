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
    // async fn get_settings(self, context: Context) -> StdResult<GDriverSettings, GetSettingsError> {
    //     todo!()
    // }

    async fn get_file_by_name(self, context: Context, name: OsString, parent: DriveId) -> StdResult<DriveId, GetFileByPathError> {
        todo!()
    }

    async fn get_file_by_path(
        self,
        context: Context,
        path: PathBuf,
    ) -> StdResult<DriveId, GetFileByPathError> {
        todo!()
    }

    async fn write_local_change(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), WriteLocalChangeError> {
        todo!()
    }

    async fn get_metadata_for_file(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), GetMetadataError> {
        todo!()
    }

    async fn download_content_for_file(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), GetContentError> {
        todo!()
    }

    async fn list_files_in_directory(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<Vec<ReadDirResult>, GetFileListError> {
        Err(GetFileListError::Other)
    }

    async fn list_files_in_directory_with_offset(
        self,
        context: Context,
        id: DriveId,
        offset: u64,
    ) -> StdResult<Vec<ReadDirResult>, GetFileListError> {
        Err(GetFileListError::Other)
    }

    async fn mark_file_as_deleted(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), MarkFileAsDeletedError> {
        todo!()
    }

    async fn mark_file_for_keeping_local(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), MarkFileForKeepingLocalError> {
        todo!()
    }

    async fn unmark_file_for_keeping_local(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<(), UnmarkFileForKeepingLocalError> {
        todo!()
    }

    #[doc = " Returns true if the file was had remote changes and was updadet"]
    async fn update_changes_for_file(
        self,
        context: Context,
        id: DriveId,
    ) -> StdResult<bool, UpdateChangesError> {
        todo!()
    }

    async fn update_changes(self, context: Context) -> StdResult<(), UpdateChangesError> {
        let drive = self.drive.try_lock();
        match drive {
            Ok(mut drive) => {
                drive.update().await.map_err(|e| {
                    info!("Error while updating: {e}");
                    dbg!(e);
                    UpdateChangesError::Remote
                })?;
            }
            Err(_) => {
                return Err(UpdateChangesError::Running);
            }
        }
        Ok(())
    }

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

    let drive = Drive::new();
    let m = Arc::new(Mutex::new(drive));

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
                drive: m.clone(),
            };
            channel.execute(server.serve()).for_each(spawn)
        })
        // Max 10 channels.
        .buffer_unordered(10)
        .for_each(|_| async {})
        .await;
    Ok(())
}
