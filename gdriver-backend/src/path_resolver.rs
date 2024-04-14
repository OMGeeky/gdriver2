use crate::drive::Drive;
use crate::prelude::*;
use gdriver_common::ipc::gdriver_service::ReadDirResult;
use gdriver_common::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PathResolver {
    parents: HashMap<DriveId, Vec<DriveId>>,
    children: HashMap<DriveId, Vec<ReadDirResult>>,
}

impl PathResolver {
    pub fn new() -> Self {
        Self {
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    pub async fn get_id_from_path(&mut self, path: &Path, drive: &Drive) -> Result<DriveId> {
        let segments: Vec<_> = path
            .to_str()
            .ok_or(PathResolveError::InvalidPath)?
            .split('/')
            .collect();
        let mut current = ROOT_ID.clone();
        self.update_from_drive(drive).await?;
        for segment in segments {
            current = self
                .get_id_from_parent_and_name(segment, &current)
                .ok_or("path-segment not found")?;
        }
        return Ok(current);
    }
    pub fn get_id_from_parent_and_name(&self, name: &str, parent: &DriveId) -> Option<DriveId> {
        if let Some(children) = self.children.get(parent) {
            if let Some(x) = children.into_iter().find(|x| x.name.eq(name)) {
                return Some(x.id.clone());
            }
        }
        None
    }

    async fn update_from_drive(&mut self, drive: &Drive) -> Result<()> {
        todo!()
    }
    pub(crate) fn add_relationship(&mut self, parent: DriveId, entry: ReadDirResult) {
        todo!()
    }
    pub(crate) fn remove_relationship(&mut self, parent: DriveId, entry: ReadDirResult) {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum PathResolveError {
    #[error("The path provided was invalid")]
    InvalidPath,
}
