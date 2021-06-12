use anyhow::*;
use serde_json::json;

use super::utils::*;
use crate::{DocumentsCount, Filter};

#[test]
fn test_crud() -> Result<()> {
    let arhiv = new_prime();

    let original_data = json!({ "test": "test" });

    // CREATE
    let id = {
        let document = new_document(original_data.clone());
        arhiv.stage_document(document.clone())?;
        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 1);

        document.id
    };

    // READ
    {
        let other_document = arhiv.get_document(&id)?.unwrap();

        assert_eq!(other_document.data, original_data);
        assert_eq!(other_document.rev.is_staged(), true);
    }

    // UPDATE
    {
        let mut other_document = arhiv.get_document(&id)?.unwrap();
        other_document.data = json!({ "test": "1" });
        arhiv.stage_document(other_document.clone())?;

        assert_eq!(arhiv.get_document(&id)?.unwrap().data, other_document.data);
    }

    // ARCHIVE
    {
        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 1);
        let mut other_document = arhiv.get_document(&id)?.unwrap();
        other_document.archived = true;
        arhiv.stage_document(other_document)?;

        assert_eq!(arhiv.get_document(&id)?.unwrap().archived, true);
        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 0);
    }

    // DELETE
    {
        let document = new_document(json!({ "test": "test" }));
        arhiv.stage_document(document.clone())?;

        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 1);

        arhiv.delete_document(&document.id)?;

        assert_eq!(
            arhiv.get_document(&document.id)?.unwrap().is_tombstone(),
            true
        );
        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 0);
    }

    Ok(())
}

#[tokio::test]
async fn test_status() -> Result<()> {
    let arhiv = new_prime();

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 0,
                documents_new: 0,
                attachments_committed: 0,
                attachments_updated: 0,
                attachments_new: 0,
                tombstones_committed: 0,
                tombstones_updated: 0,
                tombstones_new: 0,
            }
        );
    }

    // create document
    let document = new_document(json!({ "test": "test" }));
    arhiv.stage_document(document.clone())?;

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
                attachments_committed: 0,
                attachments_updated: 0,
                attachments_new: 0,
                tombstones_committed: 0,
                tombstones_updated: 0,
                tombstones_new: 0,
            }
        );
    }

    // update document
    arhiv.stage_document(document.clone())?;

    // create another document
    arhiv.stage_document(new_document(json!({ "test": "test" })))?;

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 1,
                documents_new: 1,
                attachments_committed: 0,
                attachments_updated: 0,
                attachments_new: 0,
                tombstones_committed: 0,
                tombstones_updated: 0,
                tombstones_new: 0,
            }
        );

        assert_eq!(status.documents_count.count_staged_documents(), 2);
    }

    // delete document
    arhiv.delete_document(&document.id)?;

    {
        let status = arhiv.get_status()?;
        assert_eq!(
            status.documents_count,
            DocumentsCount {
                documents_committed: 0,
                documents_updated: 0,
                documents_new: 1,
                attachments_committed: 0,
                attachments_updated: 0,
                attachments_new: 0,
                tombstones_committed: 0,
                tombstones_updated: 1,
                tombstones_new: 0,
            }
        );
    }

    Ok(())
}
