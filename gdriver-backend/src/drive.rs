use std::collections::HashMap;

use crate::drive::google_drive::GoogleDrive;
use crate::path_resolver::PathResolver;
use chrono::{DateTime, Utc};

use crate::prelude::*;
mod google_drive;
pub struct Drive {
    tracked_files: HashMap<DriveId, DateTime<Utc>>,
    pub path_resolver: PathResolver,
    google_drive: GoogleDrive,
}
impl Drive {
    #[instrument()]
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tracked_files: HashMap::new(),
            path_resolver: PathResolver::new(),
            google_drive: GoogleDrive::new().await?,
        })
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
    pub async fn update(&mut self) -> Result<()> {
        let changes = self.google_drive.get_changes().await?;
        if changes.is_empty() {
            info!("No changes");
            return Ok(());
        }
        for change in changes {
            dbg!(change);
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
