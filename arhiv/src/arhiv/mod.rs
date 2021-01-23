use crate::{config::Config, data_service::DataService, schema::DataSchema};
use crate::{entities::*, replica::NetworkService};
use crate::{path_manager::PathManager, storage::*};
use anyhow::*;
use chrono::Utc;
use rs_utils::{ensure_file_exists, get_file_hash_sha256, FsTransaction};
use status::Status;

mod status;
mod sync;
pub mod test_arhiv;

pub struct Arhiv {
    pub schema: DataSchema,
    pub(crate) storage: Storage,
    pub(crate) config: Config,
    pub(crate) data_service: DataService,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open(Config::must_read().0).expect("must be able to open arhiv")
    }

    pub fn open(config: Config) -> Result<Arhiv> {
        let path_manager = PathManager::new(config.get_root_dir());
        path_manager.assert_dirs_exist()?;
        path_manager.assert_db_file_exists()?;

        let storage = Storage::open(path_manager.get_db_file())?;

        let schema = DataSchema::new();
        let data_service = DataService::new(path_manager);

        // check if config settings are equal to db settings
        {
            let conn = storage.get_connection()?;

            let db_status = conn.get_db_status()?;

            ensure!(
                db_status.schema_version == schema.version,
                "db version {} is different from app version {}",
                db_status.schema_version,
                schema.version
            );
            ensure!(
                db_status.arhiv_id == config.get_arhiv_id(),
                "db arhiv_id {} is different from config.arhiv_id {}",
                db_status.arhiv_id,
                config.get_arhiv_id(),
            );
            ensure!(
                db_status.is_prime == config.is_prime(),
                "db is_prime {} is different from config {}",
                db_status.is_prime,
                config.is_prime(),
            );
        }

        log::info!("Open arhiv in {}", config.get_root_dir());

        Ok(Arhiv {
            schema,
            config,
            storage,
            data_service,
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

        let path_manager = PathManager::new(config.get_root_dir());
        path_manager.create_dirs()?;

        let storage = Storage::create(path_manager.get_db_file())?;

        let schema = DataSchema::new();
        let data_service = DataService::new(path_manager);

        let mut conn = storage.get_writable_connection()?;
        let tx = conn.get_tx()?;

        // initial settings
        tx.put_db_status(DbStatus {
            arhiv_id: config.get_arhiv_id().to_string(),
            is_prime: config.is_prime(),
            db_rev: 0.into(),
            schema_version: schema.version,
            last_sync_time: chrono::MIN_DATETIME,
        })?;

        tx.commit()?;

        log::info!("Created arhiv in {}", config.get_root_dir());

        Ok(Arhiv {
            schema,
            config,
            storage,
            data_service,
        })
    }

    fn get_network_service(&self) -> Result<NetworkService> {
        let network_service = NetworkService::new(self.config.get_prime_url()?, &self.data_service);

        Ok(network_service)
    }

    pub fn get_status(&self) -> Result<Status> {
        let root_dir = self.config.get_root_dir().to_string();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let conn = self.storage.get_connection()?;

        let db_status = conn.get_db_status()?;
        let (committed_documents, staged_documents) = conn.count_documents()?;
        let last_update_time = conn.get_last_update_time()?;

        Ok(Status {
            db_status,
            last_update_time,
            debug_mode,
            root_dir,
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

            let path = self.data_service.get_staged_file_path(&new_attachment.id);

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

    pub fn get_attachment_location(&self, id: &Id) -> Result<AttachmentLocation> {
        let attachment = self
            .get_document(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;

        ensure!(
            attachment.is_attachment(),
            "document {} isn't an attachment",
            id,
        );

        if attachment.rev.is_staged() {
            if !self.data_service.staged_file_exists(&id)? {
                bail!("can't find staged file for attachment {}", id);
            }

            return Ok(AttachmentLocation::File(
                self.data_service.get_staged_file_path(&id),
            ));
        }

        if self.data_service.committed_file_exists(&id)? {
            return Ok(AttachmentLocation::File(
                self.data_service.get_committed_file_path(&id),
            ));
        }

        let url = self.get_network_service()?.get_attachment_data_url(id);

        Ok(AttachmentLocation::Url(url))
    }
}
