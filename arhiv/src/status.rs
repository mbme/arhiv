use std::{fmt, time::Duration};

use anyhow::Result;
use serde::Serialize;

use baza::{
    sync::Revision, BLOBSCount, BazaConnection, DocumentsCount, DEBUG_MODE, SETTING_DATA_VERSION,
    SETTING_LAST_SYNC_TIME,
};
use rs_utils::{format_time, get_crate_version, Timestamp, MIN_TIMESTAMP};

#[derive(Serialize)]
pub struct Status {
    pub app_version: String,

    pub db_version: u8,
    pub data_version: u8,
    pub documents_count: DocumentsCount,
    pub blobs_count: BLOBSCount,
    pub conflicts_count: usize,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
    pub last_update_time: Timestamp,
    pub debug_mode: bool,
    pub root_dir: String,

    pub local_server_is_running: Option<bool>,
    pub mdns_discovery_timeout: Option<Duration>,
    pub auto_sync_interval: Option<Duration>,
    pub auto_commit_interval: Option<Duration>,
}

impl Status {
    pub fn read(conn: &BazaConnection) -> Result<Self> {
        let root_dir = conn.get_path_manager().root_dir.clone();

        let db_rev = conn.get_db_rev()?;
        let last_sync_time = conn.kvs_const_must_get(SETTING_LAST_SYNC_TIME)?;
        let db_version = conn.get_db_version()?;
        let data_version = conn.kvs_const_must_get(SETTING_DATA_VERSION)?;
        let documents_count = conn.count_documents()?;
        let blobs_count = conn.count_blobs()?;
        let conflicts_count = conn.get_coflicting_documents()?.len();
        let last_update_time = conn.get_last_update_time()?;

        Ok(Status {
            app_version: get_crate_version().to_string(),
            db_version,
            data_version,
            documents_count,
            blobs_count,
            conflicts_count,
            db_rev,
            last_sync_time,
            last_update_time,
            debug_mode: DEBUG_MODE,
            root_dir,

            local_server_is_running: None,
            mdns_discovery_timeout: None,
            auto_sync_interval: None,
            auto_commit_interval: None,
        })
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Arhiv (rev {}) in {}",
            self.db_rev.serialize(),
            self.root_dir,
        )?;

        writeln!(f)?;

        writeln!(f, "      App version: {}", self.app_version)?;
        writeln!(f, "       DB version: {}", self.db_version)?;
        writeln!(f, "   Schema version: {}", self.data_version)?;

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
                format_date(self.last_update_time)
            }
        )?;
        writeln!(
            f,
            "   Last sync time: {}",
            if self.last_sync_time == MIN_TIMESTAMP {
                "NEVER".to_string()
            } else {
                format_date(self.last_sync_time)
            }
        )?;
        writeln!(
            f,
            "      Auto-commit: {}",
            if let Some(auto_commit_interval) = self.auto_commit_interval {
                format!("{} seconds", auto_commit_interval.as_secs())
            } else {
                "disabled".to_string()
            }
        )?;
        writeln!(
            f,
            "        Auto-sync: {}",
            if let Some(auto_sync_interval) = self.auto_sync_interval {
                format!("{} seconds", auto_sync_interval.as_secs())
            } else {
                "disabled".to_string()
            }
        )?;
        writeln!(
            f,
            "   MDNS discovery: {}",
            if let Some(mdns_discovery_timeout) = self.mdns_discovery_timeout {
                format!("{} seconds", mdns_discovery_timeout.as_secs())
            } else {
                "disabled".to_string()
            }
        )?;
        writeln!(
            f,
            "     Local server: {}",
            if self.local_server_is_running.unwrap_or_default() {
                "running"
            } else {
                "not running"
            }
        )?;

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

        if self.conflicts_count > 0 {
            writeln!(f)?;
            writeln!(f, "        WARN:  found {} conflicts", self.conflicts_count)?;
            writeln!(f)?;
        }

        if self.debug_mode {
            writeln!(f)?;
            writeln!(f, "  Debug Mode")?;
        }

        Ok(())
    }
}

fn format_date(date: Timestamp) -> String {
    format_time(date, "%a %b %e %T %Y")
}
