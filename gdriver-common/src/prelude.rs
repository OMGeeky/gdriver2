pub use crate::config::Configuration;
pub use crate::config::CONFIGURATION;
pub use crate::drive_structure::drive_id::{DriveId, ROOT_ID};
pub use crate::ipc;
pub mod result {
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    pub use std::result::Result as StdResult;
}
pub(crate) use result::*;
pub use tracing::{debug, error, info, instrument, trace, warn};
