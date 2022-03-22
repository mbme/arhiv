use anyhow::Result;
use serde_json::json;

use rs_utils::workspace_relpath;

use super::utils::*;
use crate::{entities::Revision, test_arhiv::TestArhiv, BLOBSCount, DocumentsCount, Filter};

#[test]
fn test_crud() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let original_data = json!({ "test": "test" });

    // CREATE
    let id = {
        let tx = arhiv.get_tx()?;

        let mut document = new_document(original_data.clone());
        tx.stage_document(&mut document)?;
        tx.commit()?;

        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 1);

        document.id
    };

    // READ
    {
        let other_document = arhiv.get_document(&id)?.unwrap();

        assert_eq!(other_document.data, original_data.try_into().unwrap());
        assert!(other_document.rev.is_staged());
    }

    // UPDATE
    {
        let tx = arhiv.get_tx()?;

        let mut other_document = arhiv.get_document(&id)?.unwrap();
        other_document.data = json!({ "test": "1" }).try_into().unwrap();
        tx.stage_document(&mut other_document)?;
        tx.commit()?;

        assert_eq!(arhiv.get_document(&id)?.unwrap().data, other_document.data);
    }

    // DELETE
    let document_id = {
        let tx = arhiv.get_tx()?;

        let mut document = new_document(json!({ "test": "test" }));
        tx.stage_document(&mut document)?;
        tx.commit()?;

        document.id
    };

    assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 2);

    {
        let tx = arhiv.get_tx()?;

        tx.erase_document(&document_id)?;
        tx.commit()?;

        let erased_document = arhiv.get_document(&document_id)?.unwrap();

        assert!(erased_document.is_erased());
        assert_eq!(erased_document.prev_rev, Revision::STAGING);

        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 2);
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
#[tokio::test]
async fn test_status() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 0,
                documents_new: 0,
                erased_documents_committed: 0,
                erased_documents_staged: 0,
                snapshots: 0,
            }
        );
        assert_eq!(
            status.blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 0,
                local_blobs_count: 0,
                local_used_blobs_count: 0,
            }
        );
    }

    // create document with blob
    let mut document = {
        let mut tx = arhiv.get_tx()?;

        let blob_id = tx.add_blob(&workspace_relpath("resources/k2.jpg"), false)?;
        let mut document = new_document(json!({
            "test": "test",
            "blob": blob_id,
        }));
        tx.stage_document(&mut document)?;

        tx.commit()?;

        document
    };

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.blobs_count,
            BLOBSCount {
                blobs_staged: 1,
                total_blobs_count: 1,
                local_blobs_count: 1,
                local_used_blobs_count: 1,
            }
        );
    }

    // commit document
    arhiv.sync().await?;

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 1,
                documents_updated: 0,
                documents_new: 0,
                erased_documents_committed: 0,
                erased_documents_staged: 0,
                snapshots: 1,
            }
        );

        assert_eq!(
            status.blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 1,
                local_blobs_count: 1,
                local_used_blobs_count: 1,
            }
        );
    }

    {
        let tx = arhiv.get_tx()?;

        // update document
        tx.stage_document(&mut document)?;

        // create another document
        tx.stage_document(&mut new_document(json!({ "test": "test" })))?;

        tx.commit()?;
    }

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 1,
                documents_new: 1,
                erased_documents_committed: 0,
                erased_documents_staged: 0,
                snapshots: 3,
            }
        );

        assert_eq!(status.documents_count.count_staged_documents(), 2);
    }

    // delete document
    {
        let tx = arhiv.get_tx()?;

        tx.erase_document(&document.id)?;

        tx.commit()?;
    }

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 0,
                documents_new: 1,
                erased_documents_committed: 0,
                erased_documents_staged: 1,
                snapshots: 3,
            }
        );

        assert_eq!(
            status.blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 0,
                local_blobs_count: 1,
                local_used_blobs_count: 0,
            }
        );
    }

    arhiv.sync().await?;

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 1,
                documents_updated: 0,
                documents_new: 0,
                erased_documents_committed: 1,
                erased_documents_staged: 0,
                snapshots: 2,
            }
        );

        assert_eq!(
            status.blobs_count,
            BLOBSCount {
                blobs_staged: 0,
                total_blobs_count: 0,
                local_blobs_count: 0,
                local_used_blobs_count: 0,
            }
        );
    }

    Ok(())
}
