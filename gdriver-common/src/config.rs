use super::*;
use crate::prelude::*;
use confique::{Config, Partial};
use std::net::{IpAddr, Ipv6Addr};
const IP_DEFAULT: IpAddr = IpAddr::V6(Ipv6Addr::LOCALHOST);
#[derive(Debug, Serialize, Deserialize, Config, Clone)]
pub struct Configuration {
    #[config(default = 33333)]
    pub port: u16,
    //    #[config(default = Test)]
    pub ip: std::net::IpAddr,
}
pub fn load_config() -> Result<Configuration> {
    Ok(add_default_locations(Config::builder()).load()?)
}
pub fn load_config_with_path(path: &Path) -> Result<Configuration> {
    Ok(add_default_locations(Config::builder().file(path)).load()?)
}
fn add_default_locations(
    builder: confique::Builder<Configuration>,
) -> confique::Builder<Configuration> {
    type P = <Configuration as Config>::Partial;
    let prebuilt = P {
        ip: Some(IP_DEFAULT),
        ..P::empty()
    };
    builder.env().file("config.toml").preloaded(prebuilt)
}

use lazy_static::lazy_static;
lazy_static! {
    pub static ref CONFIGURATION: Configuration = load_config().unwrap();
}
