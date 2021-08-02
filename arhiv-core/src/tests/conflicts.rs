use anyhow::*;

use super::utils::*;

#[tokio::test]
async fn test_conflicts() -> Result<()> {
    let arhiv = new_prime();

    let mut document = empty_document();
    arhiv.stage_document(&mut document)?;
    arhiv.sync().await?;

    // update the same document
    arhiv.stage_document(&mut document)?;

    assert_eq!(arhiv.get_status()?.conflicts_count, 0);

    // set wrong prev_rev
    {
        let conn = arhiv.db.open_connection(true)?;
        conn.execute(
            "UPDATE documents_snapshots SET prev_rev = 0 WHERE rev = 0",
            [],
        )?;
    }

    assert_eq!(arhiv.get_status()?.conflicts_count, 1);

    Ok(())
}
