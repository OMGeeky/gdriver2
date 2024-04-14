use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs::File, path::Path};

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

impl Metadata {
    pub fn root() -> Self {
        Self {
            state: FileState::Root,
            size: 0,
            last_accessed: (0, 0),
            last_modified: (0, 0),
            last_metadata_changed: (0, 0),
            kind: FileKind::Directory,
            mode: 0,
            hardlinks: 0,
            uid: 0,
            gid: 0,
            xattrs: Default::default(),
        }
    }
}

pub fn read_metadata_file(path: &Path) -> Result<Metadata> {
    debug!("Reading metadata file: {:?}", path);
    let reader = File::open(path)?;
    Ok(serde_json::from_reader(reader)?)
}
pub fn write_metadata_file(path: &Path, metadata: &Metadata) -> Result<()> {
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
