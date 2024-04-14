use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum PathResolveError {
    #[error("The path provided was invalid")]
    InvalidPath,
    #[error("Some other error occurred")]
    Other(String),
}
