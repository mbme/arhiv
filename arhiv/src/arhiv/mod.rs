use self::storage::{get_rev, put_document, Storage};
use crate::config::Config;
use crate::entities::*;
use anyhow::*;

mod storage;

pub struct Arhiv {
    storage: Storage,
    pub config: Config,
    pub prime: bool,
}

impl Arhiv {
    pub fn open(config: Config, prime: bool) -> Result<Arhiv> {
        let root_dir = &config.arhiv_root.clone();

        Ok(Arhiv {
            config,
            storage: Storage::open(root_dir)?,
            prime,
        })
    }

    pub fn create(config: Config) -> Result<()> {
        let root_dir = &config.arhiv_root.clone();

        Storage::create(root_dir)?;

        Ok(())
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

    pub fn get_attachment_data_path(&self, id: &Id) -> Result<Option<String>> {
        unimplemented!();
    }

    pub fn save_attachment(&self, file: &str, move_file: bool) -> Attachment {
        unimplemented!();
    }

    pub fn sync(&self) -> Result<()> {
        unimplemented!();
    }

    pub fn apply_changeset(&self, changeset: Changeset) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;

        let rev = get_rev(&tx)?;

        if changeset.replica_rev > rev {
            return Err(anyhow!(
                "replica_rev {} is greater than prime rev {}",
                changeset.replica_rev,
                rev
            ));
        }

        if changeset.is_empty() {
            return Ok(());
        }

        let new_rev = rev + 1;

        for mut document in changeset.documents {
            // FIXME merge documents
            document.rev = new_rev;
            put_document(&tx, &document)?;
        }

        // for mut attachment in changeset.attachments {
        //     attachment.rev = new_rev;
        //     if self.storage.has_attachment_data(&attachment.id) {
        //         self.storage.add_attachment(&attachment)?;
        //     } else {
        //         return Err(anyhow!("Got attachment {} without a file", attachment.id));
        //     }
        // }

        tx.commit()?;

        Ok(())
    }

    pub fn get_changes(&self, replica_rev: Revision) -> Result<ChangesetResponse> {
        unimplemented!()
    }
}
