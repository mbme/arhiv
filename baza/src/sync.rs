use anyhow::{ensure, Result};

use rs_utils::log;

use crate::baza::Baza;
use crate::db::BazaConnection;
use crate::document_expert::DocumentExpert;
use crate::entities::{Changeset, Document};

impl Baza {
    pub fn apply_changeset(
        &self,
        tx: &mut BazaConnection,
        changeset: Changeset,
    ) -> Result<Vec<Document>> {
        log::debug!("applying changeset {}", &changeset);

        let schema = self.get_schema();

        // FIXME remove arhiv id
        // let arhiv_id = tx.get_setting(&SETTING_ARHIV_ID)?;
        // ensure!(
        //     changeset.arhiv_id == arhiv_id,
        //     "changeset arhiv_id {} must be equal to {}",
        //     changeset.arhiv_id,
        //     arhiv_id,
        // );

        ensure!(
            changeset.data_version == self.data_version,
            "changeset data_version {} must be equal to {}",
            changeset.data_version,
            self.data_version,
        );

        ensure!(
            !tx.has_staged_documents()?,
            "there must be no staged changes"
        );

        let db_rev = tx.get_db_rev()?;

        ensure!(
            changeset.base_rev <= db_rev,
            "base_rev {} is greater than local db rev {}",
            changeset.base_rev,
            db_rev,
        );

        let mut conflicts = vec![];

        if changeset.is_empty() {
            log::debug!("empty changeset, ignoring");
            return Ok(conflicts);
        }

        let new_rev = db_rev.inc();
        log::debug!("current rev is {}, new rev is {}", db_rev, new_rev);

        let document_expert = DocumentExpert::new(schema);

        for mut document in changeset.documents {
            if tx.has_snapshot(&document.id, document.rev)? {
                log::warn!("Got duplicate snapshot of the {}, ignoring", &document);

                continue;
            }

            if document.is_erased() {
                document.rev = new_rev;

                tx.put_document(&document)?;

                // erase history of erased documents
                tx.erase_document_history(&document.id)?;

                continue;
            }

            match tx.get_last_snapshot(&document.id)? {
                // on conflict
                Some(prev_snapshot) if prev_snapshot.rev != document.prev_rev => {
                    if prev_snapshot.is_erased() {
                        log::warn!(
                            "Got an update for erased document {}, ignoring",
                            &document.id
                        );
                        continue;
                    }

                    if document.data != prev_snapshot.data {
                        log::warn!("Got data conflict on document {}", &document.id);
                        conflicts.push(document);
                        continue;
                    }
                }
                _ => {}
            }

            document.rev = new_rev;

            tx.put_document(&document)?;

            for blob_id in document_expert
                .extract_refs(&document.document_type, &document.data)?
                .blobs
            {
                let blob = tx.get_blob(&blob_id);

                ensure!(
                    blob.exists()?,
                    "Document {} references unknown blob {}",
                    &document.id,
                    &blob_id,
                );
            }
        }

        log::debug!("successfully applied a changeset");

        Ok(conflicts)
    }
}
