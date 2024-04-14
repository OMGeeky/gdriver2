use crate::filesystem::attributes::{read_inode_attributes_from_meta_file, ConvertFileType};
use crate::filesystem::errors::FilesystemError;
use crate::prelude::macros::*;
use crate::prelude::*;
use anyhow::anyhow;
use bimap::BiMap;
use fuser::{KernelConfig, ReplyAttr, ReplyDirectory, ReplyEntry, Request};
use gdriver_common::drive_structure::drive_id::DriveId;
use gdriver_common::drive_structure::drive_id::ROOT_ID;
use gdriver_common::ipc::gdriver_service::errors::GDriverServiceError;
use gdriver_common::ipc::gdriver_service::GDriverServiceClient;
use gdriver_common::ipc::gdriver_service::SETTINGS;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::os::raw::c_int;
use std::time::Duration;
use tarpc::context::current as current_context;
use tokio::sync::mpsc::Receiver;

mod macros;

//TODO2: Decide if this is a good TTL
const TTL: Duration = Duration::from_secs(2);

type Inode = u64;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct FileIdentifier {
    parent: Inode,
    name: OsString,
}
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum ShutdownRequest {
    Gracefully,
    Force,
}
pub struct Filesystem {
    gdriver_client: GDriverServiceClient,

    entry_ids: BiMap<Inode, DriveId>,
    ino_to_file_handles: HashMap<Inode, Vec<u64>>,
    next_ino: u64,
    entry_name_parent_to_ino: BiMap<FileIdentifier, Inode>,
    shutdown_signal_receiver: Receiver<ShutdownRequest>,
}

impl Filesystem {
    pub fn new(
        gdriver_client: GDriverServiceClient,
        shutdown_signal_receiver: Receiver<ShutdownRequest>,
    ) -> Self {
        Self {
            gdriver_client,
            entry_ids: BiMap::new(),
            ino_to_file_handles: HashMap::new(),
            next_ino: 222,
            entry_name_parent_to_ino: BiMap::new(),
            shutdown_signal_receiver,
        }
    }
    fn generate_ino(&mut self) -> Inode {
        let ino = self.next_ino;
        self.next_ino += 1;
        ino
    }
}

//region DriveFilesystem ino_to_id
impl Filesystem {
    fn get_id_from_ino(&self, ino: Inode) -> Option<&DriveId> {
        self.entry_ids.get_by_left(&ino)
    }
    fn get_ino_from_id(&mut self, id: DriveId) -> Inode {
        let x = self.entry_ids.get_by_right(&id);
        if let Some(ino) = x {
            return *ino;
        }
        self.add_id(id)
    }
    fn remove_id(&mut self, id: DriveId) -> Result<Inode> {
        if let Some((ino, _)) = self.entry_ids.remove_by_right(&id) {
            Ok(ino)
        } else {
            Err(Box::from(anyhow!("could not find id {}", id)))
        }
    }
    fn add_id(&mut self, id: DriveId) -> Inode {
        let ino = self.generate_ino();
        trace!("adding new ino for drive id: {} => {}", id, ino);
        self.entry_ids.insert(ino, id);
        ino
    }
}

//endregion
mod attributes;

impl fuser::Filesystem for Filesystem {
    //region init
    #[instrument(skip(self, _req, _config))]
    fn init(&mut self, _req: &Request<'_>, _config: &mut KernelConfig) -> StdResult<(), c_int> {
        self.entry_ids.insert(1, ROOT_ID.clone());

        send_request!(self.gdriver_client.update_changes(current_context()))
            .map_err(|e| {
                error!("Got a connection error while updating changes for on init. ");
                dbg!(e);
                libc::ECANCELED
            })?
            .map_err(|e| {
                error!("Error while updating changes on init");
                dbg!(e);
                libc::EIO
            })?;
        Ok(())
    }
    //endregion
    //region lookup
    #[instrument(skip(self, _req, reply))]
    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let metadata = utils::lookup::lookup(self, parent, name.to_os_string());
        match metadata {
            Ok(metadata) => {
                reply.entry(&TTL, &metadata.into(), 0);
            }
            Err(e) => {
                error!("Got an error during lookup: {e:?}");
                match e {
                    FilesystemError::Rpc(_) => reply.error(libc::EREMOTEIO),
                    FilesystemError::IO(_) => reply.error(libc::EIO),
                    FilesystemError::Service(_) | FilesystemError::NotFound => {
                        reply.error(libc::ENOENT)
                    }
                    FilesystemError::Other(e) => {
                        dbg!(e);
                        todo!("Handle other errors and decide what error code should be used here")
                    }
                }
            }
        }
    }
    //endregion
    #[instrument(skip(self, _req, reply))]
    fn getattr(&mut self, _req: &Request<'_>, ino: u64, reply: ReplyAttr) {
        let id = self.get_id_from_ino(ino);
        info!("Reading dir: {id:?}/{ino}");
        match id {
            None => {}
            Some(id) => {
                let result = utils::get_attributes(self, id, ino);
                match result {
                    Ok(attr) => {
                        reply.attr(&TTL, &attr.into());
                    }
                    Err(e) => {
                        error!("Got an error during readdir: {}", e);
                        dbg!(e);
                        reply.error(libc::EIO);
                    }
                }
            }
        }
    }
    #[instrument(skip(self, _req, reply))]
    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let id = self.get_id_from_ino(ino);
        info!("Reading dir: {id:?}/{ino}");
        if let Err(e) = utils::update::update(self) {
            error!("Got an error during update in readdir: {}", e);
            dbg!(e);
            reply.error(libc::EIO);
            return;
        }

        match id {
            None => {}
            Some(id) => {
                let result = utils::readdir::readdir(self, id.clone(), offset as u64);
                match result {
                    Ok(entries) => {
                        let mut counter = 0;
                        for entry in entries {
                            let ino = self.get_ino_from_id(entry.id);
                            counter += 1;
                            let buffer_full =
                                reply.add(ino, counter, entry.kind.into_ft(), entry.name);
                            if buffer_full {
                                debug!("Buffer full after {counter}");
                                break;
                            }
                        }
                        debug!("sending ok");
                        reply.ok();
                    }
                    Err(e) => {
                        error!("Got an error during readdir: {}", e);
                        dbg!(e);
                        reply.error(libc::EIO);
                    }
                }
            }
        }
    }
}
mod errors {
    use gdriver_common::ipc::gdriver_service::errors::GDriverServiceError;
    use std::error::Error;
    use tarpc::client::RpcError;

    #[derive(Debug, thiserror::Error)]
    pub enum FilesystemError {
        #[error("Error while executing RPC: {0}")]
        Rpc(#[from] RpcError),
        #[error("Could not find entity specified")]
        NotFound,
        #[error("IO Error")]
        IO(#[source] Box<dyn Error>),
        #[error("Service returned Error: {0}")]
        Service(#[from] GDriverServiceError),
        #[error("Some other error occurred: {0}")]
        Other(#[source] Box<dyn Error>),
    }
}
mod utils {
    use super::*;
    use crate::filesystem::attributes::InodeAttributes;
    pub mod update {
        use super::*;
        #[instrument(skip(fs))]
        pub fn update(fs: &Filesystem) -> StdResult<(), FilesystemError> {
            info!("Updating changes");
            send_request!(fs.gdriver_client.update_changes(current_context(),))?
                .map_err(GDriverServiceError::from)?;
            Ok(())
        }
    }
    pub mod lookup {
        use super::*;
        use crate::filesystem::attributes::InodeAttributes;
        use crate::filesystem::errors::FilesystemError;
        use futures::TryFutureExt;
        use gdriver_common::ipc::gdriver_service::errors::GetFileByPathError;

        pub fn lookup(
            fs: &mut Filesystem,
            parent: Inode,
            name: OsString,
        ) -> StdResult<InodeAttributes, FilesystemError> {
            let id: DriveId;
            let ino: Inode;

            let name = name.to_os_string();
            let ino_opt = fs.entry_name_parent_to_ino.get_by_left(&FileIdentifier {
                parent,
                name: name.clone(),
            });

            match ino_opt {
                None => {
                    //we don't know this name with this parent already, so we have to look it up
                    let parent_id = fs
                        .entry_ids
                        .get_by_left(&parent)
                        .ok_or(FilesystemError::NotFound)?;
                    trace!(
                        "looking for child of parent:{} with name: {:?}",
                        parent_id,
                        name
                    );
                    id = send_request!(fs.gdriver_client.get_file_by_name(
                        current_context(),
                        name.to_os_string(),
                        parent_id.clone()
                    ))?
                    .map_err(GDriverServiceError::from)?;

                    ino = fs.add_id(id.clone());
                }
                Some(i) => {
                    ino = *i;
                    id = fs
                        .get_id_from_ino(*i)
                        .ok_or(FilesystemError::NotFound)?
                        .clone();
                }
            }
            get_attributes(fs, &id, ino)
        }
    }
    pub(crate) fn get_attributes(
        fs: &Filesystem,
        id: &DriveId,
        ino: Inode,
    ) -> StdResult<InodeAttributes, FilesystemError> {
        let open_file_handles = fs.ino_to_file_handles.get(&ino).map(Vec::len).unwrap_or(0) as u64;
        send_request!(fs
            .gdriver_client
            .get_metadata_for_file(current_context(), id.clone()))?
        .map_err(GDriverServiceError::from)?;
        let meta_path = SETTINGS.get_metadata_file_path(&id);
        let metadata = read_inode_attributes_from_meta_file(&meta_path, ino, open_file_handles)
            .map_err(FilesystemError::IO)?;
        Ok(metadata)
    }
    pub mod readdir {
        use super::*;
        pub fn readdir(
            fs: &mut Filesystem,
            id: DriveId,
            offset: u64,
        ) -> StdResult<Vec<gdriver_common::ipc::gdriver_service::ReadDirResult>, FilesystemError>
        {
            let res = send_request!(fs.gdriver_client.list_files_in_directory_with_offset(
                current_context(),
                id,
                offset as u64
            ))?
            .map_err(GDriverServiceError::from)?;
            Ok(res)
        }
    }
}
