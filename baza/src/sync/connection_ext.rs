use std::collections::HashSet;

use anyhow::{ensure, Result};

use rs_utils::{log, now};

use crate::{
    entities::{Id, BLOB},
    BazaConnection, DocumentExpert,
};

use super::{Changeset, Ping, Revision};

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
    pub fn get_ping(&self) -> Result<Ping> {
        let ping = Ping {
            instance_id: self.get_instance_id()?,
            data_version: self.get_data_version()?,
            rev: self.get_db_rev()?,
            timestamp: now(),
        };

        Ok(ping)
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

        let schema = self.get_schema();
        let document_expert = DocumentExpert::new(&schema);

        let mut missing_blobs = HashSet::new();

        for document in &changeset.documents {
            if self.has_snapshot(&document.id, &document.rev)? {
                log::warn!("Got duplicate snapshot of the {}, ignoring", &document);
                continue;
            }

            self.put_document(&document)?;
            summary.new_snapshots += 1;

            if document.is_erased() {
                // erase history of erased documents
                self.erase_document_history(&document.id, &document.rev)?;
                summary.erased_documents.insert(document.id.clone());
            }

            for blob_id in document_expert
                .extract_refs(&document.class, &document.data)?
                .blobs
            {
                let blob = self.get_blob(&blob_id);

                if !blob.exists()? {
                    missing_blobs.insert(blob);
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

    pub fn get_changeset(&mut self, min_rev: &Revision) -> Result<Changeset> {
        ensure!(
            !self.has_staged_documents()?,
            "there must be no staged changes"
        );

        Ok(Changeset {
            data_version: self.get_data_version()?,
            documents: self.list_document_revisions(min_rev)?,
        })
    }
}
