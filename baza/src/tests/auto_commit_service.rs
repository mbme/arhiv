use std::{sync::Arc, time::Duration};

use anyhow::Result;
use serde_json::Value;
use tokio::time::{advance, sleep};

use crate::{entities::Id, tests::new_document_snapshot, AutoCommitService, Baza};

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_auto_commit_on_start() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    baza.add_document(Id::new(), Value::Null)?;

    let mut service =
        AutoCommitService::new(baza.clone(), Duration::from_secs(10)).with_fake_time();

    assert!(baza.get_connection()?.has_staged_documents()?);

    advance(service.get_auto_commit_timeout() * 2).await;

    service.start()?;

    sleep(Duration::from_secs(1)).await;

    assert!(!baza.get_connection()?.has_staged_documents()?);

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_schedule_auto_commit_on_start() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    let mut service =
        AutoCommitService::new(baza.clone(), Duration::from_secs(10)).with_fake_time();

    baza.add_document(Id::new(), Value::Null)?;

    assert!(baza.get_connection()?.has_staged_documents()?);

    service.start()?;

    advance(service.get_auto_commit_timeout() * 2).await;
    sleep(Duration::from_secs(1)).await;

    assert!(!baza.get_connection()?.has_staged_documents()?);

    // should schedule auto commit on staged document
    // should reschedule auto commit on staged document

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_schedule_auto_commit_on_staged_document() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    let mut service =
        AutoCommitService::new(baza.clone(), Duration::from_secs(10)).with_fake_time();

    service.start()?;

    sleep(Duration::from_secs(1)).await;

    {
        let tx = baza.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    assert!(baza.get_connection()?.has_staged_documents()?);

    sleep(Duration::from_secs(1)).await;
    advance(service.get_auto_commit_timeout() * 2).await;
    sleep(Duration::from_secs(1)).await;

    assert!(!baza.get_connection()?.has_staged_documents()?);

    Ok(())
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn test_reschedule_auto_commit_on_staged_document() -> Result<()> {
    let baza = Arc::new(Baza::new_test_baza());
    let mut service =
        AutoCommitService::new(baza.clone(), Duration::from_secs(10)).with_fake_time();

    service.start()?;

    sleep(Duration::from_secs(1)).await;

    {
        let tx = baza.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    advance(service.get_auto_commit_timeout() - Duration::from_secs(2)).await;
    sleep(Duration::from_secs(1)).await;

    assert!(baza.get_connection()?.has_staged_documents()?);

    {
        let tx = baza.get_tx()?;
        let mut document = new_document_snapshot(Id::new(), Value::Null);
        tx.stage_document(&mut document)?;
        tx.commit()?;
    }

    advance(service.get_auto_commit_timeout() - Duration::from_secs(2)).await;
    sleep(Duration::from_secs(1)).await;

    assert!(baza.get_connection()?.has_staged_documents()?);

    Ok(())
}
