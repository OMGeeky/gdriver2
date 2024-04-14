use crate::drive::Drive;
use crate::prelude::*;
use gdriver_common::ipc::gdriver_service::ReadDirResult;
use gdriver_common::path_resolve_error::PathResolveError;
use gdriver_common::prelude::*;
use std::collections::HashMap;
use std::path::Path;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PathResolver {
    /// A map of children to their parents
    parents: HashMap<DriveId, Vec<DriveId>>,
    /// A map of parents to their children with id, name and type (folder/file/symlink)
    children: HashMap<DriveId, Vec<ReadDirResult>>,
}

impl PathResolver {
    pub fn new() -> Self {
        Self {
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    pub async fn get_id_from_path(
        &mut self,
        path: &Path,
    ) -> StdResult<Option<DriveId>, PathResolveError> {
        let segments: Vec<_> = path
            .to_str()
            .ok_or(PathResolveError::InvalidPath)?
            .split('/')
            .collect();
        let mut current = ROOT_ID.clone();
        for segment in segments {
            current = self.get_id_from_parent_and_name(segment, &current).ok_or(
                PathResolveError::Other("path-segment not found".to_string()),
            )?;
        }
        return Ok(Some(current));
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
    /// Add a relationship between a parent and a child
    pub(crate) fn add_relationship(&mut self, parent: DriveId, entry: ReadDirResult) {
        match self.parents.get_mut(&entry.id) {
            Some(x) => x.push(parent.clone()),
            None => {
                self.parents.insert(entry.id.clone(), vec![parent.clone()]);
            }
        };
        match self.children.get_mut(&parent) {
            Some(x) => x.push(entry.clone()),
            None => {
                self.children.insert(parent.clone(), vec![entry.clone()]);
            }
        }
    }
    /// Remove the relationship between a parent and a child
    pub(crate) fn remove_relationship(&mut self, parent: DriveId, entry: ReadDirResult) {
        self.parents
            .get_mut(&entry.id)
            .map(|x| x.retain(|e| e != &parent));
        self.children
            .get_mut(&parent)
            .map(|x| x.retain(|e| e.id != entry.id));
    }
}
