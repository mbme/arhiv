use std::{
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    process::Command,
};

use serde_json::json;

use baza_common::{TempFile, file_exists, read_all_as_string};

use crate::{
    BazaManager, BazaPaths,
    entities::{Id, new_document, new_empty_document},
    schema::DataSchema,
};

use super::{CommitCheckpoint, CommitTestAction};

fn open_existing_manager(test_dir: &str) -> BazaManager {
    let manager = BazaManager::new(
        BazaPaths::new_for_tests(test_dir),
        DataSchema::new_test_schema(),
    );
    manager.unlock("test password".into()).unwrap();

    manager
}

fn backup_files_count(dir: &str) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().ends_with("-backup"))
        .count()
}

fn checkpoints() -> [CommitCheckpoint; 5] {
    [
        CommitCheckpoint::DbBackedUp,
        CommitCheckpoint::BlobsMoved,
        CommitCheckpoint::DbWritten,
        CommitCheckpoint::StateBackedUp,
        CommitCheckpoint::StateWritten,
    ]
}

#[test]
fn test_commit_rollback_restores_pre_commit_archive() {
    for checkpoint in checkpoints() {
        let temp_dir = TempFile::new_with_details("commit_rollback", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let existing_document = new_document(json!({ "test": "committed" }));
        let staged_document = new_empty_document();
        let blob_file = temp_dir.new_child("blob");
        blob_file.write_str("new blob").unwrap();

        let asset_id = {
            let mut baza = manager.open_mut().unwrap();
            baza.stage_document(existing_document.clone(), &None)
                .unwrap();
            baza.commit().unwrap();

            baza.stage_document(staged_document.clone(), &None).unwrap();
            let asset = baza.create_asset(&blob_file.path).unwrap();
            baza.commit_test_action = Some((checkpoint, CommitTestAction::Fail));

            assert!(baza.commit().is_err(), "commit must fail at {checkpoint:?}");

            asset.id
        };

        let manager = open_existing_manager(&temp_dir.path);
        let baza = manager.open().unwrap();

        assert!(baza.get_document(&existing_document.id).is_some());
        assert!(baza.get_document(&staged_document.id).is_some());
        assert!(baza.has_staged_documents());
        assert_eq!(
            read_all_as_string(baza.get_asset_data(&asset_id).unwrap()).unwrap(),
            "new blob"
        );
        drop(baza);

        assert!(!manager.paths.storage_blob_exists(&asset_id).unwrap());
        assert!(file_exists(&manager.paths.get_state_blob_path(&asset_id)).unwrap());
        assert_eq!(backup_files_count(&manager.paths.storage_dir), 0);
        assert_eq!(backup_files_count(&manager.paths.state_dir), 0);
    }
}

#[derive(Debug, PartialEq, Eq)]
struct CrashOutcome {
    checkpoint: CommitCheckpoint,
    can_open: bool,
    open_panics: bool,
    has_staged_documents: Option<bool>,
    new_asset_is_visible: Option<bool>,
    storage_db_exists: bool,
    state_file_exists: bool,
    storage_blob_exists: bool,
    state_blob_exists: bool,
    storage_backup_files: usize,
    state_backup_files: usize,
}

fn inspect_crash_outcome(
    test_dir: &str,
    checkpoint: CommitCheckpoint,
    asset_id: &Id,
) -> CrashOutcome {
    let paths = BazaPaths::new_for_tests(test_dir);
    let storage_db_exists = paths.storage_main_db_file_exists().unwrap();
    let state_file_exists = paths.state_file_exists().unwrap();
    let storage_blob_exists = paths.storage_blob_exists(asset_id).unwrap();
    let state_blob_exists = file_exists(&paths.get_state_blob_path(asset_id)).unwrap();
    let storage_backup_files = backup_files_count(&paths.storage_dir);
    let state_backup_files = backup_files_count(&paths.state_dir);

    let manager = open_existing_manager(test_dir);
    let opened = catch_unwind(AssertUnwindSafe(|| manager.open()));
    let (can_open, open_panics, has_staged_documents, new_asset_is_visible) = match opened {
        Ok(Ok(baza)) => (
            true,
            false,
            Some(baza.has_staged_documents()),
            Some(baza.get_document(asset_id).is_some()),
        ),
        Ok(Err(_)) => (false, false, None, None),
        Err(_) => (false, true, None, None),
    };

    CrashOutcome {
        checkpoint,
        can_open,
        open_panics,
        has_staged_documents,
        new_asset_is_visible,
        storage_db_exists,
        state_file_exists,
        storage_blob_exists,
        state_blob_exists,
        storage_backup_files,
        state_backup_files,
    }
}

fn abort_commit(test_dir: &str, checkpoint_name: &str) {
    let status = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("baza::commit_tests::test_commit_abrupt_termination")
        .env("BAZA_COMMIT_ABORT_TEST_DIR", test_dir)
        .env("BAZA_COMMIT_ABORT_CHECKPOINT", checkpoint_name)
        .status()
        .unwrap();
    assert!(
        !status.success(),
        "child process must abort at {checkpoint_name}"
    );
}

#[test]
fn test_commit_abrupt_termination() {
    if let Ok(test_dir) = std::env::var("BAZA_COMMIT_ABORT_TEST_DIR") {
        let checkpoint = match std::env::var("BAZA_COMMIT_ABORT_CHECKPOINT")
            .unwrap()
            .as_str()
        {
            "after-db-backup" => CommitCheckpoint::DbBackedUp,
            "after-blob-move" => CommitCheckpoint::BlobsMoved,
            "after-db-write" => CommitCheckpoint::DbWritten,
            "after-state-backup" => CommitCheckpoint::StateBackedUp,
            "after-state-write" => CommitCheckpoint::StateWritten,
            value => panic!("Unknown commit checkpoint {value}"),
        };

        let manager = open_existing_manager(&test_dir);
        let mut baza = manager.open_mut().unwrap();
        baza.commit_test_action = Some((checkpoint, CommitTestAction::Abort));
        let _ = baza.commit();
        panic!("commit did not abort at {checkpoint:?}");
    }

    let expected = [
        CrashOutcome {
            checkpoint: CommitCheckpoint::DbBackedUp,
            can_open: true,
            open_panics: false,
            has_staged_documents: Some(true),
            new_asset_is_visible: Some(true),
            storage_db_exists: false,
            state_file_exists: true,
            storage_blob_exists: false,
            state_blob_exists: true,
            storage_backup_files: 1,
            state_backup_files: 0,
        },
        CrashOutcome {
            checkpoint: CommitCheckpoint::BlobsMoved,
            can_open: true,
            open_panics: false,
            has_staged_documents: Some(true),
            new_asset_is_visible: Some(true),
            storage_db_exists: false,
            state_file_exists: true,
            storage_blob_exists: true,
            state_blob_exists: false,
            storage_backup_files: 1,
            state_backup_files: 0,
        },
        CrashOutcome {
            checkpoint: CommitCheckpoint::DbWritten,
            can_open: true,
            open_panics: false,
            has_staged_documents: Some(true),
            new_asset_is_visible: Some(true),
            storage_db_exists: true,
            state_file_exists: true,
            storage_blob_exists: true,
            state_blob_exists: false,
            storage_backup_files: 1,
            state_backup_files: 0,
        },
        CrashOutcome {
            checkpoint: CommitCheckpoint::StateBackedUp,
            can_open: true,
            open_panics: false,
            has_staged_documents: Some(false),
            new_asset_is_visible: Some(true),
            storage_db_exists: true,
            state_file_exists: false,
            storage_blob_exists: true,
            state_blob_exists: false,
            storage_backup_files: 1,
            state_backup_files: 1,
        },
        CrashOutcome {
            checkpoint: CommitCheckpoint::StateWritten,
            can_open: true,
            open_panics: false,
            has_staged_documents: Some(false),
            new_asset_is_visible: Some(true),
            storage_db_exists: true,
            state_file_exists: true,
            storage_blob_exists: true,
            state_blob_exists: false,
            storage_backup_files: 1,
            state_backup_files: 1,
        },
    ];

    for (checkpoint, checkpoint_name) in [
        (CommitCheckpoint::DbBackedUp, "after-db-backup"),
        (CommitCheckpoint::BlobsMoved, "after-blob-move"),
        (CommitCheckpoint::DbWritten, "after-db-write"),
        (CommitCheckpoint::StateBackedUp, "after-state-backup"),
        (CommitCheckpoint::StateWritten, "after-state-write"),
    ] {
        let temp_dir = TempFile::new_with_details("commit_abort", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let blob_file = temp_dir.new_child("blob");
        blob_file.write_str("new blob").unwrap();
        let asset_id = {
            let mut baza = manager.open_mut().unwrap();
            baza.stage_document(new_empty_document(), &None).unwrap();
            let asset = baza.create_asset(&blob_file.path).unwrap();
            baza.save_changes().unwrap();
            asset.id
        };

        abort_commit(&temp_dir.path, checkpoint_name);

        let outcome = inspect_crash_outcome(&temp_dir.path, checkpoint, &asset_id);
        assert_eq!(
            outcome,
            *expected
                .iter()
                .find(|outcome| outcome.checkpoint == checkpoint)
                .unwrap()
        );
    }
}

#[test]
fn test_commit_recovery_removes_stale_db_backup_before_next_crash() {
    let temp_dir = TempFile::new_with_details("commit_recovery", "");
    temp_dir.mkdir().unwrap();

    let manager = BazaManager::new_for_tests(&temp_dir.path);
    let document = new_empty_document();
    {
        let mut baza = manager.open_mut().unwrap();
        baza.stage_document(document.clone(), &None).unwrap();
        baza.save_changes().unwrap();
    }
    drop(manager);

    abort_commit(&temp_dir.path, "after-db-write");

    let manager = open_existing_manager(&temp_dir.path);
    let baza = manager.open().unwrap();
    assert!(baza.has_staged_documents());
    drop(baza);
    assert_eq!(backup_files_count(&manager.paths.storage_dir), 0);
    drop(manager);

    abort_commit(&temp_dir.path, "after-db-backup");

    let manager = open_existing_manager(&temp_dir.path);
    let baza = manager.open().unwrap();
    assert!(baza.get_document(&document.id).is_some());
    assert!(baza.has_staged_documents());
    drop(baza);
    assert!(manager.paths.storage_main_db_file_exists().unwrap());
    assert_eq!(backup_files_count(&manager.paths.storage_dir), 0);
}
