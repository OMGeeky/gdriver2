use crate::prelude::*;
use futures::{future, prelude::*};
use gdriver_common::drive_structure::meta;
use gdriver_common::ipc::gdriver_service::SETTINGS;
use std::net::SocketAddr;
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

mod drive;
mod path_resolver;
mod prelude;
mod sample;
mod service;

pub(crate) async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
#[tokio::main]
async fn main() -> Result<()> {
    gdriver_common::tracing_setup::init_tracing();
    SETTINGS.initialize_dirs()?;
    let root_meta_file = SETTINGS.get_metadata_file_path(&ROOT_ID);
    let root_meta = meta::Metadata::root();
    meta::write_metadata_file_to_path(&root_meta_file, &root_meta)?;

    //   sample::main().await?;
    service::start().await?;
    Ok(())
}
