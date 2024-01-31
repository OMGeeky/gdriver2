use std::{error::Error, net::IpAddr, result::Result as StdResult};

use gdriver_common::{ipc::sample::*, prelude::*};
use tarpc::{client, tokio_serde::formats::Json};

type Result<T> = StdResult<T, Box<dyn Error>>;

#[tokio::main]
async fn main() -> Result<()> {
    service::start().await?;
    Ok(())
}
mod sample;

mod service;
