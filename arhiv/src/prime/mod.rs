use crate::entities::*;
use anyhow::*;
use config::PrimeConfig;
use std::collections::HashMap;
use storage::Storage;

mod config;
mod state;
mod storage;

pub struct Prime {
    storage: Storage,
    config: PrimeConfig,
}

impl Prime {
    pub fn open(config: PrimeConfig) -> Prime {
        let root_dir = &config.arhiv_root.clone();
        Prime {
            config,
            storage: Storage::open(root_dir).expect("storage must exist"),
        }
    }

    pub fn create(config: PrimeConfig) -> Result<Prime> {
        let root_dir = &config.arhiv_root.clone();
        Ok(Prime {
            config,
            storage: Storage::create(root_dir)?,
        })
    }

    fn get_changeset_response(&self, replica_rev: Revision) -> ChangesetResponse {
        let documents = self.storage.get_documents(replica_rev + 1);
        let attachments = self.storage.get_attachments(replica_rev + 1);

        ChangesetResponse {
            primary_rev: self.storage.get_rev(),
            replica_rev,
            documents,
            attachments,
        }
    }

    pub fn exchange(
        &self,
        changeset: Changeset,
        files: HashMap<String, String>,
    ) -> Result<ChangesetResponse> {
        let rev = self.storage.get_rev();
        if changeset.replica_rev > rev {
            return Err(anyhow!(
                "replica_rev {} is greater than prime rev {}",
                changeset.replica_rev,
                rev
            ));
        }

        if changeset.is_empty() {
            return Ok(self.get_changeset_response(changeset.replica_rev));
        }

        let new_rev = rev + 1;

        for mut document in changeset.documents {
            document.rev = new_rev;
            self.storage.put_document(&document)?;
        }

        for mut attachment in changeset.attachments {
            attachment.rev = new_rev;
            if let Some(file_path) = files.get(&attachment.id) {
                self.storage.add_attachment(&attachment)?;
                self.storage
                    .add_attachment_data(&attachment.id, file_path, true)?;
            } else {
                return Err(anyhow!("Got attachment {} without a file", attachment.id));
            }
        }

        self.storage.set_rev(new_rev);

        Ok(self.get_changeset_response(changeset.replica_rev))
    }
}
