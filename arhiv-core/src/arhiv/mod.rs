use anyhow::*;
use chrono::Utc;

use self::db::*;
pub use self::db::{
    apply_migrations, DocumentsCount, Filter, FilterMode, ListPage, Matcher, OrderBy,
};
use self::network_service::NetworkService;
use self::status::Status;
use crate::config::Config;
use crate::definitions::get_definitions;
use crate::entities::*;
use crate::schema::DataSchema;
use rs_utils::log;

mod backup;
mod db;
mod network_service;
mod status;
mod sync;

pub struct Arhiv {
    pub config: Config,
    pub(crate) db: DB,
    pub schema: DataSchema,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open_with_config(Config::must_read().0).expect("must be able to open arhiv")
    }

    pub fn open() -> Result<Arhiv> {
        let config = Config::read()?.0;

        Arhiv::open_with_config(config)
    }

    pub fn open_with_config(config: Config) -> Result<Arhiv> {
        let mut schema = DataSchema::new();
        schema.modules.append(&mut get_definitions());

        let mut db = DB::open(config.arhiv_root.to_string())?;
        db.with_schema_search(schema.clone());

        // check app and db version
        {
            let conn = db.get_connection()?;

            let db_version = conn.get_setting(SETTING_DB_VERSION)?;

            ensure!(
                db_version == DB::VERSION,
                "db version {} is different from app db version {}",
                db_version,
                DB::VERSION,
            );
        }

        log::debug!("Open arhiv in {}", config.arhiv_root);

        Ok(Arhiv { config, db, schema })
    }

    pub fn create(config: Config, arhiv_id: String, prime: bool) -> Result<Arhiv> {
        log::info!(
            "Initializing {} arhiv '{}' in {}",
            if prime { "prime" } else { "replica" },
            arhiv_id,
            config.arhiv_root
        );

        let mut schema = DataSchema::new();
        schema.modules.append(&mut get_definitions());

        let mut db = DB::create(config.arhiv_root.to_string())?;
        db.with_schema_search(schema.clone());

        let tx = db.get_tx()?;

        // initial settings
        tx.set_setting(SETTING_ARHIV_ID, arhiv_id)?;
        tx.set_setting(SETTING_IS_PRIME, prime)?;
        tx.set_setting(SETTING_DB_VERSION, DB::VERSION)?;
        tx.set_setting(SETTING_LAST_SYNC_TIME, chrono::MIN_DATETIME)?;

        tx.commit()?;

        log::info!("Created arhiv in {}", config.arhiv_root);

        Ok(Arhiv { config, db, schema })
    }

    pub(crate) fn get_network_service(&self) -> Result<NetworkService> {
        let prime_url = &self.config.prime_url;

        ensure!(!prime_url.is_empty(), "config.prime_url is not set");

        let network_service = NetworkService::new(prime_url);

        Ok(network_service)
    }

    pub fn get_status(&self) -> Result<Status> {
        let root_dir = self.config.arhiv_root.to_string();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let conn = self.db.get_connection()?;

        let db_status = conn.get_db_status()?;
        let documents_count = conn.count_documents()?;
        let conflicts_count = conn.count_conflicts()?;
        let last_update_time = conn.get_last_update_time()?;

        Ok(Status {
            db_status,
            last_update_time,
            debug_mode,
            root_dir,
            documents_count,
            conflicts_count,
        })
    }

    pub fn is_prime(&self) -> Result<bool> {
        let conn = self.db.get_connection()?;

        conn.get_setting(SETTING_IS_PRIME)
    }

    pub fn list_documents(&self, filter: Filter) -> Result<ListPage<Document>> {
        let conn = self.db.get_connection()?;

        conn.list_documents(filter)
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.db.get_connection()?;

        conn.get_document(id)
    }

    pub fn stage_document(&self, mut updated_document: Document) -> Result<()> {
        log::debug!("Staging document {}", &updated_document.id);

        ensure!(
            !Attachment::is_attachment(&updated_document),
            "attachments must not be modified manually"
        );

        ensure!(
            !updated_document.is_tombstone(),
            "deleted documents must not be updated"
        );

        self.schema.update_refs(&mut updated_document)?;

        let tx = self.db.get_tx()?;

        let mut document = {
            if let Some(mut document) = tx.get_document(&updated_document.id)? {
                log::debug!("Updating existing document {}", &updated_document.id);

                if document.rev != Revision::STAGING {
                    // we're going to modify committed document
                    // so we need to save its revision as prev_rev of the new document
                    document.prev_rev = document.rev;
                }

                document.rev = Revision::STAGING;
                document.updated_at = Utc::now();
                document.data = updated_document.data;

                document
            } else {
                log::debug!("Creating new document {}", &updated_document.id);

                let mut new_document =
                    Document::new_with_data(updated_document.document_type, updated_document.data);
                new_document.id = updated_document.id;

                new_document
            }
        };

        document.archived = updated_document.archived;
        document.refs = updated_document.refs;
        document.snapshot_id = SnapshotId::new();

        // Validate document references
        for reference in document.refs.iter() {
            // FIXME optimize validating id
            if tx.get_document(reference)?.is_some() {
                continue;
            }

            if reference == &document.id {
                log::warn!("Document {} references itself, ignoring ref", &document.id);
                continue;
            }

            bail!(
                "Document {} reference unknown entity {}",
                &document.id,
                reference
            );
        }

        tx.put_document(&document)?;

        tx.commit()?;

        log::info!("saved document {}", document);

        Ok(())
    }

    pub fn delete_document(&self, id: &Id) -> Result<()> {
        let mut document = self
            .get_document(id)?
            .ok_or(anyhow!("can't find document {}", &id))?;

        ensure!(
            !document.is_tombstone(),
            "deleted documents must not be updated"
        );

        let tx = self.db.get_tx()?;

        if document.rev != Revision::STAGING {
            // we're going to modify committed document
            // so we need to save its revision as prev_rev of the new document
            document.prev_rev = document.rev;
        }

        document.document_type = TOMBSTONE_TYPE.to_string();
        document.rev = Revision::STAGING;
        document.snapshot_id = SnapshotId::new();
        document.refs.clear();
        document.archived = true;
        document.data = DocumentData::new();
        document.updated_at = Utc::now();

        tx.put_document(&document)?;

        // attachment data will be removed during sync

        tx.commit()?;

        log::info!("deleted document {}", document);

        Ok(())
    }

    pub fn archive_document(&self, id: &Id, archive: bool) -> Result<()> {
        let mut document = self
            .get_document(id)?
            .ok_or(anyhow!("can't find document {}", &id))?;

        if document.archived == archive {
            log::warn!(
                "document {} is already {}archived",
                document,
                if archive { "" } else { "un" }
            );

            return Ok(());
        }

        document.archived = archive;

        self.stage_document(document)
    }

    pub fn add_attachment(&self, file_path: &str, copy: bool) -> Result<Attachment> {
        log::debug!("Staging attachment {}", &file_path);

        let attachment = Attachment::new(file_path)?;

        let mut tx = self.db.get_tx()?;

        tx.add_attachment_data(&attachment.id, file_path, copy)?;
        tx.put_document(&attachment)?;

        tx.commit()?;

        log::info!("Created attachment {} from {}", &attachment.id, file_path);

        Ok(attachment)
    }

    pub fn update_attachment_data(&self, _id: &Id, _file_path: &str) -> Result<Attachment> {
        unimplemented!();
    }

    pub fn get_attachment_data(&self, id: &Id) -> Result<AttachmentData> {
        Ok(self.db.get_connection()?.get_attachment_data(id))
    }
}
