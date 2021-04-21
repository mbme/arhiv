use crate::{config::Config, entities::*, schema::SCHEMA};
use anyhow::*;
use chrono::Utc;
use rs_utils::{
    get_file_name,
    log::{debug, info, warn},
};

use self::db::*;
pub use self::db::{DocumentsCount, Filter, FilterMode, ListPage, Matcher, OrderBy};
use self::network_service::NetworkService;
use self::status::Status;

mod backup;
mod db;
mod network_service;
mod status;
mod sync;

pub struct Arhiv {
    pub config: Config,
    pub(crate) db: DB,
}

impl Arhiv {
    pub fn must_open() -> Arhiv {
        Arhiv::open(Config::must_read().0).expect("must be able to open arhiv")
    }

    pub fn open(config: Config) -> Result<Arhiv> {
        let db = DB::open(config.get_root_dir().to_string())?;

        // check if config settings are equal to db settings
        {
            let conn = db.get_connection()?;

            let db_status = conn.get_db_status()?;

            ensure!(
                db_status.db_version == DB::VERSION,
                "db version {} is different from app db version {}",
                db_status.db_version,
                DB::VERSION,
            );
            ensure!(
                db_status.schema_version == SCHEMA.version,
                "db schema version {} is different from app schema version {}",
                db_status.schema_version,
                SCHEMA.version,
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

        info!("Open arhiv in {}", config.get_root_dir());

        Ok(Arhiv { config, db })
    }

    pub fn create(config: Config) -> Result<Arhiv> {
        info!(
            "Initializing {} arhiv in {}",
            if config.is_prime() {
                "prime"
            } else {
                "replica"
            },
            config.get_root_dir()
        );

        let db = DB::create(config.get_root_dir().to_string())?;

        let tx = db.get_tx()?;

        // initial settings
        tx.set_setting(SETTING_ARHIV_ID, config.get_arhiv_id().to_string())?;
        tx.set_setting(SETTING_IS_PRIME, config.is_prime())?;
        tx.set_setting(SETTING_DB_REV, Revision::STAGING)?;
        tx.set_setting(SETTING_SCHEMA_VERSION, SCHEMA.version)?;
        tx.set_setting(SETTING_DB_VERSION, DB::VERSION)?;
        tx.set_setting(SETTING_LAST_SYNC_TIME, chrono::MIN_DATETIME)?;

        tx.commit()?;

        info!("Created arhiv in {}", config.get_root_dir());

        Ok(Arhiv { config, db })
    }

    pub(crate) fn get_network_service(&self) -> Result<NetworkService> {
        let network_service = NetworkService::new(self.config.get_prime_url()?);

        Ok(network_service)
    }

    pub fn get_status(&self) -> Result<Status> {
        let root_dir = self.config.get_root_dir().to_string();
        let debug_mode = cfg!(not(feature = "production-mode"));

        let conn = self.db.get_connection()?;

        let db_status = conn.get_db_status()?;
        let documents_count = conn.count_documents()?;
        let last_update_time = conn.get_last_update_time()?;

        Ok(Status {
            db_status,
            last_update_time,
            debug_mode,
            root_dir,
            documents_count,
        })
    }

    pub fn list_documents(&self, filter: Filter) -> Result<ListPage<Document>> {
        let conn = self.db.get_connection()?;

        conn.list_documents(filter)
    }

    pub fn get_document(&self, id: &Id) -> Result<Option<Document>> {
        let conn = self.db.get_connection()?;

        conn.get_document(id)
    }

    pub fn stage_document(&self, updated_document: Document) -> Result<()> {
        debug!("Staging document {}", &updated_document.id);

        ensure!(
            !Attachment::is_attachment(&updated_document),
            "attachments must not be modified manually"
        );

        ensure!(
            !updated_document.is_tombstone(),
            "deleted documents must not be updated"
        );

        let tx = self.db.get_tx()?;

        let mut document = {
            if let Some(mut document) = tx.get_document(&updated_document.id)? {
                debug!("Updating existing document {}", &updated_document.id);

                document.rev = Revision::STAGING; // make sure document rev is Staging
                document.updated_at = Utc::now();
                document.data = updated_document.data;

                document
            } else {
                debug!("Creating new document {}", &updated_document.id);

                let mut new_document =
                    Document::new(updated_document.document_type, updated_document.data);
                new_document.id = updated_document.id;

                new_document
            }
        };

        document.archived = updated_document.archived;
        document.refs = updated_document.refs;

        // Validate document references
        for reference in document.refs.iter() {
            // FIXME optimize validating id
            if tx.get_document(reference)?.is_some() {
                continue;
            }
            if reference == &document.id {
                warn!("Document {} references itself, ignoring ref", &document.id);
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

        info!("saved document {}", document);

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

        document.delete();
        // attachment data will be removed during sync

        tx.put_document(&document)?;

        tx.commit()?;

        info!("deleted document {}", document);

        Ok(())
    }

    pub fn add_attachment(&self, file_path: &str, copy: bool) -> Result<Attachment> {
        debug!("Staging attachment {}", &file_path);

        let mut tx = self.db.get_tx()?;

        let hash = tx.add_attachment_data(file_path, copy)?;
        let file_name = get_file_name(file_path).to_string();

        let attachment = Attachment::new(file_name, hash);

        tx.put_document(&attachment)?;

        tx.commit()?;

        info!("Created attachment {} from {}", &attachment.id, file_path);

        Ok(attachment)
    }

    pub fn update_attachment_data(&self, _id: &Id, _file_path: &str) -> Result<Attachment> {
        unimplemented!();
    }

    pub(crate) fn get_attachment_data(&self, hash: BLOBHash) -> Result<AttachmentData> {
        Ok(self.db.get_connection()?.get_attachment_data(hash))
    }

    pub(crate) fn get_attachment_data_by_id(&self, id: &Id) -> Result<AttachmentData> {
        let attachment = self.get_attachment(id)?;

        let hash = attachment.get_hash();

        let attachment_data = self.get_attachment_data(hash)?;

        Ok(attachment_data)
    }

    fn get_attachment(&self, id: &Id) -> Result<Attachment> {
        let document = self
            .get_document(&id)?
            .ok_or(anyhow!("unknown attachment {}", id))?;

        let attachment = Attachment::from(document)?;

        Ok(attachment)
    }
}
