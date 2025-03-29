use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
    sync::{RwLockReadGuard, RwLockWriteGuard},
};

use anyhow::{anyhow, ensure, Context, Result};

use rs_utils::{
    age::{read_and_decrypt_file, AgeKey, AgeReader, AgeWriter},
    log, LockFile, SecretString,
};

use crate::{baza2::Baza, entities::InstanceId};

use super::BazaManager;

#[derive(Default)]
pub struct BazaManagerState {
    key: Option<AgeKey>,
    baza: Option<Baza>,
}

impl BazaManagerState {
    fn must_get_baza(&self) -> &Baza {
        self.baza.as_ref().expect("Baza must be initialized")
    }

    fn must_get_mut_baza(&mut self) -> &mut Baza {
        self.baza.as_mut().expect("Baza must be initialized")
    }

    pub fn get_key(&self) -> Result<&AgeKey> {
        self.key.as_ref().context("Key is missing")
    }

    fn lock(&mut self) {
        self.key.take();
        self.baza.take();
    }

    pub fn unlock(&mut self, key: AgeKey) {
        self.key.replace(key);
        self.baza.take();
    }

    #[cfg(test)]
    pub fn clear_cached_baza(&mut self) {
        self.baza.take();
    }
}

pub struct BazaReadGuard<'g> {
    state: RwLockReadGuard<'g, BazaManagerState>,
}

impl Deref for BazaReadGuard<'_> {
    type Target = Baza;

    fn deref(&self) -> &Self::Target {
        self.state.must_get_baza()
    }
}

pub struct BazaWriteGuard<'g> {
    state: RwLockWriteGuard<'g, BazaManagerState>,
    _lock: LockFile,
}

impl Deref for BazaWriteGuard<'_> {
    type Target = Baza;

    fn deref(&self) -> &Self::Target {
        self.state.must_get_baza()
    }
}

impl DerefMut for BazaWriteGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state.must_get_mut_baza()
    }
}

impl BazaManager {
    pub fn open(&self) -> Result<BazaReadGuard<'_>> {
        self.maybe_read_state()?;

        let state = self.acquire_state_read_lock()?;

        Ok(BazaReadGuard { state })
    }

    pub fn open_mut(&self) -> Result<BazaWriteGuard<'_>> {
        self.maybe_read_state()?;

        let lock = self.wait_for_file_lock()?;
        let state = self.acquire_state_write_lock()?;

        Ok(BazaWriteGuard { _lock: lock, state })
    }

    pub(super) fn acquire_state_read_lock(&self) -> Result<RwLockReadGuard<'_, BazaManagerState>> {
        log::trace!("Acquiring state read lock");

        self.state
            .read()
            .map_err(|err| anyhow!("Failed to acquire read lock for the state: {err}"))
    }

    pub(super) fn acquire_state_write_lock(
        &self,
    ) -> Result<RwLockWriteGuard<'_, BazaManagerState>> {
        log::trace!("Acquiring state write lock");

        self.state
            .write()
            .map_err(|err| anyhow!("Failed to acquire write lock for the state: {err}"))
    }

    fn maybe_read_state(&self) -> Result<()> {
        ensure!(self.storage_exists()?, "Storage doesn't exist");

        if let Some(baza) = self.acquire_state_read_lock()?.baza.as_ref() {
            if baza.is_up_to_date_with_file()? {
                log::trace!("Baza state is up to date with file");
                return Ok(());
            } else {
                log::info!("Baza state is out of date with file, re-reading");
            }
        }

        log::info!("Opening baza {}", self.paths);

        let mut manager_state = self.acquire_state_write_lock()?;

        let _lock = self.wait_for_file_lock()?;

        let key = manager_state.get_key()?;

        self.merge_storages(key)?;

        let mut baza = if self.paths.state_file_exists()? {
            Baza::read(key.clone(), self.paths.clone(), self.schema.clone())?
        } else {
            Baza::create(
                InstanceId::generate(),
                key.clone(),
                self.paths.clone(),
                self.schema.clone(),
            )?
        };

        if !baza.has_staged_documents() {
            baza.update_state_from_storage()?;
            baza.remove_unused_storage_blobs()?;
        }

        manager_state.baza = Some(baza);

        Ok(())
    }

    pub fn unlock(&self, password: SecretString) -> Result<()> {
        log::info!("Unlocking baza using key file {}", self.paths.key_file);

        ensure!(self.key_exists()?, "Key file is missing");

        let _lock = self.wait_for_file_lock()?;
        let mut state = self.acquire_state_write_lock()?;

        let mut key_file_key = AgeKey::from_password(password)?;
        if cfg!(test) {
            key_file_key.test_mode();
        }

        let key = read_and_decrypt_file(&self.paths.key_file, key_file_key, true)?;

        let key = AgeKey::from_age_x25519_key(key.try_into()?)?;

        state.unlock(key);

        Ok(())
    }

    pub fn lock(&self) -> Result<()> {
        log::info!("Locking baza");

        let mut state = self.acquire_state_write_lock()?;
        state.lock();

        Ok(())
    }

    pub fn is_locked(&self) -> bool {
        !self.is_unlocked()
    }

    pub fn is_unlocked(&self) -> bool {
        let state = self
            .acquire_state_read_lock()
            .expect("Must acquire state read lock");

        state.key.is_some()
    }

    pub fn encrypt<W: Write>(&self, writer: W) -> Result<AgeWriter<W>> {
        let key = self.acquire_state_read_lock()?.get_key()?.clone();

        AgeWriter::new(writer, key)
    }

    pub fn decrypt<R: Read>(&self, reader: R) -> Result<AgeReader<R>> {
        let key = self.acquire_state_read_lock()?.get_key()?.clone();

        AgeReader::new(reader, key)
    }
}
