use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::prelude::*;

pub struct Drive {
    tracked_files: HashMap<DriveId, DateTime<Utc>>,
}
impl Drive {
    pub fn new() -> Self {
        Self {
            tracked_files: HashMap::new(),
        }
    }
    pub fn get_file_tracking_state(&self, id: &DriveId) -> TrackingState {
        let file = self.tracked_files.get(id);
        match file {
            Some(date) => TrackingState::Tracked(*date),
            None => TrackingState::Untracked,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TrackingState {
    Untracked,
    Tracked(DateTime<Utc>),
}
