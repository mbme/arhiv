use self::storage::*;
use crate::config::Config;
use crate::entities::*;
use crate::utils::{ensure_file_exists, file_exists, FsTransaction};
use anyhow::*;
use serde::{Deserialize, Serialize};
use std::path::Path;
pub use storage::QueryFilter;

mod storage;
mod sync;

#[derive(Serialize, Deserialize)]
pub enum AttachmentLocation {
    Url(String),
    File(String),
    Unknown,
}

pub struct Arhiv {
    storage: Storage,
    pub config: Config,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open(Config::must_read()).expect("must be able to open arhiv")
    }

    pub fn open(config: Config) -> Result<Arhiv> {
        let root_dir = &config.arhiv_root.clone();

        Ok(Arhiv {
            config,
            storage: Storage::open(root_dir)?,
        })
    }

    pub fn create(config: Config) -> Result<()> {
        let root_dir = &config.arhiv_root.clone();

        Storage::create(root_dir)?;

        Ok(())
    }

    pub fn get_rev(&self) -> Result<Revision> {
        let conn = self.storage.get_connection()?;

        get_rev(&conn)
    }

    fn get_mode(&self) -> QueryMode {
        if self.config.prime {
            QueryMode::Commited
        } else {
            QueryMode::All
        }
    }

    pub fn list_documents(&self, filter: Option<QueryFilter>) -> Result<Vec<Document>> {
        let conn = self.storage.get_connection()?;

        get_documents(
            &conn,
            if self.config.prime { 1 } else { 0 },
            filter.unwrap_or_default(),
        )
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.storage.get_connection()?;

        get_document(&conn, id, self.get_mode())
    }

    pub fn stage_document(&self, mut document: Document) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;

        // make sure document rev is Staging
        document.rev = 0;

        put_document(&tx, &document)?;

        tx.commit()?;

        log::trace!("staged new document {}", &document);

        Ok(())
    }

    pub fn list_attachments(&self) -> Result<Vec<Attachment>> {
        let conn = self.storage.get_connection()?;

        if self.config.prime {
            get_commited_attachments(&conn)
        } else {
            get_all_attachments(&conn)
        }
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let conn = self.storage.get_connection()?;

        get_attachment(&conn, id, self.get_mode())
    }

    pub fn get_attachment_data_path(&self, id: &Id) -> String {
        self.storage.get_attachment_file_path(id)
    }

    pub fn get_attachment_location(&self, id: &Id) -> Result<AttachmentLocation> {
        let attachment = self.get_attachment(id)?;

        if attachment.is_none() {
            bail!("unknown attachment {}", id);
        }

        let local_file_path = self.get_attachment_data_path(id);

        if file_exists(&local_file_path)? {
            return Ok(AttachmentLocation::File(local_file_path));
        }

        let primary_url = self
            .config
            .primary_url
            .as_ref()
            .ok_or(anyhow!("config.primary_url is missing"))?;

        let url = AttachmentLocation::Url(format!("{}/attachment-data/{}", primary_url, id));

        Ok(url)
    }

    pub fn stage_attachment(&self, file: &str) -> Result<Attachment> {
        ensure_file_exists(file).expect("new attachment file must exist");

        let attachment = Attachment::new(
            Path::new(file)
                .file_name()
                .expect("file must have name")
                .to_str()
                .unwrap(),
        );

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;
        let mut fs_tx = FsTransaction::new();

        put_attachment(&tx, &attachment)?;
        fs_tx.hard_link_file(
            file.to_string(),
            self.storage.get_attachment_file_path(&attachment.id),
        )?;

        tx.commit()?;
        fs_tx.commit();

        log::debug!("staged new attachment {}: {}", attachment, file);

        Ok(attachment)
    }
}
