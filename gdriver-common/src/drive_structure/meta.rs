use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

pub type TIMESTAMP = (i64, u32);

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct Metadata {
    pub state: FileState,
    pub size: u64,
    pub last_accessed: TIMESTAMP,
    pub last_modified: TIMESTAMP,
    pub last_metadata_changed: TIMESTAMP,
    pub kind: FileKind,
    pub mode: u16,
    pub hardlinks: u32,
    pub uid: u32,
    pub gid: u32,
    pub xattrs: BTreeMap<Vec<u8>, Vec<u8>>,
}
pub fn read_metadata_file(path: &Path) -> Result<Metadata> {
    let reader = File::open(path)?;
    Ok(serde_json::from_reader(reader)?)
}
pub fn write_metadata_file(path: &Path, metadata: &Metadata) -> Result<()> {
    let reader = File::open(path)?;
    Ok(serde_json::to_writer(reader, metadata)?)
}
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub enum FileState {
    Downloaded,
    Cached,
    MetadataOnly,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub enum FileKind {
    File,
    Directory,
    Symlink,
}
