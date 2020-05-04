use self::storage::*;
use crate::config::Config;
use crate::entities::*;
use crate::utils::{ensure_file_exists, FsTransaction};
use anyhow::*;
use std::path::Path;

mod storage;
mod sync;

pub struct Arhiv {
    storage: Storage,
    pub config: Config,
}

impl Arhiv {
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

    pub fn list_documents(&self) -> Result<Vec<Document>> {
        let conn = self.storage.get_connection()?;

        if self.config.prime {
            get_commited_documents(&conn)
        } else {
            get_all_documents(&conn)
        }
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
        fs_tx.move_file(
            file.to_string(),
            self.storage.get_attachment_file_path(&attachment.id),
        )?;

        tx.commit()?;
        fs_tx.commit();

        Ok(attachment)
    }
}
