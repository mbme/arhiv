use crate::entities::*;
use crate::storage::*;
use crate::{config::Config, schema::DataSchema};
use anyhow::*;
use chrono::Utc;
use rs_utils::{ensure_file_exists, get_file_hash_sha256, FsTransaction};
use serde::{Deserialize, Serialize};
pub use server::start_server;
use status::Status;
use std::sync::Arc;

mod server;
mod status;
mod sync;

pub struct Arhiv {
    pub schema: DataSchema,
    storage: Storage,
    config: Arc<Config>,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open(Config::must_read().0).expect("must be able to open arhiv")
    }

    pub fn open(config: Config) -> Result<Arhiv> {
        let config = Arc::new(config);

        let storage = Storage::open(config.clone())?;
        let schema = DataSchema::new();

        // check if config settings are equal to db settings
        {
            let conn = storage.get_connection()?;

            let schema_version = conn.get_schema_version()?;
            let arhiv_id = conn.get_arhiv_id()?;
            let is_prime = conn.is_prime()?;

            ensure!(
                schema_version == schema.version,
                "db version {} is different from app version {}",
                schema_version,
                schema.version
            );
            ensure!(
                arhiv_id == config.get_arhiv_id(),
                "db arhiv_id {} is different from config.arhiv_id {}",
                arhiv_id,
                config.get_arhiv_id(),
            );
            ensure!(
                is_prime == config.is_prime(),
                "db is_prime {} is different from config {}",
                is_prime,
                config.is_prime(),
            );
        }

        log::info!("Open arhiv in {}", config.get_root_dir());

        Ok(Arhiv {
            schema,
            config,
            storage,
        })
    }

    pub fn create(config: Config) -> Result<Arhiv> {
        log::info!(
            "Initializing {} arhiv in {}",
            if config.is_prime() {
                "prime"
            } else {
                "replica"
            },
            config.get_root_dir()
        );

        let config = Arc::new(config);

        let storage = Storage::create(config.clone())?;

        let schema = DataSchema::new();

        let mut conn = storage.get_writable_connection()?;
        let tx = conn.get_tx()?;

        // initial settings
        tx.set_setting(DbSettings::ArhivId, config.get_arhiv_id().to_string())?;
        tx.set_setting(DbSettings::IsPrime, config.is_prime().to_string())?;
        tx.set_setting(DbSettings::DbRevision, 0.to_string())?;
        tx.set_setting(DbSettings::SchemaVersion, schema.version.to_string())?;

        tx.commit()?;

        log::info!("Created arhiv in {}", config.get_root_dir());

        Ok(Arhiv {
            schema,
            config,
            storage,
        })
    }

    pub fn get_root_dir(&self) -> &str {
        self.config.get_root_dir()
    }

    pub fn get_status(&self) -> Result<Status> {
        let root_dir = self.get_root_dir().to_string();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let conn = self.storage.get_connection()?;

        let rev = conn.get_rev()?;
        let arhiv_id = conn.get_arhiv_id()?;
        let (committed_documents, staged_documents) = conn.count_documents()?;
        let is_prime = conn.is_prime()?;
        let last_update_time = conn.get_last_update_time()?;

        Ok(Status {
            arhiv_id,
            root_dir,
            rev,
            is_prime,
            debug_mode,
            last_update_time,
            committed_documents,
            staged_documents,
        })
    }

    pub fn has_staged_changes(&self) -> Result<bool> {
        let (_, staged_documents) = self.storage.get_connection()?.count_documents()?;

        Ok(staged_documents > 0)
    }

    pub fn list_documents(&self, filter: Filter) -> Result<ListPage<Document>> {
        let conn = self.storage.get_connection()?;

        conn.list_documents(filter)
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.storage.get_connection()?;

        conn.get_document(id)
    }

    pub fn stage_document(
        &self,
        updated_document: Document,
        new_attachments: Vec<AttachmentSource>,
    ) -> Result<()> {
        log::debug!(
            "Staging document {} with {} new attachments",
            &updated_document.id,
            new_attachments.len()
        );

        let mut conn = self.storage.get_writable_connection()?;
        let conn = conn.get_tx()?;

        let mut fs_tx = FsTransaction::new();

        // FIXME optimize this
        let mut document = {
            if let Some(mut document) = conn.get_document(&updated_document.id)? {
                log::debug!("Updating existing document {}", &updated_document.id);

                document.rev = Revision::STAGING; // make sure document rev is Staging
                document.updated_at = Utc::now();
                document.data = updated_document.data;

                document
            } else {
                if updated_document.is_attachment() {
                    bail!("attachments must not be created manually");
                }

                log::debug!("Creating new document {}", &updated_document.id);

                let mut new_document =
                    Document::new(updated_document.document_type, updated_document.data);
                new_document.id = updated_document.id;

                new_document
            }
        };

        document.archived = updated_document.archived;
        document.refs = updated_document.refs;
        if document.is_attachment() && !document.refs.is_empty() {
            bail!("attachment refs must be empty")
        }

        // Validate document references
        let new_attachments_ids: Vec<&Id> = new_attachments.iter().map(|item| &item.id).collect();
        for reference in document.refs.iter() {
            // FIXME optimize validating id
            if conn.get_document(reference)?.is_some() {
                continue;
            }
            if reference == &document.id {
                log::warn!("Document {} references itself, ignoring ref", &document.id);
                continue;
            }
            if new_attachments_ids.contains(&reference) {
                continue;
            }

            bail!(
                "Document {} reference unknown entity {}",
                &document.id,
                reference
            );
        }

        // Stage new attachments
        for new_attachment in new_attachments {
            if !document.refs.contains(&new_attachment.id) {
                log::warn!(
                    "Document {} new attachment is unused, ignoring: {}",
                    &document.id,
                    &new_attachment
                );
                continue;
            }

            if conn.get_document(&new_attachment.id)?.is_some() {
                log::warn!(
                    "Document {} new attachment already exists, ignoring: {}",
                    &document.id,
                    &new_attachment
                );
                continue;
            }

            let path = self
                .storage
                .get_attachment_data(new_attachment.id.clone())
                .get_staged_file_path();

            let source_path = new_attachment.file_path.to_string();
            if new_attachment.copy {
                fs_tx.copy_file(source_path.clone(), path)?;
            } else {
                fs_tx.hard_link_file(source_path.clone(), path)?;
            }

            let attachment = self.create_attachment(new_attachment)?;
            conn.put_document(&attachment)?;

            log::info!("staged new attachment {}: {}", attachment, source_path);
        }

        conn.put_document(&document)?;

        conn.commit()?;
        fs_tx.commit();

        // FIXME remove unused staged attachments

        log::debug!("staged document {}", &document);

        Ok(())
    }

    fn create_attachment(&self, source: AttachmentSource) -> Result<Document> {
        use serde_json::Map;

        ensure_file_exists(&source.file_path)?;

        let mut initial_values = Map::new();
        let hash = get_file_hash_sha256(&source.file_path)?;
        initial_values.insert("hash".to_string(), hash.into());
        initial_values.insert("filename".to_string(), source.filename.into());

        let data = self
            .schema
            .create_with_data(ATTACHMENT_TYPE.to_string(), initial_values)?;

        log::info!(
            "Created attachment {} from {}",
            &source.id,
            &source.file_path
        );

        Ok(Document {
            id: source.id.clone(),
            ..Document::new(ATTACHMENT_TYPE.to_string(), data.into())
        })
    }

    pub fn get_attachment_data(&self, id: &Id) -> AttachmentData {
        self.storage.get_attachment_data(id.clone())
    }

    pub fn get_attachment_location(&self, id: &Id) -> Result<AttachmentLocation> {
        let attachment = self
            .get_document(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;

        if !attachment.is_attachment() {
            bail!("document {} isn't an attachment", id);
        }

        let data = self.storage.get_attachment_data(id.clone());

        if attachment.rev.is_staged() {
            if !data.staged_file_exists()? {
                bail!("can't find staged file for attachment {}", id);
            }

            return Ok(AttachmentLocation::File(data.get_staged_file_path()));
        }

        if data.committed_file_exists()? {
            return Ok(AttachmentLocation::File(data.get_committed_file_path()));
        }

        Ok(AttachmentLocation::Url(data.get_url()?))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AttachmentLocation {
    Url(String),
    File(String),
}

#[cfg(test)]
impl Drop for Arhiv {
    // Remove temporary Arhiv in tests
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.get_root_dir()).expect("must be able to remove arhiv");
    }
}
