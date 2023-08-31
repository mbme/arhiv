use std::{rc::Rc, sync::Arc};

use anyhow::Result;
use serde_json::json;

use crate::{
    sync::{SyncAgent, SyncService},
    tests::{create_changeset, new_document_snapshot},
    Baza,
};

#[tokio::test]
async fn test_sync_service() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza_with_id("0"));

    {
        let mut tx = baza0.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 2 })),
        ]))?;
        tx.commit()?;
    }

    let baza1 = Rc::new(Baza::new_test_baza_with_id("1"));
    {
        let mut tx = baza1.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("3", json!({ "1": 2 })),
        ]))?;
        tx.commit()?;
    }

    let sync_service = SyncService::new(baza0.clone());

    let agent1 = SyncAgent::new_in_memory(baza1.clone())?;
    sync_service.add_agent(agent1);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 3);

    assert_eq!(sync_service.get_pings().len(), 0);
    assert_eq!(sync_service.sync().await?, false);

    sync_service.refresh_peers().await?;
    assert_eq!(sync_service.get_pings().len(), 1);
    assert_eq!(sync_service.sync().await?, true);

    let snapshots_count = baza0.get_tx()?.list_all_document_snapshots()?.len();
    assert_eq!(snapshots_count, 5);

    Ok(())
}
