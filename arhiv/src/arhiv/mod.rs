use self::storage::*;
use crate::config::Config;
use crate::entities::*;
use crate::utils::ensure_exists;
use anyhow::*;
use std::path::Path;

mod storage;

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
        get_rev(&self.storage.get_connection()?)
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

        get_documents(&conn, self.get_mode())
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.storage.get_connection()?;

        get_document(&conn, id, self.get_mode())
    }

    pub fn save_document(&self, mut document: Document) -> Result<()> {
        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;

        if self.config.prime {
            let rev = get_rev(&tx)?;
            document.rev = rev + 1;
        } else {
            document.rev = 0;
        }

        put_document(&tx, &document)?;

        tx.commit()?;

        Ok(())
    }

    pub fn list_attachments(&self) -> Result<Vec<Attachment>> {
        let conn = self.storage.get_connection()?;

        get_attachments(&conn, self.get_mode())
    }

    pub fn get_attachment(&self, id: &Id) -> Result<Option<Attachment>> {
        let conn = self.storage.get_connection()?;

        get_attachment(&conn, id, self.get_mode())
    }

    pub fn get_attachment_data_path(&self, id: &Id) -> Result<Option<String>> {
        unimplemented!();
    }

    pub fn save_attachment(&self, file: &str, move_file: bool) -> Result<Attachment> {
        ensure_exists(file, false).expect("new attachment file must exist");

        let mut attachment = Attachment::new(
            Path::new(file)
                .file_name()
                .expect("file must have name")
                .to_str()
                .unwrap(),
        );

        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;

        if self.config.prime {
            let rev = get_rev(&tx)?;
            attachment.rev = rev + 1;
        }

        put_attachment(&tx, &attachment)?;
        // FIXME save attachment data

        tx.commit()?;

        Ok(attachment)
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

        for mut attachment in changeset.attachments {
            attachment.rev = new_rev;
            put_attachment(&tx, &attachment)?;
            // FIXME save data
            // if self.storage.has_attachment_data(&attachment.id) {
            //     self.storage.add_attachment(&attachment)?;
            // } else {
            //     return Err(anyhow!("Got attachment {} without a file", attachment.id));
            // }
        }

        tx.commit()?;

        Ok(())
    }

    pub fn get_changes(&self, replica_rev: Revision) -> Result<ChangesetResponse> {
        unimplemented!()
    }
}
