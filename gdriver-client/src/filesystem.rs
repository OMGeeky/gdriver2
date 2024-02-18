use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::raw::c_int;
use std::time::Duration;

use anyhow::anyhow;
use bimap::BiMap;
use fuser::{KernelConfig, ReplyEntry, Request};
use tracing::*;

use gdriver_common::drive_structure::drive_id::{DriveId, ROOT_ID};
use gdriver_common::ipc::gdriver_service::{
    errors::GDriverServiceError, GDriverServiceClient, GDriverSettings,
};

use crate::prelude::*;
use crate::{
    reply_error_e, reply_error_o, send_request, send_request_handled, send_request_handled2,
    send_request_handled2_consuming,
};

mod macros;

//TODO2: Decide if this is a good TTL
const TTL: Duration = Duration::from_secs(2);

pub struct Filesystem {
    gdriver_client: GDriverServiceClient,

    entry_ids: BiMap<u64, DriveId>,
    ino_to_file_handles: HashMap<u64, Vec<u64>>,
    next_ino: u64,
    gdriver_settings: GDriverSettings,
}

impl Filesystem {
    pub fn new(gdriver_client: GDriverServiceClient) -> Self {
        Self {
            gdriver_client,
            entry_ids: BiMap::new(),
            ino_to_file_handles: HashMap::new(),
            next_ino: 222,
            gdriver_settings: GDriverSettings::default(),
        }
    }
    fn generate_ino(&mut self) -> u64 {
        let ino = self.next_ino;
        self.next_ino += 1;
        ino
    }
}

//region DriveFilesystem ino_to_id
impl Filesystem {
    fn get_id_from_ino(&self, ino: u64) -> Option<&DriveId> {
        self.entry_ids.get_by_left(&ino)
    }
    fn get_ino_from_id(&mut self, id: DriveId) -> u64 {
        let x = self.entry_ids.get_by_right(&id);
        if let Some(ino) = x {
            return *ino;
        }
        self.add_id(id)
    }
    fn remove_id(&mut self, id: DriveId) -> Result<u64> {
        if let Some((ino, _)) = self.entry_ids.remove_by_right(&id) {
            Ok(ino)
        } else {
            Err(Box::from(anyhow!("could not find id {}", id)))
        }
    }
    fn add_id(&mut self, id: DriveId) -> u64 {
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
        self.gdriver_settings =
            send_request!(self.gdriver_client.get_settings(tarpc::context::current()))
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
        let parent_id = self.entry_ids.get_by_left(&parent);
        let parent_id = reply_error_o!(
            parent_id,
            reply,
            libc::ENOENT,
            "Failed to find drive_id for parent ino: {}",
            parent
        );
        trace!(
            "looking for child of parent:{} with name: {:?}",
            parent_id,
            name
        );

        let id = reply_error_e!(
            send_request_handled2!(
                self.gdriver_client.get_file_by_name(
                    tarpc::context::current(),
                    name.to_os_string(),
                    parent_id.clone()
                ),
                reply
            ),
            reply,
            libc::ENOENT,
            "Could not find file by name '{:?}' under parent: {}",
            name,
            parent_id
        );
        send_request_handled2_consuming!(
            self.gdriver_client
                .get_metadata_for_file(tarpc::context::current(), id),
            reply,
            parent
        );

        todo!()
    }
}
