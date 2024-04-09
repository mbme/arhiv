use std::{fs, io::Write, sync::Arc};

use anyhow::{anyhow, ensure, Context, Result};

use baza::{
    sync::{AutoSyncTask, MDNSClientTask, MDNSDiscoveryService, SyncManager},
    AutoCommitService, AutoCommitTask, Baza, BazaOptions, Credentials, DEV_MODE,
};
use rs_utils::{
    file_exists, get_home_dir, log, must_create_file, now, path_exists, SecretBytes, SecretString,
    SelfSignedCertificate,
};

use crate::{config::ArhivConfigExt, definitions::get_standard_schema, Status};

#[derive(Default, Clone)]
pub struct ArhivOptions {
    pub discover_peers: bool,
    pub auto_commit: bool,
    pub file_browser_root_dir: Option<String>,
}

pub struct Arhiv {
    pub baza: Arc<Baza>,
    certificate: Arc<SelfSignedCertificate>,
    auto_commit_task: Option<AutoCommitTask>,
    auto_sync_task: Option<AutoSyncTask>,
    mdns_client_task: Option<MDNSClientTask>,
    sync_manager: Arc<SyncManager>,
    mdns_discovery_service: MDNSDiscoveryService,
    file_browser_root_dir: String,
}

impl Arhiv {
    pub fn exists(root_dir: &str) -> bool {
        path_exists(root_dir)
    }

    pub fn create(root_dir: impl Into<String>, auth: Credentials) -> Result<()> {
        let root_dir = root_dir.into();
        log::debug!("Arhiv root dir: {root_dir}");

        let schema = get_standard_schema();

        Baza::create(BazaOptions { root_dir, schema }, auth)?;
        log::debug!("Created new Arhiv");

        Ok(())
    }

    pub fn open(root_dir: impl Into<String>, options: ArhivOptions) -> Result<Arhiv> {
        let root_dir = root_dir.into();
        log::debug!("Arhiv root dir: {root_dir}");

        let schema = get_standard_schema();

        let certificate = Arhiv::read_or_generate_certificate(&root_dir)?;
        let certificate = Arc::new(certificate);

        let baza_options = BazaOptions { root_dir, schema };

        let baza = Baza::open(baza_options)?;
        let baza = Arc::new(baza);

        let sync_manager = SyncManager::new(baza.clone(), certificate.clone());
        let sync_manager = Arc::new(sync_manager);

        let mdns_discovery_service = MDNSDiscoveryService::new(&baza)?;

        let mut arhiv = Arhiv {
            baza,
            certificate,
            sync_manager,
            auto_commit_task: None,
            auto_sync_task: None,
            mdns_client_task: None,
            mdns_discovery_service,
            file_browser_root_dir: options
                .file_browser_root_dir
                .or_else(get_home_dir)
                .unwrap_or_else(|| "/".to_string()),
        };
        if options.auto_commit {
            arhiv.init_auto_commit_service()?;
        }
        if options.discover_peers {
            arhiv.init_auto_sync_service()?;
            arhiv.init_mdns_client_service()?;
        }

        Ok(arhiv)
    }

    pub(crate) fn start_mdns_server(&self, server_port: u16) -> Result<()> {
        self.mdns_discovery_service.start_mdns_server(server_port)
    }

    fn init_auto_commit_service(&mut self) -> Result<()> {
        let auto_commit_delay = self.baza.get_connection()?.get_auto_commit_delay()?;
        ensure!(
            !auto_commit_delay.is_zero(),
            "Config auto-commit delay must not be zero"
        );

        let service = AutoCommitService::new(self.baza.clone(), auto_commit_delay);
        let task = service.start()?;

        self.auto_commit_task = Some(task);

        Ok(())
    }

    fn init_auto_sync_service(&mut self) -> Result<()> {
        let auto_sync_delay = self.baza.get_connection()?.get_auto_sync_delay()?;
        ensure!(
            !auto_sync_delay.is_zero(),
            "Config auto-sync delay must not be zero"
        );

        let task = self.sync_manager.clone().start_auto_sync(auto_sync_delay)?;

        self.auto_sync_task = Some(task);

        Ok(())
    }

    fn init_mdns_client_service(&mut self) -> Result<()> {
        let task = self
            .mdns_discovery_service
            .start_mdns_client(self.sync_manager.clone())?;
        self.mdns_client_task = Some(task);

        Ok(())
    }

    pub fn get_status(&self) -> Result<Status> {
        let conn = self.baza.get_connection()?;

        Status::read(&conn)
    }

    pub async fn sync(&self) -> Result<bool> {
        self.sync_manager.sync().await
    }

    pub fn has_sync_agents(&self) -> bool {
        self.sync_manager.count_agents() > 0
    }

    pub fn get_file_browser_root_dir(&self) -> &str {
        &self.file_browser_root_dir
    }

    pub fn get_certificate(&self) -> &SelfSignedCertificate {
        &self.certificate
    }

    pub fn stop(&self) {
        if let Some(ref mdns_client_task) = self.mdns_client_task {
            mdns_client_task.abort();
        }

        if let Some(ref auto_commit_task) = self.auto_commit_task {
            auto_commit_task.abort();
        }

        if let Some(ref auto_sync_task) = self.auto_sync_task {
            auto_sync_task.abort();
        }

        std::thread::sleep(std::time::Duration::from_millis(100));

        log::info!("Stopped Arhiv");
    }

    pub(crate) fn read_or_generate_certificate(root_dir: &str) -> Result<SelfSignedCertificate> {
        let cert_path = format!("{root_dir}/certificate.pfx");
        let password = SecretString::new("");

        if file_exists(&cert_path)? {
            let data = fs::read(&cert_path).context("Failed to read certificate file")?;

            let data = SecretBytes::new(data);

            let certificate = SelfSignedCertificate::from_pfx_der(&password, data)?;

            log::info!("Read arhiv certificate from {cert_path}");

            Ok(certificate)
        } else {
            let certificate = generate_certificate()?;

            let friendly_name = if DEV_MODE { "arhiv-dev" } else { "arhiv" };

            let data = certificate.to_pfx_der(&password, friendly_name)?;

            // Save Arhiv's certificate in PKCS#12 format (.pfx). Browsers can use it as a client HTTPS/TLS certificate. Password is empty.
            let mut file = must_create_file(&cert_path)
                .context(anyhow!("Failed to create certificate file {cert_path}"))?;
            file.write_all(data.as_bytes())?;
            file.flush()?;

            log::info!("Wrote arhiv certificate into {cert_path}");

            Ok(certificate)
        }
    }
}

fn generate_certificate() -> Result<SelfSignedCertificate> {
    let timestamp = now();
    let certificate_id = format!("Arhiv {timestamp}");

    SelfSignedCertificate::new_x509(&certificate_id)
}
