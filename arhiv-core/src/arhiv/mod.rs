use anyhow::*;
use chrono::Utc;

use self::db::*;
pub use self::db::{
    apply_migrations, AttachmentData, Condition, DocumentsCount, Filter, FilterMode, ListPage,
    OrderBy,
};
use self::status::Status;
use self::validator::Validator;
use crate::config::Config;
use crate::definitions::get_schema;
use crate::entities::*;
use crate::schema::DataSchema;
use rs_utils::log;

mod backup;
mod db;
mod status;
mod sync;
mod validator;

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
        let schema = get_schema();

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

        let schema = get_schema();

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

    pub fn get_document(&self, id: impl Into<Id>) -> Result<Option<Document>> {
        let conn = self.db.get_connection()?;

        conn.get_document(&id.into())
    }

    pub fn stage_document(&self, document: &mut Document) -> Result<()> {
        log::debug!("Staging document {}", &document.id);

        ensure!(
            !Attachment::is_attachment(&document),
            "attachments must not be modified manually"
        );

        ensure!(
            !document.is_tombstone(),
            "deleted documents must not be updated"
        );

        let data_description = self.schema.get_data_description(&document.document_type)?;

        document.refs = data_description.extract_refs(&document.data)?;

        Validator::new(&self)
            .validate(document)
            .context("document validation failed")?;

        let tx = self.db.get_tx()?;

        if let Some(prev_document) = tx.get_document(&document.id)? {
            log::debug!("Updating existing document {}", &document.id);

            document.rev = Revision::STAGING;

            if prev_document.rev == Revision::STAGING {
                document.prev_rev = prev_document.prev_rev;
            } else {
                // we're going to modify committed document
                // so we need to save its revision as prev_rev of the new document
                document.prev_rev = prev_document.rev;
            }

            document.snapshot_id = SnapshotId::new();

            document.document_type = prev_document.document_type;

            document.created_at = prev_document.created_at;
            document.updated_at = Utc::now();
        } else {
            log::debug!("Creating new document {}", &document.id);

            document.rev = Revision::STAGING;
            document.prev_rev = Revision::STAGING;

            document.snapshot_id = SnapshotId::new();

            let now = Utc::now();
            document.created_at = now;
            document.updated_at = now;
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

        self.stage_document(&mut document)
    }

    pub fn add_attachment(&self, file_path: &str, move_file: bool) -> Result<Attachment> {
        log::debug!(
            "Staging attachment {}; move file: {}",
            &file_path,
            move_file
        );

        let attachment = Attachment::new(file_path)?;

        let mut tx = self.db.get_tx()?;

        tx.add_attachment_data(&attachment.id, file_path, move_file)?;
        tx.put_document(&attachment)?;

        tx.commit()?;

        log::info!("Created attachment {} from {}", &attachment.id, file_path);

        Ok(attachment)
    }

    pub fn get_attachment_data(&self, id: &Id) -> Result<AttachmentData> {
        Ok(self.db.get_connection()?.get_attachment_data(id))
    }
}
