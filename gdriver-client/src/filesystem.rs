use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::os::raw::c_int;
use std::time::Duration;

use anyhow::anyhow;
use bimap::BiMap;
use fuser::{KernelConfig, ReplyEntry, Request};
use tracing::*;

use gdriver_common::drive_structure::drive_id::{DriveId, ROOT_ID};
use gdriver_common::drive_structure::meta::read_metadata_file;
use gdriver_common::ipc::gdriver_service::{
    errors::GDriverServiceError, GDriverServiceClient, GDriverSettings,
};

use crate::filesystem::attributes::read_inode_attributes_from_meta_file;
use crate::filesystem::errors::FilesystemError;
use crate::prelude::macros::*;
use crate::prelude::*;
use tarpc::context::current as current_context;

mod macros;

//TODO2: Decide if this is a good TTL
const TTL: Duration = Duration::from_secs(2);

type Inode = u64;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct FileIdentifier {
    parent: Inode,
    name: OsString,
}

pub struct Filesystem {
    gdriver_client: GDriverServiceClient,

    entry_ids: BiMap<Inode, DriveId>,
    ino_to_file_handles: HashMap<Inode, Vec<u64>>,
    next_ino: u64,
    gdriver_settings: GDriverSettings,
    entry_name_parent_to_ino: BiMap<FileIdentifier, Inode>,
}

impl Filesystem {
    pub fn new(gdriver_client: GDriverServiceClient) -> Self {
        Self {
            gdriver_client,
            entry_ids: BiMap::new(),
            ino_to_file_handles: HashMap::new(),
            next_ino: 222,
            gdriver_settings: GDriverSettings::default(),
            entry_name_parent_to_ino: BiMap::new(),
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
    fn init(&mut self, _req: &Request<'_>, _config: &mut KernelConfig) -> StdResult<(), c_int> {
        self.entry_ids.insert(1, ROOT_ID.clone());
        self.gdriver_settings = send_request!(self.gdriver_client.get_settings(current_context()))
            .map_err(|e| {
                error!("Got a connection error while fetching settings: {e}");
                libc::ECONNREFUSED
            })?
            .map_err(|e| {
                error!("Got an error while fetching settings: {e}");
                trace!("details: {e:?}");
                libc::EBADMSG
            })?;

        Ok(())
    }
    //endregion
    //region lookup
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
            let open_file_handles =
                fs.ino_to_file_handles.get(&ino).map(Vec::len).unwrap_or(0) as u64;
            send_request!(fs
                .gdriver_client
                .get_metadata_for_file(current_context(), id.clone()))?
            .map_err(GDriverServiceError::from)?;
            let meta_path = fs.gdriver_settings.get_metadata_file_path(&id);
            let metadata = read_inode_attributes_from_meta_file(&meta_path, ino, open_file_handles)
                .map_err(FilesystemError::IO)?;
            Ok(metadata)
        }
    }
}
