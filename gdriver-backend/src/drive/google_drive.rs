use crate::prelude::*;
use chrono::{DateTime, Utc};
use const_format::formatcp;
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

const FIELDS_FILE: &'static str = "id, name, size, mimeType, kind, md5Checksum, parents, trashed, createdTime, modifiedTime, viewedByMeTime";
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Hash)]
pub struct FileData {
    pub id: String,
    pub name: String,
    pub size: Option<i64>,
    pub mime_type: String,
    pub kind: String,
    pub md5_checksum: Option<String>,
    pub parents: Option<Vec<String>>,
    pub trashed: Option<bool>,
    pub created_time: Option<DateTime<Utc>>,
    pub modified_time: Option<DateTime<Utc>>,
    pub viewed_by_me_time: Option<DateTime<Utc>>,
}

impl FileData {
    fn convert_from_api_file(file: File) -> Self {
        Self {
            id: file.id.unwrap_or_default(),
            name: file.name.unwrap_or_default(),
            size: file.size,
            mime_type: file.mime_type.unwrap_or_default(),
            kind: file.kind.unwrap_or_default(),
            md5_checksum: file.md5_checksum,
            parents: file.parents,
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

        let drive = GoogleDrive {
            hub,
            changes_start_page_token: None,
        };
        trace!("Successfully initialized {}", drive);
        Ok(drive)
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
    #[instrument]
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
        trace!("Got {} changes", changes.len());
        Ok(changes)
    }
    async fn set_change_start_token(&mut self, token: String) -> Result<()> {
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
            .ok();
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
}

//region debug & display traits
impl Debug for GoogleDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(type_name::<GoogleDrive>())
            .field("changes_start_page_token", &self.changes_start_page_token)
            .finish()
    }
}

impl Display for GoogleDrive {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", type_name::<GoogleDrive>())
    }
}
//endregion
