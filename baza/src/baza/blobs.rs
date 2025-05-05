use std::{
    collections::HashSet,
    fs::remove_file,
    io::{Read, Seek, copy},
};

use anyhow::{Context, Result, ensure};

use rs_utils::{
    age::{AgeKey, AgeReader, AgeWriter},
    create_file_reader, create_file_writer, file_exists, log,
};

use crate::{
    entities::{Document, Id},
    schema::ASSET_TYPE,
};

use super::Baza;

impl Baza {
    fn get_blob_path(&self, asset_id: &Id) -> Result<Option<String>> {
        let blob_path = self.paths.get_state_blob_path(asset_id);

        if file_exists(&blob_path)? {
            return Ok(Some(blob_path));
        }

        let blob_path = self.paths.get_storage_blob_path(asset_id);
        if file_exists(&blob_path)? {
            return Ok(Some(blob_path));
        }

        Ok(None)
    }

    pub fn blob_exists(&self, asset_id: &Id) -> Result<bool> {
        self.get_blob_path(asset_id).map(|path| path.is_some())
    }

    pub fn get_blob(&self, asset_id: &Id, blob_key: AgeKey) -> Result<impl Read + Seek + use<>> {
        let file_path = self
            .get_blob_path(asset_id)?
            .context("BLOB doesn't exist")?;

        let file_reader = create_file_reader(&file_path)?;
        let age_reader = AgeReader::new(file_reader, blob_key)?;

        Ok(age_reader)
    }

    pub fn add_blob(&mut self, asset_id: &Id, file_path: &str, blob_key: AgeKey) -> Result<()> {
        ensure!(
            !self.blob_exists(asset_id)?,
            "BLOB {asset_id} already exists"
        );

        let blob_path = self.paths.get_state_blob_path(asset_id);
        write_and_encrypt_blob(file_path, &blob_path, blob_key)?;

        Ok(())
    }

    fn remove_storage_blob(&mut self, asset_id: &Id) -> Result<()> {
        log::warn!("Removing storage BLOB {asset_id}");
        let file_path = self.paths.get_storage_blob_path(asset_id);

        remove_file(file_path)?;

        Ok(())
    }

    pub(crate) fn remove_unused_storage_blobs(&mut self) -> Result<()> {
        let committed_assets = self
            .iter_documents()
            .filter_map(|head| {
                if *head.get_type() == ASSET_TYPE {
                    return Some(head.get_id().clone());
                }

                None
            })
            .collect::<HashSet<_>>();
        let storage_blobs = self.paths.list_storage_blobs()?;

        // warn about missing storage BLOBs if any
        let missing_blobs = committed_assets
            .difference(&storage_blobs)
            .collect::<Vec<_>>();
        if !missing_blobs.is_empty() {
            log::warn!("There are {} missing BLOBs", missing_blobs.len());
            log::trace!("Missing BLOBs: {missing_blobs:?}");
        }

        // remove unused storage BLOBs if any
        let unused_storage_blobs = storage_blobs
            .difference(&committed_assets)
            .collect::<Vec<_>>();
        if !unused_storage_blobs.is_empty() {
            log::info!(
                "Removing {} unused storage BLOBs",
                unused_storage_blobs.len()
            );

            for blob_id in unused_storage_blobs {
                self.remove_storage_blob(blob_id)
                    .context("Failed to remove unused storage BLOB")?;
            }
        }

        Ok(())
    }

    pub(super) fn collect_new_blobs(&self, new_snapshots: &[&Document]) -> Result<HashSet<Id>> {
        let ids = new_snapshots.iter().filter_map(|doc| {
            if doc.document_type == ASSET_TYPE {
                Some(&doc.id)
            } else {
                None
            }
        });

        let mut new_blobs = HashSet::new();
        for id in ids {
            if !self.paths.storage_blob_exists(id)? {
                new_blobs.insert(id.clone());
            }
        }

        Ok(new_blobs)
    }
}

pub fn write_and_encrypt_blob(file_path: &str, blob_path: &str, key: AgeKey) -> Result<()> {
    let file_writer = create_file_writer(blob_path, false)?;
    let mut age_writer = AgeWriter::new(file_writer, key)?;

    let mut file_reader = create_file_reader(file_path)?;

    copy(&mut file_reader, &mut age_writer).context("Failed to copy & encrypt file data")?;

    age_writer.finish()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rs_utils::{TempFile, generate_alpanumeric_string, read_all_as_string};

    use crate::{baza_manager::BazaManager, entities::Id};

    #[test]
    fn test_blobs() {
        let temp_dir = TempFile::new_with_details("test_baza", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open_mut().unwrap();
        let key = baza.key.clone();

        let data = generate_alpanumeric_string(100);
        let blob1_file = temp_dir.new_child("blob1");
        blob1_file.write_str(&data).unwrap();

        let id = Id::new();
        baza.add_blob(&id, &blob1_file.path, key.clone()).unwrap();

        let blob1_state_path = baza.get_blob_path(&id).unwrap().unwrap();
        let encrypted_data = fs::read(&blob1_state_path).unwrap();

        assert_ne!(data.as_bytes(), encrypted_data);

        let decrypted_data = read_all_as_string(baza.get_blob(&id, key.clone()).unwrap()).unwrap();

        assert_eq!(data, decrypted_data);
    }
}
