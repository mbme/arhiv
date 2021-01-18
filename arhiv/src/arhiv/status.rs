use serde::Serialize;
use std::fmt;

use crate::entities::{Revision, Timestamp};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub arhiv_id: String,
    pub root_dir: String,
    pub is_prime: bool,
    pub rev: Revision,
    pub last_update_time: Timestamp,
    pub debug_mode: bool,

    pub committed_documents: u32,
    pub staged_documents: u32,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Arhiv {}/{} (rev {}) on {}",
            self.arhiv_id,
            if self.is_prime { "prime" } else { "replica" },
            self.rev,
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
            "  Documents: {} committed, {} staged",
            self.committed_documents, self.staged_documents
        )?;

        if self.debug_mode {
            writeln!(f, "  Debug Mode")?;
        }

        Ok(())
    }
}
