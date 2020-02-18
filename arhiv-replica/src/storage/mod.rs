use crate::entities::*;
use anyhow::*;
use state::StorageState;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

mod state;

pub struct Storage {
    root_path: String,
}

// fn ensure_dir_exists(path: &str, create: bool) -> Result<()> {
//     match fs::metadata(path) {
//         Ok(metadata) if !metadata.is_dir() => {
//             return Err(anyhow!("path isn't a directory: {}", path));
//         }

//         Ok(_) => Ok(()),

//         Err(_) if create => {
//             fs::create_dir(path).context(format!("Failed to create directory {}", path))?;
//             Ok(())
//         }

//         Err(_) => Err(anyhow!("path doesn't exist {}", path)),
//     }
// }

fn ensure_exists(path: &str, dir: bool) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if dir && !metadata.is_dir() => {
            return Err(anyhow!("path isn't a directory: {}", path));
        }

        Ok(metadata) if !dir && !metadata.is_file() => {
            return Err(anyhow!("path isn't a file: {}", path));
        }

        Ok(_) => Ok(()),

        Err(_) => Err(anyhow!("path doesn't exist {}", path)),
    }
}

impl Storage {
    pub fn open(path: &str) -> Result<Storage> {
        let replica = Storage {
            root_path: path.to_owned(),
        };

        replica.assert_dirs_and_state()?;

        // TODO lock file

        Ok(replica)
    }

    pub fn create(path: &str, primary_url: &str) -> Result<Storage> {
        if Path::new(path).exists() {
            return Err(anyhow!("path already exists: {}", path));
        }

        let replica = Storage {
            root_path: path.to_owned(),
        };

        StorageState::new(primary_url).write(&replica.get_state_file())?;

        Ok(replica)
    }

    fn assert_dirs_and_state(&self) -> Result<()> {
        ensure_exists(&self.root_path, true)?;
        ensure_exists(&self.get_state_file(), false)?;
        ensure_exists(&self.get_documents_directory(), true)?;
        ensure_exists(&self.get_documents_local_directory(), true)?;
        ensure_exists(&self.get_attachments_directory(), true)?;
        ensure_exists(&self.get_attachments_local_directory(), true)?;
        ensure_exists(&self.get_attachments_data_directory(), true)?;

        Ok(())
    }

    fn get_state_file(&self) -> String {
        format!("{}/arhiv-state.json", self.root_path)
    }

    fn get_documents_directory(&self) -> String {
        format!("{}/documents", self.root_path)
    }

    fn get_documents_local_directory(&self) -> String {
        format!("{}/documents-local", self.root_path)
    }

    fn get_attachments_directory(&self) -> String {
        format!("{}/attachments", self.root_path)
    }

    fn get_attachments_local_directory(&self) -> String {
        format!("{}/attachments-local", self.root_path)
    }

    fn get_attachments_data_directory(&self) -> String {
        format!("{}/attachments-data", self.root_path)
    }

    fn get_document_path(&self, id: &str) -> String {
        format!("{}/{}.json", self.get_documents_directory(), id)
    }

    fn get_document_local_path(&self, id: &str) -> String {
        format!("{}/{}.json", self.get_documents_local_directory(), id)
    }

    fn get_attachment_path(&self, id: &str) -> String {
        format!("{}/{}.json", self.get_attachments_directory(), id)
    }

    fn get_attachment_local_path(&self, id: &str) -> String {
        format!("{}/{}.json", self.get_attachments_local_directory(), id)
    }

    fn get_attachment_data_path(&self, id: &str) -> String {
        format!("{}/{}", self.get_attachments_data_directory(), id)
    }
}

impl Storage {
    pub fn get_state(&self) -> StorageState {
        // TODO lazy
        StorageState::read(&self.get_state_file()).expect("must be able to read replica state file")
    }

    pub fn get_rev(&self) -> Revision {
        self.get_state().replica_rev
    }
}

impl Storage {
    fn get_items<T: FromStr>(&self, path: &str) -> Vec<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        fs::read_dir(path)
            .unwrap()
            .map(|entry| {
                fs::read_to_string(entry.unwrap().path())
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .collect()
    }

    fn get_item<T: FromStr>(&self, path: &str) -> Option<T>
    where
        <T as FromStr>::Err: std::fmt::Debug,
    {
        if !Path::new(path).exists() {
            return None;
        }

        Some(fs::read_to_string(path).unwrap().parse().unwrap())
    }

    pub fn get_documents(&self) -> Vec<Document> {
        self.get_items(&self.get_documents_directory())
    }

    pub fn get_documents_local(&self) -> Vec<Document> {
        self.get_items(&self.get_documents_local_directory())
    }

    pub fn get_attachments(&self) -> Vec<Attachment> {
        self.get_items(&self.get_attachments_directory())
    }

    pub fn get_attachments_local(&self) -> Vec<Attachment> {
        self.get_items(&self.get_attachments_local_directory())
    }

    pub fn get_document(&self, id: &str) -> Option<Document> {
        self.get_item(&self.get_document_path(id))
    }

    pub fn get_document_local(&self, id: &str) -> Option<Document> {
        self.get_item(&self.get_document_local_path(id))
    }

    pub fn get_attachment(&self, id: &str) -> Option<Attachment> {
        self.get_item(&self.get_attachment_path(id))
    }

    pub fn get_attachment_local(&self, id: &str) -> Option<Attachment> {
        self.get_item(&self.get_attachment_local_path(id))
    }
}

impl Storage {
    pub fn get_changeset(&self) -> (Changeset, HashMap<String, String>) {
        let changeset = Changeset {
            replica_rev: self.get_rev(),
            documents: self.get_documents_local(),
            attachments: self.get_attachments_local(),
        };

        let mut files = HashMap::new();

        for attachment in changeset.attachments.iter() {
            files.insert(
                attachment.id.clone(),
                self.get_attachment_local_path(&attachment.id),
            );
        }

        (changeset, files)
    }
}
