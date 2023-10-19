use std::sync::Arc;

use anyhow::Result;
use serde_json::json;

use crate::{sync::SyncManager, tests::new_document, Baza, BazaEvent};

#[tokio::test]
async fn test_events() -> Result<()> {
    let baza0 = Arc::new(Baza::new_test_baza());

    let mut events0 = baza0.get_events_channel();

    {
        let event_future0 = events0.recv();

        let mut tx = baza0.get_tx()?;
        let mut document = new_document(json!({}));
        tx.stage_document(&mut document)?;
        tx.commit()?;

        let event0 = event_future0.await?;

        assert_eq!(event0, BazaEvent::DocumentStaged {});
    }

    {
        let event_future0 = events0.recv();

        let mut tx = baza0.get_tx()?;
        tx.commit_staged_documents()?;
        tx.commit()?;

        let event0 = event_future0.await?;

        assert_eq!(event0, BazaEvent::DocumentsCommitted {});
    }

    let mut sync_manager0 = SyncManager::new(baza0.clone())?;
    let baza1 = Arc::new(Baza::new_test_baza());
    {
        let event_future0 = events0.recv();

        sync_manager0.add_in_mem_agent(baza1.clone())?;

        let event0 = event_future0.await?;

        assert_eq!(event0, BazaEvent::PeerDiscovered {});
    }

    let mut events1 = baza1.get_events_channel();
    {
        let event_future0 = events0.recv();
        let event_future1 = events1.recv();

        sync_manager0.sync().await?;

        let event0 = event_future0.await?;
        assert_eq!(event0, BazaEvent::Synced {});

        let event1 = event_future1.await?;
        assert_eq!(event1, BazaEvent::InstanceOutdated {});
    }

    Ok(())
}
