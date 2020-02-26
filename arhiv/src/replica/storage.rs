use crate::entities::*;
pub use crate::storage::{StateDTO, Storage};
use anyhow::*;
use std::fs;
use std::path::Path;
use std::str::FromStr;

impl Storage {
    pub fn get_rev(&self) -> Revision {
        self.state.read().unwrap().rev
    }

    pub fn set_rev(&self, new_rev: Revision) {
        let current_rev = self.get_rev();

        assert_eq!(
            new_rev > current_rev,
            true,
            "new rev must be greater than current rev"
        );

        self.state
            .write(StateDTO { rev: new_rev })
            .expect("must be able to write replica state file");
    }

    fn get_document_path(&self, id: &Id) -> String {
        format!("{}/{}.json", self.pf.get_documents_directory(), id)
    }

    fn get_attachment_path(&self, id: &Id) -> String {
        format!("{}/{}.json", self.pf.get_attachments_directory(), id)
    }

    pub fn get_attachment_data_path(&self, id: &Id) -> String {
        format!("{}/{}", self.pf.get_attachments_data_directory(), id)
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
        self.get_items(&self.pf.get_documents_directory())
    }

    pub fn get_attachments(&self) -> Vec<Attachment> {
        self.get_items(&self.pf.get_attachments_directory())
    }

    pub fn get_document(&self, id: &Id) -> Option<Document> {
        self.get_item(&self.get_document_path(id))
    }

    pub fn get_attachment(&self, id: &Id) -> Option<Attachment> {
        self.get_item(&self.get_attachment_path(id))
    }
}

impl Storage {
    pub fn put_document(&self, document: &Document) -> Result<()> {
        fs::write(self.get_document_path(&document.id), document.serialize())?;

        Ok(())
    }

    pub fn put_attachment(&self, attachment: &Attachment) -> Result<()> {
        fs::write(
            self.get_attachment_path(&attachment.id),
            attachment.serialize(),
        )?;

        Ok(())
    }

    pub fn put_attachment_data(&self, id: &Id, src: &str, move_file: bool) -> Result<()> {
        let dst = self.get_attachment_data_path(id);

        if move_file {
            fs::rename(src, dst)?;
        } else {
            fs::copy(src, dst)?;
        }

        Ok(())
    }

    pub fn remove_attachment_data(&self, id: &Id) -> Result<()> {
        fs::remove_file(self.get_attachment_data_path(id))?;

        Ok(())
    }
}
