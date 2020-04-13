use self::storage::{get_rev, Storage};
use crate::config::Config;
use crate::entities::*;
use anyhow::*;

mod storage;

pub struct Arhiv {
    pub storage: Storage,
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

    pub fn create(config: Config) -> Result<Arhiv> {
        let root_dir = &config.arhiv_root.clone();

        Ok(Arhiv {
            config,
            storage: Storage::create(root_dir)?,
        })
    }

    pub fn get_rev(&self) -> Result<Revision> {
        get_rev(&self.storage.get_connection()?)
    }

    pub fn list_documents(&self) -> Result<Vec<Document>> {
        let conn = self.storage.get_connection();

        unimplemented!();
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        unimplemented!();
    }

    pub fn save_document(&self, mut document: Document) {
        unimplemented!();
    }

    pub fn list_attachments(&self) -> Result<Vec<Attachment>> {
        unimplemented!();
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        unimplemented!();
    }

    pub fn save_attachment(&self, file: &str, move_file: bool) -> Attachment {
        unimplemented!();
    }

    pub fn sync(&self) -> Result<()> {
        unimplemented!();
    }
}
