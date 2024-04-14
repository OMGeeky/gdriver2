use crate::prelude::*;
use futures::{future, prelude::*};
use std::net::SocketAddr;
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};

mod drive;
mod prelude;
mod sample;
mod service;

pub(crate) async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
#[tokio::main]
async fn main() -> Result<()> {
    gdriver_common::tracing_setup::init_tracing();
    //   sample::main().await?;
    service::start().await?;
    Ok(())
}
