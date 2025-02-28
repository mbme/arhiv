use std::fmt;

use anyhow::Result;
use serde::Serialize;

use baza::{
    baza2::{BLOBSCount, Baza, DocumentsCount, Locks},
    entities::Revision,
    AutoCommitService, DEV_MODE,
};
use rs_utils::{default_date_time_format, get_crate_version, Timestamp};

use crate::ServerInfo;

#[derive(Serialize)]
pub struct Status<'b> {
    pub app_version: String,
    pub instance_id: String,

    pub storage_version: u8,
    pub data_version: u8,
    pub documents_count: DocumentsCount,
    pub blobs_count: BLOBSCount,
    pub locks: &'b Locks,

    pub db_rev: Revision,
    pub last_update_time: Option<Timestamp>,
    pub dev_mode: bool,
    pub root_dir: String,

    pub auto_commit_delay_in_seconds: u64,

    pub server_port: Option<u16>,
}

impl<'b> Status<'b> {
    pub fn read(baza: &'b Baza) -> Result<Self> {
        let root_dir = baza.get_storage_dir().to_string();

        let info = baza.get_info();

        let instance_id = baza.get_instance_id().to_string();
        let db_rev = baza.get_single_latest_revision().clone();
        let storage_version = info.storage_version;
        let data_version = info.data_version;
        let documents_count = baza.count_documents()?;
        let blobs_count = baza.count_blobs()?;
        let last_update_time = baza.find_last_modification_time();
        let locks = baza.list_document_locks();
        let server_port = ServerInfo::get_server_port(&root_dir)?;
        let auto_commit_delay_in_seconds = AutoCommitService::DEFAULT_AUTO_COMMIT_DELAY.as_secs();

        Ok(Status {
            instance_id,
            app_version: get_crate_version().to_string(),
            storage_version,
            data_version,
            documents_count,
            blobs_count,
            db_rev,
            last_update_time,
            dev_mode: DEV_MODE,
            root_dir,
            locks,
            auto_commit_delay_in_seconds,
            server_port,
        })
    }
}

impl fmt::Display for Status<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dev_mode {
            writeln!(f)?;
            writeln!(
                f,
                "------------------------------ DEV MODE ------------------------------"
            )?;
        }

        writeln!(f)?;

        writeln!(f, "             Root dir: {}", self.root_dir)?;
        writeln!(f, "          Instance id: {}", self.instance_id)?;

        writeln!(f)?;

        writeln!(f, "          App version: {}", self.app_version)?;
        writeln!(f, "      Storage version: {}", self.storage_version)?;
        writeln!(f, "       Schema version: {}", self.data_version)?;
        writeln!(f, "          DB revision: {}", self.db_rev.serialize())?;

        writeln!(f)?;

        writeln!(
            f,
            "      Is modified: {}",
            self.documents_count.count_staged() > 0
        )?;

        writeln!(f)?;

        writeln!(
            f,
            " Last update time: {}",
            self.last_update_time
                .map_or("NEVER".to_string(), default_date_time_format)
        )?;

        writeln!(f)?;
        writeln!(
            f,
            "Auto-commit delay: {} seconds",
            self.auto_commit_delay_in_seconds
        )?;

        writeln!(f)?;
        if let Some(server_port) = self.server_port {
            writeln!(f, "   Local Server: running on port {server_port}")?;
        } else {
            writeln!(f, "   Local Server: not running")?;
        }
        writeln!(f)?;

        writeln!(
            f,
            "        Documents: {} committed, {} staged ({} updated, {} new)",
            self.documents_count.documents_committed,
            self.documents_count.count_staged_documents(),
            self.documents_count.documents_updated,
            self.documents_count.documents_new,
        )?;
        writeln!(
            f,
            " Erased Documents: {} committed, {} staged",
            self.documents_count.erased_documents_committed,
            self.documents_count.erased_documents_staged,
        )?;
        writeln!(
            f, //
            "        Snapshots: {}",
            self.documents_count.snapshots
        )?;

        writeln!(f)?;

        writeln!(
            f,
            "            BLOBs: {} referenced; {} present: {} new, {} in storage",
            self.blobs_count.total_referenced_blobs,
            self.blobs_count.count_present_blobs(),
            self.blobs_count.blobs_staged,
            self.blobs_count.blobs_in_storage,
        )?;

        writeln!(f)?;
        writeln!(f, "            Locks: {}", self.locks.len())?;
        for (id, reason) in self.locks {
            writeln!(f, "   {:>30}: {reason}", id)?;
        }

        if self.documents_count.conflicts_count > 0 {
            writeln!(f)?;
            writeln!(
                f,
                "        WARN:  found {} conflicts",
                self.documents_count.conflicts_count
            )?;
            writeln!(f)?;
        }

        Ok(())
    }
}
