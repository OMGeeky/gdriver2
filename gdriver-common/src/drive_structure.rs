use serde::{Deserialize, Serialize};

pub mod drive_id;
pub mod meta;
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct DriveObject {
    pub id: drive_id::DriveId,
    pub metadata: meta::Metadata,
}
