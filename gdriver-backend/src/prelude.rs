pub(crate) type Result<T> = StdResult<T, Box<dyn Error>>;
pub(crate) use std::{error::Error, result::Result as StdResult};
