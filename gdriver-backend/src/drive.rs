use crate::apply_change;
use crate::drive::google_drive::{FileData, GoogleDrive};
use crate::path_resolver::PathResolver;
use chrono::{DateTime, Utc};
use gdriver_common::drive_structure::meta::{read_metadata_by_id, write_metadata_file, Metadata};
use google_drive3::api::Change;
use std::collections::HashMap;

use crate::prelude::*;
mod google_drive;
pub struct Drive {
    tracked_files: HashMap<DriveId, DateTime<Utc>>,
    pub path_resolver: PathResolver,
    google_drive: GoogleDrive,
    pub offline_mode: bool,
}
impl Drive {
    #[instrument()]
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tracked_files: HashMap::new(),
            path_resolver: PathResolver::new(),
            google_drive: GoogleDrive::new().await?,
            offline_mode: false,
        })
    }
    pub fn set_offline_mode(&mut self, offline_mode: bool) {
        self.offline_mode = offline_mode;
    }
    #[instrument(skip(self))]
    pub fn get_file_tracking_state(&self, id: &DriveId) -> TrackingState {
        let file = self.tracked_files.get(id);
        match file {
            Some(date) => TrackingState::Tracked(*date),
            None => TrackingState::Untracked,
        }
    }

    #[instrument(skip(self))]
    pub async fn get_all_file_metas(&mut self) -> Result<()> {
        let has_existing_token = self.google_drive.has_local_change_token().await;
        //TODO: show an error when offline and no local data exists
        if !has_existing_token {
            //only get start token & data if this is the first time & we don't have it
            self.google_drive.get_change_start_token().await?;
            let files = self.google_drive.get_all_file_metas().await?;

            self.path_resolver.reset()?;
            for file in files {
                let parents = file.parents.clone();
                let meta = file.into_meta()?;
                self.path_resolver
                    .add_relationships_for_meta(parents, &meta)?;

                write_metadata_file(&meta)?;
            }
        } else {
            self.path_resolver.load_from_disk()?;
        }

        Ok(())
    }
    pub async fn download_meta_for_file(&self, id: &DriveId) -> Result<()> {
        let meta = self.google_drive.get_meta_for_file(id).await?;
        write_metadata_file(&meta.into_meta()?)?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn update(&mut self) -> Result<()> {
        if self.offline_mode {
            info!("Offline mode, skipping update");
            return Ok(());
        }
        let changes = self.google_drive.get_changes().await?;
        if changes.is_empty() {
            info!("No changes");
            return Ok(());
        }
        for change in changes {
            // dbg!(&change);
            self.process_change(change)?;
        }
        Ok(()) //TODO: implement this
    }
    #[instrument(skip(self, change))]
    fn process_change(&mut self, change: Change) -> Result<()> {
        let file_data =
            FileData::convert_from_api_file(change.file.ok_or("change had no file data")?);
        let parents = file_data
            .parents
            .clone()
            .into_iter()
            .map(Into::into)
            .collect();
        let new_meta = file_data.into_meta()?;
        info!("Processing change: {:?}", new_meta);
        let id: DriveId = change.file_id.clone().ok_or("No file id in change")?.into();
        let original_meta = read_metadata_by_id(&id);
        if original_meta.is_err() {
            info!("File not found so it has to be new: {:?}", id);
            self.path_resolver
                .add_relationships_for_meta(parents, &new_meta)?;
            write_metadata_file(&new_meta)?;
            return Ok(());
        }
        if change.removed.unwrap_or_default() {
            info!("File removed: {:?}", id);
            todo!("Do something when a file is removed from drive");
            return Ok(());
        }
        let mut original_meta = original_meta?;
        self.process_parents_changes(parents, &id, &new_meta)?;
        Self::process_meta_changes(new_meta, &mut original_meta)?;
        Ok(())
    }

    fn process_meta_changes(new_meta: Metadata, original_meta: &mut Metadata) -> Result<()> {
        let mut has_meta_changed = false;

        apply_change!(original_meta, new_meta, last_modified, has_meta_changed, where: {
            original_meta.last_modified < new_meta.last_modified
        });
        apply_change!(original_meta, new_meta, last_accessed, has_meta_changed, where: {
            original_meta.last_accessed < new_meta.last_accessed
        });
        apply_change!(original_meta, new_meta, last_metadata_changed, has_meta_changed, where: {
            original_meta.last_metadata_changed < new_meta.last_metadata_changed
        });
        apply_change!(original_meta, new_meta, name, has_meta_changed);
        apply_change!(original_meta, new_meta, size, has_meta_changed);
        apply_change!(original_meta, new_meta, permissions, has_meta_changed);
        apply_change!(original_meta, new_meta, extra_attributes, has_meta_changed);
        info!("Has changed: {}", has_meta_changed);
        if has_meta_changed {
            write_metadata_file(&original_meta)?;
        }
        Ok(())
    }

    fn process_parents_changes(
        &mut self,
        parents: Vec<DriveId>,
        id: &DriveId,
        new_meta: &Metadata,
    ) -> Result<()> {
        let original_parents = self.path_resolver.get_parents(&id)?.clone();
        if original_parents != parents {
            info!("Parents changed: {:?}", id);
            self.path_resolver
                .remove_relationships_for_id(&original_parents, &new_meta.id)?;
            self.path_resolver
                .add_relationships_for_meta(parents, &new_meta)?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn ping(&self) -> Result<()> {
        self.google_drive.ping().await
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrackingState {
    Untracked,
    Tracked(DateTime<Utc>),
}
mod macros;
