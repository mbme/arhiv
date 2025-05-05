use anyhow::Result;
use serde::Serialize;

use crate::{BazaStorage, schema::ASSET_TYPE};

use super::Baza;

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct BLOBSCount {
    pub total_referenced_blobs: usize,
    pub blobs_staged: usize,
    pub blobs_in_storage: usize,
}

impl BLOBSCount {
    pub fn count_present_blobs(&self) -> usize {
        self.blobs_in_storage + self.blobs_staged
    }
}

#[derive(Serialize, Debug, PartialEq, Eq, Default)]
pub struct DocumentsCount {
    pub documents_committed: usize,
    pub documents_updated: usize,
    pub documents_new: usize,

    pub conflicts_count: usize,

    pub erased_documents_committed: usize,
    pub erased_documents_staged: usize,

    pub snapshots: usize,
}

impl DocumentsCount {
    #[must_use]
    pub fn count_staged_documents(&self) -> usize {
        self.documents_updated + self.documents_new
    }

    #[must_use]
    pub fn count_staged(&self) -> usize {
        self.count_staged_documents() + self.erased_documents_staged
    }
}

impl Baza {
    pub fn count_blobs(&self) -> Result<BLOBSCount> {
        let blobs_staged = self.paths.list_state_blobs()?.len();
        let blobs_in_storage = self.paths.list_storage_blobs()?.len();

        let total_referenced_blobs = self
            .state
            .iter_documents()
            .filter(|head| head.get_type() == &ASSET_TYPE)
            .count();

        Ok(BLOBSCount {
            blobs_in_storage,
            blobs_staged,
            total_referenced_blobs,
        })
    }

    pub fn count_documents(&self) -> Result<DocumentsCount> {
        let mut count: DocumentsCount = Default::default();

        for head in self.iter_documents() {
            if head.is_committed() {
                count.documents_committed += 1;
            }
            if head.is_staged() {
                if head.is_new_document() {
                    count.documents_new += 1;
                } else {
                    count.documents_updated += 1;
                }
            }
            if head.is_conflict() {
                count.conflicts_count += 1;
            }

            if head.is_original_erased() {
                count.erased_documents_committed += 1;
            }

            if head.is_staged_erased() {
                count.erased_documents_staged += 1;
            }
        }

        let storage = BazaStorage::read_file(&self.paths.storage_main_db_file, self.key.clone())?;
        count.snapshots = storage.index.len();

        Ok(count)
    }
}
