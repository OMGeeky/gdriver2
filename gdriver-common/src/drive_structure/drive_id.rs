use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

lazy_static! {
    pub static ref ROOT_ID: DriveId = DriveId(String::from("root"));
}
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct DriveId(pub String);

impl<T> From<T> for DriveId
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        DriveId(s.into())
    }
}

impl AsRef<str> for DriveId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Display for DriveId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0)
    }
}
