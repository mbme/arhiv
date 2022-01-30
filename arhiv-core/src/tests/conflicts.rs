use anyhow::Result;

use crate::test_arhiv::TestArhiv;

use super::utils::*;

#[tokio::test]
async fn test_conflicts() -> Result<()> {
    let arhiv = TestArhiv::new_prime();

    let mut document = empty_document();
    arhiv.stage_document(&mut document)?;
    arhiv.sync().await?;

    // update the same document
    arhiv.stage_document(&mut document)?;

    assert_eq!(arhiv.get_status()?.conflicts_count, 0);

    // set wrong prev_rev
    {
        let tx = arhiv.get_tx()?;

        tx.conn.execute(
            "UPDATE documents_snapshots SET prev_rev = 0 WHERE rev = 0",
            [],
        )?;

        tx.commit()?;
    }

    assert_eq!(arhiv.get_status()?.conflicts_count, 1);

    Ok(())
}
