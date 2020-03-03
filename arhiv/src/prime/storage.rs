use super::state::{StateDTO, StorageState};
use crate::common::PathFinder;
use crate::entities::*;
use anyhow::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Storage {
    pub pf: PathFinder,
    pub state: StorageState,
}

impl Storage {
    pub fn open(root_path: &str) -> Result<Storage> {
        let pf = PathFinder::new(root_path.to_string());
        pf.assert_dirs_exist()?;

        let state = StorageState::new(root_path);
        state.assert_exists()?;

        // TODO lock file

        Ok(Storage { pf, state })
    }

    pub fn create(root_path: &str) -> Result<Storage> {
        let pf = PathFinder::new(root_path.to_string());
        pf.create_dirs()?; // create required dirs

        let state = StorageState::new(root_path);
        state.write(StateDTO {
            rev: 0,
            documents: HashMap::new(),
            attachments: HashMap::new(),
        })?;

        let replica = Storage { pf, state };

        println!("created arhiv storage in {}", root_path);

        Ok(replica)
    }
}

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

        let mut state = self.state.read().unwrap();
        state.rev = new_rev;
        self.state.write(state).unwrap();
    }

    fn get_document_path(&self, id: &Id, rev: Revision) -> String {
        format!("{}/{}/{}.json", self.pf.get_documents_directory(), id, rev)
    }

    fn get_attachment_path(&self, id: &Id) -> String {
        format!("{}/{}.json", self.pf.get_attachments_directory(), id)
    }

    fn get_attachment_data_path(&self, id: &Id) -> String {
        format!("{}/{}", self.pf.get_attachments_data_directory(), id)
    }
}

impl Storage {
    fn get_document(&self, id: &Id, rev: Revision) -> Result<Document> {
        fs::read_to_string(self.get_document_path(id, rev))?.parse()
    }

    pub fn get_documents(&self, min_rev: Revision) -> Vec<Document> {
        let state = self.state.read().unwrap();

        state
            .documents
            .iter()
            .filter(|(_id, &rev)| rev >= min_rev)
            .map(|(id, &rev)| self.get_document(id, rev).unwrap())
            .collect()
    }

    fn get_attachment(&self, id: &Id) -> Result<Attachment> {
        fs::read_to_string(self.get_attachment_path(id))?.parse()
    }

    pub fn get_attachments(&self, min_rev: Revision) -> Vec<Attachment> {
        let state = self.state.read().unwrap();

        state
            .attachments
            .iter()
            .filter(|(_id, &rev)| rev >= min_rev)
            .map(|(id, _)| self.get_attachment(id).unwrap())
            .collect()
    }
}

impl Storage {
    pub fn put_document(&self, document: &Document) -> Result<()> {
        let file_path = self.get_document_path(&document.id, document.rev);
        if Path::new(&file_path).exists() {
            return Err(anyhow!(
                "document {}/{} already exists",
                document.id,
                document.rev
            ));
        }

        fs::write(file_path, document.serialize())?;
        let mut state = self.state.read()?;
        state.documents.insert(document.id.clone(), document.rev);
        self.state.write(state)?;

        Ok(())
    }

    pub fn add_attachment(&self, attachment: &Attachment) -> Result<()> {
        let file_path = self.get_attachment_path(&attachment.id);
        if Path::new(&file_path).exists() {
            return Err(anyhow!("attachment {} already exists", attachment.id));
        }

        fs::write(file_path, attachment.serialize())?;
        let mut state = self.state.read()?;
        state
            .attachments
            .insert(attachment.id.clone(), attachment.rev);
        self.state.write(state)?;

        Ok(())
    }

    pub fn add_attachment_data(&self, id: &Id, src: &str, move_file: bool) -> Result<()> {
        let dst = self.get_attachment_data_path(id);
        if Path::new(&dst).exists() {
            return Err(anyhow!("attachment data {} already exists", dst));
        }

        if move_file {
            fs::rename(src, dst)?;
        } else {
            fs::copy(src, dst)?;
        }

        Ok(())
    }
}
