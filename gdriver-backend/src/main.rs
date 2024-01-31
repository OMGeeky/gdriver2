use futures::{future, prelude::*};
use std::net::SocketAddr;
use tarpc::{
    context,
    server::{self, incoming::Incoming, Channel},
    tokio_serde::formats::Json,
};
mod prelude;
use crate::prelude::*;
pub(crate) use gdriver_common::prelude::*;
mod drive;
mod sample;
mod service;

pub(crate) async fn spawn(fut: impl Future<Output = ()> + Send + 'static) {
    tokio::spawn(fut);
}
#[tokio::main]
async fn main() -> Result<()> {
    //   sample::main().await?;
    service::start().await?;
    Ok(())
}
