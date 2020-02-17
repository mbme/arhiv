use crate::documents::Document;
use anyhow::*;
use std::fs;
use std::path::Path;

pub struct ReplicaStorage {
    root_path: String,
}

fn ensure_dir_exists(path: &str, create: bool) -> Result<()> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_dir() => {
            return Err(anyhow!("Replica root isn't a directory: {}", path));
        }

        Ok(_) => Ok(()),

        Err(_) if create => {
            fs::create_dir(path)
                .context(format!("Failed to create replica root directory {}", path))?;
            Ok(())
        }

        Err(_) => Err(anyhow!("Replica root doesn't exist {}", path)),
    }
}

impl ReplicaStorage {
    pub fn open(path: &str) -> Result<ReplicaStorage> {
        ReplicaStorage::open_or_create(path, false)
    }

    pub fn open_or_create(path: &str, create: bool) -> Result<ReplicaStorage> {
        let replica = ReplicaStorage {
            root_path: path.to_owned(),
        };

        ensure_dir_exists(&replica.root_path, create)?;
        ensure_dir_exists(&replica.get_documents_directory(), create)?;
        ensure_dir_exists(&replica.get_attachments_directory(), create)?;

        // TODO lock file

        Ok(replica)
    }

    fn get_documents_directory(&self) -> String {
        format!("{}/documents", self.root_path)
    }

    fn get_attachments_directory(&self) -> String {
        format!("{}/attachments", self.root_path)
    }

    fn get_document_path(&self, id: &str) -> String {
        format!("{}/documents/{}", self.root_path, id)
    }

    pub fn get_documents(&self) -> Vec<Document> {
        fs::read_dir(&self.root_path)
            .expect("root dir must exist")
            .map(|entry| {
                let entry = entry.expect("must be able to read entry");
                // TODO more checks

                Document::parse(
                    &fs::read_to_string(entry.path()).expect("must be able to read file"),
                )
                .expect("must be able to parse document")
            })
            .collect()
    }

    pub fn get_document(&self, id: &str) -> Option<Document> {
        let path_str = self.get_document_path(id);
        let path = Path::new(&path_str);

        if !path.exists() {
            return None;
        }

        Some(fs::read_to_string(path).unwrap().parse().unwrap())
    }
}
