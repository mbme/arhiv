use std::{
    collections::HashSet,
    io::{copy, Read},
};

use anyhow::{ensure, Context, Result};

use rs_utils::{
    age::{AgeReader, AgeWriter},
    create_file_reader, create_file_writer, file_exists, log,
};

use crate::entities::BLOBId;

use super::Baza;

impl Baza {
    fn get_local_blob_path(&self, id: &BLOBId) -> String {
        self.paths.get_state_blob_path(id)
    }

    fn get_blob_path(&self, id: &BLOBId) -> Result<Option<String>> {
        let blob_path = self.get_local_blob_path(id);

        if file_exists(&blob_path)? {
            return Ok(Some(blob_path));
        }

        let blob_path = self.paths.get_storage_blob_path(id);
        if file_exists(&blob_path)? {
            return Ok(Some(blob_path));
        }

        Ok(None)
    }

    pub fn blob_exists(&self, blob_id: &BLOBId) -> Result<bool> {
        self.get_blob_path(blob_id).map(|path| path.is_some())
    }

    pub fn get_blob(&self, blob_id: &BLOBId) -> Result<impl Read> {
        let file_path = self.get_blob_path(blob_id)?.context("BLOB doesn't exist")?;

        let file_reader = create_file_reader(&file_path)?;
        let age_reader = AgeReader::new(file_reader, self.key.clone())?;

        Ok(age_reader)
    }

    pub fn list_blobs(&self) -> Result<HashSet<BLOBId>> {
        self.paths.list_blobs()
    }

    pub fn add_blob(&mut self, file_path: &str) -> Result<BLOBId> {
        ensure!(
            file_exists(file_path)?,
            "BLOB source must exist and must be a file"
        );

        let blob_id = BLOBId::from_file(file_path)?;
        if self.blob_exists(&blob_id)? {
            log::debug!("blob {blob_id} already exists");

            return Ok(blob_id);
        }

        let blob_path = self.get_local_blob_path(&blob_id);

        let file_writer = create_file_writer(&blob_path, false)?;
        let mut age_writer = AgeWriter::new(file_writer, self.key.clone())?;

        let mut file_reader = create_file_reader(file_path)?;

        copy(&mut file_reader, &mut age_writer).context("Failed to copy & encrypt file data")?;

        age_writer.finish()?;

        log::info!("Created blob {blob_id} from {file_path}");

        Ok(blob_id)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use rs_utils::{generate_alpanumeric_string, read_all_as_string, TempFile};

    use crate::baza2::baza_manager::BazaManager;

    #[test]
    fn test_blobs() {
        let temp_dir = TempFile::new_with_details("test_baza", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let mut baza = manager.open().unwrap();

        let data = generate_alpanumeric_string(100);
        let blob1_file = temp_dir.new_child("blob1");
        blob1_file.write_str(&data).unwrap();

        let blob1 = baza.add_blob(&blob1_file.path).unwrap();

        let blob1_state_path = baza.get_blob_path(&blob1).unwrap().unwrap();
        let encrypted_data = fs::read(&blob1_state_path).unwrap();

        assert_ne!(data.as_bytes(), encrypted_data);

        let decrypted_data = read_all_as_string(baza.get_blob(&blob1).unwrap()).unwrap();

        assert_eq!(data, decrypted_data);
    }
}
