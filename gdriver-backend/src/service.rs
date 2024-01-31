use chrono::Duration;
use gdriver_common::{
    drive_structure::drive_id::{DriveId, ROOT_ID},
    ipc::gdriver_service::*,
};
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::Mutex;

use crate::drive::Drive;

use super::*;
#[derive(Clone)]
struct GdriverServer {
    socket_address: SocketAddr,
    drive: Arc<Mutex<Drive>>,
}
impl GDriverService for GdriverServer {
    async fn do_something2(
        self,
        _: ::tarpc::context::Context,
        req: BackendActionRequest,
    ) -> std::result::Result<String, BackendActionError> {
        println!("You are connected from {}", self.socket_address);

        match req {
            BackendActionRequest::ShutdownGracefully => {
                println!("Shutdown request received, but I dont want to.");
                Err(BackendActionError::CouldNotComplete)
                //Ok(String::from("OK. Shutting down"))
            }
            BackendActionRequest::UpdateChanges => {
                println!("UpdateChanges request received");
                let drive = &self.drive;
                print_sample_tracking_state(drive).await;

                Ok(String::from("OK"))
            }
            BackendActionRequest::Ping => {
                println!("Ping request received");
                Ok(String::from("Pong"))
            }
            BackendActionRequest::RunLong => {
                println!("RunLong request received");
                long_running_task(&self.drive).await;
                Ok(String::from("OK"))
            }
            BackendActionRequest::StartLong => {
                println!("StartLong request received");
                tokio::spawn(async move { long_running_task(&self.drive).await });
                Ok(String::from("OK"))
            }
        }
    }

    async fn get_file_by_path(
        self,
        context: ::tarpc::context::Context,
        path: PathBuf,
    ) -> StdResult<DriveId, GetFileByPathError> {
        todo!()
    }

    #[doc = " Returns true if the file was had remote changes and was updadet"]
    async fn update_changes_for_file(
        self,
        context: ::tarpc::context::Context,
        id: DriveId,
    ) -> StdResult<bool, UpdateChangesError> {
        todo!()
    }

    async fn update_changes(
        self,
        context: ::tarpc::context::Context,
    ) -> StdResult<(), UpdateChangesError> {
        todo!()
    }
}
async fn long_running_task(drive: &Arc<Mutex<Drive>>) {
    thread::sleep(Duration::seconds(10).to_std().unwrap());
    print_sample_tracking_state(drive).await;
}
async fn print_sample_tracking_state(drive: &Arc<Mutex<Drive>>) {
    let lock = drive.lock();
    let drive = lock.await;
    let state = drive.get_file_tracking_state(&ROOT_ID);
    dbg!(state);
}
pub async fn start() -> Result<()> {
    println!("Hello, world!");
    let config = &CONFIGURATION;
    println!("Config: {:?}", **config);

    let drive = Drive::new();
    let m = Arc::new(Mutex::new(drive));

    let server_addr = (config.ip, config.port);
    let mut listener = tarpc::serde_transport::tcp::listen(&server_addr, Json::default).await?;
    listener.config_mut().max_frame_length(usize::MAX);

    println!("Listening");
    listener
        // Ignore accept errors.
        .filter_map(|r| future::ready(r.ok()))
        .map(server::BaseChannel::with_defaults)
        // // Limit channels to 1 per IP.
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
