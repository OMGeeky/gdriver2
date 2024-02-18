use crate::prelude::*;
use fuser::FileType;
use gdriver_common::drive_structure::meta::{read_metadata_file, FileKind, Metadata};
use std::collections::BTreeMap;
use std::os::raw::c_int;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tarpc::serde::{Deserialize, Serialize};

type Inode = u64;
const BLOCK_SIZE: u64 = 512;
trait ConvertFileType {
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
fn time_now() -> (i64, u32) {
    time_from_system_time(&SystemTime::now())
}

fn system_time_from_time(secs: i64, nsecs: u32) -> SystemTime {
    if secs >= 0 {
        UNIX_EPOCH + Duration::new(secs as u64, nsecs)
    } else {
        UNIX_EPOCH - Duration::new((-secs) as u64, nsecs)
    }
}

fn time_from_system_time(system_time: &SystemTime) -> (i64, u32) {
    // Convert to signed 64-bit time with epoch at 0
    match system_time.duration_since(UNIX_EPOCH) {
        Ok(duration) => (duration.as_secs() as i64, duration.subsec_nanos()),
        Err(before_epoch_error) => (
            -(before_epoch_error.duration().as_secs() as i64),
            before_epoch_error.duration().subsec_nanos(),
        ),
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct InodeAttributes {
    pub inode: Inode,
    pub open_file_handles: u64, // Ref count of open file handles to this inode
    pub size: u64,
    pub last_accessed: (i64, u32),
    pub last_modified: (i64, u32),
    pub last_metadata_changed: (i64, u32),
    pub kind: FileKind,
    // Permissions and special mode bits
    pub mode: u16,
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
        mode: metadata.mode,
        hardlinks: metadata.hardlinks,
        uid: metadata.uid,
        gid: metadata.gid,
        xattrs: metadata.xattrs,
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
            atime: system_time_from_time(attrs.last_accessed.0, attrs.last_accessed.1),
            mtime: system_time_from_time(attrs.last_modified.0, attrs.last_modified.1),
            ctime: system_time_from_time(
                attrs.last_metadata_changed.0,
                attrs.last_metadata_changed.1,
            ),
            crtime: SystemTime::UNIX_EPOCH,
            kind: attrs.kind.into_ft(),
            perm: attrs.mode,
            nlink: attrs.hardlinks,
            uid: attrs.uid,
            gid: attrs.gid,
            rdev: 0,
            blksize: BLOCK_SIZE as u32,
            flags: 0,
        }
    }
}
