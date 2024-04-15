use crate::filesystem::{GDRIVER_GROUP_ID, USER_ID};
use crate::prelude::*;
use fuser::FileType;
use gdriver_common::drive_structure::meta::{read_metadata_file, FileKind, Metadata, TIMESTAMP};
use gdriver_common::time_utils;
use gdriver_common::time_utils::time_from_system_time;
use std::collections::BTreeMap;
use std::os::raw::c_int;
use std::path::Path;
use std::time::SystemTime;
use tarpc::serde::{Deserialize, Serialize};

type Inode = u64;
const BLOCK_SIZE: u64 = 512;
pub trait ConvertFileType {
    fn from_ft(kind: FileType) -> Self;
    fn into_ft(self) -> FileType;
}
impl ConvertFileType for FileKind {
    fn from_ft(kind: fuser::FileType) -> Self {
        match kind {
            FileType::Directory => FileKind::Directory,
            FileType::RegularFile => FileKind::File,
            FileType::Symlink => FileKind::Symlink,
            _ => FileKind::File,
        }
    }
    fn into_ft(self) -> fuser::FileType {
        match self {
            FileKind::File => fuser::FileType::RegularFile,
            FileKind::Directory => fuser::FileType::Directory,
            FileKind::Symlink => fuser::FileType::Symlink,
        }
    }
}

#[derive(Debug)]
enum XattrNamespace {
    Security,
    System,
    Trusted,
    User,
}

fn parse_xattr_namespace(key: &[u8]) -> StdResult<XattrNamespace, c_int> {
    let user = b"user.";
    if key.len() < user.len() {
        return Err(libc::ENOTSUP);
    }
    if key[..user.len()].eq(user) {
        return Ok(XattrNamespace::User);
    }

    let system = b"system.";
    if key.len() < system.len() {
        return Err(libc::ENOTSUP);
    }
    if key[..system.len()].eq(system) {
        return Ok(XattrNamespace::System);
    }

    let trusted = b"trusted.";
    if key.len() < trusted.len() {
        return Err(libc::ENOTSUP);
    }
    if key[..trusted.len()].eq(trusted) {
        return Ok(XattrNamespace::Trusted);
    }

    let security = b"security";
    if key.len() < security.len() {
        return Err(libc::ENOTSUP);
    }
    if key[..security.len()].eq(security) {
        return Ok(XattrNamespace::Security);
    }

    return Err(libc::ENOTSUP);
}
#[derive(Serialize, Deserialize)]
pub(crate) struct InodeAttributes {
    pub inode: Inode,
    pub open_file_handles: u64, // Ref count of open file handles to this inode
    pub size: u64,
    pub last_accessed: TIMESTAMP,
    pub last_modified: TIMESTAMP,
    pub last_metadata_changed: TIMESTAMP,
    pub kind: FileKind,
    // Permissions and special mode bits
    pub permissions: u16,
    pub hardlinks: u32,
    pub uid: u32,
    pub gid: u32,
    pub xattrs: BTreeMap<Vec<u8>, Vec<u8>>,
}
pub(crate) fn read_inode_attributes_from_metadata(
    metadata: Metadata,
    inode: Inode,
    open_file_handles: u64,
) -> InodeAttributes {
    InodeAttributes {
        inode,
        open_file_handles,
        size: metadata.size,
        last_accessed: metadata.last_accessed,
        last_modified: metadata.last_modified,
        last_metadata_changed: metadata.last_metadata_changed,
        kind: metadata.kind,
        permissions: metadata.permissions,
        hardlinks: 0,
        uid: *USER_ID,
        gid: *GDRIVER_GROUP_ID,
        xattrs: metadata.extra_attributes,
    }
}
pub(crate) fn read_inode_attributes_from_meta_file(
    meta_path: &Path,
    inode: Inode,
    open_file_handles: u64,
) -> Result<InodeAttributes> {
    let metadata = read_metadata_file(meta_path)?;
    Ok(read_inode_attributes_from_metadata(
        metadata,
        inode,
        open_file_handles,
    ))
}

impl From<InodeAttributes> for fuser::FileAttr {
    fn from(attrs: InodeAttributes) -> Self {
        fuser::FileAttr {
            ino: attrs.inode,
            size: attrs.size,
            blocks: (attrs.size + BLOCK_SIZE - 1) / BLOCK_SIZE,
            atime: time_utils::system_time_from_timestamp(
                attrs.last_accessed.0,
                attrs.last_accessed.1,
            ),
            mtime: time_utils::system_time_from_timestamp(
                attrs.last_modified.0,
                attrs.last_modified.1,
            ),
            ctime: time_utils::system_time_from_timestamp(
                attrs.last_metadata_changed.0,
                attrs.last_metadata_changed.1,
            ),
            crtime: SystemTime::UNIX_EPOCH,
            kind: attrs.kind.into_ft(),
            perm: attrs.permissions,
            nlink: attrs.hardlinks,
            uid: attrs.uid,
            gid: attrs.gid,
            rdev: 0,
            blksize: BLOCK_SIZE as u32,
            flags: 0,
        }
    }
}
