mod blobs;
mod stats;
mod validator;

use std::{
    collections::{HashMap, HashSet},
    fs::remove_file,
    io::{Read, Seek},
    time::Instant,
};

use anyhow::{Context, Result, bail, ensure};
use thiserror::Error;

use rs_utils::{
    FsTransaction, Timestamp, age::AgeKey, file_exists, get_file_name, get_file_size,
    get_media_type, log,
};

use crate::{
    BazaInfo, BazaState, BazaStorage, DocumentHead, Filter, ListPage, Locks,
    baza_paths::BazaPaths,
    baza_storage::{STORAGE_VERSION, create_container_patch},
    entities::{
        Document, DocumentKey, DocumentLock, DocumentLockKey, DocumentType, Id, InstanceId,
        LatestRevComputer, Revision,
    },
    merge_expert::MergeExpert,
    schema::{ASSET_TYPE, Asset, AssetData, DataSchema},
};

pub use blobs::write_and_encrypt_blob;
pub use stats::{BLOBSCount, DocumentsCount};
pub use validator::ValidationError;

use super::baza_storage::DocumentsIndex;

#[derive(Error, Debug)]
#[error(transparent)]
pub enum StagingError {
    Validation(#[from] ValidationError),
    Other(#[from] anyhow::Error),
}

pub struct Baza {
    state: BazaState,
    state_file_modification_time: Timestamp,
    paths: BazaPaths,
    key: AgeKey,
}

impl Baza {
    pub fn create(
        instance_id: InstanceId,
        key: AgeKey,
        paths: BazaPaths,
        schema: DataSchema,
    ) -> Result<Self> {
        log::info!(
            "Creating new state file {} for instance {instance_id}",
            paths.state_file
        );

        paths.ensure_dirs_exist()?;

        ensure!(!paths.state_file_exists()?, "State file already exists");

        let db_files = paths.list_storage_db_files()?;
        ensure!(!db_files.is_empty(), "No existing db files found");

        // Use main db file if exists, otherwise use the first in the list
        let mut db_file = &paths.storage_main_db_file;
        if !db_files.contains(db_file) {
            db_file = &db_files[0];
        }
        log::info!("Using {db_file} db file to create new state file");

        let mut storage = BazaStorage::read_file(db_file, key.clone())?;

        let info = storage.get_info()?;

        let mut state = BazaState::new(instance_id, info.clone(), schema);
        state.write(&paths, key.clone())?;

        let state_file_modification_time = paths.read_state_file_modification_time()?;

        log::info!("Created new state file {}", paths.state_file);

        Ok(Baza {
            state,
            state_file_modification_time,
            paths,
            key,
        })
    }

    pub fn read(key: AgeKey, paths: BazaPaths, schema: DataSchema) -> Result<Self> {
        paths.ensure_dirs_exist()?;

        let latest_data_version = schema.get_latest_data_version();

        let state = BazaState::read(&paths, key.clone(), schema)?;
        let state_file_modification_time = paths.read_state_file_modification_time()?;

        ensure!(
            state.get_info().data_version == latest_data_version,
            "State data version mismatch"
        );
        ensure!(
            state.get_info().storage_version == STORAGE_VERSION,
            "Storage version mismatch"
        );

        Ok(Baza {
            state,
            state_file_modification_time,
            paths,
            key,
        })
    }

    pub(crate) fn update_state_from_storage(&mut self) -> Result<()> {
        log::debug!("Updating state from storage");

        let mut storage =
            BazaStorage::read_file(&self.paths.storage_main_db_file, self.key.clone())?;

        let latest_snapshots_count = update_state_from_storage(&mut self.state, &mut storage)?;

        if latest_snapshots_count > 0 {
            log::info!("Got {latest_snapshots_count} latest snapshots from the storage");
        }

        self.save_changes()?;

        Ok(())
    }

    #[cfg(test)]
    pub fn create_storage_file(&self, file_path: &str, docs: &[Document]) {
        use rs_utils::create_file_writer;

        use crate::baza_storage::create_storage;

        let mut storage_writer = create_file_writer(file_path, false).unwrap();
        create_storage(
            &mut storage_writer,
            self.key.clone(),
            self.get_info().clone(),
            docs,
        )
        .unwrap();
    }

    pub fn get_info(&self) -> &BazaInfo {
        self.state.get_info()
    }

    pub fn get_instance_id(&self) -> &InstanceId {
        self.state.get_instance_id()
    }

    pub fn get_data_version(&self) -> u8 {
        self.state.get_info().data_version
    }

    pub fn get_single_latest_revision(&self) -> &Revision {
        self.state.get_single_latest_revision()
    }

    pub fn get_schema(&self) -> &DataSchema {
        self.state.get_schema()
    }

    pub fn get_storage_dir(&self) -> &str {
        &self.paths.storage_dir
    }

    pub fn get_state_dir(&self) -> &str {
        &self.paths.state_dir
    }

    pub fn find_last_modification_time(&self) -> Option<Timestamp> {
        self.state.find_last_modification_time()
    }

    pub fn list_document_locks(&self) -> &Locks {
        self.state.list_document_locks()
    }

    pub fn has_document_locks(&self) -> bool {
        self.state.has_document_locks()
    }

    pub fn is_document_locked(&self, id: &Id) -> bool {
        self.state.is_document_locked(id)
    }

    pub fn lock_document(&mut self, id: &Id, reason: impl Into<String>) -> Result<&DocumentLock> {
        let reason = reason.into();
        log::debug!("Locking document {id}: {reason}");
        self.state.lock_document(id, reason)
    }

    pub fn unlock_document(&mut self, id: &Id, key: &DocumentLockKey) -> Result<()> {
        log::debug!("Unlocking document {id}");

        self.state.unlock_document(id, key)
    }

    pub fn unlock_document_without_key(&mut self, id: &Id) -> Result<()> {
        log::info!("Unlocking document {id} without a key");

        self.state.unlock_document_without_key(id)
    }

    pub fn get_document(&self, id: &Id) -> Option<&DocumentHead> {
        self.state.get_document(id)
    }

    pub fn must_get_document(&self, id: &Id) -> Result<&Document> {
        self.state.must_get_document(id)
    }

    pub fn stage_document(
        &mut self,
        document: Document,
        lock_key: &Option<DocumentLockKey>,
    ) -> std::result::Result<&Document, StagingError> {
        log::debug!("Staging document {}", &document.id);

        self.validate_staged(&document)?;

        let document = self.state.stage_document(document, lock_key)?;

        Ok(document)
    }

    pub fn erase_document(&mut self, id: &Id) -> Result<()> {
        log::debug!("Erasing document {id}");

        self.state.erase_document(id)
    }

    pub fn has_staged_documents(&self) -> bool {
        self.state.has_staged_documents()
    }

    pub fn has_conflicts(&self) -> bool {
        self.state.has_conflicts()
    }

    pub fn iter_conflicts(&self) -> impl Iterator<Item = &DocumentHead> {
        self.state
            .iter_documents()
            .filter(|head| head.is_conflict())
    }

    pub fn iter_documents(&self) -> impl Iterator<Item = &DocumentHead> {
        self.state.iter_documents()
    }

    pub fn list_documents(&self, filter: &Filter) -> Result<ListPage<'_>> {
        let start_time = Instant::now();

        let result = self.state.list_documents(filter);

        let duration = start_time.elapsed();
        log::info!("Listed documents in {:?}", duration);

        result
    }

    pub fn find_document_backrefs(&self, id: &Id) -> HashSet<Id> {
        self.state.find_document_backrefs(id)
    }

    pub fn find_document_collections(&self, id: &Id) -> HashSet<Id> {
        self.state.find_document_collections(id)
    }

    pub fn update_document_collections(
        &mut self,
        document_id: &Id,
        collections: &Vec<Id>,
    ) -> Result<()> {
        log::debug!("Updating collections of document {document_id}");

        self.state
            .update_document_collections(document_id, collections)
    }

    pub fn get_asset(&self, asset_id: &Id) -> Result<Option<Asset>> {
        let head = if let Some(head) = self.get_document(asset_id) {
            head
        } else {
            return Ok(None);
        };

        let asset: Asset = head.get_single_document().clone().convert()?;

        Ok(Some(asset))
    }

    pub fn get_asset_data(&self, asset_id: &Id) -> Result<impl Read + Seek + use<>> {
        let asset = self.get_asset(asset_id)?.context("Asset not found")?;

        let blob_key = AgeKey::from_age_x25519_key(asset.data.age_x25519_key)?;

        self.get_blob(&asset.id, blob_key)
    }

    pub fn create_asset(&mut self, file_path: &str) -> Result<Asset> {
        log::info!("Creating Asset from {file_path}");

        ensure!(
            file_exists(file_path)?,
            "Asset source must exist and must be a file"
        );

        let filename = get_file_name(file_path).to_string();
        let media_type = get_media_type(file_path)?;
        let size = get_file_size(file_path)?;

        let blob_key = AgeKey::generate_age_x25519_key();

        let age_x25519_key = blob_key.serialize();

        let asset = Document::new_with_data(
            DocumentType::new(ASSET_TYPE),
            AssetData {
                filename,
                media_type,
                size,
                age_x25519_key,
            },
        );

        self.add_blob(&asset.id, file_path, blob_key)?;

        let document = asset.into_document()?;
        let document = self.stage_document(document, &None)?.clone();

        log::info!("Created asset {} from {file_path}", document.id);

        document.convert()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.state.is_modified()
    }

    pub(crate) fn is_up_to_date_with_file(&self) -> Result<bool> {
        let is_up_to_date =
            self.state_file_modification_time == self.paths.read_state_file_modification_time()?;

        Ok(is_up_to_date)
    }

    pub fn save_changes(&mut self) -> Result<()> {
        if self.state.is_modified() {
            self.state.write(&self.paths, self.key.clone())?;
            self.state_file_modification_time = self.paths.read_state_file_modification_time()?;
            log::info!("Saved state changes");
        }

        Ok(())
    }

    pub fn commit(&mut self) -> Result<HashSet<Id>> {
        if !self.has_staged_documents() {
            log::debug!("Can't commit: nothing to commit");
            return Ok(Default::default());
        }

        if self.state.has_document_locks() {
            bail!("Can't commit: some documents are locked");
        }

        self.save_changes()?;

        let mut fs_tx = FsTransaction::new();

        // backup db file
        let old_db_file = fs_tx.move_to_backup(self.paths.storage_main_db_file.clone())?;

        // open old db file
        let storage = BazaStorage::read_file(&old_db_file, self.key.clone())?;

        // update state
        self.state.commit()?;

        // collect snapshots that aren't present in the storage
        let new_snapshots = self
            .state
            .iter_documents()
            .flat_map(|head| head.iter_original_snapshots())
            .filter(|document| !storage.contains(&DocumentKey::for_document(document)))
            .collect::<Vec<_>>();
        log::info!("Commit: {} new document snapshots", new_snapshots.len());

        let committed_ids = new_snapshots
            .iter()
            .map(|doc| doc.id.clone())
            .collect::<HashSet<_>>();

        // collect new blobs that are used by new snapshots
        let new_blobs = self.collect_new_blobs(&new_snapshots)?;
        log::info!("Commit: {} new BLOBs", new_blobs.len());

        // move blobs
        for new_blob_id in new_blobs {
            let state_blob_path = self.paths.get_state_blob_path(&new_blob_id);
            let storage_blob_path = self.paths.get_storage_blob_path(&new_blob_id);

            fs_tx.move_file(state_blob_path, storage_blob_path, true)?;
        }

        // write changes to db file
        let mut patch = create_container_patch(new_snapshots.into_iter())?;
        for key in self.get_storage_keys_to_erase(&storage)? {
            patch.insert(key, None);
        }
        storage.patch_and_save_to_file(&self.paths.storage_main_db_file, patch)?;

        // backup state file
        fs_tx.move_to_backup(self.paths.state_file.clone())?;

        // write changes to state file
        self.save_changes()?;

        fs_tx.commit()?;
        log::info!("Commit: finished");

        // remove unused state BLOBs if any
        let unused_state_blobs = self.paths.list_state_blobs()?;
        if !unused_state_blobs.is_empty() {
            log::info!("Removing {} unused state BLOBs", unused_state_blobs.len());

            for blob_id in unused_state_blobs {
                let file_path = self.paths.get_state_blob_path(&blob_id);
                remove_file(file_path).context("Failed to remove unused state BLOB")?;
            }
        }

        Ok(committed_ids)
    }

    /// collect keys of storage documents that are known to be erased in the state
    fn get_storage_keys_to_erase<R: Read>(&self, storage: &BazaStorage<R>) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        for key in storage.index.iter() {
            if let Some(head) = self.state.get_document(&key.id) {
                if !head.is_original_erased() {
                    continue;
                }

                let is_old_snapshot = head
                    .iter_all_revs()
                    .all(|head_rev| key.rev.is_older_than(head_rev));

                if is_old_snapshot {
                    keys.push(key.serialize());
                }
            }
        }

        Ok(keys)
    }
}

impl Drop for Baza {
    fn drop(&mut self) {
        if self.has_unsaved_changes() {
            log::error!("Dropping Baza with unsaved changes");
        }
    }
}

fn create_index_map(index: &DocumentsIndex) -> HashMap<Id, (HashSet<Revision>, Option<Revision>)> {
    let mut document_revs_map: HashMap<&Id, HashSet<&Revision>> = HashMap::new();

    // insert all ids & revs into the map
    for key in index.iter() {
        let entry = document_revs_map.entry(&key.id).or_default();

        entry.insert(&key.rev);
    }

    let mut result = HashMap::new();

    // calculate max rev per document
    for (id, revs) in &mut document_revs_map.into_iter() {
        let mut latest_rev_computer = LatestRevComputer::new();

        latest_rev_computer.update(revs.iter().copied());

        let latest_revs = latest_rev_computer.get();

        // if conflict - also find base rev
        let base_rev = if latest_revs.len() > 1 {
            Revision::find_base_rev(&latest_revs, revs.iter().copied()).cloned()
        } else {
            None
        };

        let latest_revs = latest_revs.into_iter().cloned().collect::<HashSet<_>>();

        result.insert(id.clone(), (latest_revs, base_rev));
    }

    result
}

fn update_state_from_storage<R: Read>(
    state: &mut BazaState,
    storage: &mut BazaStorage<R>,
) -> Result<usize> {
    if state.has_staged_documents() {
        return Ok(0);
    }

    let storage_info = storage.get_info()?;
    ensure!(
        storage_info == state.get_info(),
        "state info and storage info must match"
    );

    let mut storage_index_map = create_index_map(&storage.index);

    // leave only documents that are outdated in the state
    storage_index_map.retain(|id, (max_revs, _base_rev)| {
        let is_in_state = state.get_document(id).is_some_and(|document_head| {
            let state_max_revs = document_head.get_original_revs();

            // document in the state is up to date
            state_max_revs == max_revs.iter().collect()
        });

        !is_in_state
    });

    // collect snapshot keys for outdated documents
    let mut latest_snapshot_keys =
        storage_index_map
            .iter()
            .fold(HashSet::new(), |mut acc, (id, (max_revs, base_rev))| {
                acc.extend(
                    max_revs
                        .iter()
                        .map(|rev| DocumentKey::new(id.clone(), rev.clone())),
                );

                if let Some(base_rev) = base_rev {
                    acc.insert(DocumentKey::new(id.clone(), (*base_rev).clone()));
                }

                acc
            });

    // read necessary snapshots from the storage
    let mut latest_snapshots = HashMap::new();
    while !latest_snapshot_keys.is_empty() {
        let (ref key, ref raw_document) = storage.next().context("No records in the storage")??;

        if !latest_snapshot_keys.contains(key) {
            continue;
        }

        let document: Document =
            serde_json::from_str(raw_document).context("Failed to parse raw document")?;

        latest_snapshots.insert(key.clone(), document);

        latest_snapshot_keys.remove(key);
    }

    let merge_expert = MergeExpert::new(state.get_schema().clone());
    let mut latest_snapshots_count = 0;

    for (id, (max_revs, base_rev)) in storage_index_map {
        let snapshots = max_revs.into_iter().map(|rev| {
            let key = DocumentKey::new(id.clone(), rev.clone());

            latest_snapshots.remove(&key).expect("Snapshot is missing")
        });

        let mut document_head = DocumentHead::new(snapshots)?;

        // count how many new snapshots we've read from storage
        if let Some(head) = state.get_document(&id) {
            latest_snapshots_count += document_head
                .get_original_revs()
                .difference(&head.get_original_revs())
                .count();
        } else {
            latest_snapshots_count += document_head.get_snapshots_count();
        }

        // 3-way merge conflict
        if document_head.is_conflict() {
            let base = base_rev.map(|base_rev| {
                let key = DocumentKey::new(id.clone(), base_rev.clone());

                latest_snapshots.remove(&key).expect("Snapshot is missing")
            });

            let merged = merge_expert
                .merge_originals(base, document_head.iter_original_snapshots().collect())?;

            document_head.modify(merged)?;
        }

        state.insert_document_head(document_head)?;
    }

    // collect total snapshot count per document
    let mut document_snapshot_counts: HashMap<&Id, usize> = HashMap::new();
    for key in storage.index.iter() {
        *document_snapshot_counts.entry(&key.id).or_insert(0) += 1;
    }

    // update snapshots count in the state
    for (id, snapshots_count) in document_snapshot_counts {
        state.update_snapshots_count(id, snapshots_count)?;
    }

    Ok(latest_snapshots_count)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use rs_utils::age::AgeKey;

    use crate::{
        BazaState, DocumentHead, baza_storage::create_test_storage, entities::new_document,
    };

    use super::update_state_from_storage;

    #[test]
    fn test_update_state_from_storage() {
        let key = AgeKey::generate_age_x25519_key();

        let doc_a = new_document(json!({})).with_rev(json!({ "a": 1 }));
        let doc_a1 = doc_a.clone().with_rev(json!({ "b": 1 }));
        let doc_a2 = doc_a.clone().with_rev(json!({ "b": 2 }));

        let doc_b = new_document(json!({})).with_rev(json!({ "b": 1 }));
        let doc_b1 = doc_b.clone().with_rev(json!({ "b": 2 }));

        let doc_c = new_document(json!({})).with_rev(json!({ "c": 3 }));

        let mut state = BazaState::new_test_state();
        state.insert_snapshots(vec![doc_a.clone(), doc_a1.clone(), doc_b.clone()]);

        let mut storage = create_test_storage(
            key.clone(),
            &[
                doc_a.clone(),
                doc_a1.clone(),
                doc_a2.clone(),
                doc_b.clone(),
                doc_b1.clone(),
                doc_c.clone(),
            ],
        );

        let changes = update_state_from_storage(&mut state, &mut storage).unwrap();
        assert_eq!(changes, 3);

        {
            let head = state.get_document(&doc_a.id).unwrap();
            assert_eq!(head.get_snapshots_count(), 3);

            let expected_head =
                DocumentHead::new([doc_a.clone(), doc_a2.clone()].into_iter()).unwrap();
            assert_eq!(head.get_original_revs(), expected_head.get_original_revs());

            assert!(head.is_staged());
        }

        assert_eq!(
            *state.get_document(&doc_b.id).unwrap(),
            DocumentHead::new_committed(doc_b1)
                .unwrap()
                .with_snapshots_count(2),
        );

        assert_eq!(
            *state.get_document(&doc_c.id).unwrap(),
            DocumentHead::new_committed(doc_c).unwrap()
        );
    }
}
