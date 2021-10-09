use std::convert::TryInto;

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
        let mut document = new_document(original_data.clone());
        arhiv.stage_document(&mut document)?;
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
        let mut other_document = arhiv.get_document(&id)?.unwrap();
        other_document.data = json!({ "test": "1" }).try_into().unwrap();
        arhiv.stage_document(&mut other_document)?;

        assert_eq!(arhiv.get_document(&id)?.unwrap().data, other_document.data);
    }

    // DELETE
    {
        let mut document = new_document(json!({ "test": "test" }));
        arhiv.stage_document(&mut document)?;

        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 2);

        arhiv.delete_document(&document.id)?;

        assert!(arhiv.get_document(&document.id)?.unwrap().is_tombstone());
        assert_eq!(arhiv.list_documents(Filter::default())?.items.len(), 2);
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
    let mut document = new_document(json!({ "test": "test" }));
    arhiv.stage_document(&mut document)?;

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
    arhiv.stage_document(&mut document)?;

    // create another document
    arhiv.stage_document(&mut new_document(json!({ "test": "test" })))?;

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
