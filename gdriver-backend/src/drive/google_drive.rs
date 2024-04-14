use crate::prelude::*;
use const_format::formatcp;
use gdriver_common::ipc::gdriver_service::SETTINGS;
use gdriver_common::prelude::*;
use google_drive3::api::{Change, File, Scope, StartPageToken};
use google_drive3::hyper::client::HttpConnector;
use google_drive3::hyper::{Body, Client, Response};
use google_drive3::hyper_rustls::HttpsConnector;
use google_drive3::DriveHub;
use google_drive3::{hyper_rustls, oauth2};
use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use tokio::fs;

const FIELDS_FILE: &'static str = "id, name, size, mimeType, kind, md5Checksum, parents, trashed, createdTime, modifiedTime, viewedByMeTime";
const FIELDS_CHANGE: &str = formatcp!("changes(removed, fileId, changeType, file({FIELDS_FILE}))");
#[derive(Clone)]
pub struct GoogleDrive {
    hub: DriveHub<HttpsConnector<HttpConnector>>,
    changes_start_page_token: Option<String>,
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
        let mut page_token = Some(self.change_start_token().await?);
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
            self.changes_start_page_token = body.new_start_page_token;
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

    async fn change_start_token(&mut self) -> Result<String> {
        Ok(match &self.changes_start_page_token {
            None => {
                //
                let token = fs::read_to_string(SETTINGS.get_changes_file_path())
                    .await
                    .unwrap_or_default();
                self.changes_start_page_token = if !token.is_empty() {
                    Some(token)
                } else {
                    Some(self.get_changes_start_token_from_api().await?)
                };
                self.changes_start_page_token
                    .clone()
                    .expect("We just set it")
            }
            Some(start_token) => start_token.clone(),
        })
    }
    async fn get_changes_start_token_from_api(&self) -> Result<String> {
        let (response, body) = self
            .hub
            .changes()
            .get_start_page_token()
            .supports_all_drives(false)
            .doit()
            .await?;
        if response.status().is_success() {
            let start_page_token = body.start_page_token.unwrap_or_default();
            Ok(start_page_token)
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
