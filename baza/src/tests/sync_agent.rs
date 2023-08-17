use std::rc::Rc;

use anyhow::Result;
use serde_json::json;

use crate::{
    sync::{agent::SyncAgent, network::InMemorySyncNetwork},
    tests::{create_changeset, new_document_snapshot},
    Baza,
};

#[tokio::test]
async fn test_sync_agent() -> Result<()> {
    let network = InMemorySyncNetwork::new();

    {
        let baza = Rc::new(Baza::new_test_baza_with_id("0"));

        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 2, "1": 1 })),
            new_document_snapshot("2", json!({ "0": 2 })),
        ]))?;
        tx.commit()?;

        let agent = SyncAgent::new(baza, &network)?;
        network.add_agent(agent);
    }

    {
        let baza = Rc::new(Baza::new_test_baza_with_id("1"));

        let mut tx = baza.get_tx()?;
        tx.apply_changeset(create_changeset(vec![
            new_document_snapshot("1", json!({ "0": 1, "1": 1 })),
            new_document_snapshot("1", json!({ "0": 1, "1": 2 })),
            new_document_snapshot("3", json!({ "1": 2 })),
        ]))?;
        tx.commit()?;

        let agent = SyncAgent::new(baza, &network)?;
        network.add_agent(agent);
    }

    {
        let agents = network.agents.borrow();

        let agent1 = &agents[0];
        let snapshots_count = agent1.baza.get_tx()?.list_all_document_snapshots()?.len();
        assert_eq!(snapshots_count, 3);

        assert_eq!(agent1.get_pings()?.len(), 0);
        assert_eq!(agent1.sync().await?, false);

        agent1.refresh_peers().await?;

        assert_eq!(agent1.get_pings()?.len(), 1);
        assert_eq!(agent1.sync().await?, true);

        let snapshots_count = agent1.baza.get_tx()?.list_all_document_snapshots()?.len();
        assert_eq!(snapshots_count, 5);
    }

    Ok(())
}
