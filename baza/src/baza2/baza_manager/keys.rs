use anyhow::{Context, Result};

use rs_utils::{
    age::{
        encrypt_and_write, encrypt_and_write_file, read_and_decrypt, read_and_decrypt_file, AgeKey,
    },
    log, ExposeSecret, FsTransaction, LockFile, SecretString,
};

use crate::baza2::BazaStorage;

use super::BazaManager;

impl BazaManager {
    pub(super) fn generate_key_file(
        &self,
        key_file_key: AgeKey,
        lock_file: &LockFile,
    ) -> Result<AgeKey> {
        log::debug!("Generating new key file {}", self.paths.key_file);

        let key_file = AgeKey::generate_age_x25519_key();

        self.write_key_file(
            key_file_key,
            key_file.serialize().expose_secret().as_bytes(),
            lock_file,
        )?;

        Ok(key_file)
    }

    pub fn change_key_file_password(
        &self,
        old_password: SecretString,
        new_password: SecretString,
    ) -> Result<()> {
        log::warn!("Changing key file password {}", self.paths.key_file);

        let mut old_key_file_key = AgeKey::from_password(old_password)?;
        if cfg!(test) {
            old_key_file_key.test_mode();
        }
        let data = read_and_decrypt_file(&self.paths.key_file, old_key_file_key, true)?;

        let mut new_key_file_key = AgeKey::from_password(new_password)?;
        if cfg!(test) {
            new_key_file_key.test_mode();
        }

        let lock = self.wait_for_file_lock()?;
        self.write_key_file(new_key_file_key, data.expose_secret(), &lock)?;

        self.lock()?;

        Ok(())
    }

    pub fn export_key(
        &self,
        old_password: SecretString,
        new_password: SecretString,
    ) -> Result<Vec<u8>> {
        log::warn!("Exporting key file {}", self.paths.key_file);

        let mut old_key_file_key = AgeKey::from_password(old_password)?;
        if cfg!(test) {
            old_key_file_key.test_mode();
        }
        let key_data = read_and_decrypt_file(&self.paths.key_file, old_key_file_key, true)?;

        let mut new_key_file_key = AgeKey::from_password(new_password)?;
        if cfg!(test) {
            new_key_file_key.test_mode();
        }
        let encrypted_key_data =
            encrypt_and_write(Vec::new(), new_key_file_key, key_data.expose_secret(), true)?;

        Ok(encrypted_key_data)
    }

    pub fn import_key(&self, encrypted_key_data: Vec<u8>, password: SecretString) -> Result<()> {
        log::warn!("Importing key into file {}", self.paths.key_file);

        let mut key_file_key = AgeKey::from_password(password)?;
        if cfg!(test) {
            key_file_key.test_mode();
        }

        let new_key_data =
            read_and_decrypt(encrypted_key_data.as_ref(), key_file_key.clone(), true)?;
        let new_key_data: SecretString = new_key_data.try_into()?;
        let new_key = AgeKey::from_age_x25519_key(new_key_data.clone())?;

        let lock = self.wait_for_file_lock()?;

        self.assert_is_valid_key(new_key.clone(), &lock)?;

        self.write_key_file(key_file_key, new_key_data.expose_secret().as_bytes(), &lock)?;

        self.lock()?;

        Ok(())
    }

    pub fn verify_key(&self, encrypted_key_data: Vec<u8>, password: SecretString) -> Result<bool> {
        log::debug!("Verifying key");

        let mut key_file_key = AgeKey::from_password(password)?;
        if cfg!(test) {
            key_file_key.test_mode();
        }

        let key_data = read_and_decrypt(encrypted_key_data.as_ref(), key_file_key.clone(), true)?;
        let key_data: SecretString = key_data.try_into()?;
        let key = AgeKey::from_age_x25519_key(key_data.clone())?;

        let lock = self.wait_for_file_lock()?;
        let is_valid = self.assert_is_valid_key(key, &lock).is_ok();

        log::info!(
            "Verifying key: key is {}",
            if is_valid { "valid" } else { "invalid" }
        );

        Ok(is_valid)
    }

    fn assert_is_valid_key(&self, key: AgeKey, _lock_file: &LockFile) -> Result<()> {
        let db_path = if self.paths.storage_main_db_file_exists()? {
            self.paths.storage_main_db_file.clone()
        } else {
            let db_files = self.paths.list_storage_db_files()?;

            db_files
                .into_iter()
                .next()
                .context("No existing db files found")?
        };

        log::debug!("Using db file to check if key is valid: {db_path}");

        BazaStorage::read_file(&self.paths.storage_main_db_file, key)?;

        Ok(())
    }

    fn write_key_file(
        &self,
        key_file_key: AgeKey,
        data: &[u8],
        _lock_file: &LockFile,
    ) -> Result<()> {
        log::debug!("Writing key into key file {}", self.paths.key_file);

        let mut fs_tx = FsTransaction::new();

        if self.paths.key_file_exists()? {
            fs_tx.move_to_backup(&self.paths.key_file)?;
        }

        encrypt_and_write_file(&self.paths.key_file, key_file_key, data, true)?;

        fs_tx.commit()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rs_utils::TempFile;

    use crate::baza2::baza_manager::BazaManager;

    #[test]
    fn test_change_password() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);
        let storage_dir = manager.paths.storage_dir.clone();
        let state_dir = manager.paths.state_dir.clone();
        let downloads_dir = manager.paths.downloads_dir.clone();
        let schema = manager.schema.clone();

        manager
            .change_key_file_password("test password".into(), "new password".into())
            .unwrap();

        {
            let manager = BazaManager::new(storage_dir, state_dir, downloads_dir, schema);

            assert!(
                manager.unlock("test password".into()).is_err(),
                "Can't open with old password"
            );
            manager.unlock("new password".into()).unwrap();
        }
    }

    #[test]
    fn test_export_import_key() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);

        // Export the key with a new password
        let exported_key = manager
            .export_key("test password".into(), "export password".into())
            .unwrap();

        manager
            .import_key(exported_key, "export password".into())
            .unwrap();

        // Ensure the new manager can unlock with the exported key
        assert!(
            manager.unlock("test password".into()).is_err(),
            "Can't open with old password"
        );
        manager.unlock("export password".into()).unwrap();
    }

    #[test]
    fn test_verify_key() {
        let temp_dir = TempFile::new_with_details("baza_manager", "");
        temp_dir.mkdir().unwrap();

        let manager = BazaManager::new_for_tests(&temp_dir.path);

        // Export the key with a new password
        let exported_key = manager
            .export_key("test password".into(), "export password".into())
            .unwrap();

        // Verify the key with the correct password
        assert!(
            manager
                .verify_key(exported_key.clone(), "export password".into())
                .unwrap(),
            "Key should be valid with the correct password"
        );
    }
}
