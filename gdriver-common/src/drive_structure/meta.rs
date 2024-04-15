use crate::ipc::gdriver_service::SETTINGS;
use crate::prelude::*;
use crate::time_utils::time_now;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, path::Path};

/// Timestamp is a tuple of (seconds, nanoseconds)
///
/// This is a duration since the Unix epoch in seconds + nanoseconds.
pub type TIMESTAMP = (i64, u32);

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Metadata {
    pub id: DriveId,
    pub state: FileState,
    pub size: u64,
    pub last_accessed: TIMESTAMP,
    pub last_modified: TIMESTAMP,
    pub last_metadata_changed: TIMESTAMP,
    pub kind: FileKind,
    pub permissions: u16,
    pub extra_attributes: BTreeMap<Vec<u8>, Vec<u8>>,
}

const PERMISSIONS_RWXRWXRWX: u16 = 0b111_111_111; // 511;

impl Metadata {
    pub fn root() -> Self {
        Self {
            id: ROOT_ID.clone(),
            state: FileState::Root,
            size: 0,
            last_accessed: time_now(),
            last_modified: time_now(),
            last_metadata_changed: time_now(),
            kind: FileKind::Directory,
            permissions: PERMISSIONS_RWXRWXRWX,
            extra_attributes: Default::default(),
        }
    }
}

pub fn read_metadata_file(path: &Path) -> Result<Metadata> {
    debug!("Reading metadata file: {:?}", path);
    let reader = File::open(path)?;
    Ok(serde_json::from_reader(reader)?)
}
pub fn write_metadata_file(metadata: &Metadata) -> Result<()> {
    let path = SETTINGS.get_metadata_file_path(&metadata.id);
    write_metadata_file_to_path(&path, metadata)
}
pub fn write_metadata_file_to_path(path: &Path, metadata: &Metadata) -> Result<()> {
    debug!("Writing metadata file: {:?}", path);
    let reader = File::create(path)?;
    Ok(serde_json::to_writer(reader, metadata)?)
}
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub enum FileState {
    Downloaded,
    Cached,
    MetadataOnly,
    Root,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
}
