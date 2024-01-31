pub use crate::config::Configuration;
pub use crate::config::CONFIGURATION;
pub use crate::ipc;
pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) use std::result::Result as StdResult;
