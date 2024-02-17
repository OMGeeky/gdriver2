pub(crate) type Result<T> = StdResult<T, Box<dyn Error>>;
pub(crate) use gdriver_common::drive_structure::drive_id::{DriveId, ROOT_ID};
pub(crate) use std::{error::Error, result::Result as StdResult};
