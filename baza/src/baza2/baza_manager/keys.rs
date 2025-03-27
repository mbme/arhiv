use anyhow::{Context, Result};

use rs_utils::{
    age::{
        encrypt_and_write, encrypt_and_write_file, read_and_decrypt, read_and_decrypt_file, AgeKey,
    },
    log, ExposeSecret, FsTransaction, SecretString,
};

use crate::baza2::BazaStorage;

use super::BazaManager;

impl BazaManager {
    pub(super) fn generate_key_file(&self, key_file_key: AgeKey) -> Result<AgeKey> {
        let key_file = AgeKey::generate_age_x25519_key();

        encrypt_and_write_file(
            &self.paths.key_file,
            key_file_key,
            key_file.serialize().expose_secret().as_bytes(),
            true,
        )?;

        log::debug!("Generated new key file {}", self.paths.key_file);

        Ok(key_file)
    }

    pub fn change_key_file_password(
        &self,
        old_password: SecretString,
        new_password: SecretString,
    ) -> Result<()> {
        log::warn!("Changing key file password {}", self.paths.key_file);

        let old_key_file_key = AgeKey::from_password(old_password)?;
        let data = read_and_decrypt_file(&self.paths.key_file, old_key_file_key, true)?;

        let new_key_file_key = AgeKey::from_password(new_password)?;

        let mut fs_tx = FsTransaction::new();
        fs_tx.move_to_backup(&self.paths.key_file)?;
        encrypt_and_write_file(
            &self.paths.key_file,
            new_key_file_key,
            data.expose_secret(),
            true,
        )?;
        fs_tx.commit()?;

        self.lock()?;

        Ok(())
    }

    pub fn export_key(
        &self,
        old_password: SecretString,
        new_password: SecretString,
    ) -> Result<Vec<u8>> {
        log::warn!("Exporting key file {}", self.paths.key_file);

        let old_key_file_key = AgeKey::from_password(old_password)?;
        let key_data = read_and_decrypt_file(&self.paths.key_file, old_key_file_key, true)?;

        let new_key_file_key = AgeKey::from_password(new_password)?;
        let encrypted_key_data =
            encrypt_and_write(Vec::new(), new_key_file_key, key_data.expose_secret(), true)?;

        Ok(encrypted_key_data)
    }

    pub fn import_key(&self, encrypted_key_data: Vec<u8>, password: SecretString) -> Result<()> {
        log::warn!("Importing key into file {}", self.paths.key_file);

        let key_file_key = AgeKey::from_password(password)?;

        let new_key_data =
            read_and_decrypt(encrypted_key_data.as_ref(), key_file_key.clone(), true)?;
        let new_key_data: SecretString = new_key_data.try_into()?;
        let new_key = AgeKey::from_age_x25519_key(new_key_data.clone())?;

        let _lock = self.wait_for_file_lock()?;

        // try open storage with key
        BazaStorage::read_file(&self.paths.storage_main_db_file, new_key.clone())
            .context("Can't decrypt storage with provided key")?;

        // write key file
        let mut fs_tx = FsTransaction::new();
        fs_tx.move_to_backup(&self.paths.key_file)?;
        encrypt_and_write_file(
            &self.paths.key_file,
            key_file_key,
            new_key_data.expose_secret().as_bytes(),
            true,
        )?;
        fs_tx.commit()?;

        self.lock()?;

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
}
