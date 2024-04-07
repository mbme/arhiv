use std::fmt;

use anyhow::Result;
use serde::Serialize;

use baza::{entities::Revision, BLOBSCount, BazaConnection, DocumentsCount, Locks, DEBUG_MODE};
use rs_utils::{default_date_time_format, get_crate_version, Timestamp, MIN_TIMESTAMP};

use crate::{ArhivConfigExt, ArhivServer};

#[derive(Serialize)]
pub struct Status {
    pub app_version: String,
    pub instance_id: String,

    pub db_version: u8,
    pub data_version: u8,
    pub computed_data_version: u8,
    pub documents_count: DocumentsCount,
    pub blobs_count: BLOBSCount,
    pub conflicts_count: usize,
    pub locks: Locks,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
    pub last_update_time: Timestamp,
    pub debug_mode: bool,
    pub root_dir: String,

    pub auto_sync_delay_in_seconds: u64,
    pub auto_commit_delay_in_seconds: u64,

    pub server_port: Option<u16>,
}

impl Status {
    pub fn read(conn: &BazaConnection) -> Result<Self> {
        let root_dir = conn.get_path_manager().root_dir.clone();

        let instance_id = conn.get_instance_id()?.to_string();
        let db_rev = conn.get_db_rev()?;
        let last_sync_time = conn.get_last_sync_time()?;
        let db_version = conn.get_db_version()?;
        let data_version = conn.get_data_version()?;
        let computed_data_version = conn.get_computed_data_version()?;
        let documents_count = conn.count_documents()?;
        let blobs_count = conn.count_blobs()?;
        let conflicts_count = conn.get_coflicting_documents()?.len();
        let last_update_time = conn.get_last_update_time()?;
        let locks = conn.list_document_locks()?;
        let server_port = ArhivServer::get_server_port()?;
        let auto_sync_delay_in_seconds = conn.get_auto_sync_delay()?.as_secs();
        let auto_commit_delay_in_seconds = conn.get_auto_commit_delay()?.as_secs();

        Ok(Status {
            instance_id,
            app_version: get_crate_version().to_string(),
            db_version,
            data_version,
            computed_data_version,
            documents_count,
            blobs_count,
            conflicts_count,
            db_rev,
            last_sync_time,
            last_update_time,
            debug_mode: DEBUG_MODE,
            root_dir,
            locks,
            auto_sync_delay_in_seconds,
            auto_commit_delay_in_seconds,
            server_port,
        })
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.debug_mode {
            writeln!(f)?;
            writeln!(
                f,
                "----------------------------- DEBUG MODE -----------------------------"
            )?;
        }

        writeln!(f)?;

        writeln!(f, "             Root dir: {}", self.root_dir)?;
        writeln!(f, "          Instance id: {}", self.instance_id)?;

        writeln!(f)?;

        writeln!(f, "          App version: {}", self.app_version)?;
        writeln!(f, "           DB version: {}", self.db_version)?;
        writeln!(f, "       Schema version: {}", self.data_version)?;
        writeln!(f, "Computed data version: {}", self.computed_data_version)?;
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
            if self.last_update_time == MIN_TIMESTAMP {
                "NEVER".to_string()
            } else {
                default_date_time_format(self.last_update_time)
            }
        )?;
        writeln!(
            f,
            "   Last sync time: {}",
            if self.last_sync_time == MIN_TIMESTAMP {
                "NEVER".to_string()
            } else {
                default_date_time_format(self.last_sync_time)
            }
        )?;

        writeln!(f)?;
        writeln!(
            f,
            "Auto-commit delay: {} seconds",
            self.auto_commit_delay_in_seconds
        )?;
        writeln!(
            f,
            "  Auto-sync delay: {} seconds",
            self.auto_sync_delay_in_seconds
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
            "            BLOBs: {} total ({} new), {} local ({} unused)",
            self.blobs_count.total_blobs_count,
            self.blobs_count.blobs_staged,
            self.blobs_count.local_blobs_count,
            self.blobs_count.local_blobs_count - self.blobs_count.local_used_blobs_count,
        )?;

        writeln!(f)?;
        writeln!(f, "            Locks: {}", self.locks.len())?;
        for (id, reason) in &self.locks {
            writeln!(f, "   {:>30}: {reason}", id)?;
        }

        if self.conflicts_count > 0 {
            writeln!(f)?;
            writeln!(f, "        WARN:  found {} conflicts", self.conflicts_count)?;
            writeln!(f)?;
        }

        Ok(())
    }
}
