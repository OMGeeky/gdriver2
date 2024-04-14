use std::collections::HashMap;

use crate::drive::google_drive::GoogleDrive;
use crate::path_resolver::PathResolver;
use chrono::{DateTime, Utc};
use gdriver_common::drive_structure::meta::{write_metadata_file, write_metadata_file_to_path};
use gdriver_common::ipc::gdriver_service::ReadDirResult;

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
                for parent in file.parents.clone() {
                    let relation_data = ReadDirResult {
                        id: file.id.clone().into(),
                        name: file.name.clone(),
                        kind: file.kind.clone(),
                    };
                    self.path_resolver
                        .add_relationship(parent.into(), relation_data)?;
                }
                let meta = file.into_meta()?;
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
            dbg!(&change);
        }
        Err("Not implemented".into())

        // Ok(()) //TODO: implement this
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
