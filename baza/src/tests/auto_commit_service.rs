use std::{sync::Arc, time::Duration};

use anyhow::Result;
use serde_json::Value;
use tokio::time::{advance, sleep, Instant};

use crate::{entities::Id, tests::new_document_snapshot, AutoCommitService, Baza};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_auto_commit_on_start() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    baza.add_document(Id::new(), Value::Null)?;

    let auto_commit_timeout = Duration::from_secs(10);
    let service = AutoCommitService::new(baza.clone(), auto_commit_timeout).with_fake_time();

    assert!(baza.get_connection()?.has_staged_documents()?);

    advance(auto_commit_timeout * 2).await;

    let _task = service.start()?;

    sleep(Duration::from_secs(1)).await;

    assert!(!baza.get_connection()?.has_staged_documents()?);

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_schedule_auto_commit_on_start() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    let auto_commit_timeout = Duration::from_secs(10);
    let service = AutoCommitService::new(baza.clone(), auto_commit_timeout).with_fake_time();

    baza.add_document(Id::new(), Value::Null)?;

    assert!(baza.get_connection()?.has_staged_documents()?);

    advance(auto_commit_timeout * 2).await;

    service.start()?;

    sleep(Duration::from_secs(1)).await;

    assert!(!baza.get_connection()?.has_staged_documents()?);

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_schedule_auto_commit_on_staged_document() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    let auto_commit_timeout = Duration::from_secs(10);
    let service = AutoCommitService::new(baza.clone(), auto_commit_timeout).with_fake_time();

    service.start()?;

    sleep(Duration::from_secs(1)).await;

    {
        let mut tx = baza.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    assert!(baza.get_connection()?.has_staged_documents()?);

    sleep(Duration::from_secs(1)).await;
    advance(auto_commit_timeout * 2).await;
    sleep(Duration::from_secs(1)).await;

    assert!(!baza.get_connection()?.has_staged_documents()?);

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_reschedule_auto_commit_on_staged_document() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    let auto_commit_timeout = Duration::from_secs(10);
    let service = AutoCommitService::new(baza.clone(), auto_commit_timeout).with_fake_time();
    let start_instant = Instant::now();

    service.start()?;

    sleep(Duration::from_secs(1)).await;

    {
        let mut tx = baza.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    advance(auto_commit_timeout - Duration::from_secs(2)).await;
    sleep(Duration::from_secs(1)).await;

    assert!(baza.get_connection()?.has_staged_documents()?);

    {
        let mut tx = baza.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document)?;

        // modify updated_at to take elapsed tokio time into account
        document.updated_at += Instant::now().duration_since(start_instant);
        tx.put_document(&document)?;

        tx.commit()?;
    }

    advance(auto_commit_timeout - Duration::from_secs(2)).await;
    sleep(Duration::from_secs(1)).await;

    assert!(baza.get_connection()?.has_staged_documents()?);

    Ok(())
}
