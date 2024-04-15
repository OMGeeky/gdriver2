use fuser::{MountOption, Session, SessionUnmounter};
use std::{error::Error, net::IpAddr, result::Result as StdResult};
use tokio::sync::mpsc::{channel, Sender};

use crate::filesystem::{Filesystem, ShutdownRequest};
use gdriver_common::{ipc::sample::*, prelude::*};
use tarpc::context::Context;
use tarpc::{client, tokio_serde::formats::Json};
use tokio::task::JoinHandle;

type Result<T> = StdResult<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    gdriver_common::tracing_setup::init_tracing();
    check_setup()?;
    // service::start().await?;
    let mount_options = &[MountOption::RW];
    let (tx, rx) = channel(1);
    let gdriver_client = service::create_client(CONFIGURATION.ip, CONFIGURATION.port).await?;
    gdriver_client
        .set_offline_mode(Context::current(), true) //TODO make this configurable
        .await??;
    let f = Filesystem::new(gdriver_client, rx);
    mount(f, &"/var/tmp/gdriver2_mount", mount_options, tx)
        .await?
        .await?;
    Ok(())
}

fn check_setup() -> Result<()> {
    // let _ = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
    //     .map_err(|_| "GOOGLE_APPLICATION_CREDENTIALS env var not set")?;
    let _ = &*filesystem::GDRIVER_GROUP_ID;
    let _ = &*filesystem::USER_ID;

    Ok(())
}

pub mod prelude;
mod sample;

mod filesystem;
mod service;

async fn mount(
    fs: Filesystem,
    mountpoint: &str,
    options: &[MountOption],
    sender: Sender<ShutdownRequest>,
) -> Result<JoinHandle<()>> {
    let mut session = Session::new(fs, mountpoint.as_ref(), options)?;
    let session_ender = session.unmount_callable();
    let end_program_signal_handle = tokio::spawn(async move {
        let _ = end_program_signal_awaiter(sender, session_ender).await;
    });
    debug!("Mounting fuse filesystem");
    tokio::task::spawn_blocking(move || {
        let _ = session.run();
    })
    .await?;
    debug!("Stopped with mounting");
    // Ok(session_ender)
    Ok(end_program_signal_handle)
}

async fn end_program_signal_awaiter(
    sender: Sender<ShutdownRequest>,
    mut session_unmounter: SessionUnmounter,
) -> Result<()> {
    info!("Waiting for Ctrl-C");
    println!("Waiting for Ctrl-C");
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl_c event");
    println!(); //to not have ^C on the same line as the next log if it is directly in a console
    info!("got signal to end program");
    sender.send(ShutdownRequest::Gracefully).await?;
    info!("sent stop command to file uploader");
    info!("unmounting...");
    session_unmounter.unmount()?;
    info!("unmounted");
    Ok(())
}
