use std::collections::HashSet;

use anyhow::{ensure, Result};

use rs_utils::log;

use crate::{
    entities::{Id, Revision, BLOB},
    BazaConnection, DocumentExpert,
};

use super::{Changeset, ChangesetRequest};

#[derive(Default)]
pub struct ChangesetSummary {
    pub new_snapshots: usize,
    pub erased_documents: HashSet<Id>,
    pub missing_blobs: HashSet<BLOB>,
}

impl ChangesetSummary {
    pub fn has_changes(&self) -> bool {
        self.new_snapshots > 0
    }
}

impl BazaConnection {
    pub fn get_changeset_request(&self) -> Result<ChangesetRequest> {
        let request = ChangesetRequest {
            instance_id: self.get_instance_id()?,
            data_version: self.get_data_version()?,
            rev: self.get_db_rev()?,
        };

        Ok(request)
    }

    pub fn apply_changeset(&mut self, changeset: Changeset) -> Result<ChangesetSummary> {
        let mut summary = ChangesetSummary::default();

        if changeset.is_empty() {
            log::warn!("got empty changeset, ignoring");
            return Ok(summary);
        }

        let data_version = self.get_data_version()?;
        ensure!(
            changeset.data_version == data_version,
            "changeset data_version {} must be equal to {}",
            changeset.data_version,
            data_version,
        );

        ensure!(
            !self.has_staged_documents()?,
            "there must be no staged changes"
        );

        let locks = self.list_document_locks()?;
        ensure!(locks.is_empty(), "there are {} pending locks", locks.len());

        let schema = self.get_schema();
        let document_expert = DocumentExpert::new(&schema);

        for document in &changeset.documents {
            if self.has_snapshot(&document.id, document.get_rev()?)? {
                log::warn!("Got duplicate snapshot of the {}, ignoring", &document);
                continue;
            }

            self.put_document(document)?;
            summary.new_snapshots += 1;

            if document.is_erased() {
                // erase history of erased documents
                self.erase_document_history(&document.id, document.get_rev()?)?;
                summary.erased_documents.insert(document.id.clone());
            }

            for blob_id in document_expert
                .extract_refs(&document.document_type, &document.data)?
                .blobs
            {
                let blob = self.get_blob(&blob_id);

                if !blob.exists()? {
                    summary.missing_blobs.insert(blob);
                }
            }
        }

        log::debug!(
            "Successfully applied a changeset {}, {} new BLOBs are missing",
            &changeset,
            summary.missing_blobs.len()
        );

        Ok(summary)
    }

    pub fn get_changeset(&self, min_rev: &Revision) -> Result<Changeset> {
        Ok(Changeset {
            data_version: self.get_data_version()?,
            documents: self.list_document_revisions(min_rev, true)?,
        })
    }
}
