use self::storage::*;
use crate::config::Config;
use crate::entities::*;
use crate::utils::{ensure_exists, FsTransaction};
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
        ensure_exists(file, false).expect("new attachment file must exist");

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

    pub fn get_changeset(&self) -> Result<Changeset> {
        let mut conn = self.storage.get_writable_connection()?;
        let tx = conn.transaction()?;

        let rev = get_rev(&tx)?;
        let documents = get_staged_documents(&tx)?;
        let attachments = get_staged_attachments(&tx)?;
        let new_attachments = attachments
            .into_iter()
            .map(|attachment| NewAttachment {
                file_path: self.storage.get_attachment_file_path(&attachment.id),
                attachment,
            })
            .collect();

        Ok(Changeset {
            replica_rev: rev,
            documents,
            new_attachments,
        })
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

        let mut fs_tx = FsTransaction::new();
        for new_attachment in changeset.new_attachments {
            // save attachment
            let mut attachment = new_attachment.attachment;
            attachment.rev = new_rev;
            put_attachment(&tx, &attachment)?;

            // save attachment file
            fs_tx.move_file(
                new_attachment.file_path,
                self.storage.get_attachment_file_path(&attachment.id),
            )?;
        }

        tx.commit()?;
        fs_tx.commit();

        Ok(())
    }

    pub fn get_changes(&self, replica_rev: Revision) -> Result<ChangesetResponse> {
        let conn = self.storage.get_connection()?;

        let documents = get_commited_documents_with_rev(&conn, replica_rev + 1)?;
        let attachments = get_commited_attachments_with_rev(&conn, replica_rev + 1)?;

        Ok(ChangesetResponse {
            primary_rev: get_rev(&conn)?,
            replica_rev,
            documents,
            attachments,
        })
    }
}
