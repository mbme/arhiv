use crate::entities::*;
use anyhow::*;
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub struct Storage {
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

impl Storage {
    pub fn open(path: &str) -> Result<Storage> {
        Storage::open_or_create(path, false)
    }

    pub fn open_or_create(path: &str, create: bool) -> Result<Storage> {
        let replica = Storage {
            root_path: path.to_owned(),
        };

        ensure_dir_exists(&replica.root_path, create)?;
        ensure_dir_exists(&replica.get_documents_directory(), create)?;
        ensure_dir_exists(&replica.get_documents_local_directory(), create)?;
        ensure_dir_exists(&replica.get_attachments_directory(), create)?;
        ensure_dir_exists(&replica.get_attachments_local_directory(), create)?;
        ensure_dir_exists(&replica.get_attachments_data_directory(), create)?;

        // TODO lock file

        Ok(replica)
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
