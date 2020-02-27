use crate::utils::ensure_exists;
use anyhow::*;
use std::fs;
use std::path::Path;

pub struct PathFinder {
    pub root_path: String,
}

impl PathFinder {
    pub fn new(root_path: String) -> PathFinder {
        PathFinder { root_path }
    }

    pub fn get_documents_directory(&self) -> String {
        format!("{}/documents", self.root_path)
    }

    pub fn get_attachments_directory(&self) -> String {
        format!("{}/attachments", self.root_path)
    }

    pub fn get_attachments_data_directory(&self) -> String {
        format!("{}/attachments-data", self.root_path)
    }

    pub fn assert_dirs_exist(&self) -> Result<()> {
        ensure_exists(&self.root_path, true)?;
        ensure_exists(&self.get_documents_directory(), true)?;
        ensure_exists(&self.get_attachments_directory(), true)?;
        ensure_exists(&self.get_attachments_data_directory(), true)?;

        Ok(())
    }

    pub fn create_dirs(&self) -> Result<()> {
        let path = Path::new(&self.root_path);

        if !path.is_absolute() {
            return Err(anyhow!("path must be absolute: {}", &self.root_path));
        }

        if path.exists() {
            return Err(anyhow!("path already exists: {}", &self.root_path));
        }

        fs::create_dir(&self.root_path)?;
        fs::create_dir(&self.get_documents_directory())?;
        fs::create_dir(&self.get_attachments_directory())?;
        fs::create_dir(&self.get_attachments_data_directory())?;

        Ok(())
    }
}
