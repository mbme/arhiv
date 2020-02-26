use crate::entities::*;
use crate::utils::ensure_exists;
use anyhow::*;
use pathfinder::PathFinder;
use state::StorageState;
use std::fs;
use std::path::Path;
use std::str::FromStr;

mod pathfinder;
mod state;

pub struct Storage {
    pf: PathFinder,
}

impl Storage {
    pub fn open(path: &str) -> Result<Storage> {
        let replica = Storage {
            pf: PathFinder::new(path),
        };

        ensure_exists(&replica.pf.root_path, true)?;
        ensure_exists(&replica.pf.get_state_file(), false)?;
        ensure_exists(&replica.pf.get_documents_directory(), true)?;
        ensure_exists(&replica.pf.get_attachments_directory(), true)?;
        ensure_exists(&replica.pf.get_attachments_data_directory(), true)?;

        // TODO lock file

        Ok(replica)
    }

    pub fn create(path_str: &str) -> Result<Storage> {
        let path = Path::new(path_str);

        if !path.is_absolute() {
            return Err(anyhow!("path must be absolute: {}", path_str));
        }

        if path.exists() {
            return Err(anyhow!("path already exists: {}", path_str));
        }

        let replica = Storage {
            pf: PathFinder::new(path_str),
        };

        // create required dirs
        fs::create_dir(&replica.pf.root_path)?;
        fs::create_dir(&replica.pf.get_documents_directory())?;
        fs::create_dir(&replica.pf.get_attachments_directory())?;
        fs::create_dir(&replica.pf.get_attachments_data_directory())?;

        // create state file
        StorageState::new().write(&replica.pf.get_state_file())?;

        println!("created arhiv replica in {}", path_str);

        Ok(replica)
    }
}

impl Storage {
    pub fn get_state(&self) -> StorageState {
        // TODO lazy
        StorageState::read(&self.pf.get_state_file())
            .expect("must be able to read replica state file")
    }

    pub fn get_rev(&self) -> Revision {
        self.get_state().replica_rev
    }

    pub fn set_rev(&self, new_rev: Revision) {
        let mut state = self.get_state();

        assert_eq!(
            new_rev > state.replica_rev,
            true,
            "new rev must be greater than current rev"
        );
        state.replica_rev = new_rev;

        state
            .write(&self.pf.get_state_file())
            .expect("must be able to write replica state file");
    }

    pub fn get_attachment_data_path(&self, id: &Id) -> String {
        self.pf.get_attachment_data_path(id)
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
        self.get_item(&self.pf.get_document_path(id))
    }

    pub fn get_attachment(&self, id: &Id) -> Option<Attachment> {
        self.get_item(&self.pf.get_attachment_path(id))
    }
}

impl Storage {
    pub fn put_document(&self, document: &Document) -> Result<()> {
        fs::write(
            self.pf.get_document_path(&document.id),
            document.serialize(),
        )?;

        Ok(())
    }

    pub fn put_attachment(&self, attachment: &Attachment) -> Result<()> {
        fs::write(
            self.pf.get_attachment_path(&attachment.id),
            attachment.serialize(),
        )?;

        Ok(())
    }

    pub fn put_attachment_data(&self, id: &Id, src: &str, move_file: bool) -> Result<()> {
        let dst = self.pf.get_attachment_data_path(id);

        if move_file {
            fs::rename(src, dst)?;
        } else {
            fs::copy(src, dst)?;
        }

        Ok(())
    }

    pub fn remove_attachment_data(&self, id: &Id) -> Result<()> {
        fs::remove_file(self.pf.get_attachment_data_path(id))?;

        Ok(())
    }
}
