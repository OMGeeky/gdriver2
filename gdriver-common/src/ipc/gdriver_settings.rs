use crate::prelude::*;
use crate::project_dirs::PROJECT_DIRS;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GDriverSettings {
    metadata_path: PathBuf,
    cache_path: PathBuf,
    downloaded_path: PathBuf,
    data_path: PathBuf,
}
impl GDriverSettings {
    #[instrument(skip(self))]
    pub fn initialize_dirs(&self) -> Result<()> {
        info!("Initializing dirs");
        let dirs = vec![
            &self.metadata_path,
            &self.cache_path,
            &self.downloaded_path,
            &self.data_path,
        ];
        for dir in dirs {
            info!("Creating dir: {:?}", dir);
            std::fs::create_dir_all(dir)?;
        }
        info!("Dirs created");
        Ok(())
    }
}
impl GDriverSettings {
    pub fn metadata_path(&self) -> &Path {
        &self.metadata_path
    }
    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }
    pub fn downloaded_path(&self) -> &Path {
        &self.downloaded_path
    }
    pub fn data_path(&self) -> &Path {
        &self.data_path
    }

    pub fn get_changes_file_path(&self) -> PathBuf {
        self.data_path.join("changes.txt")
    }
    pub fn get_path_resolver_file_path(&self) -> PathBuf {
        self.data_path.join("relations.json")
    }

    pub fn get_metadata_file_path(&self, id: &DriveId) -> PathBuf {
        self.metadata_path.join(id.as_ref()).with_extension("meta")
    }
    pub fn get_downloaded_file_path(&self, id: &DriveId) -> PathBuf {
        self.downloaded_path.join(id.as_ref())
    }
    pub fn get_cache_file_path(&self, id: &DriveId) -> PathBuf {
        self.cache_path.join(id.as_ref())
    }
}

impl Default for GDriverSettings {
    fn default() -> Self {
        let p = &PROJECT_DIRS;
        Self {
            metadata_path: p.data_dir().join("meta"),
            downloaded_path: p.data_dir().join("downloads"),
            cache_path: p.cache_dir().to_path_buf(),
            data_path: p.data_dir().join("data"),
        }
    }
}
