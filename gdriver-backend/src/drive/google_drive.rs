use crate::prelude::*;
use chrono::{DateTime, Utc};
use const_format::formatcp;
use gdriver_common::drive_structure::meta::{FileKind, FileState, Metadata, DEFAULT_PERMISSIONS};
use gdriver_common::time_utils::datetime_to_timestamp;
use gdriver_common::{ipc::gdriver_service::SETTINGS, prelude::*};
use google_drive3::api::File;
use google_drive3::{
    api::{Change, Scope},
    hyper::{client::HttpConnector, Client},
    hyper_rustls::{self, HttpsConnector},
    oauth2, DriveHub,
};
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use tokio::fs;

const FIELDS_FILE: &'static str = "id, name, size, mimeType, kind, md5Checksum, parents, trashed, createdTime, modifiedTime, viewedByMeTime, capabilities";
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct FileData {
    pub id: String,
    pub name: String,
    pub size: Option<i64>,
    pub mime_type: String,
    pub kind: FileKind,
    pub md5_checksum: Option<String>,
    pub parents: Vec<String>,
    pub trashed: Option<bool>,
    pub created_time: Option<DateTime<Utc>>,
    pub modified_time: Option<DateTime<Utc>>,
    pub viewed_by_me_time: Option<DateTime<Utc>>,
}

impl FileData {
    pub(crate) fn into_meta(self) -> Result<Metadata> {
        let last_modified = datetime_to_timestamp(self.modified_time.unwrap_or_default())?;
        Ok(Metadata {
            id: self.id.into(),
            kind: self.kind,
            name: self.name,
            size: self.size.unwrap_or_default() as u64,
            last_accessed: datetime_to_timestamp(self.viewed_by_me_time.unwrap_or_default())?,
            last_modified,
            extra_attributes: Default::default(),
            state: FileState::MetadataOnly,
            permissions: DEFAULT_PERMISSIONS, //TODO: parse permissions
            last_metadata_changed: last_modified,
        })
    }
}

impl FileData {
    pub(crate) fn convert_from_api_file(file: File) -> Self {
        let kind = file.kind.unwrap_or_default();
        info!(
            "Converting file with id {:?} with parent: {:?}",
            file.id, file.parents
        );
        let kind = match kind.as_str() {
            "drive#file" => FileKind::File,
            "drive#folder" => FileKind::Directory,
            "drive#link" => FileKind::Symlink,
            _ => todo!("Handle kind: {}", kind),
        };
        Self {
            id: file.id.unwrap_or_default(),
            name: file.name.unwrap_or_default(),
            size: file.size,
            mime_type: file.mime_type.unwrap_or_default(),
            kind,
            md5_checksum: file.md5_checksum,
            parents: file.parents.unwrap_or(vec![ROOT_ID.0.clone()]),
            trashed: file.trashed,
            created_time: file.created_time,
            modified_time: file.modified_time,
            viewed_by_me_time: file.viewed_by_me_time,
        }
    }
}
const FIELDS_CHANGE: &str = formatcp!(
    "nextPageToken, newStartPageToken, changes(removed, fileId, changeType, file({}))",
    FIELDS_FILE
);
#[derive(Clone)]
pub struct GoogleDrive {
    hub: DriveHub<HttpsConnector<HttpConnector>>,
    changes_start_page_token: Option<String>,
    root_alt_id: DriveId,
}

impl GoogleDrive {
    #[instrument]
    pub(crate) async fn get_all_file_metas(&self) -> Result<Vec<FileData>> {
        let mut page_token: Option<String> = None;
        let mut files = Vec::new();
        loop {
            let (response, body) = self
                .hub
                .files()
                .list()
                .supports_all_drives(false)
                .spaces("drive")
                .page_token(&page_token.unwrap_or_default())
                .param("fields", &format!("nextPageToken, files({})", FIELDS_FILE))
                .doit()
                .await?;
            page_token = body.next_page_token;
            if response.status().is_success() {
                files.extend(
                    body.files
                        .unwrap_or_default()
                        .into_iter()
                        .map(|mut f| {
                            self.map_in_file(Some(&mut f));
                            f
                        })
                        .map(FileData::convert_from_api_file),
                );
            } else {
                error!("Could not get files: {:?}", response);
                return Err("Could not get files".into());
            }
            if page_token.is_none() {
                info!("No more pages");
                break;
            }
        }
        Ok(files)
    }
    #[instrument]
    pub(crate) async fn get_meta_for_file(&self, id: &DriveId) -> Result<FileData> {
        let (response, mut body) = self
            .hub
            .files()
            .get(id.as_ref())
            .supports_all_drives(false)
            .param("fields", &FIELDS_FILE)
            .doit()
            .await?;
        if response.status().is_success() {
            self.map_in_file(Some(&mut body));
            return Ok(FileData::convert_from_api_file(body));
        }
        Err("Error while fetching metadata".into())
    }
}

impl GoogleDrive {
    #[instrument]
    pub(crate) async fn new() -> Result<Self> {
        trace!("Initializing GoogleDrive client.");
        let auth = oauth2::read_application_secret("auth/client_secret.json").await?;

        let auth = oauth2::InstalledFlowAuthenticator::builder(
            auth,
            oauth2::InstalledFlowReturnMethod::HTTPRedirect,
        )
        .persist_tokens_to_disk("auth/tokens.json")
        .build()
        .await?;
        let http_client = Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                // .enable_http2()
                .build(),
        );
        let hub = DriveHub::new(http_client, auth);

        let mut drive = GoogleDrive {
            hub,
            changes_start_page_token: None,
            root_alt_id: ROOT_ID.clone(),
        };
        info!("Updating ROOT alt");
        drive.update_alt_root().await?;
        info!("Updated ROOT alt to {}", drive.root_alt_id);
        trace!("Successfully initialized {:?}", drive);
        Ok(drive)
    }
    async fn update_alt_root(&mut self) -> Result<()> {
        let (response, body) = self
            .hub
            .files()
            .get(ROOT_ID.as_ref())
            .param("fields", "id")
            .doit()
            .await?;
        if response.status().is_success() {
            self.root_alt_id = body.id.unwrap_or(ROOT_ID.to_string()).into();
        }

        Ok(())
    }
    #[instrument]
    pub(crate) async fn ping(&self) -> Result<()> {
        let (response, body) = self
            .hub
            .about()
            .get()
            .param("fields", "user(emailAddress)")
            .add_scope(Scope::Readonly)
            .doit()
            .await?;
        let status_code = response.status();
        let email = body
            .user
            .unwrap_or_default()
            .email_address
            .unwrap_or_default();
        trace!("response status: {}, email: '{}'", status_code, email);
        if status_code.is_success() {
            return Ok(());
        }
        error!(
            "Did not get expected result on ping: {} {:?}",
            status_code, response
        );

        Err("Did not get expected result on ping".into())
    }
    //region changes
    #[instrument(skip(self))]
    pub async fn get_changes(&mut self) -> Result<Vec<Change>> {
        info!("Getting changes");
        let mut page_token = Some(self.get_change_start_token().await?);
        let mut changes = Vec::new();
        while let Some(current_page_token) = page_token {
            info!("Getting changes with page token: {}", current_page_token);
            let (response, body) = self
                .hub
                .changes()
                .list(current_page_token.as_str())
                .param("fields", FIELDS_CHANGE)
                .page_size(2) //TODO: Change this to a more reasonable value
                .include_corpus_removals(true) //TODO3: Check if this is useful
                .supports_all_drives(false)
                .restrict_to_my_drive(true)
                .include_removed(true)
                .include_items_from_all_drives(false)
                .doit()
                .await?;
            if let Some(token) = body.new_start_page_token {
                self.set_change_start_token(token).await?;
            }
            if response.status().is_success() {
                changes.extend(body.changes.unwrap_or_default());
                page_token = body.next_page_token;
            } else {
                error!("Could not get changes: {:?}", response);
                return Err("Could not get changes".into());
            }
        }
        changes
            .iter_mut()
            .for_each(|change| self.map_id_in_change(change));
        trace!("Got {} changes", changes.len());
        Ok(changes)
    }

    async fn set_change_start_token(&mut self, token: String) -> Result<()> {
        if self.changes_start_page_token.as_ref() == Some(&token) {
            return Ok(());
        }
        info!("Setting start page token: {}", token);
        fs::write(SETTINGS.get_changes_file_path(), token.clone()).await?;
        self.changes_start_page_token = Some(token);
        Ok(())
    }

    pub async fn get_change_start_token(&mut self) -> Result<String> {
        Ok(match &self.changes_start_page_token {
            None => {
                info!("Getting start page token");
                let has_local_token = self.has_local_change_token().await;
                if !has_local_token {
                    self.update_change_start_token_from_api().await?;
                }
                info!("Got start page token: {:?}", self.changes_start_page_token);
                self.changes_start_page_token
                    .clone()
                    .expect("We just set it")
            }
            Some(start_token) => {
                info!("Using cached start page token");
                start_token.clone()
            }
        })
    }

    pub(crate) async fn has_local_change_token(&mut self) -> bool {
        !self
            .get_local_change_start_token()
            .await
            .unwrap_or_default()
            .is_empty()
    }

    pub async fn get_local_change_start_token(&mut self) -> Option<String> {
        self.changes_start_page_token = fs::read_to_string(SETTINGS.get_changes_file_path())
            .await
            .ok()
            .map(|s| s.trim().to_string());
        self.changes_start_page_token.clone()
    }
    async fn update_change_start_token_from_api(&mut self) -> Result<()> {
        info!("Getting start page token from API");
        let (response, body) = self
            .hub
            .changes()
            .get_start_page_token()
            .supports_all_drives(false)
            .doit()
            .await?;
        if response.status().is_success() {
            let token = body.start_page_token.clone().unwrap_or_default();
            self.set_change_start_token(token).await?;
            Ok(())
        } else {
            Err("Could not get start page token".into())
        }
    }
    //endregion
    //region map alt_root_id to ROOT_ID
    fn map_id_in_change(&self, i: &mut Change) {
        i.file_id.as_mut().map(|id| self.map_id(id));
        let file = i.file.as_mut();
        self.map_in_file(file);
    }

    fn map_in_file(&self, file: Option<&mut File>) {
        file.map(|f| {
            f.parents.as_mut().map(|parents| {
                parents
                    .iter_mut()
                    .inspect(|i| println!("parent: {i}"))
                    .for_each(|id| self.map_id(id))
            });
            f.id.as_mut().map(|id| self.map_id(id));
        });
    }

    fn map_id(&self, id: &mut String) {
        if self.root_alt_id.0.eq(id) {
            info!("replacing {id} with {}", ROOT_ID.as_ref());
            *id = ROOT_ID.0.clone();
        } else {
            // info!("{id} did not match {}", self.root_alt_id);
        }
    }
    //endregion
}
//region debug & display traits
impl Debug for GoogleDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<GoogleDrive>())
            .field("changes_start_page_token", &self.changes_start_page_token)
            .field("root_alt_id", &self.root_alt_id)
            .finish()
    }
}

impl Display for GoogleDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<GoogleDrive>())
    }
}
//endregion
