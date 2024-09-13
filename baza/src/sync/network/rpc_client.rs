use std::{str::FromStr, sync::Arc, time::Duration};

use anyhow::{bail, Context, Result};
use reqwest::{header, Client, Url};

use rs_utils::{log, AuthToken, Download, ResponseVerifier};

use crate::{
    entities::BLOB,
    sync::changeset::{Changeset, ChangesetRequest},
    Baza,
};

use super::auth::{create_shared_network_verifier, ServerCertVerifier, CLIENT_AUTH_TOKEN_HEADER};

#[derive(Clone)]
pub struct BazaClient {
    rpc_server_url: Url,
    client: Client,
    response_verifier: Arc<ServerCertVerifier>,
    downloads_dir: String,
}

impl BazaClient {
    pub fn new(url: &str, baza: &Baza) -> Result<Self> {
        let downloads_dir = baza.get_path_manager().downloads_dir.clone();

        let hmac = create_shared_network_verifier(baza)?;

        let rpc_server_url = Url::from_str(url).context("failed to parse url")?;

        let mut default_headers = header::HeaderMap::new();

        let auth_token = AuthToken::generate(&hmac).serialize();
        default_headers.insert(
            CLIENT_AUTH_TOKEN_HEADER,
            header::HeaderValue::from_str(&auth_token)?,
        );

        let client = Client::builder()
            .https_only(true)
            .danger_accept_invalid_certs(true) // TODO find another way, since this allows expired certificates
            .tls_info(true)
            .default_headers(default_headers)
            .connect_timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build HTTPS client")?;

        let response_verifier = Arc::new(ServerCertVerifier::new(hmac));

        Ok(BazaClient {
            rpc_server_url,
            client,
            response_verifier,
            downloads_dir,
        })
    }

    pub async fn download_blob(&self, blob: &BLOB) -> Result<()> {
        log::debug!(
            "Baza Server {}: downloading BLOB {}",
            self.rpc_server_url,
            blob.id
        );

        if blob.exists()? {
            bail!(
                "can't download BLOB: file {} already exists",
                blob.file_path
            );
        }

        let blob_url = self.rpc_server_url.join("/blobs/")?.join(&blob.id)?;

        let mut download = Download::new_with_path(blob_url.as_str(), &self.downloads_dir)?;
        download.keep_completed_file(true);
        download.keep_download_file(true);
        download.use_custom_http_client(self.client.clone());
        download.use_response_verifier(self.response_verifier.clone());

        let download_result = download.start().await?;

        tokio::fs::rename(&download_result.file_path, &blob.file_path)
            .await
            .context("failed to move downloaded blob into blob dir")?;

        log::debug!(
            "Baza Server {}: downloaded BLOB {}",
            self.rpc_server_url,
            blob.id
        );

        Ok(())
    }

    pub async fn fetch_changes(&self, request: &ChangesetRequest) -> Result<Changeset> {
        log::debug!("Baza Server {}: fetching changes", self.rpc_server_url,);

        let response = self
            .client
            .post(self.rpc_server_url.join("/changeset")?)
            .json(request)
            .send()
            .await?;

        self.response_verifier.verify(&response)?;

        let status = response.status();
        let body = response.text().await?;

        if status.is_success() {
            serde_json::from_str(&body).context("failed to parse server response")
        } else {
            bail!("Server responded with error: {}\n{}", status, body);
        }
    }

    pub fn get_url(&self) -> &str {
        self.rpc_server_url.as_str()
    }
}
