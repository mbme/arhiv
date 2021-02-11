use serde::Serialize;
use std::fmt;

use crate::{
    db::{DbStatus, DocumentsCount},
    entities::Timestamp,
};

#[derive(Serialize)]
pub struct Status {
    pub db_status: DbStatus,
    pub documents_count: DocumentsCount,

    pub last_update_time: Timestamp,
    pub debug_mode: bool,
    pub root_dir: String,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Arhiv {}/{} (rev {}) on {}",
            self.db_status.arhiv_id,
            self.db_status.get_prime_status(),
            self.db_status.db_rev,
            self.root_dir,
        )?;

        writeln!(
            f,
            "  Last update time: {}",
            if self.last_update_time == chrono::MIN_DATETIME {
                "NEVER".to_string()
            } else {
                self.last_update_time.to_string()
            }
        )?;
        writeln!(
            f,
            "  Last sync time: {}",
            if self.db_status.last_sync_time == chrono::MIN_DATETIME {
                "NEVER".to_string()
            } else {
                self.db_status.last_sync_time.to_string()
            }
        )?;
        writeln!(
            f,
            "  Documents:   {} committed, {} staged ({} updated, {} new)",
            self.documents_count.documents_committed,
            self.documents_count.count_staged_documents(),
            self.documents_count.documents_updated,
            self.documents_count.documents_new,
        )?;
        writeln!(
            f,
            "  Attachments: {} committed, {} staged ({} updated, {} new)",
            self.documents_count.documents_committed,
            self.documents_count.count_staged_attachments(),
            self.documents_count.documents_updated,
            self.documents_count.documents_new,
        )?;

        if self.debug_mode {
            writeln!(f, "  Debug Mode")?;
        }

        Ok(())
    }
}
