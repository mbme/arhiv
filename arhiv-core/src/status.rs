use std::fmt;

use serde::Serialize;

use baza::{sync::Revision, BLOBSCount, DocumentsCount};
use rs_utils::{format_time, Timestamp, MIN_TIMESTAMP};

#[derive(Serialize, Debug)]
pub struct DbStatus {
    pub data_version: u8,

    pub db_rev: Revision,
    pub last_sync_time: Timestamp,
}

#[derive(Serialize)]
pub struct Status {
    pub app_version: String,

    pub db_status: DbStatus,
    pub db_version: u8,
    pub data_version: u8,
    pub documents_count: DocumentsCount,
    pub blobs_count: BLOBSCount,
    pub conflicts_count: u32,

    pub last_update_time: Timestamp,
    pub debug_mode: bool,
    pub root_dir: String,
}

impl Status {
    pub fn is_sync_required(&self) -> bool {
        self.documents_count.count_staged_documents() > 0
            || self.documents_count.erased_documents_staged > 0
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Arhiv (rev {}) in {}",
            self.db_status.db_rev, self.root_dir,
        )?;

        writeln!(f)?;

        writeln!(f, "      App version: {}", self.app_version)?;
        writeln!(f, "       DB version: {}", self.db_version)?;
        writeln!(f, "   Schema version: {}", self.data_version)?;

        writeln!(f)?;

        writeln!(
            f,
            "           Synced: {}",
            self.documents_count.count_staged() == 0
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
            if self.db_status.last_sync_time == MIN_TIMESTAMP {
                "NEVER".to_string()
            } else {
                format_date(self.db_status.last_sync_time)
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
