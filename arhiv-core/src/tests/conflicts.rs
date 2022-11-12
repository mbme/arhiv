use anyhow::Result;

use crate::{test_arhiv::TestArhiv, BazaConnectionExt};

use super::utils::*;

#[tokio::test]
async fn test_conflicts() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let mut document = {
        let tx = arhiv.baza.get_tx()?;

        let mut document = empty_document();
        tx.stage_document(&mut document)?;

        tx.commit()?;

        document
    };

    arhiv.sync().await?;

    // update the same document
    {
        let tx = arhiv.baza.get_tx()?;
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    assert_eq!(
        arhiv.baza.get_connection()?.get_status()?.conflicts_count,
        0
    );

    // set wrong prev_rev
    {
        let tx = arhiv.baza.get_tx()?;

        tx.get_connection().execute(
            "UPDATE documents_snapshots SET prev_rev = 0 WHERE rev = 0",
            [],
        )?;

        tx.commit()?;
    }

    assert_eq!(
        arhiv.baza.get_connection()?.get_status()?.conflicts_count,
        1
    );

    Ok(())
}

#[tokio::test]
async fn test_deleted_document_isnt_conflict() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let document = {
        let tx = arhiv.baza.get_tx()?;

        let mut document = empty_document();
        tx.stage_document(&mut document)?;

        tx.commit()?;

        document
    };

    arhiv.sync().await?;

    // update the same document
    {
        let tx = arhiv.baza.get_tx()?;
        tx.erase_document(&document.id)?;
        tx.commit()?;
    }

    assert_eq!(
        arhiv.baza.get_connection()?.get_status()?.conflicts_count,
        0
    );

    Ok(())
}
